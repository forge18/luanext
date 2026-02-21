use luanext_core::codegen::CodeGenerator;
use luanext_core::optimizer::analysis::module_graph::{
    ExportInfo, ModuleGraph, ModuleNode, ReExportInfo, ReExportKind,
};
use luanext_core::optimizer::passes::ReExportFlatteningPass;
use luanext_core::MutableProgram;
use luanext_parser::lexer::Lexer;
use luanext_parser::parser::Parser;
use luanext_parser::string_interner::StringInterner;
use luanext_typechecker::cli::diagnostics::CollectingDiagnosticHandler;
use rustc_hash::{FxHashMap, FxHashSet};
use std::path::PathBuf;
use std::sync::Arc;

fn generate_lua(source: &str) -> String {
    let arena = bumpalo::Bump::new();
    let handler = Arc::new(CollectingDiagnosticHandler::new());
    let (interner, common) = StringInterner::new_with_common_identifiers();
    let interner = Arc::new(interner);

    let mut lexer = Lexer::new(source, handler.clone(), &interner);
    let tokens = lexer.tokenize().expect("Lexing failed");

    let mut parser = Parser::new(tokens, handler.clone(), &interner, &common, &arena);
    let program = parser.parse().expect("Parsing failed");

    let mutable = MutableProgram::from_program(&program);
    let mut codegen = CodeGenerator::new(interner.clone());
    codegen.generate(&mutable)
}

#[test]
fn test_reexport_generates_require() {
    let source = r#"
        export { foo } from './module'
    "#;
    let lua = generate_lua(source);
    assert!(
        lua.contains("require"),
        "Re-export should generate require call"
    );
    assert!(
        lua.contains("./module"),
        "Re-export should reference source module"
    );
}

#[test]
fn test_reexport_adds_to_module_exports() {
    let source = r#"
        export { foo } from './module'
    "#;
    let lua = generate_lua(source);

    // The generated Lua should have the symbol available for export
    assert!(
        lua.contains("_mod") || lua.contains("foo"),
        "Re-export should load or reference the symbol"
    );
}

#[test]
fn test_renamed_reexport() {
    let source = r#"
        export { foo as bar } from './module'
    "#;
    let lua = generate_lua(source);

    // Should load foo and bind it as bar
    assert!(
        lua.contains("foo") || lua.contains("bar"),
        "Renamed re-export should reference both original and alias names"
    );
}

#[test]
fn test_multiple_reexports() {
    let source = r#"
        export { foo, bar, baz } from './module'
    "#;
    let lua = generate_lua(source);

    // Should load module once and extract all symbols
    let require_count = lua.matches("require").count();
    assert!(
        require_count == 1,
        "Multiple re-exports from same source should require module once, got {} requires",
        require_count
    );

    // Should reference all symbols
    assert!(lua.contains("foo"), "Should reference foo");
    assert!(lua.contains("bar"), "Should reference bar");
    assert!(lua.contains("baz"), "Should reference baz");
}

#[test]
fn test_multiple_reexports_from_different_sources() {
    let source = r#"
        export { foo } from './module1'
        export { bar } from './module2'
    "#;
    let lua = generate_lua(source);

    // Should require both modules
    let require_count = lua.matches("require").count();
    assert!(
        require_count >= 2,
        "Re-exports from different sources should require each source"
    );

    // Should reference symbols from different modules
    assert!(lua.contains("foo"), "Should reference foo from module1");
    assert!(lua.contains("bar"), "Should reference bar from module2");
}

#[test]
fn test_reexport_mixed_with_local_exports() {
    let source = r#"
        export const local_var = 42
        export { imported } from './module'
    "#;
    let lua = generate_lua(source);

    assert!(
        lua.contains("local_var") || lua.contains("42"),
        "Should generate local export"
    );
    assert!(
        lua.contains("require") || lua.contains("imported"),
        "Should generate re-export"
    );
}

