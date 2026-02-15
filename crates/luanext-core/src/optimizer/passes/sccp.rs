//! Sparse Conditional Constant Propagation (SCCP) Pass
//!
//! Combines constant propagation with unreachable code detection.
//! Tracks constant values through variable assignments and evaluates
//! conditions to determine which branches are reachable.
//!
//! Unlike simple constant folding (which only folds within a single expression),
//! SCCP tracks values across statements and can resolve branch conditions
//! like `if x > 3` when `x` is known to be a constant.
//!
//! # Examples
//!
//! ```lua
//! -- Before:
//! local x = 5
//! if x > 3 then
//!     print("yes")
//! else
//!     print("no")
//! end
//!
//! -- After (condition resolved, jump threading cleans up):
//! local x = 5
//! if true then
//!     print("yes")
//! else
//!     print("no")
//! end
//! ```

use crate::optimizer::BlockVisitor;
use bumpalo::Bump;
use luanext_parser::ast::expression::{BinaryOp, Expression, ExpressionKind, Literal, UnaryOp};
use luanext_parser::ast::pattern::Pattern;
use luanext_parser::ast::statement::{Block, ForStatement, Statement};
use luanext_parser::string_interner::StringId;
use rustc_hash::FxHashMap;

/// Lattice value for SCCP analysis.
/// Represents the known state of a variable.
#[derive(Debug, Clone)]
enum LatticeValue {
    /// Variable has a known constant value.
    Constant(Literal),
    /// Variable value is unknown (could be anything).
    Top,
}

/// Sparse Conditional Constant Propagation pass.
pub struct SccpPass {
    /// Maps variable names to their lattice values.
    lattice: FxHashMap<StringId, LatticeValue>,
}

impl SccpPass {
    pub fn new() -> Self {
        Self {
            lattice: FxHashMap::default(),
        }
    }

    /// Try to evaluate an expression to a constant using the current lattice.
    fn try_evaluate(&self, expr: &Expression) -> Option<Literal> {
        match &expr.kind {
            ExpressionKind::Literal(lit) => Some(lit.clone()),
            ExpressionKind::Identifier(var) => {
                if let Some(LatticeValue::Constant(lit)) = self.lattice.get(var) {
                    Some(lit.clone())
                } else {
                    None
                }
            }
            ExpressionKind::Binary(op, left, right) => {
                let left_val = self.try_evaluate(left)?;
                let right_val = self.try_evaluate(right)?;
                self.evaluate_binary(*op, &left_val, &right_val)
            }
            ExpressionKind::Unary(op, operand) => {
                let val = self.try_evaluate(operand)?;
                self.evaluate_unary(*op, &val)
            }
            ExpressionKind::Parenthesized(inner) => self.try_evaluate(inner),
            _ => None,
        }
    }

    /// Evaluate a binary operation on two constant literals.
    fn evaluate_binary(&self, op: BinaryOp, left: &Literal, right: &Literal) -> Option<Literal> {
        match op {
            // Numeric operations
            BinaryOp::Add => self.numeric_op(left, right, |a, b| a + b),
            BinaryOp::Subtract => self.numeric_op(left, right, |a, b| a - b),
            BinaryOp::Multiply => self.numeric_op(left, right, |a, b| a * b),
            BinaryOp::Divide => {
                let r = self.to_f64(right)?;
                if r == 0.0 {
                    return None; // Avoid division by zero
                }
                self.numeric_op(left, right, |a, b| a / b)
            }
            BinaryOp::Modulo => {
                let r = self.to_f64(right)?;
                if r == 0.0 {
                    return None;
                }
                self.numeric_op(left, right, |a, b| a % b)
            }
            BinaryOp::Power => self.numeric_op(left, right, |a, b| a.powf(b)),

            // Comparison operations
            BinaryOp::LessThan => self.comparison_op(left, right, |a, b| a < b, |a, b| a < b),
            BinaryOp::LessThanOrEqual => {
                self.comparison_op(left, right, |a, b| a <= b, |a, b| a <= b)
            }
            BinaryOp::GreaterThan => self.comparison_op(left, right, |a, b| a > b, |a, b| a > b),
            BinaryOp::GreaterThanOrEqual => {
                self.comparison_op(left, right, |a, b| a >= b, |a, b| a >= b)
            }
            BinaryOp::Equal => Some(Literal::Boolean(self.literals_equal(left, right))),
            BinaryOp::NotEqual => Some(Literal::Boolean(!self.literals_equal(left, right))),

            // Logical operations
            BinaryOp::And => {
                if self.is_truthy(left) {
                    Some(right.clone())
                } else {
                    Some(left.clone())
                }
            }
            BinaryOp::Or => {
                if self.is_truthy(left) {
                    Some(left.clone())
                } else {
                    Some(right.clone())
                }
            }

            // String concatenation
            BinaryOp::Concatenate => {
                if let (Literal::String(a), Literal::String(b)) = (left, right) {
                    Some(Literal::String(format!("{}{}", a, b)))
                } else {
                    None
                }
            }

            _ => None,
        }
    }

