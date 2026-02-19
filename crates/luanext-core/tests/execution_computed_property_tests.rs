//! Execution tests for computed property names in object/table literals and destructuring.
//!
//! Computed properties use the `[expr] = value` syntax to dynamically compute the key.
//! Parser: ObjectProperty::Computed { key, value }, codegen: `[expr] = value` in tables.
//! Note: The type checker doesn't track computed keys in the static type, so we use
//! index signatures or bracket access for reading computed properties.

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

#[test]
fn test_computed_key_string_variable() {
    // { [key] = value } where key is a string variable, read via bracket access
    let source = r#"
        const key = "name"
        const obj: {[k: string]: number} = { [key] = 42 }
        result: number = obj["name"]
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_computed_key_string_literal() {
    // { ["hello"] = value } with a literal string key
    let source = r#"
        const obj: {[k: string]: number} = { ["hello"] = 99 }
        result: number = obj["hello"]
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 99);
}

#[test]
fn test_computed_key_number_expression() {
    // { [1 + 1] = "two" } with a numeric expression key
    let source = r#"
        const arr: {[key: number]: string} = { [1 + 1] = "two" }
        result: string = arr[2]
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "two");
}

#[test]
fn test_computed_key_with_spread() {
    // { ...base, [dynamic] = value } merges base then adds computed key
    // Use index signature type so the computed key is accessible
    let source = r#"
        const base: {[k: string]: number} = { a = 1, b = 2 }
        const key = "c"
        const obj = { ...base, [key] = 3 }
        ra: number = obj["a"]
        rb: number = obj["b"]
        rc: number = obj["c"]
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
fn test_computed_key_overrides_spread() {
    // Computed key after spread should override existing value
    let source = r#"
        const base = { x = 10, y = 20 }
        const key = "x"
        const obj = { ...base, [key] = 99 }
        rx: number = obj.x
        ry: number = obj.y
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let rx: i64 = executor.execute_and_get(&lua_code, "rx").unwrap();
    let ry: i64 = executor.execute_and_get(&lua_code, "ry").unwrap();
    assert_eq!(rx, 99);
    assert_eq!(ry, 20);
}

#[test]
fn test_multiple_computed_keys() {
    // Multiple computed properties in the same object
    let source = r#"
        const k1 = "alpha"
        const k2 = "beta"
        const obj: {[k: string]: number} = { [k1] = 1, [k2] = 2 }
        r1: number = obj["alpha"]
        r2: number = obj["beta"]
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let r1: i64 = executor.execute_and_get(&lua_code, "r1").unwrap();
    let r2: i64 = executor.execute_and_get(&lua_code, "r2").unwrap();
    assert_eq!(r1, 1);
    assert_eq!(r2, 2);
}

#[test]
fn test_computed_key_with_function_call() {
    // Using a function call result as the computed key
    let source = r#"
        function getKey(): string {
            return "result"
        }
        const obj: {[k: string]: number} = { [getKey()] = 42 }
        result: number = obj["result"]
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_computed_key_mixed_with_regular() {
    // Mix of regular and computed properties — use spread with index signature
    // to ensure all properties are accessible through the type system
    let source = r#"
        const key = "dynamic"
        const base: {[k: string]: number} = {}
        const obj = { ...base, static_prop = 1, [key] = 2, another = 3 }
        r1: number = obj["static_prop"]
        r2: number = obj["dynamic"]
        r3: number = obj["another"]
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let r1: i64 = executor.execute_and_get(&lua_code, "r1").unwrap();
    let r2: i64 = executor.execute_and_get(&lua_code, "r2").unwrap();
    let r3: i64 = executor.execute_and_get(&lua_code, "r3").unwrap();
    assert_eq!(r1, 1);
    assert_eq!(r2, 2);
    assert_eq!(r3, 3);
}

#[test]
fn test_computed_key_destructuring() {
    // Destructuring with a computed key: const { [key]: val } = obj
    let source = r#"
        const key = "x"
        const obj = { x = 42, y = 99 }
        const { [key]: val } = obj
        result: number = val
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_computed_key_codegen_output() {
    // Verify the generated Lua uses bracket notation for computed keys
    let source = r#"
        const key = "x"
        const obj: {[k: string]: number} = { [key] = 42 }
    "#;

    let lua_code = compile(source).unwrap();
    // Computed keys should use [key] = value syntax in the Lua output
    assert!(
        lua_code.contains("[key]"),
        "Expected computed key bracket notation in output, got:\n{lua_code}"
    );
}
