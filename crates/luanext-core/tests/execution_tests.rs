//! Execution tests for validating generated Lua code
//!
//! These tests compile LuaNext source code to Lua and then execute it
//! to verify that the generated code produces correct results.

use luanext_test_helpers::compile::{compile, compile_with_stdlib};
use luanext_test_helpers::LuaExecutor;

// ============================================================================
// Category 1: Arithmetic & Literals (6 tests - includes variable scope demo)
// ============================================================================

#[test]
fn test_variable_scoping_patterns() {
    // Demonstrates all four variable declaration patterns
    let source = r#"
        -- Implicit global (type annotation required)
        implicit_global: number = 100

        -- Explicit global (optional, clearer intent)
        global explicit_global: number = 200

        -- Const (immutable, generates local in Lua)
        const CONSTANT_VALUE: number = 42

        -- Local variable in function (demonstrates scoping)
        function calculate(): number {
            local temp: number = 10
            return temp * CONSTANT_VALUE
        }

        result: number = calculate()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    // Verify global variables (implicit and explicit)
    let implicit: i64 = executor.execute_and_get(&lua_code, "implicit_global").unwrap();
    let explicit: i64 = executor.execute_and_get(&lua_code, "explicit_global").unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();

    assert_eq!(implicit, 100);
    assert_eq!(explicit, 200);
    assert_eq!(result, 420); // 10 * 42 (const is used in calculation but not accessible globally)
}

#[test]
fn test_integer_arithmetic() {
    let source = r#"
        x: number = 1 + 2 * 3
        y: number = 10 - x
        z: number = y * 2
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let x: i64 = executor.execute_and_get(&lua_code, "x").unwrap();
    let y: i64 = executor.execute_and_get(&lua_code, "y").unwrap();
    let z: i64 = executor.execute_and_get(&lua_code, "z").unwrap();

    assert_eq!(x, 7);
    assert_eq!(y, 3);
    assert_eq!(z, 6);
}

#[test]
fn test_float_arithmetic() {
    let source = r#"
        x: number = 10.5 / 2
        y: number = 3.14 * 2
        z: number = x + y
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let x: f64 = executor.execute_and_get(&lua_code, "x").unwrap();
    let y: f64 = executor.execute_and_get(&lua_code, "y").unwrap();
    let z: f64 = executor.execute_and_get(&lua_code, "z").unwrap();

    assert!((x - 5.25).abs() < 0.001);
    assert!((y - 6.28).abs() < 0.001);
    assert!((z - 11.53).abs() < 0.001);
}

#[test]
fn test_string_concatenation() {
    let source = r#"
        hello: string = "hello"
        world: string = "world"
        result: string = hello .. " " .. world
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "hello world");
}

#[test]
fn test_boolean_logic() {
    let source = r#"
        a: boolean = true and false
        b: boolean = true or false
        c: boolean = not true
        d: boolean = true and true
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let a: bool = executor.execute_and_get(&lua_code, "a").unwrap();
    let b: bool = executor.execute_and_get(&lua_code, "b").unwrap();
    let c: bool = executor.execute_and_get(&lua_code, "c").unwrap();
    let d: bool = executor.execute_and_get(&lua_code, "d").unwrap();

    assert_eq!(a, false);
    assert_eq!(b, true);
    assert_eq!(c, false);
    assert_eq!(d, true);
}

#[test]
fn test_nil_handling() {
    let source = r#"
        x: number | nil = nil
        y: number | nil = 42
        is_x_nil: boolean = x == nil
        is_y_nil: boolean = y == nil
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let is_x_nil: bool = executor.execute_and_get(&lua_code, "is_x_nil").unwrap();
    let is_y_nil: bool = executor.execute_and_get(&lua_code, "is_y_nil").unwrap();

    assert_eq!(is_x_nil, true);
    assert_eq!(is_y_nil, false);
}

// ============================================================================
// Category 2: Functions (6 tests)
// ============================================================================

#[test]
fn test_function_declaration_and_call() {
    let source = r#"
        function add(a: number, b: number): number {
            return a + b
        }
        result: number = add(5, 3)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 8);
}

#[test]
fn test_function_return_values() {
    let source = r#"
        function get_ten(): number {
            return 10
        }
        function get_greeting(): string {
            return "hello"
        }
        num: number = get_ten()
        greeting: string = get_greeting()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let num: i64 = executor.execute_and_get(&lua_code, "num").unwrap();
    let greeting: String = executor.execute_and_get(&lua_code, "greeting").unwrap();

    assert_eq!(num, 10);
    assert_eq!(greeting, "hello");
}

