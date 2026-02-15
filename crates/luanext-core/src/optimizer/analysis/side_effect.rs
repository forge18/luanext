//! Side-effect analysis for functions and expressions.
//!
//! Tracks which functions have observable side effects, enabling optimizations
//! like function cloning, dead call elimination, and interprocedural optimizations.
//!
//! Side effects tracked:
//! - Global variable reads and writes
//! - Table mutations
//! - I/O operations (print, io.*, file operations)
//! - Calls to unknown/unanalyzable functions
//! - Exception throwing
//! - Environment access (_ENV, getfenv, setfenv)

use luanext_parser::ast::expression::ExpressionKind;
use luanext_parser::ast::statement::{Block, ForStatement, Statement};
use luanext_parser::string_interner::{StringId, StringInterner};
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::Arc;

/// Classification of side effects for a function or expression.
#[derive(Debug, Clone, Default)]
pub struct SideEffects {
    /// Reads from global variables.
    pub global_reads: FxHashSet<StringId>,
    /// Writes to global variables.
    pub global_writes: FxHashSet<StringId>,
    /// Mutates table fields. Key = table variable name, Value = None means
    /// dynamic/unknown fields, Some(set) means specific known fields.
    pub table_mutations: FxHashMap<StringId, Option<FxHashSet<StringId>>>,
    /// Performs I/O operations (print, io.*, file operations).
    pub has_io: bool,
    /// Calls functions with unknown/unanalyzable effects.
    pub calls_unknown: bool,
    /// May throw exceptions (error(), throw, etc.).
    pub may_throw: bool,
    /// Accesses environment (_ENV, getfenv, setfenv).
    pub accesses_environment: bool,
}

impl SideEffects {
    /// Returns true if this is definitely pure (no observable side effects).
    pub fn is_pure(&self) -> bool {
        self.global_writes.is_empty()
            && self.table_mutations.is_empty()
            && !self.has_io
            && !self.calls_unknown
            && !self.may_throw
            && !self.accesses_environment
    }

    /// Returns true if this is read-only (reads globals but doesn't write).
    pub fn is_read_only(&self) -> bool {
        self.global_writes.is_empty()
            && self.table_mutations.is_empty()
            && !self.has_io
            && !self.calls_unknown
            && !self.accesses_environment
    }

    /// Merge another SideEffects into this one (union of effects).
    pub fn merge(&mut self, other: &SideEffects) {
        self.global_reads.extend(&other.global_reads);
        self.global_writes.extend(&other.global_writes);
        for (table, fields) in &other.table_mutations {
            match self.table_mutations.get_mut(table) {
                Some(existing) => {
                    // If either is None (dynamic), result is None
                    if fields.is_none() {
                        *existing = None;
                    } else if let (Some(existing_fields), Some(new_fields)) = (existing, fields) {
                        existing_fields.extend(new_fields);
                    }
                }
                None => {
                    self.table_mutations.insert(*table, fields.clone());
                }
            }
        }
        self.has_io |= other.has_io;
        self.calls_unknown |= other.calls_unknown;
        self.may_throw |= other.may_throw;
        self.accesses_environment |= other.accesses_environment;
    }
}

/// Side-effect analysis results for a whole program.
#[derive(Debug)]
pub struct SideEffectInfo {
    /// Side effects for each named function.
    pub function_effects: FxHashMap<StringId, SideEffects>,
    /// Set of functions known to be pure (no side effects at all).
    pub pure_functions: FxHashSet<StringId>,
    /// Set of functions known to be read-only (read globals, but no writes).
    pub read_only_functions: FxHashSet<StringId>,
    /// Known pure builtins (math.*, string.*, etc.).
    pub known_pure_builtins: FxHashSet<StringId>,
}

impl SideEffectInfo {
    /// Query whether a function is known to be pure.
    pub fn is_pure(&self, name: StringId) -> bool {
        self.pure_functions.contains(&name) || self.known_pure_builtins.contains(&name)
    }

