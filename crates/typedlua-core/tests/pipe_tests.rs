use typedlua_core::di::DiContainer;

fn compile_and_check(source: &str) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile(source)
}

#[test]
fn test_simple_pipe() {
    let source = r#"
        const double = (x: number): number => x * 2
        const value = 5
        const result = value |> double
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
    assert!(result.is_ok(), "Simple pipe should compile");
}

#[test]
fn test_pipe_with_method() {
    let source = r#"
        class StringUtils {
            public static trim(s: string): string {
                return s
            }

            public static uppercase(s: string): string {
                return s
            }
        }

        const result = " hello " |> StringUtils.trim |> StringUtils.uppercase
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Pipe with method should compile");
}

#[test]
fn test_pipe_chain() {
    let source = r#"
        const double = (x: number): number => x * 2
        const addOne = (x: number): number => x + 1
        const value = 5
        const result = value |> double |> addOne |> double
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Pipe chain should compile");
}

#[test]
fn test_pipe_with_expression() {
    let source = r#"
        const add = (a: number, b: number): number => a + b
        const result = 1 |> add(?, 2)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Pipe with expression should compile");
}

#[test]
fn test_pipe_multiple_arguments() {
    let source = r#"
        const sum = (a: number, b: number, c: number): number => a + b + c
        const result = 1 |> sum(2, ?, 3)
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Pipe with multiple arguments should compile"
    );
}

#[test]
fn test_pipe_into_function_call() {
    let source = r#"
        const arr = [1, 2, 3]
        const result = arr |> table.concat(",")
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Pipe into function call should compile");
}

#[test]
fn test_pipe_into_method_call() {
    let source = r#"
        const s = "hello world"
        const result = s |> string.upper |> string.sub(?, 1, 5)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Pipe into method call should compile");
}

#[test]
fn test_pipe_preserves_types() {
    let source = r#"
        const toString = (x: number): string => tostring(x)
        const len = (s: string): number => #s
        const n = 42 |> toString |> len
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Pipe should preserve types");
}

#[test]
fn test_pipe_complex_chain() {
    let source = r#"
        const double = (x: number): number => x * 2
        const add = (a: number, b: number): number => a + b
        const triple = (x: number): number => x * 3

        const result = 1 |> add(?, 2) |> double |> triple |> add(?, 4)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Complex pipe chain should compile");
}

#[test]
fn test_pipe_with_table() {
    let source = r#"
        const result = { a: 1, b: 2 } |> table.unpack
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Pipe with table should compile");
}

#[test]
fn test_pipe_returning_multiple() {
    let source = r#"
        function getPair(): [number, number] {
            return 1, 2
        }

        const [a, b] = nil |> getPair
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Pipe returning multiple should compile");
}

#[test]
fn test_pipe_right_associative() {
    let source = r#"
        const f = (x: number): number => x + 1
        const g = (x: number): number => x * 2
        const result = 5 |> f |> g
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Pipe should be left-associative");
}

#[test]
fn test_pipe_with_nil_coalescing() {
    let source = r#"
        const opt: number | nil = nil
        const result = opt ?? 10 |> double
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Pipe with nil coalescing should compile");
}

#[test]
fn test_pipe_into_arrow_function() {
    let source = r#"
        const transform = (f: (number) => number, x: number): number => f(x)
        const result = 5 |> transform(?, double)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Pipe into arrow function should compile");
}

#[test]
fn test_pipe_generic_function() {
    let source = r#"
        const identity = <T>(x: T): T => x
        const result = 42 |> identity |> identity
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Pipe with generic function should compile");
}

#[test]
fn test_pipe_with_callback() {
    let source = r#"
        const arr = [1, 2, 3]
        const result = arr |> table.concat(?, ",")
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Pipe with callback should compile");
}

#[test]
fn test_pipe_self_parameter() {
    let source = r#"
        class Math {
            public static add(a: number, b: number): number {
                return a + b
            }
        }

        const result = 5 |> Math.add(?, 3)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Pipe with self parameter should compile");
}

#[test]
fn test_pipe_chained_methods() {
    let source = r#"
        class Builder {
            public value: number = 0

            public add(n: number): Builder {
                self.value = self.value + n
                return self
            }

            public multiply(n: number): Builder {
                self.value = self.value * n
                return self
            }
        }

        const result = new Builder() |> .add(2) |> .multiply(3)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Pipe with chained methods should compile");
}

#[test]
fn test_pipe_composition_style() {
    let source = r#"
        const double = (x: number): number => x * 2
        const square = (x: number): number => x * x
        const addOne = (x: number): number => x + 1

        const composed = addOne << square << double
        const result = composed(3)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Pipe composition should compile");
}
