//! Common Subexpression Elimination (CSE) Pass
//!
//! Eliminates redundant computations by detecting duplicate expressions
//! and replacing them with references to previously computed values.
//!
//! # Algorithm
//!
//! Uses value numbering with expression hashing:
//! 1. For each statement in a block, compute a hash of RHS expressions
//! 2. If the hash exists and the expression is pure (no side effects):
//!    - Replace with the previously computed variable
//! 3. Otherwise, add the expression to the value number table
//! 4. Clear value numbers at control flow merges
//!
//! # Examples
//!
//! ```lua
//! -- Before:
//! local a = b + c
//! local d = b + c  -- duplicate computation
//!
//! -- After:
//! local a = b + c
//! local d = a      -- eliminated duplicate
//! ```
//!
//! ```lua
//! -- Before:
//! local x = t.field
//! local y = t.field  -- duplicate access
//!
//! -- After:
//! local x = t.field
//! local y = x         -- eliminated duplicate
//! ```

use crate::optimizer::BlockVisitor;
use bumpalo::Bump;
use luanext_parser::ast::expression::{BinaryOp, Expression, ExpressionKind, Literal, UnaryOp};
use luanext_parser::ast::pattern::Pattern;
use luanext_parser::ast::statement::{Block, ForStatement, Statement};
use luanext_parser::string_interner::StringId;
use rustc_hash::FxHashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Common Subexpression Elimination pass.
///
/// Uses local value numbering to detect and eliminate duplicate computations.
pub struct CommonSubexpressionEliminationPass {
    /// Map from expression hash to (variable name, statement index).
    /// When we see the same expression again, we can replace it.
    value_numbers: FxHashMap<u64, (StringId, usize)>,
    /// Current statement index (for tracking freshness).
    current_stmt_index: usize,
}

impl CommonSubexpressionEliminationPass {
    pub fn new() -> Self {
        Self {
            value_numbers: FxHashMap::default(),
            current_stmt_index: 0,
        }
    }

    /// Clear value numbers (called at control flow merges).
    fn clear_value_numbers(&mut self) {
        self.value_numbers.clear();
    }

    /// Compute a hash for an expression (for value numbering).
    ///
    /// Only pure expressions are hashed - expressions with side effects
    /// return None.
    fn hash_expression(&self, expr: &Expression<'_>) -> Option<u64> {
        if !self.is_pure_expression(expr) {
            return None;
        }

        let mut hasher = DefaultHasher::new();
        self.hash_expression_recursive(expr, &mut hasher);
        Some(hasher.finish())
    }

    /// Check if an expression is pure (no side effects).
    fn is_pure_expression(&self, expr: &Expression<'_>) -> bool {
        match &expr.kind {
            // Literals and identifiers are pure
            ExpressionKind::Literal(_) | ExpressionKind::Identifier(_) => true,

            // Unary operations are pure if operand is pure
            ExpressionKind::Unary(_, operand) => self.is_pure_expression(operand),

            // Binary operations are pure if both operands are pure
            ExpressionKind::Binary(_, left, right) => {
                self.is_pure_expression(left) && self.is_pure_expression(right)
            }

            // Member access is pure (assumes no __index metamethod side effects)
            ExpressionKind::Member(obj, _) => self.is_pure_expression(obj),

            // Index access is pure if key and object are pure
            ExpressionKind::Index(obj, key) => {
                self.is_pure_expression(obj) && self.is_pure_expression(key)
            }

            // Function calls are NOT pure (may have side effects)
            ExpressionKind::Call(_, _, _) => false,

            // Method calls are NOT pure
            ExpressionKind::MethodCall(_, _, _, _) => false,

            // Functions, arrays, objects could be pure but we're conservative
            ExpressionKind::Function(_)
            | ExpressionKind::Arrow(_)
            | ExpressionKind::Object(_)
            | ExpressionKind::Array(_) => false,

            // Conditionals are pure if all branches are pure
            ExpressionKind::Conditional(cond, then_expr, else_expr) => {
                self.is_pure_expression(cond)
                    && self.is_pure_expression(then_expr)
                    && self.is_pure_expression(else_expr)
            }

            // Other expressions are conservatively considered impure
            _ => false,
        }
    }

