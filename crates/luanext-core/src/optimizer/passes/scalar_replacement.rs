// =============================================================================
// O3: Scalar Replacement of Aggregates (SRA)
// =============================================================================
//
// Replaces local table/object variables with individual scalar local variables
// when the table does not escape the local scope. This eliminates table
// allocation, hash lookups, and GC pressure.
//
// Safety constraints:
// - Only replaces tables assigned via `const`/`let` with object literal initializers
// - Only replaces when ALL accesses are static member reads/writes (`.field`)
// - Table must NOT escape: not passed to functions, not returned, not assigned
//   to other variables, not used as method receiver, not spread
// - No computed index access (`obj[expr]`) — only named member access
// - No use of the table variable itself (only its fields)
// - Maximum 8 fields per table to limit local variable proliferation
//
// Example transformation:
//   const point = { x: 1, y: 2 }
//   const dx = point.x + 10
//   const dy = point.y + 20
// →
//   const point__x = 1
//   const point__y = 2
//   const dx = point__x + 10
//   const dy = point__y + 20
//
// This eliminates the table allocation entirely, replacing hash-table lookups
// with direct local variable accesses (which map to Lua VM registers).

use crate::config::OptimizationLevel;
use crate::optimizer::{AstFeatures, WholeProgramPass};
use crate::MutableProgram;
use bumpalo::Bump;
use luanext_parser::ast::expression::{ArrayElement, Expression, ExpressionKind, ObjectProperty};
use luanext_parser::ast::pattern::Pattern;
use luanext_parser::ast::statement::{
    Block, ForStatement, Statement, VariableDeclaration, VariableKind,
};
use luanext_parser::ast::Spanned;
use luanext_parser::span::Span;
use luanext_parser::string_interner::{StringId, StringInterner};
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::Arc;

/// Maximum number of fields in an object eligible for SRA
const MAX_FIELDS: usize = 8;

/// Return type for candidate extraction: (var_name, fields[(name, index)], field_expressions)
type CandidateInfo<'arena> = (StringId, Vec<(StringId, usize)>, Vec<Expression<'arena>>);

pub struct ScalarReplacementPass {
    interner: Arc<StringInterner>,
}

impl ScalarReplacementPass {
    pub fn new(interner: Arc<StringInterner>) -> Self {
        Self { interner }
    }
}

