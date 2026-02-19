//! Lua 5.5 compatibility tests.
//!
//! Lua 5.5 (released December 2025) introduces several new features:
//!   - `global` declaration keyword (LuaNext already supports this!)
//!   - Named vararg: `function f(...name) end`
//!   - Compact arrays: `{1, 2, 3}` uses integer keys starting at 1 (same as before, but optimized)
//!   - Improved incremental GC
//!
//! CURRENT STATUS: mlua 0.10 only supports up to Lua 5.4. Lua 5.5 runtime tests
//! require mlua 0.11.6+ with the "lua55" feature flag.
//!
//! UPGRADE PATH:
//!   1. Bump `mlua` in Cargo.toml: `mlua = { version = "0.11.6", features = ["lua55", "vendored"] }`
//!   2. Add `LuaTarget::Lua55` variant to `luanext_core::codegen::LuaTarget`
//!   3. Add `Lua55Strategy` in `crates/luanext-core/src/codegen/strategies/`
//!
//! The tests marked `#[ignore]` document the intended behavior for Lua 5.5.
//! The remaining tests verify current Lua 5.4 output for features that Lua 5.5
//! will continue to support.

use luanext_core::codegen::LuaTarget;
use luanext_test_helpers::compile::{compile, compile_with_target};
use luanext_test_helpers::LuaExecutor;

#[test]
fn test_global_keyword_generates_without_local() {
    // LuaNext `global` keyword is already supported and maps to Lua 5.5 global declarations.
    // The generated Lua omits the `local` keyword — valid in Lua 5.4 as a global assignment.
    let source = r#"
        global count: number = 42
    "#;
    let lua_code = compile(source).unwrap();
    // Should not have `local count` — globals are emitted without `local`
    assert!(
        !lua_code.contains("local count"),
        "global keyword should generate without 'local', got:\n{lua_code}"
    );
    assert!(
        lua_code.contains("count = 42"),
        "global variable should be assigned without local, got:\n{lua_code}"
    );
}

#[test]
fn test_global_variable_accessible_at_runtime() {
    // Globals defined without `local` are accessible from the Lua global table
    let source = r#"
        global x: number = 100
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "x").unwrap();
    assert_eq!(result, 100);
}

#[test]
fn test_array_literal_output_is_valid_lua() {
    // Array literals should produce valid Lua table constructors in all versions
    let source = r#"
        arr: number[] = [1, 2, 3, 4, 5]
        result: number = arr[3]
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(
        result, 3,
        "1-indexed array access should return 3rd element"
    );
}

#[test]
#[ignore = "Requires mlua 0.11.6+ with lua55 feature. See upgrade path in module docs."]
fn test_lua55_named_vararg() {
    // Lua 5.5 allows: function f(...args) -- args is a table of vararg values
    // This would require a new LuaTarget::Lua55 variant and Lua55Strategy
    let source = r#"
        function sum(...args: number[]): number {
            local total: number = 0
            for i in args do
                total += i
            end
            return total
        }
        result: number = sum(1, 2, 3)
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 6);
}

#[test]
fn test_lua55_global_declaration_native_syntax() {
    // Lua 5.5 adds native `global` declaration syntax.
    // With Lua55Strategy: generates `global x = 42` (native Lua 5.5 syntax)
    let source = r#"
        global count: number = 42
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua55).unwrap();
    assert!(
        lua_code.contains("global count = 42"),
        "Expected native 'global count = 42' in Lua 5.5 output, got:\n{lua_code}"
    );
}

#[test]
fn test_lua55_native_continue() {
    // Lua 5.5 has native `continue` keyword — no goto/label emulation needed
    let source = r#"
        local total: number = 0
        for i = 1, 10 do
            if i == 5 then
                continue
            end
            total = total + i
        end
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua55).unwrap();
    assert!(
        lua_code.contains("continue"),
        "Expected native 'continue' in Lua 5.5 output, got:\n{lua_code}"
    );
    assert!(
        !lua_code.contains("goto __continue"),
        "Should not use goto emulation for Lua 5.5, got:\n{lua_code}"
    );
    assert!(
        !lua_code.contains("::__continue::"),
        "Should not have continue label for Lua 5.5, got:\n{lua_code}"
    );
}

#[test]
fn test_lua55_native_bitwise() {
    // Lua 5.5 supports native bitwise operators (same as 5.3+)
    let source = r#"
        const x: number = 15 & 7
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua55).unwrap();
    assert!(
        lua_code.contains("&"),
        "Expected native & operator in Lua 5.5 output, got:\n{lua_code}"
    );
}

#[test]
fn test_lua55_native_integer_divide() {
    // Lua 5.5 supports native // operator (same as 5.3+)
    let source = r#"
        const x: number = 10 // 3
    "#;
    let lua_code = compile_with_target(source, LuaTarget::Lua55).unwrap();
    assert!(
        lua_code.contains("//"),
        "Expected native // operator in Lua 5.5 output, got:\n{lua_code}"
    );
}
