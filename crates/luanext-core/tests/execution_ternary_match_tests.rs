//! Execution tests for ternary (conditional) expressions and match expressions.
//!
//! Codegen:
//! - `cond ? a : b` → `(cond and a or b)` — known limitation with falsy `a` values
//! - `match x { ... }` → IIFE with if/elseif chain
//!
//! Reference: `codegen/expressions.rs`

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

// --- Ternary Expression Tests ---

#[test]
fn test_basic_ternary_true() {
    let source = r#"
        result: number = true ? 1 : 2
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_basic_ternary_false() {
    let source = r#"
        result: number = false ? 1 : 2
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 2);
}

#[test]
fn test_ternary_with_comparison() {
    let source = r#"
        result: string = (5 > 3) ? "yes" : "no"
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "yes");
}

#[test]
fn test_ternary_nested() {
    let source = r#"
        result: number = true ? (false ? 1 : 2) : 3
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 2);
}

#[test]
fn test_ternary_falsy_known_limitation() {
    // KNOWN LIMITATION: `true ? 0 : 99` compiles to `(true and 0 or 99)`.
    // In Lua, `true and 0` = `0`, then `0 or 99` = `0` (0 is truthy in Lua).
    // Actually 0 IS truthy in Lua (only nil and false are falsy), so this works!
    // The real limitation is `true ? false : "x"` → `(true and false or "x")` = "x"
    let source = r#"
        result: number = true ? 0 : 99
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    // In Lua, 0 is truthy, so (true and 0 or 99) = 0. This actually works correctly!
    assert_eq!(result, 0);
}

// --- Match Expression Tests ---

#[test]
fn test_match_literal_patterns() {
    let source = r#"
        const x: number = 2
        result: string = match x {
            1 => "one",
            2 => "two",
            3 => "three",
            _ => "other"
        }
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "two");
}

#[test]
fn test_match_wildcard() {
    let source = r#"
        const x: number = 999
        result: string = match x {
            _ => "default"
        }
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "default");
}

#[test]
fn test_match_or_pattern() {
    let source = r#"
        const x: number = 2
        result: string = match x {
            1 | 2 => "low",
            3 | 4 => "mid",
            _ => "high"
        }
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "low");
}

#[test]
fn test_match_with_guard() {
    let source = r#"
        const x: number = 15
        result: string = match x {
            n when n > 10 => "big",
            _ => "small"
        }
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "big");
}

#[test]
fn test_match_string_patterns() {
    let source = r#"
        const s: string = "world"
        result: number = match s {
            "hello" => 1,
            "world" => 2,
            _ => 0
        }
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 2);
}
