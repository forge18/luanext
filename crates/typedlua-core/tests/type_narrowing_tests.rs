use typedlua_core::di::DiContainer;

fn type_check(source: &str) -> Result<(), String> {
    let mut container = DiContainer::test_default();
    container.compile(source)?;
    Ok(())
}

#[test]
fn test_literal_narrowing_number() {
    let source = r#"
        const x: number = 42
        const result = match x {
            42 => "the answer"
            _ => "other"
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Literal narrowing should work");
}

#[test]
fn test_literal_narrowing_string() {
    let source = r#"
        const status: "pending" | "active" | "done" = "active"
        match status {
            "pending" => "waiting"
            "active" => "running"
            "done" => "finished"
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "String literal narrowing should work");
}

#[test]
fn test_boolean_narrowing() {
    let source = r#"
        const flag: boolean = true
        if flag {
            const b: true = flag
        } else {
            const b: false = flag
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Boolean narrowing should work");
}

#[test]
fn test_type_guard_narrowing() {
    let source = r#"
        const x: number | string = 42
        if typeof(x) == "number" {
            const n: number = x
        } else {
            const s: string = x
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Type guard narrowing should work");
}

#[test]
fn test_nullable_narrowing() {
    let source = r#"
        const x: number | nil = nil
        if x != nil {
            const n: number = x
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Nullable narrowing should work");
}

#[test]
fn test_narrowing_in_match() {
    let source = r#"
        const value: { kind: "a", a: number } | { kind: "b", b: string } = { kind: "a", a: 1 }
        match value {
            { kind: "a", a } => const n: number = a
            { kind: "b", b } => const s: string = b
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Narrowing in match should work");
}

#[test]
fn test_is_operator_narrowing() {
    let source = r#"
        const x: unknown = 42
        if x is number {
            const n: number = x
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "IS operator narrowing should work");
}

#[test]
fn test_assertion_narrowing() {
    let source = r#"
        const x: number | nil = nil
        if x != nil {
            const n: number = x
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Assertion narrowing should work");
}

#[test]
fn test_narrowing_with_exhaustiveness() {
    let source = r#"
        const x: "a" | "b" | "c" = "a"
        match x {
            "a" => const a = "a"
            "b" => const b = "b"
            "c" => const c = "c"
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Narrowing with exhaustiveness should work");
}

#[test]
fn test_narrowing_assignment() {
    let source = r#"
        let x: number | nil = nil
        if x != nil {
            x = x
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Narrowing in assignment should work");
}

#[test]
fn test_narrowing_loop() {
    let source = r#"
        const arr: number[] | nil = nil
        for i in 0..10 {
            if arr != nil {
                const a: number[] = arr
            }
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Narrowing in loop should work");
}

#[test]
fn test_nested_narrowing() {
    let source = r#"
        type A = { kind: "a", value: number }
        type B = { kind: "b", value: string }
        const x: A | B | nil = nil
        if x != nil {
            if x.kind == "a" {
                const n: number = x.value
            }
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Nested narrowing should work");
}

#[test]
fn test_custom_type_guard() {
    let source = r#"
        function isNumber(x: unknown): x is number {
            return typeof(x) == "number"
        }

        const x: unknown = 42
        if isNumber(x) {
            const n: number = x
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Custom type guard should work");
}

#[test]
fn test_narrowing_preserves_union() {
    let source = r#"
        const x: number | string = 42
        const y = x
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Narrowing should preserve union");
}

#[test]
fn test_discriminant_narrowing() {
    let source = r#"
        type ClickEvent = { type: "click", x: number, y: number }
        type KeyEvent = { type: "key", key: string }

        const event: ClickEvent | KeyEvent = { type: "click", x: 1, y: 2 }
        if event.type == "click" {
            const x: number = event.x
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Discriminant narrowing should work");
}

#[test]
fn test_narrowing_function_return() {
    let source = r#"
        function parse(input: string): { success: boolean, value: number } | nil {
            return nil
        }

        const result = parse("test")
        if result != nil && result.success {
            const n: number = result.value
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Narrowing function return should work");
}

#[test]
fn test_narrowing_inferred_type() {
    let source = r#"
        const value: number | string = 42
        const inferred = value
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Inferred type narrowing should work");
}

#[test]
fn test_narrowing_with_equality() {
    let source = r#"
        const x: 1 | 2 | 3 = 2
        if x == 2 {
            const y: 2 = x
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Equality narrowing should work");
}

#[test]
fn test_narrowing_in_ternary() {
    let source = r#"
        const x: number | nil = nil
        const y = x != nil ? (x as number) + 1 : 0
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Narrowing in ternary should work");
}

#[test]
fn test_narrowing_complex_union() {
    let source = r#"
        type A = { kind: "a" }
        type B = { kind: "b" }
        type C = { kind: "c" }

        const x: A | B | C = { kind: "a" }
        match x {
            { kind: "a" } => const a = "a"
            { kind: "b" } => const b = "b"
            { kind: "c" } => const c = "c"
        }
    "#;

    let result = type_check(source);
    assert!(result.is_ok(), "Complex union narrowing should work");
}
