// =============================================================================
// O3: Generic Specialization Pass
// =============================================================================

use crate::config::OptimizationLevel;
use crate::optimizer::WholeProgramPass;
use crate::MutableProgram;
use crate::{build_substitutions, instantiate_function_declaration};
use bumpalo::Bump;
use luanext_parser::ast::expression::{ArrayElement, Expression, ExpressionKind, ObjectProperty};
use luanext_parser::ast::statement::{ForStatement, FunctionDeclaration, Statement};
use luanext_parser::ast::types::Type;
use luanext_parser::string_interner::{StringId, StringInterner};
use rustc_hash::FxHashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Computes a hash of type arguments for caching specialized functions
fn hash_type_args(type_args: &[Type<'_>]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for t in type_args {
        // Hash the debug representation - simple but effective
        format!("{:?}", t.kind).hash(&mut hasher);
    }
    hasher.finish()
}

/// Generic specialization pass
/// Creates specialized versions of generic functions for known types
#[derive(Default)]
pub struct GenericSpecializationPass {
    interner: Option<Arc<StringInterner>>,
    /// Maps (function_name, type_args_hash) -> specialized_function_name
    specializations: FxHashMap<(StringId, u64), StringId>,
    /// Counter for generating unique specialization IDs
    next_spec_id: usize,
}

impl GenericSpecializationPass {
    pub fn new(interner: Arc<StringInterner>) -> Self {
        Self {
            interner: Some(interner),
            specializations: FxHashMap::default(),
            next_spec_id: 0,
        }
    }
}

/// Temporary run-time context holding arena-lifetime data for a single pass execution.
///
/// ## Lifetime Management Pattern
///
/// This struct uses two lifetimes to separate concerns:
/// - `'arena`: Tied to the AST arena allocator - data with this lifetime is arena-allocated
/// - `'pass`: Tied to the mutable borrow of the pass - much shorter than 'arena
///
/// The separation allows us to:
/// 1. Store arena-allocated AST nodes (`FunctionDeclaration<'arena>`, `Statement<'arena>`)
/// 2. Access pass-level state (specializations map, interner) via `&'pass mut`
/// 3. Keep all arena data localized to this temporary context, not in the pass itself
///
/// This pattern is necessary because:
/// - Arena data can't outlive the arena (which is reset between compilations)
/// - Pass state needs to persist across multiple runs for caching
/// - Rust's lifetime system prevents accidentally storing arena data in the pass
struct SpecializationContext<'arena, 'pass> {
    pass: &'pass mut GenericSpecializationPass,
    /// Collected generic function declarations (arena-allocated AST nodes)
    generic_functions: FxHashMap<StringId, FunctionDeclaration<'arena>>,
    /// New specialized function declarations to add to program (arena-allocated)
    new_functions: Vec<Statement<'arena>>,
}

impl<'arena, 'pass> SpecializationContext<'arena, 'pass> {
    fn new(pass: &'pass mut GenericSpecializationPass) -> Self {
        Self {
            pass,
            generic_functions: FxHashMap::default(),
            new_functions: Vec::new(),
        }
    }

    /// Collects all generic function declarations from the program
    fn collect_generic_functions(&mut self, program: &MutableProgram<'arena>) {
        for stmt in &program.statements {
            if let Statement::Function(func) = stmt {
                if func.type_parameters.is_some() {
                    self.generic_functions.insert(func.name.node, func.clone());
                }
            }
        }
    }

