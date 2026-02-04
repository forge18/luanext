use typedlua_core::di::DiContainer;

fn compile_and_check(source: &str) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile(source)
}

#[test]
fn test_simple_null_coalesce() {
    let source = r#"
        const value: number | nil = nil
        const result = value ?? 42
    "#;

    let result = compile_and_check(source);
    match &result {
        Ok(output) => {
            println!("Success! Generated code:\n{}", output);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    assert!(result.is_ok(), "Null coalescing should compile");
}

#[test]
fn test_null_coalesce_with_variable() {
    let source = r#"
        let opt: string | nil = nil
        const result = opt ?? "default"
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Null coalescing with variable should compile"
    );
}

#[test]
fn test_null_coalesce_nested() {
    let source = r#"
        const a: number | nil = nil
        const b: number | nil = nil
        const c: number | nil = nil
        const result = a ?? b ?? c ?? 0
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Nested null coalescing should compile");
}

#[test]
fn test_null_coalesce_in_expression() {
    let source = r#"
        const a: number | nil = nil
        const result = (a ?? 10) * 2
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Null coalescing in expression should compile"
    );
}

#[test]
fn test_null_coalesce_function_call() {
    let source = r#"
        function get(): number | nil {
            return nil
        }
        const result = get() ?? 42
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Null coalescing with function call should compile"
    );
}

#[test]
fn test_null_coalesce_method_call() {
    let source = r#"
        class Container {
            public value: number | nil = nil
        }

        const c = new Container()
        const result = c.value ?? 0
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Null coalescing with method should compile");
}

#[test]
fn test_null_coalesce_type_narrowing() {
    let source = r#"
        const x: number | nil = nil
        const y = x ?? 0
        const z: number = y
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Null coalescing should narrow type");
}

#[test]
fn test_null_coalesce_table_access() {
    let source = r#"
        const obj = { a: nil, b: 2 }
        const result = obj.a ?? obj.b
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Null coalescing with table access should compile"
    );
}

#[test]
fn test_null_coalesce_array_access() {
    let source = r#"
        const arr: number[] = []
        const result = arr[0] ?? -1
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Null coalescing with array access should compile"
    );
}

#[test]
fn test_null_coalesce_complex_right() {
    let source = r#"
        const getDefault = () => 100
        const x: number | nil = nil
        const result = x ?? getDefault()
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Null coalescing with function call RHS should compile"
    );
}

#[test]
fn test_null_coalesce_left_associative() {
    let source = r#"
        const a: number | nil = nil
        const b: number | nil = nil
        const c: number | nil = nil
        const result = a ?? b ?? c ?? 1
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Left-associative null coalescing should compile"
    );
}

#[test]
fn test_null_coalesce_with_and_or() {
    let source = r#"
        const a: number | nil = nil
        const b: number | nil = nil
        const result = (a ?? 1) + (b ?? 2)
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Null coalescing with arithmetic should compile"
    );
}

#[test]
fn test_null_coalesce_object_literal() {
    let source = r#"
        const x: number | nil = nil
        const result = { value: x ?? 0 }
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Null coalescing in object literal should compile"
    );
}

#[test]
fn test_null_coalesce_array_literal() {
    let source = r#"
        const x: number | nil = nil
        const result = [x ?? 0, 1, 2]
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Null coalescing in array literal should compile"
    );
}

#[test]
fn test_null_coalesce_ternary() {
    let source = r#"
        const x: number | nil = nil
        const result = x ?? 0 > 10 ? "big" : "small"
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Null coalescing with ternary should compile"
    );
}

#[test]
fn test_null_coalesce_function_param() {
    let source = r#"
        function f(x: number | nil): number {
            return x ?? 0
        }
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Null coalescing in function param should compile"
    );
}

#[test]
fn test_null_coalesce_return() {
    let source = r#"
        function f(x: number | nil): number {
            return x ?? 42
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Null coalescing in return should compile");
}

#[test]
fn test_null_coalesce_generic() {
    let source = r#"
        function f<T>(x: T | nil, default: T): T {
            return x ?? default
        }
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Null coalescing with generics should compile"
    );
}

#[test]
fn test_null_coalesce_union_right() {
    let source = r#"
        const x: number | nil = nil
        const result: number | string = x ?? "default"
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Null coalescing with union RHS should compile"
    );
}

#[test]
fn test_null_coalesce_chained_with_other_operators() {
    let source = r#"
        const a: number | nil = nil
        const b: number | nil = nil
        const result = a ?? 1 + b ?? 2
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Null coalescing with other operators should compile"
    );
}
