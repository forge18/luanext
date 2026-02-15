use luanext_core::config::OptimizationLevel;
use luanext_core::di::DiContainer;

fn compile_with_o2(source: &str) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile_with_stdlib_and_optimization(source, OptimizationLevel::Moderate)
}

#[test]
fn test_if_true_inlines_then_block() {
    let source = r#"
        if true then
            print("hello")
        end
    "#;

    let result = compile_with_o2(source).expect("compilation failed");
    println!("if true then: {}", result);
    assert!(
        result.contains("print"),
        "Then block should be inlined: {}",
        result
    );
    assert!(
        !result.contains("if"),
        "If statement should be eliminated: {}",
        result
    );
}

#[test]
fn test_if_false_removes_entirely() {
    let source = r#"
        if false then
            print("dead")
        end
        print("alive")
    "#;

    let result = compile_with_o2(source).expect("compilation failed");
    println!("if false: {}", result);
    assert!(
        result.contains("alive"),
        "Surviving code should remain: {}",
        result
    );
    assert!(
        !result.contains("dead"),
        "Dead branch should be removed: {}",
        result
    );
}

#[test]
fn test_if_true_with_else_keeps_then() {
    let source = r#"
        if true then
            print("taken")
        else
            print("dead")
        end
    "#;

    let result = compile_with_o2(source).expect("compilation failed");
    println!("if true else: {}", result);
    assert!(
        result.contains("taken"),
        "Then branch should be kept: {}",
        result
    );
    assert!(
        !result.contains("dead"),
        "Else branch should be removed: {}",
        result
    );
}

#[test]
fn test_if_false_with_else_keeps_else() {
    let source = r#"
        if false then
            print("dead")
        else
            print("taken")
        end
    "#;

    let result = compile_with_o2(source).expect("compilation failed");
    println!("if false else: {}", result);
    assert!(
        result.contains("taken"),
        "Else branch should be kept: {}",
        result
    );
    assert!(
        !result.contains("dead"),
        "Then branch should be removed: {}",
        result
    );
}

#[test]
fn test_if_false_promotes_elseif() {
    let source = r#"
        local x = 10
        if false then
            print("dead")
        elseif x > 5 then
            print("taken")
        end
    "#;

    let result = compile_with_o2(source).expect("compilation failed");
    println!("if false elseif: {}", result);
    assert!(
        !result.contains("dead"),
        "Dead first branch should be removed: {}",
        result
    );
    assert!(
        result.contains("taken"),
        "Elseif branch should be promoted: {}",
        result
    );
}

#[test]
fn test_if_nil_is_falsy() {
    let source = r#"
        if nil then
            print("dead")
        else
            print("taken")
        end
    "#;

    let result = compile_with_o2(source).expect("compilation failed");
    println!("if nil: {}", result);
    assert!(
        result.contains("taken"),
        "nil is falsy, else should be taken: {}",
        result
    );
    assert!(
        !result.contains("dead"),
        "nil condition should eliminate then branch: {}",
        result
    );
}

#[test]
fn test_while_false_removed() {
    let source = r#"
        while false do
            print("dead")
        end
        print("alive")
    "#;

    let result = compile_with_o2(source).expect("compilation failed");
    println!("while false: {}", result);
    assert!(
        result.contains("alive"),
        "Code after dead loop should remain: {}",
        result
    );
    assert!(
        !result.contains("dead"),
        "Dead loop body should be removed: {}",
        result
    );
}

#[test]
fn test_nested_if_true_in_function() {
    let source = r#"
        function foo()
            if true then
                return 42
            else
                return 0
            end
        end
        print(foo())
    "#;

    let result = compile_with_o2(source).expect("compilation failed");
    println!("nested if true in function: {}", result);
    assert!(
        result.contains("42"),
        "True branch return should be kept: {}",
        result
    );
}

#[test]
fn test_chained_constant_if() {
    let source = r#"
        if true then
            if true then
                print("deep")
            end
        end
    "#;

    let result = compile_with_o2(source).expect("compilation failed");
    println!("chained if true: {}", result);
    assert!(
        result.contains("deep"),
        "Deeply nested true branches should all be inlined: {}",
        result
    );
    assert!(
        !result.contains("if"),
        "All if statements should be eliminated: {}",
        result
    );
}

#[test]
fn test_number_literal_is_truthy() {
    let source = r#"
        if 1 then
            print("taken")
        else
            print("dead")
        end
    "#;

    let result = compile_with_o2(source).expect("compilation failed");
    println!("if 1: {}", result);
    assert!(
        result.contains("taken"),
        "Number literals are truthy in Lua: {}",
        result
    );
}
