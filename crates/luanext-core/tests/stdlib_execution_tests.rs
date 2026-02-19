//! Execution tests for Lua standard library integration.
//!
//! Verifies that LuaNext code correctly calls and interoperates with Lua's
//! built-in libraries: math, string, table, and global type/conversion functions.
//!
//! Uses `compile_with_stdlib` so the type checker has stdlib definitions available.
//! Lua builtins (math, string, table, type, tostring, tonumber) are always
//! available at runtime — no import statements needed.
//!
//! Reference: Lua 5.4 reference manual, §6 (standard libraries)

use luanext_test_helpers::compile::compile_with_stdlib;
use luanext_test_helpers::LuaExecutor;

// ============================================================================
// Math Library
// ============================================================================

#[test]
fn test_math_floor() {
    let source = r#"
        result: number = math.floor(3.7)
    "#;
    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 3);
}

#[test]
fn test_math_ceil() {
    let source = r#"
        result: number = math.ceil(3.2)
    "#;
    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 4);
}

#[test]
fn test_math_abs() {
    let source = r#"
        result: number = math.abs(-5)
    "#;
    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 5);
}

#[test]
fn test_math_max() {
    let source = r#"
        result: number = math.max(1, 5, 3)
    "#;
    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 5);
}

#[test]
fn test_math_min() {
    let source = r#"
        result: number = math.min(1, 5, 3)
    "#;
    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_math_sqrt() {
    let source = r#"
        result: number = math.sqrt(16.0)
    "#;
    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: f64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert!((result - 4.0).abs() < 1e-9, "Expected 4.0, got {}", result);
}

// ============================================================================
// String Library
// ============================================================================

#[test]
fn test_string_len() {
    // string.len returns the byte length of a string
    let source = r#"
        result: number = string.len("hello")
    "#;
    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 5);
}

#[test]
fn test_string_upper() {
    let source = r#"
        result: string = string.upper("hello world")
    "#;
    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "HELLO WORLD");
}

#[test]
fn test_string_lower() {
    let source = r#"
        result: string = string.lower("HELLO WORLD")
    "#;
    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "hello world");
}

#[test]
fn test_string_sub() {
    // Lua strings are 1-indexed; sub(1, 5) gives first 5 chars
    let source = r#"
        result: string = string.sub("hello world", 1, 5)
    "#;
    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "hello");
}

// ============================================================================
// Global Type/Conversion Functions
// ============================================================================

#[test]
fn test_tostring_number() {
    let source = r#"
        result: string = tostring(42)
    "#;
    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "42");
}

#[test]
fn test_tonumber_string() {
    let source = r#"
        result: number = tonumber("10")
    "#;
    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 10);
}
