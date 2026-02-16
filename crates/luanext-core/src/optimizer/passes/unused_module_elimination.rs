use crate::optimizer::analysis::module_graph::ModuleGraph;
use std::path::Path;
use std::sync::Arc;

/// Unused Module Elimination
///
/// This is not a traditional optimization pass - it's a filter applied at the CLI level
/// to skip compilation of modules that are not reachable from any entry point.
///
/// Unlike other passes that transform AST, this pass provides helper methods to check
/// if a module should be compiled.
///
/// ## Example
/// ```text
/// Entry: main.luax
/// - imports: ./used.luax
/// - doesn't import: ./unused.luax
///
/// Result: ./unused.luax is not compiled, saving compilation time
/// ```
///
/// ## Optimization Level
/// - O0/O1: Disabled (compile all modules)
/// - O2: Enabled (skip unreachable modules)
/// - O3: Enabled (skip unreachable modules)
///
/// ## Safety
/// This optimization is conservative:
/// - Entry points are always compiled
/// - Modules with `is_reachable = false` are skipped
/// - If module is not in graph, default to compiling it (graceful degradation)
pub struct UnusedModuleEliminationPass {
    module_graph: Arc<ModuleGraph>,
}

// TODO: Remove this allow once integrated into CLI (Phase 3)
#[allow(dead_code)]
impl UnusedModuleEliminationPass {
    pub fn new(module_graph: Arc<ModuleGraph>) -> Self {
        Self { module_graph }
    }

    /// Check if a module should be compiled based on reachability
    pub fn should_compile(&self, module_path: &Path) -> bool {
        // If module is an entry point, always compile
        if self.module_graph.entry_points.contains(module_path) {
            return true;
        }

        // Check reachability in graph
        match self.module_graph.modules.get(module_path) {
            Some(node) => node.is_reachable,
            None => {
                // Module not in graph - conservatively compile it
                // This handles cases where graph may be stale or incomplete
                true
            }
        }
    }

    /// Get all modules that should be compiled (reachable modules)
    pub fn get_modules_to_compile(&self) -> Vec<std::path::PathBuf> {
        self.module_graph
            .modules
            .values()
            .filter(|node| node.is_reachable)
            .map(|node| node.path.clone())
            .collect()
    }

    /// Get all modules that can be skipped (unreachable modules)
    pub fn get_modules_to_skip(&self) -> Vec<std::path::PathBuf> {
        self.module_graph
            .modules
            .values()
            .filter(|node| !node.is_reachable)
            .map(|node| node.path.clone())
            .collect()
    }