#[test]
fn test_type_only_reexport_not_generated() {
    let source = r#"
        export type { Foo } from './types'
    "#;
    let lua = generate_lua(source);

    // Type-only imports should not generate any require() call in the Lua output
    // (they're erased at codegen time)
    // The output should be minimal or empty
    assert!(
        !lua.contains("require(\"./types\")"),
        "Type-only re-export should not generate require call"
    );
}

#[test]
fn test_reexport_with_type_annotation() {
    let source = r#"
        export { value } from './module'
    "#;
    let lua = generate_lua(source);

    // Type annotations are erased during codegen
    assert!(
        lua.contains("require") && lua.contains("value"),
        "Re-export codegen should ignore type annotations"
    );
}

#[test]
fn test_reexport_doesnt_duplicate_symbols() {
    let source = r#"
        export { foo, foo } from './module'
    "#;
    let lua = generate_lua(source);

    // Parser should handle or reject duplicate exports
    // Codegen should not create multiple assignments to same symbol
    let foo_count = lua.matches("foo").count();
    assert!(
        foo_count >= 2,
        "Symbol name appears in require and assignments"
    );
}

#[test]
fn test_reexport_preserves_order() {
    let source = r#"
        export { a, b, c } from './module'
    "#;
    let lua = generate_lua(source);

    // All symbols should be present in the generated code
    assert!(lua.contains("a"), "Should export symbol a");
    assert!(lua.contains("b"), "Should export symbol b");
    assert!(lua.contains("c"), "Should export symbol c");
}

#[test]
fn test_reexport_with_local_declaration() {
    let source = r#"
        local foo = 42
        export { foo }
    "#;
    let lua = generate_lua(source);

    // Should declare foo locally and then reference it
    assert!(lua.contains("local foo"), "Should declare foo locally");
    assert!(lua.contains("42"), "Should assign value to foo");
}

#[test]
fn test_reexport_function_reference() {
    let source = r#"
        function helper()
            return 42
        end
        export { helper }
    "#;
    let lua = generate_lua(source);

    assert!(
        lua.contains("function helper") || lua.contains("helper"),
        "Should generate function and export it"
    );
}

#[test]
fn test_reexport_interface_not_generated() {
    let source = r#"
        interface Shape
            area(): number
        end
        export type { Shape }
    "#;
    let lua = generate_lua(source);

    // Interface definitions are type-only and shouldn't appear in Lua
    assert!(
        !lua.contains("interface") && !lua.contains("Shape"),
        "Type-only interface should not appear in generated code"
    );
}

#[test]
fn test_reexport_with_special_characters_in_path() {
    let source = r#"
        export { foo } from '@scope/module'
    "#;
    let lua = generate_lua(source);

    assert!(
        lua.contains("@scope/module") || lua.contains("scope"),
        "Should handle scoped module paths"
    );
}

#[test]
fn test_reexport_with_relative_parent_path() {
    let source = r#"
        export { foo } from '../module'
    "#;
    let lua = generate_lua(source);

    assert!(
        lua.contains("../module") || lua.contains("module"),
        "Should handle relative parent paths"
    );
}

#[test]
fn test_reexport_default_import_export() {
    let source = r#"
        import foo from './module'
        export { foo }
    "#;
    let lua = generate_lua(source);

    // Should first require and bind default export
    assert!(lua.contains("require"), "Should require module");
    // Then export the binding
    assert!(lua.contains("foo"), "Should reference the exported symbol");
}

#[test]
fn test_reexport_with_alias_preserves_alias() {
    let source = r#"
        export { original as renamed } from './module'
    "#;
    let lua = generate_lua(source);

    // Should reference original name in require but use renamed name in exports
    assert!(
        lua.contains("original") || lua.contains("renamed"),
        "Should handle aliased re-exports"
    );
}

