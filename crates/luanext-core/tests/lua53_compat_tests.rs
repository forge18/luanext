//! Lua 5.3 compatibility tests: native bitwise operators (same syntax as 5.4).
//!
//! Lua 5.3 introduced native bitwise operators (`&`, `|`, `~`, `<<`, `>>`) and
//! native integer division (`//`). Since Lua 5.4 is a superset of 5.3, these
//! features run correctly on the mlua 5.4 runtime.
//!
//! These tests verify:
//! 1. When targeting Lua 5.3, generated code uses native operators (same as 5.4)
//! 2. Generated code does NOT use `bit32.` (that's Lua 5.2's approach)
//! 3. Runtime values are correct

use luanext_test_helpers::compile::compile_with_target;
use luanext_test_helpers::{LuaExecutor, LuaTarget};

#[test]
fn test_lua53_bitwise_and_executes_correctly() {
    let source = r#"
        result: number = 15 & 7
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua53).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 7);
}

#[test]
fn test_lua53_bitwise_or_executes_correctly() {
    let source = r#"
        result: number = 5 | 3
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua53).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 7);
}

#[test]
fn test_lua53_shift_left_executes_correctly() {
    let source = r#"
        result: number = 1 << 3
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua53).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 8);
}

#[test]
fn test_lua53_shift_right_executes_correctly() {
    let source = r#"
        result: number = 64 >> 2
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua53).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 16);
}

#[test]
fn test_lua53_integer_division_executes_correctly() {
    let source = r#"
        result: number = 17 // 5
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua53).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 3, "17 // 5 == 3 (floor division)");
}

#[test]
fn test_lua53_uses_native_bitwise_operator_not_bit32() {
    let source = r#"
        result: number = 10 & 6
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua53).unwrap();
    // Lua 5.3 uses native & operator, not bit32.band()
    assert!(
        lua_code.contains('&'),
        "Lua 5.3 target should use native & operator"
    );
    assert!(
        !lua_code.contains("bit32."),
        "Lua 5.3 target should NOT use bit32 library"
    );
}

#[test]
fn test_lua53_uses_native_floor_division_not_math_floor() {
    let source = r#"
        result: number = 10 // 3
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua53).unwrap();
    assert!(
        lua_code.contains("//"),
        "Lua 5.3 target should use native // operator"
    );
    assert!(
        !lua_code.contains("math.floor"),
        "Lua 5.3 target should NOT use math.floor for //"
    );
}

#[test]
fn test_lua53_same_result_as_lua54() {
    // Lua 5.3 and 5.4 should produce identical runtime results for bitwise ops
    let source = r#"
        result: number = (0xFF & 0x0F) | (3 << 2)
    "#;
    use luanext_test_helpers::compile::compile;

    let lua53_code = compile_with_target(source, LuaTarget::Lua53).unwrap();
    let lua54_code = compile(source).unwrap();

    let executor = LuaExecutor::new().unwrap();
    let result53: i64 = executor.execute_and_get(&lua53_code, "result").unwrap();
    let result54: i64 = executor.execute_and_get(&lua54_code, "result").unwrap();
    assert_eq!(
        result53, result54,
        "Lua 5.3 and 5.4 targets should give same result"
    );
}
