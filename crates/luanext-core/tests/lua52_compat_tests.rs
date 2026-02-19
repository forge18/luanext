//! Lua 5.2 compatibility tests: bitwise via bit32 library.
//!
//! Lua 5.2 does not have native bitwise operators. Instead, it uses the `bit32`
//! library: `bit32.band()`, `bit32.bor()`, `bit32.bxor()`, `bit32.lshift()`,
//! `bit32.rshift()`, `bit32.bnot()`. Integer division uses `math.floor(x / y)`.
//!
//! The Lua 5.2 strategy now emits a bit32 polyfill preamble (pure Lua arithmetic),
//! making the generated code self-contained and executable on any Lua runtime.
//! On a real Lua 5.2 runtime, the polyfill harmlessly shadows the built-in bit32.

use luanext_test_helpers::compile::compile_with_target;
use luanext_test_helpers::{LuaExecutor, LuaTarget};

#[test]
fn test_lua52_bitwise_and_syntax_and_runtime() {
    let source = r#"
        result: number = 15 & 7
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua52).unwrap();
    assert!(
        lua_code.contains("bit32.band("),
        "Lua 5.2 target should use bit32.band(), got:\n{lua_code}"
    );
    assert!(
        !lua_code.contains(" & "),
        "Lua 5.2 target should NOT use native & operator"
    );

    // Runtime verification: 15 & 7 = 7
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 7);
}

#[test]
fn test_lua52_bitwise_or_syntax_and_runtime() {
    let source = r#"
        result: number = 5 | 3
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua52).unwrap();
    assert!(
        lua_code.contains("bit32.bor("),
        "Lua 5.2 target should use bit32.bor(), got:\n{lua_code}"
    );

    // Runtime verification: 5 | 3 = 7
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 7);
}

#[test]
fn test_lua52_bitwise_xor_syntax_and_runtime() {
    let source = r#"
        result: number = 12 ~ 10
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua52).unwrap();
    assert!(
        lua_code.contains("bit32.bxor("),
        "Lua 5.2 target should use bit32.bxor(), got:\n{lua_code}"
    );

    // Runtime verification: 12 ^ 10 = 6
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 6);
}

#[test]
fn test_lua52_shift_left_syntax_and_runtime() {
    let source = r#"
        result: number = 1 << 4
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua52).unwrap();
    assert!(
        lua_code.contains("bit32.lshift("),
        "Lua 5.2 target should use bit32.lshift(), got:\n{lua_code}"
    );
    assert!(
        !lua_code.contains(" << "),
        "Lua 5.2 target should NOT use native << operator"
    );

    // Runtime verification: 1 << 4 = 16
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 16);
}

#[test]
fn test_lua52_shift_right_syntax_and_runtime() {
    let source = r#"
        result: number = 256 >> 4
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua52).unwrap();
    assert!(
        lua_code.contains("bit32.rshift("),
        "Lua 5.2 target should use bit32.rshift(), got:\n{lua_code}"
    );

    // Runtime verification: 256 >> 4 = 16
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 16);
}

#[test]
fn test_lua52_integer_division_syntax_and_runtime() {
    let source = r#"
        result: number = 10 // 3
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua52).unwrap();
    assert!(
        lua_code.contains("math.floor("),
        "Lua 5.2 target should use math.floor() for //, got:\n{lua_code}"
    );
    assert!(
        !lua_code.contains("//"),
        "Lua 5.2 target should NOT use native // operator"
    );

    // Runtime verification: 10 // 3 = 3
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 3);
}

#[test]
fn test_lua52_preamble_contains_bit32_polyfill() {
    // Lua 5.2 now emits a bit32 polyfill preamble (not Lua 5.1 helpers)
    let source = r#"
        result: number = 15 & 7
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua52).unwrap();
    assert!(
        lua_code.contains("bit32 = {}"),
        "Lua 5.2 target should emit bit32 polyfill preamble, got:\n{lua_code}"
    );
    assert!(
        !lua_code.contains("_bit_band"),
        "Lua 5.2 target should NOT inject Lua 5.1 bitwise helper preamble"
    );
}

#[test]
fn test_lua52_complex_expression_runtime() {
    let source = r#"
        result: number = (15 & 7) | (3 << 2)
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua52).unwrap();
    assert!(lua_code.contains("bit32.band("), "Should translate &");
    assert!(lua_code.contains("bit32.bor("), "Should translate |");
    assert!(lua_code.contains("bit32.lshift("), "Should translate <<");

    // Runtime verification: (15 & 7) | (3 << 2) = 7 | 12 = 15
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 15);
}