impl<'arena> WholeProgramPass<'arena> for ScalarReplacementPass {
    fn name(&self) -> &'static str {
        "scalar-replacement"
    }

    fn min_level(&self) -> OptimizationLevel {
        OptimizationLevel::Aggressive
    }

    fn required_features(&self) -> AstFeatures {
        AstFeatures::HAS_OBJECTS
    }

    fn run(
        &mut self,
        program: &mut MutableProgram<'arena>,
        arena: &'arena Bump,
    ) -> Result<bool, String> {
        let mut changed = false;

        // Process top-level statements
        changed |= self.process_stmts(&mut program.statements, arena);

        // Process nested blocks (function bodies, etc.)
        for stmt in &mut program.statements {
            changed |= self.process_nested(stmt, arena);
        }

        Ok(changed)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl ScalarReplacementPass {
    /// Process a statement list, looking for SRA opportunities
    fn process_stmts<'arena>(
        &self,
        stmts: &mut Vec<Statement<'arena>>,
        arena: &'arena Bump,
    ) -> bool {
        let mut changed = false;
        let mut i = 0;

        while i < stmts.len() {
            if let Some((var_name, fields, field_exprs)) =
                self.try_extract_candidate(&stmts[i], arena)
            {
                // Check if the table escapes in the remaining statements
                let remaining = &stmts[i + 1..];
                if !table_escapes(var_name, remaining)
                    && all_accesses_are_fields(var_name, remaining)
                {
                    // Perform scalar replacement
                    let scalar_vars =
                        self.create_scalar_variables(&fields, &field_exprs, var_name, arena);

                    // Replace the table declaration with scalar variable declarations
                    stmts.remove(i);
                    for (j, scalar_stmt) in scalar_vars.into_iter().enumerate() {
                        stmts.insert(i + j, scalar_stmt);
                    }

                    // Rewrite all member accesses in subsequent statements
                    let field_map: FxHashMap<StringId, StringId> = fields
                        .iter()
                        .map(|(field_name, _)| {
                            let scalar_name = self.make_scalar_name(var_name, *field_name);
                            (*field_name, scalar_name)
                        })
                        .collect();

                    let num_scalars = field_map.len();
                    for stmt in stmts[i + num_scalars..].iter_mut() {
                        rewrite_member_access(stmt, var_name, &field_map, arena);
                    }

                    changed = true;
                    // Skip past the inserted scalar vars
                    i += num_scalars;
                    continue;
                }
            }
            i += 1;
        }

        changed
    }

    /// Process nested blocks inside statements
    fn process_nested<'arena>(&self, stmt: &mut Statement<'arena>, arena: &'arena Bump) -> bool {
        match stmt {
            Statement::Function(func) => self.process_block(&mut func.body, arena),
            Statement::If(if_stmt) => {
                let mut changed = self.process_block(&mut if_stmt.then_block, arena);
                let mut new_else_ifs: Vec<_> = if_stmt.else_ifs.to_vec();
                let mut eic = false;
                for ei in &mut new_else_ifs {
                    eic |= self.process_block(&mut ei.block, arena);
                }
                if eic {
                    if_stmt.else_ifs = arena.alloc_slice_clone(&new_else_ifs);
                    changed = true;
                }
                if let Some(eb) = &mut if_stmt.else_block {
                    changed |= self.process_block(eb, arena);
                }
                changed
            }
            Statement::While(while_stmt) => self.process_block(&mut while_stmt.body, arena),
            Statement::For(for_stmt) => match &**for_stmt {
                ForStatement::Numeric(num_ref) => {
                    let mut new_num = (**num_ref).clone();
                    let fc = self.process_block(&mut new_num.body, arena);
                    if fc {
                        *stmt = Statement::For(
                            arena.alloc(ForStatement::Numeric(arena.alloc(new_num))),
                        );
                    }
                    fc
                }
                ForStatement::Generic(gen_ref) => {
                    let mut new_gen = gen_ref.clone();
                    let fc = self.process_block(&mut new_gen.body, arena);
                    if fc {
                        *stmt = Statement::For(arena.alloc(ForStatement::Generic(new_gen)));
                    }
                    fc
                }
            },
            Statement::Repeat(repeat) => self.process_block(&mut repeat.body, arena),
            Statement::Block(block) => self.process_block(block, arena),
            _ => false,
        }
    }

    fn process_block<'arena>(&self, block: &mut Block<'arena>, arena: &'arena Bump) -> bool {
        let mut stmts: Vec<Statement<'arena>> = block.statements.to_vec();
        let mut changed = self.process_stmts(&mut stmts, arena);

        // Also process nested blocks
        for stmt in &mut stmts {
            changed |= self.process_nested(stmt, arena);
        }

        if changed {
            block.statements = arena.alloc_slice_clone(&stmts);
        }
        changed
    }

    /// Try to extract a candidate table from a variable declaration
    fn try_extract_candidate<'arena>(
        &self,
        stmt: &Statement<'arena>,
        _arena: &'arena Bump,
    ) -> Option<CandidateInfo<'arena>> {
        let var = match stmt {
            Statement::Variable(v) => v,
            _ => return None,
        };

        // Must be a simple identifier pattern
        let var_name = match &var.pattern {
            Pattern::Identifier(ident) => ident.node,
            _ => return None,
        };

        // Initializer must be an object literal
        let props = match &var.initializer.kind {
            ExpressionKind::Object(props) => *props,
            _ => return None,
        };

        // Must have ≤ MAX_FIELDS, all simple Property (no computed, no spread)
        if props.len() > MAX_FIELDS || props.is_empty() {
            return None;
        }

        let mut fields = Vec::with_capacity(props.len());
        let mut field_exprs = Vec::with_capacity(props.len());
        let mut seen_fields = FxHashSet::default();

        for (idx, prop) in props.iter().enumerate() {
            match prop {
                ObjectProperty::Property { key, value, .. } => {
                    if !seen_fields.insert(key.node) {
                        return None; // Duplicate field name
                    }
                    fields.push((key.node, idx));
                    field_exprs.push((**value).clone());
                }
                // Computed keys or spread — not eligible
                ObjectProperty::Computed { .. } | ObjectProperty::Spread { .. } => return None,
            }
        }

        Some((var_name, fields, field_exprs))
    }

    /// Create scalar variable declarations for each field
    fn create_scalar_variables<'arena>(
        &self,
        fields: &[(StringId, usize)],
        field_exprs: &[Expression<'arena>],
        var_name: StringId,
        _arena: &'arena Bump,
    ) -> Vec<Statement<'arena>> {
        fields
            .iter()
            .enumerate()
            .map(|(i, (field_name, _))| {
                let scalar_name = self.make_scalar_name(var_name, *field_name);
                Statement::Variable(VariableDeclaration {
                    kind: VariableKind::Const,
                    pattern: Pattern::Identifier(Spanned::new(scalar_name, Span::default())),
                    type_annotation: None,
                    initializer: field_exprs[i].clone(),
                    span: Span::default(),
                })
            })
            .collect()
    }

    /// Generate a scalar variable name: `tableName__fieldName`
    fn make_scalar_name(&self, table_name: StringId, field_name: StringId) -> StringId {
        let table_str = self.interner.resolve(table_name);
        let field_str = self.interner.resolve(field_name);
        let scalar_str = format!("{}__{}", table_str, field_str);
        self.interner.get_or_intern(&scalar_str)
    }
}

