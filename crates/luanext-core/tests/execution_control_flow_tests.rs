//! Execution tests for advanced control flow - numeric for with step,
//! generic for with ipairs/pairs, for-in destructuring, repeat-until,
//! nested loops with break, and complex loop conditions.
//!
//! Note: `continue` is skipped because default codegen emits Lua 5.5
//! `continue` keyword, but mlua runs Lua 5.4 which doesn't support it.

use luanext_test_helpers::compile::{compile, compile_with_stdlib};
use luanext_test_helpers::LuaExecutor;

// ============================================================================
// Numeric For Loops
// ============================================================================

#[test]
fn test_numeric_for_with_step() {
    // for i = 1, 10, 2 do (step of 2 -> 1, 3, 5, 7, 9)
    let source = r#"
        sum: number = 0
        count: number = 0
        for i = 1, 10, 2 do
            sum = sum + i
            count = count + 1
        end
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let sum: i64 = executor.execute_and_get(&lua_code, "sum").unwrap();
    let count: i64 = executor.execute_and_get(&lua_code, "count").unwrap();
    assert_eq!(sum, 25); // 1 + 3 + 5 + 7 + 9
    assert_eq!(count, 5);
}

#[test]
fn test_numeric_for_countdown() {
    // for i = 5, 1, -1 do (negative step, counting down)
    let source = r#"
        result: string = ""
        for i = 5, 1, -1 do
            result = result .. tostring(i)
        end
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "54321");
}

// ============================================================================
// Generic For Loops
// ============================================================================

#[test]
fn test_generic_for_ipairs() {
    // for i, v in ipairs(arr) do
    let source = r#"
        local arr = {10, 20, 30, 40}
        sum: number = 0
        index_sum: number = 0
        for i, v in ipairs(arr) do
            sum = sum + v
            index_sum = index_sum + i
        end
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let sum: i64 = executor.execute_and_get(&lua_code, "sum").unwrap();
    let index_sum: i64 = executor.execute_and_get(&lua_code, "index_sum").unwrap();
    assert_eq!(sum, 100); // 10 + 20 + 30 + 40
    assert_eq!(index_sum, 10); // 1 + 2 + 3 + 4
}

#[test]
fn test_generic_for_pairs() {
    // for k, v in pairs(table) do - count entries
    let source = r#"
        local t = {a = 1, b = 2, c = 3}
        count: number = 0
        total: number = 0
        for k, v in pairs(t) do
            count = count + 1
            total = total + v
        end
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let count: i64 = executor.execute_and_get(&lua_code, "count").unwrap();
    let total: i64 = executor.execute_and_get(&lua_code, "total").unwrap();
    assert_eq!(count, 3);
    assert_eq!(total, 6); // 1 + 2 + 3
}

// ============================================================================
// For-In Destructuring
// ============================================================================

#[test]
fn test_for_in_array_destructuring() {
    // for [a, b] in items do - destructure array elements
    let source = r#"
        local items: number[][] = {{1, 10}, {2, 20}, {3, 30}}
        sum_first: number = 0
        sum_second: number = 0
        for [a, b] in items do
            sum_first = sum_first + a
            sum_second = sum_second + b
        end
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let sum_first: i64 = executor.execute_and_get(&lua_code, "sum_first").unwrap();
    let sum_second: i64 = executor.execute_and_get(&lua_code, "sum_second").unwrap();
    assert_eq!(sum_first, 6); // 1 + 2 + 3
    assert_eq!(sum_second, 60); // 10 + 20 + 30
}

#[test]
fn test_for_in_object_destructuring() {
    // for {name, value} in items do - destructure object fields
    let source = r#"
        local points: {name: string, value: number}[] = {
            {name = "a", value = 10},
            {name = "b", value = 20},
            {name = "c", value = 30}
        }
        total: number = 0
        names: string = ""
        for {name, value} in points do
            total = total + value
            names = names .. name
        end
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let total: i64 = executor.execute_and_get(&lua_code, "total").unwrap();
    let names: String = executor.execute_and_get(&lua_code, "names").unwrap();
    assert_eq!(total, 60); // 10 + 20 + 30
    assert_eq!(names, "abc");
}

// ============================================================================
// Repeat-Until
// ============================================================================

#[test]
fn test_repeat_until_loop() {
    let source = r#"
        count: number = 0
        sum: number = 0
        repeat
            count = count + 1
            sum = sum + count
        until count >= 5
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let count: i64 = executor.execute_and_get(&lua_code, "count").unwrap();
    let sum: i64 = executor.execute_and_get(&lua_code, "sum").unwrap();
    assert_eq!(count, 5);
    assert_eq!(sum, 15); // 1 + 2 + 3 + 4 + 5
}

// ============================================================================
// Nested Loops and Break
// ============================================================================

#[test]
fn test_nested_for_loops_with_break() {
    // Break only exits the inner loop
    let source = r#"
        total: number = 0
        outer_count: number = 0
        for i = 1, 3 do
            outer_count = outer_count + 1
            for j = 1, 10 do
                if j > 3 then
                    break
                end
                total = total + j
            end
        end
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let total: i64 = executor.execute_and_get(&lua_code, "total").unwrap();
    let outer_count: i64 = executor.execute_and_get(&lua_code, "outer_count").unwrap();
    assert_eq!(total, 18); // 3 outer iterations * (1+2+3) = 18
    assert_eq!(outer_count, 3); // All outer iterations complete
}

#[test]
fn test_while_with_complex_condition() {
    // While loop with and/or conditions
    let source = r#"
        x: number = 0
        y: number = 100
        steps: number = 0
        while x < 10 and y > 0 do
            x = x + 1
            y = y - 15
            steps = steps + 1
        end
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let steps: i64 = executor.execute_and_get(&lua_code, "steps").unwrap();
    let x: i64 = executor.execute_and_get(&lua_code, "x").unwrap();
    let y: i64 = executor.execute_and_get(&lua_code, "y").unwrap();
    assert_eq!(steps, 7); // y goes: 85, 70, 55, 40, 25, 10, -5 (stops at -5)
    assert_eq!(x, 7);
    assert_eq!(y, -5);
}

#[test]
fn test_nested_loops_accumulator() {
    // Nested loops building a multiplication table sum
    let source = r#"
        total: number = 0
        for i = 1, 4 do
            for j = 1, 4 do
                total = total + i * j
            end
        end
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let total: i64 = executor.execute_and_get(&lua_code, "total").unwrap();
    // Sum of i*j for i=1..4, j=1..4 = (1+2+3+4)*(1+2+3+4) = 10*10 = 100
    assert_eq!(total, 100);
}
