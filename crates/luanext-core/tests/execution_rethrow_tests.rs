//! Execution tests for rethrow, nested try/catch, and try expressions.
//!
//! Codegen:
//! - `rethrow` → `error(__error)` (re-raises the caught error)
//! - `try { ... } catch(e) { ... }` → pcall/xpcall wrapper
//! - `try expr catch fallback` → pcall IIFE returning fallback on error
//!
//! Reference: `codegen/statements.rs`

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

#[test]
fn test_rethrow_basic() {
    // Rethrow propagates error from inner catch to outer catch
    let source = r#"
        caught: string = ""
        try {
            try {
                throw "inner error"
            } catch (e) {
                rethrow
            }
        } catch (outer_e) {
            caught = outer_e
        }
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let caught: String = executor.execute_and_get(&lua_code, "caught").unwrap();
    assert!(
        caught.contains("inner error"),
        "Outer catch should receive rethrown error, got: {}",
        caught
    );
}

#[test]
fn test_rethrow_preserves_message() {
    // The rethrown error has the same message as the original
    let source = r#"
        original: string = "specific error 42"
        caught: string = ""
        try {
            try {
                throw original
            } catch (e) {
                rethrow
            }
        } catch (outer_e) {
            caught = outer_e
        }
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let caught: String = executor.execute_and_get(&lua_code, "caught").unwrap();
    assert!(
        caught.contains("specific error 42"),
        "Rethrown error should preserve message, got: {}",
        caught
    );
}

#[test]
fn test_nested_try_with_rethrow() {
    // Three levels of nesting: innermost throws, middle rethrows, outermost catches
    let source = r#"
        level: string = ""
        try {
            try {
                try {
                    throw "deep error"
                } catch (e1) {
                    rethrow
                }
            } catch (e2) {
                rethrow
            }
        } catch (e3) {
            level = e3
        }
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let level: String = executor.execute_and_get(&lua_code, "level").unwrap();
    assert!(
        level.contains("deep error"),
        "Error should propagate through two rethrows, got: {}",
        level
    );
}

#[test]
fn test_code_before_rethrow() {
    // Code before rethrow should execute, then error propagates
    let source = r#"
        logged: boolean = false
        caught: string = ""
        try {
            try {
                throw "test"
            } catch (e) {
                logged = true
                rethrow
            }
        } catch (outer_e) {
            caught = outer_e
        }
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let logged: bool = executor.execute_and_get(&lua_code, "logged").unwrap();
    let caught: String = executor.execute_and_get(&lua_code, "caught").unwrap();
    assert!(logged, "Code before rethrow should execute");
    assert!(
        caught.contains("test"),
        "Error should still propagate after rethrow"
    );
}

#[test]
fn test_try_expression_catch_variable() {
    // try expression: `try risky() catch 99` returns fallback when error occurs
    let source = r#"
        function risky(): number {
            throw "oops"
            return 0
        }
        result: number = try risky() catch 99
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 99);
}

#[test]
fn test_try_expression_success_path() {
    // try expression returns the successful value when no error
    let source = r#"
        function safe(): number {
            return 42
        }
        result: number = try safe() catch 0
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_rethrow_with_side_effects() {
    // Side effects in catch execute before rethrow propagates
    let source = r#"
        counter: number = 0
        caught: string = ""
        try {
            try {
                throw "err"
            } catch (e) {
                counter += 1
                counter += 1
                counter += 1
                rethrow
            }
        } catch (outer_e) {
            caught = outer_e
        }
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let counter: i64 = executor.execute_and_get(&lua_code, "counter").unwrap();
    let caught: String = executor.execute_and_get(&lua_code, "caught").unwrap();
    assert_eq!(
        counter, 3,
        "All three increments should execute before rethrow"
    );
    assert!(caught.contains("err"), "Error should propagate");
}
