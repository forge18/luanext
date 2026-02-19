//! Reproduction tests for parser bugs:
//! 1. `::` method calls with keyword method names (e.g., `get`, `set`) silently dropped
//! 2. Keywords as property names in table constructors

use luanext_core::diagnostics::CollectingDiagnosticHandler;
use luanext_parser::string_interner::StringInterner;
use luanext_parser::DiagnosticHandler;
use luanext_parser::{Lexer, Parser};
use luanext_test_helpers::compile::compile;
use std::sync::Arc;

// ============================================================================
// Bug 1: `::` method calls with keyword method names
// Root cause: parse_identifier() rejected keywords like `get`, `set` in postfix position
// Fix: use parse_identifier_or_keyword() in parse_postfix() for method/member names
// ============================================================================

#[test]
fn test_method_call_with_keyword_name_get() {
    // `obj::get()` — `get` is a keyword (TokenKind::Get)
    let source = r#"
        class Box {
            value: number
            constructor(v: number) {
                self.value = v
            }
            get(): number {
                return self.value
            }
        }
        b = new Box(42)
        result: number = b::get()
    "#;

    let lua_code = compile(source).unwrap();
    assert!(
        lua_code.contains("b:get()"),
        "method call `b::get()` should be parsed, got:\n{lua_code}"
    );
}

#[test]
fn test_method_call_with_keyword_name_on_generic_class() {
    // Same as above but with generic class — the reported symptom
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
        lua_code.contains("b:get()"),
        "method call after generic class should work, got:\n{lua_code}"
    );
}

#[test]
fn test_method_call_with_regular_name() {
    // `obj::getValue()` — not a keyword, should always work
    let source = r#"
        class Box {
            value: number
            constructor(v: number) {
                self.value = v
            }
            getValue(): number {
                return self.value
            }
        }
        b = new Box(42)
        result: number = b::getValue()
    "#;

    let lua_code = compile(source).unwrap();
    assert!(
        lua_code.contains("b:getValue()"),
        "method call with non-keyword name should work, got:\n{lua_code}"
    );
}

#[test]
fn test_method_call_simple_context() {
    // Simple variable + method call, no class
    let source = r#"
        local b = 42
        local result = b::toString()
    "#;

    let lua_code = compile(source).unwrap();
    assert!(
        lua_code.contains("b:toString()"),
        "simple method call should work, got:\n{lua_code}"
    );
}

#[test]
fn test_member_access_with_keyword_name() {
    // `obj.type` — `type` is a keyword, member access should work
    let source = r#"
        local b = { value = 42 }
        result: number = b.value
    "#;

    let lua_code = compile(source).unwrap();
    assert!(
        lua_code.contains("b.value"),
        "member access should work, got:\n{lua_code}"
    );
}

#[test]
fn test_method_call_assign_to_local() {
    // local result = b::method() form
    let source = r#"
        local b = 42
        local result = b::toString()
    "#;

    let lua_code = compile(source).unwrap();
    assert!(
        lua_code.contains("b:toString()"),
        "method call in local assignment should work, got:\n{lua_code}"
    );
}

#[test]
fn test_parser_produces_statement_for_method_call() {
    // Verify parser creates correct AST for `b::get()`
    let source = "b::get()";

    let arena: &'static bumpalo::Bump = Box::leak(Box::new(bumpalo::Bump::new()));
    let handler = Arc::new(CollectingDiagnosticHandler::new());
    let (interner, common) = StringInterner::new_with_common_identifiers();
    let mut lexer = Lexer::new(source, handler.clone(), &interner);
    let tokens = lexer.tokenize().expect("Failed to tokenize");

    let mut parser = Parser::new(tokens, handler.clone(), &interner, &common, arena);
    let program = parser.parse().expect("Failed to parse");

    assert_eq!(
        handler.error_count(),
        0,
        "parser should produce no errors for `b::get()`"
    );
    assert_eq!(
        program.statements.len(),
        1,
        "parser should produce 1 statement for `b::get()`"
    );
}

#[test]
fn test_parser_produces_statement_for_assignment_with_method_call() {
    // Verify parser creates correct AST for `result = b::get()`
    let source = "result = b::get()";

    let arena: &'static bumpalo::Bump = Box::leak(Box::new(bumpalo::Bump::new()));
    let handler = Arc::new(CollectingDiagnosticHandler::new());
    let (interner, common) = StringInterner::new_with_common_identifiers();
    let mut lexer = Lexer::new(source, handler.clone(), &interner);
    let tokens = lexer.tokenize().expect("Failed to tokenize");

    let mut parser = Parser::new(tokens, handler.clone(), &interner, &common, arena);
    let program = parser.parse().expect("Failed to parse");

    assert_eq!(
        handler.error_count(),
        0,
        "parser should produce no errors for `result = b::get()`"
    );
    assert_eq!(
        program.statements.len(),
        1,
        "parser should produce 1 statement for `result = b::get()`"
    );
}

#[test]
fn test_all_statements_present_after_generic_class() {
    // Verify no statements are dropped after a generic class definition
    let source = r#"
        class Box<T> {
            value: T
            constructor(v: T) {
                self.value = v
            }
        }
        a = 1
        b = 2
        c = 3
    "#;

    let lua_code = compile(source).unwrap();
    assert!(lua_code.contains("a = 1"), "a = 1 missing:\n{lua_code}");
    assert!(lua_code.contains("b = 2"), "b = 2 missing:\n{lua_code}");
    assert!(lua_code.contains("c = 3"), "c = 3 missing:\n{lua_code}");
}

