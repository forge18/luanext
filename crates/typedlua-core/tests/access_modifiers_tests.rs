use typedlua_core::di::DiContainer;

fn type_check(source: &str) -> Result<(), String> {
    let mut container = DiContainer::test_default();
    container.compile_with_stdlib(source)?;
    Ok(())
}

#[test]
fn test_class_with_public_members() {
    let source = r#"
        class Point {
            public x: number = 0
            public y: number = 0
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Class with public members should type-check successfully"
    );
}

#[test]
fn test_class_with_private_members() {
    let source = r#"
        class Point {
            private x: number = 0
            private y: number = 0
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Class with private members should type-check successfully"
    );
}

#[test]
fn test_class_with_protected_members() {
    let source = r#"
        class Point {
            protected x: number = 0
            protected y: number = 0
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Class with protected members should type-check successfully"
    );
}

#[test]
fn test_class_with_mixed_access_modifiers() {
    let source = r#"
        class BankAccount {
            private balance: number = 0
            protected owner: string = "unknown"
            public account_id: string = "12345"
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Class with mixed access modifiers should type-check successfully"
    );
}

#[test]
fn test_private_access_from_outside_fails() {
    let source = r#"
        class Secret {
            private password: string = "secret"
        }

        const s = new Secret()
        const try = s.password
    "#;

    assert!(
        type_check(source).is_err(),
        "Private member should not be accessible from outside"
    );
}

#[test]
fn test_protected_access_from_outside_fails() {
    let source = r#"
        class Base {
            protected data: number = 42
        }

        class Derived extends Base {
            public get_data(): number {
                return self.data
            }
        }

        const d = new Derived()
        const try = d.data
    "#;

    assert!(
        type_check(source).is_err(),
        "Protected member should not be accessible from outside class hierarchy"
    );
}

#[test]
fn test_public_access_works() {
    let source = r#"
        class Counter {
            public value: number = 0
        }

        const c = new Counter()
        const v = c.value
    "#;

    assert!(
        type_check(source).is_ok(),
        "Public member should be accessible from outside"
    );
}

#[test]
fn test_private_access_within_class() {
    let source = r#"
        class Calculator {
            private result: number = 0

            public add(n: number) {
                self.result = self.result + n
            }

            public get_result(): number {
                return self.result
            }
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Private member should be accessible within class methods"
    );
}

#[test]
fn test_protected_access_in_subclass() {
    let source = r#"
        class Base {
            protected value: number = 10
        }

        class Derived extends Base {
            public double(): number {
                return self.value * 2
            }
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Protected member should be accessible in subclass methods"
    );
}

#[test]
fn test_access_modifier_with_inheritance() {
    let source = r#"
        class Animal {
            public name: string
            protected age: number
            private id: string

            constructor(name: string, age: number) {
                self.name = name
                self.age = age
                self.id = name .. "_" .. tostring(age)
            }
        }

        class Dog extends Animal {
            public breed: string

            constructor(name: string, age: number, breed: string) {
                super(name, age)
                self.breed = breed
            }
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Inheritance with access modifiers should work"
    );
}

#[test]
fn test_static_member_access() {
    let source = r#"
        class MathUtils {
            public static PI: number = 3.14159
            protected static version: string = "1.0"
            private static instance: MathUtils = new MathUtils()

            public static get_pi(): number {
                return self.PI
            }
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Static members with access modifiers should work"
    );
}

#[test]
fn test_readonly_modifier() {
    let source = r#"
        class Config {
            public readonly api_key: string = "abc123"
        }

        const c = new Config()
        const key = c.api_key
    "#;

    assert!(
        type_check(source).is_ok(),
        "Readonly modifier should allow reading"
    );
}
