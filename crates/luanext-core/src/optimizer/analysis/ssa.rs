//! Static Single Assignment (SSA) form construction.
//!
//! SSA form assigns each variable a unique version at each definition point and
//! inserts phi-functions at join points where multiple definitions could reach.
//!
//! Required for: aggressive constant propagation, copy propagation, common
//! subexpression elimination (CSE).
//!
//! Uses the Cytron et al. algorithm:
//! 1. Collect variable definitions per block
//! 2. Place phi-functions using dominance frontiers (iterative worklist)
//! 3. Rename variables via dominator tree preorder walk

use super::cfg::{BlockId, ControlFlowGraph};
use super::dominance::DominatorTree;
use luanext_parser::ast::expression::ExpressionKind;
use luanext_parser::ast::pattern::Pattern;
use luanext_parser::ast::statement::{ForStatement, Statement};
use luanext_parser::string_interner::StringId;
use rustc_hash::{FxHashMap, FxHashSet};

/// A versioned SSA variable: original name + version number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SsaVar {
    /// The original variable name (interned string).
    pub name: StringId,
    /// The SSA version number (0 = initial/undefined, 1+ = definitions).
    pub version: u32,
}

impl std::fmt::Display for SsaVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{:?}_{}", self.name, self.version)
    }
}

/// A phi-function at a join point.
///
/// `phi(target) = { (block_i, operand_i) }` means: the value of the variable
/// at this point is `operand_i` if control came from `block_i`.
#[derive(Debug, Clone)]
pub struct PhiFunction {
    /// The SSA variable this phi defines.
    pub target: SsaVar,
    /// For each predecessor block, the SSA version of the variable coming from
    /// that block.
    pub operands: Vec<(BlockId, SsaVar)>,
}

/// SSA form for a single function or top-level scope.
///
/// This is a parallel data structure — it does NOT modify the AST. Optimization
/// passes query this mapping to understand variable versioning and data flow.
#[derive(Debug)]
pub struct SsaForm {
    /// Phi-functions at each block's entry, keyed by block ID.
    pub phi_functions: FxHashMap<BlockId, Vec<PhiFunction>>,
    /// For each statement index, the SSA variables it defines.
    pub definitions: FxHashMap<usize, Vec<SsaVar>>,
    /// For each statement index, the SSA variables it uses.
    pub uses: FxHashMap<usize, Vec<SsaVar>>,
    /// Current version counter for each original variable (for generating new versions).
    pub version_counters: FxHashMap<StringId, u32>,
    /// Reaching definition: for each (block, variable), the SSA version that is
    /// live at the end of that block.
    pub reaching_defs: FxHashMap<(BlockId, StringId), SsaVar>,
    /// All variables tracked by this SSA form.
    pub all_variables: FxHashSet<StringId>,
}

