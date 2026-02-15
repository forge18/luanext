//! Copy Propagation Optimization Pass
//!
//! Replaces variable uses with their constant or simple values when safe.
//! This is a dataflow optimization that simplifies code by
//! eliminating redundant variable reads.
//!
//! # Examples
//!
//! ```lua
//! -- Before:
//! local x = 5
//! local y = x + 1
//! print(y)
//!
//! -- After:
//! local x = 5
//! local y = 5 + 1  -- x replaced with 5
//! print(y)
//! ```
//!
//! ```lua
//! -- Before:
//! local x = y
//! local z = x
//!
//! -- After:
//! local x = y
//! local z = y  -- x replaced with y
//! ```
//!
//! # Safety Constraints
//!
//! Copy propagation is blocked by:
//! - Aliasing (table mutations may invalidate copied values)
//! - Control flow merges (phi functions with multiple incoming values)
//! - Side effects (function calls may change state)

use crate::optimizer::BlockVisitor;
use bumpalo::Bump;
use luanext_parser::ast::expression::{Expression, ExpressionKind, Literal};
use luanext_parser::ast::pattern::Pattern;
use luanext_parser::ast::statement::{Block, ForStatement, Statement};
use luanext_parser::ast::Spanned;
use luanext_parser::string_interner::StringId;
use rustc_hash::FxHashMap;

/// Represents a value that can be safely propagated to replace a variable use.
#[derive(Debug, Clone)]
enum PropagationValue {
    /// A compile-time constant (literal).
    Constant(Literal),
    /// Another variable (variable-to-variable copy).
    Variable(StringId),
    /// A simple member access (e.g., `x.field`) that doesn't escape.
    /// Stored as (base_variable, field_name).
    SimpleMember(StringId, StringId),
}

/// Copy propagation optimization pass.
///
/// This pass implements copy propagation to eliminate redundant
/// variable reads by replacing them with their defining values when safe.
pub struct CopyPropagationPass {
    /// Map from variable names to their propagatable values.
    /// This is rebuilt for each block to handle control flow.
    copy_values: FxHashMap<StringId, PropagationValue>,
}

impl CopyPropagationPass {
    pub fn new() -> Self {
        Self {
            copy_values: FxHashMap::default(),
        }
    }

    /// Analyze a variable declaration and track propagatable values.
    fn analyze_variable_decl(&mut self, name: StringId, init: &Expression) {
        match &init.kind {
            // Constant literals can be propagated
            ExpressionKind::Literal(lit) => {
                self.copy_values
                    .insert(name, PropagationValue::Constant(lit.clone()));
            }
            // Variable-to-variable copies
            ExpressionKind::Identifier(source_var) => {
                // Check if source is already a constant
                if let Some(PropagationValue::Constant(lit)) = self.copy_values.get(source_var) {
                    self.copy_values
                        .insert(name, PropagationValue::Constant(lit.clone()));
                } else {
                    self.copy_values
                        .insert(name, PropagationValue::Variable(*source_var));
                }
            }
            // Simple member access (non-escaping)
            ExpressionKind::Member(base, field) => {
                if let ExpressionKind::Identifier(base_var) = &base.kind {
                    self.copy_values
                        .insert(name, PropagationValue::SimpleMember(*base_var, field.node));
                }
            }
            // Any other expression is not propagatable
            _ => {
                // Conservatively invalidate any existing propagation for this variable
                self.copy_values.remove(&name);
            }
        }
    }

    /// Invalidate propagation values that may be affected by aliasing.
    ///
    /// When a table field or variable is mutated, we conservatively
    /// invalidate any propagated values that might be affected.
    fn invalidate_on_mutation(&mut self, mutated_var: StringId) {
        // Remove direct mappings to the mutated variable
        self.copy_values.remove(&mutated_var);

        // Remove any SimpleMember propagations that reference this variable
        self.copy_values.retain(|_, value| match value {
            PropagationValue::SimpleMember(base, _) => *base != mutated_var,
            PropagationValue::Variable(var) => *var != mutated_var,
            PropagationValue::Constant(_) => true,
        });
    }

