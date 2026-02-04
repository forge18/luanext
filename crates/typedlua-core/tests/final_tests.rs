use typedlua_core::di::DiContainer;

fn type_check(source: &str) -> Result<(), String> {
    let mut container = DiContainer::test_default();
    container.compile_with_stdlib(source)?;
    Ok(())
}

#[test]
fn test_final_class_cannot_be_extended() {
    let source = r#"
        final class Animal {
            speak(): void {
                print("...")
            }
        }

        class Dog extends Animal {
            speak(): void {
                print("Woof!")
            }
        }
    "#;

    let result = type_check(source);
    assert!(result.is_err(), "Extending a final class should fail");
    assert!(
        result.unwrap_err().contains("Cannot extend final class"),
        "Error message should mention final class"
    );
}

#[test]
fn test_final_method_cannot_be_overridden() {
    let source = r#"
        class Animal {
            final speak(): void {
                print("...")
            }
        }

        class Dog extends Animal {
            override speak(): void {
                print("Woof!")
            }
        }
    "#;

    let result = type_check(source);
    assert!(result.is_err(), "Overriding a final method should fail");
}

#[test]
fn test_final_class_with_methods() {
    let source = r#"
        final class MathUtils {
            public static add(a: number, b: number): number {
                return a + b
            }

            public static multiply(a: number, b: number): number {
                return a * b
            }
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Final class with methods should compile");
}

#[test]
fn test_final_class_with_fields() {
    let source = r#"
        final class Config {
            public api_key: string = "secret"
            public timeout: number = 30
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Final class with fields should compile");
}

#[test]
fn test_final_class_instantiation() {
    let source = r#"
        final class Point {
            public x: number
            public y: number

            constructor(x: number, y: number) {
                self.x = x
                self.y = y
            }
        }

        const p = new Point(1, 2)
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Instantiating final class should work");
}

#[test]
fn test_final_class_can_implement_interface() {
    let source = r#"
        interface Drawable {
            draw(): void
        }

        final class Circle implements Drawable {
            public draw(): void {
                print("circle")
            }
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Final class can implement interface");
}

#[test]
fn test_final_method_basic() {
    let source = r#"
        class Base {
            public final compute(x: number): number {
                return x * 2
            }
        }

        class Derived extends Base {
            public otherMethod(): number {
                return self.compute(5)
            }
        }
    "#;

    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Final method in non-final class should work"
    );
}

#[test]
fn test_final_override_final_fails() {
    let source = r#"
        class Base {
            public final method(): void {
            }
        }

        class Derived extends Base {
            override final method(): void {
            }
        }
    "#;

    let result = type_check(source);
    assert!(result.is_err(), "override final should fail");
}

#[test]
fn test_final_class_with_constructor() {
    let source = r#"
        final class Singleton {
            private static instance: Singleton | nil = nil

            private constructor() {
            }

            public static getInstance(): Singleton {
                if self.instance == nil {
                    self.instance = new Singleton()
                }
                return self.instance
            }
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Final class with constructor should work");
}

#[test]
fn test_final_static_method() {
    let source = r#"
        final class MathOps {
            public static PI: number = 3.14159

            public static circleArea(r: number): number {
                return self.PI * r * r
            }
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Final class with static method should work");
}

#[test]
fn test_final_abstract_method_fails() {
    let source = r#"
        abstract class Base {
            abstract final method(): void
        }
    "#;

    let result = type_check(source);
    assert!(result.is_err(), "abstract final should fail");
}

#[test]
fn test_final_class_can_have_getter_setter() {
    let source = r#"
        final class Counter {
            private _value: number = 0

            public get value(): number {
                return self._value
            }

            public set value(v: number) {
                self._value = v
            }
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Final class with getter/setter should work");
}

#[test]
fn test_final_class_extends_non_final() {
    let source = r#"
        class Base {
        }

        final class Derived extends Base {
            public value: number = 0
        }
    "#;

    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Final class extending non-final should work"
    );
}

#[test]
fn test_non_final_class_extends_final_fails() {
    let source = r#"
        final class Base {
        }

        class Derived extends Base {
        }
    "#;

    let result = type_check(source);
    assert!(result.is_err(), "Non-final class cannot extend final");
}

#[test]
fn test_final_method_called_from_subclass() {
    let source = r#"
        class Base {
            public final getValue(): number {
                return 42
            }
        }

        class Derived extends Base {
            public useBaseValue(): number {
                return self.getValue()
            }
        }
    "#;

    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Calling final method from subclass should work"
    );
}

#[test]
fn test_final_class_with_generic() {
    let source = r#"
        final class Container<T> {
            public value: T

            constructor(v: T) {
                self.value = v
            }

            public get(): T {
                return self.value
            }
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Final generic class should work");
}

#[test]
fn test_final_class_properties() {
    let source = r#"
        final class ReadOnlyPoint {
            public readonly x: number
            public readonly y: number

            constructor(x: number, y: number) {
                self.x = x
                self.y = y
            }
        }
    "#;

    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Final class with readonly properties should work"
    );
}

#[test]
fn test_final_class_private_member() {
    let source = r#"
        final class Encapsulated {
            private _secret: string = "hidden"

            public reveal(): string {
                return self._secret
            }
        }
    "#;

    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Final class with private members should work"
    );
}

#[test]
fn test_final_class_protected_member() {
    let source = r#"
        final class Base {
            protected _value: number = 0
        }

        class Derived extends Base {
            public getValue(): number {
                return self._value
            }
        }
    "#;

    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Final class with protected members should work"
    );
}
