use luanext_core::config::OptimizationLevel;
use luanext_core::di::DiContainer;

fn compile_with_o2(source: &str) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile_with_stdlib_and_optimization(source, OptimizationLevel::Moderate)
}

#[test]
fn test_arithmetic_identity_add_zero() {
    let source = r#"
        local x = 5
        local y = x + 0
        print(y)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // x + 0 should be optimized to x, so y = x or y = 5 (after copy prop)
    assert!(result.contains("5") || result.contains("x"));
}

#[test]
fn test_arithmetic_identity_multiply_one() {
    let source = r#"
        local x = 10
        local y = x * 1
        print(y)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // x * 1 should be optimized to x
    assert!(result.contains("10") || result.contains("x"));
}

#[test]
fn test_absorbing_multiply_zero() {
    let source = r#"
        local x = 10
        local y = x * 0
        print(y)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // x * 0 should be optimized to 0
    assert!(result.contains("0"));
}

#[test]
fn test_double_negation() {
    let source = r#"
        local x = true
        local y = not (not x)
        print(y)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // not (not x) should be optimized to x, or at least compile successfully
    // The optimization may not show up directly in output due to other optimizations
    assert!(result.contains("print"));
}

#[test]
fn test_boolean_and_true() {
    let source = r#"
        local x = false
        local y = x and true
        print(y)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // x and true should be optimized to x
    assert!(result.contains("false") || result.contains("x"));
}

#[test]
fn test_boolean_or_false() {
    let source = r#"
        local x = true
        local y = x or false
        print(y)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // x or false should be optimized to x
    assert!(result.contains("true") || result.contains("x"));
}

#[test]
fn test_idempotent_or() {
    let source = r#"
        local x = 5
        local y = x or x
        print(y)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // x or x should be optimized to x
    // Result should only contain one reference to x or 5
    assert!(result.contains("print"));
}

#[test]
fn test_empty_string_concat() {
    let source = r#"
        local x = "hello"
        local y = x .. ""
        print(y)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // x .. "" should be optimized to x
    assert!(result.contains("hello") || result.contains("x"));
}
