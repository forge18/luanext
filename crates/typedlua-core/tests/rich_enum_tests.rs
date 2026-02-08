use typedlua_core::di::DiContainer;

fn compile_and_check(source: &str) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile(source)
}

// =============================================================================
// Java-style rich enums: members with constructor args, fields, methods
// defined inside the enum body (not via impl blocks)
// =============================================================================

#[test]
fn test_rich_enum_with_constructor_args() {
    // Java-style: enum constants with positional arguments mapped to fields
    let source = r#"
        enum Planet {
            Mercury(3.303e23, 2.4397e6),
            Venus(4.869e24, 6.0518e6),
            Earth(5.972e24, 6.371e6),
        }
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Enum with constructor args should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_rich_enum_with_methods() {
    // Java-style: methods defined inside the enum body
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
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Enum with methods should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_rich_enum_with_fields_and_constructor() {
    // Java-style: fields, constructor, and methods all inside enum body
    let source = r#"
        enum Planet {
            mass: number
            radius: number

            Mercury(3.303e23, 2.4397e6),
            Venus(4.869e24, 6.0518e6),
            Earth(5.972e24, 6.371e6),

            surfaceGravity(): number {
                return 6.67300e-11 * self.mass / (self.radius * self.radius)
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Enum with fields and constructor should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_rich_enum_with_function_keyword() {
    // Methods can also be defined with the `function` keyword
    let source = r#"
        enum Color {
            Red,
            Green,
            Blue,

            function name(): string {
                return "color"
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Enum with function keyword method should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_rich_enum_with_interface() {
    // Java-style: implements clause on the enum itself
    let source = r#"
        interface Describable {
            describe(): string
        }

        enum Season implements Describable {
            Spring,
            Summer,
            Autumn,
            Winter,

            describe(): string {
                return "a season"
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Enum implementing interface should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_rich_enum_simple_match() {
    // Match on simple enum uses bare variant names (like Java switch case labels)
    let source = r#"
        enum Color {
            Red,
            Green,
            Blue,
        }

        const c: Color = Color.Red
        const result = match c {
            Red => "red"
            Green => "green"
            Blue => "blue"
        }
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Simple enum match should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_rich_enum_match_with_wildcard() {
    // Match with wildcard fallback
    let source = r#"
        enum State {
            Idle,
            Running,
            Paused,
            Stopped,
        }

        function handle(state: State): string
            return match state {
                Idle => "idle",
                Running => "running",
                _ => "other",
            }
        end
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Enum match with wildcard should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_rich_enum_qualified_access() {
    // Accessing enum constants with qualified names (EnumName.Variant)
    let source = r#"
        enum Priority {
            Low,
            Medium,
            High,
        }

        const p = Priority.High
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Qualified enum access should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_rich_enum_simple() {
    // Simple enum with no fields or methods (like Java enum with just constants)
    let source = r#"
        enum LogLevel {
            Debug,
            Info,
            Warning,
            Error,
        }
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Simple enum should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_rich_enum_export() {
    let source = r#"
        export enum Status {
            Pending,
            Active,
            Done,
        }
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Exported enum should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_rich_enum_with_constructor() {
    // Explicit constructor in enum body
    let source = r#"
        enum Coin {
            value: number

            Penny(1),
            Nickel(5),
            Dime(10),
            Quarter(25),

            constructor(value: number) {
                self.value = value
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Enum with constructor should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_rich_enum_method_using_self() {
    // Methods that reference self to inspect enum state
    let source = r#"
        enum Coin {
            value: number

            Penny(1),
            Nickel(5),
            Dime(10),
            Quarter(25),

            getLabel(): string {
                return "coin worth " .. self.value
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Enum method with self should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_rich_enum_multiple_methods() {
    // Multiple methods defined inside enum body
    let source = r#"
        enum Priority {
            Low,
            Medium,
            High,

            ordinal(): number {
                return self.__ordinal
            }

            name(): string {
                return self.__name
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Enum with multiple methods should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_rich_enum_generic_declaration() {
    // Simple enum (generics on enums are not yet supported in the parser)
    let source = r#"
        enum Maybe {
            Something,
            Nothing,
        }
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Simple enum should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_rich_enum_with_method_params() {
    // Methods with parameters
    let source = r#"
        enum MathOp {
            Add,
            Sub,
            Mul,

            apply(a: number, b: number): number {
                return a + b
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Enum method with params should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_rich_enum_enum_values() {
    // Enum with explicit string values
    let source = r#"
        enum HttpMethod {
            Get = "GET",
            Post = "POST",
            Put = "PUT",
            Delete = "DELETE",
        }
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Enum with string values should compile: {:?}",
        result.err()
    );
}