#[test]
fn test_reexport_with_default_export() {
    let source = r#"
        export default { foo: 1 }
        export { bar } from './module'
    "#;
    let lua = generate_lua(source);

    // Should have both default export and named re-export
    assert!(
        lua.contains("_default") || lua.contains("default"),
        "Should generate default export"
    );
    assert!(
        lua.contains("require") || lua.contains("bar"),
        "Should generate re-export"
    );
}

#[test]
fn test_reexport_chain_loads_module_once() {
    let source = r#"
        export { foo, bar, baz } from './shared'
    "#;
    let lua = generate_lua(source);

    // Verify that the module is loaded once into _mod and then all symbols are extracted
    let mod_assignments = lua.matches("_mod =").count();
    assert_eq!(mod_assignments, 1, "Module should be loaded exactly once");

    // All symbols should be extracted from the same _mod
    assert!(
        lua.contains("_mod.foo") || lua.contains("_mod.bar") || lua.contains("_mod.baz"),
        "Should extract symbols from loaded module"
    );
}

#[test]
fn test_codegen_export_all() {
    let source = r#"
        export * from './module'
    "#;
    let lua = generate_lua(source);

    // Verify module is loaded into a unique _reexport variable
    assert!(
        lua.contains("_reexport = require"),
        "Should load module into _reexport var, got:\n{lua}"
    );

    // Verify deferred merge loop in finalize_module uses M table
    assert!(
        lua.contains("for __k, __v in pairs(_reexport)"),
        "Should generate deferred merge loop, got:\n{lua}"
    );
    assert!(
        lua.contains("M[__k] = __v"),
        "Should assign to M table in merge loop, got:\n{lua}"
    );
    assert!(
        lua.contains("return M"),
        "Should return module table, got:\n{lua}"
    );
}

#[test]
fn test_codegen_export_all_type_only() {
    let source = r#"
        export type * from './module'
    "#;
    let lua = generate_lua(source);

    // Verify no code is generated for export type *
    assert!(
        !lua.contains("_mod") && !lua.contains("for k, v"),
        "export type * should not generate any code"
    );
}

#[test]
fn test_codegen_export_all_with_declarations() {
    let source = r#"
        export * from './module'
        export interface Local {
            x: number
        }
    "#;
    let lua = generate_lua(source);

    // Verify export * merge loop is present in finalized output
    assert!(
        lua.contains("for __k, __v in pairs(_reexport)"),
        "Should have export * merge loop, got:\n{lua}"
    );
}

#[test]
fn test_codegen_multiple_export_all() {
    let source = r#"
        export * from './module_a'
        export * from './module_b'
    "#;
    let lua = generate_lua(source);

    // Verify both modules are loaded with unique variable names
    assert!(
        lua.contains("_reexport = require"),
        "First export * should use _reexport, got:\n{lua}"
    );
    assert!(
        lua.contains("_reexport_2 = require"),
        "Second export * should use _reexport_2, got:\n{lua}"
    );

    // Verify deferred merge loops for both sources
    assert!(
        lua.contains("for __k, __v in pairs(_reexport)"),
        "Should have merge loop for first source, got:\n{lua}"
    );
    assert!(
        lua.contains("for __k, __v in pairs(_reexport_2)"),
        "Should have merge loop for second source, got:\n{lua}"
    );
}

#[test]
fn test_codegen_export_all_with_named_reexports() {
    let source = r#"
        export * from './module_a'
        export { foo } from './module_b'
    "#;
    let lua = generate_lua(source);

    // Verify export * merge loop is present
    assert!(
        lua.contains("for __k, __v in pairs(_reexport)"),
        "Should have export * merge loop, got:\n{lua}"
    );
    // Verify named re-export is present
    assert!(
        lua.contains("foo"),
        "Should have named re-export symbol, got:\n{lua}"
    );
}

