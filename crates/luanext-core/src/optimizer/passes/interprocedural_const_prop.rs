// =============================================================================
// O3: Interprocedural Constant Propagation
// =============================================================================
//
// Propagates constant values across function boundaries by analyzing all call
// sites for each function. When ALL call sites pass the same constant value
// for a parameter, that parameter is replaced with the constant inside the
// function body.
//
// This pass performs fixed-point iteration over the call graph:
// 1. Collect all function declarations and their call sites
// 2. For each parameter of each function, check if all callers provide
//    the same constant value
// 3. If so, substitute the constant into the function body and remove
//    the parameter from both declaration and all call sites
// 4. Repeat until no more changes (handles chains of constant propagation)
//
// Safety constraints:
// - Only propagates literal constants (numbers, strings, booleans, nil)
// - Skips functions with rest/vararg parameters
// - Skips functions called with spread arguments
// - Maximum 3 fixed-point iterations to prevent pathological cases
// - Only processes simple identifier parameters (no destructuring)
//
// Example transformation:
//   function multiply(x: number, factor: number): number
//     return x * factor
//   end
//   multiply(10, 2)
//   multiply(20, 2)
//   multiply(30, 2)
// →
//   function multiply(x: number): number
//     return x * 2
//   end
//   multiply(10)
//   multiply(20)
//   multiply(30)

use crate::config::OptimizationLevel;
use crate::optimizer::{AstFeatures, WholeProgramPass};
use crate::MutableProgram;
use bumpalo::Bump;
use luanext_parser::ast::expression::{
    Argument, ArrayElement, Expression, ExpressionKind, Literal, ObjectProperty,
};
use luanext_parser::ast::pattern::Pattern;
use luanext_parser::ast::statement::{Block, ForStatement, FunctionDeclaration, Statement};
use luanext_parser::string_interner::{StringId, StringInterner};
use rustc_hash::FxHashMap;
use std::sync::Arc;

/// Maximum fixed-point iterations
const MAX_ITERATIONS: usize = 3;

pub struct InterproceduralConstPropPass {
    _interner: Arc<StringInterner>,
}

impl InterproceduralConstPropPass {
    pub fn new(interner: Arc<StringInterner>) -> Self {
        Self {
            _interner: interner,
        }
    }
}

impl Default for InterproceduralConstPropPass {
    fn default() -> Self {
        Self {
            _interner: Arc::new(StringInterner::new()),
        }
    }
}

/// Represents a constant value observed at all call sites for a parameter
#[derive(Clone, Debug, PartialEq)]
enum ConstValue {
    /// All call sites pass the same integer
    Integer(i64),
    /// All call sites pass the same float
    Number(f64),
    /// All call sites pass the same string
    String(String),
    /// All call sites pass the same boolean
    Boolean(bool),
    /// All call sites pass nil
    Nil,
    /// Call sites pass different values (not constant)
    Varying,
}

impl ConstValue {
    fn from_expr(expr: &Expression<'_>) -> Option<Self> {
        match &expr.kind {
            ExpressionKind::Literal(Literal::Integer(n)) => Some(ConstValue::Integer(*n)),
            ExpressionKind::Literal(Literal::Number(n)) => Some(ConstValue::Number(*n)),
            ExpressionKind::Literal(Literal::String(s)) => Some(ConstValue::String(s.clone())),
            ExpressionKind::Literal(Literal::Boolean(b)) => Some(ConstValue::Boolean(*b)),
            ExpressionKind::Literal(Literal::Nil) => Some(ConstValue::Nil),
            _ => None,
        }
    }

    fn merge(&self, other: &ConstValue) -> ConstValue {
        if self == other {
            self.clone()
        } else {
            ConstValue::Varying
        }
    }

    fn to_expression_kind<'arena>(&self) -> ExpressionKind<'arena> {
        match self {
            ConstValue::Integer(n) => ExpressionKind::Literal(Literal::Integer(*n)),
            ConstValue::Number(n) => ExpressionKind::Literal(Literal::Number(*n)),
            ConstValue::String(s) => ExpressionKind::Literal(Literal::String(s.clone())),
            ConstValue::Boolean(b) => ExpressionKind::Literal(Literal::Boolean(*b)),
            ConstValue::Nil => ExpressionKind::Literal(Literal::Nil),
            ConstValue::Varying => unreachable!("Cannot convert Varying to expression"),
        }
    }
}

