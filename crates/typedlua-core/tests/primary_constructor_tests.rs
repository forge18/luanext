use typedlua_core::di::DiContainer;

fn parse(source: &str) -> Result<(), String> {
    let mut container = DiContainer::test_default();
    container.compile(source)?;
    Ok(())
}

fn type_check(source: &str) -> Result<(), String> {
    let mut container = DiContainer::test_default();
    container.compile(source)?;
    Ok(())
}

fn compile_and_check(source: &str) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile(source)
}

#[test]
fn test_primary_constructor_basic() {
    let source = r#"
        class Point(x: number, y: number) {
            public x = x
            public y = y
        }
    "#;

    assert!(
        parse(source).is_ok(),
        "Basic primary constructor should parse"
    );
}

#[test]
fn test_primary_constructor_with_modifiers() {
    let source = r#"
        class Person(public name: string, private age: number) {
        }
    "#;

    assert!(
        parse(source).is_ok(),
        "Constructor with modifiers should parse"
    );
}

#[test]
fn test_primary_constructor_type_annotations() {
    let source = r#"
        class Container<T>(value: T, size: number) {
            public value = value
            public size = size
        }
    "#;

    assert!(
        parse(source).is_ok(),
        "Constructor with generic type annotation should parse"
    );
}

#[test]
fn test_primary_constructor_default_values() {
    let source = r#"
        class Config(host: string = "localhost", port: number = 8080) {
            public host = host
            public port = port
        }
    "#;

    assert!(
        parse(source).is_ok(),
        "Constructor with default values should parse"
    );
}

#[test]
fn test_primary_constructor_body() {
    let source = r#"
        class Counter(initial: number) {
            public count = initial

            constructor() {
                self.count = self.count + 10
            }

            public increment() {
                self.count = self.count + 1
            }
        }
    "#;

    assert!(parse(source).is_ok(), "Constructor with body should parse");
}

#[test]
fn test_primary_constructor_inheritance() {
    let source = r#"
        class Person(name: string, age: number) {
            public name = name
            public age = age
        }

        class Employee(name: string, age: number, id: string) extends Person(name, age) {
            public id = id
        }
    "#;

    assert!(
        parse(source).is_ok(),
        "Inheritance with primary constructor should parse"
    );
}

#[test]
fn test_primary_constructor_super_call() {
    let source = r#"
        class Base(value: number) {
            public value = value
        }

        class Derived(value: number, extra: string) extends Base(value) {
            public extra = extra
        }
    "#;

    assert!(parse(source).is_ok(), "Super call should parse");
}

#[test]
fn test_primary_constructor_generic() {
    let source = r#"
        class Box<T>(value: T) {
            public value = value
        }
    "#;

    assert!(
        parse(source).is_ok(),
        "Generic primary constructor should parse"
    );
}

#[test]
fn test_primary_constructor_overloads() {
    let source = r#"
        class Point(x: number, y: number) {
            public x = x
            public y = y

            constructor()
            constructor(x: number)
            constructor(x: number, y: number) {
                self.x = x
                self.y = y
            }
        }
    "#;

    assert!(parse(source).is_ok(), "Constructor overloads should parse");
}

#[test]
fn test_primary_constructor_rest_params() {
    let source = r#"
        class Sum(...nums: number[]) {
            public total = 0

            constructor(...nums: number[]) {
                let s = 0
                for n in nums {
                    s = s + n
                }
                self.total = s
            }
        }
    "#;

    assert!(
        parse(source).is_ok(),
        "Constructor with rest params should parse"
    );
}

#[test]
fn test_primary_constructor_property_initializers() {
    let source = r#"
        class Rectangle(width: number, height: number) {
            public width = width
            public height = height
            public area = width * height
        }
    "#;

    assert!(parse(source).is_ok(), "Property initializers should parse");
}

#[test]
fn test_primary_constructor_validation() {
    let source = r#"
        class Positive(value: number) {
            public value = value

            constructor(value: number) {
                if value <= 0 {
                    error("Value must be positive")
                }
                self.value = value
            }
        }
    "#;

    assert!(
        parse(source).is_ok(),
        "Constructor with validation should parse"
    );
}

#[test]
fn test_primary_constructor_complex() {
    let source = r#"
        class BankAccount(
            public owner: string,
            private balance: number = 0,
            currency: string = "USD"
        ) {
            public currency = currency

            public deposit(amount: number) {
                self.balance = self.balance + amount
            }

            public getBalance(): number {
                return self.balance
            }
        }
    "#;

    assert!(parse(source).is_ok(), "Complex constructor should parse");
}

#[test]
fn test_primary_constructor_this_access() {
    let source = r#"
        class Circle(radius: number) {
            public radius = radius
            public diameter = radius * 2

            constructor(radius: number) {
                this.radius = radius
                this.diameter = this.radius * 2
            }
        }
    "#;

    assert!(
        parse(source).is_ok(),
        "Constructor with this access should parse"
    );
}

#[test]
fn test_primary_constructor_type_narrowing() {
    let source = r#"
        class Wrapper<T>(value: T | nil) {
            public value = value

            constructor(value: T | nil) {
                if value == nil {
                    this.value = nil
                } else {
                    this.value = value
                }
            }
        }
    "#;

    assert!(
        parse(source).is_ok(),
        "Constructor with type narrowing should parse"
    );
}

#[test]
fn test_multiple_constructors_with_primary() {
    let source = r#"
        class Builder(size: number) {
            public size = size
            public items: any[] = []

            constructor()
            constructor(size: number, capacity: number) {
                self.size = size
                self.items = []
            }

            public add(item: any) {
                self.items.push(item)
            }
        }
    "#;

    assert!(
        parse(source).is_ok(),
        "Multiple constructors with primary should parse"
    );
}

#[test]
fn test_primary_constructor_function_parameter() {
    let source = r#"
        class Callback(fn: () => void) {
            public fn = fn

            constructor(fn: () => void) {
                this.fn = fn
            }

            public execute() {
                fn()
            }
        }
    "#;

    assert!(
        parse(source).is_ok(),
        "Constructor with function parameter should parse"
    );
}
