use crate::config::OptimizationLevel;
use crate::optimizer::{ExprVisitor, WholeProgramPass};
use crate::MutableProgram;
use bumpalo::Bump;
use luanext_parser::ast::expression::{BinaryOp, Expression, ExpressionKind, Literal, UnaryOp};

pub struct AlgebraicSimplificationPass;

impl AlgebraicSimplificationPass {
    pub fn new() -> Self {
        Self
    }
}

impl<'arena> ExprVisitor<'arena> for AlgebraicSimplificationPass {
    fn visit_expr(&mut self, expr: &mut Expression<'arena>, arena: &'arena Bump) -> bool {
        self.simplify_expression(expr, arena)
    }
}

impl<'arena> WholeProgramPass<'arena> for AlgebraicSimplificationPass {
    fn name(&self) -> &'static str {
        "algebraic-simplification"
    }

    fn min_level(&self) -> OptimizationLevel {
        OptimizationLevel::Minimal
    }

    fn run(
        &mut self,
        program: &mut MutableProgram<'arena>,
        arena: &'arena Bump,
    ) -> Result<bool, String> {
        let mut changed = false;

        for stmt in &mut program.statements {
            changed |= self.simplify_statement(stmt, arena);
        }

        Ok(changed)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl AlgebraicSimplificationPass {
    fn simplify_statement<'arena>(
        &mut self,
        stmt: &mut luanext_parser::ast::statement::Statement<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        use luanext_parser::ast::statement::{ForStatement, Statement};

        match stmt {
            Statement::Variable(decl) => self.simplify_expression(&mut decl.initializer, arena),
            Statement::Expression(expr) => self.simplify_expression(expr, arena),
            Statement::If(if_stmt) => {
                let mut changed = self.simplify_expression(&mut if_stmt.condition, arena);
                changed |= self.simplify_block(&mut if_stmt.then_block, arena);
                let mut new_else_ifs: Vec<_> = if_stmt.else_ifs.to_vec();
                let mut eic = false;
                for else_if in &mut new_else_ifs {
                    eic |= self.simplify_expression(&mut else_if.condition, arena);
                    eic |= self.simplify_block(&mut else_if.block, arena);
                }
                if eic {
                    if_stmt.else_ifs = arena.alloc_slice_clone(&new_else_ifs);
                    changed = true;
                }
                if let Some(else_block) = &mut if_stmt.else_block {
                    changed |= self.simplify_block(else_block, arena);
                }
                changed
            }
            Statement::While(while_stmt) => {
                let mut changed = self.simplify_expression(&mut while_stmt.condition, arena);
                changed |= self.simplify_block(&mut while_stmt.body, arena);
                changed
            }
            Statement::For(for_stmt) => match &**for_stmt {
                ForStatement::Numeric(for_num_ref) => {
                    let mut new_num = (**for_num_ref).clone();
                    let mut changed = self.simplify_expression(&mut new_num.start, arena);
                    changed |= self.simplify_expression(&mut new_num.end, arena);
                    if let Some(step) = &mut new_num.step {
                        changed |= self.simplify_expression(step, arena);
                    }
                    changed |= self.simplify_block(&mut new_num.body, arena);
                    if changed {
                        *stmt = Statement::For(
                            arena.alloc(ForStatement::Numeric(arena.alloc(new_num))),
                        );
                    }
                    changed
                }
                ForStatement::Generic(for_gen_ref) => {
                    let mut new_gen = for_gen_ref.clone();
                    let mut iters: Vec<_> = new_gen.iterators.to_vec();
                    let mut changed = false;
                    for expr in &mut iters {
                        changed |= self.simplify_expression(expr, arena);
                    }
                    if changed {
                        new_gen.iterators = arena.alloc_slice_clone(&iters);
                    }
                    changed |= self.simplify_block(&mut new_gen.body, arena);
                    if changed {
                        *stmt = Statement::For(arena.alloc(ForStatement::Generic(new_gen)));
                    }
                    changed
                }
            },
            Statement::Return(ret_stmt) => {
                let mut vals: Vec<_> = ret_stmt.values.to_vec();
                let mut changed = false;
                for expr in &mut vals {
                    changed |= self.simplify_expression(expr, arena);
                }
                if changed {
                    ret_stmt.values = arena.alloc_slice_clone(&vals);
                }
                changed
            }
            _ => false,
        }
    }

    fn simplify_block<'arena>(
        &mut self,
        block: &mut luanext_parser::ast::statement::Block<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        let mut stmts: Vec<_> = block.statements.to_vec();
        let mut changed = false;
        for stmt in &mut stmts {
            changed |= self.simplify_statement(stmt, arena);
        }
        if changed {
            block.statements = arena.alloc_slice_clone(&stmts);
        }
        changed
    }

    fn simplify_expression<'arena>(
        &mut self,
        expr: &mut Expression<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        match &expr.kind {
            ExpressionKind::Binary(op, left, right) => {
                let op = *op;
                let mut new_left = (**left).clone();
                let mut new_right = (**right).clone();
                let mut changed = self.simplify_expression(&mut new_left, arena);
                changed |= self.simplify_expression(&mut new_right, arena);

                // Algebraic simplifications
                match op {
                    // x + 0 = x or 0 + x = x
                    BinaryOp::Add => {
                        if is_zero(&new_right.kind) {
                            *expr = new_left;
                            return true;
                        }
                        if is_zero(&new_left.kind) {
                            *expr = new_right;
                            return true;
                        }
                    }
                    // x - 0 = x
                    BinaryOp::Subtract => {
                        if is_zero(&new_right.kind) {
                            *expr = new_left;
                            return true;
                        }
                    }
                    // x * 0 = 0 or 0 * x = 0
                    BinaryOp::Multiply => {
                        if is_zero(&new_right.kind) || is_zero(&new_left.kind) {
                            expr.kind = ExpressionKind::Literal(Literal::Number(0.0));
                            return true;
                        }
                        // x * 1 = x or 1 * x = x
                        if is_one(&new_right.kind) {
                            *expr = new_left;
                            return true;
                        }
                        if is_one(&new_left.kind) {
                            *expr = new_right;
                            return true;
                        }
                    }
                    // x / 1 = x
                    BinaryOp::Divide => {
                        if is_one(&new_right.kind) {
                            *expr = new_left;
                            return true;
                        }
                    }
                    // true && x = x, false && x = false
                    BinaryOp::And => {
                        if let ExpressionKind::Literal(Literal::Boolean(b)) = &new_left.kind {
                            if *b {
                                *expr = new_right;
                            } else {
                                expr.kind = ExpressionKind::Literal(Literal::Boolean(false));
                            }
                            return true;
                        }
                        if let ExpressionKind::Literal(Literal::Boolean(b)) = &new_right.kind {
                            if *b {
                                *expr = new_left;
                            } else {
                                expr.kind = ExpressionKind::Literal(Literal::Boolean(false));
                            }
                            return true;
                        }
                    }
                    // true || x = true, false || x = x
                    BinaryOp::Or => {
                        if let ExpressionKind::Literal(Literal::Boolean(b)) = &new_left.kind {
                            if *b {
                                expr.kind = ExpressionKind::Literal(Literal::Boolean(true));
                            } else {
                                *expr = new_right;
                            }
                            return true;
                        }
                        if let ExpressionKind::Literal(Literal::Boolean(b)) = &new_right.kind {
                            if *b {
                                expr.kind = ExpressionKind::Literal(Literal::Boolean(true));
                            } else {
                                *expr = new_left;
                            }
                            return true;
                        }
                    }
                    _ => {}
                }

                if changed {
                    expr.kind =
                        ExpressionKind::Binary(op, arena.alloc(new_left), arena.alloc(new_right));
                }
                changed
            }
            ExpressionKind::Unary(op, operand) => {
                let op = *op;
                let mut new_operand = (**operand).clone();
                let changed = self.simplify_expression(&mut new_operand, arena);

                // !!x = x (double negation)
                if let UnaryOp::Not = op {
                    if let ExpressionKind::Unary(UnaryOp::Not, inner) = &new_operand.kind {
                        *expr = (**inner).clone();
                        return true;
                    }
                }

                if changed {
                    expr.kind = ExpressionKind::Unary(op, arena.alloc(new_operand));
                }
                changed
            }
            ExpressionKind::Call(func, args, type_args) => {
                let mut new_func = (**func).clone();
                let mut func_changed = self.simplify_expression(&mut new_func, arena);
                let mut new_args: Vec<_> = args.to_vec();
                let mut args_changed = false;
                for arg in &mut new_args {
                    args_changed |= self.simplify_expression(&mut arg.value, arena);
                }
                let type_args = *type_args;
                if func_changed || args_changed {
                    expr.kind = ExpressionKind::Call(
                        arena.alloc(new_func),
                        arena.alloc_slice_clone(&new_args),
                        type_args,
                    );
                    func_changed = true;
                }
                func_changed
            }
            ExpressionKind::Member(obj, member) => {
                let member = member.clone();
                let mut new_obj = (**obj).clone();
                let changed = self.simplify_expression(&mut new_obj, arena);
                if changed {
                    expr.kind = ExpressionKind::Member(arena.alloc(new_obj), member);
                }
                changed
            }
            _ => false,
        }
    }
}

// Helper functions
fn is_zero(expr: &ExpressionKind<'_>) -> bool {
    matches!(
        expr,
        ExpressionKind::Literal(Literal::Number(n)) if *n == 0.0
    )
}

fn is_one(expr: &ExpressionKind<'_>) -> bool {
    matches!(
        expr,
        ExpressionKind::Literal(Literal::Number(n)) if *n == 1.0
    )
}

impl Default for AlgebraicSimplificationPass {
    fn default() -> Self {
        Self::new()
    }
}
