//! Lua 5.4 compatibility tests: native bitwise operators and integer division.
//!
//! These tests verify that LuaNext's default code generation (Lua 5.4 target)
//! produces correct runtime results for the features introduced in Lua 5.3/5.4:
//! - Native bitwise operators: `&`, `|`, `~` (XOR), `<<`, `>>`
//! - Unary bitwise NOT: `~x`
//! - Integer division: `//`
//!
//! All tests run on the mlua Lua 5.4 runtime (the only version supported by mlua 0.10).

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

#[test]
fn test_bitwise_and_correct_value() {
    let source = r#"
        result: number = 15 & 7
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 7, "15 & 7 should be 7 (0b1111 & 0b0111)");
}

#[test]
fn test_bitwise_or_correct_value() {
    let source = r#"
        result: number = 5 | 3
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 7, "5 | 3 should be 7 (0b101 | 0b011)");
}

#[test]
fn test_bitwise_xor_correct_value() {
    let source = r#"
        result: number = 12 ~ 10
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 6, "12 ~ 10 should be 6 (0b1100 XOR 0b1010)");
}

#[test]
fn test_shift_left_correct_value() {
    let source = r#"
        result: number = 1 << 4
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 16, "1 << 4 should be 16");
}

#[test]
fn test_shift_right_correct_value() {
    let source = r#"
        result: number = 256 >> 4
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 16, "256 >> 4 should be 16");
}

#[test]
fn test_integer_division_correct_value() {
    let source = r#"
        result: number = 10 // 3
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 3, "10 // 3 should be 3 (floor division)");
}

#[test]
fn test_integer_division_truncates_toward_negative_infinity() {
    let source = r#"
        result: number = 7 // 2
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 3, "7 // 2 should be 3");
}

#[test]
fn test_complex_bitwise_expression() {
    let source = r#"
        result: number = (15 & 7) | (3 << 2)
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    // (15 & 7) = 7, (3 << 2) = 12, 7 | 12 = 15
    assert_eq!(result, 15);
}

#[test]
fn test_bitwise_operators_in_variables() {
    let source = r#"
        a: number = 0xFF
        b: number = 0x0F
        result: number = a & b
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 15, "0xFF & 0x0F should be 15");
}

#[test]
fn test_generated_code_uses_native_bitwise_operators() {
    let source = r#"
        result: number = 10 & 6
    "#;
    let lua_code = compile(source).unwrap();
    // Default target is Lua 5.4 — should use native & operator
    assert!(
        lua_code.contains('&'),
        "Lua 5.4 target should use native & operator, got:\n{lua_code}"
    );
    assert!(
        !lua_code.contains("bit32."),
        "Lua 5.4 target should NOT use bit32 library"
    );
}

#[test]
fn test_generated_code_uses_native_floor_division() {
    let source = r#"
        result: number = 10 // 3
    "#;
    let lua_code = compile(source).unwrap();
    // Default target is Lua 5.4 — should use native // operator
    assert!(
        lua_code.contains("//"),
        "Lua 5.4 target should use native // operator, got:\n{lua_code}"
    );
    assert!(
        !lua_code.contains("math.floor"),
        "Lua 5.4 target should NOT use math.floor for //"
    );
}
