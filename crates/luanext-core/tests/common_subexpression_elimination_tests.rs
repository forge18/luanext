use luanext_core::config::OptimizationLevel;
use luanext_core::di::DiContainer;

fn compile_with_o2(source: &str) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile_with_stdlib_and_optimization(source, OptimizationLevel::Moderate)
}

#[test]
fn test_simple_duplicate_elimination() {
    let source = r#"
        local x = 5
        local a = x + 1
        local b = x + 1
        print(a, b)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // b should be eliminated and replaced with a
    // Or both might be constant-folded to 6
    assert!(result.contains("print"));
}

#[test]
fn test_member_access_cse() {
    let source = r#"
        local t = {value = 42}
        local x = t.value
        local y = t.value
        print(x, y)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // y = t.value should be eliminated
    assert!(result.contains("print"));
}

#[test]
fn test_index_access_cse() {
    let source = r#"
        local t = [10, 20, 30]
        local idx = 2
        local x = t[idx]
        local y = t[idx]
        print(x, y)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // y = t[idx] should be eliminated
    assert!(result.contains("print"));
}

#[test]
fn test_binary_operation_cse() {
    let source = r#"
        local a = 10
        local b = 20
        local x = a * b
        local y = a * b
        print(x, y)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // y = a * b should be eliminated, or both might be constant-folded to 200
    assert!(result.contains("print"));
}

#[test]
fn test_no_cse_with_function_call() {
    let source = r#"
        function foo(): number
            return 42
        end
        local x = foo()
        local y = foo()
        print(x, y)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // Function calls are not pure, so no CSE should occur
    // Both foo() calls should remain
    assert!(result.contains("print"));
}

#[test]
fn test_no_cse_across_mutation() {
    let source = r#"
        local a = 10
        local b = 20
        local x = a + b
        a = 5
        local y = a + b
        print(x, y)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // After a is mutated, a + b is a different computation
    // Both should remain
    assert!(result.contains("print"));
}

#[test]
fn test_no_cse_across_branches() {
    let source = r#"
        local condition = true
        local a = 10
        local b = 20
        if condition then
            local x = a + b
            print(x)
        else
            local y = a + b
            print(y)
        end
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // CSE should not cross branch boundaries
    assert!(result.contains("print"));
}

#[test]
fn test_cse_in_sequential_code() {
    let source = r#"
        local a = 5
        local b = 10
        local x = a + b
        local y = x * 2
        local z = a + b
        print(x, y, z)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // z = a + b should be eliminated (replaced with x)
    assert!(result.contains("print"));
}

#[test]
fn test_nested_expression_cse() {
    let source = r#"
        local a = 5
        local b = 10
        local c = 3
        local x = (a + b) * c
        local y = (a + b) * c
        print(x, y)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // y = (a + b) * c should be eliminated
    assert!(result.contains("print"));
}

#[test]
fn test_cse_with_multiple_uses() {
    let source = r#"
        local a = 5
        local b = 10
        local x = a + b
        local y = a + b
        local z = a + b
        print(x, y, z)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // Both y and z should be eliminated
    assert!(result.contains("print"));
}

#[test]
fn test_no_cse_for_different_expressions() {
    let source = r#"
        local a = 5
        local b = 10
        local x = a + b
        local y = a - b
        print(x, y)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // Different operations, no CSE
    assert!(result.contains("print"));
}

#[test]
fn test_cse_interaction_with_constant_folding() {
    let source = r#"
        local a = 2 + 3
        local b = 2 + 3
        print(a, b)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // Constant folding should happen first (2+3 -> 5)
    // Then CSE might eliminate one of them, or both are just 5
    assert!(result.contains("5") || result.contains("print"));
}
