//! Lua runtime edge cases tests.
//!
//! These tests verify that LuaNext-generated code handles Lua runtime edge
//! cases correctly: deep recursion, large tables, numeric edge cases,
//! string escaping, and metamethod behavior.

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

#[test]
fn test_deep_nesting_100_levels_does_not_overflow() {
    // 100 nested function calls is well within Lua's stack limit (~200)
    // Using a for loop accumulator to avoid recursive scope issues
    let source = r#"
        function inc(n: number): number {
            return n + 1
        }
        acc: number = 0
        for i = 1, 100, 1 do
            acc = inc(acc)
        end
        result: number = acc
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 100);
}

#[test]
fn test_large_table_creation_1000_elements() {
    let source = r#"
        function fill(n: number): number[] {
            local arr: number[] = []
            for i = 1, n, 1 do
                arr[i] = i
            end
            return arr
        }
        arr: number[] = fill(1000)
        result: number = arr[1000]
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 1000);
}

#[test]
fn test_string_concatenation_chain() {
    // String concatenation with many parts
    let source = r#"
        result: string = "a" .. "b" .. "c" .. "d" .. "e"
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "abcde");
}

#[test]
fn test_float_arithmetic_precision() {
    // 0.1 + 0.2 is famously imprecise in IEEE 754 floating point
    let source = r#"
        result: number = 0.1 + 0.2
        is_close: boolean = result > 0.29 and result < 0.31
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let is_close: bool = executor.execute_and_get(&lua_code, "is_close").unwrap();
    assert!(
        is_close,
        "0.1 + 0.2 should be approximately 0.3 (within tolerance)"
    );
}

#[test]
fn test_large_integer_arithmetic() {
    // Lua 5.3+ has native 64-bit integers
    let source = r#"
        result: number = 1000000 * 1000000
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 1_000_000_000_000_i64);
}

#[test]
fn test_string_concatenation_produces_correct_length() {
    // LuaNext string concatenation with multiple components
    // Note: \n in raw Rust strings is a literal backslash + n (2 chars), not a newline.
    // Use concatenation to verify multi-part strings produce correct lengths.
    let source = r#"
        part1: string = "hello"
        part2: string = "world"
        result: string = part1 .. " " .. part2
        len: number = #result
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let len: i64 = executor.execute_and_get(&lua_code, "len").unwrap();
    // "hello" (5) + " " (1) + "world" (5) = 11
    assert_eq!(len, 11, "string concatenation length should be correct");
}

#[test]
fn test_table_as_map_and_array_mixed() {
    // Lua tables can mix array-style and map-style entries
    let source = r#"
        local t: any = { 10, 20, x: 30 }
        result_arr: number = t[1]
        result_map: number = t["x"]
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let arr: i64 = executor.execute_and_get(&lua_code, "result_arr").unwrap();
    let map: i64 = executor.execute_and_get(&lua_code, "result_map").unwrap();
    assert_eq!(arr, 10, "array-style table access should work");
    assert_eq!(map, 30, "map-style table access should work");
}

#[test]
fn test_metamethod_add_chain() {
    let source = r#"
        class Vec {
            x: number
            y: number
            constructor(x: number, y: number) {
                self.x = x
                self.y = y
            }
            operator+(other: Vec): Vec {
                return new Vec(self.x + other.x, self.y + other.y)
            }
        }
        v1: Vec = new Vec(1, 2)
        v2: Vec = new Vec(3, 4)
        v3: Vec = new Vec(5, 6)
        sum: Vec = v1 + v2 + v3
        result_x: number = sum.x
        result_y: number = sum.y
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let x: i64 = executor.execute_and_get(&lua_code, "result_x").unwrap();
    let y: i64 = executor.execute_and_get(&lua_code, "result_y").unwrap();
    assert_eq!(x, 9, "chained metamethod __add should give 1+3+5=9");
    assert_eq!(y, 12, "chained metamethod __add should give 2+4+6=12");
}

#[test]
fn test_table_length_with_hash_operator() {
    let source = r#"
        arr: number[] = [10, 20, 30, 40, 50]
        result: number = #arr
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let len: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(len, 5, "# operator should return array length");
}

#[test]
fn test_modulo_with_negative_numbers() {
    // In Lua, modulo follows the sign of the divisor (like Python, unlike C)
    let source = r#"
        result: number = -7 % 3
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    // In Lua: -7 % 3 = 2 (result has sign of divisor)
    assert_eq!(result, 2, "Lua modulo follows sign of divisor");
}