    /// Recursively hash an expression structure.
    fn hash_expression_recursive(&self, expr: &Expression<'_>, hasher: &mut DefaultHasher) {
        match &expr.kind {
            ExpressionKind::Literal(lit) => {
                "lit".hash(hasher);
                match lit {
                    Literal::Number(n) => n.to_bits().hash(hasher),
                    Literal::Integer(i) => i.hash(hasher),
                    Literal::String(s) => s.hash(hasher),
                    Literal::Boolean(b) => b.hash(hasher),
                    Literal::Nil => "nil".hash(hasher),
                }
            }
            ExpressionKind::Identifier(id) => {
                "id".hash(hasher);
                id.hash(hasher);
            }
            ExpressionKind::Unary(op, operand) => {
                "unary".hash(hasher);
                match op {
                    UnaryOp::Negate => "neg".hash(hasher),
                    UnaryOp::Not => "not".hash(hasher),
                    UnaryOp::Length => "len".hash(hasher),
                    UnaryOp::BitwiseNot => "bnot".hash(hasher),
                }
                self.hash_expression_recursive(operand, hasher);
            }
            ExpressionKind::Binary(op, left, right) => {
                "binary".hash(hasher);
                match op {
                    BinaryOp::Add => "add".hash(hasher),
                    BinaryOp::Subtract => "sub".hash(hasher),
                    BinaryOp::Multiply => "mul".hash(hasher),
                    BinaryOp::Divide => "div".hash(hasher),
                    BinaryOp::IntegerDivide => "idiv".hash(hasher),
                    BinaryOp::Modulo => "mod".hash(hasher),
                    BinaryOp::Power => "pow".hash(hasher),
                    BinaryOp::Concatenate => "concat".hash(hasher),
                    BinaryOp::Equal => "eq".hash(hasher),
                    BinaryOp::NotEqual => "ne".hash(hasher),
                    BinaryOp::LessThan => "lt".hash(hasher),
                    BinaryOp::LessThanOrEqual => "le".hash(hasher),
                    BinaryOp::GreaterThan => "gt".hash(hasher),
                    BinaryOp::GreaterThanOrEqual => "ge".hash(hasher),
                    BinaryOp::And => "and".hash(hasher),
                    BinaryOp::Or => "or".hash(hasher),
                    BinaryOp::BitwiseAnd => "band".hash(hasher),
                    BinaryOp::BitwiseOr => "bor".hash(hasher),
                    BinaryOp::BitwiseXor => "bxor".hash(hasher),
                    BinaryOp::ShiftLeft => "shl".hash(hasher),
                    BinaryOp::ShiftRight => "shr".hash(hasher),
                    BinaryOp::NullCoalesce => "nullcoal".hash(hasher),
                    BinaryOp::Instanceof => "instanceof".hash(hasher),
                }
                self.hash_expression_recursive(left, hasher);
                self.hash_expression_recursive(right, hasher);
            }
            ExpressionKind::Member(obj, field) => {
                "member".hash(hasher);
                self.hash_expression_recursive(obj, hasher);
                field.node.hash(hasher);
            }
            ExpressionKind::Index(obj, key) => {
                "index".hash(hasher);
                self.hash_expression_recursive(obj, hasher);
                self.hash_expression_recursive(key, hasher);
            }
            ExpressionKind::Conditional(cond, then_expr, else_expr) => {
                "cond".hash(hasher);
                self.hash_expression_recursive(cond, hasher);
                self.hash_expression_recursive(then_expr, hasher);
                self.hash_expression_recursive(else_expr, hasher);
            }
            _ => {
                // Other expression types: hash a unique identifier
                // This ensures different expression types don't collide
                "other".hash(hasher);
            }
        }
    }

    /// Try to find a CSE opportunity in a variable declaration.
    ///
    /// Returns (variable_name, replacement_identifier) if a CSE was found.
    fn find_cse_opportunity(&self, expr: &Expression<'_>) -> Option<StringId> {
        let hash = self.hash_expression(expr)?;
        let (existing_var, _) = self.value_numbers.get(&hash)?;
        Some(*existing_var)
    }

    /// Record a variable declaration in the value number table.
    fn record_value_number(&mut self, var_name: StringId, expr: &Expression<'_>) {
        if let Some(hash) = self.hash_expression(expr) {
            self.value_numbers
                .insert(hash, (var_name, self.current_stmt_index));
        }
    }