/// Information gathered about a function's call sites
struct FunctionInfo {
    /// For each parameter index, the merged constant value across all call sites
    param_constants: Vec<ConstValue>,
    /// Number of call sites found
    call_count: usize,
}

/// Per-run context holding analysis results
struct PropagationContext {
    /// Maps function name -> gathered information about its call sites
    function_info: FxHashMap<StringId, FunctionInfo>,
    /// Which parameters of which functions should be propagated
    /// Maps function name -> list of (param_index, constant_value)
    propagation_targets: FxHashMap<StringId, Vec<(usize, ConstValue)>>,
}

impl PropagationContext {
    fn new() -> Self {
        Self {
            function_info: FxHashMap::default(),
            propagation_targets: FxHashMap::default(),
        }
    }
}

impl<'arena> WholeProgramPass<'arena> for InterproceduralConstPropPass {
    fn name(&self) -> &'static str {
        "interprocedural-const-prop"
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
        let mut total_changed = false;

        for _iteration in 0..MAX_ITERATIONS {
            let mut ctx = PropagationContext::new();

            // Phase 1: Collect eligible function declarations
            let eligible_functions = collect_eligible_functions(program);

            if eligible_functions.is_empty() {
                break;
            }

            // Phase 2: Analyze all call sites to gather constant argument info
            for stmt in program.statements.iter() {
                analyze_call_sites_in_statement(stmt, &eligible_functions, &mut ctx);
            }

            // Phase 3: Determine which parameters can be propagated
            // A parameter is propagatable if:
            // - It has at least 1 call site
            // - ALL call sites provide the same constant value for it
            for (func_name, info) in &ctx.function_info {
                if info.call_count == 0 {
                    continue;
                }
                let mut targets = Vec::new();
                for (i, cv) in info.param_constants.iter().enumerate() {
                    if *cv != ConstValue::Varying {
                        targets.push((i, cv.clone()));
                    }
                }
                if !targets.is_empty() {
                    ctx.propagation_targets.insert(*func_name, targets);
                }
            }

            if ctx.propagation_targets.is_empty() {
                break;
            }

            // Phase 4: Apply propagation - modify function bodies and call sites
            let mut changed = false;

            // 4a: Substitute constants into function bodies and remove parameters
            for stmt in &mut program.statements {
                if let Statement::Function(func) = stmt {
                    if let Some(targets) = ctx.propagation_targets.get(&func.name.node) {
                        // Build substitution map
                        let mut subs: FxHashMap<StringId, ConstValue> = FxHashMap::default();
                        let mut remove_indices: Vec<usize> = Vec::new();

                        for (idx, cv) in targets {
                            if *idx < func.parameters.len() {
                                if let Pattern::Identifier(ident) = &func.parameters[*idx].pattern {
                                    subs.insert(ident.node, cv.clone());
                                    remove_indices.push(*idx);
                                }
                            }
                        }

                        if !subs.is_empty() {
                            // Substitute in body
                            let new_body = substitute_consts_in_block(&func.body, &subs, arena);
                            func.body = new_body;

                            // Remove propagated parameters
                            let new_params: Vec<_> = func
                                .parameters
                                .iter()
                                .enumerate()
                                .filter(|(i, _)| !remove_indices.contains(i))
                                .map(|(_, p)| p.clone())
                                .collect();
                            func.parameters = arena.alloc_slice_clone(&new_params);

                            changed = true;
                        }
                    }
                }
            }

            // 4b: Remove constant arguments from call sites
            for stmt in &mut program.statements {
                if rewrite_call_sites_in_statement(stmt, &ctx.propagation_targets, arena) {
                    changed = true;
                }
            }

            if !changed {
                break;
            }
            total_changed = true;
        }

