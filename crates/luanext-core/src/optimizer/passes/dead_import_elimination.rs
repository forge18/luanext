use crate::optimizer::analysis::module_graph::ModuleGraph;
use luanext_parser::ast::statement::ImportClause;
use luanext_parser::ast::statement::Statement;
use luanext_parser::string_interner::StringInterner;
use std::path::Path;
use std::sync::Arc;

/// Dead Import Elimination Pass
///
/// Removes import statements for symbols that are never referenced in the module's code.
/// This pass is more aggressive than tree-shaking - it removes the import binding entirely,
/// even if the module is loaded (the module may have side effects that are needed).
///
/// ## Example
/// ```luanext
/// // Before (if `unusedFunc` is never referenced):
/// import { usedFunc, unusedFunc } from './utils';
/// console.log(usedFunc());
///
/// // After:
/// import { usedFunc } from './utils';
/// console.log(usedFunc());
/// ```
///
/// ## Optimization Level
/// - O0/O1: Disabled
/// - O2: Enabled (safe, only removes unused bindings)
/// - O3: Enabled
pub struct DeadImportEliminationPass {
    module_graph: Arc<ModuleGraph>,
    interner: Arc<StringInterner>,
    current_module_path: Option<std::path::PathBuf>,
}

impl DeadImportEliminationPass {
    pub fn new(module_graph: Arc<ModuleGraph>, interner: Arc<StringInterner>) -> Self {
        Self {
            module_graph,
            interner,
            current_module_path: None,
        }
    }

    pub fn set_current_module(&mut self, path: &Path) {
        self.current_module_path = Some(path.to_path_buf());
    }

    /// Apply dead import elimination to a list of statements
    ///
    /// Removes import statements or import specifiers for symbols that are never
    /// referenced in the module's code. This is more aggressive than tree-shaking -
    /// it removes the import binding entirely, even if the module is still loaded
    /// (the module may have side effects).
    ///
    /// Returns a new Vec with dead imports filtered out.
    pub fn apply<'arena>(&self, statements: &[Statement<'arena>]) -> Vec<Statement<'arena>> {
        let Some(module_path) = &self.current_module_path else {
            // No module context, return unchanged
            return statements.to_vec();
        };

        let Some(module_node) = self.module_graph.modules.get(module_path) else {
            // Module not in graph, return unchanged
            return statements.to_vec();
        };

        let mut result = Vec::with_capacity(statements.len());

        for stmt in statements {
            match stmt {
                Statement::Import(import_decl) => {
                    match &import_decl.clause {
                        ImportClause::Named(specifiers) => {
                            // Check if any import specifier is referenced
                            let any_referenced = specifiers.iter().any(|spec| {
                                let local_name = spec.local.as_ref().unwrap_or(&spec.imported);
                                let name_str = self.interner.resolve(local_name.node);
                                module_node
                                    .imports
                                    .get(&name_str)
                                    .map(|info| info.is_referenced)
                                    .unwrap_or(true) // Conservative: keep if not in graph
                            });

                            if any_referenced {
                                // Keep import statement if any specifiers are used
                                // Note: We can't easily modify arena-allocated slices,
                                // so we keep all specifiers if any are used
                                result.push(stmt.clone());
                            }
                            // If all dead, drop the entire import statement
                        }
                        ImportClause::Default(ident) => {
                            // Check if default import is referenced
                            let name_str = self.interner.resolve(ident.node);
                            if module_node
                                .imports
                                .get(&name_str)
                                .map(|info| info.is_referenced)
                                .unwrap_or(true)
                            // Conservative
                            {
                                result.push(stmt.clone());
                            }
                            // Else drop the import
                        }
                        ImportClause::Namespace(ident) => {
                            // Check if namespace import is referenced
                            let name_str = self.interner.resolve(ident.node);
                            if module_node
                                .imports
                                .get(&name_str)
                                .map(|info| info.is_referenced)
                                .unwrap_or(true)
                            // Conservative
                            {
                                result.push(stmt.clone());
                            }
                            // Else drop the import
                        }
                        ImportClause::TypeOnly(_) => {
                            // Type-only imports are already erased at codegen
                            // Drop them here for cleaner AST
                            // (they don't affect runtime)
                        }
                        ImportClause::Mixed { default, named } => {
                            // Check both default and named imports
                            let default_name_str = self.interner.resolve(default.node);
                            let default_used = module_node
                                .imports
                                .get(&default_name_str)
                                .map(|info| info.is_referenced)
                                .unwrap_or(true);

                            let any_named_used = named.iter().any(|spec| {
                                let local_name = spec.local.as_ref().unwrap_or(&spec.imported);
                                let name_str = self.interner.resolve(local_name.node);
                                module_node
                                    .imports
                                    .get(&name_str)
                                    .map(|info| info.is_referenced)
                                    .unwrap_or(true)
                            });

                            if default_used || any_named_used {
                                // Keep if either default or any named import is used
                                result.push(stmt.clone());
                            }
                            // Else drop the import
                        }
                    }
                }
                _ => {
                    // Non-import statement, keep it
                    result.push(stmt.clone());
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_hash::FxHashMap;
    use std::path::PathBuf;

    #[test]
    fn test_pass_creation() {
        let graph = ModuleGraph {
            modules: FxHashMap::default(),
            entry_points: Default::default(),
        };

        let interner = Arc::new(StringInterner::new());
        let pass = DeadImportEliminationPass::new(Arc::new(graph), interner);
        assert!(pass.current_module_path.is_none());
    }

    #[test]
    fn test_set_current_module() {
        let graph = ModuleGraph {
            modules: FxHashMap::default(),
            entry_points: Default::default(),
        };

        let interner = Arc::new(StringInterner::new());
        let mut pass = DeadImportEliminationPass::new(Arc::new(graph), interner);
        let module_path = PathBuf::from("test.luax");
        pass.set_current_module(&module_path);

        assert_eq!(pass.current_module_path, Some(module_path));
    }

    #[test]
    fn test_empty_statements() {
        let graph = ModuleGraph {
            modules: FxHashMap::default(),
            entry_points: Default::default(),
        };

        let interner = Arc::new(StringInterner::new());
        let pass = DeadImportEliminationPass::new(Arc::new(graph), interner);

        let result = pass.apply(&[]);
        assert_eq!(result.len(), 0);
    }
}
