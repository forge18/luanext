//! Alias analysis for Lua programs.
//!
//! Tracks which expressions may refer to the same memory location.
//! Required for: escape analysis, scalar replacement of aggregates.
//!
//! Lua-specific simplifications:
//! - Primitive types (number, string, boolean, nil) are value types and never alias.
//! - Only tables and closures can alias (no pointer arithmetic).
//! - Global variables conservatively may alias anything.
//!
//! This is a flow-insensitive, intraprocedural analysis using union-find
//! for alias class tracking.

use luanext_parser::ast::expression::ExpressionKind;
use luanext_parser::ast::pattern::Pattern;
use luanext_parser::ast::statement::{Block, ForStatement, Statement};
use luanext_parser::string_interner::StringId;
use rustc_hash::{FxHashMap, FxHashSet};

/// An abstract memory location in a Lua program.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MemoryLocation {
    /// A local variable by name.
    Local(StringId),
    /// A global variable by name.
    Global(StringId),
    /// A table field access: (table_variable, field_name).
    TableField(StringId, StringId),
    /// A table with a dynamic (computed) index: (table_variable).
    TableDynamic(StringId),
    /// An upvalue captured by a closure.
    Upvalue(StringId),
}

/// Result of querying whether two memory locations may alias.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AliasResult {
    /// Definitely do not alias (distinct memory locations).
    NoAlias,
    /// May possibly alias (conservative approximation).
    MayAlias,
    /// Definitely refer to the same memory location.
    MustAlias,
}

/// Alias analysis results for a function scope.
///
/// Tracks alias classes (sets of variables that may point to the same table),
/// escaped variables, and table-typed variables.
#[derive(Debug)]
pub struct AliasInfo {
    /// Sets of variables that may alias each other ("alias classes").
    /// Each set represents a group of variables that could refer to the same
    /// underlying table object.
    pub alias_classes: Vec<FxHashSet<MemoryLocation>>,
    /// Map from memory location to its alias class index.
    pub location_to_class: FxHashMap<MemoryLocation, usize>,
    /// Variables that have escaped their declaring scope (passed to unknown
    /// functions, stored in tables, returned from the function).
    pub escaped: FxHashSet<StringId>,
    /// Variables known to be table-typed (can participate in aliasing).
    pub table_variables: FxHashSet<StringId>,
}

impl AliasInfo {
    /// Query whether two memory locations may alias.
    pub fn query(&self, a: &MemoryLocation, b: &MemoryLocation) -> AliasResult {
        // Same location always aliases itself
        if a == b {
            return AliasResult::MustAlias;
        }

        // Globals conservatively may alias anything
        if matches!(a, MemoryLocation::Global(_)) || matches!(b, MemoryLocation::Global(_)) {
            return AliasResult::MayAlias;
        }

        // Upvalues conservatively may alias their corresponding locals
        match (a, b) {
            (MemoryLocation::Upvalue(u), MemoryLocation::Local(l))
            | (MemoryLocation::Local(l), MemoryLocation::Upvalue(u)) => {
                if u == l {
                    return AliasResult::MustAlias;
                }
            }
            _ => {}
        }

        // Check if both are in the same alias class
        let class_a = self.location_to_class.get(a);
        let class_b = self.location_to_class.get(b);

        match (class_a, class_b) {
            (Some(&ca), Some(&cb)) if ca == cb => AliasResult::MayAlias,
            (Some(_), Some(_)) => AliasResult::NoAlias,
            // If either is not in any class, check if it's a local that
            // can't alias (non-table primitives never alias)
            _ => {
                let a_is_primitive = self.is_non_table_local(a);
                let b_is_primitive = self.is_non_table_local(b);
                if a_is_primitive || b_is_primitive {
                    AliasResult::NoAlias
                } else {
                    AliasResult::MayAlias
                }
            }
        }
    }