    /// Try to propagate a variable use to its defining value.
    fn try_propagate<'arena>(
        &self,
        var: StringId,
        arena: &'arena Bump,
    ) -> Option<Expression<'arena>> {
        let value = self.copy_values.get(&var)?;

        match value {
            PropagationValue::Constant(lit) => Some(Expression {
                kind: ExpressionKind::Literal(lit.clone()),
                span: Default::default(),
                annotated_type: None,
                receiver_class: None,
            }),
            PropagationValue::Variable(source_var) => Some(Expression {
                kind: ExpressionKind::Identifier(*source_var),
                span: Default::default(),
                annotated_type: None,
                receiver_class: None,
            }),
            PropagationValue::SimpleMember(base_var, field_name) => {
                let base_expr = Expression {
                    kind: ExpressionKind::Identifier(*base_var),
                    span: Default::default(),
                    annotated_type: None,
                    receiver_class: None,
                };
                Some(Expression {
                    kind: ExpressionKind::Member(
                        arena.alloc(base_expr),
                        Spanned::new(*field_name, Default::default()),
                    ),
                    span: Default::default(),
                    annotated_type: None,
                    receiver_class: None,
                })
            }
        }
    }

    /// Propagate copies in an expression, returning true if changed.
    fn propagate_in_expr<'arena>(
        &self,
        expr: &mut Expression<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        let mut changed = false;

        match &mut expr.kind {
            ExpressionKind::Identifier(var) => {
                if let Some(new_expr) = self.try_propagate(*var, arena) {
                    // Preserve type annotation if present
                    let old_type = expr.annotated_type.clone();
                    *expr = new_expr;
                    if old_type.is_some() {
                        expr.annotated_type = old_type;
                    }
                    changed = true;
                }
            }
            ExpressionKind::Binary(_, left, right) => {
                let mut new_left = (**left).clone();
                let mut new_right = (**right).clone();
                let left_changed = self.propagate_in_expr(&mut new_left, arena);
                let right_changed = self.propagate_in_expr(&mut new_right, arena);
                if left_changed || right_changed {
                    if let ExpressionKind::Binary(op, _, _) = &expr.kind {
                        expr.kind = ExpressionKind::Binary(
                            *op,
                            arena.alloc(new_left),
                            arena.alloc(new_right),
                        );
                        changed = true;
                    }
                }
            }
            ExpressionKind::Unary(op, operand) => {
                let mut new_operand = (**operand).clone();
                if self.propagate_in_expr(&mut new_operand, arena) {
                    expr.kind = ExpressionKind::Unary(*op, arena.alloc(new_operand));
                    changed = true;
                }
            }
            ExpressionKind::Member(base, field) => {
                let mut new_base = (**base).clone();
                if self.propagate_in_expr(&mut new_base, arena) {
                    expr.kind = ExpressionKind::Member(arena.alloc(new_base), field.clone());
                    changed = true;
                }
            }
            ExpressionKind::Index(base, index) => {
                let mut new_base = (**base).clone();
                let mut new_index = (**index).clone();
                let base_changed = self.propagate_in_expr(&mut new_base, arena);
                let index_changed = self.propagate_in_expr(&mut new_index, arena);
                if base_changed || index_changed {
                    expr.kind =
                        ExpressionKind::Index(arena.alloc(new_base), arena.alloc(new_index));
                    changed = true;
                }
            }
            ExpressionKind::Call(func, args, type_args) => {
                let mut new_func = (**func).clone();
                changed |= self.propagate_in_expr(&mut new_func, arena);

                let mut new_args: Vec<_> = args.to_vec();
                for arg in &mut new_args {
                    changed |= self.propagate_in_expr(&mut arg.value, arena);
                }

                if changed {
                    expr.kind = ExpressionKind::Call(
                        arena.alloc(new_func),
                        arena.alloc_slice_clone(&new_args),
                        *type_args,
                    );
                }
            }
            ExpressionKind::Array(elements) => {
                let mut new_elements: Vec<_> = elements.to_vec();
                for elem in &mut new_elements {
                    match elem {
                        luanext_parser::ast::expression::ArrayElement::Expression(e) => {
                            changed |= self.propagate_in_expr(e, arena);
                        }
                        luanext_parser::ast::expression::ArrayElement::Spread(e) => {
                            changed |= self.propagate_in_expr(e, arena);
                        }
                    }
                }
                if changed {
                    expr.kind = ExpressionKind::Array(arena.alloc_slice_clone(&new_elements));
                }
            }
            ExpressionKind::Object(props) => {
                let mut new_props: Vec<_> = props.to_vec();
                for prop in &mut new_props {
                    use luanext_parser::ast::expression::ObjectProperty;
                    match prop {
                        ObjectProperty::Property { value, .. } => {
                            let mut new_value = (**value).clone();
                            if self.propagate_in_expr(&mut new_value, arena) {
                                *value = arena.alloc(new_value);
                                changed = true;
                            }
                        }
                        ObjectProperty::Computed { value, key, .. } => {
                            let mut new_key = (**key).clone();
                            let mut new_value = (**value).clone();
                            let key_changed = self.propagate_in_expr(&mut new_key, arena);
                            let val_changed = self.propagate_in_expr(&mut new_value, arena);
                            if key_changed || val_changed {
                                *key = arena.alloc(new_key);
                                *value = arena.alloc(new_value);
                                changed = true;
                            }
                        }
                        ObjectProperty::Spread { value, .. } => {
                            let mut new_value = (**value).clone();
                            if self.propagate_in_expr(&mut new_value, arena) {
                                *value = arena.alloc(new_value);
                                changed = true;
                            }
                        }
                    }
                }
                if changed {
                    expr.kind = ExpressionKind::Object(arena.alloc_slice_clone(&new_props));
                }
            }
            ExpressionKind::Conditional(cond, then_expr, else_expr) => {
                let mut new_cond = (**cond).clone();
                let mut new_then = (**then_expr).clone();
                let mut new_else = (**else_expr).clone();
                let cond_changed = self.propagate_in_expr(&mut new_cond, arena);
                let then_changed = self.propagate_in_expr(&mut new_then, arena);
                let else_changed = self.propagate_in_expr(&mut new_else, arena);
                if cond_changed || then_changed || else_changed {
                    expr.kind = ExpressionKind::Conditional(
                        arena.alloc(new_cond),
                        arena.alloc(new_then),
                        arena.alloc(new_else),
                    );
                    changed = true;
                }
            }
            ExpressionKind::Assignment(target, op, value) => {
                let mut new_value = (**value).clone();
                if self.propagate_in_expr(&mut new_value, arena) {
                    expr.kind = ExpressionKind::Assignment(target, *op, arena.alloc(new_value));
                    changed = true;
                }
            }
            ExpressionKind::Parenthesized(inner) => {
                let mut new_inner = (**inner).clone();
                if self.propagate_in_expr(&mut new_inner, arena) {
                    expr.kind = ExpressionKind::Parenthesized(arena.alloc(new_inner));
                    changed = true;
                }
            }
            // Other expression kinds don't need propagation or are not yet supported
            _ => {}
        }

        changed
    }

    /// Propagate copies in a block of statements.
    fn propagate_in_block<'arena>(
        &mut self,
        block: &mut Block<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        // Blocks have immutable statement slices, so we need to clone
        let mut stmts: Vec<_> = block.statements.to_vec();
        let mut changed = false;
        for stmt in &mut stmts {
            changed |= self.propagate_in_statement(stmt, arena);
        }
        if changed {
            block.statements = arena.alloc_slice_clone(&stmts);
        }
        changed
    }

    /// Propagate copies in a single statement.
    fn propagate_in_statement<'arena>(
        &mut self,
        stmt: &mut Statement<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        match stmt {
            Statement::Variable(decl) => {
                // First, propagate in the initializer
                let changed = self.propagate_in_expr(&mut decl.initializer, arena);

                // Then, analyze this declaration for future propagation
                // Only handle simple identifier patterns for now
                if let Pattern::Identifier(ident) = &decl.pattern {
                    self.analyze_variable_decl(ident.node, &decl.initializer);
                }

                changed
            }
            Statement::Expression(expr) => {
                let changed = self.propagate_in_expr(expr, arena);

                // Check for mutations that invalidate propagation
                if let ExpressionKind::Assignment(target, _, _) = &expr.kind {
                    if let ExpressionKind::Identifier(var) = &target.kind {
                        self.invalidate_on_mutation(*var);
                    }
                }

                changed
            }
            Statement::If(if_stmt) => {
                // Propagate in condition
                let mut changed = self.propagate_in_expr(&mut if_stmt.condition, arena);

                // Save current copy_values state before entering branches
                let saved_state = self.copy_values.clone();

                // Propagate in then branch
                changed |= self.propagate_in_block(&mut if_stmt.then_block, arena);

                // Restore state for else-if and else branches
                self.copy_values = saved_state.clone();

                // Propagate in else-if branches
                let mut new_else_ifs: Vec<_> = if_stmt.else_ifs.to_vec();
                let mut eic = false;
                for else_if in &mut new_else_ifs {
                    eic |= self.propagate_in_expr(&mut else_if.condition, arena);
                    self.copy_values = saved_state.clone();
                    eic |= self.propagate_in_block(&mut else_if.block, arena);
                }
                if eic {
                    if_stmt.else_ifs = arena.alloc_slice_clone(&new_else_ifs);
                    changed = true;
                }

                // Propagate in else branch
                if let Some(else_block) = &mut if_stmt.else_block {
                    self.copy_values = saved_state.clone();
                    changed |= self.propagate_in_block(else_block, arena);
                }

                // After branches merge, clear copy_values (conservative)
                self.copy_values.clear();

                changed
            }
            Statement::While(while_stmt) => {
                // Propagate into condition using current copy_values
                let mut changed = self.propagate_in_expr(&mut while_stmt.condition, arena);
                // Clear before loop body (variables may be modified)
                self.copy_values.clear();
                changed |= self.propagate_in_block(&mut while_stmt.body, arena);
                changed
            }
            Statement::For(for_stmt) => {
                let changed = match &**for_stmt {
                    ForStatement::Numeric(for_num_ref) => {
                        let mut new_num = (**for_num_ref).clone();
                        // Propagate into loop bounds using current copy_values
                        let mut ch = self.propagate_in_expr(&mut new_num.start, arena);
                        ch |= self.propagate_in_expr(&mut new_num.end, arena);
                        if let Some(step) = &mut new_num.step {
                            ch |= self.propagate_in_expr(step, arena);
                        }
                        // Clear copy_values before loop body (variables may be modified)
                        self.copy_values.clear();
                        ch |= self.propagate_in_block(&mut new_num.body, arena);
                        if ch {
                            *stmt = Statement::For(
                                arena.alloc(ForStatement::Numeric(arena.alloc(new_num))),
                            );
                        }
                        ch
                    }
                    ForStatement::Generic(for_gen_ref) => {
                        let mut new_gen = for_gen_ref.clone();
                        let mut ch = false;

                        // Propagate into iterators using current copy_values
                        let mut iters: Vec<_> = new_gen.iterators.to_vec();
                        for iter in &mut iters {
                            ch |= self.propagate_in_expr(iter, arena);
                        }
                        if ch {
                            new_gen.iterators = arena.alloc_slice_clone(&iters);
                        }

                        // Clear copy_values before loop body (variables may be modified)
                        self.copy_values.clear();
                        ch |= self.propagate_in_block(&mut new_gen.body, arena);
                        if ch {
                            *stmt = Statement::For(arena.alloc(ForStatement::Generic(new_gen)));
                        }
                        ch
                    }
                };
                // copy_values already cleared above
                changed
            }
            Statement::Return(ret) => {
                let mut vals: Vec<_> = ret.values.to_vec();
                let mut changed = false;
                for expr in &mut vals {
                    changed |= self.propagate_in_expr(expr, arena);
                }
                if changed {
                    ret.values = arena.alloc_slice_clone(&vals);
                }
                changed
            }
            Statement::Function(func) => {
                // Save state before entering function scope
                let saved_state = self.copy_values.clone();
                self.copy_values.clear(); // Functions have separate scope

                let changed = self.propagate_in_block(&mut func.body, arena);

                // Restore state after function
                self.copy_values = saved_state;

                changed
            }
            Statement::Block(block) => {
                // Nested block - continue with current copy_values
                self.propagate_in_block(block, arena)
            }
            Statement::Repeat(repeat_stmt) => {
                // Clear before loop body (variables may be modified)
                self.copy_values.clear();
                let mut changed = self.propagate_in_block(&mut repeat_stmt.body, arena);
                // Propagate into 'until' condition (uses values from loop body)
                changed |= self.propagate_in_expr(&mut repeat_stmt.until, arena);
                // Clear after loop
                self.copy_values.clear();
                changed
            }
            _ => false,
        }
    }
}

impl<'arena> BlockVisitor<'arena> for CopyPropagationPass {
    fn visit_block_stmts(
        &mut self,
        stmts: &mut Vec<Statement<'arena>>,
        arena: &'arena Bump,
    ) -> bool {
        // Clear copy_values at the start of each basic block
        self.copy_values.clear();

        let mut changed = false;
        for stmt in stmts {
            changed |= self.propagate_in_statement(stmt, arena);
        }
        changed
    }
}
