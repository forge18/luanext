//! Peephole Optimization Pass
//!
//! Applies small local pattern-matching optimizations to simplify code.
//! These are algebraic simplifications and constant-based branch elimination
//! that complement the constant folding pass.
//!
//! # Examples
//!
//! ```lua
//! -- Arithmetic identities:
//! x + 0  →  x
//! x - 0  →  x
//! x * 1  →  x
//! x * 0  →  0
//!
//! -- Boolean simplifications:
//! not (not x)  →  x
//! x and true   →  x
//! x or false   →  x
//!
//! -- Conditional constants:
//! if true then A else B   →  A
//! if false then A else B  →  B
//!
//! -- Idempotent operations:
//! x or x   →  x
//! x and x  →  x
//! ```

use crate::optimizer::ExprVisitor;
use bumpalo::Bump;
use luanext_parser::ast::expression::{BinaryOp, Expression, ExpressionKind, Literal, UnaryOp};

/// Peephole optimization pass.
///
/// Applies local pattern-based optimizations to expressions.
pub struct PeepholeOptimizationPass;

impl Default for PeepholeOptimizationPass {
    fn default() -> Self {
        Self::new()
    }
}

impl PeepholeOptimizationPass {
    pub fn new() -> Self {
        Self
    }

    /// Check if two expressions are equivalent (same identifier or literal).
    fn exprs_equivalent(left: &Expression, right: &Expression) -> bool {
        match (&left.kind, &right.kind) {
            (ExpressionKind::Identifier(a), ExpressionKind::Identifier(b)) => a == b,
            (ExpressionKind::Literal(a), ExpressionKind::Literal(b)) => match (a, b) {
                (Literal::Number(n1), Literal::Number(n2)) => (n1 - n2).abs() < f64::EPSILON,
                (Literal::String(s1), Literal::String(s2)) => s1 == s2,
                (Literal::Boolean(b1), Literal::Boolean(b2)) => b1 == b2,
                (Literal::Nil, Literal::Nil) => true,
                _ => false,
            },
            _ => false,
        }
    }

