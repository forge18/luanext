//! LuaJIT codegen tests.
//!
//! Tests that LuaNext correctly generates code targeting LuaJIT.
//! LuaJIT is based on Lua 5.1 with extensions:
//!   - Built-in `bit` library (not pure-Lua helpers)
//!   - `goto`/label support (unlike standard Lua 5.1)
//!   - No native bitwise operators (uses function calls)
//!   - No native integer division (uses math.floor)
//!   - No `global` keyword (bare assignment)
//!   - No built-in preamble needed (bit lib is C-side)

use luanext_core::codegen::LuaTarget;
use luanext_test_helpers::compile::compile_with_target;

#[test]
fn test_luajit_bitwise_and() {
    let source = r#"
        const a: number = 15
        const b: number = 7
        const x: number = a & b
    "#;
    let lua_code = compile_with_target(source, LuaTarget::LuaJIT).unwrap();
    assert!(
        lua_code.contains("bit.band(a, b)"),
        "Expected bit.band for LuaJIT, got:\n{lua_code}"
    );
}

#[test]
fn test_luajit_bitwise_or() {
    let source = r#"
        const a: number = 5
        const b: number = 3
        const x: number = a | b
    "#;
    let lua_code = compile_with_target(source, LuaTarget::LuaJIT).unwrap();
    assert!(
        lua_code.contains("bit.bor(a, b)"),
        "Expected bit.bor for LuaJIT, got:\n{lua_code}"
    );
}

#[test]
fn test_luajit_bitwise_xor() {
    let source = r#"
        const a: number = 10
        const b: number = 6
        const x: number = a ~ b
    "#;
    let lua_code = compile_with_target(source, LuaTarget::LuaJIT).unwrap();
    assert!(
        lua_code.contains("bit.bxor(a, b)"),
        "Expected bit.bxor for LuaJIT, got:\n{lua_code}"
    );
}

#[test]
fn test_luajit_shift_left() {
    let source = r#"
        const a: number = 1
        const x: number = a << 2
    "#;
    let lua_code = compile_with_target(source, LuaTarget::LuaJIT).unwrap();
    assert!(
        lua_code.contains("bit.lshift(a, 2)"),
        "Expected bit.lshift for LuaJIT, got:\n{lua_code}"
    );
}

#[test]
fn test_luajit_shift_right() {
    let source = r#"
        const a: number = 16
        const x: number = a >> 3
    "#;
    let lua_code = compile_with_target(source, LuaTarget::LuaJIT).unwrap();
    assert!(
        lua_code.contains("bit.rshift(a, 3)"),
        "Expected bit.rshift for LuaJIT, got:\n{lua_code}"
    );
}

#[test]
fn test_luajit_integer_divide() {
    let source = r#"
        const a: number = 10
        const b: number = 3
        const x: number = a // b
    "#;
    let lua_code = compile_with_target(source, LuaTarget::LuaJIT).unwrap();
    assert!(
        lua_code.contains("math.floor(a / b)"),
        "Expected math.floor for LuaJIT integer division, got:\n{lua_code}"
    );
}

#[test]
fn test_luajit_no_preamble() {
    // LuaJIT has bit lib built-in — should NOT emit pure-Lua helper functions
    let source = r#"
        const a: number = 15
        const b: number = 7
        const x: number = a & b
    "#;
    let lua_code = compile_with_target(source, LuaTarget::LuaJIT).unwrap();
    assert!(
        !lua_code.contains("_bit_band"),
        "LuaJIT should not emit Lua 5.1 style _bit_band preamble, got:\n{lua_code}"
    );
    assert!(
        !lua_code.contains("local function _bit_"),
        "LuaJIT should not emit helper function preamble, got:\n{lua_code}"
    );
}

#[test]
fn test_luajit_continue_uses_goto() {
    // LuaJIT supports goto (unlike standard Lua 5.1), so continue uses goto
    let source = r#"
        local total: number = 0
        for i = 1, 10 do
            if i == 5 then
                continue
            end
            total = total + i
        end
    "#;
    let lua_code = compile_with_target(source, LuaTarget::LuaJIT).unwrap();
    assert!(
        lua_code.contains("goto __continue"),
        "Expected goto __continue for LuaJIT, got:\n{lua_code}"
    );
    assert!(
        lua_code.contains("::__continue::"),
        "Expected ::__continue:: label for LuaJIT, got:\n{lua_code}"
    );
}

#[test]
fn test_luajit_global_bare_assignment() {
    // LuaJIT has no `global` keyword — just bare assignment
    let source = "global x: number = 42";
    let lua_code = compile_with_target(source, LuaTarget::LuaJIT).unwrap();
    assert!(
        lua_code.contains("x = 42"),
        "Expected bare 'x = 42' for LuaJIT global, got:\n{lua_code}"
    );
    assert!(
        !lua_code.contains("local x"),
        "LuaJIT global should not emit 'local', got:\n{lua_code}"
    );
    assert!(
        !lua_code.contains("global x"),
        "LuaJIT should not emit 'global' keyword, got:\n{lua_code}"
    );
}

#[test]
fn test_luajit_local_still_works() {
    // Regular local declarations should still work
    let source = "const x: number = 42";
    let lua_code = compile_with_target(source, LuaTarget::LuaJIT).unwrap();
    assert!(
        lua_code.contains("local x = 42"),
        "Expected 'local x = 42' for const, got:\n{lua_code}"
    );
}

#[test]
fn test_luajit_vs_lua51_preamble_difference() {
    // Lua 5.1 emits pure-Lua helper functions, LuaJIT does not
    let source = r#"
        const a: number = 15
        const b: number = 7
        const x: number = a & b
    "#;
    let lua51_code = compile_with_target(source, LuaTarget::Lua51).unwrap();
    let luajit_code = compile_with_target(source, LuaTarget::LuaJIT).unwrap();

    // Lua 5.1 should have preamble helpers
    assert!(
        lua51_code.contains("_bit_band"),
        "Lua 5.1 should use _bit_band helper, got:\n{lua51_code}"
    );

    // LuaJIT should use bit library directly
    assert!(
        luajit_code.contains("bit.band"),
        "LuaJIT should use bit.band, got:\n{luajit_code}"
    );
}

#[test]
fn test_luajit_vs_lua51_continue_difference() {
    // Lua 5.1 uses repeat..until hack (no goto), LuaJIT uses goto
    let source = r#"
        local total: number = 0
        for i = 1, 10 do
            if i == 5 then
                continue
            end
            total = total + i
        end
    "#;
    let lua51_code = compile_with_target(source, LuaTarget::Lua51).unwrap();
    let luajit_code = compile_with_target(source, LuaTarget::LuaJIT).unwrap();

    // Lua 5.1 uses repeat..until true hack (no goto support)
    assert!(
        lua51_code.contains("repeat"),
        "Lua 5.1 should use repeat..until for continue, got:\n{lua51_code}"
    );

    // LuaJIT uses goto (supported via LuaJIT extension)
    assert!(
        luajit_code.contains("goto __continue"),
        "LuaJIT should use goto for continue, got:\n{luajit_code}"
    );
}
