//! Execution tests for mutual recursion with classes via forward declarations.
//!
//! When a block contains 2+ class declarations, the codegen emits forward
//! declarations (`local ClassName`) before the class bodies. This allows
//! classes to reference each other regardless of definition order.

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

#[test]
fn test_two_classes_mutual_method_reference() {
    // Class A has a method that creates B, and B has a method that creates A
    let source = r#"
        class A(public name: string) {
            makeB(): B {
                return new B("from_" .. self.name)
            }
        }
        class B(public name: string) {
            makeA(): A {
                return new A("from_" .. self.name)
            }
        }
        const a = new A("alice")
        const b = a::makeB()
        const a2 = b::makeA()
        result: string = a2.name
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "from_from_alice");
}

#[test]
fn test_forward_declaration_doesnt_break_single_class() {
    // A single class should still work normally (no forward declarations emitted)
    let source = r#"
        class Solo(public x: number) {
        }
        const s = new Solo(42)
        result: number = s.x
    "#;

    let lua_code = compile(source).unwrap();
    // Single class should NOT have forward declarations
    let lines: Vec<&str> = lua_code.lines().collect();
    let first_meaningful = lines.iter().find(|l| !l.trim().is_empty()).unwrap();
    assert!(
        first_meaningful.contains("local Solo"),
        "Single class should use 'local ClassName = {{}}', got:\n{lua_code}"
    );

    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_three_way_circular_reference() {
    // A -> B -> C -> A circular reference chain
    let source = r#"
        class NodeA(public value: number) {
            toB(): NodeB {
                return new NodeB(self.value + 1)
            }
        }
        class NodeB(public value: number) {
            toC(): NodeC {
                return new NodeC(self.value + 1)
            }
        }
        class NodeC(public value: number) {
            toA(): NodeA {
                return new NodeA(self.value + 1)
            }
        }
        const a = new NodeA(1)
        const b = a::toB()
        const c = b::toC()
        const a2 = c::toA()
        result: number = a2.value
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 4);
}

#[test]
fn test_forward_declared_class_with_inheritance() {
    // Class B extends A, but A references B — forward declaration required
    // Child must have its own primary constructor to forward args to Base
    let source = r#"
        class Base(public name: string) {
            createChild(): Child {
                return new Child("child_of_" .. self.name)
            }
        }
        class Child(public name: string) extends Base {
        }
        const b = new Base("root")
        const c = b::createChild()
        result: string = c.name
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "child_of_root");
}

#[test]
fn test_forward_declaration_codegen_output() {
    // Verify that forward declarations are emitted before class bodies
    let source = r#"
        class X(public y: number) {
        }
        class Y(public x: number) {
        }
    "#;

    let lua_code = compile(source).unwrap();
    // Forward declarations should appear before class bodies
    let fwd_x = lua_code.find("local X\n");
    let fwd_y = lua_code.find("local Y\n");
    let body_x = lua_code.find("X = {}");
    let body_y = lua_code.find("Y = {}");

    assert!(
        fwd_x.is_some(),
        "Should have forward declaration for X, got:\n{lua_code}"
    );
    assert!(
        fwd_y.is_some(),
        "Should have forward declaration for Y, got:\n{lua_code}"
    );
    assert!(
        body_x.is_some(),
        "Should have class body for X, got:\n{lua_code}"
    );
    assert!(
        body_y.is_some(),
        "Should have class body for Y, got:\n{lua_code}"
    );
    assert!(
        fwd_x.unwrap() < body_x.unwrap(),
        "Forward declaration for X should come before body"
    );
    assert!(
        fwd_y.unwrap() < body_y.unwrap(),
        "Forward declaration for Y should come before body"
    );
}

#[test]
fn test_mixed_classes_and_non_classes() {
    // Block has classes mixed with functions and variables
    let source = r#"
        function helper(): number {
            return 100
        }
        class Foo(public val: number) {
            getBar(): Bar {
                return new Bar(self.val * 2)
            }
        }
        const x = helper()
        class Bar(public val: number) {
        }
        const f = new Foo(5)
        const b = f::getBar()
        result: number = b.val + x
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 110);
}

#[test]
fn test_mutual_factory_methods() {
    // Both classes have factory methods creating the other
    let source = r#"
        class Cat(public name: string) {
            befriend(): Dog {
                return new Dog("buddy_of_" .. self.name)
            }
        }
        class Dog(public name: string) {
            befriend(): Cat {
                return new Cat("pal_of_" .. self.name)
            }
        }
        const cat = new Cat("whiskers")
        const dog = cat::befriend()
        const cat2 = dog::befriend()
        result: string = cat2.name
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "pal_of_buddy_of_whiskers");
}

#[test]
fn test_class_field_type_reference() {
    // Classes reference each other's types in fields and methods
    let source = r#"
        class Container(public value: number) {
            wrap(): Wrapper {
                return new Wrapper(self)
            }
        }
        class Wrapper(public inner: Container) {
            getValue(): number {
                return self.inner.value
            }
        }
        const c = new Container(42)
        const w = c::wrap()
        result: number = w::getValue()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}