    /// Process a variable declaration for CSE.
    ///
    /// If the initializer matches an existing expression, replace it with
    /// a reference to the existing variable.
    fn process_variable_decl<'arena>(
        &mut self,
        decl: &mut luanext_parser::ast::statement::VariableDeclaration<'arena>,
        _arena: &'arena Bump,
    ) -> bool {
        // Extract variable name
        let var_name = match &decl.pattern {
            Pattern::Identifier(ident) => ident.node,
            _ => return false, // Only handle simple identifiers for now
        };

        // Check for CSE opportunity
        if let Some(existing_var) = self.find_cse_opportunity(&decl.initializer) {
            // Replace with existing variable
            decl.initializer = Expression {
                kind: ExpressionKind::Identifier(existing_var),
                span: decl.initializer.span,
                annotated_type: decl.initializer.annotated_type.clone(),
                receiver_class: None,
            };
            return true;
        }

        // No CSE found, record this declaration
        self.record_value_number(var_name, &decl.initializer);
        false
    }

    /// Process statements in a block for CSE.
    fn process_block_for_cse<'arena>(
        &mut self,
        block: &mut Block<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        let mut stmts: Vec<_> = block.statements.to_vec();
        let mut changed = false;

        for (idx, stmt) in stmts.iter_mut().enumerate() {
            self.current_stmt_index = idx;
            changed |= self.process_statement_for_cse(stmt, arena);
        }

        if changed {
            block.statements = arena.alloc_slice_clone(&stmts);
        }

        changed
    }

    /// Process a single statement for CSE.
    fn process_statement_for_cse<'arena>(
        &mut self,
        stmt: &mut Statement<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        match stmt {
            Statement::Variable(decl) => self.process_variable_decl(decl, arena),

            Statement::Function(func) => {
                // Save state
                let saved_value_numbers = self.value_numbers.clone();
                let changed = self.process_block_for_cse(&mut func.body, arena);
                // Restore state (function scope is separate)
                self.value_numbers = saved_value_numbers;
                changed
            }

            Statement::Block(block) => {
                // Save state
                let saved_value_numbers = self.value_numbers.clone();
                let changed = self.process_block_for_cse(block, arena);
                // Restore state (block scope may shadow)
                self.value_numbers = saved_value_numbers;
                changed
            }

            Statement::If(if_stmt) => {
                // Process then branch
                let saved_value_numbers = self.value_numbers.clone();
                let mut changed = self.process_block_for_cse(&mut if_stmt.then_block, arena);
                self.value_numbers = saved_value_numbers.clone();

                // Process else-if branches
                let mut new_else_ifs: Vec<_> = if_stmt.else_ifs.to_vec();
                for else_if in &mut new_else_ifs {
                    self.value_numbers = saved_value_numbers.clone();
                    changed |= self.process_block_for_cse(&mut else_if.block, arena);
                }
                if changed {
                    if_stmt.else_ifs = arena.alloc_slice_clone(&new_else_ifs);
                }

                // Process else branch
                if let Some(else_block) = &mut if_stmt.else_block {
                    self.value_numbers = saved_value_numbers.clone();
                    changed |= self.process_block_for_cse(else_block, arena);
                }

                // Clear value numbers after if (values may differ across branches)
                self.clear_value_numbers();
                changed
            }

            Statement::While(while_stmt) => {
                let changed = self.process_block_for_cse(&mut while_stmt.body, arena);
                // Clear after loop (iterations may change values)
                self.clear_value_numbers();
                changed
            }

            Statement::For(for_stmt) => match &**for_stmt {
                ForStatement::Numeric(for_num) => {
                    let mut new_num = (**for_num).clone();
                    let changed = self.process_block_for_cse(&mut new_num.body, arena);
                    self.clear_value_numbers();
                    if changed {
                        *stmt = Statement::For(
                            arena.alloc(ForStatement::Numeric(arena.alloc(new_num))),
                        );
                    }
                    changed
                }
                ForStatement::Generic(for_gen) => {
                    let mut new_gen = for_gen.clone();
                    let changed = self.process_block_for_cse(&mut new_gen.body, arena);
                    self.clear_value_numbers();
                    if changed {
                        *stmt = Statement::For(arena.alloc(ForStatement::Generic(new_gen)));
                    }
                    changed
                }
            },

            Statement::Repeat(repeat_stmt) => {
                let changed = self.process_block_for_cse(&mut repeat_stmt.body, arena);
                self.clear_value_numbers();
                changed
            }

            _ => false,
        }
    }
}

impl<'arena> BlockVisitor<'arena> for CommonSubexpressionEliminationPass {
    fn visit_block_stmts(
        &mut self,
        stmts: &mut Vec<Statement<'arena>>,
        arena: &'arena Bump,
    ) -> bool {
        let mut changed = false;
        self.value_numbers.clear();
        self.current_stmt_index = 0;

        for (idx, stmt) in stmts.iter_mut().enumerate() {
            self.current_stmt_index = idx;
            changed |= self.process_statement_for_cse(stmt, arena);
        }

        changed
    }
}
