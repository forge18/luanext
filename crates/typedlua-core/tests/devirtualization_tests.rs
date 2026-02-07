use typedlua_core::config::OptimizationLevel;
use typedlua_core::di::DiContainer;

fn compile_with_optimization_level(
    source: &str,
    level: OptimizationLevel,
) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile_with_optimization(source, level)
}

fn compile_with_o3(source: &str) -> Result<String, String> {
    compile_with_optimization_level(source, OptimizationLevel::O3)
}

#[test]
fn test_final_class_devirtualization() {
    let source = r#"
        final class MathOps {
            public static add(a: number, b: number): number {
                return a + b
            }
        }

        const result = MathOps.add(1, 2)
    "#;

    let output = compile_with_o3(source).unwrap();
    println!("Final class O3 output:\n{}", output);
}

#[test]
fn test_sealed_class_devirtualization() {
    let source = r#"
        @sealed
        class Calculator {
            public add(a: number, b: number): number {
                return a + b
            }
        }

        const c = new Calculator()
        const result = c.add(1, 2)
    "#;

    let output = compile_with_o3(source).unwrap();
    println!("Sealed class O3 output:\n{}", output);
}

#[test]
fn test_private_method_direct_call() {
    let source = r#"
        class MyClass {
            private helper(x: number): number {
                return x * 2
            }

            public compute(x: number): number {
                return self.helper(x) + 1
            }
        }

        const m = new MyClass()
        const result = m.compute(5)
    "#;

    let output = compile_with_o3(source).unwrap();
    println!("Private method O3 output:\n{}", output);
}

#[test]
fn test_final_method_devirtualization() {
    let source = r#"
        class Base {
            public final compute(x: number): number {
                return x + 1
            }
        }

        class Derived extends Base {
            public compute(x: number): number {
                return x + 2
            }
        }
    "#;

    let output = compile_with_o3(source).unwrap();
    println!("Final method O3 output:\n{}", output);
}

#[test]
fn test_private_field_access() {
    let source = r#"
        class Counter {
            private _count: number = 0

            public increment(): void {
                self._count = self._count + 1
            }

            public get(): number {
                return self._count
            }
        }

        const c = new Counter()
        c.increment()
        const result = c.get()
    "#;

    let output = compile_with_o3(source).unwrap();
    println!("Private field O3 output:\n{}", output);
}

#[test]
fn test_final_class_with_inheritance() {
    // Final classes cannot be extended - this should produce an error
    let source = r#"
        final class FinalBase {
            public value: number = 42
        }

        class Derived extends FinalBase {
            public other: string = "test"
        }

        const d = new Derived()
        const v = d.value
    "#;

    let result = compile_with_o3(source);
    assert!(
        result.is_err(),
        "Extending a final class should produce an error"
    );
}

#[test]
fn test_private_method_called_multiple_times() {
    let source = r#"
        class Processor {
            private processItem(item: number): number {
                return item * 2 + 1
            }

            public processAll(x: number): number {
                return self.processItem(x) + self.processItem(x + 1)
            }
        }

        const p = new Processor()
        const result = p.processAll(5)
    "#;

    let output = compile_with_o3(source).unwrap();
    println!("Multiple private calls O3 output:\n{}", output);
}

#[test]
fn test_getter_devirtualization() {
    let source = r#"
        class MyClass {
            private _value: number = 0

            public get value(): number {
                return self._value
            }
        }

        const m = new MyClass()
        const v = m.value
    "#;

    let output = compile_with_o3(source).unwrap();
    println!("Getter O3 output:\n{}", output);
}

#[test]
fn test_static_method_direct_call() {
    let source = r#"
        class MathUtils {
            public static double(x: number): number {
                return x * 2
            }
        }

        const result = MathUtils.double(5)
    "#;

    let output = compile_with_o3(source).unwrap();
    println!("Static method O3 output:\n{}", output);
}

