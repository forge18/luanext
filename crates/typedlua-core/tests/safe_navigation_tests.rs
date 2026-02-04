use typedlua_core::di::DiContainer;

fn compile_and_check(source: &str) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile(source)
}

#[test]
fn test_optional_member_access() {
    let source = r#"
        const user = {name: "Alice"}
        const name = user?.name
    "#;

    let result = compile_and_check(source);
    match &result {
        Ok(output) => {
            println!("Success! Generated code:\n{}", output);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    assert!(result.is_ok(), "Optional member access should compile");
}

#[test]
fn test_optional_member_access_nil() {
    let source = r#"
        const user: {name: string} | nil = nil
        const name = user?.name
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Optional member access nil should compile");
}

#[test]
fn test_optional_chaining() {
    let source = r#"
        type Person = { contact: { email: string } | nil } | nil
        const p: Person = nil
        const email = p?.contact?.email
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Optional chaining should compile");
}

#[test]
fn test_optional_method_call() {
    let source = r#"
        type Calculator = { compute: () => number } | nil
        const calc: Calculator = nil
        const result = calc?.compute()
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Optional method call should compile");
}

#[test]
fn test_optional_element_access() {
    let source = r#"
        const arr: number[] | nil = nil
        const first = arr?[0]
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Optional element access should compile");
}

#[test]
fn test_optional_invocation() {
    let source = r#"
        const fn: (() => number) | nil = nil
        const result = fn?()
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Optional invocation should compile");
}

#[test]
fn test_optional_chaining_with_expression() {
    let source = r#"
        type A = { b: { c: number } | nil } | nil
        const a: A = nil
        const result = a?.b?.c ?? 0
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Optional chaining with expression should compile"
    );
}

#[test]
fn test_optional_chaining_multiple_levels() {
    let source = r#"
        type Level1 = { level2: { level3: { value: number } | nil } | nil } | nil
        const l1: Level1 = nil
        const value = l1?.level2?.level3?.value
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Multiple levels should compile");
}

#[test]
fn test_optional_with_null_coalescing() {
    let source = r#"
        type Config = { timeout: number } | nil
        const config: Config = nil
        const timeout = config?.timeout ?? 30
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Optional with null coalescing should compile"
    );
}

#[test]
fn test_optional_in_arrow_function() {
    let source = r#"
        const getObject = (): { value: number } | nil => nil
        const result = getObject()?.value
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Optional in arrow function should compile");
}

#[test]
fn test_optional_with_index_signature() {
    let source = r#"
        type Dict = { [string]: { value: number } } | nil
        const d: Dict = nil
        const value = d?.["key"]?.value
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Optional with index signature should compile"
    );
}

#[test]
fn test_optional_chaining_in_object_literal() {
    let source = r#"
        const obj = {
            nested: nil as { value: number } | nil
        }
        const value = obj.nested?.value
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Optional in object literal should compile");
}

#[test]
fn test_optional_chaining_in_array_literal() {
    let source = r#"
        const arr = [
            { value: 1 },
            nil as { value: number } | nil,
            { value: 3 }
        ]
        const value = arr[1]?.value
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Optional in array literal should compile");
}

#[test]
fn test_optional_type_narrowing() {
    let source = r#"
        type T = { x: number } | nil
        const t: T = nil
        const x = t?.x
        const doubled = (x ?? 0) * 2
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Optional type narrowing should compile");
}

#[test]
fn test_optional_with_generic() {
    let source = r#"
        function getOrDefault<T>(value: T | nil, default: T): T {
            return value ?? default
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Optional with generic should compile");
}

#[test]
fn test_optional_chaining_complex() {
    let source = r#"
        type Company = {
            departments: {
                [string]: {
                    manager: {
                        name: string
                    } | nil
                } | nil
            } | nil
        } | nil

        const c: Company = nil
        const name = c?.departments?.["engineering"]?.manager?.name
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Complex optional chaining should compile");
}

#[test]
fn test_optional_with_method_chaining() {
    let source = r#"
        class Builder {
            public value: string = ""

            public append(s: string): Builder {
                self.value = self.value .. s
                return self
            }
        }

        const b: Builder | nil = nil
        const result = b?.append("hello")?.append("world")?.value
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Optional method chaining should compile");
}

#[test]
fn test_optional_preserves_types() {
    let source = r#"
        type Obj = { value: number } | nil
        const obj: Obj = nil
        const maybeNumber: number | nil = obj?.value
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Optional should preserve types");
}

#[test]
fn test_optional_in_conditional() {
    let source = r#"
        type Item = { active: boolean } | nil
        const item: Item = nil
        if item?.active == true {
            const x = 1
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Optional in conditional should compile");
}
