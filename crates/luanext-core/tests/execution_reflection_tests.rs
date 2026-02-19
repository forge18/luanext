//! Execution tests for assertType intrinsic and runtime type validation.
//!
//! Codegen:
//! - `assertType<string>(x)` → runtime IIFE that checks `type(x)` and errors on mismatch
//! - `assertType<number>(x)` → runtime check for `type(x) == "number"`
//! - `assertType<boolean>(x)` → runtime check for `type(x) == "boolean"`
//! - `assertType<string | number>(x)` → union check
//! - `assertType<string?>(x)` → nullable check (nil is valid)
//! - `assertType<MyClass>(x)` → metatable check via `getmetatable(x) == MyClass`
//!
//! Reference: `codegen/expressions.rs` lines 933-1310

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

// --- assertType Success Cases ---

#[test]
fn test_assert_type_string_pass() {
    let source = r#"
        const x: string = "hello"
        result: string = assertType<string>(x)
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_assert_type_number_pass() {
    let source = r#"
        const x: number = 42
        result: number = assertType<number>(x)
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_assert_type_boolean_pass() {
    let source = r#"
        const x: boolean = true
        result: boolean = assertType<boolean>(x)
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: bool = executor.execute_and_get(&lua_code, "result").unwrap();
    assert!(result);
}

// --- assertType Failure Cases ---

#[test]
fn test_assert_type_string_fail() {
    // assertType<string>(42) should throw a runtime error
    let source = r#"
        caught: boolean = false
        try {
            const x: number = 42
            assertType<string>(x)
        } catch (e) {
            caught = true
        }
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let caught: bool = executor.execute_and_get(&lua_code, "caught").unwrap();
    assert!(caught, "assertType<string>(42) should throw an error");
}

#[test]
fn test_assert_type_number_fail() {
    // assertType<number>("hello") should throw a runtime error
    let source = r#"
        caught: boolean = false
        try {
            const x: string = "hello"
            assertType<number>(x)
        } catch (e) {
            caught = true
        }
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let caught: bool = executor.execute_and_get(&lua_code, "caught").unwrap();
    assert!(caught, "assertType<number>('hello') should throw an error");
}

// --- assertType with Class ---

#[test]
fn test_assert_type_class_instance() {
    let source = r#"
        class MyClass {
            value: number
            constructor(v: number) {
                self.value = v
            }
        }
        const obj = new MyClass(10)
        result: number = assertType<MyClass>(obj).value
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 10);
}

// --- assertType with Union Types ---

#[test]
fn test_assert_type_union_string() {
    let source = r#"
        const x: string = "hello"
        result: string = assertType<string | number>(x)
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_assert_type_union_number() {
    let source = r#"
        const x: number = 42
        result: number = assertType<string | number>(x)
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

// --- assertType with Nullable ---

#[test]
fn test_assert_type_nullable_with_value() {
    let source = r#"
        const x: string = "hello"
        result: string = assertType<string?>(x)
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_assert_type_nullable_nil() {
    // nil is valid for a nullable type
    let source = r#"
        function get_nil(): string | nil {
            return nil
        }
        caught: boolean = false
        try {
            assertType<string?>(get_nil())
        } catch (e) {
            caught = true
        }
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let caught: bool = executor.execute_and_get(&lua_code, "caught").unwrap();
    assert!(
        !caught,
        "assertType<string?>(nil) should NOT throw an error"
    );
}

// --- assertType Returns Value ---

#[test]
fn test_assert_type_returns_value() {
    // assertType should return the value it checked
    let source = r#"
        result: number = assertType<number>(100) + 1
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 101);
}

#[test]
fn test_assert_type_error_message() {
    // The error message should mention the expected type
    let source = r#"
        msg: string = ""
        try {
            const x: number = 42
            assertType<string>(x)
        } catch (e) {
            msg = e
        }
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let msg: String = executor.execute_and_get(&lua_code, "msg").unwrap();
    assert!(
        msg.contains("string"),
        "Error should mention expected type 'string', got: {}",
        msg
    );
}
