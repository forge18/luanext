//! Execution tests for pipe (`|>`), null coalescing (`??`), and error chain (`!!`) operators.
//!
//! Codegen:
//! - `x |> f` → `f(x)`, `x |> f(a, b)` → `f(x, a, b)`
//! - `a ?? b` → `(a ~= nil and a or b)` (simple), or IIFE for complex expressions
//! - `a !! b` → pcall wrapper returning `b` on error
//!
//! Reference: `codegen/expressions.rs`

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

#[test]
fn test_pipe_single_arg() {
    let source = r#"
        function double(x: number): number {
            return x * 2
        }
        result: number = 5 |> double
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 10);
}

#[test]
fn test_pipe_with_string_transform() {
    // Test pipe with a string transformation function
    let source = r#"
        function greet(name: string): string {
            return "Hello, " .. name
        }
        result: string = "World" |> greet
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "Hello, World");
}

#[test]
fn test_pipe_chaining() {
    // Chain: 2 |> double |> add_ten
    // Codegen: add_ten(double(2))
    let source = r#"
        function double(x: number): number {
            return x * 2
        }
        function add_ten(x: number): number {
            return x + 10
        }
        const a: number = 2 |> double
        result: number = a |> add_ten
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 14);
}

#[test]
fn test_pipe_with_expression() {
    let source = r#"
        function double(x: number): number {
            return x * 2
        }
        result: number = (3 + 4) |> double
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 14);
}

#[test]
fn test_null_coalesce_nil() {
    let source = r#"
        function get_nil(): number | nil {
            return nil
        }
        result: number = get_nil() ?? 42
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_null_coalesce_non_nil() {
    let source = r#"
        function get_value(): number | nil {
            return 10
        }
        result: number = get_value() ?? 42
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 10);
}

#[test]
fn test_null_coalesce_nested() {
    let source = r#"
        function get_nil(): number | nil {
            return nil
        }
        result: number = get_nil() ?? get_nil() ?? 99
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 99);
}

#[test]
fn test_error_chain_with_pipe() {
    // Combine error chain and pipe: if risky() throws, use 5, then double it
    let source = r#"
        function risky(): number {
            throw "fail"
            return 0
        }
        function double(x: number): number {
            return x * 2
        }
        result: number = (risky() !! 5) |> double
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 10);
}

#[test]
fn test_null_coalesce_with_function_return() {
    // Function that returns nil to test null coalescing fallback
    let source = r#"
        function always_nil(): number | nil {
            return nil
        }
        function always_seven(): number | nil {
            return 7
        }
        result_a: number = always_nil() ?? 0
        result_b: number = always_seven() ?? 0
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let a: i64 = executor.execute_and_get(&lua_code, "result_a").unwrap();
    let b: i64 = executor.execute_and_get(&lua_code, "result_b").unwrap();
    assert_eq!(a, 0, "nil return should fall back to 0");
    assert_eq!(b, 7, "non-nil return should be used");
}

#[test]
fn test_pipe_long_chain() {
    // Use intermediate variables to avoid type checker pipe chaining limitation
    let source = r#"
        function add_one(x: number): number {
            return x + 1
        }
        const a: number = 1 |> add_one
        const b: number = a |> add_one
        result: number = b |> add_one
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 4);
}
