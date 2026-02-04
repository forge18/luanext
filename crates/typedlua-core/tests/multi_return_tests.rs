use typedlua_core::di::DiContainer;

fn type_check(source: &str) -> Result<(), String> {
    let mut container = DiContainer::test_default();
    container.compile(source)?;
    Ok(())
}

#[test]
fn test_single_return_value() {
    let source = r#"
        function get_number(): number {
            return 42
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Single return value should type-check"
    );
}

#[test]
fn test_tuple_return_type() {
    let source = r#"
        function get_coords(): [number, number] {
            return 10, 20
        }
    "#;

    assert!(type_check(source).is_ok(), "Tuple return should type-check");
}

#[test]
fn test_multi_return_all_checked() {
    let source = r#"
        function get_values(): [number, number, number] {
            return 1, 2, 3
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Multi-return should type-check all values"
    );
}

#[test]
fn test_multi_return_with_different_types() {
    let source = r#"
        function mixed(): [string, number, boolean] {
            return "text", 42, true
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Mixed type multi-return should type-check"
    );
}

#[test]
fn test_multi_return_assignment() {
    let source = r#"
        function get_point(): [number, number] {
            return 10, 20
        }

        const [x, y] = get_point()
    "#;

    assert!(
        type_check(source).is_ok(),
        "Multi-return assignment should type-check"
    );
}

#[test]
fn test_multi_return_partial_assignment() {
    let source = r#"
        function get_triple(): [number, number, number] {
            return 1, 2, 3
        }

        const [first, , third] = get_triple()
    "#;

    assert!(
        type_check(source).is_ok(),
        "Partial multi-return should type-check"
    );
}

#[test]
fn test_multi_return_rest_assignment() {
    let source = r#"
        function get_many(): [number, number, number, number, number] {
            return 1, 2, 3, 4, 5
        }

        const [first, ...rest] = get_many()
    "#;

    assert!(
        type_check(source).is_ok(),
        "Rest in multi-return should type-check"
    );
}

#[test]
fn test_multi_return_function_param() {
    let source = r#"
        function process_pair(p: [number, number]): number {
            const [a, b] = p
            return a + b
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Multi-return as param should type-check"
    );
}

#[test]
fn test_multi_return_variadic() {
    let source = r#"
        function sum(...nums: number[]): number {
            let s = 0
            for n in nums {
                s = s + n
            }
            return s
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Variadic function should type-check"
    );
}

#[test]
fn test_multi_return_with_nil() {
    let source = r#"
        function maybe(): [number] | nil {
            return nil
        }

        const result: number | nil = maybe()[0]
    "#;

    assert!(
        type_check(source).is_ok(),
        "Multi-return with nil should type-check"
    );
}

#[test]
fn test_chained_multi_return() {
    let source = r#"
        function f(): [number, number] {
            return 1, 2
        }

        function g(): [number, number, number] {
            const [a, b] = f()
            return a, b, a + b
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Chained multi-return should type-check"
    );
}

#[test]
fn test_multi_return_in_match() {
    let source = r#"
        function get(): [number, string] {
            return 42, "answer"
        }

        const result = match get() {
            [n, s] => s .. tostring(n)
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Multi-return in match should type-check"
    );
}

#[test]
fn test_multi_return_annotated() {
    let source = r#"
        function get(): [a: number, b: string] {
            return 1, "test"
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Annotated multi-return should type-check"
    );
}

#[test]
fn test_async_multi_return() {
    let source = r#"
        async function fetch(): [string, string] {
            return "data", "success"
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Async multi-return should type-check");
}

#[test]
fn test_multi_return_generic() {
    let source = r#"
        function pair<T>(a: T, b: T): [T, T] {
            return a, b
        }

        const [n, m] = pair(1, 2)
        const [s1, s2] = pair("a", "b")
    "#;

    assert!(
        type_check(source).is_ok(),
        "Generic multi-return should type-check"
    );
}

#[test]
fn test_multi_return_never() {
    let source = r#"
        function fail(): never {
            error("failed")
        }
    "#;

    assert!(type_check(source).is_ok(), "Never return should type-check");
}

#[test]
fn test_multi_return_void() {
    let source = r#"
        function no_return(): void {
            return
        }
    "#;

    assert!(type_check(source).is_ok(), "Void return should type-check");
}
