//! Lua 5.1 compatibility tests: bitwise via injected helper functions.
//!
//! Lua 5.1 has no native bitwise operators and no `bit32` library. LuaNext
//! handles this by injecting a preamble of pure-Lua helper functions:
//!   - `_bit_band(a, b)`, `_bit_bor(a, b)`, `_bit_bxor(a, b)`
//!   - `_bit_lshift(a, n)`, `_bit_rshift(a, n)`, `_bit_bnot(a)`
//!
//! These helper functions are valid plain Lua (no 5.1-specific syntax),
//! so they can be EXECUTED on the mlua Lua 5.4 runtime. This means both
//! output-syntax tests AND runtime value tests are possible here.
//!
//! Integer division uses `math.floor(a / b)` (available in all Lua versions).

use luanext_test_helpers::compile::compile_with_target;
use luanext_test_helpers::{LuaExecutor, LuaTarget};

#[test]
fn test_lua51_preamble_is_injected() {
    let source = r#"
        result: number = 15 & 7
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua51).unwrap();
    // The preamble defines _bit_band and other helpers
    assert!(
        lua_code.contains("_bit_band"),
        "Lua 5.1 target should inject bitwise helper preamble, got:\n{lua_code}"
    );
}

#[test]
fn test_lua51_bitwise_and_uses_helper_and_executes_correctly() {
    let source = r#"
        result: number = 15 & 7
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua51).unwrap();
    // Verify syntax
    assert!(
        lua_code.contains("_bit_band("),
        "Lua 5.1 target should use _bit_band()"
    );
    // Verify execution
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 7, "15 & 7 should be 7");
}

#[test]
fn test_lua51_bitwise_or_uses_helper_and_executes_correctly() {
    let source = r#"
        result: number = 5 | 3
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua51).unwrap();
    assert!(lua_code.contains("_bit_bor("));
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 7, "5 | 3 should be 7");
}

#[test]
fn test_lua51_bitwise_xor_uses_helper_and_executes_correctly() {
    let source = r#"
        result: number = 12 ~ 10
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua51).unwrap();
    assert!(lua_code.contains("_bit_bxor("));
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 6, "12 XOR 10 should be 6");
}

#[test]
fn test_lua51_shift_left_uses_helper_and_executes_correctly() {
    let source = r#"
        result: number = 1 << 3
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua51).unwrap();
    assert!(lua_code.contains("_bit_lshift("));
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 8, "1 << 3 should be 8");
}

#[test]
fn test_lua51_shift_right_uses_helper_and_executes_correctly() {
    let source = r#"
        result: number = 64 >> 2
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua51).unwrap();
    assert!(lua_code.contains("_bit_rshift("));
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 16, "64 >> 2 should be 16");
}

#[test]
fn test_lua51_integer_division_uses_math_floor_and_executes_correctly() {
    let source = r#"
        result: number = 10 // 3
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua51).unwrap();
    assert!(
        lua_code.contains("math.floor("),
        "Lua 5.1 target should use math.floor() for //"
    );
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 3, "10 // 3 should be 3");
}

#[test]
fn test_lua51_no_native_bitwise_operators_in_output() {
    let source = r#"
        result: number = 15 & 7
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua51).unwrap();
    // After the preamble definition, the expression should not use native & operator
    // The preamble itself may reference & in comments, but the actual expression should not
    assert!(
        !lua_code.contains("15 & 7"),
        "Lua 5.1 target should translate & to helper call"
    );
}

#[test]
fn test_lua51_complex_expression_executes_correctly() {
    let source = r#"
        result: number = (15 & 7) | (3 << 2)
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua51).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    // (15 & 7) = 7, (3 << 2) = 12, 7 | 12 = 15
    assert_eq!(result, 15);
}

#[test]
fn test_lua51_semantic_equivalence_with_lua54() {
    // Lua 5.1 target (with preamble) should produce same runtime results as Lua 5.4 target
    let source = r#"
        result: number = (0xFF & 0x0F) | (3 << 2)
    "#;
    use luanext_test_helpers::compile::compile;

    let lua51_code = compile_with_target(source, LuaTarget::Lua51).unwrap();
    let lua54_code = compile(source).unwrap();

    let executor = LuaExecutor::new().unwrap();
    let result51: i64 = executor.execute_and_get(&lua51_code, "result").unwrap();
    let result54: i64 = executor.execute_and_get(&lua54_code, "result").unwrap();
    assert_eq!(
        result51, result54,
        "Lua 5.1 preamble helpers should produce same result as native Lua 5.4 operators"
    );
}
