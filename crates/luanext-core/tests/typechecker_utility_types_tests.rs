//! Type checker tests for utility types (Partial, Required, Record, Pick, Omit,
//! NonNilable, Nilable, ReturnType, Parameters).
//!
//! These are compile-time type validation tests — they don't execute Lua.
//! They verify that the type checker correctly accepts code using utility types.
//!
//! Note: Negative tests (rejection of invalid code) are limited because the
//! type checker doesn't fully validate structural subtyping for utility-transformed
//! types. Positive acceptance tests are reliable.
//!
//! Reference: `crates/luanext-typechecker/src/types/utility_types.rs`

use luanext_test_helpers::compile::type_check;

#[test]
fn test_partial_accepts_incomplete() {
    // Partial<T> makes all properties optional, so missing props are OK
    let source = r#"
        interface User {
            name: string
            age: number
        }
        const partial_user: Partial<User> = { name: "Alice" }
    "#;
    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Partial<User> should accept incomplete object, got: {:?}",
        result.err()
    );
}

#[test]
fn test_partial_accepts_empty() {
    // Partial<T> should accept an empty object (all properties are optional)
    let source = r#"
        interface User {
            name: string
            age: number
        }
        const empty: Partial<User> = {}
    "#;
    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Partial<User> should accept empty object, got: {:?}",
        result.err()
    );
}

#[test]
fn test_partial_accepts_complete() {
    // Partial<T> also accepts a complete object
    let source = r#"
        interface User {
            name: string
            age: number
        }
        const full: Partial<User> = { name: "Alice", age: 30 }
    "#;
    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Partial<User> should accept complete object, got: {:?}",
        result.err()
    );
}

#[test]
fn test_required_accepts_complete() {
    // Required<T> accepts when all properties are present
    let source = r#"
        interface Config {
            host?: string
            port?: number
        }
        const config: Required<Config> = { host: "localhost", port: 8080 }
    "#;
    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Required<Config> should accept complete object, got: {:?}",
        result.err()
    );
}

#[test]
fn test_record_accepts_matching() {
    // Record<string, number> accepts string keys with number values
    let source = r#"
        const scores: Record<string, number> = { alice: 100, bob: 85 }
    "#;
    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Record<string, number> should accept matching key-value pairs, got: {:?}",
        result.err()
    );
}

#[test]
fn test_pick_selects_fields() {
    // Pick<T, K> keeps only selected fields
    let source = r#"
        interface User {
            name: string
            age: number
            email: string
        }
        const name_only: Pick<User, "name"> = { name: "Alice" }
    "#;
    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Pick<User, 'name'> should accept object with just 'name', got: {:?}",
        result.err()
    );
}

#[test]
fn test_omit_excludes_fields() {
    // Omit<T, K> removes specified fields
    let source = r#"
        interface User {
            name: string
            age: number
            email: string
        }
        const no_email: Omit<User, "email"> = { name: "Alice", age: 30 }
    "#;
    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Omit<User, 'email'> should accept object without 'email', got: {:?}",
        result.err()
    );
}

#[test]
fn test_nilable_accepts_nil() {
    // Nilable<T> adds nil to a type
    let source = r#"
        const x: Nilable<string> = nil
    "#;
    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Nilable<string> should accept nil, got: {:?}",
        result.err()
    );
}

#[test]
fn test_nilable_accepts_value() {
    // Nilable<T> also accepts the base type
    let source = r#"
        const x: Nilable<string> = "hello"
    "#;
    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Nilable<string> should accept string value, got: {:?}",
        result.err()
    );
}

#[test]
fn test_return_type_extraction() {
    // ReturnType<T> extracts the return type of a function type
    let source = r#"
        function greet(): string {
            return "hello"
        }
        const x: ReturnType<typeof greet> = "world"
    "#;
    let result = type_check(source);
    assert!(
        result.is_ok(),
        "ReturnType should extract string return type, got: {:?}",
        result.err()
    );
}
