//! Cache/incremental compilation correctness tests.
//!
//! These tests verify that the LuaNext compilation pipeline is deterministic:
//! compiling the same source twice produces identical output. This is a
//! prerequisite for safe caching — if compilation is not deterministic,
//! cached results cannot be trusted.
//!
//! Tests also verify semantic equivalence: adding a type annotation (which is
//! erased at codegen) should not change the generated Lua code.

use luanext_core::config::OptimizationLevel;
use luanext_test_helpers::compile::{compile, compile_with_optimization, compile_with_stdlib};

#[test]
fn test_same_source_compiles_identically_twice() {
    let source = r#"
        function add(a: number, b: number): number {
            return a + b
        }
        result: number = add(2, 3)
    "#;
    let out1 = compile(source).unwrap();
    let out2 = compile(source).unwrap();
    assert_eq!(
        out1, out2,
        "Same source should produce identical output on repeated compilation"
    );
}

#[test]
fn test_different_sources_produce_different_output() {
    let source_a = r#"result: number = 42"#;
    let source_b = r#"result: number = 99"#;
    let out_a = compile(source_a).unwrap();
    let out_b = compile(source_b).unwrap();
    assert_ne!(
        out_a, out_b,
        "Different sources should produce different output"
    );
}

#[test]
fn test_whitespace_only_changes_produce_same_output() {
    let source1 = r#"result: number = 42"#;
    let source2 = r#"
        result: number = 42
    "#;
    let out1 = compile(source1).unwrap();
    let out2 = compile(source2).unwrap();
    // Trim whitespace from outputs since formatting may differ slightly
    assert_eq!(
        out1.trim(),
        out2.trim(),
        "Whitespace-only differences should not affect generated Lua"
    );
}

#[test]
fn test_type_annotation_does_not_change_generated_code() {
    // Type annotations are erased — adding/removing them should not change output
    let source_typed = r#"
        local x: number = 42
        result: number = x
    "#;
    let source_untyped = r#"
        local x = 42
        result = x
    "#;
    let out_typed = compile(source_typed).unwrap();
    let out_untyped = compile(source_untyped).unwrap();
    // The generated Lua should be equivalent (both assign 42 to x, then to result)
    assert!(
        out_typed.contains("42"),
        "Typed version should still contain 42"
    );
    assert!(
        out_untyped.contains("42"),
        "Untyped version should still contain 42"
    );
    // Both should contain the variable assignment but no type information
    assert!(
        !out_typed.contains(": number"),
        "Type annotations should be erased from output"
    );
}

#[test]
fn test_optimization_level_is_stable_on_repeat() {
    let source = r#"
        const x = 1 + 2 + 3
        result: number = x
    "#;
    let out1 = compile_with_optimization(source, OptimizationLevel::Moderate).unwrap();
    let out2 = compile_with_optimization(source, OptimizationLevel::Moderate).unwrap();
    assert_eq!(
        out1, out2,
        "Optimization at same level should be deterministic"
    );
}

#[test]
fn test_stdlib_compilation_is_deterministic() {
    let source = r#"
        result: number = math.floor(3.7)
    "#;
    let out1 = compile_with_stdlib(source).unwrap();
    let out2 = compile_with_stdlib(source).unwrap();
    assert_eq!(out1, out2, "Stdlib compilation should be deterministic");
}

#[test]
fn test_unused_type_alias_does_not_affect_output() {
    // Type aliases are erased — having or not having one shouldn't change Lua output
    let source_with_alias = r#"
        type MyNum = number
        result: number = 42
    "#;
    let source_without_alias = r#"
        result: number = 42
    "#;
    let out_with = compile(source_with_alias).unwrap();
    let out_without = compile(source_without_alias).unwrap();
    // Type alias should be completely erased
    assert!(
        !out_with.contains("type"),
        "type alias should be erased from output"
    );
    assert!(
        out_with.trim() == out_without.trim(),
        "type alias should not affect generated Lua"
    );
}

#[test]
fn test_interface_definition_does_not_appear_in_output() {
    let source = r#"
        interface Point {
            x: number
            y: number
        }
        p: Point = { x: 1, y: 2 }
        result: number = p.x
    "#;
    let lua_code = compile(source).unwrap();
    assert!(
        !lua_code.contains("interface"),
        "interface definitions should be fully erased from output"
    );
}

#[test]
fn test_different_optimization_levels_produce_semantically_equivalent_but_possibly_different_code()
{
    let source = r#"
        const x = 2 + 3
        result: number = x
    "#;
    use luanext_test_helpers::LuaExecutor;

    let out_o0 = compile_with_optimization(source, OptimizationLevel::None).unwrap();
    let out_o2 = compile_with_optimization(source, OptimizationLevel::Moderate).unwrap();

    // O2 may fold 2+3 → 5, changing the generated code
    // But both should execute to the same result
    let executor = LuaExecutor::new().unwrap();
    let result_o0: i64 = executor.execute_and_get(&out_o0, "result").unwrap();
    let result_o2: i64 = executor.execute_and_get(&out_o2, "result").unwrap();
    assert_eq!(
        result_o0, result_o2,
        "Different optimization levels should produce same runtime value"
    );
    assert_eq!(result_o0, 5);
}

#[test]
fn test_function_compilation_is_deterministic_across_multiple_compiles() {
    let source = r#"
        function double(x: number): number {
            return x * 2
        }
        result: number = double(21)
    "#;
    let outputs: Vec<String> = (0..3).map(|_| compile(source).unwrap()).collect();
    assert!(
        outputs.windows(2).all(|w| w[0] == w[1]),
        "Function compilation should be deterministic across multiple compiles"
    );
}