// =============================================================================
// Escape Analysis
// =============================================================================

/// Check if a table variable escapes the local scope in the given statements.
/// A table "escapes" if it is:
/// - Passed as an argument to a function call
/// - Returned from the current scope
/// - Assigned to another variable
/// - Used as a method call receiver (obj:method())
/// - Used in a spread expression
/// - Used as an identifier directly (not as part of member access)
fn table_escapes(var_name: StringId, stmts: &[Statement<'_>]) -> bool {
    for stmt in stmts {
        if stmt_escapes(var_name, stmt) {
            return true;
        }
    }
    false
}

fn stmt_escapes(var_name: StringId, stmt: &Statement<'_>) -> bool {
    match stmt {
        Statement::Expression(expr) => expr_escapes(var_name, expr),
        Statement::Variable(var) => expr_escapes(var_name, &var.initializer),
        Statement::Return(ret) => {
            // If the table is returned, it escapes
            ret.values.iter().any(|v| expr_uses_directly(var_name, v))
        }
        Statement::If(if_stmt) => {
            expr_escapes(var_name, &if_stmt.condition)
                || block_escapes(var_name, &if_stmt.then_block)
                || if_stmt.else_ifs.iter().any(|ei| {
                    expr_escapes(var_name, &ei.condition) || block_escapes(var_name, &ei.block)
                })
                || if_stmt
                    .else_block
                    .as_ref()
                    .is_some_and(|b| block_escapes(var_name, b))
        }
        Statement::While(while_stmt) => {
            expr_escapes(var_name, &while_stmt.condition)
                || block_escapes(var_name, &while_stmt.body)
        }
        Statement::For(for_stmt) => match &**for_stmt {
            ForStatement::Numeric(num) => {
                expr_escapes(var_name, &num.start)
                    || expr_escapes(var_name, &num.end)
                    || num
                        .step
                        .as_ref()
                        .is_some_and(|s| expr_escapes(var_name, s))
                    || block_escapes(var_name, &num.body)
            }
            ForStatement::Generic(gen) => {
                gen.iterators.iter().any(|i| expr_escapes(var_name, i))
                    || block_escapes(var_name, &gen.body)
            }
        },
        Statement::Function(func) => block_escapes(var_name, &func.body),
        Statement::Block(block) => block_escapes(var_name, block),
        Statement::Repeat(repeat) => {
            block_escapes(var_name, &repeat.body) || expr_escapes(var_name, &repeat.until)
        }
        _ => false,
    }
}

fn block_escapes(var_name: StringId, block: &Block<'_>) -> bool {
    block.statements.iter().any(|s| stmt_escapes(var_name, s))
}

/// Check if an expression causes the table to escape.
/// Member access (obj.field) does NOT cause escape — that's what we want to replace.
/// But using the variable directly as an argument, return value, or assignment target does.
fn expr_escapes(var_name: StringId, expr: &Expression<'_>) -> bool {
    match &expr.kind {
        // Member access on the table is fine — this is what we replace
        ExpressionKind::Member(obj, _) if is_var(var_name, obj) => false,
        // OptionalMember on the table is fine too
        ExpressionKind::OptionalMember(obj, _) if is_var(var_name, obj) => false,

        // Assignment to a member field is fine: obj.x = value
        // But we need to check if the value escapes
        ExpressionKind::Assignment(target, _, value) => {
            if let ExpressionKind::Member(obj, _) = &target.kind {
                if is_var(var_name, obj) {
                    // target is obj.field — fine. But check value for escapes.
                    return expr_escapes(var_name, value);
                }
            }
            // If the table is on the RHS of assignment to something else, it escapes
            expr_uses_directly(var_name, target) || expr_escapes(var_name, value)
        }

        // Function/method call: if the table is passed as an argument, it escapes
        ExpressionKind::Call(callee, args, _) => {
            // If callee IS the table identifier, it escapes (called as function)
            expr_uses_directly(var_name, callee)
                || args.iter().any(|a| expr_uses_directly(var_name, &a.value))
                || expr_escapes_children(var_name, expr)
        }
        ExpressionKind::MethodCall(obj, _, args, _) => {
            // If the table is the method receiver, it escapes (metatables)
            is_var(var_name, obj) || args.iter().any(|a| expr_uses_directly(var_name, &a.value))
        }

        // Index access: obj[expr] — not safe for SRA
        ExpressionKind::Index(obj, _) if is_var(var_name, obj) => true,

        // For all other expressions, recurse into children
        _ => expr_escapes_children(var_name, expr),
    }
}

/// Recurse into expression children checking for escapes
fn expr_escapes_children(var_name: StringId, expr: &Expression<'_>) -> bool {
    match &expr.kind {
        ExpressionKind::Binary(_, left, right) => {
            expr_escapes(var_name, left) || expr_escapes(var_name, right)
        }
        ExpressionKind::Unary(_, operand) => expr_escapes(var_name, operand),
        ExpressionKind::Call(callee, args, _) => {
            expr_escapes(var_name, callee) || args.iter().any(|a| expr_escapes(var_name, &a.value))
        }
        ExpressionKind::MethodCall(obj, _, args, _) => {
            expr_escapes(var_name, obj) || args.iter().any(|a| expr_escapes(var_name, &a.value))
        }
        ExpressionKind::Member(obj, _) => expr_escapes(var_name, obj),
        ExpressionKind::Index(obj, idx) => {
            expr_escapes(var_name, obj) || expr_escapes(var_name, idx)
        }
        ExpressionKind::Conditional(c, t, e) => {
            expr_escapes(var_name, c) || expr_escapes(var_name, t) || expr_escapes(var_name, e)
        }
        ExpressionKind::Parenthesized(inner) => expr_escapes(var_name, inner),
        ExpressionKind::Array(elems) => elems.iter().any(|e| match e {
            ArrayElement::Expression(ex) | ArrayElement::Spread(ex) => {
                expr_uses_directly(var_name, ex)
            }
        }),
        ExpressionKind::Object(props) => props.iter().any(|p| match p {
            ObjectProperty::Property { value, .. } => expr_escapes(var_name, value),
            ObjectProperty::Computed { key, value, .. } => {
                expr_escapes(var_name, key) || expr_escapes(var_name, value)
            }
            ObjectProperty::Spread { value, .. } => expr_uses_directly(var_name, value),
        }),
        ExpressionKind::Assignment(target, _, value) => {
            expr_escapes(var_name, target) || expr_escapes(var_name, value)
        }
        ExpressionKind::Pipe(l, r) | ExpressionKind::ErrorChain(l, r) => {
            expr_escapes(var_name, l) || expr_escapes(var_name, r)
        }
        ExpressionKind::TypeAssertion(inner, _) => expr_escapes(var_name, inner),
        _ => false,
    }
}

/// Check if an expression directly uses the variable (not through member access)
fn expr_uses_directly(var_name: StringId, expr: &Expression<'_>) -> bool {
    match &expr.kind {
        ExpressionKind::Identifier(id) => *id == var_name,
        ExpressionKind::Member(obj, _) if is_var(var_name, obj) => {
            // Member access is NOT a direct use (it's a field access)
            false
        }
        // Recurse into children
        ExpressionKind::Binary(_, l, r) => {
            expr_uses_directly(var_name, l) || expr_uses_directly(var_name, r)
        }
        ExpressionKind::Unary(_, o) => expr_uses_directly(var_name, o),
        ExpressionKind::Parenthesized(i) => expr_uses_directly(var_name, i),
        ExpressionKind::Conditional(c, t, e) => {
            expr_uses_directly(var_name, c)
                || expr_uses_directly(var_name, t)
                || expr_uses_directly(var_name, e)
        }
        ExpressionKind::Call(callee, args, _) => {
            expr_uses_directly(var_name, callee)
                || args.iter().any(|a| expr_uses_directly(var_name, &a.value))
        }
        ExpressionKind::MethodCall(obj, _, args, _) => {
            expr_uses_directly(var_name, obj)
                || args.iter().any(|a| expr_uses_directly(var_name, &a.value))
        }
        ExpressionKind::Member(obj, _) => expr_uses_directly(var_name, obj),
        ExpressionKind::Index(obj, idx) => {
            expr_uses_directly(var_name, obj) || expr_uses_directly(var_name, idx)
        }
        ExpressionKind::Assignment(t, _, v) => {
            expr_uses_directly(var_name, t) || expr_uses_directly(var_name, v)
        }
        _ => false,
    }
}

/// Check if expression is a direct identifier reference to var_name
fn is_var(var_name: StringId, expr: &Expression<'_>) -> bool {
    matches!(&expr.kind, ExpressionKind::Identifier(id) if *id == var_name)
}

// =============================================================================
// Access Pattern Validation
// =============================================================================

/// Verify that ALL uses of the table are member accesses (no bare identifier use)
fn all_accesses_are_fields(var_name: StringId, stmts: &[Statement<'_>]) -> bool {
    for stmt in stmts {
        if !stmt_all_accesses_are_fields(var_name, stmt) {
            return false;
        }
    }
    true
}

fn stmt_all_accesses_are_fields(var_name: StringId, stmt: &Statement<'_>) -> bool {
    match stmt {
        Statement::Expression(expr) => expr_all_accesses_are_fields(var_name, expr),
        Statement::Variable(var) => expr_all_accesses_are_fields(var_name, &var.initializer),
        Statement::Return(ret) => ret
            .values
            .iter()
            .all(|v| expr_all_accesses_are_fields(var_name, v)),
        Statement::If(if_stmt) => {
            expr_all_accesses_are_fields(var_name, &if_stmt.condition)
                && block_all_accesses(var_name, &if_stmt.then_block)
                && if_stmt.else_ifs.iter().all(|ei| {
                    expr_all_accesses_are_fields(var_name, &ei.condition)
                        && block_all_accesses(var_name, &ei.block)
                })
                && if_stmt
                    .else_block
                    .as_ref()
                    .is_none_or(|b| block_all_accesses(var_name, b))
        }
        Statement::While(while_stmt) => {
            expr_all_accesses_are_fields(var_name, &while_stmt.condition)
                && block_all_accesses(var_name, &while_stmt.body)
        }
        Statement::For(for_stmt) => match &**for_stmt {
            ForStatement::Numeric(num) => {
                expr_all_accesses_are_fields(var_name, &num.start)
                    && expr_all_accesses_are_fields(var_name, &num.end)
                    && num
                        .step
                        .as_ref()
                        .is_none_or(|s| expr_all_accesses_are_fields(var_name, s))
                    && block_all_accesses(var_name, &num.body)
            }
            ForStatement::Generic(gen) => {
                gen.iterators
                    .iter()
                    .all(|i| expr_all_accesses_are_fields(var_name, i))
                    && block_all_accesses(var_name, &gen.body)
            }
        },
        Statement::Function(func) => block_all_accesses(var_name, &func.body),
        Statement::Block(block) => block_all_accesses(var_name, block),
        Statement::Repeat(repeat) => {
            block_all_accesses(var_name, &repeat.body)
                && expr_all_accesses_are_fields(var_name, &repeat.until)
        }
        _ => true,
    }
}

fn block_all_accesses(var_name: StringId, block: &Block<'_>) -> bool {
    block
        .statements
        .iter()
        .all(|s| stmt_all_accesses_are_fields(var_name, s))
}

fn expr_all_accesses_are_fields(var_name: StringId, expr: &Expression<'_>) -> bool {
    match &expr.kind {
        // Direct use of the table — NOT ok (bare identifier)
        ExpressionKind::Identifier(id) if *id == var_name => false,

        // Member access on the table — OK (this is what we replace)
        ExpressionKind::Member(obj, _) if is_var(var_name, obj) => true,

        // Assignment to member — OK, but check value
        ExpressionKind::Assignment(target, _, value) => {
            if let ExpressionKind::Member(obj, _) = &target.kind {
                if is_var(var_name, obj) {
                    return expr_all_accesses_are_fields(var_name, value);
                }
            }
            expr_all_accesses_are_fields(var_name, target)
                && expr_all_accesses_are_fields(var_name, value)
        }

        // Index access on the table — NOT ok (dynamic access)
        ExpressionKind::Index(obj, _) if is_var(var_name, obj) => false,

        // Recurse into all children
        ExpressionKind::Binary(_, l, r) => {
            expr_all_accesses_are_fields(var_name, l) && expr_all_accesses_are_fields(var_name, r)
        }
        ExpressionKind::Unary(_, o) => expr_all_accesses_are_fields(var_name, o),
        ExpressionKind::Parenthesized(i) => expr_all_accesses_are_fields(var_name, i),
        ExpressionKind::Conditional(c, t, e) => {
            expr_all_accesses_are_fields(var_name, c)
                && expr_all_accesses_are_fields(var_name, t)
                && expr_all_accesses_are_fields(var_name, e)
        }
        ExpressionKind::Call(callee, args, _) => {
            expr_all_accesses_are_fields(var_name, callee)
                && args
                    .iter()
                    .all(|a| expr_all_accesses_are_fields(var_name, &a.value))
        }
        ExpressionKind::MethodCall(obj, _, args, _) => {
            expr_all_accesses_are_fields(var_name, obj)
                && args
                    .iter()
                    .all(|a| expr_all_accesses_are_fields(var_name, &a.value))
        }
        ExpressionKind::Member(obj, _) => expr_all_accesses_are_fields(var_name, obj),
        ExpressionKind::Index(obj, idx) => {
            expr_all_accesses_are_fields(var_name, obj)
                && expr_all_accesses_are_fields(var_name, idx)
        }
        ExpressionKind::Array(elems) => elems.iter().all(|e| match e {
            ArrayElement::Expression(ex) | ArrayElement::Spread(ex) => {
                expr_all_accesses_are_fields(var_name, ex)
            }
        }),
        ExpressionKind::Object(props) => props.iter().all(|p| match p {
            ObjectProperty::Property { value, .. } => expr_all_accesses_are_fields(var_name, value),
            ObjectProperty::Computed { key, value, .. } => {
                expr_all_accesses_are_fields(var_name, key)
                    && expr_all_accesses_are_fields(var_name, value)
            }
            ObjectProperty::Spread { value, .. } => expr_all_accesses_are_fields(var_name, value),
        }),
        ExpressionKind::Pipe(l, r) | ExpressionKind::ErrorChain(l, r) => {
            expr_all_accesses_are_fields(var_name, l) && expr_all_accesses_are_fields(var_name, r)
        }
        ExpressionKind::TypeAssertion(inner, _) => expr_all_accesses_are_fields(var_name, inner),

        // All other nodes (literals, other identifiers, etc.)
        _ => true,
    }
}

// =============================================================================
// Member Access Rewriting
// =============================================================================

/// Rewrite member accesses of `var_name.field` → `var_name__field` in a statement
fn rewrite_member_access<'arena>(
    stmt: &mut Statement<'arena>,
    var_name: StringId,
    field_map: &FxHashMap<StringId, StringId>,
    arena: &'arena Bump,
) -> bool {
    match stmt {
        Statement::Expression(expr) => rewrite_expr(expr, var_name, field_map, arena),
        Statement::Variable(var) => rewrite_expr(&mut var.initializer, var_name, field_map, arena),
        Statement::Return(ret) => {
            let mut values: Vec<Expression<'arena>> = ret.values.to_vec();
            let mut changed = false;
            for v in &mut values {
                changed |= rewrite_expr(v, var_name, field_map, arena);
            }
            if changed {
                ret.values = arena.alloc_slice_clone(&values);
            }
            changed
        }
        Statement::If(if_stmt) => {
            let mut changed = rewrite_expr(&mut if_stmt.condition, var_name, field_map, arena);
            changed |= rewrite_block(&mut if_stmt.then_block, var_name, field_map, arena);
            let mut new_else_ifs: Vec<_> = if_stmt.else_ifs.to_vec();
            let mut eic = false;
            for ei in &mut new_else_ifs {
                eic |= rewrite_expr(&mut ei.condition, var_name, field_map, arena);
                eic |= rewrite_block(&mut ei.block, var_name, field_map, arena);
            }
            if eic {
                if_stmt.else_ifs = arena.alloc_slice_clone(&new_else_ifs);
                changed = true;
            }
            if let Some(eb) = &mut if_stmt.else_block {
                changed |= rewrite_block(eb, var_name, field_map, arena);
            }
            changed
        }
        Statement::While(while_stmt) => {
            let mut changed = rewrite_expr(&mut while_stmt.condition, var_name, field_map, arena);
            changed |= rewrite_block(&mut while_stmt.body, var_name, field_map, arena);
            changed
        }
        Statement::For(for_stmt) => match &**for_stmt {
            ForStatement::Numeric(num_ref) => {
                let mut new_num = (**num_ref).clone();
                let mut fc = false;
                fc |= rewrite_expr(&mut new_num.start, var_name, field_map, arena);
                fc |= rewrite_expr(&mut new_num.end, var_name, field_map, arena);
                if let Some(step) = &mut new_num.step {
                    fc |= rewrite_expr(step, var_name, field_map, arena);
                }
                fc |= rewrite_block(&mut new_num.body, var_name, field_map, arena);
                if fc {
                    *stmt =
                        Statement::For(arena.alloc(ForStatement::Numeric(arena.alloc(new_num))));
                }
                fc
            }
            ForStatement::Generic(gen_ref) => {
                let mut new_gen = gen_ref.clone();
                let mut fc = false;
                let mut new_iters: Vec<Expression<'arena>> = new_gen.iterators.to_vec();
                for iter in &mut new_iters {
                    fc |= rewrite_expr(iter, var_name, field_map, arena);
                }
                if fc {
                    new_gen.iterators = arena.alloc_slice_clone(&new_iters);
                }
                fc |= rewrite_block(&mut new_gen.body, var_name, field_map, arena);
                if fc {
                    *stmt = Statement::For(arena.alloc(ForStatement::Generic(new_gen)));
                }
                fc
            }
        },
        Statement::Function(func) => rewrite_block(&mut func.body, var_name, field_map, arena),
        Statement::Block(block) => rewrite_block(block, var_name, field_map, arena),
        Statement::Repeat(repeat) => {
            let mut changed = rewrite_block(&mut repeat.body, var_name, field_map, arena);
            changed |= rewrite_expr(&mut repeat.until, var_name, field_map, arena);
            changed
        }
        _ => false,
    }
}

fn rewrite_block<'arena>(
    block: &mut Block<'arena>,
    var_name: StringId,
    field_map: &FxHashMap<StringId, StringId>,
    arena: &'arena Bump,
) -> bool {
    let mut stmts: Vec<Statement<'arena>> = block.statements.to_vec();
    let mut changed = false;
    for s in &mut stmts {
        changed |= rewrite_member_access(s, var_name, field_map, arena);
    }
    if changed {
        block.statements = arena.alloc_slice_clone(&stmts);
    }
    changed
}

