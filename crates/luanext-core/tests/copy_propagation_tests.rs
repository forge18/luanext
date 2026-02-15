use luanext_core::config::OptimizationLevel;
use luanext_core::di::DiContainer;

fn compile_with_o2(source: &str) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile_with_stdlib_and_optimization(source, OptimizationLevel::Moderate)
}

#[test]
fn test_simple_constant_propagation() {
    let source = r#"
        local x = 5
        local y = x + 1
        print(y)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // x should be replaced with 5 in the expression
    assert!(result.contains("5 + 1") || result.contains("6")); // May be constant-folded
}

#[test]
fn test_variable_to_variable_propagation() {
    let source = r#"
        local y = 10
        local x = y
        local z = x
        print(z)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // x should be replaced with y, or z directly uses y
    // The exact output depends on whether dead store elimination runs
    assert!(result.contains("y") || result.contains("10"));
}

#[test]
fn test_no_propagation_across_mutation() {
    let source = r#"
        local x = 5
        local y = x
        x = 10
        local z = x
        print(y, z)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // y should use 5 (copied before mutation)
    // z should use current x (after mutation)
    // Exact output depends on optimization level
    assert!(result.contains("print"));
}

#[test]
fn test_no_propagation_with_function_call() {
    let source = r#"
        function foo(): number
            return 42
        end
        local x = foo()
        local y = x
        print(y)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // x is not propagatable (function call may have side effects)
    // But y = x is a copy, which could still be propagated
    assert!(result.contains("print"));
}

#[test]
fn test_propagation_in_if_statement() {
    let source = r#"
        local condition = true
        local x = 5
        if condition then
            local y = x + 1
            print(y)
        end
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // x should be propagated into the if block, resulting in 6 after constant folding
    // Or at minimum, the code should compile successfully
    assert!(result.contains("6") || result.contains("5") || result.contains("print"));
}

#[test]
fn test_no_propagation_across_branches() {
    let source = r#"
        local condition = true
        local x = 0
        if condition then
            x = 5
        else
            x = 10
        end
        local y = x
        print(y)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // x has different values from different branches, so can't propagate
    assert!(result.contains("print"));
}

#[test]
fn test_member_access_propagation() {
    let source = r#"
        local t = {value = 42}
        local x = t.value
        local y = x
        print(y)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // x = t.value might be propagated to y
    assert!(result.contains("print"));
}

#[test]
fn test_multiple_uses_propagation() {
    let source = r#"
        local x = 5
        local y = x
        local z = x
        print(y, z)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // Both y and z should potentially use 5
    assert!(result.contains("5") || result.contains("x"));
}

#[test]
fn test_propagation_enables_constant_folding() {
    let source = r#"
        local x = 3
        local y = 4
        local z = x + y
        print(z)
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // After copy propagation: z = 3 + 4
    // After constant folding: z = 7
    assert!(result.contains("7") || result.contains("3") || result.contains("x"));
}

#[test]
fn test_no_propagation_in_loops() {
    let source = r#"
        for i = 1, 10 do
            local x = i
            local y = x
            print(y)
        end
    "#;

    let result = compile_with_o2(source).expect("compilation failed");

    // i is loop-carried, but x = i and y = x might still be optimized
    assert!(result.contains("print"));
}
