use typedlua_core::di::DiContainer;

fn compile_and_check(source: &str) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile(source)
}

#[test]
fn test_simple_class_declaration() {
    let source = r#"
        class Point {
            x: number
            y: number

            constructor(x: number, y: number) {
                self.x = x
                self.y = y
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Simple class should compile");
}

#[test]
fn test_class_with_methods() {
    let source = r#"
        class Counter {
            private _count: number = 0

            public increment(): void {
                self._count = self._count + 1
            }

            public getCount(): number {
                return self._count
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Class with methods should compile");
}

#[test]
fn test_class_inheritance() {
    let source = r#"
        class Animal {
            public name: string

            constructor(name: string) {
                self.name = name
            }

            public speak(): void {
                print("...")
            }
        }

        class Dog extends Animal {
            constructor() {
                super("Dog")
            }

            public speak(): void {
                print("Woof!")
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Class inheritance should compile");
}

#[test]
fn test_abstract_class() {
    let source = r#"
        abstract class Shape {
            public abstract area(): number
        }

        class Circle extends Shape {
            private radius: number

            constructor(radius: number) {
                self.radius = radius
            }

            public area(): number {
                return 3.14159 * self.radius * self.radius
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Abstract class should compile");
}

#[test]
fn test_interface() {
    let source = r#"
        interface Drawable {
            draw(): void
        }

        class Circle implements Drawable {
            public draw(): void {
                print("circle")
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Interface should compile");
}

#[test]
fn test_interface_extends() {
    let source = r#"
        interface A {
            a(): void
        }

        interface B {
            b(): void
        }

        interface C extends A, B {
            c(): void
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Interface extends should compile");
}

#[test]
fn test_class_implements_multiple_interfaces() {
    let source = r#"
        interface A {
            a(): void
        }

        interface B {
            b(): void
        }

        class MyClass implements A, B {
            public a(): void {
                print("a")
            }

            public b(): void {
                print("b")
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Multiple interfaces should compile");
}

#[test]
fn test_static_methods() {
    let source = r#"
        class MathUtils {
            public static add(a: number, b: number): number {
                return a + b
            }

            public static PI: number = 3.14159
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Static methods should compile");
}

#[test]
fn test_getter_setter() {
    let source = r#"
        class Temperature {
            private _celsius: number = 0

            public get celsius(): number {
                return self._celsius
            }

            public set celsius(v: number) {
                self._celsius = v
            }

            public get fahrenheit(): number {
                return self._celsius * 9 / 5 + 32
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Getter/setter should compile");
}

#[test]
fn test_constructor_overloading() {
    let source = r#"
        class Point {
            public x: number
            public y: number

            constructor(x: number, y: number) {
                self.x = x
                self.y = y
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Constructor should compile");
}

#[test]
fn test_private_constructor() {
    let source = r#"
        class Singleton {
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

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Private constructor should compile");
}

#[test]
fn test_super_call() {
    let source = r#"
        class Base {
            public value: number = 0

            constructor(value: number) {
                self.value = value
            }
        }

        class Derived extends Base {
            public extra: number

            constructor(value: number, extra: number) {
                super(value)
                self.extra = extra
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Super call should compile");
}

#[test]
fn test_property_with_initializer() {
    let source = r#"
        class Config {
            public host: string = "localhost"
            public port: number = 8080
            public debug: boolean = false
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Property initializer should compile");
}

#[test]
fn test_readonly_property() {
    let source = r#"
        class ImmutablePoint {
            public readonly x: number
            public readonly y: number

            constructor(x: number, y: number) {
                self.x = x
                self.y = y
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Readonly property should compile");
}

#[test]
fn test_nested_class() {
    let source = r#"
        class Outer {
            public value: number = 0

            class Inner {
                public innerValue: number = 0

                public getTotal(): number {
                    return self.innerValue
                }
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Nested class should compile");
}

#[test]
fn test_generic_class() {
    let source = r#"
        class Box<T> {
            public value: T

            constructor(value: T) {
                self.value = value
            }

            public get(): T {
                return self.value
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Generic class should compile");
}

#[test]
fn test_class_with_type_parameters() {
    let source = r#"
        class Container<T, U> {
            public first: T
            public second: U

            constructor(first: T, second: U) {
                self.first = first
                self.second = second
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Class with type parameters should compile");
}