    /// Query whether a function is known to be read-only.
    pub fn is_read_only(&self, name: StringId) -> bool {
        self.read_only_functions.contains(&name) || self.known_pure_builtins.contains(&name)
    }

    /// Get the side effects of a function, if analyzed.
    pub fn effects(&self, name: StringId) -> Option<&SideEffects> {
        self.function_effects.get(&name)
    }
}

/// Builder that analyzes side effects across a program.
pub struct SideEffectAnalyzer {
    interner: Arc<StringInterner>,
    /// Functions defined in the program: name -> body statements reference info.
    function_names: Vec<StringId>,
    /// Local variables known in scope (to distinguish from globals).
    local_scope: FxHashSet<StringId>,
    /// Known pure builtin names.
    known_pure: FxHashSet<StringId>,
    /// Known I/O function names.
    known_io: FxHashSet<StringId>,
    /// Known environment function names.
    known_env: FxHashSet<StringId>,
    /// Known error function names.
    known_error: FxHashSet<StringId>,
}

impl SideEffectAnalyzer {
    /// Create a new analyzer with known builtin classifications.
    pub fn new(interner: Arc<StringInterner>) -> Self {
        let mut known_pure = FxHashSet::default();
        let mut known_io = FxHashSet::default();
        let mut known_env = FxHashSet::default();
        let mut known_error = FxHashSet::default();

        // Pure math functions
        for name in &[
            "math.abs",
            "math.ceil",
            "math.floor",
            "math.max",
            "math.min",
            "math.sqrt",
            "math.sin",
            "math.cos",
            "math.tan",
            "math.asin",
            "math.acos",
            "math.atan",
            "math.exp",
            "math.log",
            "math.fmod",
            "math.huge",
            "math.pi",
            "math.random", // Note: not truly pure, but no observable side effects
        ] {
            known_pure.insert(interner.get_or_intern(name));
        }

        // Pure string functions
        for name in &[
            "string.sub",
            "string.len",
            "string.byte",
            "string.char",
            "string.rep",
            "string.reverse",
            "string.format",
            "string.upper",
            "string.lower",
            "string.find",
            "string.match",
            "string.gmatch",
            "string.gsub",
        ] {
            known_pure.insert(interner.get_or_intern(name));
        }

        // Pure table/utility functions
        for name in &[
            "table.concat",
            "table.move",
            "type",
            "tostring",
            "tonumber",
            "select",
            "rawget",
            "rawlen",
            "pairs",
            "ipairs",
            "next",
            "unpack",
            "pcall",
            "xpcall",
            "#", // length operator
        ] {
            known_pure.insert(interner.get_or_intern(name));
        }

        // I/O functions
        for name in &[
            "print",
            "io.write",
            "io.read",
            "io.open",
            "io.close",
            "io.flush",
            "io.input",
            "io.output",
            "io.lines",
            "io.tmpfile",
            "os.execute",
            "os.rename",
            "os.remove",
            "os.exit",
        ] {
            known_io.insert(interner.get_or_intern(name));
        }

        // Environment access
        for name in &["getfenv", "setfenv", "_ENV"] {
            known_env.insert(interner.get_or_intern(name));
        }

        // Error/throw
        for name in &["error", "assert"] {
            known_error.insert(interner.get_or_intern(name));
        }

        SideEffectAnalyzer {
            interner,
            function_names: Vec::new(),
            local_scope: FxHashSet::default(),
            known_pure,
            known_io,
            known_env,
            known_error,
        }
    }