#[test]
fn test_multiple_return_values() {
    let source = r#"
        function swap(a: number, b: number): (number, number) {
            return b, a
        }
        global x: number = 0
        global y: number = 0
        x, y = swap(1, 2)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let x: i64 = executor.execute_and_get(&lua_code, "x").unwrap();
    let y: i64 = executor.execute_and_get(&lua_code, "y").unwrap();

    assert_eq!(x, 2);
    assert_eq!(y, 1);
}

#[test]
fn test_closures_and_upvalues() {
    let source = r#"
        function make_counter()
            local count: number = 0
            return function(): number
                count = count + 1
                return count
            end
        end
        counter = make_counter()
        first: number = counter()
        second: number = counter()
        third: number = counter()
    "#;

    let lua_code = match compile(source) {
        Ok(code) => code,
        Err(e) => panic!("Compilation failed: {}", e),
    };
    println!("Generated Lua:\n{}", lua_code);
    let executor = LuaExecutor::new().unwrap();
    match executor.execute(&lua_code) {
        Ok(_) => {},
        Err(e) => panic!("Execution failed: {}\nGenerated Lua:\n{}", e, lua_code),
    }

    let first: i64 = executor.execute_and_get(&lua_code, "first").unwrap();
    let second: i64 = executor.execute_and_get(&lua_code, "second").unwrap();
    let third: i64 = executor.execute_and_get(&lua_code, "third").unwrap();

    assert_eq!(first, 1);
    assert_eq!(second, 2);
    assert_eq!(third, 3);
}

#[test]
fn test_recursion() {
    let source = r#"
        function factorial(n: number): number {
            if n <= 1 then
                return 1
            else
                return n * factorial(n - 1)
            end
        }
        result: number = factorial(5)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 120);
}

#[test]
fn test_default_parameters() {
    let source = r#"
        function greet(name: string = "World"): string {
            return "Hello, " .. name
        }
        greeting1: string = greet("Alice")
        greeting2: string = greet()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let greeting1: String = executor.execute_and_get(&lua_code, "greeting1").unwrap();
    let greeting2: String = executor.execute_and_get(&lua_code, "greeting2").unwrap();

    assert_eq!(greeting1, "Hello, Alice");
    assert_eq!(greeting2, "Hello, World");
}

// ============================================================================
// Category 3: Control Flow (5 tests)
// ============================================================================

#[test]
fn test_if_else_branches() {
    let source = r#"
        function classify(x: number): string {
            if x > 0 then
                return "positive"
            elseif x < 0 then
                return "negative"
            else
                return "zero"
            end
        }
        result1: string = classify(10)
        result2: string = classify(-5)
        result3: string = classify(0)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let result1: String = executor.execute_and_get(&lua_code, "result1").unwrap();
    let result2: String = executor.execute_and_get(&lua_code, "result2").unwrap();
    let result3: String = executor.execute_and_get(&lua_code, "result3").unwrap();

    assert_eq!(result1, "positive");
    assert_eq!(result2, "negative");
    assert_eq!(result3, "zero");
}

#[test]
fn test_while_loop() {
    let source = r#"
        sum: number = 0
        local i: number = 1
        while i <= 5 do
            sum = sum + i
            i = i + 1
        end
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let sum: i64 = executor.execute_and_get(&lua_code, "sum").unwrap();
    assert_eq!(sum, 15); // 1 + 2 + 3 + 4 + 5
}

#[test]
fn test_for_loop_numeric() {
    let source = r#"
        sum: number = 0
        for i = 1, 5 do
            sum = sum + i
        end
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let sum: i64 = executor.execute_and_get(&lua_code, "sum").unwrap();
    assert_eq!(sum, 15);
}

#[test]
fn test_for_in_loop() {
    let source = r#"
        local items = [10, 20, 30]
        sum: number = 0
        for _, value in ipairs(items) do
            sum = sum + value
        end
    "#;

    let lua_code = match compile_with_stdlib(source) {
        Ok(code) => code,
        Err(e) => panic!("Compilation failed: {}", e),
    };
    println!("Generated Lua:\n{}", lua_code);
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let sum: i64 = executor.execute_and_get(&lua_code, "sum").unwrap();
    assert_eq!(sum, 60);
}