        Ok(total_changed)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Collect function names that are eligible for interprocedural constant propagation
fn collect_eligible_functions(program: &MutableProgram<'_>) -> FxHashMap<StringId, usize> {
    let mut eligible = FxHashMap::default();
    for stmt in &program.statements {
        if let Statement::Function(func) = stmt {
            if is_eligible(func) {
                eligible.insert(func.name.node, func.parameters.len());
            }
        }
    }
    eligible
}

fn is_eligible(func: &FunctionDeclaration<'_>) -> bool {
    // Must have parameters
    if func.parameters.is_empty() {
        return false;
    }
    // No rest parameters
    if func.parameters.iter().any(|p| p.is_rest) {
        return false;
    }
    // All params must be simple identifiers
    if !func
        .parameters
        .iter()
        .all(|p| matches!(p.pattern, Pattern::Identifier(_)))
    {
        return false;
    }
    // Skip generic functions
    if func.type_parameters.is_some() {
        return false;
    }
    true
}

/// Analyze call sites in a statement to gather constant argument information
fn analyze_call_sites_in_statement(
    stmt: &Statement<'_>,
    eligible: &FxHashMap<StringId, usize>,
    ctx: &mut PropagationContext,
) {
    match stmt {
        Statement::Expression(expr) => {
            analyze_call_sites_in_expr(expr, eligible, ctx);
        }
        Statement::Variable(var) => {
            analyze_call_sites_in_expr(&var.initializer, eligible, ctx);
        }
        Statement::Return(ret) => {
            for v in ret.values.iter() {
                analyze_call_sites_in_expr(v, eligible, ctx);
            }
        }
        Statement::If(if_stmt) => {
            analyze_call_sites_in_expr(&if_stmt.condition, eligible, ctx);
            for s in if_stmt.then_block.statements.iter() {
                analyze_call_sites_in_statement(s, eligible, ctx);
            }
            for ei in if_stmt.else_ifs.iter() {
                analyze_call_sites_in_expr(&ei.condition, eligible, ctx);
                for s in ei.block.statements.iter() {
                    analyze_call_sites_in_statement(s, eligible, ctx);
                }
            }
            if let Some(eb) = &if_stmt.else_block {
                for s in eb.statements.iter() {
                    analyze_call_sites_in_statement(s, eligible, ctx);
                }
            }
        }
        Statement::While(while_stmt) => {
            analyze_call_sites_in_expr(&while_stmt.condition, eligible, ctx);
            for s in while_stmt.body.statements.iter() {
                analyze_call_sites_in_statement(s, eligible, ctx);
            }
        }
        Statement::For(for_stmt) => match &**for_stmt {
            ForStatement::Numeric(num) => {
                analyze_call_sites_in_expr(&num.start, eligible, ctx);
                analyze_call_sites_in_expr(&num.end, eligible, ctx);
                if let Some(step) = &num.step {
                    analyze_call_sites_in_expr(step, eligible, ctx);
                }
                for s in num.body.statements.iter() {
                    analyze_call_sites_in_statement(s, eligible, ctx);
                }
            }
            ForStatement::Generic(gen) => {
                for iter in gen.iterators.iter() {
                    analyze_call_sites_in_expr(iter, eligible, ctx);
                }
                for s in gen.body.statements.iter() {
                    analyze_call_sites_in_statement(s, eligible, ctx);
                }
            }
        },
        Statement::Function(func) => {
            for s in func.body.statements.iter() {
                analyze_call_sites_in_statement(s, eligible, ctx);
            }
        }
        Statement::Block(block) => {
            for s in block.statements.iter() {
                analyze_call_sites_in_statement(s, eligible, ctx);
            }
        }
        Statement::Repeat(repeat) => {
            for s in repeat.body.statements.iter() {
                analyze_call_sites_in_statement(s, eligible, ctx);
            }
            analyze_call_sites_in_expr(&repeat.until, eligible, ctx);
        }
        _ => {}
    }
}

fn analyze_call_sites_in_expr(
    expr: &Expression<'_>,
    eligible: &FxHashMap<StringId, usize>,
    ctx: &mut PropagationContext,
) {
    match &expr.kind {
        ExpressionKind::Call(callee, args, _type_args) => {
            // Recurse into sub-expressions
            analyze_call_sites_in_expr(callee, eligible, ctx);
            for arg in args.iter() {
                analyze_call_sites_in_expr(&arg.value, eligible, ctx);
            }

            // Check if this is a direct call to an eligible function
            if let ExpressionKind::Identifier(func_name) = &callee.kind {
                if let Some(&param_count) = eligible.get(func_name) {
                    // Skip calls with spread arguments
                    if args.iter().any(|a| a.is_spread) {
                        // Mark all params as varying
                        let info =
                            ctx.function_info
                                .entry(*func_name)
                                .or_insert_with(|| FunctionInfo {
                                    param_constants: vec![ConstValue::Varying; param_count],
                                    call_count: 0,
                                });
                        for cv in &mut info.param_constants {
                            *cv = ConstValue::Varying;
                        }
                        info.call_count += 1;
                        return;
                    }

                    let info =
                        ctx.function_info
                            .entry(*func_name)
                            .or_insert_with(|| FunctionInfo {
                                param_constants: vec![ConstValue::Varying; param_count],
                                call_count: 0,
                            });

                    // First call site initializes the constant values
                    if info.call_count == 0 {
                        for (i, cv) in info.param_constants.iter_mut().enumerate() {
                            if i < args.len() {
                                *cv = ConstValue::from_expr(&args[i].value)
                                    .unwrap_or(ConstValue::Varying);
                            } else {
                                // Missing argument - treat as nil
                                *cv = ConstValue::Nil;
                            }
                        }
                    } else {
                        // Subsequent calls: merge with existing
                        for (i, cv) in info.param_constants.iter_mut().enumerate() {
                            if *cv == ConstValue::Varying {
                                continue;
                            }
                            let arg_val = if i < args.len() {
                                ConstValue::from_expr(&args[i].value).unwrap_or(ConstValue::Varying)
                            } else {
                                ConstValue::Nil
                            };
                            *cv = cv.merge(&arg_val);
                        }
                    }
                    info.call_count += 1;
                }
            }
        }

        // Recurse into sub-expressions
        ExpressionKind::Binary(_, left, right) => {
            analyze_call_sites_in_expr(left, eligible, ctx);
            analyze_call_sites_in_expr(right, eligible, ctx);
        }
        ExpressionKind::Unary(_, operand) => {
            analyze_call_sites_in_expr(operand, eligible, ctx);
        }
        ExpressionKind::Assignment(target, _, value) => {
            analyze_call_sites_in_expr(target, eligible, ctx);
            analyze_call_sites_in_expr(value, eligible, ctx);
        }
        ExpressionKind::MethodCall(obj, _, args, _) => {
            analyze_call_sites_in_expr(obj, eligible, ctx);
            for arg in args.iter() {
                analyze_call_sites_in_expr(&arg.value, eligible, ctx);
            }
        }
        ExpressionKind::Member(obj, _) => {
            analyze_call_sites_in_expr(obj, eligible, ctx);
        }
        ExpressionKind::Index(obj, index) => {
            analyze_call_sites_in_expr(obj, eligible, ctx);
            analyze_call_sites_in_expr(index, eligible, ctx);
        }
        ExpressionKind::Array(elements) => {
            for elem in elements.iter() {
                match elem {
                    ArrayElement::Expression(e) | ArrayElement::Spread(e) => {
                        analyze_call_sites_in_expr(e, eligible, ctx);
                    }
                }
            }
        }
        ExpressionKind::Object(props) => {
            for prop in props.iter() {
                match prop {
                    ObjectProperty::Property { value, .. } => {
                        analyze_call_sites_in_expr(value, eligible, ctx);
                    }
                    ObjectProperty::Computed { key, value, .. } => {
                        analyze_call_sites_in_expr(key, eligible, ctx);
                        analyze_call_sites_in_expr(value, eligible, ctx);
                    }
                    ObjectProperty::Spread { value, .. } => {
                        analyze_call_sites_in_expr(value, eligible, ctx);
                    }
                }
            }
        }
        ExpressionKind::Conditional(cond, then_e, else_e) => {
            analyze_call_sites_in_expr(cond, eligible, ctx);
            analyze_call_sites_in_expr(then_e, eligible, ctx);
            analyze_call_sites_in_expr(else_e, eligible, ctx);
        }
        ExpressionKind::Parenthesized(inner) => {
            analyze_call_sites_in_expr(inner, eligible, ctx);
        }
        ExpressionKind::Pipe(left, right) => {
            analyze_call_sites_in_expr(left, eligible, ctx);
            analyze_call_sites_in_expr(right, eligible, ctx);
        }
        _ => {}
    }
}

// =============================================================================
// Constant substitution in function bodies
// =============================================================================

fn substitute_consts_in_block<'arena>(
    block: &Block<'arena>,
    subs: &FxHashMap<StringId, ConstValue>,
    arena: &'arena Bump,
) -> Block<'arena> {
    let new_stmts: Vec<_> = block
        .statements
        .iter()
        .map(|s| substitute_consts_in_stmt(s, subs, arena))
        .collect();
    Block {
        statements: arena.alloc_slice_clone(&new_stmts),
        span: block.span,
    }
}