    /// Creates a specialized version of a generic function with concrete type arguments
    fn specialize_function(
        &mut self,
        arena: &'arena Bump,
        func: &FunctionDeclaration<'arena>,
        type_args: &[Type<'arena>],
    ) -> Option<StringId> {
        let interner = self.pass.interner.as_ref()?;
        let type_params = func.type_parameters.as_ref()?;

        // Build type substitution map
        let substitutions = match build_substitutions(type_params, type_args) {
            Ok(s) => s,
            Err(_) => return None,
        };

        // Check cache first
        let type_args_hash = hash_type_args(type_args);
        let cache_key = (func.name.node, type_args_hash);
        if let Some(&specialized_name) = self.pass.specializations.get(&cache_key) {
            return Some(specialized_name);
        }

        // Generate specialized function name: funcName__spec{id}
        let orig_name = interner.resolve(func.name.node);
        let specialized_name_str = format!("{}__spec{}", orig_name, self.pass.next_spec_id);
        self.pass.next_spec_id += 1;

        // Intern the new name
        let specialized_name = interner.get_or_intern(&specialized_name_str);

        // Create specialized function by instantiating with type substitutions
        let mut specialized_func = instantiate_function_declaration(arena, func, &substitutions);
        specialized_func.name = luanext_parser::ast::Spanned::new(specialized_name, func.name.span);

        // Add to cache and to list of new functions
        self.pass
            .specializations
            .insert(cache_key, specialized_name);
        self.new_functions
            .push(Statement::Function(specialized_func));

        Some(specialized_name)
    }

    /// Processes a statement looking for call sites to specialize.
    /// Uses clone-and-rebuild for arena-allocated sub-structures.
    fn specialize_calls_in_statement(
        &mut self,
        stmt: &mut Statement<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        let mut changed = false;

        match stmt {
            Statement::Variable(var_decl) => {
                if self.specialize_calls_in_expression(&mut var_decl.initializer, arena) {
                    changed = true;
                }
            }
            Statement::Expression(expr) => {
                if self.specialize_calls_in_expression(expr, arena) {
                    changed = true;
                }
            }
            Statement::Return(ret) => {
                let mut values: Vec<Expression<'arena>> = ret.values.to_vec();
                let mut ret_changed = false;
                for value in &mut values {
                    if self.specialize_calls_in_expression(value, arena) {
                        ret_changed = true;
                    }
                }
                if ret_changed {
                    ret.values = arena.alloc_slice_clone(&values);
                    changed = true;
                }
            }
            Statement::If(if_stmt) => {
                if self.specialize_calls_in_expression(&mut if_stmt.condition, arena) {
                    changed = true;
                }
                // then_block
                {
                    let mut stmts: Vec<Statement<'arena>> = if_stmt.then_block.statements.to_vec();
                    let mut bc = false;
                    for s in &mut stmts {
                        if self.specialize_calls_in_statement(s, arena) {
                            bc = true;
                        }
                    }
                    if bc {
                        if_stmt.then_block.statements = arena.alloc_slice_clone(&stmts);
                        changed = true;
                    }
                }
                // else_ifs
                {
                    let mut new_else_ifs: Vec<_> = if_stmt.else_ifs.to_vec();
                    let mut eic = false;
                    for else_if in &mut new_else_ifs {
                        if self.specialize_calls_in_expression(&mut else_if.condition, arena) {
                            eic = true;
                        }
                        let mut stmts: Vec<Statement<'arena>> = else_if.block.statements.to_vec();
                        let mut bc = false;
                        for s in &mut stmts {
                            if self.specialize_calls_in_statement(s, arena) {
                                bc = true;
                            }
                        }
                        if bc {
                            else_if.block.statements = arena.alloc_slice_clone(&stmts);
                            eic = true;
                        }
                    }
                    if eic {
                        if_stmt.else_ifs = arena.alloc_slice_clone(&new_else_ifs);
                        changed = true;
                    }
                }
                // else_block
                if let Some(else_block) = &mut if_stmt.else_block {
                    let mut stmts: Vec<Statement<'arena>> = else_block.statements.to_vec();
                    let mut bc = false;
                    for s in &mut stmts {
                        if self.specialize_calls_in_statement(s, arena) {
                            bc = true;
                        }
                    }
                    if bc {
                        else_block.statements = arena.alloc_slice_clone(&stmts);
                        changed = true;
                    }
                }
            }
            Statement::While(while_stmt) => {
                if self.specialize_calls_in_expression(&mut while_stmt.condition, arena) {
                    changed = true;
                }
                let mut stmts: Vec<Statement<'arena>> = while_stmt.body.statements.to_vec();
                let mut bc = false;
                for s in &mut stmts {
                    if self.specialize_calls_in_statement(s, arena) {
                        bc = true;
                    }
                }
                if bc {
                    while_stmt.body.statements = arena.alloc_slice_clone(&stmts);
                    changed = true;
                }
            }
            Statement::For(for_stmt) => match &**for_stmt {
                ForStatement::Numeric(num_ref) => {
                    let mut new_num = (**num_ref).clone();
                    let mut fc = false;
                    if self.specialize_calls_in_expression(&mut new_num.start, arena) {
                        fc = true;
                    }
                    if self.specialize_calls_in_expression(&mut new_num.end, arena) {
                        fc = true;
                    }
                    if let Some(step) = &mut new_num.step {
                        if self.specialize_calls_in_expression(step, arena) {
                            fc = true;
                        }
                    }
                    let mut stmts: Vec<Statement<'arena>> = new_num.body.statements.to_vec();
                    let mut bc = false;
                    for s in &mut stmts {
                        if self.specialize_calls_in_statement(s, arena) {
                            bc = true;
                        }
                    }
                    if bc {
                        new_num.body.statements = arena.alloc_slice_clone(&stmts);
                        fc = true;
                    }
                    if fc {
                        *stmt = Statement::For(
                            arena.alloc(ForStatement::Numeric(arena.alloc(new_num))),
                        );
                        changed = true;
                    }
                }
                ForStatement::Generic(gen_ref) => {
                    let mut new_gen = gen_ref.clone();
                    let mut fc = false;
                    let mut new_iters: Vec<Expression<'arena>> = new_gen.iterators.to_vec();
                    let mut ic = false;
                    for iter in &mut new_iters {
                        if self.specialize_calls_in_expression(iter, arena) {
                            ic = true;
                        }
                    }
                    if ic {
                        new_gen.iterators = arena.alloc_slice_clone(&new_iters);
                        fc = true;
                    }
                    let mut stmts: Vec<Statement<'arena>> = new_gen.body.statements.to_vec();
                    let mut bc = false;
                    for s in &mut stmts {
                        if self.specialize_calls_in_statement(s, arena) {
                            bc = true;
                        }
                    }
                    if bc {
                        new_gen.body.statements = arena.alloc_slice_clone(&stmts);
                        fc = true;
                    }
                    if fc {
                        *stmt = Statement::For(arena.alloc(ForStatement::Generic(new_gen)));
                        changed = true;
                    }
                }
            },
            Statement::Function(func) => {
                let mut stmts: Vec<Statement<'arena>> = func.body.statements.to_vec();
                let mut bc = false;
                for s in &mut stmts {
                    if self.specialize_calls_in_statement(s, arena) {
                        bc = true;
                    }
                }
                if bc {
                    func.body.statements = arena.alloc_slice_clone(&stmts);
                    changed = true;
                }
            }
            Statement::Block(block) => {
                let mut stmts: Vec<Statement<'arena>> = block.statements.to_vec();
                let mut bc = false;
                for s in &mut stmts {
                    if self.specialize_calls_in_statement(s, arena) {
                        bc = true;
                    }
                }
                if bc {
                    block.statements = arena.alloc_slice_clone(&stmts);
                    changed = true;
                }
            }
            Statement::Repeat(repeat) => {
                let mut stmts: Vec<Statement<'arena>> = repeat.body.statements.to_vec();
                let mut bc = false;
                for s in &mut stmts {
                    if self.specialize_calls_in_statement(s, arena) {
                        bc = true;
                    }
                }
                if bc {
                    repeat.body.statements = arena.alloc_slice_clone(&stmts);
                    changed = true;
                }
                if self.specialize_calls_in_expression(&mut repeat.until, arena) {
                    changed = true;
                }
            }
            Statement::Throw(throw) => {
                if self.specialize_calls_in_expression(&mut throw.expression, arena) {
                    changed = true;
                }
            }
            // Other statements don't contain call expressions we care about
            _ => {}
        }

        changed
    }

    /// Processes an expression looking for call sites to specialize.
    /// Uses clone-and-rebuild for arena-allocated sub-expressions.
    fn specialize_calls_in_expression(
        &mut self,
        expr: &mut Expression<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        let mut changed = false;

        match &expr.kind {
            ExpressionKind::Call(callee, args, type_args) => {
                // Clone sub-expressions for mutation
                let mut new_callee = (**callee).clone();
                let mut new_args: Vec<_> = args.to_vec();
                let mut type_args_val = *type_args;

                // First process nested expressions
                let mut sub_changed = false;
                if self.specialize_calls_in_expression(&mut new_callee, arena) {
                    sub_changed = true;
                }
                for arg in &mut new_args {
                    if self.specialize_calls_in_expression(&mut arg.value, arena) {
                        sub_changed = true;
                    }
                }

                // Check if this is a call to a generic function with concrete type args
                if let Some(ta) = type_args_val {
                    if !ta.is_empty() {
                        // Check if callee is a direct identifier reference to a generic function
                        if let ExpressionKind::Identifier(func_name) = &new_callee.kind {
                            if let Some(func) = self.generic_functions.get(func_name).cloned() {
                                // Specialize this call
                                if let Some(specialized_name) =
                                    self.specialize_function(arena, &func, ta)
                                {
                                    // Replace callee with specialized function name
                                    new_callee.kind = ExpressionKind::Identifier(specialized_name);
                                    // Clear type arguments since the function is now monomorphic
                                    type_args_val = None;
                                    sub_changed = true;
                                }
                            }
                        }
                    }
                }

                if sub_changed {
                    expr.kind = ExpressionKind::Call(
                        arena.alloc(new_callee),
                        arena.alloc_slice_clone(&new_args),
                        type_args_val,
                    );
                    changed = true;
                }
            }

            ExpressionKind::Binary(op, left, right) => {
                let op = *op;
                let mut new_left = (**left).clone();
                let mut new_right = (**right).clone();
                let lc = self.specialize_calls_in_expression(&mut new_left, arena);
                let rc = self.specialize_calls_in_expression(&mut new_right, arena);
                if lc || rc {
                    expr.kind =
                        ExpressionKind::Binary(op, arena.alloc(new_left), arena.alloc(new_right));
                    changed = true;
                }
            }

            ExpressionKind::Unary(op, operand) => {
                let op = *op;
                let mut new_operand = (**operand).clone();
                if self.specialize_calls_in_expression(&mut new_operand, arena) {
                    expr.kind = ExpressionKind::Unary(op, arena.alloc(new_operand));
                    changed = true;
                }
            }

            ExpressionKind::Assignment(target, op, value) => {
                let op = *op;
                let mut new_target = (**target).clone();
                let mut new_value = (**value).clone();
                let tc = self.specialize_calls_in_expression(&mut new_target, arena);
                let vc = self.specialize_calls_in_expression(&mut new_value, arena);
                if tc || vc {
                    expr.kind = ExpressionKind::Assignment(
                        arena.alloc(new_target),
                        op,
                        arena.alloc(new_value),
                    );
                    changed = true;
                }
            }

            ExpressionKind::MethodCall(obj, method, args, type_args) => {
                let method = method.clone();
                let type_args = *type_args;
                let mut new_obj = (**obj).clone();
                let mut new_args: Vec<_> = args.to_vec();
                let mut sub_changed = false;
                if self.specialize_calls_in_expression(&mut new_obj, arena) {
                    sub_changed = true;
                }
                for arg in &mut new_args {
                    if self.specialize_calls_in_expression(&mut arg.value, arena) {
                        sub_changed = true;
                    }
                }
                // Method specialization is more complex - skip for now
                if sub_changed {
                    expr.kind = ExpressionKind::MethodCall(
                        arena.alloc(new_obj),
                        method,
                        arena.alloc_slice_clone(&new_args),
                        type_args,
                    );
                    changed = true;
                }
            }

            ExpressionKind::Member(obj, member) => {
                let member = member.clone();
                let mut new_obj = (**obj).clone();
                if self.specialize_calls_in_expression(&mut new_obj, arena) {
                    expr.kind = ExpressionKind::Member(arena.alloc(new_obj), member);
                    changed = true;
                }
            }

            ExpressionKind::Index(obj, index) => {
                let mut new_obj = (**obj).clone();
                let mut new_index = (**index).clone();
                let oc = self.specialize_calls_in_expression(&mut new_obj, arena);
                let ic = self.specialize_calls_in_expression(&mut new_index, arena);
                if oc || ic {
                    expr.kind = ExpressionKind::Index(arena.alloc(new_obj), arena.alloc(new_index));
                    changed = true;
                }
            }

            ExpressionKind::Array(elements) => {
                let mut new_elements: Vec<_> = elements.to_vec();
                let mut ec = false;
                for elem in &mut new_elements {
                    match elem {
                        ArrayElement::Expression(e) | ArrayElement::Spread(e) => {
                            if self.specialize_calls_in_expression(e, arena) {
                                ec = true;
                            }
                        }
                    }
                }
                if ec {
                    expr.kind = ExpressionKind::Array(arena.alloc_slice_clone(&new_elements));
                    changed = true;
                }
            }

            ExpressionKind::Object(props) => {
                let mut new_props: Vec<_> = props.to_vec();
                let mut pc = false;
                for prop in &mut new_props {
                    match prop {
                        ObjectProperty::Property { key, value, span } => {
                            let mut new_val = (**value).clone();
                            if self.specialize_calls_in_expression(&mut new_val, arena) {
                                *prop = ObjectProperty::Property {
                                    key: key.clone(),
                                    value: arena.alloc(new_val),
                                    span: *span,
                                };
                                pc = true;
                            }
                        }
                        ObjectProperty::Computed { key, value, span } => {
                            let mut new_key = (**key).clone();
                            let mut new_val = (**value).clone();
                            let kc = self.specialize_calls_in_expression(&mut new_key, arena);
                            let vc = self.specialize_calls_in_expression(&mut new_val, arena);
                            if kc || vc {
                                *prop = ObjectProperty::Computed {
                                    key: arena.alloc(new_key),
                                    value: arena.alloc(new_val),
                                    span: *span,
                                };
                                pc = true;
                            }
                        }
                        ObjectProperty::Spread { value, span } => {
                            let mut new_val = (**value).clone();
                            if self.specialize_calls_in_expression(&mut new_val, arena) {
                                *prop = ObjectProperty::Spread {
                                    value: arena.alloc(new_val),
                                    span: *span,
                                };
                                pc = true;
                            }
                        }
                    }
                }
                if pc {
                    expr.kind = ExpressionKind::Object(arena.alloc_slice_clone(&new_props));
                    changed = true;
                }
            }

            ExpressionKind::Conditional(cond, then_expr, else_expr) => {
                let mut new_cond = (**cond).clone();
                let mut new_then = (**then_expr).clone();
                let mut new_else = (**else_expr).clone();
                let cc = self.specialize_calls_in_expression(&mut new_cond, arena);
                let tc = self.specialize_calls_in_expression(&mut new_then, arena);
                let ec = self.specialize_calls_in_expression(&mut new_else, arena);
                if cc || tc || ec {
                    expr.kind = ExpressionKind::Conditional(
                        arena.alloc(new_cond),
                        arena.alloc(new_then),
                        arena.alloc(new_else),
                    );
                    changed = true;
                }
            }

            ExpressionKind::Pipe(left, right) => {
                let mut new_left = (**left).clone();
                let mut new_right = (**right).clone();
                let lc = self.specialize_calls_in_expression(&mut new_left, arena);
                let rc = self.specialize_calls_in_expression(&mut new_right, arena);
                if lc || rc {
                    expr.kind = ExpressionKind::Pipe(arena.alloc(new_left), arena.alloc(new_right));
                    changed = true;
                }
            }

            ExpressionKind::Parenthesized(inner) => {
                let mut new_inner = (**inner).clone();
                if self.specialize_calls_in_expression(&mut new_inner, arena) {
                    expr.kind = ExpressionKind::Parenthesized(arena.alloc(new_inner));
                    changed = true;
                }
            }

            ExpressionKind::TypeAssertion(inner, ty) => {
                let ty = ty.clone();
                let mut new_inner = (**inner).clone();
                if self.specialize_calls_in_expression(&mut new_inner, arena) {
                    expr.kind = ExpressionKind::TypeAssertion(arena.alloc(new_inner), ty);
                    changed = true;
                }
            }

            ExpressionKind::OptionalCall(callee, args, type_args)
            | ExpressionKind::OptionalMethodCall(callee, _, args, type_args) => {
                // Capture discriminant info before borrow
                let is_method_call = matches!(&expr.kind, ExpressionKind::OptionalMethodCall(..));
                let method = if let ExpressionKind::OptionalMethodCall(_, m, _, _) = &expr.kind {
                    Some(m.clone())
                } else {
                    None
                };
                let type_args = *type_args;
                let mut new_callee = (**callee).clone();
                let mut new_args: Vec<_> = args.to_vec();
                let mut sub_changed = false;
                if self.specialize_calls_in_expression(&mut new_callee, arena) {
                    sub_changed = true;
                }
                for arg in &mut new_args {
                    if self.specialize_calls_in_expression(&mut arg.value, arena) {
                        sub_changed = true;
                    }
                }
                if sub_changed {
                    if is_method_call {
                        expr.kind = ExpressionKind::OptionalMethodCall(
                            arena.alloc(new_callee),
                            method.unwrap(),
                            arena.alloc_slice_clone(&new_args),
                            type_args,
                        );
                    } else {
                        expr.kind = ExpressionKind::OptionalCall(
                            arena.alloc(new_callee),
                            arena.alloc_slice_clone(&new_args),
                            type_args,
                        );
                    }
                    changed = true;
                }
            }

            ExpressionKind::OptionalMember(obj, member) => {
                let member = member.clone();
                let mut new_obj = (**obj).clone();
                if self.specialize_calls_in_expression(&mut new_obj, arena) {
                    expr.kind = ExpressionKind::OptionalMember(arena.alloc(new_obj), member);
                    changed = true;
                }
            }

            ExpressionKind::OptionalIndex(obj, index) => {
                let mut new_obj = (**obj).clone();
                let mut new_index = (**index).clone();
                let oc = self.specialize_calls_in_expression(&mut new_obj, arena);
                let ic = self.specialize_calls_in_expression(&mut new_index, arena);
                if oc || ic {
                    expr.kind =
                        ExpressionKind::OptionalIndex(arena.alloc(new_obj), arena.alloc(new_index));
                    changed = true;
                }
            }

            ExpressionKind::New(callee, args, type_args) => {
                let type_args = *type_args;
                let mut new_callee = (**callee).clone();
                let mut new_args: Vec<_> = args.to_vec();
                let mut sub_changed = false;
                if self.specialize_calls_in_expression(&mut new_callee, arena) {
                    sub_changed = true;
                }
                for arg in &mut new_args {
                    if self.specialize_calls_in_expression(&mut arg.value, arena) {
                        sub_changed = true;
                    }
                }
                if sub_changed {
                    expr.kind = ExpressionKind::New(
                        arena.alloc(new_callee),
                        arena.alloc_slice_clone(&new_args),
                        type_args,
                    );
                    changed = true;
                }
            }

            ExpressionKind::ErrorChain(left, right) => {
                let mut new_left = (**left).clone();
                let mut new_right = (**right).clone();
                let lc = self.specialize_calls_in_expression(&mut new_left, arena);
                let rc = self.specialize_calls_in_expression(&mut new_right, arena);
                if lc || rc {
                    expr.kind =
                        ExpressionKind::ErrorChain(arena.alloc(new_left), arena.alloc(new_right));
                    changed = true;
                }
            }

            // Literals, identifiers, self, super - no calls to specialize
            _ => {}
        }

        changed
    }
}

impl<'arena> WholeProgramPass<'arena> for GenericSpecializationPass {
    fn name(&self) -> &'static str {
        "generic-specialization"
    }

