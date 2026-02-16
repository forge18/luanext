//! Advanced optimizer analysis infrastructure.
//!
//! This module provides program analysis data structures that are consumed by
//! optimization passes but do not modify the AST themselves. The analyses
//! form a dependency chain:
//!
//! ```text
//! CFG (needs only AST)
//!  └─> Dominance (needs CFG)
//!       └─> SSA (needs CFG + Dominance)
//!
//! Alias Analysis (needs only AST, independent)
//! Side-Effect Analysis (needs only AST, independent)
//! ```
//!
//! All analysis data structures use `usize` statement indices and `StringId`
//! variable names rather than `&'arena` AST references, keeping them decoupled
//! from arena lifetimes.

pub mod alias;
pub mod cfg;
pub mod dominance;
pub mod module_graph;
pub mod side_effect;
pub mod ssa;

pub use alias::{AliasAnalyzer, AliasInfo, AliasResult, MemoryLocation};
pub use cfg::{BasicBlock, BlockId, CfgBuilder, ControlFlowGraph, Terminator};
pub use dominance::DominatorTree;
pub use module_graph::{
    ExportInfo, ImportInfo, ModuleGraph, ModuleNode, ReExportInfo, ReExportKind,
};
pub use side_effect::{SideEffectAnalyzer, SideEffectInfo, SideEffects};
pub use ssa::{PhiFunction, SsaForm, SsaVar};

use crate::MutableProgram;
use luanext_parser::ast::statement::Statement;
use luanext_parser::string_interner::{StringId, StringInterner};
use rustc_hash::FxHashMap;
use std::sync::Arc;

/// Per-function analysis results (CFG + dominance + SSA + alias).
#[derive(Debug)]
pub struct FunctionAnalysis {
    /// Control flow graph for this function.
    pub cfg: ControlFlowGraph,
    /// Dominator tree computed from the CFG.
    pub dominators: DominatorTree,
    /// SSA form with phi-functions and versioned variables.
    pub ssa: SsaForm,
    /// Alias information for variables in this function.
    pub alias_info: AliasInfo,
}

/// Program-wide analysis context.
///
/// Holds per-function analyses and program-wide side-effect information.
/// Computed once before the optimizer's fixed-point loop, then read by
/// optimization passes that need CFG/SSA/alias/side-effect data.
#[derive(Debug)]
pub struct AnalysisContext {
    /// Per-function analyses, keyed by function name (`StringId`).
    /// The top-level scope uses a sentinel key from `top_level_key()`.
    function_analyses: FxHashMap<StringId, FunctionAnalysis>,
    /// Program-wide side-effect analysis.
    side_effects: Option<SideEffectInfo>,
    /// The sentinel key used for the top-level scope.
    top_level_key: Option<StringId>,
}

impl AnalysisContext {
    /// Create an empty analysis context.
    pub fn new() -> Self {
        AnalysisContext {
            function_analyses: FxHashMap::default(),
            side_effects: None,
            top_level_key: None,
        }
    }

    /// Compute all analyses for the given program.
    ///
    /// This builds CFG, dominance tree, SSA form, and alias info for each
    /// function, plus program-wide side-effect analysis.
    pub fn compute(
        &mut self,
        program: &MutableProgram<'_>,
        interner: Arc<StringInterner>,
    ) -> Result<(), String> {
        let statements = &program.statements;
        let top_key = interner.get_or_intern("<top-level>");
        self.top_level_key = Some(top_key);

        // Analyze top-level scope
        let top_analysis = Self::analyze_function_body(statements);
        self.function_analyses.insert(top_key, top_analysis);

        // Analyze each function declaration
        for stmt in statements.iter() {
            if let Statement::Function(func) = stmt {
                let body_stmts = func.body.statements;
                let func_analysis = Self::analyze_function_body(body_stmts);
                self.function_analyses.insert(func.name.node, func_analysis);
            }
        }

        // Side-effect analysis (program-wide)
        let se_analyzer = SideEffectAnalyzer::new(interner);
        self.side_effects = Some(se_analyzer.analyze(statements));

        Ok(())
    }

    /// Analyze a single function body: build CFG → dominance → SSA → alias.
    fn analyze_function_body(statements: &[Statement<'_>]) -> FunctionAnalysis {
        let cfg = CfgBuilder::build(statements);
        let dominators = DominatorTree::build(&cfg);
        let ssa = SsaForm::build(&cfg, &dominators, statements);
        let alias_info = AliasAnalyzer::new().analyze(statements);

        FunctionAnalysis {
            cfg,
            dominators,
            ssa,
            alias_info,
        }
    }

    /// Get the analysis for the top-level scope.
    pub fn top_level(&self) -> Option<&FunctionAnalysis> {
        self.top_level_key
            .and_then(|key| self.function_analyses.get(&key))
    }

    /// Get the analysis for a specific function by name.
    pub fn function_analysis(&self, name: StringId) -> Option<&FunctionAnalysis> {
        self.function_analyses.get(&name)
    }

    /// Get the top-level CFG.
    pub fn top_level_cfg(&self) -> Option<&ControlFlowGraph> {
        self.top_level().map(|a| &a.cfg)
    }

    /// Get program-wide side-effect info.
    pub fn side_effects(&self) -> Option<&SideEffectInfo> {
        self.side_effects.as_ref()
    }

    /// Returns all function names that have been analyzed.
    pub fn analyzed_functions(&self) -> Vec<StringId> {
        self.function_analyses.keys().copied().collect()
    }
}

impl Default for AnalysisContext {
    fn default() -> Self {
        Self::new()
    }
}
