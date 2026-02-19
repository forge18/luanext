//! Runtime error behavior tests.
//!
//! These tests verify that LuaNext-generated Lua code behaves correctly when
//! encountering runtime errors. Tests cover:
//!   - Nil field access errors
//!   - Calling nil as a function
//!   - Division by zero behavior (Lua returns inf, not an error)
//!   - Error propagation through try/catch
//!   - String/number coercions (Lua is permissive about these)
//!   - Explicit `throw` producing catchable errors

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

#[test]
fn test_nil_field_access_causes_runtime_error() {
    // Accessing a field on nil causes a Lua runtime error.
    // Use raw Lua to test this runtime behavior directly (LuaNext's `any` type
    // does not permit nil assignment at the type-checker level).
    let lua_code = r#"
        local status, err = pcall(function()
            local x = nil
            local val = x.field
        end)
        ok = not status
    "#
    .to_string();
    let executor = LuaExecutor::new().unwrap();
    let ok: bool = executor.execute_and_get(&lua_code, "ok").unwrap();
    assert!(ok, "nil field access should cause a Lua runtime error");
}

#[test]
fn test_calling_nil_as_function_causes_runtime_error() {
    let source = r#"
        ok: boolean = false
        try {
            local f: any = nil
            f()
        } catch (e) {
            ok = true
        }
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let ok: bool = executor.execute_and_get(&lua_code, "ok").unwrap();
    assert!(
        ok,
        "calling nil should cause a runtime error caught by catch block"
    );
}

#[test]
fn test_explicit_throw_produces_catchable_error() {
    let source = r#"
        caught: boolean = false
        try {
            throw "intentional error"
        } catch (e) {
            caught = true
        }
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let caught: bool = executor.execute_and_get(&lua_code, "caught").unwrap();
    assert!(caught, "throw should produce an error caught by catch");
}

#[test]
fn test_error_message_is_accessible_in_catch() {
    let source = r#"
        msg: string = ""
        try {
            throw "specific error message"
        } catch (e) {
            msg = e
        }
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let msg: String = executor.execute_and_get(&lua_code, "msg").unwrap();
    assert!(
        msg.contains("specific error message"),
        "catch variable should contain the thrown message, got: {msg}"
    );
}

#[test]
fn test_float_division_by_zero_returns_infinity() {
    // In Lua, float division by zero returns inf (not an error)
    let source = r#"
        result: number = 1.0 / 0.0
        is_inf: boolean = result > 1e308
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let is_inf: bool = executor.execute_and_get(&lua_code, "is_inf").unwrap();
    assert!(is_inf, "1.0/0.0 should produce infinity in Lua");
}

#[test]
fn test_error_propagates_from_called_function() {
    let source = r#"
        function do_fail(): void {
            throw "error in function"
        }
        caught: boolean = false
        try {
            do_fail()
        } catch (e) {
            caught = true
        }
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let caught: bool = executor.execute_and_get(&lua_code, "caught").unwrap();
    assert!(
        caught,
        "error thrown in called function should propagate to outer catch"
    );
}

#[test]
fn test_nested_try_catch_inner_handles_error() {
    let source = r#"
        inner_caught: boolean = false
        outer_caught: boolean = false
        try {
            try {
                throw "inner error"
            } catch (e) {
                inner_caught = true
            }
        } catch (e) {
            outer_caught = true
        }
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let inner: bool = executor.execute_and_get(&lua_code, "inner_caught").unwrap();
    let outer: bool = executor.execute_and_get(&lua_code, "outer_caught").unwrap();
    assert!(inner, "inner catch should handle the inner error");
    assert!(
        !outer,
        "outer catch should NOT be triggered when inner handles it"
    );
}

#[test]
fn test_string_to_number_coercion_in_arithmetic() {
    // Lua silently coerces string numbers to numeric values in arithmetic.
    // LuaNext generates Lua that inherits this behavior.
    // Use a global (not local) so execute_and_get can retrieve it.
    let lua_code = r#"
        local s = "42"
        result = s + 0
    "#
    .to_string();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42, "Lua silently coerces '42' to 42 in arithmetic");
}

#[test]
fn test_uncaught_error_causes_execution_failure() {
    // An uncaught error (no try/catch) should cause execution to fail
    let source = r#"
        throw "uncaught error"
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let ok = executor.execute_ok(&lua_code);
    assert!(!ok, "uncaught throw should cause execution failure");
}

#[test]
fn test_array_out_of_bounds_returns_nil_not_error() {
    // In Lua, accessing an out-of-bounds array index returns nil (not an error)
    let source = r#"
        arr: number[] = [1, 2, 3]
        result: any = arr[10]
        is_nil: boolean = result == nil
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let is_nil: bool = executor.execute_and_get(&lua_code, "is_nil").unwrap();
    assert!(
        is_nil,
        "out-of-bounds array access should return nil in Lua"
    );
}
