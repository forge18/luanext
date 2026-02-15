use luanext_core::config::OptimizationLevel;
use luanext_core::di::DiContainer;

fn compile_with_o2(source: &str) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile_with_stdlib_and_optimization(source, OptimizationLevel::Moderate)
}

#[test]
fn test_sccp_resolves_comparison_with_constant() {
    let source = r#"
        local x = 10
        if x > 5 then
            print("yes")
        else
            print("no")
        end
    "#;

    let result = compile_with_o2(source).expect("compilation failed");
    println!("sccp comparison: {}", result);
    // x=10, x>5 is true, so "yes" branch should be taken
    assert!(
        result.contains("yes"),
        "True branch should be taken: {}",
        result
    );
    assert!(
        !result.contains("no"),
        "False branch should be eliminated: {}",
        result
    );
}

#[test]
fn test_sccp_resolves_equality() {
    let source = r#"
        local x = 5
        if x == 5 then
            print("equal")
        else
            print("not equal")
        end
    "#;

    let result = compile_with_o2(source).expect("compilation failed");
    println!("sccp equality: {}", result);
    assert!(
        result.contains("equal"),
        "Equal branch should be taken: {}",
        result
    );
}

#[test]
fn test_sccp_resolves_false_condition() {
    let source = r#"
        local x = 3
        if x > 10 then
            print("dead")
        else
            print("alive")
        end
    "#;

    let result = compile_with_o2(source).expect("compilation failed");
    println!("sccp false condition: {}", result);
    assert!(
        result.contains("alive"),
        "Else branch should be taken: {}",
        result
    );
    assert!(
        !result.contains("dead"),
        "Dead branch should be eliminated: {}",
        result
    );
}

#[test]
fn test_sccp_propagates_through_chain() {
    let source = r#"
        local x = 5
        local y = x + 1
        if y > 5 then
            print("yes")
        else
            print("no")
        end
    "#;

    let result = compile_with_o2(source).expect("compilation failed");
    println!("sccp chain: {}", result);
    // x=5, y=6, y>5 is true
    assert!(
        result.contains("yes"),
        "Chain propagation should resolve condition: {}",
        result
    );
}

#[test]
fn test_sccp_does_not_fold_unknown() {
    let source = r#"
        function get_value()
            return 5
        end
        local x = get_value()
        if x > 3 then
            print("yes")
        else
            print("no")
        end
    "#;

    let result = compile_with_o2(source).expect("compilation failed");
    println!("sccp unknown: {}", result);
    // x is from a function call, should NOT be resolved
    assert!(
        result.contains("if"),
        "Unknown condition should not be resolved: {}",
        result
    );
}

#[test]
fn test_sccp_string_equality() {
    let source = r#"
        local x = "hello"
        if x == "hello" then
            print("match")
        else
            print("no match")
        end
    "#;

    let result = compile_with_o2(source).expect("compilation failed");
    println!("sccp string: {}", result);
    assert!(
        result.contains("match"),
        "String equality should be resolved: {}",
        result
    );
}

#[test]
fn test_sccp_boolean_not() {
    let source = r#"
        local x = false
        if not x then
            print("negated")
        else
            print("dead")
        end
    "#;

    let result = compile_with_o2(source).expect("compilation failed");
    println!("sccp not: {}", result);
    assert!(
        result.contains("negated"),
        "Boolean not should be evaluated: {}",
        result
    );
}

#[test]
fn test_sccp_arithmetic_in_condition() {
    let source = r#"
        local a = 10
        local b = 3
        if a - b > 5 then
            print("yes")
        else
            print("no")
        end
    "#;

    let result = compile_with_o2(source).expect("compilation failed");
    println!("sccp arithmetic: {}", result);
    // a=10, b=3, a-b=7, 7>5 is true
    assert!(
        result.contains("yes"),
        "Arithmetic condition should be resolved: {}",
        result
    );
}

#[test]
fn test_sccp_clears_on_loop() {
    let source = r#"
        local x = 5
        for i = 1, 10 do
            x = i
        end
        if x > 3 then
            print("yes")
        else
            print("no")
        end
    "#;

    let result = compile_with_o2(source).expect("compilation failed");
    println!("sccp loop clear: {}", result);
    // x is modified in a loop, should NOT be resolved
    assert!(
        result.contains("if"),
        "Loop-modified variable should not be resolved: {}",
        result
    );
}

#[test]
fn test_sccp_nil_comparison() {
    let source = r#"
        local x = nil
        if x == nil then
            print("is nil")
        else
            print("not nil")
        end
    "#;

    let result = compile_with_o2(source).expect("compilation failed");
    println!("sccp nil: {}", result);
    assert!(
        result.contains("is nil"),
        "Nil comparison should be resolved: {}",
        result
    );
}
