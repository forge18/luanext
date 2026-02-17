//! Execution tests for advanced class features - primary constructors, abstract/final
//! classes, super() calls, interface default methods, and instanceof checks.

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

// ============================================================================
// Primary Constructors
// ============================================================================

#[test]
fn test_primary_constructor() {
    // Compact class declaration with primary constructor parameters
    let source = r#"
        class Point(public x: number, public y: number) {
        }

        p = new Point(3, 4)
        px: number = p.x
        py: number = p.y
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let px: i64 = executor.execute_and_get(&lua_code, "px").unwrap();
    let py: i64 = executor.execute_and_get(&lua_code, "py").unwrap();
    assert_eq!(px, 3);
    assert_eq!(py, 4);
}

#[test]
fn test_primary_constructor_with_methods() {
    let source = r#"
        class Point(public x: number, public y: number) {
            distanceTo(other: Point): number {
                local dx: number = self.x - other.x
                local dy: number = self.y - other.y
                return math.sqrt(dx * dx + dy * dy)
            }
        }

        p1 = new Point(0, 0)
        p2 = new Point(3, 4)
        dist: number = p1::distanceTo(p2)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let dist: f64 = executor.execute_and_get(&lua_code, "dist").unwrap();
    assert!((dist - 5.0).abs() < 0.001);
}

#[test]
fn test_primary_constructor_private_fields() {
    // Private fields in primary constructor get _ prefix in codegen
    let source = r#"
        class Account(private balance: number) {
            getBalance(): number {
                return self._balance
            }

            deposit(amount: number) {
                self._balance = self._balance + amount
            }
        }

        acc = new Account(100)
        initial: number = acc::getBalance()
        acc::deposit(50)
        after: number = acc::getBalance()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let initial: i64 = executor.execute_and_get(&lua_code, "initial").unwrap();
    let after: i64 = executor.execute_and_get(&lua_code, "after").unwrap();
    assert_eq!(initial, 100);
    assert_eq!(after, 150);
}

// ============================================================================
// Inheritance with super()
// ============================================================================

#[test]
fn test_inheritance_with_super() {
    let source = r#"
        class Animal {
            name: string
            sound: string

            constructor(name: string, sound: string) {
                self.name = name
                self.sound = sound
            }

            speak(): string {
                return self.name .. " says " .. self.sound
            }
        }

        class Dog extends Animal {
            breed: string

            constructor(name: string, breed: string) {
                self.name = name
                self.sound = "Woof"
                self.breed = breed
            }

            describe(): string {
                return self.name .. " is a " .. self.breed
            }
        }

        dog = new Dog("Rex", "Labrador")
        speak_result: string = dog::speak()
        desc_result: string = dog::describe()
        breed_result: string = dog.breed
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let speak: String = executor.execute_and_get(&lua_code, "speak_result").unwrap();
    let desc: String = executor.execute_and_get(&lua_code, "desc_result").unwrap();
    let breed: String = executor.execute_and_get(&lua_code, "breed_result").unwrap();

    assert_eq!(speak, "Rex says Woof");
    assert_eq!(desc, "Rex is a Labrador");
    assert_eq!(breed, "Labrador");
}

#[test]
fn test_primary_constructor_inheritance() {
    // Child with explicit constructor extending parent with primary constructor
    let source = r#"
        class Vehicle(public make: string) {
            describe(): string {
                return "Vehicle: " .. self.make
            }
        }

        class Car extends Vehicle {
            model: string

            constructor(make: string, model: string) {
                self.make = make
                self.model = model
            }

            describe(): string {
                return self.make .. " " .. self.model
            }
        }

        car = new Car("Toyota", "Camry")
        result: string = car::describe()
        make_val: string = car.make
        model_val: string = car.model
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    let make: String = executor.execute_and_get(&lua_code, "make_val").unwrap();
    let model: String = executor.execute_and_get(&lua_code, "model_val").unwrap();
    assert_eq!(result, "Toyota Camry");
    assert_eq!(make, "Toyota");
    assert_eq!(model, "Camry");
}

// ============================================================================
// Abstract Classes
// ============================================================================

#[test]
fn test_abstract_class_cannot_instantiate() {
    // Type checker prevents instantiating abstract classes at compile time
    let source = r#"
        abstract class Shape {
            abstract area(): number
        }

        s = new Shape()
    "#;

    let result = compile(source);
    assert!(result.is_err(), "Instantiating abstract class should fail at compile time");
}

