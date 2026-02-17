/// Unit tests for ModuleGraph data structures
///
/// These tests verify the basic data structures and functionality of the LTO module graph.
use luanext_core::optimizer::analysis::module_graph::{ModuleGraph, ModuleNode, ExportInfo, ImportInfo};
use rustc_hash::{FxHashMap, FxHashSet};
use std::path::PathBuf;

/// Helper to create a test module node
fn create_module_node(path: PathBuf) -> ModuleNode {
    ModuleNode {
        path,
        exports: FxHashMap::default(),
        imports: FxHashMap::default(),
        re_exports: Vec::new(),
        is_reachable: false,
    }
}

#[test]
fn test_module_graph_empty() {
    let graph = ModuleGraph {
        modules: FxHashMap::default(),
        entry_points: FxHashSet::default(),
    };

    assert_eq!(graph.modules.len(), 0);
    assert_eq!(graph.entry_points.len(), 0);
}

#[test]
fn test_module_graph_single_entry() {
    let path = PathBuf::from("main.luax");
    let mut graph = ModuleGraph {
        modules: FxHashMap::default(),
        entry_points: FxHashSet::default(),
    };

    graph.entry_points.insert(path.clone());

    let mut module = create_module_node(path.clone());
    module.is_reachable = true;

    graph.modules.insert(path.clone(), module);

    assert_eq!(graph.modules.len(), 1);
    assert_eq!(graph.entry_points.len(), 1);
    assert!(graph.entry_points.contains(&path));

    let module = graph.modules.get(&path).unwrap();
    assert!(module.is_reachable);
}

#[test]
fn test_export_info_structure() {
    let export = ExportInfo {
        name: "greet".to_string(),
        is_used: false,
        is_type_only: false,
        is_default: false,
    };

    assert_eq!(export.name, "greet");
    assert!(!export.is_used);
    assert!(!export.is_type_only);
    assert!(!export.is_default);
}

#[test]
fn test_import_info_structure() {
    let import = ImportInfo {
        name: "greet".to_string(),
        source_module: PathBuf::from("./utils.luax"),
        source_symbol: "greet".to_string(),
        is_referenced: false,
        is_type_only: false,
    };

    assert_eq!(import.name, "greet");
    assert_eq!(import.source_module, PathBuf::from("./utils.luax"));
    assert_eq!(import.source_symbol, "greet");
    assert!(!import.is_referenced);
    assert!(!import.is_type_only);
}

#[test]
fn test_module_node_with_exports() {
    let path = PathBuf::from("utils.luax");
    let mut module = create_module_node(path);

    module.exports.insert(
        "greet".to_string(),
        ExportInfo {
            name: "greet".to_string(),
            is_used: false,
            is_type_only: false,
            is_default: false,
        },
    );

    module.exports.insert(
        "farewell".to_string(),
        ExportInfo {
            name: "farewell".to_string(),
            is_used: false,
            is_type_only: false,
            is_default: false,
        },
    );

    assert_eq!(module.exports.len(), 2);
    assert!(module.exports.contains_key("greet"));
    assert!(module.exports.contains_key("farewell"));
}

#[test]
fn test_module_node_with_imports() {
    let path = PathBuf::from("main.luax");
    let mut module = create_module_node(path);

    module.imports.insert(
        "greet".to_string(),
        ImportInfo {
            name: "greet".to_string(),
            source_module: PathBuf::from("./utils.luax"),
            source_symbol: "greet".to_string(),
            is_referenced: true,
            is_type_only: false,
        },
    );

    module.imports.insert(
        "unused".to_string(),
        ImportInfo {
            name: "unused".to_string(),
            source_module: PathBuf::from("./utils.luax"),
            source_symbol: "unused".to_string(),
            is_referenced: false,
            is_type_only: false,
        },
    );

    assert_eq!(module.imports.len(), 2);
    assert!(module.imports.contains_key("greet"));
    assert!(module.imports.contains_key("unused"));

    let greet = module.imports.get("greet").unwrap();
    assert!(greet.is_referenced);

    let unused = module.imports.get("unused").unwrap();
    assert!(!unused.is_referenced);
}

#[test]
fn test_module_graph_reachability() {
    let main_path = PathBuf::from("main.luax");
    let used_path = PathBuf::from("used.luax");
    let unused_path = PathBuf::from("unused.luax");

    let mut graph = ModuleGraph {
        modules: FxHashMap::default(),
        entry_points: FxHashSet::default(),
    };

    graph.entry_points.insert(main_path.clone());

    let mut main_module = create_module_node(main_path.clone());
    main_module.is_reachable = true;

    let mut used_module = create_module_node(used_path.clone());
    used_module.is_reachable = true;

    let mut unused_module = create_module_node(unused_path.clone());
    unused_module.is_reachable = false;

    graph.modules.insert(main_path.clone(), main_module);
    graph.modules.insert(used_path.clone(), used_module);
    graph.modules.insert(unused_path.clone(), unused_module);

    assert_eq!(graph.modules.len(), 3);

    let reachable_count = graph
        .modules
        .values()
        .filter(|m| m.is_reachable)
        .count();

    assert_eq!(reachable_count, 2);
}

#[test]
fn test_export_usage_tracking() {
    let utils_path = PathBuf::from("utils.luax");
    let mut graph = ModuleGraph {
        modules: FxHashMap::default(),
        entry_points: FxHashSet::default(),
    };

    let mut module = create_module_node(utils_path.clone());

    // Add used export
    module.exports.insert(
        "used".to_string(),
        ExportInfo {
            name: "used".to_string(),
            is_used: true,
            is_type_only: false,
            is_default: false,
        },
    );

    // Add unused export
    module.exports.insert(
        "unused".to_string(),
        ExportInfo {
            name: "unused".to_string(),
            is_used: false,
            is_type_only: false,
            is_default: false,
        },
    );

    graph.modules.insert(utils_path.clone(), module);

    let module = graph.modules.get(&utils_path).unwrap();

    let used = module.exports.get("used").unwrap();
    assert!(used.is_used);

    let unused = module.exports.get("unused").unwrap();
    assert!(!unused.is_used);
}

#[test]
fn test_type_only_imports() {
    let path = PathBuf::from("main.luax");
    let mut module = create_module_node(path);

    module.imports.insert(
        "User".to_string(),
        ImportInfo {
            name: "User".to_string(),
            source_module: PathBuf::from("./types.luax"),
            source_symbol: "User".to_string(),
            is_referenced: false,
            is_type_only: true,
        },
    );

    let user_import = module.imports.get("User").unwrap();
    assert!(user_import.is_type_only);
    assert!(!user_import.is_referenced); // Type-only imports don't count as runtime references
}

#[test]
fn test_type_only_exports() {
    let path = PathBuf::from("types.luax");
    let mut module = create_module_node(path);

    module.exports.insert(
        "User".to_string(),
        ExportInfo {
            name: "User".to_string(),
            is_used: false,
            is_type_only: true,
            is_default: false,
        },
    );

    let user_export = module.exports.get("User").unwrap();
    assert!(user_export.is_type_only);
}

#[test]
fn test_default_exports() {
    let path = PathBuf::from("config.luax");
    let mut module = create_module_node(path);

    module.exports.insert(
        "default".to_string(),
        ExportInfo {
            name: "default".to_string(),
            is_used: true,
            is_type_only: false,
            is_default: true,
        },
    );

    let default_export = module.exports.get("default").unwrap();
    assert!(default_export.is_default);
    assert!(default_export.is_used);
}
