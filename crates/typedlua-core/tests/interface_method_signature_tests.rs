use typedlua_core::di::DiContainer;

fn type_check(source: &str) -> Result<(), String> {
    let mut container = DiContainer::test_default();
    container.compile(source)?;
    Ok(())
}

#[test]
fn test_interface_method_correct_signature() {
    let source = r#"
        interface Calculator {
            add(a: number, b: number): number
        }

        class BasicCalculator implements Calculator {
            add(a: number, b: number): number {
                return 0
            }
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Correct method signature should pass"
    );
}

#[test]
fn test_interface_method_wrong_param_count() {
    let source = r#"
        interface Calculator {
            add(a: number, b: number): number
        }

        class BasicCalculator implements Calculator {
            add(a: number): number {
                return 0
            }
        }
    "#;

    assert!(type_check(source).is_err(), "Wrong param count should fail");
}

#[test]
fn test_interface_method_wrong_param_type() {
    let source = r#"
        interface Calculator {
            add(a: number, b: number): number
        }

        class BasicCalculator implements Calculator {
            add(a: number, b: string): number {
                return 0
            }
        }
    "#;

    assert!(type_check(source).is_err(), "Wrong param type should fail");
}

#[test]
fn test_interface_method_wrong_return_type() {
    let source = r#"
        interface Calculator {
            add(a: number, b: number): number
        }

        class BasicCalculator implements Calculator {
            add(a: number, b: number): string {
                return ""
            }
        }
    "#;

    assert!(type_check(source).is_err(), "Wrong return type should fail");
}

#[test]
fn test_interface_method_extra_params() {
    let source = r#"
        interface Calculator {
            add(a: number, b: number): number
        }

        class BasicCalculator implements Calculator {
            add(a: number, b: number, c: number): number {
                return a + b + c
            }
        }
    "#;

    assert!(type_check(source).is_err(), "Extra params should fail");
}

#[test]
fn test_interface_method_optional_params() {
    let source = r#"
        interface Calculator {
            add(a: number, b?: number): number
        }

        class BasicCalculator implements Calculator {
            add(a: number, b?: number): number {
                return a + (b ?? 0)
            }
        }
    "#;

    assert!(type_check(source).is_ok(), "Optional params should pass");
}

#[test]
fn test_interface_getter() {
    let source = r#"
        interface ValueHolder {
            value: number
            getValue(): number
        }

        class Holder implements ValueHolder {
            value: number = 0
            getValue(): number {
                return self.value
            }
        }
    "#;

    assert!(type_check(source).is_ok(), "Interface getter should pass");
}

#[test]
fn test_interface_setter() {
    let source = r#"
        interface Writable {
            value: number
            setValue(v: number): void
        }

        class Writer implements Writable {
            value: number = 0
            setValue(v: number): void {
                self.value = v
            }
        }
    "#;

    assert!(type_check(source).is_ok(), "Interface setter should pass");
}

#[test]
fn test_interface_with_generic_method() {
    let source = r#"
        interface Container {
            getItem<T>(key: string): T | nil
            setItem<T>(key: string, value: T): void
        }

        class MapContainer implements Container {
            items: { [string]: any } = {}

            getItem<T>(key: string): T | nil {
                return self.items[key] as T
            }

            setItem<T>(key: string, value: T): void {
                self.items[key] = value
            }
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Interface with generic method should pass"
    );
}

#[test]
fn test_interface_method_covariant_return() {
    let source = r#"
        interface Base {
            getObject(): { value: number }
        }

        interface Derived {
            getObject(): { value: number; extra: string }
        }

        class Impl implements Derived {
            getObject(): { value: number; extra: string } {
                return { value: 0, extra: "" }
            }
        }
    "#;

    assert!(type_check(source).is_ok(), "Covariant return should pass");
}

#[test]
fn test_interface_static_method() {
    let source = r#"
        interface Factory {
            static create(): Factory
        }

        class MyFactory implements Factory {
            static create(): Factory {
                return new MyFactory()
            }
        }
    "#;

    assert!(
        type_check(source).is_err(),
        "Static method in interface should fail"
    );
}

#[test]
fn test_interface_method_rest_params() {
    let source = r#"
        interface Summer {
            sum(...nums: number[]): number
        }

        class BasicSummer implements Summer {
            sum(...nums: number[]): number {
                let s = 0
                for n in nums {
                    s = s + n
                }
                return s
            }
        }
    "#;

    assert!(type_check(source).is_ok(), "Rest params should pass");
}

#[test]
fn test_interface_method_this_param() {
    let source = r#"
        interface Counter {
            increment(self): void
        }

        class MyCounter implements Counter {
            count: number = 0
            increment(self): void {
                self.count = self.count + 1
            }
        }
    "#;

    assert!(type_check(source).is_ok(), "This param should pass");
}

#[test]
fn test_interface_multiple_methods() {
    let source = r#"
        interface MultiOps {
            add(a: number, b: number): number
            sub(a: number, b: number): number
            mul(a: number, b: number): number
        }

        class Calculator implements MultiOps {
            add(a: number, b: number): number { return a + b }
            sub(a: number, b: number): number { return a - b }
            mul(a: number, b: number): number { return a * b }
        }
    "#;

    assert!(type_check(source).is_ok(), "Multiple methods should pass");
}

#[test]
fn test_interface_method_type_alias() {
    let source = r#"
        type BinaryOp = (a: number, b: number) => number

        interface Calculator {
            compute: BinaryOp
        }

        class SimpleCalc implements Calculator {
            compute: BinaryOp = (a, b) => a + b
        }
    "#;

    assert!(type_check(source).is_ok(), "Method type alias should pass");
}

#[test]
fn test_interface_async_method() {
    let source = r#"
        interface Fetcher {
            async fetch(url: string): string
        }

        class HttpFetcher implements Fetcher {
            async fetch(url: string): string {
                return ""
            }
        }
    "#;

    assert!(type_check(source).is_ok(), "Async method should pass");
}

#[test]
fn test_interface_getter_setter() {
    let source = r#"
        interface PropertyHolder {
            value: number
            getValue(): number
            setValue(v: number): void
        }

        class Holder implements PropertyHolder {
            value: number = 0
            getValue(): number { return self.value }
            setValue(v: number): void { self.value = v }
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Getter/setter combination should pass"
    );
}

#[test]
fn test_interface_constructor_signature() {
    let source = r#"
        interface Constructable {
            new(x: number): Constructable
        }

        class MyClass implements Constructable {
            x: number
            constructor(x: number) {
                self.x = x
            }
        }
    "#;

    assert!(
        type_check(source).is_ok(),
        "Constructor signature should pass"
    );
}
