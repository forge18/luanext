use crate::optimizer::analysis::module_graph::ModuleGraph;
#[allow(unused_imports)] // Used in tests and future implementation
use luanext_parser::ast::statement::{
    ExportDeclaration, ExportKind, ExportSpecifier, ImportClause, ImportDeclaration,
    ImportSpecifier, Statement,
};
use luanext_parser::string_interner::StringInterner;
use std::path::Path;
use std::sync::Arc;

/// Re-export Flattening Pass
///
/// Optimizes re-export chains by flattening them to reduce runtime `require()` overhead.
/// When module A re-exports from B, and B re-exports from C, this pass can flatten
/// A to directly import from C instead of going through B.
///
/// ## Example
/// ```luanext
/// // Before:
/// // a.luax: export { foo } from './b';
/// // b.luax: export { foo } from './c';
/// // c.luax: export function foo() { }
///
/// // After (in a.luax):
/// // Direct import from c, skipping b:
/// import { foo } from './c';
/// export { foo };
/// ```
///
/// ## Optimization Level
/// - O0/O1: Disabled
/// - O2: Disabled (conservative)
/// - O3: Enabled (aggressive whole-program optimization)
///
/// ## Safety Considerations
/// This optimization is sound because:
/// 1. Module tables are immutable once returned from require()
/// 2. Re-exports are static declarations (no runtime logic)
/// 3. We preserve the same symbol naming and visibility
///
/// However, it changes the order of module evaluation:
/// - Before: A requires B, B requires C
/// - After: A requires C, A requires B (if B has other exports)
///
/// If B has side effects in its module body, they may execute in a different order.
/// For this reason, this pass is only enabled at O3.
pub struct ReExportFlatteningPass {
    #[allow(dead_code)] // Will be used in full implementation
    module_graph: Arc<ModuleGraph>,
    interner: Arc<StringInterner>,
    current_module_path: Option<std::path::PathBuf>,
}

// TODO: Remove this allow once integrated into CLI (Phase 3)
#[allow(dead_code)]
impl ReExportFlatteningPass {
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

    /// Apply re-export flattening to a list of statements
    ///
    /// Transforms re-export chains into direct imports + local exports to reduce
    /// the number of module requires at runtime.
    ///
    /// Example transformation:
    /// ```
    /// // Before: export { foo } from './b';  (where b.luax re-exports from c.luax)
    /// // After:  import { foo } from './c';
    /// //         export { foo };
    /// ```
    ///
    /// Returns a new Vec with flattened re-exports.
    pub fn apply<'arena>(&self, statements: &[Statement<'arena>]) -> Vec<Statement<'arena>> {
        let Some(module_path) = &self.current_module_path else {
            return statements.to_vec();
        };

        let mut result = Vec::with_capacity(statements.len());

        for stmt in statements {
            match stmt {
                Statement::Export(export_decl) => {
                    match &export_decl.kind {
                        ExportKind::Named {
                            specifiers,
                            source: Some(source),
                            is_type_only: _,
                        } => {
                            // This is a re-export: export { x, y } from './other'
                            // Try to flatten the chain
                            let mut _flattened = false;

                            // Try to resolve the re-export chain for each specifier
                            for spec in specifiers.iter() {
                                let exported_name = spec.exported.as_ref().unwrap_or(&spec.local);
                                let exported_name_str = self.interner.resolve(exported_name.node);

                                // Try to resolve via module graph
                                if let Some((_original_module, _original_symbol)) = self
                                    .resolve_reexport_source(
                                        module_path,
                                        &exported_name_str,
                                        source,
                                    )
                                {
                                    // Successfully resolved to original source
                                    // We could flatten this, but it requires arena allocation
                                    // for new Import + Export statements
                                    // For now, mark as flattened candidate but keep original
                                    _flattened = true;
                                }
                            }

                            // Keep the original for now
                            // Full implementation requires creating new Import/Export nodes
                            // in the arena, which is complex
                            result.push(stmt.clone());
                        }
                        _ => {
                            // Not a re-export, keep unchanged
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

    /// Resolve a re-export chain to find the original source module and symbol
    ///
    /// Returns `Some((original_module_path, original_symbol_name))` if the chain
    /// can be resolved, or `None` if resolution fails.
    fn resolve_reexport_source(
        &self,
        _current_module: &std::path::Path,
        _symbol_name: &str,
        _source_module: &str,
    ) -> Option<(std::path::PathBuf, String)> {
        // This would use ModuleGraph::resolve_re_export_chain()
        // Implementation requires proper module path resolution
        // For now, return None (no flattening)
        None
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
        let pass = ReExportFlatteningPass::new(Arc::new(graph), interner);
        assert!(pass.current_module_path.is_none());
    }

    #[test]
    fn test_set_current_module() {
        let graph = ModuleGraph {
            modules: FxHashMap::default(),
            entry_points: Default::default(),
        };

        let interner = Arc::new(StringInterner::new());
        let mut pass = ReExportFlatteningPass::new(Arc::new(graph), interner);
        let module_path = PathBuf::from("test.luax");
        pass.set_current_module(&module_path);

        assert_eq!(pass.current_module_path, Some(module_path));
    }
}
