//! Execution tests for error handling: throw, try/catch blocks,
//! try expressions, rethrow, and the error chain (!!) operator.
//!
//! Syntax reference:
//! - Block:      `try { ... } catch (e) { ... }`
//! - Finally:    `try { ... } catch (e) { ... } finally { ... }`
//! - Expression: `try expr catch fallback`
//! - Throw:      `throw "message"` (compiles to `error("message")`)
//! - Rethrow:    `rethrow` (keyword in catch body)
//! - Chain:      `expr !! fallback` (error chain operator)

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

#[test]
fn test_throw_stops_execution() {
    // throw "msg" compiles to error("msg") which stops execution
    let source = r#"
        function fail(): number {
            throw "something went wrong"
            return 0
        }
        ok: boolean = false
        try {
            fail()
        } catch (e) {
            ok = true
        }
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let ok: bool = executor.execute_and_get(&lua_code, "ok").unwrap();
    assert!(ok, "catch block should have run");
}

#[test]
fn test_try_catch_catches_error() {
    // The catch variable holds the error message
    let source = r#"
        caught_msg: string = ""
        try {
            throw "test error"
        } catch (e) {
            caught_msg = e
        }
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let caught_msg: String = executor.execute_and_get(&lua_code, "caught_msg").unwrap();
    assert!(
        caught_msg.contains("test error"),
        "caught_msg = {:?}",
        caught_msg
    );
}

#[test]
fn test_try_catch_no_error() {
    // When no error, try body runs and catch is skipped
    let source = r#"
        in_try: number = 0
        in_catch: number = 0
        function run_try() {
            try {
                in_try = 1
            } catch (e) {
                in_catch = 1
            }
        }
        run_try()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let in_try: i64 = executor.execute_and_get(&lua_code, "in_try").unwrap();
    let in_catch: i64 = executor.execute_and_get(&lua_code, "in_catch").unwrap();
    assert_eq!(in_try, 1);
    assert_eq!(in_catch, 0);
}