#[test]
fn test_private_static_method() {
    let source = r#"
        class Helper {
            private static format(x: number): string {
                return tostring(x)
            }

            public static process(x: number): string {
                return self.format(x)
            }
        }

        const result = Helper.process(42)
    "#;

    let output = compile_with_o3(source).unwrap();
    println!("Private static O3 output:\n{}", output);
}

#[test]
fn test_inline_method_call() {
    let source = r#"
        class Inline {
            public identity(x: number): number {
                return x
            }
        }

        const i = new Inline()
        const a = i.identity(1)
        const b = i.identity(2)
        const c = i.identity(3)
    "#;

    let output = compile_with_o3(source).unwrap();
    println!("Inline method O3 output:\n{}", output);
}

#[test]
fn test_private_getter() {
    let source = r#"
        class MyClass {
            private _data: number = 42

            private get doubled(): number {
                return self._data * 2
            }

            public getValue(): number {
                return self.doubled
            }
        }

        const m = new MyClass()
        const result = m.getValue()
    "#;

    let output = compile_with_o3(source).unwrap();
    println!("Private getter O3 output:\n{}", output);
}

#[test]
fn test_private_setter() {
    let source = r#"
        class MyClass {
            private _value: number = 0

            private set increment(v: number) {
                self._value = self._value + v
            }

            public add(n: number): void {
                self.increment = n
            }

            public get(): number {
                return self._value
            }
        }

        const m = new MyClass()
        m.add(5)
        const result = m.get()
    "#;

    let output = compile_with_o3(source).unwrap();
    println!("Private setter O3 output:\n{}", output);
}

#[test]
fn test_constructor_inlining() {
    let source = r#"
        class Point {
            public x: number
            public y: number

            constructor(x: number, y: number) {
                self.x = x
                self.y = y
            }
        }

        const p = new Point(1, 2)
        const result = p.x + p.y
    "#;

    let output = compile_with_o3(source).unwrap();
    println!("Constructor O3 output:\n{}", output);
}

#[test]
fn test_private_constructor() {
    let source = r#"
        class Singleton {
            private static instance: Singleton | nil = nil

            private constructor() {
            }

            public static getInstance(): Singleton {
                if self.instance == nil then
                    self.instance = new Singleton()
                end
                return self.instance
            }

            public value: number = 42
        }

        const s = Singleton.getInstance()
        const v = s.value
    "#;

    let output = compile_with_o3(source).unwrap();
    println!("Private constructor O3 output:\n{}", output);
}

#[test]
fn test_final_class_method_chain() {
    let source = r#"
        final class Builder {
            public value: number = 0

            public add(n: number): Builder {
                self.value = self.value + n
                return self
            }

            public multiply(n: number): Builder {
                self.value = self.value * n
                return self
            }
        }

        const result = new Builder().add(1).multiply(2).value
    "#;

    let output = compile_with_o3(source).unwrap();
    println!("Method chain O3 output:\n{}", output);
}

#[test]
fn test_static_field_direct_access() {
    let source = r#"
        class Config {
            public static PI: number = 3.14159
            public static DEBUG: boolean = false
        }

        const pi = Config.PI
        const debug = Config.DEBUG
    "#;

    let output = compile_with_o3(source).unwrap();
    println!("Static field O3 output:\n{}", output);
}

#[test]
fn test_private_static_field() {
    let source = r#"
        class Counter {
            private static _count: number = 0

            public static increment(): void {
                self._count = self._count + 1
            }

            public static getCount(): number {
                return self._count
            }
        }

        Counter.increment()
        Counter.increment()
        const result = Counter.getCount()
    "#;

    let output = compile_with_o3(source).unwrap();
    println!("Private static field O3 output:\n{}", output);
}

#[test]
fn test_devirtualization_with_interface() {
    let source = r#"
        interface Drawable {
            draw(): void
        }

        final class Circle implements Drawable {
            public draw(): void {
                const x = 1
            }
        }

        const c: Drawable = new Circle()
        c.draw()
    "#;

    let output = compile_with_o3(source).unwrap();
    println!("Interface O3 output:\n{}", output);
}
