use typedlua_core::di::DiContainer;

fn type_check(source: &str) -> Result<(), String> {
    let mut container = DiContainer::test_default();
    container.compile(source)?;
    Ok(())
}

#[test]
fn test_boolean_exhaustive_with_wildcard() {
    let source = r#"
        const x: boolean = true
        const result = match x {
            _ => "default"
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Boolean with wildcard should be exhaustive");
}

#[test]
fn test_boolean_not_exhaustive_without_wildcard() {
    let source = r#"
        const x: boolean = true
        const result = match x {
            true => "yes"
        }
    "#;

    let result = type_check(source);
    assert!(
        result.is_err(),
        "Boolean without wildcard should not be exhaustive"
    );
}

#[test]
fn test_enum_exhaustive_match() {
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

    let result = type_check(source);
    assert!(result.is_ok(), "Complete enum match should be exhaustive");
}

#[test]
fn test_enum_not_exhaustive() {
    let source = r#"
        enum Color {
            Red,
            Green,
            Blue,
        }

        const c: Color = Color.Red
        const result = match c {
            Red => "red"
        }
    "#;

    let result = type_check(source);
    assert!(
        result.is_err(),
        "Incomplete enum match should not be exhaustive"
    );
}

#[test]
fn test_union_exhaustive_match() {
    let source = r#"
        type NumOrStr = number | string
        const x: NumOrStr = 42
        const result = match x {
            n: number => tostring(n)
            s: string => s
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Complete union match should be exhaustive");
}

#[test]
fn test_union_not_exhaustive() {
    let source = r#"
        type NumOrStr = number | string
        const x: NumOrStr = 42
        const result = match x {
            n: number => tostring(n)
        }
    "#;

    let result = type_check(source);
    assert!(
        result.is_err(),
        "Incomplete union match should not be exhaustive"
    );
}

#[test]
fn test_literal_type_exhaustive() {
    let source = r#"
        const x: "a" | "b" | "c" = "a"
        const result = match x {
            "a" => 1
            "b" => 2
            "c" => 3
        }
    "#;

    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Complete literal match should be exhaustive"
    );
}

#[test]
fn test_literal_type_not_exhaustive() {
    let source = r#"
        const x: "a" | "b" | "c" = "a"
        const result = match x {
            "a" => 1
            "b" => 2
        }
    "#;

    let result = type_check(source);
    assert!(
        result.is_err(),
        "Incomplete literal match should not be exhaustive"
    );
}

#[test]
fn test_if_else_chain_exhaustive() {
    let source = r#"
        const x: boolean = true
        if x {
            const a = "yes"
        } else {
            const a = "no"
        }
    "#;

    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Complete if-else chain should be exhaustive"
    );
}

#[test]
fn test_if_else_chain_not_exhaustive() {
    let source = r#"
        const x: boolean = true
        if x {
            const a = "yes"
        }
    "#;

    let result = type_check(source);
    assert!(
        result.is_err(),
        "Incomplete if chain should not be exhaustive"
    );
}

#[test]
fn test_never_type_exhaustive() {
    let source = r#"
        function f(x: never): void {
            match x {
            }
        }
    "#;

    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Never type should be exhaustively matchable"
    );
}

#[test]
fn test_nested_enum_exhaustive() {
    let source = r#"
        enum Inner {
            A,
            B,
        }

        enum Outer {
            X,
            Y,
        }

        const x: Inner | Outer = Inner.A
        const result = match x {
            Inner.A => 1
            Inner.B => 2
            Outer.X => 3
            Outer.Y => 4
        }
    "#;

    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Nested enum union match should be exhaustive"
    );
}

#[test]
fn test_never_return_exhaustive() {
    let source = r#"
        function throwError(msg: string): never {
            throw msg
        }

        const x: boolean = true
        if x {
            throwError("error")
        } else {
            const y = "ok"
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Never return should make if exhaustive");
}

#[test]
fn test_nullable_type_exhaustive() {
    let source = r#"
        const x: number | nil = nil
        if x != nil {
            const n = x
        } else {
            const none = x
        }
    "#;

    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Nullable type with nil check should be exhaustive"
    );
}

#[test]
fn test_nullable_not_exhaustive() {
    let source = r#"
        const x: number | nil = nil
        if x != nil {
            const n = x
        }
    "#;

    let result = type_check(source);
    assert!(
        result.is_err(),
        "Incomplete nullable check should not be exhaustive"
    );
}

#[test]
fn test_object_pattern_exhaustive() {
    let source = r#"
        type Point = { x: number, y: number }
        const p: Point = { x: 1, y: 2 }
        if p.x == 1 and p.y == 2 {
            const found = true
        }
    "#;

    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Object pattern with all checks should be exhaustive"
    );
}

#[test]
fn test_guard_type_exhaustive() {
    let source = r#"
        const x: number | string = 42
        if typeof(x) == "number" {
            const n: number = x
        } else {
            const s: string = x
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Type guard should make check exhaustive");
}

#[test]
fn test_match_with_default() {
    let source = r#"
        const x: number = 1
        const result = match x {
            1 => "one"
            2 => "two"
            _ => "other"
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Match with default should be exhaustive");
}

#[test]
fn test_generic_exhaustive() {
    let source = r#"
        function f<T extends string | number>(x: T): string {
            if typeof(x) == "string" {
                const s: string = x
                return s
            } else {
                const n: number = x
                return tostring(n)
            }
        }
    "#;

    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Generic with type guard should be exhaustive"
    );
}

#[test]
fn test_interface_union_exhaustive() {
    let source = r#"
        type A = { kind: "a", a: number }
        type B = { kind: "b", b: string }

        function f(x: A | B): void {
            if x.kind == "a" {
                const a_val: number = x.a
            } else {
                const b_val: string = x.b
            }
        }
    "#;

    let result = type_check(source);
    assert!(
        result.is_ok(),
        "Interface union with kind check should be exhaustive"
    );
}
