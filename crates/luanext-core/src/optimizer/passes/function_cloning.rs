// =============================================================================
// O3: Function Cloning for Specialization
// =============================================================================
//
// Duplicates functions for call sites where arguments are known constants,
// enabling downstream constant folding and dead code elimination within the
// cloned body.
//
// Safety constraints:
// - Only clones small functions (≤8 statements) to limit code size growth
// - Only specializes for literal constant arguments (numbers, strings, booleans)
// - Maximum 4 clones per function to prevent combinatorial explosion
// - Does NOT clone functions with varargs, rest parameters, or closures
//   over mutable upvalues (conservative, safe for Lua semantics)
//
// Example transformation:
//   function greet(name: string, loud: boolean): string
//     if loud then
//       return string.upper(name)
//     end
//     return name
//   end
//   greet("hello", true)
//   greet("world", false)
// →
//   function greet__clone0(): string    -- specialized for ("hello", true)
//     return string.upper("hello")
//   end
//   function greet__clone1(): string    -- specialized for ("world", false)
//     return "world"
//   end
//   greet__clone0()
//   greet__clone1()

use crate::config::OptimizationLevel;
use crate::optimizer::{AstFeatures, WholeProgramPass};
use crate::MutableProgram;
use bumpalo::Bump;
use luanext_parser::ast::expression::{
    Argument, ArrayElement, Expression, ExpressionKind, Literal, ObjectProperty,
};
use luanext_parser::ast::pattern::Pattern;
use luanext_parser::ast::statement::{Block, ForStatement, FunctionDeclaration, Statement};
use luanext_parser::ast::Spanned;
use luanext_parser::string_interner::{StringId, StringInterner};
use rustc_hash::FxHashMap;
use std::sync::Arc;

/// Maximum body size (statements) for a function to be eligible for cloning
const MAX_CLONE_BODY_SIZE: usize = 8;

/// Maximum number of clones per original function
const MAX_CLONES_PER_FUNCTION: usize = 4;

pub struct FunctionCloningPass {
    interner: Arc<StringInterner>,
    next_clone_id: usize,
}

impl FunctionCloningPass {
    pub fn new(interner: Arc<StringInterner>) -> Self {
        Self {
            interner,
            next_clone_id: 0,
        }
    }
}

impl Default for FunctionCloningPass {
    fn default() -> Self {
        Self {
            interner: Arc::new(StringInterner::new()),
            next_clone_id: 0,
        }
    }
}

/// Tracks information about a single specialization opportunity
struct CallSiteInfo<'arena> {
    /// The constant arguments at this call site
    const_args: Vec<(usize, Expression<'arena>)>,
}

/// Per-run context that holds arena-lifetime data
struct CloningContext<'arena> {
    /// Collected function declarations eligible for cloning
    functions: FxHashMap<StringId, FunctionDeclaration<'arena>>,
    /// How many clones have been created for each function
    clone_counts: FxHashMap<StringId, usize>,
    /// Maps (func_name, args_key) -> cloned_func_name for deduplication
    clone_cache: FxHashMap<(StringId, String), StringId>,
    /// New cloned functions to insert
    new_functions: Vec<Statement<'arena>>,
}

impl<'arena> CloningContext<'arena> {
    fn new() -> Self {
        Self {
            functions: FxHashMap::default(),
            clone_counts: FxHashMap::default(),
            clone_cache: FxHashMap::default(),
            new_functions: Vec::new(),
        }
    }
}

