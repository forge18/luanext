//! Execution tests for multi-assignment.
//!
//! Multi-assignment: `a, b = 1, 2` compiles to native Lua multi-assignment.
//!
//! Reference: `codegen/statements.rs`
//!
//! Note: Label/goto tests are excluded because `::label::` syntax conflicts
//! with the `::` method call operator in the parser (silent parse failure).

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

#[test]
fn test_multi_value_assignment() {
    let source = r#"
        a: number = 0
        b: number = 0
        a, b = 1, 2
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let a: i64 = executor.execute_and_get(&lua_code, "a").unwrap();
    let b: i64 = executor.execute_and_get(&lua_code, "b").unwrap();
    assert_eq!(a, 1);
    assert_eq!(b, 2);
}

#[test]
fn test_swap_pattern() {
    let source = r#"
        a: number = 10
        b: number = 20
        a, b = b, a
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let a: i64 = executor.execute_and_get(&lua_code, "a").unwrap();
    let b: i64 = executor.execute_and_get(&lua_code, "b").unwrap();
    assert_eq!(a, 20);
    assert_eq!(b, 10);
}

#[test]
fn test_function_multi_return() {
    let source = r#"
        function get_pair(): (number, number) {
            return 10, 20
        }
        a: number = 0
        b: number = 0
        a, b = get_pair()
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let a: i64 = executor.execute_and_get(&lua_code, "a").unwrap();
    let b: i64 = executor.execute_and_get(&lua_code, "b").unwrap();
    assert_eq!(a, 10);
    assert_eq!(b, 20);
}

#[test]
fn test_multi_assign_with_expressions() {
    let source = r#"
        a: number = 0
        b: number = 0
        a, b = 1 + 2, 3 * 4
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let a: i64 = executor.execute_and_get(&lua_code, "a").unwrap();
    let b: i64 = executor.execute_and_get(&lua_code, "b").unwrap();
    assert_eq!(a, 3);
    assert_eq!(b, 12);
}

#[test]
fn test_multi_assign_excess_values() {
    // Extra values on the RHS are discarded
    let source = r#"
        a: number = 0
        b: number = 0
        a, b = 1, 2, 3
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let a: i64 = executor.execute_and_get(&lua_code, "a").unwrap();
    let b: i64 = executor.execute_and_get(&lua_code, "b").unwrap();
    assert_eq!(a, 1);
    assert_eq!(b, 2);
}

#[test]
fn test_multi_assign_fewer_values() {
    // Fewer values than targets: extras get nil
    let source = r#"
        a: number = 99
        b: number = 99
        c: number = 99
        a, b, c = 1, 2
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let a: i64 = executor.execute_and_get(&lua_code, "a").unwrap();
    let b: i64 = executor.execute_and_get(&lua_code, "b").unwrap();
    assert_eq!(a, 1);
    assert_eq!(b, 2);
    // c should be nil (Lua native behavior: fewer values → nil for remaining targets)
    let c_is_nil: bool = executor
        .execute_with_result::<bool>("return c == nil")
        .unwrap();
    assert!(
        c_is_nil,
        "c should be nil when there are fewer values than targets"
    );
}

#[test]
fn test_multi_assign_three_way_rotate() {
    // Three-way rotation: a=b, b=c, c=a
    let source = r#"
        a: number = 1
        b: number = 2
        c: number = 3
        a, b, c = b, c, a
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let a: i64 = executor.execute_and_get(&lua_code, "a").unwrap();
    let b: i64 = executor.execute_and_get(&lua_code, "b").unwrap();
    let c: i64 = executor.execute_and_get(&lua_code, "c").unwrap();
    assert_eq!(a, 2);
    assert_eq!(b, 3);
    assert_eq!(c, 1);
}

#[test]
fn test_multi_assign_with_function_calls() {
    // RHS expressions include function calls
    let source = r#"
        function double(x: number): number {
            return x * 2
        }
        function triple(x: number): number {
            return x * 3
        }
        a: number = 0
        b: number = 0
        a, b = double(5), triple(5)
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let a: i64 = executor.execute_and_get(&lua_code, "a").unwrap();
    let b: i64 = executor.execute_and_get(&lua_code, "b").unwrap();
    assert_eq!(a, 10);
    assert_eq!(b, 15);
}
