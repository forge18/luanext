//! Execution tests for type aliases - verify they are fully erased at runtime
//! and produce no observable runtime impact.

use luanext_test_helpers::compile::{compile, compile_with_stdlib};
use luanext_test_helpers::LuaExecutor;

#[test]
fn test_type_alias_no_runtime_impact() {
    let source = r#"
        type Str = string
        x: Str = "hello"
        result: string = x .. " world"
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let x: String = executor.execute_and_get(&lua_code, "x").unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();

    assert_eq!(x, "hello");
    assert_eq!(result, "hello world");
}

#[test]
fn test_type_alias_in_function_sig() {
    let source = r#"
        type Num = number

        function double(x: Num): Num {
            return x * 2
        }

        result: number = double(21)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_type_alias_union() {
    let source = r#"
        type NumOrStr = number | string

        function describe(x: NumOrStr): string {
            if type(x) == "number" then
                return "number"
            else
                return "string"
            end
        }

        r1: string = describe(42)
        r2: string = describe("hello")
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let r1: String = executor.execute_and_get(&lua_code, "r1").unwrap();
    let r2: String = executor.execute_and_get(&lua_code, "r2").unwrap();

    assert_eq!(r1, "number");
    assert_eq!(r2, "string");
}

#[test]
fn test_type_alias_generic_erased() {
    // Generic type aliases are fully erased - just use the underlying table at runtime
    let source = r#"
        type Box<T> = { value: T }

        box: Box<number> = { value = 42 }
        result: number = box.value
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_type_alias_complex() {
    // Complex structural type alias - erased, just a plain table at runtime
    let source = r#"
        type Point = { x: number, y: number }

        function distance(p: Point): number {
            return p.x * p.x + p.y * p.y
        }

        p: Point = { x = 3, y = 4 }
        result: number = distance(p)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 25);
}