/// Rewrite member access expressions: `obj.field` → `obj__field` identifier
fn rewrite_expr<'arena>(
    expr: &mut Expression<'arena>,
    var_name: StringId,
    field_map: &FxHashMap<StringId, StringId>,
    arena: &'arena Bump,
) -> bool {
    match &expr.kind {
        // obj.field → obj__field
        ExpressionKind::Member(obj, member) if is_var(var_name, obj) => {
            if let Some(&scalar_name) = field_map.get(&member.node) {
                expr.kind = ExpressionKind::Identifier(scalar_name);
                return true;
            }
            false
        }

        // obj.field = value → obj__field = value
        ExpressionKind::Assignment(target, op, value) => {
            let op = *op;
            let mut new_target = (**target).clone();
            let mut new_value = (**value).clone();
            let tc = rewrite_expr(&mut new_target, var_name, field_map, arena);
            let vc = rewrite_expr(&mut new_value, var_name, field_map, arena);
            if tc || vc {
                expr.kind =
                    ExpressionKind::Assignment(arena.alloc(new_target), op, arena.alloc(new_value));
                return true;
            }
            false
        }

        ExpressionKind::Binary(op, left, right) => {
            let op = *op;
            let mut new_left = (**left).clone();
            let mut new_right = (**right).clone();
            let lc = rewrite_expr(&mut new_left, var_name, field_map, arena);
            let rc = rewrite_expr(&mut new_right, var_name, field_map, arena);
            if lc || rc {
                expr.kind =
                    ExpressionKind::Binary(op, arena.alloc(new_left), arena.alloc(new_right));
                true
            } else {
                false
            }
        }

        ExpressionKind::Unary(op, operand) => {
            let op = *op;
            let mut new_operand = (**operand).clone();
            if rewrite_expr(&mut new_operand, var_name, field_map, arena) {
                expr.kind = ExpressionKind::Unary(op, arena.alloc(new_operand));
                true
            } else {
                false
            }
        }

        ExpressionKind::Call(callee, args, type_args) => {
            let type_args = *type_args;
            let mut new_callee = (**callee).clone();
            let mut new_args: Vec<_> = args.to_vec();
            let mut sub_changed = false;
            sub_changed |= rewrite_expr(&mut new_callee, var_name, field_map, arena);
            for arg in &mut new_args {
                sub_changed |= rewrite_expr(&mut arg.value, var_name, field_map, arena);
            }
            if sub_changed {
                expr.kind = ExpressionKind::Call(
                    arena.alloc(new_callee),
                    arena.alloc_slice_clone(&new_args),
                    type_args,
                );
                true
            } else {
                false
            }
        }

        ExpressionKind::Conditional(cond, then_e, else_e) => {
            let mut new_cond = (**cond).clone();
            let mut new_then = (**then_e).clone();
            let mut new_else = (**else_e).clone();
            let cc = rewrite_expr(&mut new_cond, var_name, field_map, arena);
            let tc = rewrite_expr(&mut new_then, var_name, field_map, arena);
            let ec = rewrite_expr(&mut new_else, var_name, field_map, arena);
            if cc || tc || ec {
                expr.kind = ExpressionKind::Conditional(
                    arena.alloc(new_cond),
                    arena.alloc(new_then),
                    arena.alloc(new_else),
                );
                true
            } else {
                false
            }
        }

        ExpressionKind::Parenthesized(inner) => {
            let mut new_inner = (**inner).clone();
            if rewrite_expr(&mut new_inner, var_name, field_map, arena) {
                expr.kind = ExpressionKind::Parenthesized(arena.alloc(new_inner));
                true
            } else {
                false
            }
        }

        ExpressionKind::Member(obj, member) => {
            let member = member.clone();
            let mut new_obj = (**obj).clone();
            if rewrite_expr(&mut new_obj, var_name, field_map, arena) {
                expr.kind = ExpressionKind::Member(arena.alloc(new_obj), member);
                true
            } else {
                false
            }
        }

        ExpressionKind::Index(obj, index) => {
            let mut new_obj = (**obj).clone();
            let mut new_index = (**index).clone();
            let oc = rewrite_expr(&mut new_obj, var_name, field_map, arena);
            let ic = rewrite_expr(&mut new_index, var_name, field_map, arena);
            if oc || ic {
                expr.kind = ExpressionKind::Index(arena.alloc(new_obj), arena.alloc(new_index));
                true
            } else {
                false
            }
        }

        ExpressionKind::Pipe(left, right) => {
            let mut new_left = (**left).clone();
            let mut new_right = (**right).clone();
            let lc = rewrite_expr(&mut new_left, var_name, field_map, arena);
            let rc = rewrite_expr(&mut new_right, var_name, field_map, arena);
            if lc || rc {
                expr.kind = ExpressionKind::Pipe(arena.alloc(new_left), arena.alloc(new_right));
                true
            } else {
                false
            }
        }

        ExpressionKind::MethodCall(obj, method, args, type_args) => {
            let method = method.clone();
            let type_args = *type_args;
            let mut new_obj = (**obj).clone();
            let mut new_args: Vec<_> = args.to_vec();
            let mut sub_changed = false;
            sub_changed |= rewrite_expr(&mut new_obj, var_name, field_map, arena);
            for arg in &mut new_args {
                sub_changed |= rewrite_expr(&mut arg.value, var_name, field_map, arena);
            }
            if sub_changed {
                expr.kind = ExpressionKind::MethodCall(
                    arena.alloc(new_obj),
                    method,
                    arena.alloc_slice_clone(&new_args),
                    type_args,
                );
                true
            } else {
                false
            }
        }

        ExpressionKind::TypeAssertion(inner, ty) => {
            let ty = ty.clone();
            let mut new_inner = (**inner).clone();
            if rewrite_expr(&mut new_inner, var_name, field_map, arena) {
                expr.kind = ExpressionKind::TypeAssertion(arena.alloc(new_inner), ty);
                true
            } else {
                false
            }
        }

        // Leaf nodes — nothing to rewrite
        _ => false,
    }
}
