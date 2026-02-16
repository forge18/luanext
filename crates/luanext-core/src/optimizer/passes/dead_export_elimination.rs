use crate::optimizer::analysis::module_graph::ModuleGraph;
use luanext_parser::ast::statement::{ExportKind, Statement};
use luanext_parser::string_interner::StringInterner;
use std::path::Path;
use std::sync::Arc;

/// Dead Export Elimination Pass
///
/// Removes export statements for symbols that are never imported by any other module.
/// This pass is conservative - it only removes the export statement itself, not the
/// underlying function/variable definition (which may still be used locally).
///
/// ## Example
/// ```luanext
/// // Before (if `unusedFunc` is never imported):
/// export function unusedFunc() { return 42; }
/// function usedLocally() { return unusedFunc(); }
///
/// // After:
/// function unusedFunc() { return 42; }  // Export removed, definition kept
/// function usedLocally() { return unusedFunc(); }
/// ```
///
/// ## Optimization Level
/// - O0/O1: Disabled
/// - O2: Enabled for non-entry modules
/// - O3: Enabled for all modules
pub struct DeadExportEliminationPass {
    module_graph: Arc<ModuleGraph>,
    interner: Arc<StringInterner>,
    current_module_path: Option<std::path::PathBuf>,
}

impl DeadExportEliminationPass {
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

    /// Apply dead export elimination to a list of statements
    ///
    /// Removes export statements for symbols that are never imported by any other module.
    /// This pass only removes the export wrapper - the underlying declaration is kept if
    /// it might be used locally.
    ///
    /// Returns a new Vec with dead exports filtered out.
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
                Statement::Export(export_decl) => {
                    match &export_decl.kind {
                        ExportKind::Declaration(inner_stmt) => {
                            // Extract the symbol name from the declaration
                            let symbol_id = self.get_declaration_name(inner_stmt);

                            if let Some(id) = symbol_id {
                                let name_str = self.interner.resolve(id);
                                // Check if this export is used
                                if let Some(export_info) = module_node.exports.get(&name_str) {
                                    if export_info.is_used {
                                        // Export is used, keep it
                                        result.push(stmt.clone());
                                    } else {
                                        // Export is dead, unwrap to just the declaration
                                        result.push((*inner_stmt).clone());
                                    }
                                } else {
                                    // Not in graph, conservatively keep it
                                    result.push(stmt.clone());
                                }
                            } else {
                                // Can't extract name, conservatively keep it
                                result.push(stmt.clone());
                            }
                        }
                        ExportKind::Named { specifiers, source, .. } => {
                            if source.is_none() {
                                // Local named exports: export { foo, bar };
                                // Check if any specifier is used
                                let any_used = specifiers
                                    .iter()
                                    .any(|spec| {
                                        let export_name = spec.exported.as_ref().unwrap_or(&spec.local);
                                        let name_str = self.interner.resolve(export_name.node);
                                        module_node.exports.get(&name_str)
                                            .map(|info| info.is_used)
                                            .unwrap_or(true) // Conservative: keep if not in graph
                                    });

                                if any_used {
                                    // Keep statement if any exports are used
                                    // Note: We can't modify arena-allocated slices easily,
                                    // so we keep all specifiers if any are used
                                    result.push(stmt.clone());
                                }
                                // If all dead, drop the entire export statement
                            } else {
                                // Re-export: export { x } from './other'
                                // Keep re-exports for now (handled by ReExportFlatteningPass)
                                result.push(stmt.clone());
                            }
                        }
                        ExportKind::All { .. } | ExportKind::Default(_) => {
                            // Keep export * and default exports for now
                            // These require more sophisticated analysis
                            result.push(stmt.clone());
                        }
                    }
                }
                _ => {
                    // Non-export statement, keep it
                    result.push(stmt.clone());
                }
            }
        }

        result
    }

    /// Extract the symbol name from a declaration statement
    fn get_declaration_name(&self, stmt: &Statement<'_>) -> Option<luanext_parser::string_interner::StringId> {
        match stmt {
            Statement::Function(func) => Some(func.name.node),
            Statement::Variable(var) => {
                // For now, only handle simple identifier patterns
                use luanext_parser::ast::pattern::Pattern;
                match &var.pattern {
                    Pattern::Identifier(ident) => Some(ident.node),
                    _ => None, // Complex patterns not supported yet
                }
            }
            Statement::Class(class) => Some(class.name.node),
            Statement::Enum(enum_decl) => Some(enum_decl.name.node),
            Statement::Interface(interface) => Some(interface.name.node),
            Statement::TypeAlias(alias) => Some(alias.name.node),
            _ => None,
        }
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

        let interner = Arc::new(luanext_parser::string_interner::StringInterner::new());
        let pass = DeadExportEliminationPass::new(Arc::new(graph), interner);
        assert!(pass.current_module_path.is_none());
    }

    #[test]
    fn test_set_current_module() {
        let graph = ModuleGraph {
            modules: FxHashMap::default(),
            entry_points: Default::default(),
        };

        let interner = Arc::new(luanext_parser::string_interner::StringInterner::new());
        let mut pass = DeadExportEliminationPass::new(Arc::new(graph), interner);
        let module_path = PathBuf::from("test.luax");
        pass.set_current_module(&module_path);

        assert_eq!(pass.current_module_path, Some(module_path));
    }
}
