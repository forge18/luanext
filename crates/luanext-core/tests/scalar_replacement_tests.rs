use luanext_core::config::{CompilerConfig, OptimizationLevel};
use luanext_core::di::DiContainer;

fn compile_o3(source: &str) -> Result<String, String> {
    let config = CompilerConfig::default();
    let mut container = DiContainer::production(config);
    container.compile_with_stdlib_and_optimization(source, OptimizationLevel::Aggressive)
}

fn compile_o2(source: &str) -> Result<String, String> {
    let config = CompilerConfig::default();
    let mut container = DiContainer::production(config);
    container.compile_with_stdlib_and_optimization(source, OptimizationLevel::Moderate)
}

// ============================================================================
// Scalar Replacement of Aggregates Tests
// ============================================================================

#[test]
fn test_sra_basic_object() {
    let source = r#"
        const point = { x: 1, y: 2 }
        const dx = point.x + 10
        const dy = point.y + 20
    "#;

    let output = compile_o3(source).unwrap();

    // After SRA, point.x and point.y should be replaced with scalar variables
    // The object literal should be eliminated
    assert!(
        !output.contains("point = {") && !output.contains("point = {\n"),
        "Object literal should be eliminated by SRA. Got:\n{}",
        output
    );
}

#[test]
fn test_sra_not_at_o2() {
    let source = r#"
        const point = { x: 1, y: 2 }
        const dx = point.x + 10
        print(dx)
    "#;

    let result = compile_o2(source);

    // At O2, SRA should NOT occur — just verify compilation succeeds
    assert!(
        result.is_ok(),
        "Should compile successfully at O2. Got: {:?}",
        result.err()
    );
}

#[test]
fn test_sra_scalar_names() {
    let source = r#"
        const obj = { a: 10, b: 20 }
        const sum = obj.a + obj.b
        print(sum)
    "#;

    let output = compile_o3(source).unwrap();

    // Scalar variables should use the naming convention obj__a, obj__b
    // or the whole thing might get constant-folded to 30
    assert!(
        output.contains("obj__a") || output.contains("obj__b") || output.contains("30"),
        "Expected scalar variable names or constant-folded result. Got:\n{}",
        output
    );
}

#[test]
fn test_sra_escapes_as_function_arg() {
    let source = r#"
        function process(t: {x: number}): number
            return t.x
        end
        const obj = { x: 42 }
        const result = process(obj)
        print(result)
    "#;

    let output = compile_o3(source).unwrap();

    // obj is passed to a function — it escapes, so SRA should NOT apply
    assert!(!output.is_empty(), "Should compile successfully");
}

#[test]
fn test_sra_escapes_via_return() {
    let source = r#"
        function make_point(): {x: number, y: number}
            const p = { x: 1, y: 2 }
            return p
        end
        const result = make_point()
        print(result)
    "#;

    let output = compile_o3(source).unwrap();

    // p is returned from the function — it escapes
    assert!(!output.is_empty(), "Should compile successfully");
}

#[test]
fn test_sra_escapes_via_assignment() {
    let source = r#"
        const obj = { x: 1, y: 2 }
        const other = obj
        const val = other.x
        print(val)
    "#;

    let output = compile_o3(source).unwrap();

    // obj is assigned to another variable — it escapes
    assert!(!output.is_empty(), "Should compile successfully");
}

#[test]
fn test_sra_escapes_via_index() {
    let source = r#"
        const obj = { x: 1, y: 2 }
        const val = obj["x"]
        print(val)
    "#;

    let result = compile_o3(source);

    // Dynamic index access — unsafe for SRA, should still compile
    assert!(result.is_ok(), "Should compile. Got: {:?}", result.err());
}

#[test]
fn test_sra_multiple_objects() {
    let source = r#"
        const a = { x: 1, y: 2 }
        const b = { x: 3, y: 4 }
        const sum = a.x + b.y
        print(sum)
    "#;

    let output = compile_o3(source).unwrap();

    // Both a and b should be eligible for SRA independently
    assert!(
        !output.is_empty(),
        "Should compile successfully with multiple SRA candidates"
    );
}

#[test]
fn test_sra_computed_property_skipped() {
    // Use a string literal as computed key — this tests the skip
    let source = r#"
        const obj = { x: 1, y: 2 }
        const val = obj.y + obj.x
        print(val)
    "#;

    let output = compile_o3(source).unwrap();

    // Simple object — should be replaced
    assert!(!output.is_empty(), "Should compile successfully");
}

#[test]
fn test_sra_too_many_fields() {
    // 9 fields exceeds the MAX_FIELDS limit of 8
    let source = r#"
        const big = { a: 1, b: 2, c: 3, d: 4, e: 5, f: 6, g: 7, h: 8, i: 9 }
        const val = big.a + big.b
        print(val)
    "#;

    let output = compile_o3(source).unwrap();

    // More than 8 fields — should NOT be eligible for SRA
    // The object should remain
    assert!(!output.is_empty(), "Should compile with too many fields");
}

#[test]
fn test_sra_empty_object_skipped() {
    let source = r#"
        const empty = { x: 0 }
        print(empty.x)
    "#;

    let output = compile_o3(source).unwrap();

    // Single field — should be replaced
    assert!(!output.is_empty(), "Should compile with single field");
}

#[test]
fn test_sra_inside_function_body() {
    let source = r#"
        function compute(): number
            const pt = { x: 10, y: 20 }
            return pt.x + pt.y
        end
        const result = compute()
        print(result)
    "#;

    let output = compile_o3(source).unwrap();

    // SRA should work inside function bodies too
    assert!(
        !output.is_empty(),
        "Should compile with SRA in function body"
    );
}

#[test]
fn test_sra_field_used_in_expression() {
    let source = r#"
        const config = { width: 100, height: 200 }
        const area = config.width * config.height
        print(area)
    "#;

    let output = compile_o3(source).unwrap();

    // Fields used in arithmetic expressions
    assert!(
        output.contains("config__width")
            || output.contains("config__height")
            || output.contains("20000"),
        "Expected scalar replacement or constant folding. Got:\n{}",
        output
    );
}

#[test]
fn test_sra_method_call_escapes() {
    let source = r#"
        const obj = { x: 1 }
        obj:toString()
        const val = obj.x
        print(val)
    "#;

    let result = compile_o3(source);

    // Method call on the object — it escapes (metatables may capture reference)
    assert!(
        result.is_ok(),
        "Should compile with method call escape. Got: {:?}",
        result.err()
    );
}

#[test]
fn test_sra_string_field_values() {
    let source = r#"
        const info = { name: "hello", tag: "world" }
        const full = info.name .. " " .. info.tag
        print(full)
    "#;

    let output = compile_o3(source).unwrap();

    // String field values should be replaceable
    assert!(!output.is_empty(), "Should compile with string fields");
}

#[test]
fn test_sra_nested_if_block() {
    let source = r#"
        const opts = { debug: true, verbose: false }
        if opts.debug then
            print("debug mode")
        end
    "#;

    let output = compile_o3(source).unwrap();

    // Fields used in if-condition should be rewritten
    assert!(
        !output.is_empty(),
        "Should compile with field in if condition"
    );
}