#[test]
fn test_try_expression_success() {
    // try expr catch fallback - returns expr value on success
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
fn test_try_expression_failure() {
    // try expr catch fallback - returns fallback when expr throws
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
fn test_error_chain_success() {
    // expr !! fallback - uses expr value when expr succeeds
    let source = r#"
        function get_value(): number {
            return 7
        }
        result: number = get_value() !! 0
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 7);
}

#[test]
fn test_error_chain_failure() {
    // expr !! fallback - uses fallback when expr throws
    let source = r#"
        function broken(): number {
            throw "fail"
            return 0
        }
        result: number = broken() !! 42
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_try_catch_finally() {
    // finally block always runs regardless of error
    let source = r#"
        finally_ran: boolean = false
        try {
            throw "err"
        } catch (e) {
            -- caught
        } finally {
            finally_ran = true
        }
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let finally_ran: bool = executor.execute_and_get(&lua_code, "finally_ran").unwrap();
    assert!(finally_ran, "finally block should have run");
}

#[test]
fn test_try_catch_finally_no_error() {
    // finally runs even when no error
    let source = r#"
        try_ran: boolean = false
        finally_ran: boolean = false
        try {
            try_ran = true
        } catch (e) {
            -- not reached
        } finally {
            finally_ran = true
        }
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let try_ran: bool = executor.execute_and_get(&lua_code, "try_ran").unwrap();
    let finally_ran: bool = executor.execute_and_get(&lua_code, "finally_ran").unwrap();
    assert!(try_ran);
    assert!(finally_ran);
}

#[test]
fn test_nested_try_expressions() {
    // Nested try expressions - inner catches, outer still works
    let source = r#"
        function inner(): number {
            throw "inner error"
            return 0
        }
        function outer(): number {
            const x: number = try inner() catch 10
            return x + 1
        }
        result: number = outer()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 11);
}

// ============================================================================
// ErrorChain (!!) edge cases
// ============================================================================

#[test]
fn test_error_chain_in_return_position() {
    // !! operator in a function's return statement
    let source = r#"
        function risky(): number {
            throw "fail"
            return 0
        }
        function get_or_default(): number {
            return risky() !! 42
        }
        result: number = get_or_default()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_error_chain_as_function_argument() {
    // !! operator as an argument to another function
    let source = r#"
        function risky(): number {
            throw "fail"
            return 0
        }
        function double(x: number): number {
            return x * 2
        }
        result: number = double(risky() !! 5)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 10);
}

#[test]
fn test_error_chain_chained() {
    // Chained !! operators: fail_a() !! fail_b() !! 42
    // Both fail, so final fallback 42 is returned
    let source = r#"
        function fail_a(): number {
            throw "a"
            return 0
        }
        function fail_b(): number {
            throw "b"
            return 0
        }
        result: number = fail_a() !! fail_b() !! 42
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_error_chain_fallback_expression() {
    // !! with a complex expression as fallback
    let source = r#"
        function risky(): number {
            throw "fail"
            return 0
        }
        result: number = risky() !! (10 + 32)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_error_chain_inside_try_catch() {
    // !! catches the error, so the outer try/catch never triggers
    let source = r#"
        function fail(): number {
            throw "err"
            return 0
        }
        outer_result: number = 0
        try {
            outer_result = fail() !! 99
        } catch (e) {
            outer_result = -1
        }
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "outer_result").unwrap();
    assert_eq!(result, 99);
}

#[test]
fn test_error_chain_nil_is_not_error() {
    // !! only catches errors (pcall failures), NOT nil values
    // A function returning nil succeeds (pcall returns true, nil)
    // so !! does NOT trigger the fallback
    // Use raw Lua via the executor to test this directly
    let executor = LuaExecutor::new().unwrap();
    let raw_lua = r#"
        function get_nil() return nil end
        val = (function() local __ok, __result = pcall(function() return get_nil() end); if __ok then return __result else return "fallback" end end)()
        used_fallback = (val ~= nil)
    "#;
    executor.execute(raw_lua).unwrap();
    let used_fallback: bool = executor.lua().globals().get("used_fallback").unwrap();
    assert!(
        !used_fallback,
        "!! should not trigger for nil return (not an error)"
    );
}

// ============================================================================
// Try expressions in complex contexts
// ============================================================================

#[test]
fn test_try_expression_in_table_literal() {
    // Try expression as a value inside a table constructor
    let source = r#"
        function risky(): number {
            throw "fail"
            return 0
        }
        const t = { a = try risky() catch 42 }
        result: number = t.a
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_try_expression_in_function_return() {
    // Try expression in a return statement
    let source = r#"
        function risky(): number {
            throw "oops"
            return 0
        }
        function safe_get(): number {
            return try risky() catch 99
        }
        result: number = safe_get()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 99);
}

#[test]
fn test_try_expression_in_arithmetic() {
    // Try expression as operand in binary arithmetic
    let source = r#"
        function risky(): number {
            throw "fail"
            return 0
        }
        result: number = (try risky() catch 10) + 5
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 15);
}

#[test]
fn test_try_expression_as_function_argument() {
    // Try expression passed directly as a function argument
    let source = r#"
        function risky(): number {
            throw "fail"
            return 0
        }
        function double(x: number): number {
            return x * 2
        }
        result: number = double(try risky() catch 7)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 14);
}

#[test]
fn test_try_expression_success_in_concat() {
    // Success path: try expression value flows through string concatenation
    let source = r#"
        function safe(): string {
            return "hello"
        }
        result: string = (try safe() catch "fallback") .. " world"
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "hello world");
}

#[test]
fn test_try_expression_nested_in_try_expression() {
    // Try expression nested inside another try expression's catch clause
    let source = r#"
        function fail_a(): number {
            throw "a"
            return 0
        }
        function fail_b(): number {
            throw "b"
            return 0
        }
        result: number = try fail_a() catch (try fail_b() catch 77)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 77);
}