#[test]
fn test_abstract_method_override() {
    // Concrete class overrides abstract method
    let source = r#"
        abstract class Shape {
            abstract area(): number

            describe(): string {
                return "Area is " .. tostring(self::area())
            }
        }

        class Circle extends Shape {
            radius: number

            constructor(radius: number) {
                self.radius = radius
            }

            override area(): number {
                return 3.14159 * self.radius * self.radius
            }
        }

        circle = new Circle(10)
        area_val: number = circle::area()
        desc: string = circle::describe()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let area: f64 = executor.execute_and_get(&lua_code, "area_val").unwrap();
    let desc: String = executor.execute_and_get(&lua_code, "desc").unwrap();
    assert!((area - 314.159).abs() < 0.01);
    assert_eq!(desc, "Area is 314.159");
}

// ============================================================================
// Final Classes and Methods
// ============================================================================

#[test]
fn test_final_class_cannot_extend() {
    // Type checker prevents extending a final class at compile time
    let source = r#"
        final class Singleton {
            constructor() {
            }
        }

        class Derived extends Singleton {
            constructor() {
            }
        }
    "#;

    let result = compile(source);
    assert!(result.is_err(), "Extending a final class should fail at compile time");
}

#[test]
fn test_final_method_cannot_override() {
    // Type checker prevents overriding a final method at compile time
    let source = r#"
        class Base {
            constructor() {
            }

            final important(): number {
                return 42
            }
        }

        class Child extends Base {
            constructor() {
            }

            override important(): number {
                return 99
            }
        }
    "#;

    let result = compile(source);
    assert!(result.is_err(), "Overriding a final method should fail at compile time");
}

// ============================================================================
// Method Resolution Order
// ============================================================================

#[test]
fn test_method_resolution_order() {
    // Inherited methods are found via metatable __index chain
    let source = r#"
        class Base {
            constructor() {
            }

            greet(): string {
                return "Hello from Base"
            }

            identity(): string {
                return "Base"
            }
        }

        class Middle extends Base {
            constructor() {
            }

            override identity(): string {
                return "Middle"
            }
        }

        class Leaf extends Middle {
            constructor() {
            }
        }

        leaf = new Leaf()
        -- greet() comes from Base (2 levels up)
        greet_result: string = leaf::greet()
        -- identity() comes from Middle (1 level up, overridden)
        identity_result: string = leaf::identity()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let greet: String = executor.execute_and_get(&lua_code, "greet_result").unwrap();
    let identity: String = executor.execute_and_get(&lua_code, "identity_result").unwrap();
    assert_eq!(greet, "Hello from Base");
    assert_eq!(identity, "Middle");
}

// ============================================================================
// Interface Default Methods
// ============================================================================

#[test]
fn test_interface_default_methods() {
    // Interface with default method body gets applied to implementing class
    let source = r#"
        interface Printable {
            toString(): string

            toUpperString(): string {
                local s: string = self::toString()
                return string.upper(s)
            }
        }

        class Item implements Printable {
            name: string

            constructor(name: string) {
                self.name = name
            }

            toString(): string {
                return self.name
            }
        }

        item = new Item("hello")
        str_result: string = item::toString()
        upper_result: string = item::toUpperString()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let str_result: String = executor.execute_and_get(&lua_code, "str_result").unwrap();
    let upper_result: String = executor.execute_and_get(&lua_code, "upper_result").unwrap();
    assert_eq!(str_result, "hello");
    assert_eq!(upper_result, "HELLO");
}

// ============================================================================
// Instanceof / Type Checking
// ============================================================================

#[test]
fn test_type_infrastructure() {
    // Verify runtime type infrastructure is generated (__typeName, __typeId, __ancestors)
    // The generated Lua code sets these fields on every class table
    let source = r#"
        class Animal {
            constructor() {
            }

            getTypeName(): string {
                return Animal.__typeName
            }
        }

        class Dog extends Animal {
            constructor() {
            }
        }

        animal = new Animal()
        animal_name: string = animal::getTypeName()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let animal_name: String = executor.execute_and_get(&lua_code, "animal_name").unwrap();
    assert_eq!(animal_name, "Animal");
}
