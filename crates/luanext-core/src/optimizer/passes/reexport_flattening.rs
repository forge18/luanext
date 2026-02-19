use crate::optimizer::analysis::module_graph::{
    compute_relative_require_path, resolve_relative_source, ModuleGraph,
};
use luanext_parser::ast::statement::{ExportKind, ExportSpecifier, Statement};
use luanext_parser::string_interner::StringInterner;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Re-export Flattening Pass
///
/// Optimizes re-export chains by flattening them to reduce runtime `require()` overhead.
/// When module A re-exports from B, and B re-exports from C, this pass can flatten
/// A to directly require from C instead of going through B.
///
/// ## Example
/// ```text
/// // Before:
/// // a.luax: export { foo } from './b';
/// // b.luax: export { foo } from './c';
/// // c.luax: export function foo() { }
///
/// // After (in a.luax's codegen):
/// // require("./c") instead of require("./b")
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
    module_graph: Arc<ModuleGraph>,
    interner: Arc<StringInterner>,
    current_module_path: Option<PathBuf>,
}

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

    /// Apply re-export flattening to a list of statements.
    ///
    /// For each re-export statement (`export { x } from './b'` or `export * from './b'`),
    /// resolves the re-export chain through the module graph. If the chain leads to a
    /// different module than the direct source, replaces the source path with the
    /// original module's path.
    ///
    /// Returns a new Vec with flattened re-exports.
    pub fn apply<'arena>(&self, statements: &[Statement<'arena>]) -> Vec<Statement<'arena>> {
        let Some(module_path) = &self.current_module_path else {
            return statements.to_vec();
        };

        let mut result = Vec::with_capacity(statements.len());

        for stmt in statements {
            match stmt {
                Statement::Export(export_decl) => match &export_decl.kind {
                    ExportKind::Named {
                        specifiers,
                        source: Some(source),
                        is_type_only,
                    } => {
                        if let Some(new_source) =
                            self.try_flatten_named(module_path, specifiers, source)
                        {
                            let mut new_export = export_decl.clone();
                            new_export.kind = ExportKind::Named {
                                specifiers,
                                source: Some(new_source),
                                is_type_only: *is_type_only,
                            };
                            result.push(Statement::Export(new_export));
                        } else {
                            result.push(stmt.clone());
                        }
                    }
                    ExportKind::All {
                        source,
                        is_type_only,
                    } => {
                        if let Some(new_source) = self.try_flatten_all(module_path, source) {
                            let mut new_export = export_decl.clone();
                            new_export.kind = ExportKind::All {
                                source: new_source,
                                is_type_only: *is_type_only,
                            };
                            result.push(Statement::Export(new_export));
                        } else {
                            result.push(stmt.clone());
                        }
                    }
                    _ => result.push(stmt.clone()),
                },
                _ => result.push(stmt.clone()),
            }
        }

        result
    }

    /// Try to flatten a named re-export (`export { x, y } from './b'`).
    ///
    /// Only flattens if ALL specifiers resolve to the SAME original module.
    /// Returns the new source path string, or `None` if flattening is not possible.
    fn try_flatten_named(
        &self,
        current_module: &Path,
        specifiers: &[ExportSpecifier],
        source: &str,
    ) -> Option<String> {
        let source_canonical = self.resolve_source_to_canonical(current_module, source)?;
        let mut common_original: Option<PathBuf> = None;

        for spec in specifiers {
            // Use the local name (the name in the source module), not the exported alias
            let local_name_str = self.interner.resolve(spec.local.node);

            let (original_module, _original_symbol) = self
                .module_graph
                .resolve_re_export_chain(&source_canonical, &local_name_str)?;

            // Only flatten if the original source is different from the direct source
            if original_module == source_canonical {
                return None;
            }

            match &common_original {
                None => common_original = Some(original_module),
                Some(existing) => {
                    if *existing != original_module {
                        // Different specifiers resolve to different modules — can't flatten
                        return None;
                    }
                }
            }
        }

        let original = common_original?;
        Some(compute_relative_require_path(current_module, &original))
    }

    /// Try to flatten an `export * from './b'` statement.
    ///
    /// For `export *`, we check if the source module itself is purely a re-exporter
    /// (has re-exports but no direct exports). If so, and all re-exports point to
    /// one module, we flatten. Otherwise, we check if the source module re-exports
    /// everything from a single deeper module.
    fn try_flatten_all(&self, current_module: &Path, source: &str) -> Option<String> {
        let source_canonical = self.resolve_source_to_canonical(current_module, source)?;
        let source_node = self.module_graph.modules.get(&source_canonical)?;

        // Only flatten if the source has no direct exports and exactly one re-export
        // that is also an `All` re-export — meaning it's a pure pass-through barrel file
        if !source_node.exports.is_empty() {
            return None;
        }

        if source_node.re_exports.len() != 1 {
            return None;
        }

        let re_export = &source_node.re_exports[0];
        if !matches!(
            re_export.specifiers,
            crate::optimizer::analysis::module_graph::ReExportKind::All
        ) {
            return None;
        }

        // The source is a pure barrel: `export * from './deeper'`
        // Check that the deeper module exists in the graph
        let deeper = &re_export.source_module;
        if !self.module_graph.modules.contains_key(deeper) {
            return None;
        }

        // Don't flatten to the same module we're already pointing at
        if *deeper == source_canonical {
            return None;
        }

        Some(compute_relative_require_path(current_module, deeper))
    }

    /// Resolve a relative source string to a canonical module path using the module graph.
    fn resolve_source_to_canonical(&self, current_module: &Path, source: &str) -> Option<PathBuf> {
        let current_dir = current_module.parent()?;
        let known_modules: Vec<PathBuf> = self.module_graph.modules.keys().cloned().collect();
        resolve_relative_source(current_dir, source, &known_modules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer::analysis::module_graph::{
        ExportInfo, ModuleNode, ReExportInfo, ReExportKind,
    };
    use luanext_parser::ast::statement::ExportSpecifier;
    use luanext_parser::{Ident, Span};
    use rustc_hash::{FxHashMap, FxHashSet};

    fn empty_node(path: PathBuf) -> ModuleNode {
        ModuleNode {
            path,
            exports: FxHashMap::default(),
            imports: FxHashMap::default(),
            re_exports: Vec::new(),
            is_reachable: true,
        }
    }

    fn export_info(name: &str) -> ExportInfo {
        ExportInfo {
            name: name.to_string(),
            is_type_only: false,
            is_default: false,
            is_used: true,
        }
    }

    #[test]
    fn test_pass_creation() {
        let graph = ModuleGraph {
            modules: FxHashMap::default(),
            entry_points: FxHashSet::default(),
        };

        let interner = Arc::new(StringInterner::new());
        let pass = ReExportFlatteningPass::new(Arc::new(graph), interner);
        assert!(pass.current_module_path.is_none());
    }

    #[test]
    fn test_set_current_module() {
        let graph = ModuleGraph {
            modules: FxHashMap::default(),
            entry_points: FxHashSet::default(),
        };

        let interner = Arc::new(StringInterner::new());
        let mut pass = ReExportFlatteningPass::new(Arc::new(graph), interner);
        let module_path = PathBuf::from("test.luax");
        pass.set_current_module(&module_path);

        assert_eq!(pass.current_module_path, Some(module_path));
    }

    #[test]
    fn test_no_flattening_without_chain() {
        // B directly exports `foo` — no chain to flatten
        let mut modules = FxHashMap::default();

        let a_path = PathBuf::from("/project/src/a.luax");
        let b_path = PathBuf::from("/project/src/b.luax");

        let a_node = empty_node(a_path.clone());

        let mut b_node = empty_node(b_path.clone());
        b_node.exports.insert("foo".to_string(), export_info("foo"));

        modules.insert(a_path.clone(), a_node);
        modules.insert(b_path, b_node);

        let graph = ModuleGraph {
            modules,
            entry_points: FxHashSet::default(),
        };

        let interner = Arc::new(StringInterner::new());
        let mut pass = ReExportFlatteningPass::new(Arc::new(graph), interner);
        pass.set_current_module(&a_path);

        // Empty statements — nothing to flatten
        let result = pass.apply(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_non_export_preserved() {
        let graph = ModuleGraph {
            modules: FxHashMap::default(),
            entry_points: FxHashSet::default(),
        };

        let interner = Arc::new(StringInterner::new());
        let mut pass = ReExportFlatteningPass::new(Arc::new(graph), interner);
        pass.set_current_module(&PathBuf::from("/project/src/a.luax"));

        // Pass through is preserved (empty statements return empty)
        let result = pass.apply(&[]);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_resolve_source_to_canonical() {
        let mut modules = FxHashMap::default();
        let b_path = PathBuf::from("/project/src/b.luax");
        modules.insert(b_path.clone(), empty_node(b_path.clone()));

        let graph = ModuleGraph {
            modules,
            entry_points: FxHashSet::default(),
        };

        let interner = Arc::new(StringInterner::new());
        let pass = ReExportFlatteningPass::new(Arc::new(graph), interner);

        let a_path = PathBuf::from("/project/src/a.luax");
        let resolved = pass.resolve_source_to_canonical(&a_path, "./b");
        assert_eq!(resolved, Some(b_path));
    }

    #[test]
    fn test_resolve_source_to_canonical_parent_dir() {
        let mut modules = FxHashMap::default();
        let c_path = PathBuf::from("/project/lib/c.luax");
        modules.insert(c_path.clone(), empty_node(c_path.clone()));

        let graph = ModuleGraph {
            modules,
            entry_points: FxHashSet::default(),
        };

        let interner = Arc::new(StringInterner::new());
        let pass = ReExportFlatteningPass::new(Arc::new(graph), interner);

        let a_path = PathBuf::from("/project/src/a.luax");
        let resolved = pass.resolve_source_to_canonical(&a_path, "../lib/c");
        assert_eq!(resolved, Some(c_path));
    }

    #[test]
    fn test_try_flatten_named_simple_chain() {
        // A re-exports from B, B re-exports from C, C defines foo
        // A → B → C should flatten to A → C
        let mut modules = FxHashMap::default();

        let a_path = PathBuf::from("/project/src/a.luax");
        let b_path = PathBuf::from("/project/src/b.luax");
        let c_path = PathBuf::from("/project/src/c.luax");

        modules.insert(a_path.clone(), empty_node(a_path.clone()));

        let mut b_node = empty_node(b_path.clone());
        b_node.re_exports.push(ReExportInfo {
            source_module: c_path.clone(),
            specifiers: ReExportKind::Named(vec![("foo".to_string(), "foo".to_string())]),
        });
        modules.insert(b_path.clone(), b_node);

        let mut c_node = empty_node(c_path.clone());
        c_node.exports.insert("foo".to_string(), export_info("foo"));
        modules.insert(c_path, c_node);

        let graph = ModuleGraph {
            modules,
            entry_points: FxHashSet::default(),
        };

        let interner = Arc::new(StringInterner::new());
        let foo_id = interner.get_or_intern("foo");

        let pass = ReExportFlatteningPass::new(Arc::new(graph), interner);

        let spec = ExportSpecifier {
            local: Ident {
                node: foo_id,
                span: Span::default(),
            },
            exported: None,
            span: Span::default(),
        };

        let result = pass.try_flatten_named(&a_path, &[spec], "./b");
        assert!(result.is_some());
        let new_source = result.unwrap();
        assert_eq!(new_source, "./c");
    }

    #[test]
    fn test_try_flatten_named_deep_chain() {
        // A → B → C → D, should flatten to A → D
        let mut modules = FxHashMap::default();

        let a_path = PathBuf::from("/project/src/a.luax");
        let b_path = PathBuf::from("/project/src/b.luax");
        let c_path = PathBuf::from("/project/src/c.luax");
        let d_path = PathBuf::from("/project/src/d.luax");

        modules.insert(a_path.clone(), empty_node(a_path.clone()));

        let mut b_node = empty_node(b_path.clone());
        b_node.re_exports.push(ReExportInfo {
            source_module: c_path.clone(),
            specifiers: ReExportKind::Named(vec![("foo".to_string(), "foo".to_string())]),
        });
        modules.insert(b_path, b_node);

        let mut c_node = empty_node(c_path.clone());
        c_node.re_exports.push(ReExportInfo {
            source_module: d_path.clone(),
            specifiers: ReExportKind::Named(vec![("foo".to_string(), "foo".to_string())]),
        });
        modules.insert(c_path, c_node);

        let mut d_node = empty_node(d_path.clone());
        d_node.exports.insert("foo".to_string(), export_info("foo"));
        modules.insert(d_path, d_node);

        let graph = ModuleGraph {
            modules,
            entry_points: FxHashSet::default(),
        };

        let interner = Arc::new(StringInterner::new());
        let foo_id = interner.get_or_intern("foo");

        let pass = ReExportFlatteningPass::new(Arc::new(graph), interner);

        let spec = ExportSpecifier {
            local: Ident {
                node: foo_id,
                span: Span::default(),
            },
            exported: None,
            span: Span::default(),
        };

        let result = pass.try_flatten_named(&a_path, &[spec], "./b");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "./d");
    }

    #[test]
    fn test_no_flatten_divergent_specs() {
        // export { foo, bar } from './b' where foo→C and bar→D
        // Cannot flatten since specs go to different modules
        let mut modules = FxHashMap::default();

        let a_path = PathBuf::from("/project/src/a.luax");
        let b_path = PathBuf::from("/project/src/b.luax");
        let c_path = PathBuf::from("/project/src/c.luax");
        let d_path = PathBuf::from("/project/src/d.luax");

        modules.insert(a_path.clone(), empty_node(a_path.clone()));

        let mut b_node = empty_node(b_path.clone());
        b_node.re_exports.push(ReExportInfo {
            source_module: c_path.clone(),
            specifiers: ReExportKind::Named(vec![("foo".to_string(), "foo".to_string())]),
        });
        b_node.re_exports.push(ReExportInfo {
            source_module: d_path.clone(),
            specifiers: ReExportKind::Named(vec![("bar".to_string(), "bar".to_string())]),
        });
        modules.insert(b_path, b_node);

        let mut c_node = empty_node(c_path.clone());
        c_node.exports.insert("foo".to_string(), export_info("foo"));
        modules.insert(c_path, c_node);

        let mut d_node = empty_node(d_path.clone());
        d_node.exports.insert("bar".to_string(), export_info("bar"));
        modules.insert(d_path, d_node);

        let graph = ModuleGraph {
            modules,
            entry_points: FxHashSet::default(),
        };

        let interner = Arc::new(StringInterner::new());
        let foo_id = interner.get_or_intern("foo");
        let bar_id = interner.get_or_intern("bar");

        let pass = ReExportFlatteningPass::new(Arc::new(graph), interner);

        let specs = vec![
            ExportSpecifier {
                local: Ident {
                    node: foo_id,
                    span: Span::default(),
                },
                exported: None,
                span: Span::default(),
            },
            ExportSpecifier {
                local: Ident {
                    node: bar_id,
                    span: Span::default(),
                },
                exported: None,
                span: Span::default(),
            },
        ];

        let result = pass.try_flatten_named(&a_path, &specs, "./b");
        assert!(result.is_none());
    }

    #[test]
    fn test_cycle_detection() {
        // A re-exports from B, B re-exports from A — cycle
        let mut modules = FxHashMap::default();

        let a_path = PathBuf::from("/project/src/a.luax");
        let b_path = PathBuf::from("/project/src/b.luax");

        let mut a_node = empty_node(a_path.clone());
        a_node.re_exports.push(ReExportInfo {
            source_module: b_path.clone(),
            specifiers: ReExportKind::Named(vec![("foo".to_string(), "foo".to_string())]),
        });
        modules.insert(a_path.clone(), a_node);

        let mut b_node = empty_node(b_path.clone());
        b_node.re_exports.push(ReExportInfo {
            source_module: a_path.clone(),
            specifiers: ReExportKind::Named(vec![("foo".to_string(), "foo".to_string())]),
        });
        modules.insert(b_path, b_node);

        let graph = ModuleGraph {
            modules,
            entry_points: FxHashSet::default(),
        };

        let interner = Arc::new(StringInterner::new());
        let foo_id = interner.get_or_intern("foo");

        let pass = ReExportFlatteningPass::new(Arc::new(graph), interner);

        let spec = ExportSpecifier {
            local: Ident {
                node: foo_id,
                span: Span::default(),
            },
            exported: None,
            span: Span::default(),
        };

        // resolve_re_export_chain returns None for cycles
        let result = pass.try_flatten_named(&a_path, &[spec], "./b");
        assert!(result.is_none());
    }

    #[test]
    fn test_try_flatten_all_barrel() {
        // B is a pure barrel: `export * from './c'`, C defines exports
        let mut modules = FxHashMap::default();

        let a_path = PathBuf::from("/project/src/a.luax");
        let b_path = PathBuf::from("/project/src/b.luax");
        let c_path = PathBuf::from("/project/src/c.luax");

        modules.insert(a_path.clone(), empty_node(a_path.clone()));

        let mut b_node = empty_node(b_path.clone());
        // B has no direct exports, only a re-export * from C
        b_node.re_exports.push(ReExportInfo {
            source_module: c_path.clone(),
            specifiers: ReExportKind::All,
        });
        modules.insert(b_path, b_node);

        let mut c_node = empty_node(c_path.clone());
        c_node.exports.insert("foo".to_string(), export_info("foo"));
        modules.insert(c_path, c_node);

        let graph = ModuleGraph {
            modules,
            entry_points: FxHashSet::default(),
        };

        let interner = Arc::new(StringInterner::new());
        let mut pass = ReExportFlatteningPass::new(Arc::new(graph), interner);
        pass.set_current_module(&a_path);

        let result = pass.try_flatten_all(&a_path, "./b");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "./c");
    }

    #[test]
    fn test_no_flatten_all_when_source_has_exports() {
        // B has its own exports AND re-exports — not a pure barrel
        let mut modules = FxHashMap::default();

        let a_path = PathBuf::from("/project/src/a.luax");
        let b_path = PathBuf::from("/project/src/b.luax");
        let c_path = PathBuf::from("/project/src/c.luax");

        modules.insert(a_path.clone(), empty_node(a_path.clone()));

        let mut b_node = empty_node(b_path.clone());
        b_node.exports.insert("bar".to_string(), export_info("bar"));
        b_node.re_exports.push(ReExportInfo {
            source_module: c_path.clone(),
            specifiers: ReExportKind::All,
        });
        modules.insert(b_path, b_node);

        let mut c_node = empty_node(c_path.clone());
        c_node.exports.insert("foo".to_string(), export_info("foo"));
        modules.insert(c_path, c_node);

        let graph = ModuleGraph {
            modules,
            entry_points: FxHashSet::default(),
        };

        let interner = Arc::new(StringInterner::new());
        let pass = ReExportFlatteningPass::new(Arc::new(graph), interner);

        let result = pass.try_flatten_all(&a_path, "./b");
        assert!(result.is_none());
    }

    #[test]
    fn test_no_flatten_when_source_is_direct() {
        // B directly exports foo — the chain resolves to B itself
        // Since original == source, there's nothing to flatten
        let mut modules = FxHashMap::default();

        let a_path = PathBuf::from("/project/src/a.luax");
        let b_path = PathBuf::from("/project/src/b.luax");

        modules.insert(a_path.clone(), empty_node(a_path.clone()));

        let mut b_node = empty_node(b_path.clone());
        b_node.exports.insert("foo".to_string(), export_info("foo"));
        modules.insert(b_path, b_node);

        let graph = ModuleGraph {
            modules,
            entry_points: FxHashSet::default(),
        };

        let interner = Arc::new(StringInterner::new());
        let foo_id = interner.get_or_intern("foo");

        let pass = ReExportFlatteningPass::new(Arc::new(graph), interner);

        let spec = ExportSpecifier {
            local: Ident {
                node: foo_id,
                span: Span::default(),
            },
            exported: None,
            span: Span::default(),
        };

        let result = pass.try_flatten_named(&a_path, &[spec], "./b");
        assert!(result.is_none());
    }
}