    /// Analyze side effects for all functions in a program.
    pub fn analyze(mut self, statements: &[Statement<'_>]) -> SideEffectInfo {
        let mut function_effects: FxHashMap<StringId, SideEffects> = FxHashMap::default();

        // First pass: collect function names and analyze each function body
        for stmt in statements {
            match stmt {
                Statement::Function(func) => {
                    self.function_names.push(func.name.node);
                    let effects = self.analyze_block(&func.body);
                    function_effects.insert(func.name.node, effects);
                }
                _ => {}
            }
        }

        // Analyze top-level scope
        let top_effects = self.analyze_statements(statements);
        let top_name = self.interner.get_or_intern("<top>");
        function_effects.insert(top_name, top_effects);

        // Second pass: propagate effects interprocedurally (fixed-point iteration)
        let mut changed = true;
        let max_iterations = 10;
        let mut iteration = 0;
        while changed && iteration < max_iterations {
            changed = false;
            iteration += 1;

            let names: Vec<StringId> = function_effects.keys().copied().collect();
            for name in &names {
                let current = function_effects.get(name).cloned().unwrap_or_default();
                if current.calls_unknown {
                    continue; // Already maximally pessimistic
                }

                // Check if any called function has effects we haven't propagated
                // This is a simplified propagation — in practice we'd track the call graph
                // For now, we mark unknown calls as already having full effects
            }
        }

        // Classify functions
        let mut pure_functions = FxHashSet::default();
        let mut read_only_functions = FxHashSet::default();

        for (&name, effects) in &function_effects {
            if effects.is_pure() {
                pure_functions.insert(name);
            }
            if effects.is_read_only() {
                read_only_functions.insert(name);
            }
        }

        SideEffectInfo {
            function_effects,
            pure_functions,
            read_only_functions,
            known_pure_builtins: self.known_pure,
        }
    }

    fn analyze_statements(&mut self, statements: &[Statement<'_>]) -> SideEffects {
        let mut effects = SideEffects::default();
        for stmt in statements {
            let stmt_effects = self.analyze_statement(stmt);
            effects.merge(&stmt_effects);
        }
        effects
    }

    fn analyze_block(&mut self, block: &Block<'_>) -> SideEffects {
        self.analyze_statements(block.statements)
    }

    fn analyze_statement(&mut self, stmt: &Statement<'_>) -> SideEffects {
        let mut effects = SideEffects::default();

        match stmt {
            Statement::Variable(decl) => {
                // Declaration itself is local — add to scope
                self.add_pattern_to_scope(&decl.pattern);
                let init_effects = self.analyze_expression(&decl.initializer);
                effects.merge(&init_effects);
            }
            Statement::Function(func) => {
                self.local_scope.insert(func.name.node);
                // Don't analyze the body here — it's analyzed separately
            }
            Statement::Expression(expr) => {
                let expr_effects = self.analyze_expression(expr);
                effects.merge(&expr_effects);
            }
            Statement::If(if_stmt) => {
                let cond_effects = self.analyze_expression(&if_stmt.condition);
                effects.merge(&cond_effects);
                effects.merge(&self.analyze_block(&if_stmt.then_block));
                for else_if in if_stmt.else_ifs.iter() {
                    effects.merge(&self.analyze_expression(&else_if.condition));
                    effects.merge(&self.analyze_block(&else_if.block));
                }
                if let Some(else_block) = &if_stmt.else_block {
                    effects.merge(&self.analyze_block(else_block));
                }
            }
            Statement::While(while_stmt) => {
                effects.merge(&self.analyze_expression(&while_stmt.condition));
                effects.merge(&self.analyze_block(&while_stmt.body));
            }
            Statement::For(for_stmt) => match &**for_stmt {
                ForStatement::Numeric(for_num) => {
                    self.local_scope.insert(for_num.variable.node);
                    effects.merge(&self.analyze_expression(&for_num.start));
                    effects.merge(&self.analyze_expression(&for_num.end));
                    if let Some(step) = &for_num.step {
                        effects.merge(&self.analyze_expression(step));
                    }
                    effects.merge(&self.analyze_block(&for_num.body));
                }
                ForStatement::Generic(for_gen) => {
                    for var in for_gen.variables.iter() {
                        self.local_scope.insert(var.node);
                    }
                    for iter in for_gen.iterators.iter() {
                        effects.merge(&self.analyze_expression(iter));
                    }
                    effects.merge(&self.analyze_block(&for_gen.body));
                }
            },
            Statement::Repeat(repeat_stmt) => {
                effects.merge(&self.analyze_block(&repeat_stmt.body));
                effects.merge(&self.analyze_expression(&repeat_stmt.until));
            }
            Statement::Return(ret_stmt) => {
                for val in ret_stmt.values.iter() {
                    effects.merge(&self.analyze_expression(val));
                }
            }
            Statement::Throw(_) | Statement::Rethrow(_) => {
                effects.may_throw = true;
            }
            Statement::Try(try_stmt) => {
                effects.merge(&self.analyze_block(&try_stmt.try_block));
                for catch_clause in try_stmt.catch_clauses.iter() {
                    effects.merge(&self.analyze_block(&catch_clause.body));
                }
                if let Some(finally_block) = &try_stmt.finally_block {
                    effects.merge(&self.analyze_block(finally_block));
                }
            }
            Statement::Block(block) => {
                effects.merge(&self.analyze_block(block));
            }
            _ => {}
        }

        effects
    }

    fn analyze_expression(
        &mut self,
        expr: &luanext_parser::ast::expression::Expression<'_>,
    ) -> SideEffects {
        let mut effects = SideEffects::default();

        match &expr.kind {
            ExpressionKind::Identifier(name) => {
                // Reading a variable
                if !self.local_scope.contains(name) {
                    // Global read
                    effects.global_reads.insert(*name);
                }
                // Check for environment access
                if self.known_env.contains(name) {
                    effects.accesses_environment = true;
                }
            }

            ExpressionKind::Assignment(target, _, value) => {
                effects.merge(&self.analyze_expression(value));
                match &target.kind {
                    ExpressionKind::Identifier(name) => {
                        if !self.local_scope.contains(name) {
                            effects.global_writes.insert(*name);
                        }
                    }
                    ExpressionKind::Member(obj, field) => {
                        // t.field = value → table mutation
                        if let ExpressionKind::Identifier(table_name) = &obj.kind {
                            let fields = effects
                                .table_mutations
                                .entry(*table_name)
                                .or_insert_with(|| Some(FxHashSet::default()));
                            if let Some(f) = fields {
                                f.insert(field.node);
                            }
                        }
                    }
                    ExpressionKind::Index(obj, _) => {
                        // t[expr] = value → dynamic table mutation
                        if let ExpressionKind::Identifier(table_name) = &obj.kind {
                            effects.table_mutations.insert(*table_name, None);
                        }
                    }
                    _ => {
                        effects.merge(&self.analyze_expression(target));
                    }
                }
            }

            ExpressionKind::Call(func, args, _) => {
                effects.merge(&self.analyze_expression(func));
                for arg in args.iter() {
                    effects.merge(&self.analyze_expression(&arg.value));
                }
                // Classify the call
                self.classify_call(func, &mut effects);
            }

            ExpressionKind::MethodCall(obj, method, args, _) => {
                effects.merge(&self.analyze_expression(obj));
                for arg in args.iter() {
                    effects.merge(&self.analyze_expression(&arg.value));
                }
                // Method calls on unknown objects are side-effectful
                self.classify_method_call(obj, method.node, &mut effects);
            }

            ExpressionKind::Binary(_, left, right) => {
                effects.merge(&self.analyze_expression(left));
                effects.merge(&self.analyze_expression(right));
            }

            ExpressionKind::Unary(_, operand) => {
                effects.merge(&self.analyze_expression(operand));
            }

            ExpressionKind::Conditional(cond, then_expr, else_expr) => {
                effects.merge(&self.analyze_expression(cond));
                effects.merge(&self.analyze_expression(then_expr));
                effects.merge(&self.analyze_expression(else_expr));
            }

            ExpressionKind::Member(obj, _) | ExpressionKind::OptionalMember(obj, _) => {
                effects.merge(&self.analyze_expression(obj));
            }

            ExpressionKind::Index(obj, idx) | ExpressionKind::OptionalIndex(obj, idx) => {
                effects.merge(&self.analyze_expression(obj));
                effects.merge(&self.analyze_expression(idx));
            }

            ExpressionKind::Array(elements) => {
                for elem in elements.iter() {
                    match elem {
                        luanext_parser::ast::expression::ArrayElement::Expression(e)
                        | luanext_parser::ast::expression::ArrayElement::Spread(e) => {
                            effects.merge(&self.analyze_expression(e));
                        }
                    }
                }
            }

            ExpressionKind::Object(properties) => {
                for prop in properties.iter() {
                    match prop {
                        luanext_parser::ast::expression::ObjectProperty::Property { value, .. }
                        | luanext_parser::ast::expression::ObjectProperty::Computed { value, .. }
                        | luanext_parser::ast::expression::ObjectProperty::Spread { value, .. } => {
                            effects.merge(&self.analyze_expression(value));
                        }
                    }
                }
            }

            ExpressionKind::Pipe(left, right) => {
                effects.merge(&self.analyze_expression(left));
                effects.merge(&self.analyze_expression(right));
            }

            ExpressionKind::Parenthesized(inner) | ExpressionKind::TypeAssertion(inner, _) => {
                effects.merge(&self.analyze_expression(inner));
            }

            ExpressionKind::Function(_) | ExpressionKind::Arrow(_) => {
                // Function/arrow expressions themselves don't have side effects
                // (the effects happen when called)
            }

            ExpressionKind::New(constructor, args, _) => {
                effects.merge(&self.analyze_expression(constructor));
                for arg in args.iter() {
                    effects.merge(&self.analyze_expression(&arg.value));
                }
                effects.calls_unknown = true; // Conservative: constructors may have side effects
            }

            ExpressionKind::Match(match_expr) => {
                effects.merge(&self.analyze_expression(match_expr.value));
                for arm in match_expr.arms.iter() {
                    match &arm.body {
                        luanext_parser::ast::expression::MatchArmBody::Expression(e) => {
                            effects.merge(&self.analyze_expression(e));
                        }
                        luanext_parser::ast::expression::MatchArmBody::Block(b) => {
                            effects.merge(&self.analyze_block(b));
                        }
                    }
                }
            }

            ExpressionKind::Try(try_expr) => {
                effects.merge(&self.analyze_expression(try_expr.expression));
                effects.merge(&self.analyze_expression(try_expr.catch_expression));
                effects.may_throw = true;
            }

            ExpressionKind::ErrorChain(left, right) => {
                effects.merge(&self.analyze_expression(left));
                effects.merge(&self.analyze_expression(right));
            }

            ExpressionKind::OptionalCall(func, args, _) => {
                effects.merge(&self.analyze_expression(func));
                for arg in args.iter() {
                    effects.merge(&self.analyze_expression(&arg.value));
                }
                self.classify_call(func, &mut effects);
            }

            ExpressionKind::OptionalMethodCall(obj, method, args, _) => {
                effects.merge(&self.analyze_expression(obj));
                for arg in args.iter() {
                    effects.merge(&self.analyze_expression(&arg.value));
                }
                self.classify_method_call(obj, method.node, &mut effects);
            }

            ExpressionKind::Template(template) => {
                for part in template.parts.iter() {
                    if let luanext_parser::ast::expression::TemplatePart::Expression(e) = part {
                        effects.merge(&self.analyze_expression(e));
                    }
                }
            }

            // Leaf nodes with no side effects
            ExpressionKind::Literal(_)
            | ExpressionKind::SelfKeyword
            | ExpressionKind::SuperKeyword => {}
        }

        effects
    }

    /// Classify a function call's side effects.
    fn classify_call(
        &self,
        func: &luanext_parser::ast::expression::Expression<'_>,
        effects: &mut SideEffects,
    ) {
        match &func.kind {
            ExpressionKind::Identifier(name) => {
                if self.known_pure.contains(name) {
                    // Pure call — no additional effects
                } else if self.known_io.contains(name) {
                    effects.has_io = true;
                } else if self.known_env.contains(name) {
                    effects.accesses_environment = true;
                } else if self.known_error.contains(name) {
                    effects.may_throw = true;
                } else if self.function_names.contains(name) {
                    // Call to a known local function — effects will be propagated
                    // For now, mark as unknown since we haven't done interprocedural yet
                    effects.calls_unknown = true;
                } else {
                    // Unknown function call
                    effects.calls_unknown = true;
                }
            }
            ExpressionKind::Member(obj, member) => {
                // Handle module.function pattern (e.g., math.sin, io.write)
                if let ExpressionKind::Identifier(_) = &obj.kind {
                    let qualified = self
                        .interner
                        .get_or_intern(&format!("{}.{}", "?", member.node));
                    // We already check qualified names in known_pure/known_io
                    // but the member access pattern needs the full name
                    // For now, if not in known sets, mark as unknown
                    if !self.known_pure.contains(&qualified) {
                        effects.calls_unknown = true;
                    }
                } else {
                    effects.calls_unknown = true;
                }
            }
            _ => {
                effects.calls_unknown = true;
            }
        }
    }

    /// Classify a method call's side effects.
    fn classify_method_call(
        &self,
        _obj: &luanext_parser::ast::expression::Expression<'_>,
        _method: StringId,
        effects: &mut SideEffects,
    ) {
        // Method calls are conservatively side-effectful
        // (the method may modify self or have other effects)
        effects.calls_unknown = true;
    }

    fn add_pattern_to_scope(&mut self, pattern: &luanext_parser::ast::pattern::Pattern<'_>) {
        match pattern {
            luanext_parser::ast::pattern::Pattern::Identifier(ident) => {
                self.local_scope.insert(ident.node);
            }
            luanext_parser::ast::pattern::Pattern::Array(arr_pat) => {
                for elem in arr_pat.elements.iter() {
                    if let luanext_parser::ast::pattern::ArrayPatternElement::Pattern(pwd) = elem {
                        self.add_pattern_to_scope(&pwd.pattern);
                    }
                }
            }
            luanext_parser::ast::pattern::Pattern::Object(obj_pat) => {
                for prop in obj_pat.properties.iter() {
                    if let Some(ref pat) = prop.value {
                        self.add_pattern_to_scope(pat);
                    } else {
                        self.local_scope.insert(prop.key.node);
                    }
                }
            }
            luanext_parser::ast::pattern::Pattern::Wildcard(_)
            | luanext_parser::ast::pattern::Pattern::Literal(_, _)
            | luanext_parser::ast::pattern::Pattern::Or(_)
            | luanext_parser::ast::pattern::Pattern::Template(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luanext_parser::ast::expression::{Argument, Expression, ExpressionKind, Literal};
    use luanext_parser::ast::pattern::Pattern;
    use luanext_parser::ast::statement::{
        Block, FunctionDeclaration, ReturnStatement, Statement, VariableDeclaration, VariableKind,
    };
    use luanext_parser::ast::Ident;
    use luanext_parser::span::Span;

    fn make_ident(interner: &StringInterner, name: &str) -> Ident {
        Ident {
            node: interner.get_or_intern(name),
            span: Span::dummy(),
        }
    }

    fn make_expr_nil() -> Expression<'static> {
        Expression {
            kind: ExpressionKind::Literal(Literal::Nil),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: None,
        }
    }

    fn make_number(n: f64) -> Expression<'static> {
        Expression {
            kind: ExpressionKind::Literal(Literal::Number(n)),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: None,
        }
    }

    #[test]
    fn test_pure_function_detection() {
        // function add(a, b) return a + b end
        let interner = Arc::new(StringInterner::new());
        let arena = bumpalo::Bump::new();

        let a_id = interner.get_or_intern("a");
        let b_id = interner.get_or_intern("b");

        let add_expr = Expression {
            kind: ExpressionKind::Binary(
                luanext_parser::ast::expression::BinaryOp::Add,
                arena.alloc(Expression {
                    kind: ExpressionKind::Identifier(a_id),
                    span: Span::dummy(),
                    annotated_type: None,
                    receiver_class: None,
                }),
                arena.alloc(Expression {
                    kind: ExpressionKind::Identifier(b_id),
                    span: Span::dummy(),
                    annotated_type: None,
                    receiver_class: None,
                }),
            ),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: None,
        };
        let ret_values = arena.alloc_slice_clone(&[add_expr]);
        let ret_stmt = Statement::Return(ReturnStatement {
            values: ret_values,
            span: Span::dummy(),
        });
        let body_stmts = arena.alloc_slice_clone(&[ret_stmt]);
        let body = Block {
            statements: body_stmts,
            span: Span::dummy(),
        };

        let params = arena.alloc_slice_clone(&[
            luanext_parser::ast::statement::Parameter {
                pattern: Pattern::Identifier(make_ident(&interner, "a")),
                type_annotation: None,
                default: None,
                is_rest: false,
                is_optional: false,
                span: Span::dummy(),
            },
            luanext_parser::ast::statement::Parameter {
                pattern: Pattern::Identifier(make_ident(&interner, "b")),
                type_annotation: None,
                default: None,
                is_rest: false,
                is_optional: false,
                span: Span::dummy(),
            },
        ]);

        let stmts = vec![Statement::Function(FunctionDeclaration {
            name: make_ident(&interner, "add"),
            type_parameters: None,
            parameters: params,
            return_type: None,
            throws: None,
            body,
            span: Span::dummy(),
        })];

        let analyzer = SideEffectAnalyzer::new(interner.clone());
        let info = analyzer.analyze(&stmts);

        let add_name = interner.get_or_intern("add");
        // add() only uses local parameters — should be pure
        let effects = info.effects(add_name).unwrap();
        assert!(effects.is_pure(), "add(a, b) should be pure");
    }

    #[test]
    fn test_global_write_detection() {
        // g = 42 (global assignment)
        let interner = Arc::new(StringInterner::new());
        let arena = bumpalo::Bump::new();

        let g_id = interner.get_or_intern("g");
        let assign = Expression {
            kind: ExpressionKind::Assignment(
                arena.alloc(Expression {
                    kind: ExpressionKind::Identifier(g_id),
                    span: Span::dummy(),
                    annotated_type: None,
                    receiver_class: None,
                }),
                luanext_parser::ast::expression::AssignmentOp::Assign,
                arena.alloc(make_number(42.0)),
            ),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: None,
        };

        let stmts = vec![Statement::Expression(assign)];

        let analyzer = SideEffectAnalyzer::new(interner.clone());
        let info = analyzer.analyze(&stmts);

        let top_name = interner.get_or_intern("<top>");
        let effects = info.effects(top_name).unwrap();
        assert!(
            effects.global_writes.contains(&g_id),
            "Assignment to g should be detected as global write"
        );
        assert!(!effects.is_pure(), "Global write is not pure");
    }

    #[test]
    fn test_io_function_detection() {
        // print("hello")
        let interner = Arc::new(StringInterner::new());
        let arena = bumpalo::Bump::new();

        let print_id = interner.get_or_intern("print");
        let args = arena.alloc_slice_clone(&[Argument {
            value: Expression {
                kind: ExpressionKind::Literal(Literal::String("hello".to_string())),
                span: Span::dummy(),
                annotated_type: None,
                receiver_class: None,
            },
            is_spread: false,
            span: Span::dummy(),
        }]);

        let call = Expression {
            kind: ExpressionKind::Call(
                arena.alloc(Expression {
                    kind: ExpressionKind::Identifier(print_id),
                    span: Span::dummy(),
                    annotated_type: None,
                    receiver_class: None,
                }),
                args,
                None,
            ),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: None,
        };

        let stmts = vec![Statement::Expression(call)];

        let analyzer = SideEffectAnalyzer::new(interner.clone());
        let info = analyzer.analyze(&stmts);

        let top_name = interner.get_or_intern("<top>");
        let effects = info.effects(top_name).unwrap();
        assert!(effects.has_io, "print() should be detected as I/O");
    }

    #[test]
    fn test_throw_detection() {
        use luanext_parser::ast::statement::ThrowStatement;

        let interner = Arc::new(StringInterner::new());

        let stmts = vec![Statement::Throw(ThrowStatement {
            expression: make_expr_nil(),
            span: Span::dummy(),
        })];

        let analyzer = SideEffectAnalyzer::new(interner.clone());
        let info = analyzer.analyze(&stmts);

        let top_name = interner.get_or_intern("<top>");
        let effects = info.effects(top_name).unwrap();
        assert!(effects.may_throw, "throw should set may_throw");
    }

    #[test]
    fn test_known_pure_builtins() {
        let interner = Arc::new(StringInterner::new());
        let analyzer = SideEffectAnalyzer::new(interner.clone());
        let info = analyzer.analyze(&[]);

        // math.abs should be known pure
        let math_abs = interner.get_or_intern("math.abs");
        assert!(info.is_pure(math_abs), "math.abs should be known pure");

        // string.len should be known pure
        let string_len = interner.get_or_intern("string.len");
        assert!(info.is_pure(string_len), "string.len should be known pure");

        // type should be known pure
        let type_fn = interner.get_or_intern("type");
        assert!(info.is_pure(type_fn), "type() should be known pure");
    }

    #[test]
    fn test_table_mutation_detection() {
        // t.x = 42
        let interner = Arc::new(StringInterner::new());
        let arena = bumpalo::Bump::new();

        let t_id = interner.get_or_intern("t");
        let x_ident = make_ident(&interner, "x");

        let assign = Expression {
            kind: ExpressionKind::Assignment(
                arena.alloc(Expression {
                    kind: ExpressionKind::Member(
                        arena.alloc(Expression {
                            kind: ExpressionKind::Identifier(t_id),
                            span: Span::dummy(),
                            annotated_type: None,
                            receiver_class: None,
                        }),
                        x_ident,
                    ),
                    span: Span::dummy(),
                    annotated_type: None,
                    receiver_class: None,
                }),
                luanext_parser::ast::expression::AssignmentOp::Assign,
                arena.alloc(make_number(42.0)),
            ),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: None,
        };

        let stmts = vec![Statement::Expression(assign)];

        let analyzer = SideEffectAnalyzer::new(interner.clone());
        let info = analyzer.analyze(&stmts);

        let top_name = interner.get_or_intern("<top>");
        let effects = info.effects(top_name).unwrap();
        assert!(
            effects.table_mutations.contains_key(&t_id),
            "t.x = 42 should be detected as table mutation"
        );
        assert!(!effects.is_pure(), "Table mutation is not pure");
    }

    #[test]
    fn test_local_variable_not_global() {
        // local x = 42 -- not a global write
        let interner = Arc::new(StringInterner::new());

        let stmts = vec![Statement::Variable(VariableDeclaration {
            kind: VariableKind::Local,
            pattern: Pattern::Identifier(make_ident(&interner, "x")),
            type_annotation: None,
            initializer: make_number(42.0),
            span: Span::dummy(),
        })];

        let analyzer = SideEffectAnalyzer::new(interner.clone());
        let info = analyzer.analyze(&stmts);

        let top_name = interner.get_or_intern("<top>");
        let effects = info.effects(top_name).unwrap();
        assert!(
            effects.global_writes.is_empty(),
            "Local variable declaration should not be a global write"
        );
    }
}