#[test]
fn test_break_statement() {
    let source = r#"
        sum: number = 0
        for i = 1, 10 do
            if i > 5 then
                break
            end
            sum = sum + i
        end
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let sum: i64 = executor.execute_and_get(&lua_code, "sum").unwrap();
    assert_eq!(sum, 15); // 1 + 2 + 3 + 4 + 5
}

// ============================================================================
// Category 4: Tables (5 tests)
// ============================================================================

#[test]
fn test_table_creation_and_indexing() {
    let source = r#"
        point = {x = 10, y = 20}
        x_val: number = point.x
        y_val: number = point.y
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let x_val: i64 = executor.execute_and_get(&lua_code, "x_val").unwrap();
    let y_val: i64 = executor.execute_and_get(&lua_code, "y_val").unwrap();

    assert_eq!(x_val, 10);
    assert_eq!(y_val, 20);
}

#[test]
fn test_table_destructuring() {
    let source = r#"
        point = {x = 10, y = 20}
        {x, y} = point
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let x: i64 = executor.execute_and_get(&lua_code, "x").unwrap();
    let y: i64 = executor.execute_and_get(&lua_code, "y").unwrap();

    assert_eq!(x, 10);
    assert_eq!(y, 20);
}

#[test]
fn test_array_style_tables() {
    let source = r#"
        arr = {1, 2, 3, 4, 5}
        first: number = arr[1]
        last: number = arr[5]
        len: number = #arr
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let first: i64 = executor.execute_and_get(&lua_code, "first").unwrap();
    let last: i64 = executor.execute_and_get(&lua_code, "last").unwrap();
    let len: i64 = executor.execute_and_get(&lua_code, "len").unwrap();

    assert_eq!(first, 1);
    assert_eq!(last, 5);
    assert_eq!(len, 5);
}

#[test]
fn test_mixed_key_tables() {
    let source = r#"
        mixed = {
            name = "Alice",
            age = 30,
            [1] = "first",
            [2] = "second"
        }
        name_val: string = mixed.name
        age_val: number = mixed.age
        first_val: string = mixed[1]
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let name_val: String = executor.execute_and_get(&lua_code, "name_val").unwrap();
    let age_val: i64 = executor.execute_and_get(&lua_code, "age_val").unwrap();
    let first_val: String = executor.execute_and_get(&lua_code, "first_val").unwrap();

    assert_eq!(name_val, "Alice");
    assert_eq!(age_val, 30);
    assert_eq!(first_val, "first");
}

#[test]
fn test_table_methods() {
    let source = r#"
        obj = {
            value = 10,
            get_value = function(self)
                return self.value
            end,
            set_value = function(self, v)
                self.value = v
            end
        }
        initial: number = obj::get_value()
        obj::set_value(42)
        updated: number = obj::get_value()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let initial: i64 = executor.execute_and_get(&lua_code, "initial").unwrap();
    let updated: i64 = executor.execute_and_get(&lua_code, "updated").unwrap();

    assert_eq!(initial, 10);
    assert_eq!(updated, 42);
}

// ============================================================================
// Category 5: Type System Features (4 tests)
// ============================================================================

#[test]
fn test_type_annotations_no_runtime_impact() {
    let source = r#"
        x: number = 42
        y: string = "hello"
        z: boolean = true
        -- Type annotations should not affect runtime behavior
        result: number = x + 10
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 52);
}

#[test]
fn test_optional_types() {
    let source = r#"
        x: number | nil = nil
        y: number | nil = 42

        function get_value_or_default(val: number | nil, default: number): number {
            if val == nil then
                return default
            else
                return val
            end
        }

        result1: number = get_value_or_default(x, 100)
        result2: number = get_value_or_default(y, 100)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let result1: i64 = executor.execute_and_get(&lua_code, "result1").unwrap();
    let result2: i64 = executor.execute_and_get(&lua_code, "result2").unwrap();

    assert_eq!(result1, 100);
    assert_eq!(result2, 42);
}

#[test]
fn test_union_types_runtime_behavior() {
    let source = r#"
        function process(value: number | string): string {
            if type(value) == "number" then
                return "number: " .. tostring(value)
            else
                return "string: " .. value
            end
        }

        result1: string = process(42)
        result2: string = process("hello")
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let result1: String = executor.execute_and_get(&lua_code, "result1").unwrap();
    let result2: String = executor.execute_and_get(&lua_code, "result2").unwrap();

    assert_eq!(result1, "number: 42");
    assert_eq!(result2, "string: hello");
}

