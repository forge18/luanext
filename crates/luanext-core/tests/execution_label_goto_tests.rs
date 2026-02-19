//! Execution tests for `::label::` and `goto` statements.
//!
//! Labels and goto are native Lua features (5.2+). LuaNext passes them
//! through directly: `::name::` → `::name::`, `goto name` → `goto name`.
//!
//! Since mlua runs Lua 5.4 (which supports goto), all tests execute at runtime.
//!
//! Note: Tests use functions or semicolons to ensure parser handles multi-statement
//! blocks correctly with labels and goto.

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

// ============================================================================
// Basic forward jump
// ============================================================================

#[test]
fn test_goto_forward_jump() {
    // goto skips assignment, x stays at 1
    let source = r#"
        function test(): number {
            const x: number = 1
            goto skip
            ::skip::
            return x
        }
        result: number = test()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 1);
}

// ============================================================================
// Backward jump (loop simulation)
// ============================================================================

#[test]
fn test_goto_backward_jump_loop() {
    // Simulate a loop using goto
    let source = "count: number = 0; ::top::; count = count + 1; if count < 5 then goto top end";

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let count: i64 = executor.execute_and_get(&lua_code, "count").unwrap();
    assert_eq!(count, 5);
}

// ============================================================================
// Skipping assignments via goto
// ============================================================================

#[test]
fn test_goto_skips_code_section() {
    // Function with goto that skips over a reassignment
    // Use semicolons to ensure parser handles statements correctly after goto
    let source = r#"
        function test(): number {
            local x: number = 1;
            goto after;
            x = 99;
            ::after::;
            return x
        }
        result: number = test()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 1);
}

// ============================================================================
// Multiple labels with cross-jumping
// ============================================================================

#[test]
fn test_multiple_labels() {
    // Jump between labels using semicolons as delimiters
    let source = r#"
        function test(): string {
            local path: string = ""
            goto second
            ::first::
            path = path .. "1"
            goto done
            ::second::
            path = path .. "2"
            goto first
            ::done::
            return path
        }
        result: string = test()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "21");
}

// ============================================================================
// Label inside conditional block
// ============================================================================

#[test]
fn test_label_in_if_block() {
    // goto forward inside an if block
    let source = r#"
        function test(): number {
            local x: number = 0;
            if true then
                goto inside;
                x = 99;
                ::inside::;
                x = 42
            end
            return x
        }
        result: number = test()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

// ============================================================================
// Goto exiting a for loop
// ============================================================================

#[test]
fn test_goto_exits_for_loop() {
    // Use goto to exit a loop early
    let source = r#"
        function test(): number {
            local sum: number = 0
            for i = 1, 10 do
                if i == 4 then
                    goto done
                end
                sum = sum + i
            end
            ::done::
            return sum
        }
        result: number = test()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 6); // 1 + 2 + 3
}

// ============================================================================
// Codegen output verification
// ============================================================================

#[test]
fn test_label_goto_codegen_output() {
    // Verify raw output syntax using semicolons for reliable parsing
    let source = "goto myLabel; ::myLabel::";

    let lua_code = compile(source).unwrap();
    assert!(
        lua_code.contains("goto myLabel"),
        "Should emit 'goto myLabel', got:\n{}",
        lua_code
    );
    assert!(
        lua_code.contains("::myLabel::"),
        "Should emit '::myLabel::', got:\n{}",
        lua_code
    );
}