    /// Get statistics about module elimination
    pub fn get_stats(&self) -> UnusedModuleStats {
        let total = self.module_graph.modules.len();
        let reachable = self
            .module_graph
            .modules
            .values()
            .filter(|n| n.is_reachable)
            .count();
        let eliminated = total - reachable;

        UnusedModuleStats {
            total_modules: total,
            reachable_modules: reachable,
            eliminated_modules: eliminated,
            elimination_ratio: if total > 0 {
                (eliminated as f64 / total as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

/// Statistics about unused module elimination
#[derive(Debug, Clone)]
pub struct UnusedModuleStats {
    pub total_modules: usize,
    pub reachable_modules: usize,
    pub eliminated_modules: usize,
    pub elimination_ratio: f64, // Percentage
}

impl std::fmt::Display for UnusedModuleStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Modules: {} total, {} reachable, {} eliminated ({:.1}%)",
            self.total_modules,
            self.reachable_modules,
            self.eliminated_modules,
            self.elimination_ratio
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer::analysis::module_graph::{ModuleGraph, ModuleNode};
    use rustc_hash::{FxHashMap, FxHashSet};
    use std::path::PathBuf;

    #[test]
    fn test_entry_points_always_compiled() {
        let mut graph = ModuleGraph {
            modules: FxHashMap::default(),
            entry_points: FxHashSet::default(),
        };

        let entry_path = PathBuf::from("main.luax");
        graph.entry_points.insert(entry_path.clone());

        let node = ModuleNode {
            path: entry_path.clone(),
            exports: FxHashMap::default(),
            imports: FxHashMap::default(),
            re_exports: Vec::new(),
            is_reachable: false, // Even if marked unreachable
        };
        graph.modules.insert(entry_path.clone(), node);

        let pass = UnusedModuleEliminationPass::new(Arc::new(graph));

        // Entry points always compiled, even if marked unreachable
        assert!(pass.should_compile(&entry_path));
    }

    #[test]
    fn test_reachable_modules_compiled() {
        let mut graph = ModuleGraph {
            modules: FxHashMap::default(),
            entry_points: FxHashSet::default(),
        };

        let reachable_path = PathBuf::from("reachable.luax");
        let node = ModuleNode {
            path: reachable_path.clone(),
            exports: FxHashMap::default(),
            imports: FxHashMap::default(),
            re_exports: Vec::new(),
            is_reachable: true,
        };
        graph.modules.insert(reachable_path.clone(), node);

        let pass = UnusedModuleEliminationPass::new(Arc::new(graph));

        assert!(pass.should_compile(&reachable_path));
    }

    #[test]
    fn test_unreachable_modules_skipped() {
        let mut graph = ModuleGraph {
            modules: FxHashMap::default(),
            entry_points: FxHashSet::default(),
        };

        let unreachable_path = PathBuf::from("unreachable.luax");
        let node = ModuleNode {
            path: unreachable_path.clone(),
            exports: FxHashMap::default(),
            imports: FxHashMap::default(),
            re_exports: Vec::new(),
            is_reachable: false,
        };
        graph.modules.insert(unreachable_path.clone(), node);

        let pass = UnusedModuleEliminationPass::new(Arc::new(graph));

        assert!(!pass.should_compile(&unreachable_path));
    }

    #[test]
    fn test_unknown_modules_compiled() {
        let graph = ModuleGraph {
            modules: FxHashMap::default(),
            entry_points: FxHashSet::default(),
        };

        let pass = UnusedModuleEliminationPass::new(Arc::new(graph));

        // Module not in graph - conservatively compile
        assert!(pass.should_compile(&PathBuf::from("unknown.luax")));
    }

    #[test]
    fn test_get_stats() {
        let mut graph = ModuleGraph {
            modules: FxHashMap::default(),
            entry_points: FxHashSet::default(),
        };

        // Add 3 reachable modules
        for i in 0..3 {
            let path = PathBuf::from(format!("reachable{}.luax", i));
            let node = ModuleNode {
                path: path.clone(),
                exports: FxHashMap::default(),
                imports: FxHashMap::default(),
                re_exports: Vec::new(),
                is_reachable: true,
            };
            graph.modules.insert(path, node);
        }

        // Add 2 unreachable modules
        for i in 0..2 {
            let path = PathBuf::from(format!("unreachable{}.luax", i));
            let node = ModuleNode {
                path: path.clone(),
                exports: FxHashMap::default(),
                imports: FxHashMap::default(),
                re_exports: Vec::new(),
                is_reachable: false,
            };
            graph.modules.insert(path, node);
        }

        let pass = UnusedModuleEliminationPass::new(Arc::new(graph));
        let stats = pass.get_stats();

        assert_eq!(stats.total_modules, 5);
        assert_eq!(stats.reachable_modules, 3);
        assert_eq!(stats.eliminated_modules, 2);
        assert_eq!(stats.elimination_ratio, 40.0);
    }

    #[test]
    fn test_get_modules_to_compile() {
        let mut graph = ModuleGraph {
            modules: FxHashMap::default(),
            entry_points: FxHashSet::default(),
        };

        let reachable1 = PathBuf::from("reachable1.luax");
        let reachable2 = PathBuf::from("reachable2.luax");
        let unreachable = PathBuf::from("unreachable.luax");

        for path in &[&reachable1, &reachable2] {
            let node = ModuleNode {
                path: (*path).clone(),
                exports: FxHashMap::default(),
                imports: FxHashMap::default(),
                re_exports: Vec::new(),
                is_reachable: true,
            };
            graph.modules.insert((*path).clone(), node);
        }

        let node = ModuleNode {
            path: unreachable.clone(),
            exports: FxHashMap::default(),
            imports: FxHashMap::default(),
            re_exports: Vec::new(),
            is_reachable: false,
        };
        graph.modules.insert(unreachable.clone(), node);

        let pass = UnusedModuleEliminationPass::new(Arc::new(graph));
        let to_compile = pass.get_modules_to_compile();

        assert_eq!(to_compile.len(), 2);
        assert!(to_compile.contains(&reachable1));
        assert!(to_compile.contains(&reachable2));
        assert!(!to_compile.contains(&unreachable));
    }
}