    /// Optimize a binary operation with pattern matching.
    fn optimize_binary_op<'arena>(
        &self,
        op: BinaryOp,
        left: &Expression<'arena>,
        right: &Expression<'arena>,
        _arena: &'arena Bump,
    ) -> Option<Expression<'arena>> {
        match (&left.kind, op, &right.kind) {
            // x + 0 → x
            (_, BinaryOp::Add, ExpressionKind::Literal(Literal::Number(n)))
                if n.abs() < f64::EPSILON =>
            {
                Some(left.clone())
            }
            // 0 + x → x
            (ExpressionKind::Literal(Literal::Number(n)), BinaryOp::Add, _)
                if n.abs() < f64::EPSILON =>
            {
                Some(right.clone())
            }
            // x - 0 → x
            (_, BinaryOp::Subtract, ExpressionKind::Literal(Literal::Number(n)))
                if n.abs() < f64::EPSILON =>
            {
                Some(left.clone())
            }
            // x * 1 → x
            (_, BinaryOp::Multiply, ExpressionKind::Literal(Literal::Number(n)))
                if (n - 1.0).abs() < f64::EPSILON =>
            {
                Some(left.clone())
            }
            // 1 * x → x
            (ExpressionKind::Literal(Literal::Number(n)), BinaryOp::Multiply, _)
                if (n - 1.0).abs() < f64::EPSILON =>
            {
                Some(right.clone())
            }
            // x * 0 → 0
            (_, BinaryOp::Multiply, ExpressionKind::Literal(Literal::Number(n)))
                if n.abs() < f64::EPSILON =>
            {
                Some(Expression {
                    kind: ExpressionKind::Literal(Literal::Number(0.0)),
                    span: Default::default(),
                    annotated_type: None,
                    receiver_class: None,
                })
            }
            // 0 * x → 0
            (ExpressionKind::Literal(Literal::Number(n)), BinaryOp::Multiply, _)
                if n.abs() < f64::EPSILON =>
            {
                Some(Expression {
                    kind: ExpressionKind::Literal(Literal::Number(0.0)),
                    span: Default::default(),
                    annotated_type: None,
                    receiver_class: None,
                })
            }
            // x / 1 → x
            (_, BinaryOp::Divide, ExpressionKind::Literal(Literal::Number(n)))
                if (n - 1.0).abs() < f64::EPSILON =>
            {
                Some(left.clone())
            }
            // x and true → x
            (_, BinaryOp::And, ExpressionKind::Literal(Literal::Boolean(true))) => {
                Some(left.clone())
            }
            // true and x → x
            (ExpressionKind::Literal(Literal::Boolean(true)), BinaryOp::And, _) => {
                Some(right.clone())
            }
            // x and false → false
            (_, BinaryOp::And, ExpressionKind::Literal(Literal::Boolean(false))) => {
                Some(Expression {
                    kind: ExpressionKind::Literal(Literal::Boolean(false)),
                    span: Default::default(),
                    annotated_type: None,
                    receiver_class: None,
                })
            }
            // false and x → false
            (ExpressionKind::Literal(Literal::Boolean(false)), BinaryOp::And, _) => {
                Some(Expression {
                    kind: ExpressionKind::Literal(Literal::Boolean(false)),
                    span: Default::default(),
                    annotated_type: None,
                    receiver_class: None,
                })
            }
            // x or false → x
            (_, BinaryOp::Or, ExpressionKind::Literal(Literal::Boolean(false))) => {
                Some(left.clone())
            }
            // false or x → x
            (ExpressionKind::Literal(Literal::Boolean(false)), BinaryOp::Or, _) => {
                Some(right.clone())
            }
            // x or true → true
            (_, BinaryOp::Or, ExpressionKind::Literal(Literal::Boolean(true))) => {
                Some(Expression {
                    kind: ExpressionKind::Literal(Literal::Boolean(true)),
                    span: Default::default(),
                    annotated_type: None,
                    receiver_class: None,
                })
            }
            // true or x → true
            (ExpressionKind::Literal(Literal::Boolean(true)), BinaryOp::Or, _) => {
                Some(Expression {
                    kind: ExpressionKind::Literal(Literal::Boolean(true)),
                    span: Default::default(),
                    annotated_type: None,
                    receiver_class: None,
                })
            }
            // x or x → x (idempotent)
            (_, BinaryOp::Or, _) if Self::exprs_equivalent(left, right) => Some(left.clone()),
            // x and x → x (idempotent)
            (_, BinaryOp::And, _) if Self::exprs_equivalent(left, right) => Some(left.clone()),
            // "" .. x → x (empty string concatenation)
            (ExpressionKind::Literal(Literal::String(s)), BinaryOp::Concatenate, _)
                if s.is_empty() =>
            {
                Some(right.clone())
            }
            // x .. "" → x (empty string concatenation)
            (_, BinaryOp::Concatenate, ExpressionKind::Literal(Literal::String(s)))
                if s.is_empty() =>
            {
                Some(left.clone())
            }
            _ => None,
        }
    }
}

impl<'arena> ExprVisitor<'arena> for PeepholeOptimizationPass {
    fn visit_expr(&mut self, expr: &mut Expression<'arena>, arena: &'arena Bump) -> bool {
        match &expr.kind {
            // Double negation: not (not x) → x
            ExpressionKind::Unary(UnaryOp::Not, operand) => {
                if let ExpressionKind::Unary(UnaryOp::Not, inner) = &operand.kind {
                    // Preserve type annotation if present
                    let old_type = expr.annotated_type.clone();
                    *expr = (**inner).clone();
                    if old_type.is_some() {
                        expr.annotated_type = old_type;
                    }
                    true
                } else {
                    false
                }
            }
            // Binary operation optimizations
            ExpressionKind::Binary(op, left, right) => {
                if let Some(optimized) = self.optimize_binary_op(*op, left, right, arena) {
                    // Preserve type annotation if present
                    let old_type = expr.annotated_type.clone();
                    *expr = optimized;
                    if old_type.is_some() {
                        expr.annotated_type = old_type;
                    }
                    true
                } else {
                    false
                }
            }
            // Conditional with constant condition
            ExpressionKind::Conditional(cond, then_expr, else_expr) => match &cond.kind {
                ExpressionKind::Literal(Literal::Boolean(true)) => {
                    // if true then A else B → A
                    let old_type = expr.annotated_type.clone();
                    *expr = (**then_expr).clone();
                    if old_type.is_some() {
                        expr.annotated_type = old_type;
                    }
                    true
                }
                ExpressionKind::Literal(Literal::Boolean(false)) => {
                    // if false then A else B → B
                    let old_type = expr.annotated_type.clone();
                    *expr = (**else_expr).clone();
                    if old_type.is_some() {
                        expr.annotated_type = old_type;
                    }
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }
}