impl SsaForm {
    /// Build SSA form from a CFG, dominator tree, and statement list.
    ///
    /// The `statements` parameter is the original statement list that the CFG
    /// was built from. Statement indices in the CFG reference into this slice.
    pub fn build(
        cfg: &ControlFlowGraph,
        dom_tree: &DominatorTree,
        statements: &[Statement<'_>],
    ) -> Self {
        let mut builder = SsaBuilder::new(cfg, dom_tree);
        builder.collect_definitions(cfg, statements);
        builder.place_phi_functions(cfg, dom_tree);
        builder.rename_variables(cfg, dom_tree, statements);
        builder.finalize()
    }

    /// Returns the phi-functions at a block's entry.
    pub fn phis_at(&self, block: BlockId) -> &[PhiFunction] {
        self.phi_functions
            .get(&block)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Returns the SSA variables defined by a statement.
    pub fn defs_at(&self, stmt_index: usize) -> &[SsaVar] {
        self.definitions
            .get(&stmt_index)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Returns the SSA variables used by a statement.
    pub fn uses_at(&self, stmt_index: usize) -> &[SsaVar] {
        self.uses
            .get(&stmt_index)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Returns the reaching definition of a variable at the end of a block.
    pub fn reaching_def(&self, block: BlockId, var: StringId) -> Option<SsaVar> {
        self.reaching_defs.get(&(block, var)).copied()
    }
}

/// Internal builder for SSA construction.
struct SsaBuilder<'a> {
    /// Which variables are defined in each block.
    defs_in_block: FxHashMap<BlockId, FxHashSet<StringId>>,
    /// All variables found in the program.
    all_vars: FxHashSet<StringId>,
    /// Where phi-functions have been placed: (block, variable).
    phi_placed: FxHashSet<(BlockId, StringId)>,
    /// The phi-functions per block.
    phi_functions: FxHashMap<BlockId, Vec<PhiFunction>>,
    /// Version counters per variable.
    version_counters: FxHashMap<StringId, u32>,
    /// Stack of SSA versions per variable (for renaming walk).
    version_stacks: FxHashMap<StringId, Vec<u32>>,
    /// Per-statement definitions.
    definitions: FxHashMap<usize, Vec<SsaVar>>,
    /// Per-statement uses.
    uses: FxHashMap<usize, Vec<SsaVar>>,
    /// Reaching definitions at end of each block.
    reaching_defs: FxHashMap<(BlockId, StringId), SsaVar>,

    _cfg: &'a ControlFlowGraph,
    _dom: &'a DominatorTree,
}

impl<'a> SsaBuilder<'a> {
    fn new(cfg: &'a ControlFlowGraph, dom: &'a DominatorTree) -> Self {
        SsaBuilder {
            defs_in_block: FxHashMap::default(),
            all_vars: FxHashSet::default(),
            phi_placed: FxHashSet::default(),
            phi_functions: FxHashMap::default(),
            version_counters: FxHashMap::default(),
            version_stacks: FxHashMap::default(),
            definitions: FxHashMap::default(),
            uses: FxHashMap::default(),
            reaching_defs: FxHashMap::default(),
            _cfg: cfg,
            _dom: dom,
        }
    }

    /// Step 1: Collect which variables are defined in each block.
    fn collect_definitions(&mut self, cfg: &ControlFlowGraph, statements: &[Statement<'_>]) {
        for block in &cfg.blocks {
            let mut block_defs = FxHashSet::default();
            for &stmt_idx in &block.statement_indices {
                if stmt_idx < statements.len() {
                    self.collect_stmt_defs(&statements[stmt_idx], &mut block_defs);
                }
            }
            if !block_defs.is_empty() {
                self.defs_in_block.insert(block.id, block_defs);
            }
        }
    }

    /// Collect variable names defined by a statement.
    fn collect_stmt_defs(&mut self, stmt: &Statement<'_>, block_defs: &mut FxHashSet<StringId>) {
        match stmt {
            Statement::Variable(decl) => {
                self.collect_pattern_names(&decl.pattern, block_defs);
            }
            Statement::Function(func) => {
                let name = func.name.node;
                block_defs.insert(name);
                self.all_vars.insert(name);
            }
            Statement::For(for_stmt) => match &**for_stmt {
                ForStatement::Numeric(for_num) => {
                    let name = for_num.variable.node;
                    block_defs.insert(name);
                    self.all_vars.insert(name);
                }
                ForStatement::Generic(for_gen) => {
                    for var in for_gen.variables.iter() {
                        block_defs.insert(var.node);
                        self.all_vars.insert(var.node);
                    }
                }
            },
            Statement::Expression(expr) => {
                if let ExpressionKind::Assignment(target, _, _) = &expr.kind {
                    if let ExpressionKind::Identifier(name) = &target.kind {
                        block_defs.insert(*name);
                        self.all_vars.insert(*name);
                    }
                }
            }
            _ => {}
        }
    }

    /// Extract variable names from a pattern.
    fn collect_pattern_names(
        &mut self,
        pattern: &Pattern<'_>,
        block_defs: &mut FxHashSet<StringId>,
    ) {
        match pattern {
            Pattern::Identifier(ident) => {
                block_defs.insert(ident.node);
                self.all_vars.insert(ident.node);
            }
            Pattern::Array(arr_pat) => {
                for elem in arr_pat.elements.iter() {
                    if let luanext_parser::ast::pattern::ArrayPatternElement::Pattern(pwd) = elem {
                        self.collect_pattern_names(&pwd.pattern, block_defs);
                    }
                }
            }
            Pattern::Object(obj_pat) => {
                for prop in obj_pat.properties.iter() {
                    if let Some(ref pat) = prop.value {
                        self.collect_pattern_names(pat, block_defs);
                    } else {
                        // Shorthand: { x } means x = x
                        block_defs.insert(prop.key.node);
                        self.all_vars.insert(prop.key.node);
                    }
                }
            }
            Pattern::Wildcard(_)
            | Pattern::Literal(_, _)
            | Pattern::Or(_)
            | Pattern::Template(_) => {}
        }
    }

    /// Step 2: Place phi-functions using dominance frontiers.
    fn place_phi_functions(&mut self, cfg: &ControlFlowGraph, dom_tree: &DominatorTree) {
        // For each variable, place phis at the dominance frontier of blocks that define it
        for var in self.all_vars.clone() {
            // Worklist: blocks where this variable is defined
            let mut worklist: Vec<BlockId> = self
                .defs_in_block
                .iter()
                .filter(|(_, defs)| defs.contains(&var))
                .map(|(&block, _)| block)
                .collect();

            let mut processed: FxHashSet<BlockId> = FxHashSet::default();

            while let Some(block) = worklist.pop() {
                if !processed.insert(block) {
                    continue;
                }

                for &frontier_block in dom_tree.frontier(block) {
                    if self.phi_placed.insert((frontier_block, var)) {
                        // Place a phi-function at frontier_block for this variable
                        let preds = cfg.preds(frontier_block);
                        let phi = PhiFunction {
                            target: SsaVar {
                                name: var,
                                version: 0, // Will be set during renaming
                            },
                            operands: preds
                                .iter()
                                .map(|&pred| {
                                    (
                                        pred,
                                        SsaVar {
                                            name: var,
                                            version: 0, // Will be set during renaming
                                        },
                                    )
                                })
                                .collect(),
                        };
                        self.phi_functions
                            .entry(frontier_block)
                            .or_default()
                            .push(phi);

                        // The phi defines the variable, so add to worklist
                        worklist.push(frontier_block);
                    }
                }
            }
        }
    }

    /// Step 3: Rename variables using dominator tree preorder walk.
    fn rename_variables(
        &mut self,
        cfg: &ControlFlowGraph,
        dom_tree: &DominatorTree,
        statements: &[Statement<'_>],
    ) {
        // Initialize version counters and stacks
        for &var in &self.all_vars {
            self.version_counters.insert(var, 0);
            self.version_stacks.insert(var, vec![0]); // Version 0 = undefined/initial
        }

        // Walk the dominator tree in preorder
        self.rename_block(BlockId::ENTRY, cfg, dom_tree, statements);
    }

    fn rename_block(
        &mut self,
        block: BlockId,
        cfg: &ControlFlowGraph,
        dom_tree: &DominatorTree,
        statements: &[Statement<'_>],
    ) {
        // Track how many versions we push for each variable (to pop later)
        let mut pushed: Vec<(StringId, usize)> = Vec::new();

        // Process phi-functions: each phi defines a new version
        // Collect phi variable names first, then allocate new versions to avoid double borrow
        let phi_vars: Vec<StringId> = self
            .phi_functions
            .get(&block)
            .map(|phis| phis.iter().map(|phi| phi.target.name).collect())
            .unwrap_or_default();

        for (i, var) in phi_vars.iter().enumerate() {
            let new_version = self.new_version(*var);
            let stack = self.version_stacks.get(var).unwrap();
            pushed.push((*var, stack.len()));
            if let Some(phis) = self.phi_functions.get_mut(&block) {
                phis[i].target.version = new_version;
            }
        }

        // Process statements in this block
        if let Some(cfg_block) = cfg.block(block) {
            for &stmt_idx in &cfg_block.statement_indices {
                if stmt_idx < statements.len() {
                    // First rename uses (read current version)
                    let stmt_uses = self.collect_stmt_uses(&statements[stmt_idx]);
                    let ssa_uses: Vec<SsaVar> = stmt_uses
                        .iter()
                        .map(|&var| SsaVar {
                            name: var,
                            version: self.current_version(var),
                        })
                        .collect();
                    if !ssa_uses.is_empty() {
                        self.uses.insert(stmt_idx, ssa_uses);
                    }

                    // Then rename definitions (create new versions)
                    let stmt_defs = self.collect_stmt_def_names(&statements[stmt_idx]);
                    let ssa_defs: Vec<SsaVar> = stmt_defs
                        .iter()
                        .map(|&var| {
                            let new_version = self.new_version(var);
                            let stack = self.version_stacks.get(&var).unwrap();
                            pushed.push((var, stack.len()));
                            SsaVar {
                                name: var,
                                version: new_version,
                            }
                        })
                        .collect();
                    if !ssa_defs.is_empty() {
                        self.definitions.insert(stmt_idx, ssa_defs);
                    }
                }
            }
        }

        // Record reaching definitions at end of this block
        for &var in &self.all_vars {
            let version = self.current_version(var);
            self.reaching_defs
                .insert((block, var), SsaVar { name: var, version });
        }

        // Fill in phi-function operands in successor blocks
        // Collect current versions first, then apply to phi operands to avoid double borrow
        for &succ in cfg.succs(block) {
            let phi_vars_in_succ: Vec<StringId> = self
                .phi_functions
                .get(&succ)
                .map(|phis| phis.iter().map(|phi| phi.target.name).collect())
                .unwrap_or_default();

            let versions: Vec<u32> = phi_vars_in_succ
                .iter()
                .map(|var| self.current_version(*var))
                .collect();

            if let Some(phis) = self.phi_functions.get_mut(&succ) {
                for (phi, &version) in phis.iter_mut().zip(versions.iter()) {
                    for operand in &mut phi.operands {
                        if operand.0 == block {
                            operand.1.version = version;
                        }
                    }
                }
            }
        }

        // Recurse into dominator tree children
        let children: Vec<BlockId> = dom_tree.children.get(&block).cloned().unwrap_or_default();
        for child in children {
            self.rename_block(child, cfg, dom_tree, statements);
        }

        // Pop the versions we pushed
        for (var, target_len) in pushed.into_iter().rev() {
            if let Some(stack) = self.version_stacks.get_mut(&var) {
                while stack.len() > target_len {
                    stack.pop();
                }
            }
        }
    }

    /// Allocate a new version for a variable and push it onto the stack.
    fn new_version(&mut self, var: StringId) -> u32 {
        let counter = self.version_counters.entry(var).or_insert(0);
        *counter += 1;
        let version = *counter;
        self.version_stacks.entry(var).or_default().push(version);
        version
    }

    /// Get the current (top of stack) version for a variable.
    fn current_version(&self, var: StringId) -> u32 {
        self.version_stacks
            .get(&var)
            .and_then(|stack| stack.last().copied())
            .unwrap_or(0)
    }

    /// Collect variable names used by a statement (read before write).
    fn collect_stmt_uses(&self, stmt: &Statement<'_>) -> Vec<StringId> {
        let mut uses = Vec::new();
        match stmt {
            Statement::Variable(decl) => {
                self.collect_expr_uses(&decl.initializer, &mut uses);
            }
            Statement::Expression(expr) => {
                self.collect_expr_uses(expr, &mut uses);
            }
            Statement::Return(ret) => {
                for val in ret.values.iter() {
                    self.collect_expr_uses(val, &mut uses);
                }
            }
            _ => {}
        }
        uses
    }

    /// Collect variable names used in an expression.
    fn collect_expr_uses(
        &self,
        expr: &luanext_parser::ast::expression::Expression<'_>,
        uses: &mut Vec<StringId>,
    ) {
        match &expr.kind {
            ExpressionKind::Identifier(name) => {
                if self.all_vars.contains(name) && !uses.contains(name) {
                    uses.push(*name);
                }
            }
            ExpressionKind::Binary(_, left, right) => {
                self.collect_expr_uses(left, uses);
                self.collect_expr_uses(right, uses);
            }
            ExpressionKind::Unary(_, operand) => {
                self.collect_expr_uses(operand, uses);
            }
            ExpressionKind::Assignment(target, _, value) => {
                // The RHS is a use; the LHS target may also read (for compound assignment)
                self.collect_expr_uses(value, uses);
                // For compound assignments (+=, -=, etc.), the target is also a use
                if let ExpressionKind::Identifier(name) = &target.kind {
                    if self.all_vars.contains(name) && !uses.contains(name) {
                        uses.push(*name);
                    }
                }
            }
            ExpressionKind::Call(func, args, _) => {
                self.collect_expr_uses(func, uses);
                for arg in args.iter() {
                    self.collect_expr_uses(&arg.value, uses);
                }
            }
            ExpressionKind::MethodCall(obj, _, args, _) => {
                self.collect_expr_uses(obj, uses);
                for arg in args.iter() {
                    self.collect_expr_uses(&arg.value, uses);
                }
            }
            ExpressionKind::Member(obj, _) => {
                self.collect_expr_uses(obj, uses);
            }
            ExpressionKind::Index(obj, idx) => {
                self.collect_expr_uses(obj, uses);
                self.collect_expr_uses(idx, uses);
            }
            ExpressionKind::Conditional(cond, then_expr, else_expr) => {
                self.collect_expr_uses(cond, uses);
                self.collect_expr_uses(then_expr, uses);
                self.collect_expr_uses(else_expr, uses);
            }
            ExpressionKind::Parenthesized(inner) => {
                self.collect_expr_uses(inner, uses);
            }
            ExpressionKind::Pipe(left, right) => {
                self.collect_expr_uses(left, uses);
                self.collect_expr_uses(right, uses);
            }
            // Leaf nodes and complex expressions we don't need to track
            _ => {}
        }
    }

    /// Collect variable names defined by a statement.
    fn collect_stmt_def_names(&self, stmt: &Statement<'_>) -> Vec<StringId> {
        let mut defs = Vec::new();
        match stmt {
            Statement::Variable(decl) => {
                self.collect_pattern_name_list(&decl.pattern, &mut defs);
            }
            Statement::Function(func) => {
                defs.push(func.name.node);
            }
            Statement::Expression(expr) => {
                if let ExpressionKind::Assignment(target, _, _) = &expr.kind {
                    if let ExpressionKind::Identifier(name) = &target.kind {
                        defs.push(*name);
                    }
                }
            }
            _ => {}
        }
        defs
    }

    fn collect_pattern_name_list(&self, pattern: &Pattern<'_>, names: &mut Vec<StringId>) {
        match pattern {
            Pattern::Identifier(ident) => {
                names.push(ident.node);
            }
            Pattern::Array(arr_pat) => {
                for elem in arr_pat.elements.iter() {
                    if let luanext_parser::ast::pattern::ArrayPatternElement::Pattern(pwd) = elem {
                        self.collect_pattern_name_list(&pwd.pattern, names);
                    }
                }
            }
            Pattern::Object(obj_pat) => {
                for prop in obj_pat.properties.iter() {
                    if let Some(ref pat) = prop.value {
                        self.collect_pattern_name_list(pat, names);
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

    fn finalize(self) -> SsaForm {
        SsaForm {
            phi_functions: self.phi_functions,
            definitions: self.definitions,
            uses: self.uses,
            version_counters: self.version_counters,
            reaching_defs: self.reaching_defs,
            all_variables: self.all_vars,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer::analysis::cfg::CfgBuilder;
    use crate::optimizer::analysis::dominance::DominatorTree;
    use luanext_parser::ast::expression::{Expression, ExpressionKind, Literal};
    use luanext_parser::ast::statement::{
        Block, IfStatement, Statement, VariableDeclaration, VariableKind, WhileStatement,
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

    fn make_expr_true() -> Expression<'static> {
        Expression {
            kind: ExpressionKind::Literal(Literal::Boolean(true)),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: None,
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

    fn make_var_decl<'a>(interner: &StringInterner, name: &str) -> Statement<'a> {
        Statement::Variable(VariableDeclaration {
            kind: VariableKind::Local,
            pattern: Pattern::Identifier(make_ident(interner, name)),
            type_annotation: None,
            initializer: make_expr_nil(),
            span: Span::new(0, 10, 1, 1),
        })
    }

    fn empty_block() -> Block<'static> {
        Block {
            statements: &[],
            span: Span::dummy(),
        }
    }

    #[test]
    fn test_linear_code_no_phis() {
        // local x = nil; local y = nil
        // Linear code — no phi-functions needed
        let interner = StringInterner::new();
        let stmts = vec![make_var_decl(&interner, "x"), make_var_decl(&interner, "y")];

        let cfg = CfgBuilder::build(&stmts);
        let dom_tree = DominatorTree::build(&cfg);
        let ssa = SsaForm::build(&cfg, &dom_tree, &stmts);

        // No phi-functions in linear code
        for phis in ssa.phi_functions.values() {
            assert!(phis.is_empty(), "Linear code should have no phi-functions");
        }

        // Both x and y should be tracked
        assert!(ssa.all_variables.contains(&interner.get_or_intern("x")));
        assert!(ssa.all_variables.contains(&interner.get_or_intern("y")));
    }

    #[test]
    fn test_if_else_cfg_structure() {
        // local x = nil
        // if true then <body> else <body> end
        // Verify the CFG creates the expected diamond structure with join block
        let interner = StringInterner::new();
        let arena = bumpalo::Bump::new();

        let x_id = interner.get_or_intern("x");

        let then_stmts = arena.alloc_slice_clone(&[make_var_decl(&interner, "a")]);
        let then_block = Block {
            statements: then_stmts,
            span: Span::new(15, 30, 2, 1),
        };

        let else_stmts = arena.alloc_slice_clone(&[make_var_decl(&interner, "b")]);
        let else_block = Block {
            statements: else_stmts,
            span: Span::new(30, 45, 3, 1),
        };

        let stmts = vec![
            make_var_decl(&interner, "x"),
            Statement::If(IfStatement {
                condition: make_expr_true(),
                then_block,
                else_ifs: &[],
                else_block: Some(else_block),
                span: Span::new(10, 50, 2, 1),
            }),
        ];

        let cfg = CfgBuilder::build(&stmts);
        let dom_tree = DominatorTree::build(&cfg);
        let ssa = SsaForm::build(&cfg, &dom_tree, &stmts);

        // x should be tracked
        assert!(ssa.all_variables.contains(&x_id));

        // The if/else creates a diamond structure in the CFG:
        // at least ENTRY, code block, then, else, join, EXIT = 6 blocks
        assert!(
            cfg.block_count() >= 5,
            "If/else should create at least 5 blocks (ENTRY, code, then, else, join+EXIT), got {}",
            cfg.block_count()
        );

        // x should have version 1 (defined once at top level)
        assert_eq!(
            *ssa.version_counters.get(&x_id).unwrap_or(&0),
            1,
            "x should be at version 1"
        );
    }

    #[test]
    fn test_versioning() {
        // local x = nil
        // local x = nil (redefinition)
        // Each definition should get a unique version
        let interner = StringInterner::new();
        let stmts = vec![make_var_decl(&interner, "x"), make_var_decl(&interner, "x")];

        let cfg = CfgBuilder::build(&stmts);
        let dom_tree = DominatorTree::build(&cfg);
        let ssa = SsaForm::build(&cfg, &dom_tree, &stmts);

        let x_id = interner.get_or_intern("x");

        // Should have version counter at 2 for x (two definitions)
        assert!(
            *ssa.version_counters.get(&x_id).unwrap_or(&0) >= 2,
            "x should have at least 2 versions"
        );
    }

    #[test]
    fn test_loop_cfg_structure() {
        // local x = nil
        // while true do <body> end
        // The loop should create header + body + exit blocks with back-edge
        let interner = StringInterner::new();
        let arena = bumpalo::Bump::new();

        let inner_stmts = arena.alloc_slice_clone(&[make_var_decl(&interner, "y")]);
        let loop_body = Block {
            statements: inner_stmts,
            span: Span::new(20, 40, 2, 5),
        };

        let stmts = vec![
            make_var_decl(&interner, "x"),
            Statement::While(WhileStatement {
                condition: make_expr_true(),
                body: loop_body,
                span: Span::new(10, 45, 2, 1),
            }),
        ];

        let cfg = CfgBuilder::build(&stmts);
        let dom_tree = DominatorTree::build(&cfg);
        let ssa = SsaForm::build(&cfg, &dom_tree, &stmts);

        // x should be tracked
        let x_id = interner.get_or_intern("x");
        assert!(ssa.all_variables.contains(&x_id));

        // A while loop should create a loop header
        assert!(
            !cfg.loop_headers.is_empty(),
            "While loop should create at least one loop header"
        );

        // x should have a reaching definition in the block that defines it
        let code_block = cfg
            .blocks
            .iter()
            .find(|b| !b.statement_indices.is_empty())
            .unwrap();
        let reaching = ssa.reaching_def(code_block.id, x_id);
        assert!(reaching.is_some(), "x should have a reaching definition");
    }

    #[test]
    fn test_multiple_variables() {
        // local x = nil; local y = nil
        // Both variables should be independently versioned
        let interner = StringInterner::new();
        let stmts = vec![make_var_decl(&interner, "x"), make_var_decl(&interner, "y")];

        let cfg = CfgBuilder::build(&stmts);
        let dom_tree = DominatorTree::build(&cfg);
        let ssa = SsaForm::build(&cfg, &dom_tree, &stmts);

        let x_id = interner.get_or_intern("x");
        let y_id = interner.get_or_intern("y");

        // Both should be tracked
        assert!(ssa.all_variables.contains(&x_id));
        assert!(ssa.all_variables.contains(&y_id));

        // Both should have version 1
        assert_eq!(*ssa.version_counters.get(&x_id).unwrap_or(&0), 1);
        assert_eq!(*ssa.version_counters.get(&y_id).unwrap_or(&0), 1);
    }

    #[test]
    fn test_reaching_definitions() {
        // local x = nil
        // Simple reaching def: x should have version 1 at the end of the code block
        let interner = StringInterner::new();
        let stmts = vec![make_var_decl(&interner, "x")];

        let cfg = CfgBuilder::build(&stmts);
        let dom_tree = DominatorTree::build(&cfg);
        let ssa = SsaForm::build(&cfg, &dom_tree, &stmts);

        let x_id = interner.get_or_intern("x");

        // Find the code block (not ENTRY or EXIT)
        let code_block = cfg
            .blocks
            .iter()
            .find(|b| !b.statement_indices.is_empty())
            .unwrap();

        let reaching = ssa.reaching_def(code_block.id, x_id);
        assert!(reaching.is_some(), "x should have a reaching definition");
        assert_eq!(
            reaching.unwrap().version,
            1,
            "x should be at version 1 after definition"
        );
    }

    #[test]
    fn test_phi_operand_correctness() {
        // if true then local x = nil end
        // The phi at the join should have operands from both branches
        let interner = StringInterner::new();
        let arena = bumpalo::Bump::new();

        let inner_stmts = arena.alloc_slice_clone(&[make_var_decl(&interner, "x")]);
        let then_block = Block {
            statements: inner_stmts,
            span: Span::new(15, 30, 2, 1),
        };

        let stmts = vec![
            make_var_decl(&interner, "x"),
            Statement::If(IfStatement {
                condition: make_expr_true(),
                then_block,
                else_ifs: &[],
                else_block: Some(empty_block()),
                span: Span::new(10, 50, 2, 1),
            }),
        ];

        let cfg = CfgBuilder::build(&stmts);
        let dom_tree = DominatorTree::build(&cfg);
        let ssa = SsaForm::build(&cfg, &dom_tree, &stmts);

        // Check that any phis have the correct number of operands
        for (block_id, phis) in &ssa.phi_functions {
            let preds = cfg.preds(*block_id);
            for phi in phis {
                assert_eq!(
                    phi.operands.len(),
                    preds.len(),
                    "Phi at {} should have one operand per predecessor",
                    block_id
                );
            }
        }
    }
}