    /// Evaluate a unary operation on a constant literal.
    fn evaluate_unary(&self, op: UnaryOp, val: &Literal) -> Option<Literal> {
        match op {
            UnaryOp::Negate => {
                let n = self.to_f64(val)?;
                Some(Literal::Number(-n))
            }
            UnaryOp::Not => Some(Literal::Boolean(!self.is_truthy(val))),
            UnaryOp::Length => {
                if let Literal::String(s) = val {
                    Some(Literal::Integer(s.len() as i64))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Helper: apply a numeric binary operation.
    fn numeric_op(
        &self,
        left: &Literal,
        right: &Literal,
        op: fn(f64, f64) -> f64,
    ) -> Option<Literal> {
        let l = self.to_f64(left)?;
        let r = self.to_f64(right)?;
        let result = op(l, r);
        if result.fract() == 0.0 && result >= i64::MIN as f64 && result <= i64::MAX as f64 {
            Some(Literal::Integer(result as i64))
        } else {
            Some(Literal::Number(result))
        }
    }

    /// Helper: apply a comparison operation.
    fn comparison_op(
        &self,
        left: &Literal,
        right: &Literal,
        num_cmp: fn(f64, f64) -> bool,
        str_cmp: fn(&str, &str) -> bool,
    ) -> Option<Literal> {
        match (left, right) {
            (Literal::String(a), Literal::String(b)) => Some(Literal::Boolean(str_cmp(a, b))),
            _ => {
                let l = self.to_f64(left)?;
                let r = self.to_f64(right)?;
                Some(Literal::Boolean(num_cmp(l, r)))
            }
        }
    }

    /// Convert a literal to f64 if it's numeric.
    fn to_f64(&self, lit: &Literal) -> Option<f64> {
        match lit {
            Literal::Number(n) => Some(*n),
            Literal::Integer(n) => Some(*n as f64),
            _ => None,
        }
    }

    /// Check if a literal is truthy (Lua semantics: nil and false are falsy).
    fn is_truthy(&self, lit: &Literal) -> bool {
        !matches!(lit, Literal::Nil | Literal::Boolean(false))
    }

    /// Check if two literals are equal.
    fn literals_equal(&self, a: &Literal, b: &Literal) -> bool {
        match (a, b) {
            (Literal::Nil, Literal::Nil) => true,
            (Literal::Boolean(a), Literal::Boolean(b)) => a == b,
            (Literal::String(a), Literal::String(b)) => a == b,
            _ => {
                // Compare numerically
                if let (Some(a), Some(b)) = (self.to_f64(a), self.to_f64(b)) {
                    a == b
                } else {
                    false
                }
            }
        }
    }

    /// Process a block of statements, propagating constants and resolving conditions.
    fn propagate_in_vec<'arena>(
        &mut self,
        stmts: &mut Vec<Statement<'arena>>,
        arena: &'arena Bump,
    ) -> bool {
        let mut changed = false;

        for stmt in stmts.iter_mut() {
            changed |= self.propagate_in_statement(stmt, arena);
        }

        changed
    }

    /// Process a single statement.
    fn propagate_in_statement<'arena>(
        &mut self,
        stmt: &mut Statement<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        match stmt {
            Statement::Variable(decl) => {
                // Try to evaluate the initializer to a constant
                if let Pattern::Identifier(ident) = &decl.pattern {
                    if let Some(lit) = self.try_evaluate(&decl.initializer) {
                        self.lattice.insert(ident.node, LatticeValue::Constant(lit));
                    } else {
                        self.lattice.insert(ident.node, LatticeValue::Top);
                    }
                }
                false
            }
            Statement::Expression(expr) => {
                // Handle assignment expressions
                if let ExpressionKind::Assignment(target, _, value) = &expr.kind {
                    if let ExpressionKind::Identifier(var) = &target.kind {
                        if let Some(lit) = self.try_evaluate(value) {
                            self.lattice.insert(*var, LatticeValue::Constant(lit));
                        } else {
                            self.lattice.insert(*var, LatticeValue::Top);
                        }
                    }
                }
                false
            }
            Statement::If(if_stmt) => {
                let mut changed = false;

                // Try to resolve the main condition
                if let Some(lit) = self.try_evaluate(&if_stmt.condition) {
                    let is_truthy = self.is_truthy(&lit);
                    if_stmt.condition = Expression {
                        kind: ExpressionKind::Literal(Literal::Boolean(is_truthy)),
                        span: if_stmt.condition.span,
                        annotated_type: if_stmt.condition.annotated_type.clone(),
                        receiver_class: if_stmt.condition.receiver_class.clone(),
                    };
                    changed = true;
                }

                // Save lattice state before branches
                let saved = self.lattice.clone();

                // Process then block
                self.propagate_in_block(&mut if_stmt.then_block, arena);

                // Process else-ifs
                let mut new_else_ifs: Vec<_> = if_stmt.else_ifs.to_vec();
                let mut eic = false;
                for else_if in &mut new_else_ifs {
                    self.lattice = saved.clone();
                    if let Some(lit) = self.try_evaluate(&else_if.condition) {
                        let is_truthy = self.is_truthy(&lit);
                        else_if.condition = Expression {
                            kind: ExpressionKind::Literal(Literal::Boolean(is_truthy)),
                            span: else_if.condition.span,
                            annotated_type: else_if.condition.annotated_type.clone(),
                            receiver_class: else_if.condition.receiver_class.clone(),
                        };
                        eic = true;
                    }
                    self.propagate_in_block(&mut else_if.block, arena);
                }
                if eic {
                    if_stmt.else_ifs = arena.alloc_slice_clone(&new_else_ifs);
                    changed = true;
                }

                // Process else block
                if let Some(else_block) = &mut if_stmt.else_block {
                    self.lattice = saved.clone();
                    self.propagate_in_block(else_block, arena);
                }

                // After branches merge, clear lattice (conservative)
                self.lattice.clear();

                changed
            }
            Statement::While(while_stmt) => {
                let mut changed = false;

                // Try to resolve the condition
                if let Some(lit) = self.try_evaluate(&while_stmt.condition) {
                    let is_truthy = self.is_truthy(&lit);
                    while_stmt.condition = Expression {
                        kind: ExpressionKind::Literal(Literal::Boolean(is_truthy)),
                        span: while_stmt.condition.span,
                        annotated_type: while_stmt.condition.annotated_type.clone(),
                        receiver_class: while_stmt.condition.receiver_class.clone(),
                    };
                    changed = true;
                }

                // Clear lattice for loop body (variables may change)
                self.lattice.clear();
                self.propagate_in_block(&mut while_stmt.body, arena);

                changed
            }
            Statement::For(for_stmt) => {
                // Clear lattice for loop body
                self.lattice.clear();
                match &**for_stmt {
                    ForStatement::Numeric(for_num_ref) => {
                        let mut new_num = (**for_num_ref).clone();
                        let fc = self.propagate_in_block(&mut new_num.body, arena);
                        if fc {
                            *stmt = Statement::For(
                                arena.alloc(ForStatement::Numeric(arena.alloc(new_num))),
                            );
                        }
                        fc
                    }
                    ForStatement::Generic(for_gen_ref) => {
                        let mut new_gen = for_gen_ref.clone();
                        let fc = self.propagate_in_block(&mut new_gen.body, arena);
                        if fc {
                            *stmt = Statement::For(arena.alloc(ForStatement::Generic(new_gen)));
                        }
                        fc
                    }
                }
            }
            Statement::Repeat(repeat_stmt) => {
                self.lattice.clear();
                let mut changed = self.propagate_in_block(&mut repeat_stmt.body, arena);

                // Try to resolve until condition
                if let Some(lit) = self.try_evaluate(&repeat_stmt.until) {
                    let is_truthy = self.is_truthy(&lit);
                    repeat_stmt.until = Expression {
                        kind: ExpressionKind::Literal(Literal::Boolean(is_truthy)),
                        span: repeat_stmt.until.span,
                        annotated_type: repeat_stmt.until.annotated_type.clone(),
                        receiver_class: repeat_stmt.until.receiver_class.clone(),
                    };
                    changed = true;
                }
                self.lattice.clear();

                changed
            }
            Statement::Function(func) => {
                // Functions have separate scope
                let saved = self.lattice.clone();
                self.lattice.clear();
                let changed = self.propagate_in_block(&mut func.body, arena);
                self.lattice = saved;
                changed
            }
            Statement::Block(block) => self.propagate_in_block(block, arena),
            _ => false,
        }
    }

    /// Process a block of statements.
    fn propagate_in_block<'arena>(
        &mut self,
        block: &mut Block<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        let mut stmts: Vec<_> = block.statements.to_vec();
        let changed = self.propagate_in_vec(&mut stmts, arena);
        if changed {
            block.statements = arena.alloc_slice_clone(&stmts);
        }
        changed
    }
}

impl Default for SccpPass {
    fn default() -> Self {
        Self::new()
    }
}

impl<'arena> BlockVisitor<'arena> for SccpPass {
    fn visit_block_stmts(
        &mut self,
        stmts: &mut Vec<Statement<'arena>>,
        arena: &'arena Bump,
    ) -> bool {
        self.lattice.clear();
        self.propagate_in_vec(stmts, arena)
    }
}
