// =============================================================================
// O3: Loop Unrolling Pass
// =============================================================================
//
// Unrolls numeric for-loops with small, constant iteration counts.
// Only unrolls loops that are safe to transform:
// - Numeric for-loops only (not generic for-loops with opaque iterators)
// - Constant bounds (start, end, step must be compile-time constants)
// - Small trip count (≤4 iterations to avoid code bloat)
// - No break/continue/return statements (unsafe to unroll)
//
// Example transformation:
//   for i = 1, 3 do
//     print(i)
//   end
// →
//   print(1)
//   print(2)
//   print(3)

use crate::config::OptimizationLevel;
use crate::optimizer::{AstFeatures, WholeProgramPass};
use crate::MutableProgram;
use bumpalo::Bump;
use luanext_parser::ast::expression::{Expression, ExpressionKind, Literal, UnaryOp};
use luanext_parser::ast::statement::{Block, ForNumeric, ForStatement, Statement};
use luanext_parser::string_interner::StringId;

/// Maximum number of iterations to unroll (conservative to avoid code bloat)
const MAX_UNROLL_COUNT: usize = 4;

pub struct LoopUnrollingPass;

impl LoopUnrollingPass {
    pub fn new() -> Self {
        Self
    }
}

impl<'arena> WholeProgramPass<'arena> for LoopUnrollingPass {
    fn name(&self) -> &'static str {
        "loop-unrolling"
    }

    fn min_level(&self) -> OptimizationLevel {
        OptimizationLevel::Aggressive
    }

    fn required_features(&self) -> AstFeatures {
        AstFeatures::HAS_LOOPS
    }

    fn run(
        &mut self,
        program: &mut MutableProgram<'arena>,
        arena: &'arena Bump,
    ) -> Result<bool, String> {
        let mut changed = false;
        let mut i = 0;

        while i < program.statements.len() {
            if let Some(unrolled) = self.try_unroll_statement(&program.statements[i], arena) {
                // Replace loop with unrolled statements
                program.statements.remove(i);
                for (j, stmt) in unrolled.into_iter().enumerate() {
                    program.statements.insert(i + j, stmt);
                }
                changed = true;
            } else {
                // Recursively process nested loops
                if self.process_statement(&mut program.statements[i], arena) {
                    changed = true;
                }
            }
            i += 1;
        }

        Ok(changed)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl LoopUnrollingPass {
    /// Try to unroll a statement if it's an unrollable loop
    fn try_unroll_statement<'arena>(
        &self,
        stmt: &Statement<'arena>,
        arena: &'arena Bump,
    ) -> Option<Vec<Statement<'arena>>> {
        match stmt {
            Statement::For(for_stmt) => match &**for_stmt {
                ForStatement::Numeric(for_num) => self.try_unroll_numeric_loop(for_num, arena),
                ForStatement::Generic(_) => None, // Generic loops are NOT safe to unroll
            },
            _ => None,
        }
    }

    /// Try to unroll a numeric for-loop
    fn try_unroll_numeric_loop<'arena>(
        &self,
        for_num: &ForNumeric<'arena>,
        arena: &'arena Bump,
    ) -> Option<Vec<Statement<'arena>>> {
        // Safety check: loop must not contain break/continue/return
        if self.contains_control_flow(&for_num.body) {
            return None;
        }

        // Evaluate loop bounds
        let (start, end, step) = self.evaluate_numeric_bounds(for_num)?;

        // Safety check: step must not be zero
        if step == 0.0 {
            return None;
        }

        // Calculate trip count
        let trip_count = self.calculate_trip_count(start, end, step)?;

        // Profitability check: only unroll small loops
        if trip_count == 0 || trip_count > MAX_UNROLL_COUNT {
            return None;
        }

        // Unroll the loop
        Some(self.generate_unrolled_statements(for_num, start, step, trip_count, arena))
    }

    /// Check if a block contains control flow statements (break/continue/return)
    fn contains_control_flow<'arena>(&self, block: &Block<'arena>) -> bool {
        for stmt in block.statements.iter() {
            if self.statement_contains_control_flow(stmt) {
                return true;
            }
        }
        false
    }

    fn statement_contains_control_flow<'arena>(&self, stmt: &Statement<'arena>) -> bool {
        match stmt {
            Statement::Break(_) | Statement::Continue(_) | Statement::Return(_) => true,
            Statement::If(if_stmt) => {
                self.contains_control_flow(&if_stmt.then_block)
                    || if_stmt
                        .else_ifs
                        .iter()
                        .any(|ei| self.contains_control_flow(&ei.block))
                    || if_stmt
                        .else_block
                        .as_ref()
                        .map(|b| self.contains_control_flow(b))
                        .unwrap_or(false)
            }
            Statement::Block(block) => self.contains_control_flow(block),
            Statement::While(while_stmt) => self.contains_control_flow(&while_stmt.body),
            Statement::Repeat(repeat_stmt) => self.contains_control_flow(&repeat_stmt.body),
            Statement::For(for_stmt) => match &**for_stmt {
                ForStatement::Numeric(for_num) => self.contains_control_flow(&for_num.body),
                ForStatement::Generic(for_gen) => self.contains_control_flow(&for_gen.body),
            },
            Statement::Try(try_stmt) => {
                self.contains_control_flow(&try_stmt.try_block)
                    || try_stmt
                        .catch_clauses
                        .iter()
                        .any(|c| self.contains_control_flow(&c.body))
                    || try_stmt
                        .finally_block
                        .as_ref()
                        .map(|f| self.contains_control_flow(f))
                        .unwrap_or(false)
            }
            _ => false,
        }
    }

    /// Evaluate constant numeric bounds (start, end, step)
    fn evaluate_numeric_bounds<'arena>(
        &self,
        for_num: &ForNumeric<'arena>,
    ) -> Option<(f64, f64, f64)> {
        let start = self.evaluate_constant_f64(&for_num.start)?;
        let end = self.evaluate_constant_f64(&for_num.end)?;
        let step = for_num
            .step
            .as_ref()
            .map(|s| self.evaluate_constant_f64(s))
            .unwrap_or(Some(1.0))?;
        Some((start, end, step))
    }

    fn evaluate_constant_f64<'arena>(&self, expr: &Expression<'arena>) -> Option<f64> {
        match &expr.kind {
            ExpressionKind::Literal(Literal::Number(n)) => Some(*n),
            ExpressionKind::Literal(Literal::Integer(n)) => Some(*n as f64),
            ExpressionKind::Unary(UnaryOp::Negate, operand) => {
                self.evaluate_constant_f64(operand).map(|n| -n)
            }
            _ => None,
        }
    }

    /// Calculate trip count for a numeric loop
    fn calculate_trip_count(&self, start: f64, end: f64, step: f64) -> Option<usize> {
        if step > 0.0 {
            if start > end {
                return Some(0);
            }
            let count = ((end - start) / step).floor() + 1.0;
            if count < 0.0 || count > MAX_UNROLL_COUNT as f64 {
                return None;
            }
            Some(count as usize)
        } else if step < 0.0 {
            if start < end {
                return Some(0);
            }
            let count = ((end - start) / step).floor() + 1.0;
            if count < 0.0 || count > MAX_UNROLL_COUNT as f64 {
                return None;
            }
            Some(count as usize)
        } else {
            None // step == 0 is infinite loop
        }
    }

    /// Generate unrolled loop statements
    fn generate_unrolled_statements<'arena>(
        &self,
        for_num: &ForNumeric<'arena>,
        start: f64,
        step: f64,
        trip_count: usize,
        arena: &'arena Bump,
    ) -> Vec<Statement<'arena>> {
        let mut result = Vec::with_capacity(for_num.body.statements.len() * trip_count);
        let loop_var = for_num.variable.node;

        for i in 0..trip_count {
            let value = start + (i as f64 * step);

            // Clone and substitute loop variable in each statement
            for stmt in for_num.body.statements.iter() {
                let substituted = self.substitute_loop_var(stmt, loop_var, value, arena);
                result.push(substituted);
            }
        }

        result
    }

    /// Substitute loop variable with constant value in a statement
    fn substitute_loop_var<'arena>(
        &self,
        stmt: &Statement<'arena>,
        loop_var: StringId,
        value: f64,
        arena: &'arena Bump,
    ) -> Statement<'arena> {
        match stmt {
            Statement::Expression(expr) => {
                Statement::Expression(self.substitute_in_expression(expr, loop_var, value, arena))
            }
            Statement::Block(block) => {
                Statement::Block(self.substitute_in_block(block, loop_var, value, arena))
            }
            // For other statement types, just clone them
            _ => stmt.clone(),
        }
    }

    /// Substitute loop variable in a block
    fn substitute_in_block<'arena>(
        &self,
        block: &Block<'arena>,
        loop_var: StringId,
        value: f64,
        arena: &'arena Bump,
    ) -> Block<'arena> {
        let new_stmts: Vec<_> = block
            .statements
            .iter()
            .map(|s| self.substitute_loop_var(s, loop_var, value, arena))
            .collect();
        Block {
            statements: arena.alloc_slice_clone(&new_stmts),
            span: block.span,
        }
    }

    /// Substitute loop variable in an expression
    fn substitute_in_expression<'arena>(
        &self,
        expr: &Expression<'arena>,
        loop_var: StringId,
        value: f64,
        arena: &'arena Bump,
    ) -> Expression<'arena> {
        match &expr.kind {
            ExpressionKind::Identifier(ident) if *ident == loop_var => {
                // Replace loop variable with constant value
                Expression {
                    kind: if value.fract() == 0.0 && value.abs() < (i64::MAX as f64) {
                        ExpressionKind::Literal(Literal::Integer(value as i64))
                    } else {
                        ExpressionKind::Literal(Literal::Number(value))
                    },
                    span: expr.span,
                    annotated_type: expr.annotated_type.clone(),
                    receiver_class: expr.receiver_class.clone(),
                }
            }
            ExpressionKind::Binary(op, left, right) => {
                let new_left = self.substitute_in_expression(left, loop_var, value, arena);
                let new_right = self.substitute_in_expression(right, loop_var, value, arena);
                Expression {
                    kind: ExpressionKind::Binary(
                        *op,
                        arena.alloc(new_left),
                        arena.alloc(new_right),
                    ),
                    span: expr.span,
                    annotated_type: expr.annotated_type.clone(),
                    receiver_class: expr.receiver_class.clone(),
                }
            }
            ExpressionKind::Unary(op, operand) => {
                let new_operand = self.substitute_in_expression(operand, loop_var, value, arena);
                Expression {
                    kind: ExpressionKind::Unary(*op, arena.alloc(new_operand)),
                    span: expr.span,
                    annotated_type: expr.annotated_type.clone(),
                    receiver_class: expr.receiver_class.clone(),
                }
            }
            ExpressionKind::Call(callee, args, type_args) => {
                let new_callee = self.substitute_in_expression(callee, loop_var, value, arena);
                let new_args: Vec<_> = args
                    .iter()
                    .map(|arg| luanext_parser::ast::expression::Argument {
                        value: self.substitute_in_expression(&arg.value, loop_var, value, arena),
                        is_spread: arg.is_spread,
                        span: arg.span,
                    })
                    .collect();
                Expression {
                    kind: ExpressionKind::Call(
                        arena.alloc(new_callee),
                        arena.alloc_slice_clone(&new_args),
                        *type_args,
                    ),
                    span: expr.span,
                    annotated_type: expr.annotated_type.clone(),
                    receiver_class: expr.receiver_class.clone(),
                }
            }
            ExpressionKind::Index(base, index) => {
                let new_base = self.substitute_in_expression(base, loop_var, value, arena);
                let new_index = self.substitute_in_expression(index, loop_var, value, arena);
                Expression {
                    kind: ExpressionKind::Index(arena.alloc(new_base), arena.alloc(new_index)),
                    span: expr.span,
                    annotated_type: expr.annotated_type.clone(),
                    receiver_class: expr.receiver_class.clone(),
                }
            }
            ExpressionKind::Member(base, field) => {
                let new_base = self.substitute_in_expression(base, loop_var, value, arena);
                Expression {
                    kind: ExpressionKind::Member(arena.alloc(new_base), field.clone()),
                    span: expr.span,
                    annotated_type: expr.annotated_type.clone(),
                    receiver_class: expr.receiver_class.clone(),
                }
            }
            // For other expression types, clone without substitution
            _ => expr.clone(),
        }
    }

    /// Recursively process statements to unroll nested loops
    fn process_statement<'arena>(
        &mut self,
        stmt: &mut Statement<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        match stmt {
            Statement::Block(block) => self.process_block(block, arena),
            Statement::Function(func) => self.process_block(&mut func.body, arena),
            _ => false,
        }
    }

    /// Process a block to unroll loops within it
    fn process_block<'arena>(&mut self, block: &mut Block<'arena>, arena: &'arena Bump) -> bool {
        let mut changed = false;
        let mut stmts = block.statements.to_vec();
        let mut i = 0;

        while i < stmts.len() {
            if let Some(unrolled) = self.try_unroll_statement(&stmts[i], arena) {
                stmts.remove(i);
                for (j, stmt) in unrolled.into_iter().enumerate() {
                    stmts.insert(i + j, stmt);
                }
                changed = true;
            } else if self.process_statement(&mut stmts[i], arena) {
                changed = true;
            }
            i += 1;
        }

        if changed {
            block.statements = arena.alloc_slice_clone(&stmts);
        }
        changed
    }
}

impl Default for LoopUnrollingPass {
    fn default() -> Self {
        Self::new()
    }
}
