//! Execution tests for enums - both simple (table-based) and rich (Java-style
//! with fields, methods, and constructor arguments).

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

// ============================================================================
// Simple Enums
// ============================================================================

#[test]
fn test_simple_enum_integer_indices() {
    // Simple enum members get 0-based integer indices by default
    let source = r#"
        enum Color {
            Red,
            Green,
            Blue,
        }
        r: number = Color.Red
        g: number = Color.Green
        b: number = Color.Blue
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let r: i64 = executor.execute_and_get(&lua_code, "r").unwrap();
    let g: i64 = executor.execute_and_get(&lua_code, "g").unwrap();
    let b: i64 = executor.execute_and_get(&lua_code, "b").unwrap();
    assert_eq!(r, 0);
    assert_eq!(g, 1);
    assert_eq!(b, 2);
}

#[test]
fn test_simple_enum_string_values() {
    // Enum members with explicit string values
    let source = r#"
        enum Status {
            Active = "active",
            Inactive = "inactive",
            Pending = "pending",
        }
        a: string = Status.Active
        i: string = Status.Inactive
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let a: String = executor.execute_and_get(&lua_code, "a").unwrap();
    let i: String = executor.execute_and_get(&lua_code, "i").unwrap();
    assert_eq!(a, "active");
    assert_eq!(i, "inactive");
}

#[test]
fn test_simple_enum_number_values() {
    // Enum members with explicit numeric values
    let source = r#"
        enum Priority {
            Low = 1,
            Medium = 5,
            High = 10,
        }
        low: number = Priority.Low
        high: number = Priority.High
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let low: i64 = executor.execute_and_get(&lua_code, "low").unwrap();
    let high: i64 = executor.execute_and_get(&lua_code, "high").unwrap();
    assert_eq!(low, 1);
    assert_eq!(high, 10);
}

#[test]
fn test_enum_comparison() {
    // Enum values can be compared with ==
    let source = r#"
        enum Direction {
            North,
            South,
            East,
            West,
        }
        current: number = Direction.North
        is_north: boolean = current == Direction.North
        is_south: boolean = current == Direction.South
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let is_north: bool = executor.execute_and_get(&lua_code, "is_north").unwrap();
    let is_south: bool = executor.execute_and_get(&lua_code, "is_south").unwrap();
    assert!(is_north);
    assert!(!is_south);
}

#[test]
fn test_enum_in_conditional() {
    // Enum value used in if/elseif conditional
    let source = r#"
        enum Color {
            Red,
            Green,
            Blue,
        }
        function color_name(c: number): string {
            if c == Color.Red then
                return "red"
            elseif c == Color.Green then
                return "green"
            else
                return "blue"
            end
        }
        result: string = color_name(Color.Green)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "green");
}

// ============================================================================
// Rich Enums (Java-style with fields and methods)
// ============================================================================

#[test]
fn test_rich_enum_methods() {
    // Rich enum with method - Direction.North:isVertical() should return true
    let source = r#"
        enum Direction {
            North,
            South,
            East,
            West,

            isVertical(): boolean {
                return self == Direction.North or self == Direction.South
            }
        }
        r1: boolean = Direction.North::isVertical()
        r2: boolean = Direction.East::isVertical()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let r1: bool = executor.execute_and_get(&lua_code, "r1").unwrap();
    let r2: bool = executor.execute_and_get(&lua_code, "r2").unwrap();
    assert!(r1, "North should be vertical");
    assert!(!r2, "East should not be vertical");
}

#[test]
fn test_rich_enum_field_access() {
    // Rich enum with typed fields and constructor args
    let source = r#"
        enum Planet {
            mass: number
            radius: number

            Mercury(3.303e23, 2.4397e6),
            Venus(4.869e24, 6.0518e6),
            Earth(5.972e24, 6.371e6),
        }
        earth_mass: number = Planet.Earth.mass
        mercury_radius: number = Planet.Mercury.radius
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let earth_mass: f64 = executor.execute_and_get(&lua_code, "earth_mass").unwrap();
    let mercury_radius: f64 = executor
        .execute_and_get(&lua_code, "mercury_radius")
        .unwrap();
    assert!(
        (earth_mass - 5.972e24).abs() / 5.972e24 < 0.001,
        "Earth mass should be ~5.972e24"
    );
    assert!(
        (mercury_radius - 2.4397e6).abs() / 2.4397e6 < 0.001,
        "Mercury radius should be ~2.4397e6"
    );
}

#[test]
fn test_rich_enum_ordinal_method() {
    // Rich enums (with at least one method) have :ordinal() returning 0-based index
    let source = r#"
        enum Season {
            Spring,
            Summer,
            Autumn,
            Winter,

            isWarm(): boolean {
                return self == Season.Summer
            }
        }
        spring_ord: number = Season.Spring::ordinal()
        winter_ord: number = Season.Winter::ordinal()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let spring_ord: i64 = executor.execute_and_get(&lua_code, "spring_ord").unwrap();
    let winter_ord: i64 = executor.execute_and_get(&lua_code, "winter_ord").unwrap();
    assert_eq!(spring_ord, 0);
    assert_eq!(winter_ord, 3);
}

#[test]
fn test_rich_enum_name_method() {
    // Rich enums (with at least one method) have :name() returning the member name
    let source = r#"
        enum Season {
            Spring,
            Summer,
            Autumn,
            Winter,

            isWarm(): boolean {
                return self == Season.Summer
            }
        }
        spring_name: string = Season.Spring::name()
        summer_name: string = Season.Summer::name()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let spring_name: String = executor.execute_and_get(&lua_code, "spring_name").unwrap();
    let summer_name: String = executor.execute_and_get(&lua_code, "summer_name").unwrap();
    assert_eq!(spring_name, "Spring");
    assert_eq!(summer_name, "Summer");
}

#[test]
fn test_rich_enum_as_function_argument() {
    // Rich enum value passed as a function argument
    let source = r#"
        enum Level {
            Low,
            Medium,
            High,

            label(): string {
                return self::name()
            }
        }
        function describe(level: Level): string {
            return "Level: " .. level::label()
        }
        result: string = describe(Level.High)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "Level: High");
}