#[test]
fn test_type_narrowing() {
    let source = r#"
        function get_length(value: string | nil): number {
            if value ~= nil then
                return #value
            else
                return 0
            end
        }

        len1: number = get_length("hello")
        len2: number = get_length(nil)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let len1: i64 = executor.execute_and_get(&lua_code, "len1").unwrap();
    let len2: i64 = executor.execute_and_get(&lua_code, "len2").unwrap();

    assert_eq!(len1, 5);
    assert_eq!(len2, 0);
}

// ============================================================================
// Category 6: Classes (6 tests)
// ============================================================================

#[test]
fn test_class_declaration_and_instantiation() {
    let source = r#"
        class Point {
            x: number
            y: number

            constructor(x: number, y: number) {
                self.x = x
                self.y = y
            }
        }

        p = new Point(10, 20)
        x_val: number = p.x
        y_val: number = p.y
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let x_val: i64 = executor.execute_and_get(&lua_code, "x_val").unwrap();
    let y_val: i64 = executor.execute_and_get(&lua_code, "y_val").unwrap();

    assert_eq!(x_val, 10);
    assert_eq!(y_val, 20);
}

#[test]
fn test_class_constructor() {
    let source = r#"
        class Counter {
            value: number

            constructor(initial: number = 0) {
                self.value = initial
            }
        }

        c1 = new Counter(10)
        c2 = new Counter()

        val1: number = c1.value
        val2: number = c2.value
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let val1: i64 = executor.execute_and_get(&lua_code, "val1").unwrap();
    let val2: i64 = executor.execute_and_get(&lua_code, "val2").unwrap();

    assert_eq!(val1, 10);
    assert_eq!(val2, 0);
}

#[test]
fn test_class_instance_methods() {
    let source = r#"
        class Counter {
            value: number

            constructor() {
                self.value = 0
            }

            increment() {
                self.value = self.value + 1
            }

            get_value(): number {
                return self.value
            }
        }

        c = new Counter()
        c::increment()
        c::increment()
        result: number = c::get_value()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 2);
}

#[test]
fn test_class_static_methods() {
    let source = r#"
        class Math {
            static add(a: number, b: number): number {
                return a + b
            }

            static multiply(a: number, b: number): number {
                return a * b
            }
        }

        sum: number = Math.add(5, 3)
        product: number = Math.multiply(4, 7)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let sum: i64 = executor.execute_and_get(&lua_code, "sum").unwrap();
    let product: i64 = executor.execute_and_get(&lua_code, "product").unwrap();

    assert_eq!(sum, 8);
    assert_eq!(product, 28);
}

#[test]
fn test_class_inheritance() {
    let source = r#"
        class Animal {
            name: string

            constructor(name: string) {
                self.name = name
            }

            speak(): string {
                return self.name .. " makes a sound"
            }
        }

        class Dog extends Animal {
            constructor(name: string) {
                self.name = name
            }

            override speak(): string {
                return self.name .. " barks"
            }
        }

        dog = new Dog("Rex")
        result: string = dog::speak()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "Rex barks");
}

#[test]
fn test_class_method_overriding() {
    let source = r#"
        class Base {
            value: number

            constructor() {
                self.value = 10
            }

            get_value(): number {
                return self.value
            }
        }

        class Derived extends Base {
            constructor() {
                self.value = 10
            }

            override get_value(): number {
                return self.value * 2
            }
        }

        base = new Base()
        derived = new Derived()

        base_val: number = base::get_value()
        derived_val: number = derived::get_value()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let base_val: i64 = executor.execute_and_get(&lua_code, "base_val").unwrap();
    let derived_val: i64 = executor.execute_and_get(&lua_code, "derived_val").unwrap();

    assert_eq!(base_val, 10);
    assert_eq!(derived_val, 20);
}

// ============================================================================
// Category 7: String Interpolation (3 tests)
// ============================================================================

#[test]
fn test_basic_string_interpolation() {
    let source = r#"
        name: string = "Alice"
        greeting: string = `Hello, ${name}!`
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let greeting: String = executor.execute_and_get(&lua_code, "greeting").unwrap();
    assert_eq!(greeting, "Hello, Alice!");
}

#[test]
fn test_expression_interpolation() {
    let source = r#"
        x: number = 10
        y: number = 20
        result: string = `The sum of ${x} and ${y} is ${x + y}`
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "The sum of 10 and 20 is 30");
}

