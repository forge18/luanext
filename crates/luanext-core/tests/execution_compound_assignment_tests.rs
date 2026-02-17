//! Execution tests for compound assignment operators.
//!
//! Codegen: `x += 5` expands to `x = x + 5` (same pattern for all operators).
//!
//! Reference: `codegen/expressions/binary_ops.rs`

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

#[test]
fn test_add_assign() {
    let source = r#"
        x: number = 10
        x += 5
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "x").unwrap();
    assert_eq!(result, 15);
}

#[test]
fn test_subtract_assign() {
    let source = r#"
        x: number = 20
        x -= 8
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "x").unwrap();
    assert_eq!(result, 12);
}

#[test]
fn test_multiply_assign() {
    let source = r#"
        x: number = 5
        x *= 3
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "x").unwrap();
    assert_eq!(result, 15);
}

#[test]
fn test_divide_assign() {
    let source = r#"
        x: number = 20
        x /= 4
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: f64 = executor.execute_and_get(&lua_code, "x").unwrap();
    assert!((result - 5.0).abs() < f64::EPSILON);
}

#[test]
fn test_modulo_assign() {
    let source = r#"
        x: number = 17
        x %= 5
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "x").unwrap();
    assert_eq!(result, 2);
}

#[test]
fn test_power_assign() {
    let source = r#"
        x: number = 2
        x ^= 10
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: f64 = executor.execute_and_get(&lua_code, "x").unwrap();
    assert!((result - 1024.0).abs() < f64::EPSILON);
}

#[test]
fn test_floor_divide_assign() {
    let source = r#"
        x: number = 17
        x //= 5
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "x").unwrap();
    assert_eq!(result, 3);
}

#[test]
fn test_concatenate_assign() {
    let source = r#"
        s: string = "hello"
        s ..= " world"
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "s").unwrap();
    assert_eq!(result, "hello world");
}

#[test]
fn test_bitwise_and_assign() {
    let source = r#"
        x: number = 255
        x &= 15
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "x").unwrap();
    assert_eq!(result, 15);
}

#[test]
fn test_bitwise_or_assign() {
    let source = r#"
        x: number = 240
        x |= 15
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "x").unwrap();
    assert_eq!(result, 255);
}
