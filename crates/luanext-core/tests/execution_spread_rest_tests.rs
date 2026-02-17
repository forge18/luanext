//! Execution tests for spread (...) and rest patterns in arrays and objects.

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

#[test]
fn test_array_spread_in_literal() {
    // [1, ...arr, 4] should contain all elements in order
    let source = r#"
        const mid = [2, 3]
        const arr = [1, ...mid, 4]
        r1: number = arr[1]
        r2: number = arr[2]
        r3: number = arr[3]
        r4: number = arr[4]
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let r1: i64 = executor.execute_and_get(&lua_code, "r1").unwrap();
    let r2: i64 = executor.execute_and_get(&lua_code, "r2").unwrap();
    let r3: i64 = executor.execute_and_get(&lua_code, "r3").unwrap();
    let r4: i64 = executor.execute_and_get(&lua_code, "r4").unwrap();
    assert_eq!(r1, 1);
    assert_eq!(r2, 2);
    assert_eq!(r3, 3);
    assert_eq!(r4, 4);
}

#[test]
fn test_object_spread_in_literal() {
    // {a = 1, ...base, c = 3} merges fields
    let source = r#"
        const base = {b = 2}
        const merged = {a = 1, ...base, c = 3}
        ra: number = merged.a
        rb: number = merged.b
        rc: number = merged.c
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let ra: i64 = executor.execute_and_get(&lua_code, "ra").unwrap();
    let rb: i64 = executor.execute_and_get(&lua_code, "rb").unwrap();
    let rc: i64 = executor.execute_and_get(&lua_code, "rc").unwrap();
    assert_eq!(ra, 1);
    assert_eq!(rb, 2);
    assert_eq!(rc, 3);
}

#[test]
fn test_destructuring_rest_array() {
    // [first, ...rest] = arr - rest gets remaining elements
    let source = r#"
        const arr = [10, 20, 30, 40]
        const [head, ...tail] = arr
        r_head: number = head
        r_tail_1: number = tail[1]
        r_tail_2: number = tail[2]
        r_tail_3: number = tail[3]
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let r_head: i64 = executor.execute_and_get(&lua_code, "r_head").unwrap();
    let r_tail_1: i64 = executor.execute_and_get(&lua_code, "r_tail_1").unwrap();
    let r_tail_2: i64 = executor.execute_and_get(&lua_code, "r_tail_2").unwrap();
    let r_tail_3: i64 = executor.execute_and_get(&lua_code, "r_tail_3").unwrap();
    assert_eq!(r_head, 10);
    assert_eq!(r_tail_1, 20);
    assert_eq!(r_tail_2, 30);
    assert_eq!(r_tail_3, 40);
}

#[test]
fn test_spread_empty_source() {
    // Spreading an empty array produces no elements
    let source = r#"
        const empty: number[] = []
        const arr = [1, ...empty, 2]
        r1: number = arr[1]
        r2: number = arr[2]
        len: number = #arr
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let r1: i64 = executor.execute_and_get(&lua_code, "r1").unwrap();
    let r2: i64 = executor.execute_and_get(&lua_code, "r2").unwrap();
    let len: i64 = executor.execute_and_get(&lua_code, "len").unwrap();
    assert_eq!(r1, 1);
    assert_eq!(r2, 2);
    assert_eq!(len, 2);
}

#[test]
fn test_spread_multiple_sources() {
    // [...a, ...b] combines two arrays
    let source = r#"
        const a = [1, 2]
        const b = [3, 4]
        const combined = [...a, ...b]
        r1: number = combined[1]
        r2: number = combined[2]
        r3: number = combined[3]
        r4: number = combined[4]
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let r1: i64 = executor.execute_and_get(&lua_code, "r1").unwrap();
    let r2: i64 = executor.execute_and_get(&lua_code, "r2").unwrap();
    let r3: i64 = executor.execute_and_get(&lua_code, "r3").unwrap();
    let r4: i64 = executor.execute_and_get(&lua_code, "r4").unwrap();
    assert_eq!(r1, 1);
    assert_eq!(r2, 2);
    assert_eq!(r3, 3);
    assert_eq!(r4, 4);
}

#[test]
fn test_object_spread_override() {
    // Later key in spread wins over earlier value
    let source = r#"
        const defaults = {x = 1, y = 2}
        const overrides = {x = 99, ...defaults}
        rx: number = overrides.x
        ry: number = overrides.y
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    // When spread comes after, defaults.x = 1 overwrites the explicit x = 99
    let rx: i64 = executor.execute_and_get(&lua_code, "rx").unwrap();
    let ry: i64 = executor.execute_and_get(&lua_code, "ry").unwrap();
    assert_eq!(rx, 1); // spread overwrites
    assert_eq!(ry, 2);
}

#[test]
fn test_spread_preserves_order() {
    // Spread elements appear in source order
    let source = r#"
        const nums = [5, 6, 7]
        const result = [1, 2, ...nums, 8, 9]
        r3: number = result[3]
        r4: number = result[4]
        r5: number = result[5]
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let r3: i64 = executor.execute_and_get(&lua_code, "r3").unwrap();
    let r4: i64 = executor.execute_and_get(&lua_code, "r4").unwrap();
    let r5: i64 = executor.execute_and_get(&lua_code, "r5").unwrap();
    assert_eq!(r3, 5);
    assert_eq!(r4, 6);
    assert_eq!(r5, 7);
}

#[test]
fn test_rest_after_first_element() {
    // [head, ...tail] - captures single head and rest as array
    let source = r#"
        const src = [100, 200, 300]
        const [first, ...rest] = src
        r_first: number = first
        r_rest_len: number = #rest
        r_rest_1: number = rest[1]
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let r_first: i64 = executor.execute_and_get(&lua_code, "r_first").unwrap();
    let r_rest_len: i64 = executor.execute_and_get(&lua_code, "r_rest_len").unwrap();
    let r_rest_1: i64 = executor.execute_and_get(&lua_code, "r_rest_1").unwrap();
    assert_eq!(r_first, 100);
    assert_eq!(r_rest_len, 2);
    assert_eq!(r_rest_1, 200);
}
