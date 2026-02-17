//! Execution tests for arrow functions - verify expression bodies, block bodies,
//! closures, higher-order usage, and other arrow function behaviors at runtime.

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

#[test]
fn test_arrow_expression_body() {
    let source = r#"
        const add = (x: number, y: number) => x + y
        result: number = add(3, 4)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 7);
}

#[test]
fn test_arrow_block_body() {
    let source = r#"
        const greet = (name: string) => {
            return "Hello, " .. name .. "!"
        }
        result: string = greet("World")
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "Hello, World!");
}

#[test]
fn test_arrow_no_params() {
    let source = r#"
        const get_answer = () => 42
        result: number = get_answer()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_arrow_captures_closure() {
    let source = r#"
        base: number = 10
        const add_base = (x: number) => x + base
        result: number = add_base(5)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 15);
}

#[test]
fn test_arrow_as_argument() {
    // Arrow passed as a callback to a higher-order function
    let source = r#"
        function apply(f: (number) => number, x: number): number {
            return f(x)
        }
        result: number = apply((x) => x * x, 5)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 25);
}

#[test]
fn test_arrow_returning_arrow() {
    // Curried function: (x) => (y) => x + y
    let source = r#"
        const add = (x: number) => (y: number) => x + y
        const add5 = add(5)
        result: number = add5(3)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 8);
}

#[test]
fn test_arrow_in_table() {
    let source = r#"
        const ops = {
            double = (x: number) => x * 2,
            triple = (x: number) => x * 3,
        }
        r1: number = ops.double(5)
        r2: number = ops.triple(4)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let r1: i64 = executor.execute_and_get(&lua_code, "r1").unwrap();
    let r2: i64 = executor.execute_and_get(&lua_code, "r2").unwrap();
    assert_eq!(r1, 10);
    assert_eq!(r2, 12);
}

#[test]
fn test_arrow_with_default_params() {
    let source = r#"
        function greet(name: string = "World"): string {
            return "Hello, " .. name
        }
        const greet_arrow = (n: string = "World") => "Hi, " .. n
        r1: string = greet("Alice")
        r2: string = greet()
        r3: string = greet_arrow("Bob")
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let r1: String = executor.execute_and_get(&lua_code, "r1").unwrap();
    let r2: String = executor.execute_and_get(&lua_code, "r2").unwrap();
    let r3: String = executor.execute_and_get(&lua_code, "r3").unwrap();
    assert_eq!(r1, "Hello, Alice");
    assert_eq!(r2, "Hello, World");
    assert_eq!(r3, "Hi, Bob");
}

#[test]
fn test_arrow_with_null_coalescing() {
    // Arrow with optional param and ?? operator
    let source = r#"
        const safe_double = (x: number | nil) => (x ?? 0) * 2
        r1: number = safe_double(5)
        r2: number = safe_double(nil)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let r1: i64 = executor.execute_and_get(&lua_code, "r1").unwrap();
    let r2: i64 = executor.execute_and_get(&lua_code, "r2").unwrap();
    assert_eq!(r1, 10);
    assert_eq!(r2, 0);
}

#[test]
fn test_arrow_immediately_invoked() {
    // Arrow function immediately called via intermediate variable
    let source = r#"
        const sum = (x: number, y: number) => x + y
        result: number = sum(10, 20)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 30);
}
