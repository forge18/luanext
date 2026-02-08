use crate::config::OptimizationLevel;
use crate::optimizer::{ExprVisitor, WholeProgramPass};
use crate::MutableProgram;
use bumpalo::Bump;
use luanext_parser::ast::expression::{BinaryOp, Expression, ExpressionKind, Literal, UnaryOp};

pub struct ConstantFoldingPass;

impl ConstantFoldingPass {
    pub fn new() -> Self {
        Self
    }
}

impl<'arena> ExprVisitor<'arena> for ConstantFoldingPass {
    fn visit_expr(&mut self, expr: &mut Expression<'arena>, arena: &'arena Bump) -> bool {
        self.fold_expression(expr, arena)
    }
}

impl<'arena> WholeProgramPass<'arena> for ConstantFoldingPass {
    fn name(&self) -> &'static str {
        "constant-folding"
    }

    fn min_level(&self) -> OptimizationLevel {
        OptimizationLevel::O1
    }

    fn run(
        &mut self,
        program: &mut MutableProgram<'arena>,
        arena: &'arena Bump,
    ) -> Result<bool, String> {
        let mut changed = false;

        for stmt in &mut program.statements {
            changed |= self.fold_statement(stmt, arena);
        }

        Ok(changed)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl ConstantFoldingPass {
    fn fold_statement<'arena>(
        &mut self,
        stmt: &mut luanext_parser::ast::statement::Statement<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        use luanext_parser::ast::statement::{ForStatement, Statement};

        match stmt {
            Statement::Variable(decl) => self.fold_expression(&mut decl.initializer, arena),
            Statement::Expression(expr) => self.fold_expression(expr, arena),
            Statement::If(if_stmt) => {
                let mut changed = self.fold_expression(&mut if_stmt.condition, arena);
                changed |= self.fold_block(&mut if_stmt.then_block, arena);
                let mut new_else_ifs: Vec<_> = if_stmt.else_ifs.to_vec();
                let mut eic = false;
                for else_if in &mut new_else_ifs {
                    eic |= self.fold_expression(&mut else_if.condition, arena);
                    eic |= self.fold_block(&mut else_if.block, arena);
                }
                if eic {
                    if_stmt.else_ifs = arena.alloc_slice_clone(&new_else_ifs);
                    changed = true;
                }
                if let Some(else_block) = &mut if_stmt.else_block {
                    changed |= self.fold_block(else_block, arena);
                }
                changed
            }
            Statement::While(while_stmt) => {
                let mut changed = self.fold_expression(&mut while_stmt.condition, arena);
                changed |= self.fold_block(&mut while_stmt.body, arena);
                changed
            }
            Statement::For(for_stmt) => match &**for_stmt {
                ForStatement::Numeric(for_num_ref) => {
                    let mut new_num = (**for_num_ref).clone();
                    let mut changed = self.fold_expression(&mut new_num.start, arena);
                    changed |= self.fold_expression(&mut new_num.end, arena);
                    if let Some(step) = &mut new_num.step {
                        changed |= self.fold_expression(step, arena);
                    }
                    changed |= self.fold_block(&mut new_num.body, arena);
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
                        changed |= self.fold_expression(expr, arena);
                    }
                    if changed {
                        new_gen.iterators = arena.alloc_slice_clone(&iters);
                    }
                    changed |= self.fold_block(&mut new_gen.body, arena);
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
                    changed |= self.fold_expression(expr, arena);
                }
                if changed {
                    ret_stmt.values = arena.alloc_slice_clone(&vals);
                }
                changed
            }
            Statement::Function(func) => self.fold_block(&mut func.body, arena),
            Statement::Class(_) => false,
            _ => false,
        }
    }

    fn fold_block<'arena>(
        &mut self,
        block: &mut luanext_parser::ast::statement::Block<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        let mut stmts: Vec<_> = block.statements.to_vec();
        let mut changed = false;
        for stmt in &mut stmts {
            changed |= self.fold_statement(stmt, arena);
        }
        if changed {
            block.statements = arena.alloc_slice_clone(&stmts);
        }
        changed
    }

    fn fold_expression<'arena>(
        &mut self,
        expr: &mut Expression<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        match &expr.kind {
            ExpressionKind::Binary(op, left, right) => {
                let op = *op;
                let mut new_left = (**left).clone();
                let mut new_right = (**right).clone();
                let left_changed = self.fold_expression(&mut new_left, arena);
                let right_changed = self.fold_expression(&mut new_right, arena);

                // Try to fold if both operands are literals
                if let (
                    ExpressionKind::Literal(Literal::Number(l)),
                    ExpressionKind::Literal(Literal::Number(r)),
                ) = (&new_left.kind, &new_right.kind)
                {
                    if let Some(result) = self.fold_numeric_binary_op(op, *l, *r) {
                        expr.kind = ExpressionKind::Literal(Literal::Number(result));
                        return true;
                    }
                }

                // Try to fold boolean operations
                if let (
                    ExpressionKind::Literal(Literal::Boolean(l)),
                    ExpressionKind::Literal(Literal::Boolean(r)),
                ) = (&new_left.kind, &new_right.kind)
                {
                    if let Some(result) = self.fold_boolean_binary_op(op, *l, *r) {
                        expr.kind = ExpressionKind::Literal(Literal::Boolean(result));
                        return true;
                    }
                }

                if left_changed || right_changed {
                    expr.kind =
                        ExpressionKind::Binary(op, arena.alloc(new_left), arena.alloc(new_right));
                }
                left_changed || right_changed
            }
            ExpressionKind::Unary(op, operand) => {
                let op = *op;
                let mut new_operand = (**operand).clone();
                let changed = self.fold_expression(&mut new_operand, arena);

                // Try to fold unary operations
                match (&new_operand.kind, op) {
                    (ExpressionKind::Literal(Literal::Number(n)), UnaryOp::Negate) => {
                        expr.kind = ExpressionKind::Literal(Literal::Number(-n));
                        return true;
                    }
                    (ExpressionKind::Literal(Literal::Boolean(b)), UnaryOp::Not) => {
                        expr.kind = ExpressionKind::Literal(Literal::Boolean(!b));
                        return true;
                    }
                    _ => {}
                }

                if changed {
                    expr.kind = ExpressionKind::Unary(op, arena.alloc(new_operand));
                }
                changed
            }
            ExpressionKind::Call(func, args, type_args) => {
                let mut new_func = (**func).clone();
                let mut func_changed = self.fold_expression(&mut new_func, arena);
                let mut new_args: Vec<_> = args.to_vec();
                let mut args_changed = false;
                for arg in &mut new_args {
                    args_changed |= self.fold_expression(&mut arg.value, arena);
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
            ExpressionKind::Index(obj, index) => {
                let mut new_obj = (**obj).clone();
                let mut new_index = (**index).clone();
                let obj_changed = self.fold_expression(&mut new_obj, arena);
                let index_changed = self.fold_expression(&mut new_index, arena);
                if obj_changed || index_changed {
                    expr.kind = ExpressionKind::Index(arena.alloc(new_obj), arena.alloc(new_index));
                }
                obj_changed || index_changed
            }
            ExpressionKind::Member(obj, member) => {
                let member = member.clone();
                let mut new_obj = (**obj).clone();
                let changed = self.fold_expression(&mut new_obj, arena);
                if changed {
                    expr.kind = ExpressionKind::Member(arena.alloc(new_obj), member);
                }
                changed
            }
            ExpressionKind::Object(fields) => {
                use luanext_parser::ast::expression::ObjectProperty;
                let mut new_fields: Vec<_> = fields.to_vec();
                let mut changed = false;
                for field in &mut new_fields {
                    match field {
                        ObjectProperty::Property { key, value, span } => {
                            let mut new_val = (**value).clone();
                            if self.fold_expression(&mut new_val, arena) {
                                *field = ObjectProperty::Property {
                                    key: key.clone(),
                                    value: arena.alloc(new_val),
                                    span: *span,
                                };
                                changed = true;
                            }
                        }
                        ObjectProperty::Computed { key, value, span } => {
                            let mut new_key = (**key).clone();
                            let mut new_val = (**value).clone();
                            let kc = self.fold_expression(&mut new_key, arena);
                            let vc = self.fold_expression(&mut new_val, arena);
                            if kc || vc {
                                *field = ObjectProperty::Computed {
                                    key: arena.alloc(new_key),
                                    value: arena.alloc(new_val),
                                    span: *span,
                                };
                                changed = true;
                            }
                        }
                        ObjectProperty::Spread { value, span } => {
                            let mut new_val = (**value).clone();
                            if self.fold_expression(&mut new_val, arena) {
                                *field = ObjectProperty::Spread {
                                    value: arena.alloc(new_val),
                                    span: *span,
                                };
                                changed = true;
                            }
                        }
                    }
                }
                if changed {
                    expr.kind = ExpressionKind::Object(arena.alloc_slice_clone(&new_fields));
                }
                changed
            }
            _ => false,
        }
    }

    fn fold_numeric_binary_op(&self, op: BinaryOp, left: f64, right: f64) -> Option<f64> {
        let l = left;
        let r = right;

        match op {
            BinaryOp::Add => Some(l + r),
            BinaryOp::Subtract => Some(l - r),
            BinaryOp::Multiply => Some(l * r),
            BinaryOp::Divide => {
                if r != 0.0 {
                    Some(l / r)
                } else {
                    None // Don't fold division by zero
                }
            }
            BinaryOp::Modulo => {
                if r != 0.0 {
                    Some(l % r)
                } else {
                    None
                }
            }
            BinaryOp::Power => Some(l.powf(r)),
            _ => None,
        }
    }

    fn fold_boolean_binary_op(&self, op: BinaryOp, left: bool, right: bool) -> Option<bool> {
        match op {
            BinaryOp::And => Some(left && right),
            BinaryOp::Or => Some(left || right),
            BinaryOp::Equal => Some(left == right),
            BinaryOp::NotEqual => Some(left != right),
            _ => None,
        }
    }
}

impl Default for ConstantFoldingPass {
    fn default() -> Self {
        Self::new()
    }
}
