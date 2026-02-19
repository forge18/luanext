//! Execution tests for `continue` statement across Lua targets.
//!
//! The `continue` statement is emulated differently per target:
//! - Lua 5.2-5.4: `goto __continue` + `::__continue::` label before loop `end`
//! - Lua 5.1: `repeat...until true` wrapping with `break` acting as continue
//!
//! All execution tests use default target (Lua 5.4) since mlua runs Lua 5.4.
//! Output-only tests verify codegen patterns for other targets.

use luanext_test_helpers::compile::{compile, compile_with_stdlib, compile_with_target};
use luanext_test_helpers::{LuaExecutor, LuaTarget};

// ============================================================================
// While loop continue
// ============================================================================

#[test]
fn test_continue_in_while_loop() {
    // Skip even numbers, sum only odds from 1 to 10
    let source = r#"
        sum: number = 0
        i: number = 0
        while i < 10 do
            i = i + 1
            if i % 2 == 0 then
                continue
            end
            sum = sum + i
        end
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let sum: i64 = executor.execute_and_get(&lua_code, "sum").unwrap();
    assert_eq!(sum, 25); // 1 + 3 + 5 + 7 + 9
}

// ============================================================================
// Numeric for loop continue
// ============================================================================

#[test]
fn test_continue_in_numeric_for() {
    // Skip iteration 3 in for i = 1, 5
    let source = r#"
        sum: number = 0
        for i = 1, 5 do
            if i == 3 then
                continue
            end
            sum = sum + i
        end
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let sum: i64 = executor.execute_and_get(&lua_code, "sum").unwrap();
    assert_eq!(sum, 12); // 1 + 2 + 4 + 5
}

// ============================================================================
// Generic for loop continue
// ============================================================================

#[test]
fn test_continue_in_generic_for() {
    // Skip elements greater than 3
    let source = r#"
        const items: number[] = [1, 2, 5, 3, 4]
        sum: number = 0
        for _, v in ipairs(items) do
            if v > 3 then
                continue
            end
            sum = sum + v
        end
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let sum: i64 = executor.execute_and_get(&lua_code, "sum").unwrap();
    assert_eq!(sum, 6); // 1 + 2 + 3
}

// ============================================================================
// Repeat-until loop continue
// ============================================================================

#[test]
fn test_continue_in_repeat_until() {
    // Sum odds, skip evens, stop at 10
    let source = r#"
        sum: number = 0
        i: number = 0
        repeat
            i = i + 1
            if i % 2 == 0 then
                continue
            end
            sum = sum + i
        until i >= 10
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let sum: i64 = executor.execute_and_get(&lua_code, "sum").unwrap();
    assert_eq!(sum, 25); // 1 + 3 + 5 + 7 + 9
}

// ============================================================================
// Nested loops - continue only affects inner
// ============================================================================

#[test]
fn test_continue_in_nested_loops() {
    // Continue in inner loop should not affect outer loop
    let source = r#"
        count: number = 0
        for i = 1, 3 do
            for j = 1, 4 do
                if j == 2 then
                    continue
                end
                count = count + 1
            end
        end
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let count: i64 = executor.execute_and_get(&lua_code, "count").unwrap();
    assert_eq!(count, 9); // 3 outer * 3 inner (j=1,3,4; j=2 skipped)
}

// ============================================================================
// Conditional continue
// ============================================================================

#[test]
fn test_continue_with_if_condition() {
    // Multiple conditions that trigger continue
    let source = r#"
        result: string = ""
        for i = 1, 6 do
            if i == 2 then
                continue
            end
            if i == 4 then
                continue
            end
            result = result .. tostring(i)
        end
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "1356");
}

// ============================================================================
// Output verification - Lua 5.4 (goto + label)
// ============================================================================

#[test]
fn test_continue_output_contains_goto_label() {
    let source = r#"
        for i = 1, 10 do
            if i == 5 then
                continue
            end
        end
    "#;

    let lua_code = compile(source).unwrap();
    assert!(
        lua_code.contains("goto __continue"),
        "Should emit goto __continue, got:\n{}",
        lua_code
    );
    assert!(
        lua_code.contains("::__continue::"),
        "Should emit ::__continue:: label, got:\n{}",
        lua_code
    );
    // Verify no bare "continue" keyword (only "goto __continue" should appear)
    for line in lua_code.lines() {
        let trimmed = line.trim();
        assert!(
            trimmed != "continue",
            "Should NOT emit bare 'continue' keyword, got line: {:?}",
            line
        );
    }
}

#[test]
fn test_no_continue_label_when_not_needed() {
    // Loop without continue should not have ::__continue::
    let source = r#"
        sum: number = 0
        for i = 1, 5 do
            sum = sum + i
        end
    "#;

    let lua_code = compile(source).unwrap();
    assert!(
        !lua_code.contains("::__continue::"),
        "Should not emit __continue label when no continue is used"
    );
}

// ============================================================================
// Output verification - Lua 5.1 (repeat...until true)
// ============================================================================

#[test]
fn test_continue_lua51_repeat_until_pattern() {
    let source = r#"
        sum: number = 0
        for i = 1, 10 do
            if i == 5 then
                continue
            end
            sum = sum + i
        end
    "#;

    let lua_code = compile_with_target(source, LuaTarget::Lua51).unwrap();
    assert!(
        lua_code.contains("repeat"),
        "Lua 5.1 should use repeat...until true wrapping, got:\n{}",
        lua_code
    );
    assert!(
        lua_code.contains("until true"),
        "Lua 5.1 should use repeat...until true wrapping, got:\n{}",
        lua_code
    );
    // The continue statement should compile to break (exits inner repeat)
    assert!(
        lua_code.contains("break"),
        "Lua 5.1 continue should emit 'break', got:\n{}",
        lua_code
    );
    // Should NOT have goto (Lua 5.1 has no goto)
    assert!(
        !lua_code.contains("goto"),
        "Lua 5.1 should NOT emit goto, got:\n{}",
        lua_code
    );
}

#[test]
fn test_continue_lua51_with_break_errors() {
    // Lua 5.1: continue + break in same loop should emit error
    let source = r#"
        sum: number = 0
        for i = 1, 10 do
            if i == 3 then
                continue
            end
            if i == 7 then
                break
            end
            sum = sum + i
        end
    "#;

    let lua_code = compile_with_target(source, LuaTarget::Lua51).unwrap();
    assert!(
        lua_code.contains("error("),
        "Lua 5.1 should emit error() when continue and break coexist, got:\n{}",
        lua_code
    );
}
