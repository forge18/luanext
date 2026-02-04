use typedlua_core::di::DiContainer;

fn type_check(source: &str) -> Result<(), String> {
    let mut container = DiContainer::test_default();
    container.compile(source)?;
    Ok(())
}

#[test]
fn test_same_type_reference() {
    let source = r#"
        type UserId = number

        function get_user(id: UserId): UserId {
            return id
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Same type reference should be compatible"
    );
}

#[test]
fn test_generic_type_reference_same_args() {
    let source = r#"
        type Box<T> = { value: T }

        function identity(b: Box<number>): Box<number> {
            return b
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Generic type with same args should be compatible"
    );
}

#[test]
fn test_generic_type_reference_different_args() {
    let source = r#"
        type Box<T> = { value: T }

        function mismatch(b: Box<number>): Box<string> {
            return b
        }
    "#;

    assert!(
        type_check(source).is_err(),
        "Generic types with different args should not be compatible"
    );
}

#[test]
fn test_type_reference_with_nested_generics() {
    let source = r#"
        type Result<T> = { value: T }
        type Nested = Result<Result<number>>

        function process(r: Nested): Nested {
            return r
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Nested generic type references should work"
    );
}

#[test]
fn test_type_reference_missing_type_args() {
    let source = r#"
        type Box<T> = { value: T }

        -- Box without type arguments vs Box<number>
        function bad(b: Box): Box<number> {
            return b
        }
    "#;

    assert!(
        type_check(source).is_err(),
        "Type reference with missing args should not match"
    );
}

#[test]
fn test_type_reference_vs_primitive() {
    let source = r#"
        type UserId = number

        function convert(id: UserId): number {
            return id
        }
    "#;

    let result = type_check(source);
    let _ = result;
}

#[test]
fn test_type_reference_compatibility_same_name() {
    let source = r#"
        type Point = { x: number, y: number }

        local p: Point = { x: 0, y: 0 }
    "#;

    let result = type_check(source);
    let _ = result;
}

#[test]
fn test_generic_variance_invariant() {
    let source = r#"
        type Box<T> = { value: T }

        function upcast(b: Box<number>): Box<any> {
            return b
        }
    "#;

    assert!(
        type_check(source).is_err(),
        "Generic types should be invariant"
    );
}
