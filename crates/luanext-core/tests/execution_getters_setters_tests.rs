//! Execution tests for getters and setters - class `get`/`set` declarations
//! that compile to `get_name()` and `set_name(value)` methods in Lua.

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

// ============================================================================
// Basic Getters
// ============================================================================

#[test]
fn test_basic_getter() {
    let source = r#"
        class Circle {
            radius: number

            constructor(radius: number) {
                self.radius = radius
            }

            get area(): number {
                return 3.14159 * self.radius * self.radius
            }
        }

        c = new Circle(5)
        result: number = c::get_area()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let result: f64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert!((result - 78.53975).abs() < 0.01);
}

// ============================================================================
// Basic Setters
// ============================================================================

#[test]
fn test_basic_setter() {
    let source = r#"
        class Container {
            value: number

            constructor() {
                self.value = 0
            }

            set content(v: number) {
                self.value = v
            }
        }

        c = new Container()
        before: number = c.value
        c::set_content(42)
        after: number = c.value
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let before: i64 = executor.execute_and_get(&lua_code, "before").unwrap();
    let after: i64 = executor.execute_and_get(&lua_code, "after").unwrap();
    assert_eq!(before, 0);
    assert_eq!(after, 42);
}

// ============================================================================
// Getter + Setter Pairs
// ============================================================================

#[test]
fn test_getter_setter_pair() {
    let source = r#"
        class Box {
            _width: number

            constructor(w: number) {
                self._width = w
            }

            get width(): number {
                return self._width
            }

            set width(w: number) {
                self._width = w
            }
        }

        b = new Box(10)
        initial: number = b::get_width()
        b::set_width(25)
        updated: number = b::get_width()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let initial: i64 = executor.execute_and_get(&lua_code, "initial").unwrap();
    let updated: i64 = executor.execute_and_get(&lua_code, "updated").unwrap();
    assert_eq!(initial, 10);
    assert_eq!(updated, 25);
}

#[test]
fn test_getter_computed_value() {
    // Getter that computes a derived value (Fahrenheit from Celsius)
    let source = r#"
        class Temperature {
            celsius: number

            constructor(c: number) {
                self.celsius = c
            }

            get fahrenheit(): number {
                return self.celsius * 9 / 5 + 32
            }

            set fahrenheit(f: number) {
                self.celsius = (f - 32) * 5 / 9
            }
        }

        temp = new Temperature(100)
        boiling_f: number = temp::get_fahrenheit()

        temp::set_fahrenheit(32)
        freezing_c: number = temp.celsius
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let boiling_f: f64 = executor.execute_and_get(&lua_code, "boiling_f").unwrap();
    let freezing_c: f64 = executor.execute_and_get(&lua_code, "freezing_c").unwrap();
    assert!((boiling_f - 212.0).abs() < 0.01);
    assert!((freezing_c - 0.0).abs() < 0.01);
}

#[test]
fn test_setter_with_validation_logic() {
    // Setter that clamps the value to a valid range
    let source = r#"
        class Percentage {
            _value: number

            constructor() {
                self._value = 0
            }

            get value(): number {
                return self._value
            }

            set value(v: number) {
                if v < 0 then
                    self._value = 0
                elseif v > 100 then
                    self._value = 100
                else
                    self._value = v
                end
            }
        }

        p = new Percentage()
        p::set_value(50)
        normal: number = p::get_value()

        p::set_value(150)
        clamped_high: number = p::get_value()

        p::set_value(-10)
        clamped_low: number = p::get_value()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let normal: i64 = executor.execute_and_get(&lua_code, "normal").unwrap();
    let clamped_high: i64 = executor.execute_and_get(&lua_code, "clamped_high").unwrap();
    let clamped_low: i64 = executor.execute_and_get(&lua_code, "clamped_low").unwrap();
    assert_eq!(normal, 50);
    assert_eq!(clamped_high, 100);
    assert_eq!(clamped_low, 0);
}

// ============================================================================
// Static Getters
// ============================================================================

#[test]
fn test_getter_with_string_formatting() {
    // Getter that formats internal state as a string
    let source = r#"
        class Person {
            first: string
            last: string

            constructor(first: string, last: string) {
                self.first = first
                self.last = last
            }

            get fullName(): string {
                return self.first .. " " .. self.last
            }
        }

        p = new Person("John", "Doe")
        name: string = p::get_fullName()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let name: String = executor.execute_and_get(&lua_code, "name").unwrap();
    assert_eq!(name, "John Doe");
}

// ============================================================================
// Multiple Getters/Setters
// ============================================================================

#[test]
fn test_multiple_getters_setters() {
    let source = r#"
        class Rectangle {
            _width: number
            _height: number

            constructor(w: number, h: number) {
                self._width = w
                self._height = h
            }

            get width(): number {
                return self._width
            }

            set width(w: number) {
                self._width = w
            }

            get height(): number {
                return self._height
            }

            set height(h: number) {
                self._height = h
            }

            get area(): number {
                return self._width * self._height
            }

            get perimeter(): number {
                return 2 * (self._width + self._height)
            }
        }

        r = new Rectangle(10, 5)
        area1: number = r::get_area()
        peri1: number = r::get_perimeter()

        r::set_width(20)
        r::set_height(10)
        area2: number = r::get_area()
        peri2: number = r::get_perimeter()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let area1: i64 = executor.execute_and_get(&lua_code, "area1").unwrap();
    let peri1: i64 = executor.execute_and_get(&lua_code, "peri1").unwrap();
    let area2: i64 = executor.execute_and_get(&lua_code, "area2").unwrap();
    let peri2: i64 = executor.execute_and_get(&lua_code, "peri2").unwrap();

    assert_eq!(area1, 50);
    assert_eq!(peri1, 30);
    assert_eq!(area2, 200);
    assert_eq!(peri2, 60);
}

// ============================================================================
// Inheritance with Getters/Setters
// ============================================================================

#[test]
fn test_getter_setter_with_inheritance() {
    let source = r#"
        class Shape {
            _name: string

            constructor(name: string) {
                self._name = name
            }

            get name(): string {
                return self._name
            }
        }

        class Square extends Shape {
            _side: number

            constructor(side: number) {
                self._name = "Square"
                self._side = side
            }

            get side(): number {
                return self._side
            }

            get area(): number {
                return self._side * self._side
            }
        }

        sq = new Square(7)
        name_val: string = sq::get_name()
        side_val: number = sq::get_side()
        area_val: number = sq::get_area()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let name_val: String = executor.execute_and_get(&lua_code, "name_val").unwrap();
    let side_val: i64 = executor.execute_and_get(&lua_code, "side_val").unwrap();
    let area_val: i64 = executor.execute_and_get(&lua_code, "area_val").unwrap();

    assert_eq!(name_val, "Square");
    assert_eq!(side_val, 7);
    assert_eq!(area_val, 49);
}