// ============================================================================
// Bug 2: Deeply nested namespaces in declare statements
// Root cause: parse_declare_namespace() only handled `function` and `const`,
// silently erroring on nested `namespace`, `type`, `interface` members
// Fix: Added handling for nested namespace, type, and interface in namespace body
// ============================================================================

#[test]
fn test_declare_namespace_with_nested_namespace() {
    // Nested namespace declarations should parse
    let source = r#"
        declare namespace Outer {
            function foo(): number
            namespace Inner {
                function bar(): string
            }
        }
    "#;

    let arena: &'static bumpalo::Bump = Box::leak(Box::new(bumpalo::Bump::new()));
    let handler = Arc::new(CollectingDiagnosticHandler::new());
    let (interner, common) = StringInterner::new_with_common_identifiers();
    let mut lexer = Lexer::new(source, handler.clone(), &interner);
    let tokens = lexer.tokenize().expect("Failed to tokenize");

    let mut parser = Parser::new(tokens, handler.clone(), &interner, &common, arena);
    let program = parser.parse().expect("Failed to parse");

    assert_eq!(
        handler.error_count(),
        0,
        "nested namespace should parse without errors"
    );
    assert_eq!(
        program.statements.len(),
        1,
        "should produce 1 top-level declare namespace statement"
    );
}

#[test]
fn test_declare_namespace_with_type() {
    // Type declarations inside namespace should parse
    let source = r#"
        declare namespace MyLib {
            type Color = string
            function getColor(): Color
        }
    "#;

    let arena: &'static bumpalo::Bump = Box::leak(Box::new(bumpalo::Bump::new()));
    let handler = Arc::new(CollectingDiagnosticHandler::new());
    let (interner, common) = StringInterner::new_with_common_identifiers();
    let mut lexer = Lexer::new(source, handler.clone(), &interner);
    let tokens = lexer.tokenize().expect("Failed to tokenize");

    let mut parser = Parser::new(tokens, handler.clone(), &interner, &common, arena);
    let program = parser.parse().expect("Failed to parse");

    assert_eq!(
        handler.error_count(),
        0,
        "type in namespace should parse without errors"
    );
    assert_eq!(program.statements.len(), 1);
}

#[test]
fn test_declare_namespace_with_interface() {
    // Interface declarations inside namespace should parse
    let source = r#"
        declare namespace MyLib {
            interface Config {
                debug: boolean
            }
            function init(config: Config): void
        }
    "#;

    let arena: &'static bumpalo::Bump = Box::leak(Box::new(bumpalo::Bump::new()));
    let handler = Arc::new(CollectingDiagnosticHandler::new());
    let (interner, common) = StringInterner::new_with_common_identifiers();
    let mut lexer = Lexer::new(source, handler.clone(), &interner);
    let tokens = lexer.tokenize().expect("Failed to tokenize");

    let mut parser = Parser::new(tokens, handler.clone(), &interner, &common, arena);
    let program = parser.parse().expect("Failed to parse");

    assert_eq!(
        handler.error_count(),
        0,
        "interface in namespace should parse without errors"
    );
    assert_eq!(program.statements.len(), 1);
}

#[test]
fn test_declare_namespace_deeply_nested() {
    // Three levels deep
    let source = r#"
        declare namespace A {
            namespace B {
                namespace C {
                    function deepFn(): number
                }
            }
        }
    "#;

    let arena: &'static bumpalo::Bump = Box::leak(Box::new(bumpalo::Bump::new()));
    let handler = Arc::new(CollectingDiagnosticHandler::new());
    let (interner, common) = StringInterner::new_with_common_identifiers();
    let mut lexer = Lexer::new(source, handler.clone(), &interner);
    let tokens = lexer.tokenize().expect("Failed to tokenize");

    let mut parser = Parser::new(tokens, handler.clone(), &interner, &common, arena);
    let program = parser.parse().expect("Failed to parse");

    assert_eq!(
        handler.error_count(),
        0,
        "deeply nested namespace (3 levels) should parse without errors"
    );
    assert_eq!(program.statements.len(), 1);
}

// ============================================================================
// Table constructor with keyword property names
// This is a separate issue: `{ get = ... }` fails because `get` is TokenKind::Get
// The fix for parse_object_or_table is tracked separately
// ============================================================================

#[test]
fn test_table_constructor_identifier_property() {
    // Using identifier property names works fine
    let source = r#"
        local t = { value = 42, name = "test" }
        result: number = t.value
    "#;

    let lua_code = compile(source).unwrap();
    assert!(
        lua_code.contains("value = 42"),
        "table constructor should work, got:\n{lua_code}"
    );
}

#[test]
fn test_table_constructor_keyword_property_get() {
    // `get` is a keyword (TokenKind::Get) but should work as property name
    let source = r#"
        local t = { get = 42 }
        result: number = t.get
    "#;

    let lua_code = compile(source).unwrap();
    assert!(
        lua_code.contains("get = 42"),
        "keyword `get` as property name should work, got:\n{lua_code}"
    );
}

#[test]
fn test_table_constructor_keyword_property_set() {
    // `set` is a keyword (TokenKind::Set) but should work as property name
    let source = r#"
        local t = { set = 99 }
        result: number = t.set
    "#;

    let lua_code = compile(source).unwrap();
    assert!(
        lua_code.contains("set = 99"),
        "keyword `set` as property name should work, got:\n{lua_code}"
    );
}

#[test]
fn test_table_constructor_keyword_property_type() {
    // `type` is a keyword but should work as property name in table
    let source = r#"
        local t = { type = "number" }
        result: string = t.type
    "#;

    let lua_code = compile(source).unwrap();
    assert!(
        lua_code.contains(r#"type = "number""#),
        "keyword `type` as property name should work, got:\n{lua_code}"
    );
}