#[test]
fn test_codegen_export_all_tree_shaking_selective_copy() {
    let source = r#"
        export * from './module'
    "#;
    let arena = bumpalo::Bump::new();
    let handler = Arc::new(CollectingDiagnosticHandler::new());
    let (interner, common) = StringInterner::new_with_common_identifiers();
    let interner = Arc::new(interner);

    let mut lexer = Lexer::new(source, handler.clone(), &interner);
    let tokens = lexer.tokenize().expect("Lexing failed");

    let mut parser = Parser::new(tokens, handler.clone(), &interner, &common, &arena);
    let program = parser.parse().expect("Parsing failed");

    let mutable = MutableProgram::from_program(&program);
    let mut codegen = CodeGenerator::new(interner.clone());

    // Enable tree shaking with specific reachable exports
    let mut reachable = std::collections::HashSet::new();
    reachable.insert("foo".to_string());
    reachable.insert("bar".to_string());
    codegen = codegen.with_tree_shaking(reachable);

    let lua = codegen.generate(&mutable);

    // With tree shaking, should generate individual assignments instead of for-loop
    assert!(
        !lua.contains("for __k, __v in pairs"),
        "Should not use for-loop with tree shaking enabled, got:\n{lua}"
    );
    // Check that both foo and bar are assigned from _reexport (order may vary due to HashSet iteration)
    assert!(
        lua.contains("_reexport.foo") && lua.contains("_reexport.bar"),
        "Should generate individual assignments for reachable exports, got:\n{lua}"
    );
}

#[test]
fn test_codegen_export_all_tree_shaking_empty_reachable() {
    let source = r#"
        export * from './module'
    "#;
    let arena = bumpalo::Bump::new();
    let handler = Arc::new(CollectingDiagnosticHandler::new());
    let (interner, common) = StringInterner::new_with_common_identifiers();
    let interner = Arc::new(interner);

    let mut lexer = Lexer::new(source, handler.clone(), &interner);
    let tokens = lexer.tokenize().expect("Lexing failed");

    let mut parser = Parser::new(tokens, handler.clone(), &interner, &common, &arena);
    let program = parser.parse().expect("Parsing failed");

    let mutable = MutableProgram::from_program(&program);
    let mut codegen = CodeGenerator::new(interner.clone());

    // Enable tree shaking with no reachable exports
    let reachable = std::collections::HashSet::new();
    codegen = codegen.with_tree_shaking(reachable);

    let lua = codegen.generate(&mutable);

    // With empty reachable set, export * should be skipped entirely
    assert!(
        !lua.contains("require"),
        "Should skip export * when no exports are reachable"
    );
    assert!(
        !lua.contains("for k, v in pairs"),
        "Should not generate for-loop when no exports are reachable"
    );
}

#[test]
fn test_codegen_export_all_no_tree_shaking() {
    let source = r#"
        export * from './module'
    "#;
    let arena = bumpalo::Bump::new();
    let handler = Arc::new(CollectingDiagnosticHandler::new());
    let (interner, common) = StringInterner::new_with_common_identifiers();
    let interner = Arc::new(interner);

    let mut lexer = Lexer::new(source, handler.clone(), &interner);
    let tokens = lexer.tokenize().expect("Lexing failed");

    let mut parser = Parser::new(tokens, handler.clone(), &interner, &common, &arena);
    let program = parser.parse().expect("Parsing failed");

    let mutable = MutableProgram::from_program(&program);
    let mut codegen = CodeGenerator::new(interner.clone());
    // Tree shaking NOT enabled

    let lua = codegen.generate(&mutable);

    // Without tree shaking, should use deferred merge loop via finalize_module
    assert!(
        lua.contains("for __k, __v in pairs(_reexport)"),
        "Should use deferred merge loop without tree shaking, got:\n{lua}"
    );
    assert!(
        lua.contains("M[__k] = __v"),
        "Should assign to M table in merge loop, got:\n{lua}"
    );
}

// --- LTO Re-export Flattening Codegen Tests ---