    fn min_level(&self) -> OptimizationLevel {
        OptimizationLevel::O3
    }

    fn run(
        &mut self,
        program: &mut MutableProgram<'arena>,
        arena: &'arena Bump,
    ) -> Result<bool, String> {
        // Reset state for fresh run
        self.specializations.clear();
        self.next_spec_id = 0;

        // Create a temporary context that holds arena-lifetime data.
        // This pattern keeps arena-allocated AST nodes out of `self`, allowing
        // the pass to persist across compilations while AST data is safely scoped.
        let mut ctx = SpecializationContext::new(self);

        // Phase 1: Collect all generic function declarations
        ctx.collect_generic_functions(program);

        if ctx.generic_functions.is_empty() {
            return Ok(false);
        }

        // Phase 2: Find and specialize call sites
        let mut changed = false;
        for stmt in &mut program.statements {
            if ctx.specialize_calls_in_statement(stmt, arena) {
                changed = true;
            }
        }

        // Phase 3: Add specialized functions to the program
        // Insert them after the original function declarations, not at the end
        // (to avoid being removed by dead code elimination after return statements)
        if !ctx.new_functions.is_empty() {
            // Find the last function statement index
            let mut insert_idx = 0;
            for (i, stmt) in program.statements.iter().enumerate() {
                if matches!(stmt, Statement::Function(_)) {
                    insert_idx = i + 1;
                }
            }
            // Insert new functions at that position
            for (i, func) in ctx.new_functions.drain(..).enumerate() {
                program.statements.insert(insert_idx + i, func);
            }
            changed = true;
        }

        Ok(changed)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