#[test]
fn test_nested_interpolation() {
    let source = r#"
        inner: string = "world"
        outer: string = `Hello, ${`nested ${inner}`}!`
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let outer: String = executor.execute_and_get(&lua_code, "outer").unwrap();
    assert_eq!(outer, "Hello, nested world!");
}

// ============================================================================
// Category 8: Destructuring (4 tests)
// ============================================================================

#[test]
fn test_array_destructuring() {
    let source = r#"
        local arr = [1, 2, 3];
        [a, b, c] = arr
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let a: i64 = executor.execute_and_get(&lua_code, "a").unwrap();
    let b: i64 = executor.execute_and_get(&lua_code, "b").unwrap();
    let c: i64 = executor.execute_and_get(&lua_code, "c").unwrap();

    assert_eq!(a, 1);
    assert_eq!(b, 2);
    assert_eq!(c, 3);
}

#[test]
fn test_object_destructuring() {
    let source = r#"
        local person = {name = "Alice", age = 30, city = "NYC"}
        {name, age} = person
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let name: String = executor.execute_and_get(&lua_code, "name").unwrap();
    let age: i64 = executor.execute_and_get(&lua_code, "age").unwrap();

    assert_eq!(name, "Alice");
    assert_eq!(age, 30);
}

#[test]
fn test_nested_destructuring() {
    let source = r#"
        local data = {
            user = {name = "Bob", age = 25},
            location = {city = "LA", country = "USA"}
        }
        {user: {name, age}} = data
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let name: String = executor.execute_and_get(&lua_code, "name").unwrap();
    let age: i64 = executor.execute_and_get(&lua_code, "age").unwrap();

    assert_eq!(name, "Bob");
    assert_eq!(age, 25);
}

#[test]
fn test_destructuring_with_defaults() {
    let source = r#"
        local obj = {x = 10, y = nil, z = nil}
        {x, y = 20, z = 30} = obj
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let x: i64 = executor.execute_and_get(&lua_code, "x").unwrap();
    let y: i64 = executor.execute_and_get(&lua_code, "y").unwrap();
    let z: i64 = executor.execute_and_get(&lua_code, "z").unwrap();

    assert_eq!(x, 10);
    assert_eq!(y, 20);
    assert_eq!(z, 30);
}

// ============================================================================
// Category 9: Match Expressions (3 tests)
// ============================================================================

#[test]
fn test_simple_pattern_matching() {
    let source = r#"
        function classify(x: number): string {
            return match x {
                1 => "one",
                2 => "two",
                3 => "three",
                _ => "other"
            }
        }

        result1: string = classify(1)
        result2: string = classify(3)
        result3: string = classify(99)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let result1: String = executor.execute_and_get(&lua_code, "result1").unwrap();
    let result2: String = executor.execute_and_get(&lua_code, "result2").unwrap();
    let result3: String = executor.execute_and_get(&lua_code, "result3").unwrap();

    assert_eq!(result1, "one");
    assert_eq!(result2, "three");
    assert_eq!(result3, "other");
}

#[test]
fn test_match_with_guards() {
    let source = r#"
        function categorize(x: number): string {
            return match x {
                n when n > 100 => "large",
                n when n > 10 => "medium",
                n when n > 0 => "small",
                _ => "zero or negative"
            }
        }

        result1: string = categorize(150)
        result2: string = categorize(50)
        result3: string = categorize(5)
        result4: string = categorize(-10)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let result1: String = executor.execute_and_get(&lua_code, "result1").unwrap();
    let result2: String = executor.execute_and_get(&lua_code, "result2").unwrap();
    let result3: String = executor.execute_and_get(&lua_code, "result3").unwrap();
    let result4: String = executor.execute_and_get(&lua_code, "result4").unwrap();

    assert_eq!(result1, "large");
    assert_eq!(result2, "medium");
    assert_eq!(result3, "small");
    assert_eq!(result4, "zero or negative");
}

#[test]
fn test_match_exhaustiveness() {
    let source = r#"
        type Status = "pending" | "approved" | "rejected"

        function get_message(status: Status): string {
            return match status {
                "pending" => "Waiting for approval",
                "approved" => "Request approved",
                "rejected" => "Request rejected"
            }
        }

        msg1: string = get_message("pending")
        msg2: string = get_message("approved")
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let msg1: String = executor.execute_and_get(&lua_code, "msg1").unwrap();
    let msg2: String = executor.execute_and_get(&lua_code, "msg2").unwrap();

    assert_eq!(msg1, "Waiting for approval");
    assert_eq!(msg2, "Request approved");
}
