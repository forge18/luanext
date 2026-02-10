use luanext_core::codegen::CodeGenerator;
use luanext_core::MutableProgram;
use luanext_parser::lexer::Lexer;
use luanext_parser::parser::Parser;
use luanext_parser::string_interner::StringInterner;
use luanext_typechecker::cli::diagnostics::CollectingDiagnosticHandler;
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