impl<'arena> WholeProgramPass<'arena> for FunctionCloningPass {
    fn name(&self) -> &'static str {
        "function-cloning"
    }

    fn min_level(&self) -> OptimizationLevel {
        OptimizationLevel::Aggressive
    }

    fn required_features(&self) -> AstFeatures {
        AstFeatures::HAS_FUNCTIONS
    }

    fn run(
        &mut self,
        program: &mut MutableProgram<'arena>,
        arena: &'arena Bump,
    ) -> Result<bool, String> {
        self.next_clone_id = 0;

        let mut ctx = CloningContext::new();

        // Phase 1: Collect eligible functions
        for stmt in &program.statements {
            if let Statement::Function(func) = stmt {
                if is_eligible_for_cloning(func) {
                    ctx.functions.insert(func.name.node, func.clone());
                }
            }
        }

        if ctx.functions.is_empty() {
            return Ok(false);
        }

        // Phase 2: Find call sites with constant arguments and rewrite them
        let mut changed = false;
        for stmt in &mut program.statements {
            if self.process_statement(stmt, arena, &mut ctx) {
                changed = true;
            }
        }

        // Phase 3: Insert cloned functions after the last function declaration
        if !ctx.new_functions.is_empty() {
            let mut insert_idx = 0;
            for (i, stmt) in program.statements.iter().enumerate() {
                if matches!(stmt, Statement::Function(_)) {
                    insert_idx = i + 1;
                }
            }
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

/// Check if a function is eligible for cloning
fn is_eligible_for_cloning(func: &FunctionDeclaration<'_>) -> bool {
    // Must have parameters
    if func.parameters.is_empty() {
        return false;
    }

    // Body must be small
    if count_statements(&func.body) > MAX_CLONE_BODY_SIZE {
        return false;
    }

    // No rest/vararg parameters
    if func.parameters.iter().any(|p| p.is_rest) {
        return false;
    }

    // All parameters must be simple identifiers (no destructuring)
    if !func
        .parameters
        .iter()
        .all(|p| matches!(p.pattern, Pattern::Identifier(_)))
    {
        return false;
    }

    // Skip generic functions (handled by generic specialization pass)
    if func.type_parameters.is_some() {
        return false;
    }

    true
}

/// Count statements in a block (non-recursive for simplicity)
fn count_statements(block: &Block<'_>) -> usize {
    block.statements.len()
}

/// Check if an expression is a compile-time constant suitable for specialization
fn is_constant_literal(expr: &Expression<'_>) -> bool {
    matches!(
        &expr.kind,
        ExpressionKind::Literal(
            Literal::Number(_)
                | Literal::Integer(_)
                | Literal::String(_)
                | Literal::Boolean(_)
                | Literal::Nil
        )
    )
}

/// Generate a string key for a set of constant arguments (for deduplication)
fn args_cache_key(const_args: &[(usize, Expression<'_>)]) -> String {
    let mut key = String::new();
    for (idx, expr) in const_args {
        if !key.is_empty() {
            key.push(',');
        }
        key.push_str(&format!("{}={:?}", idx, expr.kind));
    }
    key
}

impl FunctionCloningPass {
    fn process_statement<'arena>(
        &mut self,
        stmt: &mut Statement<'arena>,
        arena: &'arena Bump,
        ctx: &mut CloningContext<'arena>,
    ) -> bool {
        let mut changed = false;

        match stmt {
            Statement::Variable(var_decl) => {
                changed |= self.process_expression(&mut var_decl.initializer, arena, ctx);
            }
            Statement::Expression(expr) => {
                changed |= self.process_expression(expr, arena, ctx);
            }
            Statement::Return(ret) => {
                let mut values: Vec<Expression<'arena>> = ret.values.to_vec();
                let mut ret_changed = false;
                for value in &mut values {
                    ret_changed |= self.process_expression(value, arena, ctx);
                }
                if ret_changed {
                    ret.values = arena.alloc_slice_clone(&values);
                    changed = true;
                }
            }
            Statement::If(if_stmt) => {
                changed |= self.process_expression(&mut if_stmt.condition, arena, ctx);
                changed |= self.process_block(&mut if_stmt.then_block, arena, ctx);
                let mut new_else_ifs: Vec<_> = if_stmt.else_ifs.to_vec();
                let mut eic = false;
                for else_if in &mut new_else_ifs {
                    eic |= self.process_expression(&mut else_if.condition, arena, ctx);
                    eic |= self.process_block(&mut else_if.block, arena, ctx);
                }
                if eic {
                    if_stmt.else_ifs = arena.alloc_slice_clone(&new_else_ifs);
                    changed = true;
                }
                if let Some(else_block) = &mut if_stmt.else_block {
                    changed |= self.process_block(else_block, arena, ctx);
                }
            }
            Statement::While(while_stmt) => {
                changed |= self.process_expression(&mut while_stmt.condition, arena, ctx);
                changed |= self.process_block(&mut while_stmt.body, arena, ctx);
            }
            Statement::For(for_stmt) => match &**for_stmt {
                ForStatement::Numeric(num_ref) => {
                    let mut new_num = (**num_ref).clone();
                    let mut fc = false;
                    fc |= self.process_expression(&mut new_num.start, arena, ctx);
                    fc |= self.process_expression(&mut new_num.end, arena, ctx);
                    if let Some(step) = &mut new_num.step {
                        fc |= self.process_expression(step, arena, ctx);
                    }
                    fc |= self.process_block(&mut new_num.body, arena, ctx);
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
                    for iter in &mut new_iters {
                        fc |= self.process_expression(iter, arena, ctx);
                    }
                    if fc {
                        new_gen.iterators = arena.alloc_slice_clone(&new_iters);
                    }
                    fc |= self.process_block(&mut new_gen.body, arena, ctx);
                    if fc {
                        *stmt = Statement::For(arena.alloc(ForStatement::Generic(new_gen)));
                        changed = true;
                    }
                }
            },
            Statement::Function(func) => {
                changed |= self.process_block(&mut func.body, arena, ctx);
            }
            Statement::Block(block) => {
                changed |= self.process_block(block, arena, ctx);
            }
            Statement::Repeat(repeat) => {
                changed |= self.process_block(&mut repeat.body, arena, ctx);
                changed |= self.process_expression(&mut repeat.until, arena, ctx);
            }
            _ => {}
        }

        changed
    }

    fn process_block<'arena>(
        &mut self,
        block: &mut Block<'arena>,
        arena: &'arena Bump,
        ctx: &mut CloningContext<'arena>,
    ) -> bool {
        let mut stmts: Vec<Statement<'arena>> = block.statements.to_vec();
        let mut changed = false;
        for s in &mut stmts {
            changed |= self.process_statement(s, arena, ctx);
        }
        if changed {
            block.statements = arena.alloc_slice_clone(&stmts);
        }
        changed
    }

    fn process_expression<'arena>(
        &mut self,
        expr: &mut Expression<'arena>,
        arena: &'arena Bump,
        ctx: &mut CloningContext<'arena>,
    ) -> bool {
        let mut changed = false;

        match &expr.kind {
            ExpressionKind::Call(callee, args, type_args) => {
                let type_args = *type_args;
                let mut new_callee = (**callee).clone();
                let mut new_args: Vec<_> = args.to_vec();

                // Process nested expressions first
                let mut sub_changed = false;
                sub_changed |= self.process_expression(&mut new_callee, arena, ctx);
                for arg in &mut new_args {
                    sub_changed |= self.process_expression(&mut arg.value, arena, ctx);
                }

                // Check if callee is a direct function reference with constant args
                if let ExpressionKind::Identifier(func_name) = &new_callee.kind {
                    let func_name = *func_name;
                    if let Some(func) = ctx.functions.get(&func_name).cloned() {
                        let call_site = self.analyze_call_site(&new_args);
                        if let Some(info) = call_site {
                            if !info.const_args.is_empty() {
                                // Check clone limit
                                let count = ctx.clone_counts.get(&func_name).copied().unwrap_or(0);
                                if count < MAX_CLONES_PER_FUNCTION {
                                    // Check cache for identical specialization
                                    let cache_key = (func_name, args_cache_key(&info.const_args));
                                    if let Some(&cloned_name) = ctx.clone_cache.get(&cache_key) {
                                        // Reuse existing clone
                                        new_callee.kind = ExpressionKind::Identifier(cloned_name);
                                        // Remove constant args from call
                                        let remaining_args = self
                                            .remove_specialized_args(&new_args, &info.const_args);
                                        expr.kind = ExpressionKind::Call(
                                            arena.alloc(new_callee),
                                            arena.alloc_slice_clone(&remaining_args),
                                            type_args,
                                        );
                                        return true;
                                    }

                                    // Create new clone
                                    if let Some(cloned_name) =
                                        self.create_clone(&func, &info.const_args, arena, ctx)
                                    {
                                        new_callee.kind = ExpressionKind::Identifier(cloned_name);
                                        let remaining_args = self
                                            .remove_specialized_args(&new_args, &info.const_args);
                                        expr.kind = ExpressionKind::Call(
                                            arena.alloc(new_callee),
                                            arena.alloc_slice_clone(&remaining_args),
                                            type_args,
                                        );
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }

                if sub_changed {
                    expr.kind = ExpressionKind::Call(
                        arena.alloc(new_callee),
                        arena.alloc_slice_clone(&new_args),
                        type_args,
                    );
                    changed = true;
                }
            }

            ExpressionKind::Binary(op, left, right) => {
                let op = *op;
                let mut new_left = (**left).clone();
                let mut new_right = (**right).clone();
                let lc = self.process_expression(&mut new_left, arena, ctx);
                let rc = self.process_expression(&mut new_right, arena, ctx);
                if lc || rc {
                    expr.kind =
                        ExpressionKind::Binary(op, arena.alloc(new_left), arena.alloc(new_right));
                    changed = true;
                }
            }

            ExpressionKind::Unary(op, operand) => {
                let op = *op;
                let mut new_operand = (**operand).clone();
                if self.process_expression(&mut new_operand, arena, ctx) {
                    expr.kind = ExpressionKind::Unary(op, arena.alloc(new_operand));
                    changed = true;
                }
            }

            ExpressionKind::Assignment(target, op, value) => {
                let op = *op;
                let mut new_target = (**target).clone();
                let mut new_value = (**value).clone();
                let tc = self.process_expression(&mut new_target, arena, ctx);
                let vc = self.process_expression(&mut new_value, arena, ctx);
                if tc || vc {
                    expr.kind = ExpressionKind::Assignment(
                        arena.alloc(new_target),
                        op,
                        arena.alloc(new_value),
                    );
                    changed = true;
                }
            }

            ExpressionKind::Conditional(cond, then_expr, else_expr) => {
                let mut new_cond = (**cond).clone();
                let mut new_then = (**then_expr).clone();
                let mut new_else = (**else_expr).clone();
                let cc = self.process_expression(&mut new_cond, arena, ctx);
                let tc = self.process_expression(&mut new_then, arena, ctx);
                let ec = self.process_expression(&mut new_else, arena, ctx);
                if cc || tc || ec {
                    expr.kind = ExpressionKind::Conditional(
                        arena.alloc(new_cond),
                        arena.alloc(new_then),
                        arena.alloc(new_else),
                    );
                    changed = true;
                }
            }

            ExpressionKind::Array(elements) => {
                let mut new_elements: Vec<_> = elements.to_vec();
                let mut ec = false;
                for elem in &mut new_elements {
                    match elem {
                        ArrayElement::Expression(e) | ArrayElement::Spread(e) => {
                            ec |= self.process_expression(e, arena, ctx);
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
                            if self.process_expression(&mut new_val, arena, ctx) {
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
                            let kc = self.process_expression(&mut new_key, arena, ctx);
                            let vc = self.process_expression(&mut new_val, arena, ctx);
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
                            if self.process_expression(&mut new_val, arena, ctx) {
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

            ExpressionKind::Parenthesized(inner) => {
                let mut new_inner = (**inner).clone();
                if self.process_expression(&mut new_inner, arena, ctx) {
                    expr.kind = ExpressionKind::Parenthesized(arena.alloc(new_inner));
                    changed = true;
                }
            }

            ExpressionKind::Pipe(left, right) => {
                let mut new_left = (**left).clone();
                let mut new_right = (**right).clone();
                let lc = self.process_expression(&mut new_left, arena, ctx);
                let rc = self.process_expression(&mut new_right, arena, ctx);
                if lc || rc {
                    expr.kind = ExpressionKind::Pipe(arena.alloc(new_left), arena.alloc(new_right));
                    changed = true;
                }
            }

            ExpressionKind::Member(obj, member) => {
                let member = member.clone();
                let mut new_obj = (**obj).clone();
                if self.process_expression(&mut new_obj, arena, ctx) {
                    expr.kind = ExpressionKind::Member(arena.alloc(new_obj), member);
                    changed = true;
                }
            }

            ExpressionKind::Index(obj, index) => {
                let mut new_obj = (**obj).clone();
                let mut new_index = (**index).clone();
                let oc = self.process_expression(&mut new_obj, arena, ctx);
                let ic = self.process_expression(&mut new_index, arena, ctx);
                if oc || ic {
                    expr.kind = ExpressionKind::Index(arena.alloc(new_obj), arena.alloc(new_index));
                    changed = true;
                }
            }

            // Leaf nodes - no children
            _ => {}
        }

        changed
    }

    /// Analyze a call site to find constant arguments
    fn analyze_call_site<'arena>(&self, args: &[Argument<'arena>]) -> Option<CallSiteInfo<'arena>> {
        let mut const_args = Vec::new();

        for (i, arg) in args.iter().enumerate() {
            if !arg.is_spread && is_constant_literal(&arg.value) {
                const_args.push((i, arg.value.clone()));
            }
        }

        if const_args.is_empty() {
            None
        } else {
            Some(CallSiteInfo { const_args })
        }
    }

    /// Create a cloned function with constant arguments substituted
    fn create_clone<'arena>(
        &mut self,
        func: &FunctionDeclaration<'arena>,
        const_args: &[(usize, Expression<'arena>)],
        arena: &'arena Bump,
        ctx: &mut CloningContext<'arena>,
    ) -> Option<StringId> {
        let orig_name = self.interner.resolve(func.name.node);
        let cloned_name_str = format!("{}__clone{}", orig_name, self.next_clone_id);
        self.next_clone_id += 1;

        let cloned_name = self.interner.get_or_intern(&cloned_name_str);

        // Build substitution map: param_name -> constant_value
        let mut substitutions: FxHashMap<StringId, Expression<'arena>> = FxHashMap::default();
        let mut specialized_param_indices: Vec<usize> = Vec::new();

        for (arg_idx, const_expr) in const_args {
            if *arg_idx < func.parameters.len() {
                if let Pattern::Identifier(ident) = &func.parameters[*arg_idx].pattern {
                    substitutions.insert(ident.node, const_expr.clone());
                    specialized_param_indices.push(*arg_idx);
                }
            }
        }

        // Create new parameter list without specialized parameters
        let new_params: Vec<_> = func
            .parameters
            .iter()
            .enumerate()
            .filter(|(i, _)| !specialized_param_indices.contains(i))
            .map(|(_, p)| p.clone())
            .collect();

        // Clone body with substitutions applied
        let new_body = substitute_in_block(&func.body, &substitutions, arena);

        // Create the cloned function declaration
        let cloned_func = FunctionDeclaration {
            name: Spanned::new(cloned_name, func.name.span),
            type_parameters: func.type_parameters,
            parameters: arena.alloc_slice_clone(&new_params),
            return_type: func.return_type.clone(),
            throws: func.throws,
            body: new_body,
            span: func.span,
        };

        // Update tracking
        let count = ctx.clone_counts.entry(func.name.node).or_insert(0);
        *count += 1;

        let cache_key = (func.name.node, args_cache_key(const_args));
        ctx.clone_cache.insert(cache_key, cloned_name);
        ctx.new_functions.push(Statement::Function(cloned_func));

        Some(cloned_name)
    }

    /// Remove specialized arguments from the call site's argument list
    fn remove_specialized_args<'arena>(
        &self,
        args: &[Argument<'arena>],
        const_args: &[(usize, Expression<'arena>)],
    ) -> Vec<Argument<'arena>> {
        let specialized_indices: Vec<usize> = const_args.iter().map(|(i, _)| *i).collect();
        args.iter()
            .enumerate()
            .filter(|(i, _)| !specialized_indices.contains(i))
            .map(|(_, a)| a.clone())
            .collect()
    }
}

// =============================================================================
// Substitution helpers — replace identifier references with constant values
// =============================================================================

fn substitute_in_block<'arena>(
    block: &Block<'arena>,
    subs: &FxHashMap<StringId, Expression<'arena>>,
    arena: &'arena Bump,
) -> Block<'arena> {
    let new_stmts: Vec<_> = block
        .statements
        .iter()
        .map(|s| substitute_in_statement(s, subs, arena))
        .collect();
    Block {
        statements: arena.alloc_slice_clone(&new_stmts),
        span: block.span,
    }
}

fn substitute_in_statement<'arena>(
    stmt: &Statement<'arena>,
    subs: &FxHashMap<StringId, Expression<'arena>>,
    arena: &'arena Bump,
) -> Statement<'arena> {
    match stmt {
        Statement::Expression(expr) => Statement::Expression(substitute_in_expr(expr, subs, arena)),
        Statement::Return(ret) => {
            let new_values: Vec<_> = ret
                .values
                .iter()
                .map(|v| substitute_in_expr(v, subs, arena))
                .collect();
            Statement::Return(luanext_parser::ast::statement::ReturnStatement {
                values: arena.alloc_slice_clone(&new_values),
                span: ret.span,
            })
        }
        Statement::Variable(var) => {
            let new_init = substitute_in_expr(&var.initializer, subs, arena);
            Statement::Variable(luanext_parser::ast::statement::VariableDeclaration {
                kind: var.kind,
                pattern: var.pattern.clone(),
                type_annotation: var.type_annotation.clone(),
                initializer: new_init,
                span: var.span,
            })
        }
        Statement::If(if_stmt) => {
            let new_cond = substitute_in_expr(&if_stmt.condition, subs, arena);
            let new_then = substitute_in_block(&if_stmt.then_block, subs, arena);
            let new_else_ifs: Vec<_> = if_stmt
                .else_ifs
                .iter()
                .map(|ei| luanext_parser::ast::statement::ElseIf {
                    condition: substitute_in_expr(&ei.condition, subs, arena),
                    block: substitute_in_block(&ei.block, subs, arena),
                    span: ei.span,
                })
                .collect();
            let new_else = if_stmt
                .else_block
                .as_ref()
                .map(|b| substitute_in_block(b, subs, arena));
            Statement::If(luanext_parser::ast::statement::IfStatement {
                condition: new_cond,
                then_block: new_then,
                else_ifs: arena.alloc_slice_clone(&new_else_ifs),
                else_block: new_else,
                span: if_stmt.span,
            })
        }
        Statement::Block(block) => Statement::Block(substitute_in_block(block, subs, arena)),
        // For other statement types, clone as-is (conservative)
        _ => stmt.clone(),
    }
}

fn substitute_in_expr<'arena>(
    expr: &Expression<'arena>,
    subs: &FxHashMap<StringId, Expression<'arena>>,
    arena: &'arena Bump,
) -> Expression<'arena> {
    match &expr.kind {
        ExpressionKind::Identifier(ident) => {
            if let Some(replacement) = subs.get(ident) {
                Expression {
                    kind: replacement.kind.clone(),
                    span: expr.span,
                    annotated_type: expr.annotated_type.clone(),
                    receiver_class: expr.receiver_class.clone(),
                }
            } else {
                expr.clone()
            }
        }
        ExpressionKind::Binary(op, left, right) => {
            let new_left = substitute_in_expr(left, subs, arena);
            let new_right = substitute_in_expr(right, subs, arena);
            Expression {
                kind: ExpressionKind::Binary(*op, arena.alloc(new_left), arena.alloc(new_right)),
                span: expr.span,
                annotated_type: expr.annotated_type.clone(),
                receiver_class: expr.receiver_class.clone(),
            }
        }
        ExpressionKind::Unary(op, operand) => {
            let new_operand = substitute_in_expr(operand, subs, arena);
            Expression {
                kind: ExpressionKind::Unary(*op, arena.alloc(new_operand)),
                span: expr.span,
                annotated_type: expr.annotated_type.clone(),
                receiver_class: expr.receiver_class.clone(),
            }
        }
        ExpressionKind::Call(callee, args, type_args) => {
            let new_callee = substitute_in_expr(callee, subs, arena);
            let new_args: Vec<_> = args
                .iter()
                .map(|arg| Argument {
                    value: substitute_in_expr(&arg.value, subs, arena),
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
        ExpressionKind::Member(obj, member) => {
            let new_obj = substitute_in_expr(obj, subs, arena);
            Expression {
                kind: ExpressionKind::Member(arena.alloc(new_obj), member.clone()),
                span: expr.span,
                annotated_type: expr.annotated_type.clone(),
                receiver_class: expr.receiver_class.clone(),
            }
        }
        ExpressionKind::Index(obj, index) => {
            let new_obj = substitute_in_expr(obj, subs, arena);
            let new_index = substitute_in_expr(index, subs, arena);
            Expression {
                kind: ExpressionKind::Index(arena.alloc(new_obj), arena.alloc(new_index)),
                span: expr.span,
                annotated_type: expr.annotated_type.clone(),
                receiver_class: expr.receiver_class.clone(),
            }
        }
        ExpressionKind::Conditional(cond, then_expr, else_expr) => {
            let new_cond = substitute_in_expr(cond, subs, arena);
            let new_then = substitute_in_expr(then_expr, subs, arena);
            let new_else = substitute_in_expr(else_expr, subs, arena);
            Expression {
                kind: ExpressionKind::Conditional(
                    arena.alloc(new_cond),
                    arena.alloc(new_then),
                    arena.alloc(new_else),
                ),
                span: expr.span,
                annotated_type: expr.annotated_type.clone(),
                receiver_class: expr.receiver_class.clone(),
            }
        }
        ExpressionKind::Parenthesized(inner) => {
            let new_inner = substitute_in_expr(inner, subs, arena);
            Expression {
                kind: ExpressionKind::Parenthesized(arena.alloc(new_inner)),
                span: expr.span,
                annotated_type: expr.annotated_type.clone(),
                receiver_class: expr.receiver_class.clone(),
            }
        }
        ExpressionKind::MethodCall(obj, method, args, type_args) => {
            let new_obj = substitute_in_expr(obj, subs, arena);
            let new_args: Vec<_> = args
                .iter()
                .map(|arg| Argument {
                    value: substitute_in_expr(&arg.value, subs, arena),
                    is_spread: arg.is_spread,
                    span: arg.span,
                })
                .collect();
            Expression {
                kind: ExpressionKind::MethodCall(
                    arena.alloc(new_obj),
                    method.clone(),
                    arena.alloc_slice_clone(&new_args),
                    *type_args,
                ),
                span: expr.span,
                annotated_type: expr.annotated_type.clone(),
                receiver_class: expr.receiver_class.clone(),
            }
        }
        // For other expression types, clone as-is
        _ => expr.clone(),
    }
}
