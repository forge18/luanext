//! Type definition tests.
//!
//! These tests verify that LuaNext's type system definitions — type aliases,
//! interfaces, generics, and utility types — are correctly erased from the
//! generated Lua output while the type checker validates them correctly.
//!
//! NOTE: Tests for `.d.tl` file loading (declaration files for external Lua
//! libraries) require multi-file compilation infrastructure and live in the
//! `luanext-cli/tests/` integration tests. This file covers single-file
//! type definition patterns.

use luanext_core::config::CompilerConfig;
use luanext_core::di::DiContainer;
use luanext_core::diagnostics::{CollectingDiagnosticHandler, DiagnosticHandler};
use luanext_core::fs::MockFileSystem;
use luanext_test_helpers::compile::{compile, compile_with_stdlib};
use luanext_test_helpers::LuaExecutor;
use std::sync::Arc;

#[test]
fn test_type_alias_is_erased_from_output() {
    let source = r#"
        type UserId = number
        id: UserId = 42
        result: number = id
    "#;
    let lua_code = compile(source).unwrap();
    // Type alias should be completely erased
    assert!(
        !lua_code.contains("UserId"),
        "type alias should be erased from Lua output, got:\n{lua_code}"
    );
    assert!(
        !lua_code.contains("type "),
        "type keyword should be erased from Lua output"
    );
}

#[test]
fn test_type_alias_value_executes_correctly() {
    let source = r#"
        type Score = number
        score: Score = 100
        result: number = score
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 100);
}

#[test]
fn test_interface_is_erased_from_output() {
    let source = r#"
        interface Vector2D {
            x: number
            y: number
        }
        v: Vector2D = { x: 3, y: 4 }
        result: number = v.x
    "#;
    let lua_code = compile(source).unwrap();
    assert!(
        !lua_code.contains("interface"),
        "interface keyword should be erased from Lua output"
    );
    assert!(
        !lua_code.contains("Vector2D"),
        "interface name should not appear in Lua output"
    );
}

#[test]
fn test_interface_usage_executes_correctly() {
    let source = r#"
        interface Point {
            x: number
            y: number
        }
        p: Point = { x: 5, y: 10 }
        result: number = p.x + p.y
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 15);
}

#[test]
fn test_generic_function_is_erased_to_plain_function() {
    let source = r#"
        function identity<T>(x: T): T {
            return x
        }
        result: number = identity<number>(42)
    "#;
    let lua_code = compile(source).unwrap();
    // Generic parameters should be erased
    assert!(
        !lua_code.contains('<'),
        "generic type parameters should be erased, got:\n{lua_code}"
    );
    assert!(
        !lua_code.contains('>'),
        "generic type parameters should be erased"
    );
}

#[test]
fn test_generic_function_executes_correctly() {
    let source = r#"
        function wrap<T>(x: T): T {
            return x
        }
        result: number = wrap<number>(99)
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 99);
}

#[test]
fn test_type_mismatch_caught_at_compile_time() {
    // Passing wrong type to typed function should produce a type error diagnostic.
    // The compiler continues to generate code (non-fatal), but the error is recorded.
    let source = r#"
        function greet(name: string): void {
            local msg: string = "hello " .. name
        }
        greet(123)
    "#;
    let diagnostics = Arc::new(CollectingDiagnosticHandler::new());
    let fs = Arc::new(MockFileSystem::new());
    let mut container = DiContainer::test(CompilerConfig::default(), diagnostics.clone(), fs);
    let _ = container.compile(source);
    assert!(
        diagnostics.has_errors(),
        "type mismatch (number for string param) should produce a type error diagnostic"
    );
}

#[test]
fn test_union_type_accepts_multiple_types() {
    // union type parameter should accept both constituent types without error.
    // tostring() is a stdlib function, so compile_with_stdlib is needed.
    let source = r#"
        function stringify(x: number | string): string {
            return tostring(x)
        }
        r1: string = stringify(42)
        r2: string = stringify("hello")
    "#;
    let result = compile_with_stdlib(source);
    assert!(
        result.is_ok(),
        "union type should accept both number and string"
    );
}

#[test]
fn test_declarations_before_use_compile_correctly() {
    // Type declarations at the top, usage below — classic "header" pattern
    let source = r#"
        type Meters = number
        type Seconds = number

        function speed(dist: Meters, time: Seconds): number {
            return dist / time
        }

        result: number = speed(100, 10)
    "#;
    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 10);
}

// ============================================================================
// Generic Specialization Runtime
// ============================================================================

#[test]
fn test_generic_class_instantiation_erased() {
    // Generic class new Box<number>(42) compiles with type args fully erased
    let source = r#"
        class Box<T> {
            value: T

            constructor(v: T) {
                self.value = v
            }

            get(): T {
                return self.value
            }
        }

        b = new Box(42)
        result: number = b::get()
    "#;

    let lua_code = compile(source).unwrap();
    assert!(
        !lua_code.contains('<'),
        "generic type args should be erased, got:\n{lua_code}"
    );
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_generic_function_multiple_type_params() {
    // Generic function with multiple type parameters — all erased at runtime
    let source = r#"
        function first<A, B>(a: A, b: B): A {
            return a
        }
        result: number = first<number, string>(42, "hello")
    "#;

    let lua_code = compile(source).unwrap();
    assert!(
        !lua_code.contains('<'),
        "generic type args should be erased"
    );
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_generic_class_with_methods() {
    // Generic class with methods — verify constructor and field access work after erasure
    let source = r#"
        class Pair<A, B> {
            first: A
            second: B

            constructor(a: A, b: B) {
                self.first = a
                self.second = b
            }
        }

        const p: Pair<number, string> = new Pair<number, string>(42, "hello")
        r1: number = p.first
        r2: string = p.second
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let r1: i64 = executor.execute_and_get(&lua_code, "r1").unwrap();
    let r2: String = executor.execute_and_get(&lua_code, "r2").unwrap();
    assert_eq!(r1, 42);
    assert_eq!(r2, "hello");
}

#[test]
fn test_generic_function_different_instantiations() {
    // Same generic function called with different type arguments
    // At runtime it is the same function, just called twice
    let source = r#"
        function identity<T>(x: T): T {
            return x
        }
        r1: number = identity<number>(42)
        r2: string = identity<string>("hello")
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let r1: i64 = executor.execute_and_get(&lua_code, "r1").unwrap();
    let r2: String = executor.execute_and_get(&lua_code, "r2").unwrap();
    assert_eq!(r1, 42);
    assert_eq!(r2, "hello");
}

#[test]
fn test_generic_default_type_parameter() {
    // Generic class with default type parameter — erased at runtime
    let source = r#"
        class Container<T = number> {
            value: T

            constructor(v: T) {
                self.value = v
            }

            get(): T {
                return self.value
            }
        }

        const c: Container<number> = new Container(99)
        result: number = c::get()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 99);
}