    /// Returns true if a memory location is a local variable that is NOT
    /// table-typed (i.e., a primitive that can't alias).
    fn is_non_table_local(&self, loc: &MemoryLocation) -> bool {
        if let MemoryLocation::Local(name) = loc {
            !self.table_variables.contains(name)
        } else {
            false
        }
    }

    /// Returns true if a variable has escaped its scope.
    pub fn has_escaped(&self, name: StringId) -> bool {
        self.escaped.contains(&name)
    }

    /// Returns all locations in the same alias class as the given location.
    pub fn aliases_of(&self, loc: &MemoryLocation) -> Vec<&MemoryLocation> {
        if let Some(&class_idx) = self.location_to_class.get(loc) {
            if let Some(class) = self.alias_classes.get(class_idx) {
                return class.iter().filter(|l| *l != loc).collect();
            }
        }
        Vec::new()
    }
}

/// Builder for alias analysis using union-find.
pub struct AliasAnalyzer {
    /// Union-find parent array for alias classes.
    parent: Vec<usize>,
    /// Rank for union-find balancing.
    rank: Vec<usize>,
    /// Map from location to union-find index.
    loc_to_idx: FxHashMap<MemoryLocation, usize>,
    /// Reverse map: index to location.
    idx_to_loc: Vec<MemoryLocation>,
    /// Variables known to be table-typed.
    table_variables: FxHashSet<StringId>,
    /// Variables that have escaped.
    escaped: FxHashSet<StringId>,
    /// Local variables in scope.
    locals: FxHashSet<StringId>,
}

impl AliasAnalyzer {
    /// Create a new alias analyzer.
    pub fn new() -> Self {
        AliasAnalyzer {
            parent: Vec::new(),
            rank: Vec::new(),
            loc_to_idx: FxHashMap::default(),
            idx_to_loc: Vec::new(),
            table_variables: FxHashSet::default(),
            escaped: FxHashSet::default(),
            locals: FxHashSet::default(),
        }
    }

