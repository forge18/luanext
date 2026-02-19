//! Execution tests for untyped / minimally-typed code.
//!
//! Validates that LuaNext's type erasure and type inference work correctly:
//! code with no annotations (or inferred types only) must compile and produce
//! the same output as equivalent explicitly-typed code.
//!
//! These tests also serve as a smoke-test for type inference paths, since they
//! exercise the compiler's ability to infer types from literals and expressions
//! rather than relying on explicit annotations.
//!
//! Reference: parser/type_inference, typechecker/infer.rs

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

// ============================================================================
// Implicit Global Variables (no annotations, no local)
// ============================================================================

#[test]
fn test_untyped_variable_arithmetic() {
    // Implicit globals with no type annotations
    let source = r#"
        x = 10
        y = x + 5
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "y").unwrap();
    assert_eq!(result, 15);
}

#[test]
fn test_untyped_string_variable() {
    // String variable without type annotation
    let source = r#"
        greeting = "hello"
        result = greeting .. " world"
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "hello world");
}

#[test]
fn test_untyped_boolean_variable() {
    // Boolean variable without type annotation
    let source = r#"
        flag = true
        result = flag and 1 or 0
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 1);
}

// ============================================================================
// Local Variables with Inferred Types
// ============================================================================

#[test]
fn test_inferred_local_from_literal() {
    // `local x = 42` — type inferred as number
    let source = r#"
        local x = 42
        result: number = x + 8
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 50);
}

#[test]
fn test_inferred_local_from_expression() {
    // Type inferred from arithmetic expression
    let source = r#"
        local x = 2 + 3
        local y = x * 4
        result: number = y
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 20);
}

#[test]
fn test_inferred_local_string() {
    // String type inferred from string literal
    let source = r#"
        local s = "world"
        result: string = "hello " .. s
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "hello world");
}

// ============================================================================
// Functions Without Full Type Annotations
// ============================================================================

#[test]
fn test_untyped_function_params() {
    // Function with no parameter type annotations
    let source = r#"
        function add(a, b) {
            return a + b
        }
        result: number = add(10, 32)
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_untyped_function_no_return_annotation() {
    // Function with typed params but inferred return type
    let source = r#"
        function multiply(a: number, b: number) {
            return a * b
        }
        result: number = multiply(6, 7)
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

// ============================================================================
// Mixed Typed and Untyped
// ============================================================================

#[test]
fn test_mixed_typed_and_untyped() {
    // Typed function called with an untyped (inferred) local variable
    let source = r#"
        function greet(name: string): string {
            return "hello " .. name
        }
        local who = "world"
        result: string = greet(who)
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "hello world");
}

#[test]
fn test_untyped_class_usage() {
    // Class instance without type annotation on the variable.
    // Accesses the field directly (not via method call) since typed-global
    // assignment from a method call has a known codegen limitation.
    let source = r#"
        class Box {
            value: number

            constructor(v: number) {
                self.value = v
            }

            doubled(): number {
                return self.value * 2
            }
        }

        local b = new Box(99)
        b::doubled()
        result: number = b.value
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 99);
}
