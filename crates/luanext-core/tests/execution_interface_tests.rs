//! Execution tests for interfaces: default methods, class implements, structural typing.
//!
//! Syntax reference:
//! - Interface:         `interface Greeter { greet(): string }`
//! - Default body:      `interface Greeter { greet(): string { return "Hi" } }`
//! - Class implements:  `class User implements Greeter { ... }`
//! - Method call:       `obj::method()` (uses `::` operator)

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

#[test]
fn test_class_implements_interface() {
    // Class implements interface and provides the required method
    let source = r#"
        interface Greeter {
            greet(): string
        }
        class User implements Greeter {
            name: string
            constructor(n: string) {
                self.name = n
            }
            greet(): string {
                return "Hello, " .. self.name
            }
        }
        const u = new User("Alice")
        result: string = u::greet()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "Hello, Alice");
}

#[test]
fn test_interface_default_method() {
    // Interface default method is inherited by implementing class
    let source = r#"
        interface Greeter {
            name: string
            greet(): string {
                return "Hello, " .. self.name
            }
        }
        class User implements Greeter {
            name: string
            constructor(n: string) {
                self.name = n
            }
        }
        const u = new User("Bob")
        result: string = u::greet()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "Hello, Bob");
}

#[test]
fn test_interface_default_override() {
    // Class can override an interface default method
    let source = r#"
        interface Greeter {
            greet(): string {
                return "Hi"
            }
        }
        class FormalGreeter implements Greeter {
            greet(): string {
                return "Good day"
            }
        }
        const f = new FormalGreeter()
        result: string = f::greet()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "Good day");
}

#[test]
fn test_interface_type_erased() {
    // Interface declaration alone emits no runtime code - only default methods matter
    let source = r#"
        interface Shape {
            area(): number
        }
        class Square implements Shape {
            side: number
            constructor(s: number) {
                self.side = s
            }
            area(): number {
                return self.side * self.side
            }
        }
        const sq = new Square(5)
        result: number = sq::area()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 25);
}

#[test]
fn test_interface_multiple_implementors() {
    // Two classes implementing the same interface both work correctly
    let source = r#"
        interface Animal {
            speak(): string
        }
        class Dog implements Animal {
            speak(): string {
                return "Woof"
            }
        }
        class Cat implements Animal {
            speak(): string {
                return "Meow"
            }
        }
        const d = new Dog()
        const c = new Cat()
        r_dog: string = d::speak()
        r_cat: string = c::speak()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let r_dog: String = executor.execute_and_get(&lua_code, "r_dog").unwrap();
    let r_cat: String = executor.execute_and_get(&lua_code, "r_cat").unwrap();
    assert_eq!(r_dog, "Woof");
    assert_eq!(r_cat, "Meow");
}

#[test]
fn test_interface_default_method_uses_self() {
    // Default method uses self.field to compute result
    let source = r#"
        interface Counter {
            count: number
            doubled(): number {
                return self.count * 2
            }
        }
        class MyCounter implements Counter {
            count: number
            constructor(n: number) {
                self.count = n
            }
        }
        const c = new MyCounter(7)
        result: number = c::doubled()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 14);
}

#[test]
fn test_interface_with_multiple_methods() {
    // Interface with two methods - both callable on implementing class
    let source = r#"
        interface Formatter {
            prefix(): string {
                return "[INFO] "
            }
            format(msg: string): string {
                return self::prefix() .. msg
            }
        }
        class Logger implements Formatter {
        }
        const log = new Logger()
        result: string = log::format("hello")
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "[INFO] hello");
}

#[test]
fn test_interface_method_as_function_arg() {
    // Object satisfying interface can be passed to a function expecting the interface
    let source = r#"
        interface Printable {
            display(): string
        }
        class Point implements Printable {
            x: number
            y: number
            constructor(x: number, y: number) {
                self.x = x
                self.y = y
            }
            display(): string {
                return "(" .. self.x .. ", " .. self.y .. ")"
            }
        }
        function show(p: Printable): string {
            return p::display()
        }
        const pt = new Point(3, 4)
        result: string = show(pt)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "(3, 4)");
}