fn substitute_consts_in_stmt<'arena>(
    stmt: &Statement<'arena>,
    subs: &FxHashMap<StringId, ConstValue>,
    arena: &'arena Bump,
) -> Statement<'arena> {
    match stmt {
        Statement::Expression(expr) => {
            Statement::Expression(substitute_consts_in_expr(expr, subs, arena))
        }
        Statement::Return(ret) => {
            let new_values: Vec<_> = ret
                .values
                .iter()
                .map(|v| substitute_consts_in_expr(v, subs, arena))
                .collect();
            Statement::Return(luanext_parser::ast::statement::ReturnStatement {
                values: arena.alloc_slice_clone(&new_values),
                span: ret.span,
            })
        }
        Statement::Variable(var) => {
            let new_init = substitute_consts_in_expr(&var.initializer, subs, arena);
            Statement::Variable(luanext_parser::ast::statement::VariableDeclaration {
                kind: var.kind,
                pattern: var.pattern.clone(),
                type_annotation: var.type_annotation.clone(),
                initializer: new_init,
                span: var.span,
            })
        }
        Statement::If(if_stmt) => {
            let new_cond = substitute_consts_in_expr(&if_stmt.condition, subs, arena);
            let new_then = substitute_consts_in_block(&if_stmt.then_block, subs, arena);
            let new_else_ifs: Vec<_> = if_stmt
                .else_ifs
                .iter()
                .map(|ei| luanext_parser::ast::statement::ElseIf {
                    condition: substitute_consts_in_expr(&ei.condition, subs, arena),
                    block: substitute_consts_in_block(&ei.block, subs, arena),
                    span: ei.span,
                })
                .collect();
            let new_else = if_stmt
                .else_block
                .as_ref()
                .map(|b| substitute_consts_in_block(b, subs, arena));
            Statement::If(luanext_parser::ast::statement::IfStatement {
                condition: new_cond,
                then_block: new_then,
                else_ifs: arena.alloc_slice_clone(&new_else_ifs),
                else_block: new_else,
                span: if_stmt.span,
            })
        }
        Statement::While(while_stmt) => {
            let new_cond = substitute_consts_in_expr(&while_stmt.condition, subs, arena);
            let new_body = substitute_consts_in_block(&while_stmt.body, subs, arena);
            Statement::While(luanext_parser::ast::statement::WhileStatement {
                condition: new_cond,
                body: new_body,
                span: while_stmt.span,
            })
        }
        Statement::Block(block) => Statement::Block(substitute_consts_in_block(block, subs, arena)),
        _ => stmt.clone(),
    }
}

