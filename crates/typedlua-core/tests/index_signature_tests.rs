use typedlua_core::di::DiContainer;

fn type_check(source: &str) -> Result<(), String> {
    let mut container = DiContainer::test_default();
    container.compile(source)?;
    Ok(())
}

#[test]
fn test_interface_with_string_index_signature_compatible_properties() {
    let source = r#"
        interface StringMap {
            [key: string]: number
        }

        class NumberMap implements StringMap {
            count: number = 0
            total: number = 0
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "All properties compatible with index signature should pass"
    );
}

#[test]
fn test_interface_with_string_index_signature_incompatible_property() {
    let source = r#"
        interface StringMap {
            [key: string]: number
        }

        class MixedMap implements StringMap {
            count: number = 0
            total: number = 0
            name: string = "test"
        }
    "#;

    assert!(
        type_check(source).is_err(),
        "Incompatible property should fail"
    );
}

#[test]
fn test_index_signature_basic() {
    let source = r#"
        interface StringToNumberMap {
            [key: string]: number
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Basic index signature should pass"
    );
}

#[test]
fn test_index_signature_number_key() {
    let source = r#"
        interface NumberToStringMap {
            [key: number]: string
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Number key index signature should pass"
    );
}

#[test]
fn test_index_signature_with_type_parameter() {
    let source = r#"
        interface Map<K, V> {
            [key: K]: V
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Generic index signature should pass"
    );
}

#[test]
fn test_object_type_with_index_signature() {
    let source = r#"
        type StringDictionary = {
            [key: string]: boolean
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Object type with index signature should pass"
    );
}

#[test]
fn test_class_implementing_index_signature_interface() {
    let source = r#"
        interface Storage {
            [key: string]: any
        }

        class FileStorage implements Storage {
            files: { [string]: any } = {}
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Class implementing index signature should pass"
    );
}

#[test]
fn test_index_signature_assignment() {
    let source = r#"
        const obj: { [key: string]: number } = {}
        obj["a"] = 1
        obj.b = 2
    "#;

    assert!(
        type_check(source).is_ok(),
        "Index signature assignment should pass"
    );
}

#[test]
fn test_nested_index_signature() {
    let source = r#"
        interface Nested {
            [key: string]: { [key: string]: number }
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Nested index signature should pass"
    );
}

#[test]
fn test_index_signature_with_union_type() {
    let source = r#"
        interface Flexible {
            [key: string]: string | number | boolean
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Index signature with union type should pass"
    );
}

#[test]
fn test_index_signature_read() {
    let source = r#"
        const obj: { [key: string]: number } = { a: 1, b: 2 }
        const a = obj.a
        const c = obj["c"]
    "#;

    assert!(
        type_check(source).is_ok(),
        "Index signature read should pass"
    );
}

#[test]
fn test_compatible_index_signature() {
    let source = r#"
        interface Base {
            [key: string]: number
        }

        interface Derived {
            [key: string]: number
            extra: string
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Compatible index signature should pass"
    );
}

#[test]
fn test_index_signature_generic_constraint() {
    let source = r#"
        interface ConstrainedMap<T extends string> {
            [key: T]: number
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Constrained generic index signature should pass"
    );
}

#[test]
fn test_class_with_index_signature_property() {
    let source = r#"
        class Cache {
            public items: { [key: string]: any } = {}

            public get<T>(key: string): T | nil {
                return self.items[key] as T
            }

            public set(key: string, value: any) {
                self.items[key] = value
            }
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Class with index signature property should pass"
    );
}

#[test]
fn test_type_alias_with_index_signature() {
    let source = r#"
        type StringArray = { [index: number]: string }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Type alias with index signature should pass"
    );
}

#[test]
fn test_index_signature_compatibility_contravariant() {
    let source = r#"
        interface WriteOnly {
            [key: string]: number
        }

        const f: (obj: { [key: string]: number }) => void = (obj) => {}
    "#;

    assert!(
        type_check(source).is_ok(),
        "Index signature contravariance should pass"
    );
}

#[test]
fn test_mixed_properties_and_index_signature() {
    let source = r#"
        interface Mixed {
            name: string
            age: number
            [key: string]: string | number
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Mixed properties and index signature should pass"
    );
}

#[test]
fn test_index_signature_optional_property() {
    let source = r#"
        interface WithOptional {
            required: number
            optional?: string
            [key: string]: number | string | nil
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Index signature with optional should pass"
    );
}
