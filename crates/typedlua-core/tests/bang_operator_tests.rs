use typedlua_core::di::DiContainer;

fn compile_and_check(source: &str) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile(source)
}

#[test]
fn test_bang_with_boolean() {
    let source = r#"
        const a = true
        const b = !a
    "#;

    let output = compile_and_check(source).unwrap();
    assert!(output.contains("not a"), "Should compile ! to 'not'");
}

#[test]
fn test_bang_with_literal() {
    let source = r#"
        const a = !true
        const b = !false
    "#;

    let output = compile_and_check(source).unwrap();
    assert!(
        output.contains("not true"),
        "Should compile !true to 'not true'"
    );
    assert!(
        output.contains("not false"),
        "Should compile !false to 'not false'"
    );
}

#[test]
fn test_bang_with_expression() {
    let source = r#"
        const a = 5
        const b = !(a > 3)
    "#;

    let output = compile_and_check(source).unwrap();
    assert!(output.contains("not"), "Should compile ! operator");
}

#[test]
fn test_bang_with_variable() {
    let source = r#"
        let flag = false
        const result = !flag
    "#;

    let output = compile_and_check(source).unwrap();
    assert!(
        output.contains("not flag"),
        "Should compile !variable to 'not variable'"
    );
}

#[test]
fn test_bang_with_function_call() {
    let source = r#"
        function getValue(): boolean {
            return true
        }
        const result = !getValue()
    "#;

    let output = compile_and_check(source).unwrap();
    assert!(output.contains("not"), "Should compile !function_call");
}

#[test]
fn test_double_bang() {
    let source = r#"
        const a = true
        const b = !!a
    "#;

    let output = compile_and_check(source).unwrap();
    assert!(
        output.contains("not not a"),
        "Should compile !! to 'not not'"
    );
}

#[test]
fn test_bang_with_and() {
    let source = r#"
        const a = true
        const b = false
        const result = !(a and b)
    "#;

    let output = compile_and_check(source).unwrap();
    assert!(output.contains("not"), "Should compile !(a and b)");
}

#[test]
fn test_bang_with_or() {
    let source = r#"
        const a = true
        const b = false
        const result = !(a or b)
    "#;

    let output = compile_and_check(source).unwrap();
    assert!(output.contains("not"), "Should compile !(a or b)");
}

#[test]
fn test_bang_preserves_semantics() {
    let source = r#"
        let x: boolean | nil = nil
        const result = !x
    "#;

    let output = compile_and_check(source).unwrap();
    assert!(
        output.contains("not"),
        "Should compile !x with nil handling"
    );
}

#[test]
fn test_bang_in_condition() {
    let source = r#"
        let values = [1, 2, 3, 4, 5]
        let filtered: number[] = []
        for v in values {
            if !(v > 3) {
                filtered.push(v)
            }
        }
    "#;

    let output = compile_and_check(source).unwrap();
    assert!(output.contains("not"), "Should compile !(v > 3)");
}

#[test]
fn test_bang_with_table_access() {
    let source = r#"
        const obj = { flag: false }
        const result = !obj.flag
    "#;

    let output = compile_and_check(source).unwrap();
    assert!(output.contains("not"), "Should compile !obj.flag");
}

#[test]
fn test_bang_nested() {
    let source = r#"
        const a = true
        const b = false
        const result = !(!a and !b)
    "#;

    let output = compile_and_check(source).unwrap();
    assert!(output.contains("not not"), "Should compile !(!a and !b)");
}

#[test]
fn test_bang_chained_comparison() {
    let source = r#"
        const a = 5
        const result = !(a > 3 and a < 10)
    "#;

    let output = compile_and_check(source).unwrap();
    assert!(output.contains("not"), "Should compile !(a > 3 and a < 10)");
}

#[test]
fn test_bang_with_nil_coalescing() {
    let source = r#"
        let x: string | nil = nil
        const result = !(x ?? "default")
    "#;

    let output = compile_and_check(source).unwrap();
    assert!(output.contains("not"), "Should compile !(x ?? \"default\")");
}

#[test]
fn test_bang_type_narrowing() {
    let source = r#"
        let value: number | boolean = 42
        if !(typeof(value) == "number") {
            const b: boolean = value
        } else {
            const n: number = value
        }
    "#;

    let output = compile_and_check(source).unwrap();
    assert!(
        output.contains("not"),
        "Should compile bang in type narrowing context"
    );
}

#[test]
fn test_bang_in_return() {
    let source = r#"
        function negate(b: boolean): boolean {
            return !b
        }
        const result = negate(true)
    "#;

    let output = compile_and_check(source).unwrap();
    assert!(output.contains("not"), "Should compile !b in return");
}

#[test]
fn test_bang_complex_expression() {
    let source = r#"
        const a = 1
        const b = 2
        const c = 3
        const result = !((a + b) > c)
    "#;

    let output = compile_and_check(source).unwrap();
    assert!(output.contains("not"), "Should compile !((a + b) > c)");
}

#[test]
fn test_bang_in_array_comprehension() {
    let source = r#"
        const numbers = [1, 2, 3, 4, 5]
        const even = numbers.filter(n => !(n % 2 == 0))
    "#;

    let output = compile_and_check(source).unwrap();
    assert!(
        output.contains("not"),
        "Should compile ! in array comprehension"
    );
}
