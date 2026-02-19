//! Execution tests for optional chaining as assignment target.
//!
//! `obj?.x = value` generates `if obj ~= nil then obj.x = value end`.
//! This is a silent no-op when the object is nil, matching TypeScript semantics.

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

#[test]
fn test_optional_member_assign_non_nil() {
    // obj?.x = 42 when obj is non-nil should assign
    let source = r#"
        const obj: {x: number} = { x = 0 }
        obj?.x = 42
        result: number = obj.x
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_optional_member_assign_nil_noop() {
    // obj?.x = 42 when obj is nil should be a no-op (no error)
    let source = r#"
        const obj: {x: number} | nil = nil
        obj?.x = 42
        result: string = "ok"
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_optional_index_assign_non_nil() {
    // obj?.[key] = value when obj is non-nil should assign
    let source = r#"
        const obj: {[k: string]: number} = { x = 0 }
        const key = "x"
        obj?.[key] = 99
        result: number = obj["x"]
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 99);
}

#[test]
fn test_optional_index_assign_nil_noop() {
    // obj?.[key] = value when obj is nil should be a no-op
    let source = r#"
        const obj: {[k: string]: number} | nil = nil
        obj?.["x"] = 99
        result: string = "ok"
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_optional_assign_compound_add() {
    // obj?.x += 1 when obj is non-nil should increment
    let source = r#"
        const obj: {x: number} = { x = 10 }
        obj?.x += 5
        result: number = obj.x
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 15);
}

#[test]
fn test_optional_assign_codegen_output_simple() {
    // Verify the generated Lua uses nil check for simple identifiers
    let source = r#"
        const obj: {x: number} | nil = nil
        obj?.x = 42
    "#;

    let lua_code = compile(source).unwrap();
    assert!(
        lua_code.contains("~= nil"),
        "Should contain nil check, got:\n{lua_code}"
    );
    assert!(
        !lua_code.contains("(function()"),
        "Simple identifier should not use IIFE, got:\n{lua_code}"
    );
}

#[test]
fn test_optional_assign_codegen_output_complex() {
    // Complex expressions should use temp var to avoid double evaluation
    let source = r#"
        function getObj(): {x: number} | nil {
            return { x = 0 }
        }
        getObj()?.x = 42
    "#;

    let lua_code = compile(source).unwrap();
    assert!(
        lua_code.contains("__t"),
        "Complex expression should use temp var, got:\n{lua_code}"
    );
}

#[test]
fn test_optional_member_assign_preserves_other_fields() {
    // Optional assignment should only affect the targeted field
    let source = r#"
        const obj: {x: number, y: number} = { x = 1, y = 2 }
        obj?.x = 99
        rx: number = obj.x
        ry: number = obj.y
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let rx: i64 = executor.execute_and_get(&lua_code, "rx").unwrap();
    let ry: i64 = executor.execute_and_get(&lua_code, "ry").unwrap();
    assert_eq!(rx, 99);
    assert_eq!(ry, 2);
}