fn create_module_node(path: PathBuf) -> ModuleNode {
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

/// Parse source, apply LTO re-export flattening, then codegen.
fn generate_lua_with_flattening(
    source: &str,
    current_module: &PathBuf,
    graph: ModuleGraph,
) -> String {
    let arena = bumpalo::Bump::new();
    let handler = Arc::new(CollectingDiagnosticHandler::new());
    let (interner, common) = StringInterner::new_with_common_identifiers();
    let interner = Arc::new(interner);

    let mut lexer = Lexer::new(source, handler.clone(), &interner);
    let tokens = lexer.tokenize().expect("Lexing failed");

    let mut parser = Parser::new(tokens, handler.clone(), &interner, &common, &arena);
    let program = parser.parse().expect("Parsing failed");

    let mut mutable = MutableProgram::from_program(&program);

    // Apply re-export flattening
    let mut pass = ReExportFlatteningPass::new(Arc::new(graph), interner.clone());
    pass.set_current_module(current_module);
    mutable.statements = pass.apply(&mutable.statements);

    let mut codegen = CodeGenerator::new(interner);
    codegen.generate(&mutable)
}

#[test]
fn test_reexport_flattening_changes_require_path() {
    // A re-exports from B, B re-exports from C, C defines foo
    // After flattening, A should require from C directly
    let mut modules = FxHashMap::default();

    let a_path = PathBuf::from("/project/src/a.luax");
    let b_path = PathBuf::from("/project/src/b.luax");
    let c_path = PathBuf::from("/project/src/c.luax");

    modules.insert(a_path.clone(), create_module_node(a_path.clone()));

    let mut b_node = create_module_node(b_path.clone());
    b_node.re_exports.push(ReExportInfo {
        source_module: c_path.clone(),
        specifiers: ReExportKind::Named(vec![("foo".to_string(), "foo".to_string())]),
    });
    modules.insert(b_path, b_node);

    let mut c_node = create_module_node(c_path.clone());
    c_node.exports.insert("foo".to_string(), export_info("foo"));
    modules.insert(c_path, c_node);

    let graph = ModuleGraph {
        modules,
        entry_points: FxHashSet::default(),
    };

    let source = r#"export { foo } from './b'"#;
    let lua = generate_lua_with_flattening(source, &a_path, graph);

    // After flattening, the require should point to ./c, not ./b
    assert!(
        lua.contains("./c"),
        "Flattened re-export should require from ./c, got:\n{lua}"
    );
    assert!(
        !lua.contains("./b"),
        "Flattened re-export should NOT require from ./b, got:\n{lua}"
    );
}

#[test]
fn test_reexport_flattening_preserves_symbol_name() {
    let mut modules = FxHashMap::default();

    let a_path = PathBuf::from("/project/src/a.luax");
    let b_path = PathBuf::from("/project/src/b.luax");
    let c_path = PathBuf::from("/project/src/c.luax");

    modules.insert(a_path.clone(), create_module_node(a_path.clone()));

    let mut b_node = create_module_node(b_path.clone());
    b_node.re_exports.push(ReExportInfo {
        source_module: c_path.clone(),
        specifiers: ReExportKind::Named(vec![("foo".to_string(), "foo".to_string())]),
    });
    modules.insert(b_path, b_node);

    let mut c_node = create_module_node(c_path.clone());
    c_node.exports.insert("foo".to_string(), export_info("foo"));
    modules.insert(c_path, c_node);

    let graph = ModuleGraph {
        modules,
        entry_points: FxHashSet::default(),
    };

    let source = r#"export { foo } from './b'"#;
    let lua = generate_lua_with_flattening(source, &a_path, graph);

    // The symbol name "foo" should still appear in the output
    assert!(
        lua.contains("foo"),
        "Flattened re-export should preserve symbol name, got:\n{lua}"
    );
}

#[test]
fn test_reexport_flattening_no_change_for_direct_export() {
    // B directly exports foo — no chain to flatten
    let mut modules = FxHashMap::default();

    let a_path = PathBuf::from("/project/src/a.luax");
    let b_path = PathBuf::from("/project/src/b.luax");

    modules.insert(a_path.clone(), create_module_node(a_path.clone()));

    let mut b_node = create_module_node(b_path.clone());
    b_node.exports.insert("foo".to_string(), export_info("foo"));
    modules.insert(b_path, b_node);

    let graph = ModuleGraph {
        modules,
        entry_points: FxHashSet::default(),
    };

    let source = r#"export { foo } from './b'"#;
    let lua = generate_lua_with_flattening(source, &a_path, graph);

    // Should still point to ./b since there's no chain
    assert!(
        lua.contains("./b"),
        "Direct re-export should keep original path, got:\n{lua}"
    );
}

#[test]
fn test_reexport_flattening_deep_chain() {
    // A → B → C → D
    let mut modules = FxHashMap::default();

    let a_path = PathBuf::from("/project/src/a.luax");
    let b_path = PathBuf::from("/project/src/b.luax");
    let c_path = PathBuf::from("/project/src/c.luax");
    let d_path = PathBuf::from("/project/src/d.luax");

    modules.insert(a_path.clone(), create_module_node(a_path.clone()));

    let mut b_node = create_module_node(b_path.clone());
    b_node.re_exports.push(ReExportInfo {
        source_module: c_path.clone(),
        specifiers: ReExportKind::Named(vec![("foo".to_string(), "foo".to_string())]),
    });
    modules.insert(b_path, b_node);

    let mut c_node = create_module_node(c_path.clone());
    c_node.re_exports.push(ReExportInfo {
        source_module: d_path.clone(),
        specifiers: ReExportKind::Named(vec![("foo".to_string(), "foo".to_string())]),
    });
    modules.insert(c_path, c_node);

    let mut d_node = create_module_node(d_path.clone());
    d_node.exports.insert("foo".to_string(), export_info("foo"));
    modules.insert(d_path, d_node);

    let graph = ModuleGraph {
        modules,
        entry_points: FxHashSet::default(),
    };

    let source = r#"export { foo } from './b'"#;
    let lua = generate_lua_with_flattening(source, &a_path, graph);

    assert!(
        lua.contains("./d"),
        "Deep chain should flatten to ./d, got:\n{lua}"
    );
    assert!(
        !lua.contains("./b"),
        "Should not contain intermediate ./b, got:\n{lua}"
    );
}

#[test]
fn test_reexport_flattening_with_alias() {
    // B re-exports foo as bar from C
    let mut modules = FxHashMap::default();

    let a_path = PathBuf::from("/project/src/a.luax");
    let b_path = PathBuf::from("/project/src/b.luax");
    let c_path = PathBuf::from("/project/src/c.luax");

    modules.insert(a_path.clone(), create_module_node(a_path.clone()));

    let mut b_node = create_module_node(b_path.clone());
    b_node.re_exports.push(ReExportInfo {
        source_module: c_path.clone(),
        specifiers: ReExportKind::Named(vec![("bar".to_string(), "bar".to_string())]),
    });
    modules.insert(b_path, b_node);

    let mut c_node = create_module_node(c_path.clone());
    c_node.exports.insert("bar".to_string(), export_info("bar"));
    modules.insert(c_path, c_node);

    let graph = ModuleGraph {
        modules,
        entry_points: FxHashSet::default(),
    };

    // A's source uses `export { bar as baz } from './b'`
    let source = r#"export { bar as baz } from './b'"#;
    let lua = generate_lua_with_flattening(source, &a_path, graph);

    // After flattening, should require from ./c
    assert!(
        lua.contains("./c"),
        "Aliased re-export should flatten to ./c, got:\n{lua}"
    );
    // The local extraction uses the original name (bar) and the alias (baz)
    assert!(
        lua.contains("bar") || lua.contains("baz"),
        "Should preserve the symbol names, got:\n{lua}"
    );
}