    /// Analyze alias information from a statement list.
    pub fn analyze(mut self, statements: &[Statement<'_>]) -> AliasInfo {
        self.collect_locals(statements);
        self.analyze_statements(statements);
        self.build_result()
    }

    /// Register a memory location in the union-find, returning its index.
    fn register(&mut self, loc: MemoryLocation) -> usize {
        if let Some(&idx) = self.loc_to_idx.get(&loc) {
            return idx;
        }
        let idx = self.parent.len();
        self.parent.push(idx); // Initially points to self
        self.rank.push(0);
        self.loc_to_idx.insert(loc.clone(), idx);
        self.idx_to_loc.push(loc);
        idx
    }

    /// Find the root of a union-find set (with path compression).
    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]]; // Path halving
            x = self.parent[x];
        }
        x
    }

    /// Merge two union-find sets (union by rank).
    fn union(&mut self, a: usize, b: usize) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            return;
        }
        if self.rank[ra] < self.rank[rb] {
            self.parent[ra] = rb;
        } else if self.rank[ra] > self.rank[rb] {
            self.parent[rb] = ra;
        } else {
            self.parent[rb] = ra;
            self.rank[ra] += 1;
        }
    }

    /// First pass: collect all local variable declarations.
    fn collect_locals(&mut self, statements: &[Statement<'_>]) {
        for stmt in statements {
            match stmt {
                Statement::Variable(decl) => {
                    self.add_pattern_locals(&decl.pattern);
                }
                Statement::Function(func) => {
                    self.locals.insert(func.name.node);
                    // Collect locals inside the function body too
                    self.collect_block_locals(&func.body);
                }
                Statement::For(for_stmt) => match &**for_stmt {
                    ForStatement::Numeric(for_num) => {
                        self.locals.insert(for_num.variable.node);
                        self.collect_block_locals(&for_num.body);
                    }
                    ForStatement::Generic(for_gen) => {
                        for var in for_gen.variables.iter() {
                            self.locals.insert(var.node);
                        }
                        self.collect_block_locals(&for_gen.body);
                    }
                },
                Statement::If(if_stmt) => {
                    self.collect_block_locals(&if_stmt.then_block);
                    for else_if in if_stmt.else_ifs.iter() {
                        self.collect_block_locals(&else_if.block);
                    }
                    if let Some(else_block) = &if_stmt.else_block {
                        self.collect_block_locals(else_block);
                    }
                }
                Statement::While(while_stmt) => {
                    self.collect_block_locals(&while_stmt.body);
                }
                Statement::Repeat(repeat_stmt) => {
                    self.collect_block_locals(&repeat_stmt.body);
                }
                Statement::Block(block) => {
                    self.collect_block_locals(block);
                }
                Statement::Try(try_stmt) => {
                    self.collect_block_locals(&try_stmt.try_block);
                    for catch in try_stmt.catch_clauses.iter() {
                        self.collect_block_locals(&catch.body);
                    }
                    if let Some(finally_block) = &try_stmt.finally_block {
                        self.collect_block_locals(finally_block);
                    }
                }
                _ => {}
            }
        }
    }

    fn collect_block_locals(&mut self, block: &Block<'_>) {
        self.collect_locals(block.statements);
    }

    fn add_pattern_locals(&mut self, pattern: &Pattern<'_>) {
        match pattern {
            Pattern::Identifier(ident) => {
                self.locals.insert(ident.node);
            }
            Pattern::Array(arr_pat) => {
                for elem in arr_pat.elements.iter() {
                    if let luanext_parser::ast::pattern::ArrayPatternElement::Pattern(pwd) = elem {
                        self.add_pattern_locals(&pwd.pattern);
                    }
                }
            }
            Pattern::Object(obj_pat) => {
                for prop in obj_pat.properties.iter() {
                    if let Some(ref pat) = prop.value {
                        self.add_pattern_locals(pat);
                    } else {
                        self.locals.insert(prop.key.node);
                    }
                }
            }
            Pattern::Wildcard(_)
            | Pattern::Literal(_, _)
            | Pattern::Or(_)
            | Pattern::Template(_) => {}
        }
    }

    /// Second pass: analyze statements for aliasing relationships.
    fn analyze_statements(&mut self, statements: &[Statement<'_>]) {
        for stmt in statements {
            self.analyze_statement(stmt);
        }
    }

    fn analyze_statement(&mut self, stmt: &Statement<'_>) {
        match stmt {
            Statement::Variable(decl) => {
                // Check if initializer creates a table
                if self.is_table_expression(&decl.initializer) {
                    self.mark_pattern_as_table(&decl.pattern);
                }
                // Check for direct alias: local b = a (where a is a table)
                if let ExpressionKind::Identifier(source_name) = &decl.initializer.kind {
                    if self.table_variables.contains(source_name) {
                        // b aliases a
                        let target_names = self.pattern_names(&decl.pattern);
                        for name in target_names {
                            self.table_variables.insert(name);
                            let source_loc = if self.locals.contains(source_name) {
                                MemoryLocation::Local(*source_name)
                            } else {
                                MemoryLocation::Global(*source_name)
                            };
                            let target_loc = MemoryLocation::Local(name);
                            let si = self.register(source_loc);
                            let ti = self.register(target_loc);
                            self.union(si, ti);
                        }
                    }
                }
                // Check for escaped variables (passed to calls in initializer)
                self.check_expression_escapes(&decl.initializer);
            }
            Statement::Expression(expr) => {
                // Check for assignments that create aliases
                if let ExpressionKind::Assignment(target, _, value) = &expr.kind {
                    if let ExpressionKind::Identifier(target_name) = &target.kind {
                        if let ExpressionKind::Identifier(source_name) = &value.kind {
                            if self.table_variables.contains(source_name) {
                                self.table_variables.insert(*target_name);
                                let source_loc = if self.locals.contains(source_name) {
                                    MemoryLocation::Local(*source_name)
                                } else {
                                    MemoryLocation::Global(*source_name)
                                };
                                let target_loc = if self.locals.contains(target_name) {
                                    MemoryLocation::Local(*target_name)
                                } else {
                                    MemoryLocation::Global(*target_name)
                                };
                                let si = self.register(source_loc);
                                let ti = self.register(target_loc);
                                self.union(si, ti);
                            }
                        }
                    }
                }
                self.check_expression_escapes(expr);
            }
            Statement::Function(func) => {
                // Analyze function body for aliases
                self.analyze_block(&func.body);
                // Check for captured upvalues
                self.check_block_for_captures(&func.body);
            }
            Statement::If(if_stmt) => {
                self.analyze_block(&if_stmt.then_block);
                for else_if in if_stmt.else_ifs.iter() {
                    self.analyze_block(&else_if.block);
                }
                if let Some(else_block) = &if_stmt.else_block {
                    self.analyze_block(else_block);
                }
            }
            Statement::While(while_stmt) => {
                self.analyze_block(&while_stmt.body);
            }
            Statement::For(for_stmt) => match &**for_stmt {
                ForStatement::Numeric(for_num) => {
                    self.analyze_block(&for_num.body);
                }
                ForStatement::Generic(for_gen) => {
                    self.analyze_block(&for_gen.body);
                }
            },
            Statement::Repeat(repeat_stmt) => {
                self.analyze_block(&repeat_stmt.body);
            }
            Statement::Return(ret) => {
                // Returned values escape
                for val in ret.values.iter() {
                    if let ExpressionKind::Identifier(name) = &val.kind {
                        if self.table_variables.contains(name) {
                            self.escaped.insert(*name);
                        }
                    }
                }
            }
            Statement::Block(block) => {
                self.analyze_block(block);
            }
            Statement::Try(try_stmt) => {
                self.analyze_block(&try_stmt.try_block);
                for catch in try_stmt.catch_clauses.iter() {
                    self.analyze_block(&catch.body);
                }
                if let Some(finally_block) = &try_stmt.finally_block {
                    self.analyze_block(finally_block);
                }
            }
            _ => {}
        }
    }

    fn analyze_block(&mut self, block: &Block<'_>) {
        self.analyze_statements(block.statements);
    }

    /// Check if an expression produces a table value.
    fn is_table_expression(&self, expr: &luanext_parser::ast::expression::Expression<'_>) -> bool {
        matches!(
            &expr.kind,
            ExpressionKind::Object(_) | ExpressionKind::Array(_) | ExpressionKind::New(_, _, _)
        )
    }

    /// Mark all variables in a pattern as table-typed.
    fn mark_pattern_as_table(&mut self, pattern: &Pattern<'_>) {
        for name in self.pattern_names(pattern) {
            self.table_variables.insert(name);
        }
    }

    /// Extract all variable names from a pattern.
    fn pattern_names(&self, pattern: &Pattern<'_>) -> Vec<StringId> {
        let mut names = Vec::new();
        self.collect_pattern_names(pattern, &mut names);
        names
    }

    fn collect_pattern_names(&self, pattern: &Pattern<'_>, names: &mut Vec<StringId>) {
        match pattern {
            Pattern::Identifier(ident) => names.push(ident.node),
            Pattern::Array(arr_pat) => {
                for elem in arr_pat.elements.iter() {
                    if let luanext_parser::ast::pattern::ArrayPatternElement::Pattern(pwd) = elem {
                        self.collect_pattern_names(&pwd.pattern, names);
                    }
                }
            }
            Pattern::Object(obj_pat) => {
                for prop in obj_pat.properties.iter() {
                    if let Some(ref pat) = prop.value {
                        self.collect_pattern_names(pat, names);
                    } else {
                        names.push(prop.key.node);
                    }
                }
            }
            Pattern::Wildcard(_)
            | Pattern::Literal(_, _)
            | Pattern::Or(_)
            | Pattern::Template(_) => {}
        }
    }

    /// Check if an expression causes variables to escape (passed to unknown calls).
    fn check_expression_escapes(&mut self, expr: &luanext_parser::ast::expression::Expression<'_>) {
        match &expr.kind {
            ExpressionKind::Call(_, args, _)
            | ExpressionKind::MethodCall(_, _, args, _)
            | ExpressionKind::OptionalCall(_, args, _)
            | ExpressionKind::OptionalMethodCall(_, _, args, _)
            | ExpressionKind::New(_, args, _) => {
                for arg in args.iter() {
                    if let ExpressionKind::Identifier(name) = &arg.value.kind {
                        if self.table_variables.contains(name) {
                            self.escaped.insert(*name);
                        }
                    }
                }
            }
            ExpressionKind::Binary(_, left, right) => {
                self.check_expression_escapes(left);
                self.check_expression_escapes(right);
            }
            ExpressionKind::Unary(_, operand) => {
                self.check_expression_escapes(operand);
            }
            ExpressionKind::Conditional(cond, then_expr, else_expr) => {
                self.check_expression_escapes(cond);
                self.check_expression_escapes(then_expr);
                self.check_expression_escapes(else_expr);
            }
            ExpressionKind::Assignment(_, _, value) => {
                self.check_expression_escapes(value);
            }
            _ => {}
        }
    }

    /// Check a block for closure captures (variables used in nested functions).
    fn check_block_for_captures(&mut self, block: &Block<'_>) {
        for stmt in block.statements.iter() {
            match stmt {
                Statement::Variable(decl) => {
                    self.check_expression_for_captures(&decl.initializer);
                }
                Statement::Expression(expr) => {
                    self.check_expression_for_captures(expr);
                }
                _ => {}
            }
        }
    }

    fn check_expression_for_captures(
        &mut self,
        expr: &luanext_parser::ast::expression::Expression<'_>,
    ) {
        match &expr.kind {
            ExpressionKind::Function(_) | ExpressionKind::Arrow(_) => {
                // Variables referenced inside closures are upvalues
                // Conservative: mark all outer table variables as potentially captured
                // A more precise analysis would track exactly which variables are referenced
                for &var in &self.table_variables.clone() {
                    self.escaped.insert(var);
                    let local_loc = MemoryLocation::Local(var);
                    let upvalue_loc = MemoryLocation::Upvalue(var);
                    let li = self.register(local_loc);
                    let ui = self.register(upvalue_loc);
                    self.union(li, ui);
                }
            }
            ExpressionKind::Call(func, args, _) => {
                self.check_expression_for_captures(func);
                for arg in args.iter() {
                    self.check_expression_for_captures(&arg.value);
                }
            }
            ExpressionKind::Binary(_, left, right) => {
                self.check_expression_for_captures(left);
                self.check_expression_for_captures(right);
            }
            _ => {}
        }
    }

    /// Build the final AliasInfo from the union-find data.
    fn build_result(mut self) -> AliasInfo {
        // Group locations by their root in the union-find
        let mut root_to_class: FxHashMap<usize, usize> = FxHashMap::default();
        let mut alias_classes: Vec<FxHashSet<MemoryLocation>> = Vec::new();
        let mut location_to_class: FxHashMap<MemoryLocation, usize> = FxHashMap::default();

        // Collect entries first to avoid borrowing self immutably and mutably
        let entries: Vec<(MemoryLocation, usize)> = self
            .loc_to_idx
            .iter()
            .map(|(loc, &idx)| (loc.clone(), idx))
            .collect();

        for (loc, idx) in entries {
            let root = self.find(idx);
            let class_idx = match root_to_class.get(&root) {
                Some(&ci) => ci,
                None => {
                    let ci = alias_classes.len();
                    alias_classes.push(FxHashSet::default());
                    root_to_class.insert(root, ci);
                    ci
                }
            };
            alias_classes[class_idx].insert(loc.clone());
            location_to_class.insert(loc, class_idx);
        }

        AliasInfo {
            alias_classes,
            location_to_class,
            escaped: self.escaped,
            table_variables: self.table_variables,
        }
    }
}

impl Default for AliasAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luanext_parser::ast::expression::{Expression, ExpressionKind, Literal};
    use luanext_parser::ast::statement::{
        ReturnStatement, Statement, VariableDeclaration, VariableKind,
    };
    use luanext_parser::ast::Ident;
    use luanext_parser::span::Span;
    use luanext_parser::string_interner::StringInterner;

    fn make_ident(interner: &StringInterner, name: &str) -> Ident {
        Ident {
            node: interner.get_or_intern(name),
            span: Span::dummy(),
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
    fn test_primitive_no_alias() {
        // local x = 1; local y = 2
        // Primitives don't alias
        let interner = StringInterner::new();
        let stmts = vec![
            Statement::Variable(VariableDeclaration {
                kind: VariableKind::Local,
                pattern: Pattern::Identifier(make_ident(&interner, "x")),
                type_annotation: None,
                initializer: make_number(1.0),
                span: Span::dummy(),
            }),
            Statement::Variable(VariableDeclaration {
                kind: VariableKind::Local,
                pattern: Pattern::Identifier(make_ident(&interner, "y")),
                type_annotation: None,
                initializer: make_number(2.0),
                span: Span::dummy(),
            }),
        ];

        let analyzer = AliasAnalyzer::new();
        let info = analyzer.analyze(&stmts);

        let x_id = interner.get_or_intern("x");
        let y_id = interner.get_or_intern("y");

        let result = info.query(&MemoryLocation::Local(x_id), &MemoryLocation::Local(y_id));
        assert_eq!(
            result,
            AliasResult::NoAlias,
            "Primitive locals should not alias"
        );
    }

    #[test]
    fn test_table_assignment_alias() {
        // local a = {}; local b = a
        // b should alias a
        let interner = StringInterner::new();

        let stmts = vec![
            Statement::Variable(VariableDeclaration {
                kind: VariableKind::Local,
                pattern: Pattern::Identifier(make_ident(&interner, "a")),
                type_annotation: None,
                initializer: Expression {
                    kind: ExpressionKind::Object(&[]),
                    span: Span::dummy(),
                    annotated_type: None,
                    receiver_class: None,
                },
                span: Span::dummy(),
            }),
            Statement::Variable(VariableDeclaration {
                kind: VariableKind::Local,
                pattern: Pattern::Identifier(make_ident(&interner, "b")),
                type_annotation: None,
                initializer: Expression {
                    kind: ExpressionKind::Identifier(interner.get_or_intern("a")),
                    span: Span::dummy(),
                    annotated_type: None,
                    receiver_class: None,
                },
                span: Span::dummy(),
            }),
        ];

        let analyzer = AliasAnalyzer::new();
        let info = analyzer.analyze(&stmts);

        let a_id = interner.get_or_intern("a");
        let b_id = interner.get_or_intern("b");

        let result = info.query(&MemoryLocation::Local(a_id), &MemoryLocation::Local(b_id));
        assert_eq!(
            result,
            AliasResult::MayAlias,
            "b = a should make them alias"
        );
    }

    #[test]
    fn test_escape_detection() {
        // local t = {}; foo(t) -- t escapes
        let interner = StringInterner::new();
        let arena = bumpalo::Bump::new();

        let t_id = interner.get_or_intern("t");
        let foo_id = interner.get_or_intern("foo");

        let args = arena.alloc_slice_clone(&[luanext_parser::ast::expression::Argument {
            value: Expression {
                kind: ExpressionKind::Identifier(t_id),
                span: Span::dummy(),
                annotated_type: None,
                receiver_class: None,
            },
            is_spread: false,
            span: Span::dummy(),
        }]);

        let stmts = vec![
            Statement::Variable(VariableDeclaration {
                kind: VariableKind::Local,
                pattern: Pattern::Identifier(make_ident(&interner, "t")),
                type_annotation: None,
                initializer: Expression {
                    kind: ExpressionKind::Object(&[]),
                    span: Span::dummy(),
                    annotated_type: None,
                    receiver_class: None,
                },
                span: Span::dummy(),
            }),
            Statement::Expression(Expression {
                kind: ExpressionKind::Call(
                    arena.alloc(Expression {
                        kind: ExpressionKind::Identifier(foo_id),
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
            }),
        ];

        let analyzer = AliasAnalyzer::new();
        let info = analyzer.analyze(&stmts);

        assert!(
            info.has_escaped(t_id),
            "t should be marked as escaped after being passed to foo()"
        );
    }

    #[test]
    fn test_global_conservative() {
        // Global variables may alias anything
        let interner = StringInterner::new();
        let x_id = interner.get_or_intern("x");
        let g_id = interner.get_or_intern("g");

        let stmts: Vec<Statement<'_>> = vec![];
        let analyzer = AliasAnalyzer::new();
        let info = analyzer.analyze(&stmts);

        let result = info.query(&MemoryLocation::Global(g_id), &MemoryLocation::Local(x_id));
        assert_eq!(
            result,
            AliasResult::MayAlias,
            "Globals should conservatively may-alias locals"
        );
    }

    #[test]
    fn test_self_alias() {
        let interner = StringInterner::new();
        let x_id = interner.get_or_intern("x");

        let stmts: Vec<Statement<'_>> = vec![];
        let analyzer = AliasAnalyzer::new();
        let info = analyzer.analyze(&stmts);

        let result = info.query(&MemoryLocation::Local(x_id), &MemoryLocation::Local(x_id));
        assert_eq!(
            result,
            AliasResult::MustAlias,
            "A location should MustAlias itself"
        );
    }

    #[test]
    fn test_table_variable_tracking() {
        // local t = {} marks t as table-typed
        let interner = StringInterner::new();

        let stmts = vec![Statement::Variable(VariableDeclaration {
            kind: VariableKind::Local,
            pattern: Pattern::Identifier(make_ident(&interner, "t")),
            type_annotation: None,
            initializer: Expression {
                kind: ExpressionKind::Object(&[]),
                span: Span::dummy(),
                annotated_type: None,
                receiver_class: None,
            },
            span: Span::dummy(),
        })];

        let analyzer = AliasAnalyzer::new();
        let info = analyzer.analyze(&stmts);

        let t_id = interner.get_or_intern("t");
        assert!(
            info.table_variables.contains(&t_id),
            "t should be tracked as table-typed"
        );
    }

    #[test]
    fn test_return_escapes() {
        // local t = {}; return t
        let interner = StringInterner::new();
        let arena = bumpalo::Bump::new();
        let t_id = interner.get_or_intern("t");

        let ret_values = arena.alloc_slice_clone(&[Expression {
            kind: ExpressionKind::Identifier(t_id),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: None,
        }]);

        let stmts = vec![
            Statement::Variable(VariableDeclaration {
                kind: VariableKind::Local,
                pattern: Pattern::Identifier(make_ident(&interner, "t")),
                type_annotation: None,
                initializer: Expression {
                    kind: ExpressionKind::Object(&[]),
                    span: Span::dummy(),
                    annotated_type: None,
                    receiver_class: None,
                },
                span: Span::dummy(),
            }),
            Statement::Return(ReturnStatement {
                values: ret_values,
                span: Span::dummy(),
            }),
        ];

        let analyzer = AliasAnalyzer::new();
        let info = analyzer.analyze(&stmts);

        assert!(info.has_escaped(t_id), "t should escape when returned");
    }
}
