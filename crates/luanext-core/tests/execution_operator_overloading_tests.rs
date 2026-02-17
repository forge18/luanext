//! Execution tests for operator overloading - class `operator` declarations
//! that compile to Lua metamethods (__add, __sub, __eq, __len, etc.)

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

// ============================================================================
// Binary Arithmetic Operators
// ============================================================================

#[test]
fn test_operator_add() {
    let source = r#"
        class Vector {
            x: number
            y: number

            constructor(x: number, y: number) {
                self.x = x
                self.y = y
            }

            operator +(other: Vector): Vector {
                return new Vector(self.x + other.x, self.y + other.y)
            }
        }

        v1 = new Vector(1, 2)
        v2 = new Vector(3, 4)
        result = v1 + v2
        result_x: number = result.x
        result_y: number = result.y
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let x: i64 = executor.execute_and_get(&lua_code, "result_x").unwrap();
    let y: i64 = executor.execute_and_get(&lua_code, "result_y").unwrap();
    assert_eq!(x, 4);
    assert_eq!(y, 6);
}

#[test]
fn test_operator_subtract() {
    let source = r#"
        class Vector {
            x: number
            y: number

            constructor(x: number, y: number) {
                self.x = x
                self.y = y
            }

            operator -(other: Vector): Vector {
                return new Vector(self.x - other.x, self.y - other.y)
            }
        }

        v1 = new Vector(10, 20)
        v2 = new Vector(3, 5)
        result = v1 - v2
        result_x: number = result.x
        result_y: number = result.y
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let x: i64 = executor.execute_and_get(&lua_code, "result_x").unwrap();
    let y: i64 = executor.execute_and_get(&lua_code, "result_y").unwrap();
    assert_eq!(x, 7);
    assert_eq!(y, 15);
}

#[test]
fn test_operator_multiply() {
    let source = r#"
        class Vector {
            x: number
            y: number

            constructor(x: number, y: number) {
                self.x = x
                self.y = y
            }

            operator *(scalar: number): Vector {
                return new Vector(self.x * scalar, self.y * scalar)
            }
        }

        v = new Vector(3, 4)
        result = v * 5
        result_x: number = result.x
        result_y: number = result.y
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let x: i64 = executor.execute_and_get(&lua_code, "result_x").unwrap();
    let y: i64 = executor.execute_and_get(&lua_code, "result_y").unwrap();
    assert_eq!(x, 15);
    assert_eq!(y, 20);
}

// ============================================================================
// Comparison Operators
// ============================================================================

#[test]
fn test_operator_equal() {
    let source = r#"
        class Point {
            x: number
            y: number

            constructor(x: number, y: number) {
                self.x = x
                self.y = y
            }

            operator ==(other: Point): boolean {
                return self.x == other.x and self.y == other.y
            }
        }

        p1 = new Point(1, 2)
        p2 = new Point(1, 2)
        p3 = new Point(3, 4)
        eq1: boolean = p1 == p2
        eq2: boolean = p1 == p3
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let eq1: bool = executor.execute_and_get(&lua_code, "eq1").unwrap();
    let eq2: bool = executor.execute_and_get(&lua_code, "eq2").unwrap();
    assert!(eq1);
    assert!(!eq2);
}

#[test]
fn test_operator_less_than() {
    let source = r#"
        class Score {
            value: number

            constructor(value: number) {
                self.value = value
            }

            operator <(other: Score): boolean {
                return self.value < other.value
            }
        }

        s1 = new Score(10)
        s2 = new Score(20)
        lt1: boolean = s1 < s2
        lt2: boolean = s2 < s1
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let lt1: bool = executor.execute_and_get(&lua_code, "lt1").unwrap();
    let lt2: bool = executor.execute_and_get(&lua_code, "lt2").unwrap();
    assert!(lt1);
    assert!(!lt2);
}

// ============================================================================
// Unary Operators
// ============================================================================

#[test]
fn test_operator_unary_minus() {
    let source = r#"
        class Vector {
            x: number
            y: number

            constructor(x: number, y: number) {
                self.x = x
                self.y = y
            }

            operator -(): Vector {
                return new Vector(-self.x, -self.y)
            }
        }

        v = new Vector(3, -4)
        neg = -v
        neg_x: number = neg.x
        neg_y: number = neg.y
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let x: i64 = executor.execute_and_get(&lua_code, "neg_x").unwrap();
    let y: i64 = executor.execute_and_get(&lua_code, "neg_y").unwrap();
    assert_eq!(x, -3);
    assert_eq!(y, 4);
}

#[test]
fn test_operator_length() {
    let source = r#"
        class Collection {
            items: number[]

            constructor() {
                self.items = {}
            }

            add(item: number) {
                self.items[#self.items + 1] = item
            }

            operator #(): number {
                return #self.items
            }
        }

        c = new Collection()
        c::add(10)
        c::add(20)
        c::add(30)
        len: number = #c
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let len: i64 = executor.execute_and_get(&lua_code, "len").unwrap();
    assert_eq!(len, 3);
}

// ============================================================================
// String Operators
// ============================================================================

#[test]
fn test_operator_concatenate() {
    let source = r#"
        class Name {
            value: string

            constructor(value: string) {
                self.value = value
            }

            operator ..(other: Name): Name {
                return new Name(self.value .. " " .. other.value)
            }
        }

        first = new Name("Hello")
        last = new Name("World")
        full = first .. last
        result: string = full.value
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "Hello World");
}

// ============================================================================
// Chaining and Multiple Operators
// ============================================================================

#[test]
fn test_operator_chaining() {
    let source = r#"
        class Num {
            value: number

            constructor(value: number) {
                self.value = value
            }

            operator +(other: Num): Num {
                return new Num(self.value + other.value)
            }
        }

        a = new Num(1)
        b = new Num(2)
        c = new Num(3)
        result = a + b + c
        total: number = result.value
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let total: i64 = executor.execute_and_get(&lua_code, "total").unwrap();
    assert_eq!(total, 6);
}

#[test]
fn test_operator_multiple_types() {
    // Class with multiple operator overloads
    let source = r#"
        class Counter {
            value: number

            constructor(value: number) {
                self.value = value
            }

            operator +(other: Counter): Counter {
                return new Counter(self.value + other.value)
            }

            operator -(other: Counter): Counter {
                return new Counter(self.value - other.value)
            }

            operator ==(other: Counter): boolean {
                return self.value == other.value
            }

            operator -(): Counter {
                return new Counter(-self.value)
            }
        }

        a = new Counter(10)
        b = new Counter(3)

        sum = a + b
        diff = a - b
        neg = -a
        is_eq: boolean = a == new Counter(10)

        sum_val: number = sum.value
        diff_val: number = diff.value
        neg_val: number = neg.value
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let sum_val: i64 = executor.execute_and_get(&lua_code, "sum_val").unwrap();
    let diff_val: i64 = executor.execute_and_get(&lua_code, "diff_val").unwrap();
    let neg_val: i64 = executor.execute_and_get(&lua_code, "neg_val").unwrap();
    let is_eq: bool = executor.execute_and_get(&lua_code, "is_eq").unwrap();

    assert_eq!(sum_val, 13);
    assert_eq!(diff_val, 7);
    assert_eq!(neg_val, -10);
    assert!(is_eq);
}
