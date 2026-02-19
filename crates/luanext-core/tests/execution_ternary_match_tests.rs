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

// --- Match Expression Edge Cases ---

#[test]
fn test_match_non_exhaustive_error() {
    // Missing wildcard arm throws runtime error, caught by try/catch
    let source = r#"
        function try_match(x: number): string {
            return match x {
                1 => "one",
                2 => "two"
            }
        }
        ok: boolean = true
        try {
            try_match(999)
        } catch (e) {
            ok = false
        }
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let ok: bool = executor.execute_and_get(&lua_code, "ok").unwrap();
    assert!(!ok, "non-exhaustive match should throw runtime error");
}

#[test]
fn test_match_identifier_binding() {
    // Identifier pattern binds the match value to a local variable
    let source = r#"
        const x: number = 42
        result: number = match x {
            n => n * 2
        }
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 84);
}

#[test]
fn test_match_multiple_guards_fallthrough() {
    // Multiple guarded arms — first guard fails, second matches
    let source = r#"
        const x: number = 15
        result: string = match x {
            n when n > 100 => "huge",
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
fn test_match_in_function_call_argument() {
    // Match expression used as a function argument
    let source = r#"
        function double(x: number): number {
            return x * 2
        }
        const val: number = 3
        result: number = double(match val {
            1 => 10,
            2 => 20,
            3 => 30,
            _ => 0
        })
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 60);
}

#[test]
fn test_match_first_arm_wins() {
    // When multiple arms could match, the first matching arm is selected
    let source = r#"
        const x: number = 5
        result: string = match x {
            n when n > 0 => "positive",
            n when n > 3 => "also matches but second",
            _ => "default"
        }
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "positive");
}

#[test]
fn test_match_with_expression_bodies() {
    // Match arms with complex expression bodies (arithmetic)
    let source = r#"
        const x: number = 3
        result: number = match x {
            1 => 10 + 1,
            2 => 20 + 2,
            3 => 30 + 3,
            _ => 0
        }
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 33);
}

#[test]
fn test_match_guard_uses_bound_variable() {
    // Guard expression references the identifier bound by the pattern
    let source = r#"
        const x: number = 7
        result: string = match x {
            n when n % 2 == 0 => "even",
            n when n % 2 == 1 => "odd",
            _ => "unknown"
        }
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "odd");
}

#[test]
fn test_match_compound_guard() {
    // Compound guard with `and` — previously failed because And returned Unknown
    let source = r#"
        const x: number = 15
        result: string = match x {
            n when n > 10 and n < 20 => "teen",
            _ => "other"
        }
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "teen");
}

#[test]
fn test_match_compound_guard_or() {
    // Compound guard with `or`
    let source = r#"
        const x: number = 5
        result: string = match x {
            n when n == 1 or n == 5 => "special",
            _ => "normal"
        }
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "special");
}