fn substitute_consts_in_expr<'arena>(
    expr: &Expression<'arena>,
    subs: &FxHashMap<StringId, ConstValue>,
    arena: &'arena Bump,
) -> Expression<'arena> {
    match &expr.kind {
        ExpressionKind::Identifier(ident) => {
            if let Some(cv) = subs.get(ident) {
                Expression {
                    kind: cv.to_expression_kind(),
                    span: expr.span,
                    annotated_type: expr.annotated_type.clone(),
                    receiver_class: expr.receiver_class.clone(),
                }
            } else {
                expr.clone()
            }
        }
        ExpressionKind::Binary(op, left, right) => {
            let new_left = substitute_consts_in_expr(left, subs, arena);
            let new_right = substitute_consts_in_expr(right, subs, arena);
            Expression {
                kind: ExpressionKind::Binary(*op, arena.alloc(new_left), arena.alloc(new_right)),
                span: expr.span,
                annotated_type: expr.annotated_type.clone(),
                receiver_class: expr.receiver_class.clone(),
            }
        }
        ExpressionKind::Unary(op, operand) => {
            let new_operand = substitute_consts_in_expr(operand, subs, arena);
            Expression {
                kind: ExpressionKind::Unary(*op, arena.alloc(new_operand)),
                span: expr.span,
                annotated_type: expr.annotated_type.clone(),
                receiver_class: expr.receiver_class.clone(),
            }
        }
        ExpressionKind::Call(callee, args, type_args) => {
            let new_callee = substitute_consts_in_expr(callee, subs, arena);
            let new_args: Vec<_> = args
                .iter()
                .map(|arg| Argument {
                    value: substitute_consts_in_expr(&arg.value, subs, arena),
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
            let new_obj = substitute_consts_in_expr(obj, subs, arena);
            Expression {
                kind: ExpressionKind::Member(arena.alloc(new_obj), member.clone()),
                span: expr.span,
                annotated_type: expr.annotated_type.clone(),
                receiver_class: expr.receiver_class.clone(),
            }
        }
        ExpressionKind::Index(obj, index) => {
            let new_obj = substitute_consts_in_expr(obj, subs, arena);
            let new_index = substitute_consts_in_expr(index, subs, arena);
            Expression {
                kind: ExpressionKind::Index(arena.alloc(new_obj), arena.alloc(new_index)),
                span: expr.span,
                annotated_type: expr.annotated_type.clone(),
                receiver_class: expr.receiver_class.clone(),
            }
        }
        ExpressionKind::Conditional(cond, then_e, else_e) => {
            let new_cond = substitute_consts_in_expr(cond, subs, arena);
            let new_then = substitute_consts_in_expr(then_e, subs, arena);
            let new_else = substitute_consts_in_expr(else_e, subs, arena);
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
            let new_inner = substitute_consts_in_expr(inner, subs, arena);
            Expression {
                kind: ExpressionKind::Parenthesized(arena.alloc(new_inner)),
                span: expr.span,
                annotated_type: expr.annotated_type.clone(),
                receiver_class: expr.receiver_class.clone(),
            }
        }
        ExpressionKind::MethodCall(obj, method, args, type_args) => {
            let new_obj = substitute_consts_in_expr(obj, subs, arena);
            let new_args: Vec<_> = args
                .iter()
                .map(|arg| Argument {
                    value: substitute_consts_in_expr(&arg.value, subs, arena),
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
        _ => expr.clone(),
    }
}

// =============================================================================
// Call site rewriting — remove constant arguments from call sites
// =============================================================================

fn rewrite_call_sites_in_statement<'arena>(
    stmt: &mut Statement<'arena>,
    targets: &FxHashMap<StringId, Vec<(usize, ConstValue)>>,
    arena: &'arena Bump,
) -> bool {
    let mut changed = false;

    match stmt {
        Statement::Expression(expr) => {
            changed |= rewrite_call_sites_in_expr(expr, targets, arena);
        }
        Statement::Variable(var) => {
            changed |= rewrite_call_sites_in_expr(&mut var.initializer, targets, arena);
        }
        Statement::Return(ret) => {
            let mut values: Vec<Expression<'arena>> = ret.values.to_vec();
            let mut rc = false;
            for v in &mut values {
                rc |= rewrite_call_sites_in_expr(v, targets, arena);
            }
            if rc {
                ret.values = arena.alloc_slice_clone(&values);
                changed = true;
            }
        }
        Statement::If(if_stmt) => {
            changed |= rewrite_call_sites_in_expr(&mut if_stmt.condition, targets, arena);
            changed |= rewrite_call_sites_in_block(&mut if_stmt.then_block, targets, arena);
            let mut new_else_ifs: Vec<_> = if_stmt.else_ifs.to_vec();
            let mut eic = false;
            for ei in &mut new_else_ifs {
                eic |= rewrite_call_sites_in_expr(&mut ei.condition, targets, arena);
                eic |= rewrite_call_sites_in_block(&mut ei.block, targets, arena);
            }
            if eic {
                if_stmt.else_ifs = arena.alloc_slice_clone(&new_else_ifs);
                changed = true;
            }
            if let Some(eb) = &mut if_stmt.else_block {
                changed |= rewrite_call_sites_in_block(eb, targets, arena);
            }
        }
        Statement::While(while_stmt) => {
            changed |= rewrite_call_sites_in_expr(&mut while_stmt.condition, targets, arena);
            changed |= rewrite_call_sites_in_block(&mut while_stmt.body, targets, arena);
        }
        Statement::For(for_stmt) => match &**for_stmt {
            ForStatement::Numeric(num_ref) => {
                let mut new_num = (**num_ref).clone();
                let mut fc = false;
                fc |= rewrite_call_sites_in_expr(&mut new_num.start, targets, arena);
                fc |= rewrite_call_sites_in_expr(&mut new_num.end, targets, arena);
                if let Some(step) = &mut new_num.step {
                    fc |= rewrite_call_sites_in_expr(step, targets, arena);
                }
                fc |= rewrite_call_sites_in_block(&mut new_num.body, targets, arena);
                if fc {
                    *stmt =
                        Statement::For(arena.alloc(ForStatement::Numeric(arena.alloc(new_num))));
                    changed = true;
                }
            }
            ForStatement::Generic(gen_ref) => {
                let mut new_gen = gen_ref.clone();
                let mut fc = false;
                let mut new_iters: Vec<Expression<'arena>> = new_gen.iterators.to_vec();
                for iter in &mut new_iters {
                    fc |= rewrite_call_sites_in_expr(iter, targets, arena);
                }
                if fc {
                    new_gen.iterators = arena.alloc_slice_clone(&new_iters);
                }
                fc |= rewrite_call_sites_in_block(&mut new_gen.body, targets, arena);
                if fc {
                    *stmt = Statement::For(arena.alloc(ForStatement::Generic(new_gen)));
                    changed = true;
                }
            }
        },
        Statement::Function(func) => {
            changed |= rewrite_call_sites_in_block(&mut func.body, targets, arena);
        }
        Statement::Block(block) => {
            changed |= rewrite_call_sites_in_block(block, targets, arena);
        }
        Statement::Repeat(repeat) => {
            changed |= rewrite_call_sites_in_block(&mut repeat.body, targets, arena);
            changed |= rewrite_call_sites_in_expr(&mut repeat.until, targets, arena);
        }
        _ => {}
    }

    changed
}

fn rewrite_call_sites_in_block<'arena>(
    block: &mut Block<'arena>,
    targets: &FxHashMap<StringId, Vec<(usize, ConstValue)>>,
    arena: &'arena Bump,
) -> bool {
    let mut stmts: Vec<Statement<'arena>> = block.statements.to_vec();
    let mut changed = false;
    for s in &mut stmts {
        changed |= rewrite_call_sites_in_statement(s, targets, arena);
    }
    if changed {
        block.statements = arena.alloc_slice_clone(&stmts);
    }
    changed
}

fn rewrite_call_sites_in_expr<'arena>(
    expr: &mut Expression<'arena>,
    targets: &FxHashMap<StringId, Vec<(usize, ConstValue)>>,
    arena: &'arena Bump,
) -> bool {
    let mut changed = false;

    match &expr.kind {
        ExpressionKind::Call(callee, args, type_args) => {
            let type_args = *type_args;
            let mut new_callee = (**callee).clone();
            let mut new_args: Vec<_> = args.to_vec();

            // Recurse into sub-expressions
            let mut sub_changed = false;
            sub_changed |= rewrite_call_sites_in_expr(&mut new_callee, targets, arena);
            for arg in &mut new_args {
                sub_changed |= rewrite_call_sites_in_expr(&mut arg.value, targets, arena);
            }

            // Check if this call should have arguments removed
            if let ExpressionKind::Identifier(func_name) = &new_callee.kind {
                if let Some(prop_targets) = targets.get(func_name) {
                    let remove_indices: Vec<usize> = prop_targets.iter().map(|(i, _)| *i).collect();
                    let filtered_args: Vec<_> = new_args
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| !remove_indices.contains(i))
                        .map(|(_, a)| a.clone())
                        .collect();
                    expr.kind = ExpressionKind::Call(
                        arena.alloc(new_callee),
                        arena.alloc_slice_clone(&filtered_args),
                        type_args,
                    );
                    return true;
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
            let lc = rewrite_call_sites_in_expr(&mut new_left, targets, arena);
            let rc = rewrite_call_sites_in_expr(&mut new_right, targets, arena);
            if lc || rc {
                expr.kind =
                    ExpressionKind::Binary(op, arena.alloc(new_left), arena.alloc(new_right));
                changed = true;
            }
        }

        ExpressionKind::Unary(op, operand) => {
            let op = *op;
            let mut new_operand = (**operand).clone();
            if rewrite_call_sites_in_expr(&mut new_operand, targets, arena) {
                expr.kind = ExpressionKind::Unary(op, arena.alloc(new_operand));
                changed = true;
            }
        }

        ExpressionKind::Assignment(target, op, value) => {
            let op = *op;
            let mut new_target = (**target).clone();
            let mut new_value = (**value).clone();
            let tc = rewrite_call_sites_in_expr(&mut new_target, targets, arena);
            let vc = rewrite_call_sites_in_expr(&mut new_value, targets, arena);
            if tc || vc {
                expr.kind =
                    ExpressionKind::Assignment(arena.alloc(new_target), op, arena.alloc(new_value));
                changed = true;
            }
        }

        ExpressionKind::Conditional(cond, then_e, else_e) => {
            let mut new_cond = (**cond).clone();
            let mut new_then = (**then_e).clone();
            let mut new_else = (**else_e).clone();
            let cc = rewrite_call_sites_in_expr(&mut new_cond, targets, arena);
            let tc = rewrite_call_sites_in_expr(&mut new_then, targets, arena);
            let ec = rewrite_call_sites_in_expr(&mut new_else, targets, arena);
            if cc || tc || ec {
                expr.kind = ExpressionKind::Conditional(
                    arena.alloc(new_cond),
                    arena.alloc(new_then),
                    arena.alloc(new_else),
                );
                changed = true;
            }
        }

        ExpressionKind::Parenthesized(inner) => {
            let mut new_inner = (**inner).clone();
            if rewrite_call_sites_in_expr(&mut new_inner, targets, arena) {
                expr.kind = ExpressionKind::Parenthesized(arena.alloc(new_inner));
                changed = true;
            }
        }

        ExpressionKind::Member(obj, member) => {
            let member = member.clone();
            let mut new_obj = (**obj).clone();
            if rewrite_call_sites_in_expr(&mut new_obj, targets, arena) {
                expr.kind = ExpressionKind::Member(arena.alloc(new_obj), member);
                changed = true;
            }
        }

        ExpressionKind::Index(obj, index) => {
            let mut new_obj = (**obj).clone();
            let mut new_index = (**index).clone();
            let oc = rewrite_call_sites_in_expr(&mut new_obj, targets, arena);
            let ic = rewrite_call_sites_in_expr(&mut new_index, targets, arena);
            if oc || ic {
                expr.kind = ExpressionKind::Index(arena.alloc(new_obj), arena.alloc(new_index));
                changed = true;
            }
        }

        ExpressionKind::Pipe(left, right) => {
            let mut new_left = (**left).clone();
            let mut new_right = (**right).clone();
            let lc = rewrite_call_sites_in_expr(&mut new_left, targets, arena);
            let rc = rewrite_call_sites_in_expr(&mut new_right, targets, arena);
            if lc || rc {
                expr.kind = ExpressionKind::Pipe(arena.alloc(new_left), arena.alloc(new_right));
                changed = true;
            }
        }

        ExpressionKind::Array(elements) => {
            let mut new_elements: Vec<_> = elements.to_vec();
            let mut ec = false;
            for elem in &mut new_elements {
                match elem {
                    ArrayElement::Expression(e) | ArrayElement::Spread(e) => {
                        ec |= rewrite_call_sites_in_expr(e, targets, arena);
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
                        if rewrite_call_sites_in_expr(&mut new_val, targets, arena) {
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
                        let kc = rewrite_call_sites_in_expr(&mut new_key, targets, arena);
                        let vc = rewrite_call_sites_in_expr(&mut new_val, targets, arena);
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
                        if rewrite_call_sites_in_expr(&mut new_val, targets, arena) {
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

        // Leaf nodes
        _ => {}
    }

    changed
}
