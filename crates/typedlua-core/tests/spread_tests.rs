use typedlua_core::di::DiContainer;

fn compile_and_check(source: &str) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile_with_stdlib(source)
}

#[test]
fn test_simple_array_spread() {
    let source = r#"
        const arr1 = [1, 2]
        const arr2 = [3, 4]
        const combined = [...arr1, ...arr2]
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Simple array spread should compile");
    let output = result.unwrap();

    assert!(output.contains("(function()"), "Should generate IIFE");
}

#[test]
fn test_array_spread_multiple() {
    let source = r#"
        const a = [1]
        const b = [2]
        const c = [3]
        const combined = [...a, ...b, ...c]
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Multiple array spread should compile");
}

#[test]
fn test_array_spread_nested() {
    let source = r#"
        const arr = [[1, 2], [3, 4]]
        const flat = [...arr[0], ...arr[1]]
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Nested array spread should compile");
}

#[test]
fn test_array_spread_with_values() {
    let source = r#"
        const arr = [3, 4]
        const combined = [1, 2, ...arr, 5, 6]
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Array spread with values should compile");
}

#[test]
fn test_object_spread() {
    let source = r#"
        const obj1 = { a: 1, b: 2 }
        const obj2 = { c: 3 }
        const combined = { ...obj1, ...obj2 }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Object spread should compile");
}

#[test]
fn test_object_spread_override() {
    let source = r#"
        const obj1 = { a: 1, b: 2 }
        const obj2 = { a: 3, c: 4 }
        const combined = { ...obj1, ...obj2 }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Object spread override should compile");
}

#[test]
fn test_object_spread_with_values() {
    let source = r#"
        const obj = { x: 1 }
        const combined = { y: 2, ...obj, z: 3 }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Object spread with values should compile");
}

#[test]
fn test_mixed_spread() {
    let source = r#"
        const arr = [3, 4]
        const combined = { a: 1, ...{ b: 2 }, ...arr }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Mixed spread should compile");
}

#[test]
fn test_spread_in_function_call() {
    let source = r#"
        const nums = [1, 2, 3]
        const sum = table.unpack(...nums)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Spread in function call should compile");
}

#[test]
fn test_spread_in_constructor() {
    let source = r#"
        class Point {
            x: number
            y: number
            z: number

            constructor(x: number, y: number, z: number) {
                self.x = x
                self.y = y
                self.z = z
            }
        }

        const coords = [1, 2, 3]
        const p = new Point(...coords)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Spread in constructor should compile");
}

#[test]
fn test_spread_assignment() {
    let source = r#"
        const source = { a: 1, b: 2, c: 3 }
        const { a, ...rest } = source
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Spread assignment should compile");
}

#[test]
fn test_nested_spread() {
    let source = r#"
        const obj = { outer: { inner: { value: 1 } } }
        const { outer: { inner: { value }, ...innerRest }, ...outerRest } = obj
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Nested spread should compile");
}

#[test]
fn test_spread_with_generic() {
    let source = r#"
        function concat<T>(a: T[], b: T[]): T[] {
            return [...a, ...b]
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Spread with generic should compile");
}

#[test]
fn test_spread_in_array_comprehension() {
    let source = r#"
        const arr1 = [1, 2, 3]
        const arr2 = [4, 5, 6]
        const combined = [...arr1, ...arr2].map(n => n * 2)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Spread in comprehension should compile");
}

#[test]
fn test_spread_type_annotation() {
    let source = r#"
        const arr: number[] = [...[1, 2, 3]]
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Spread with type annotation should compile");
}

#[test]
fn test_spread_empty_array() {
    let source = r#"
        const empty: number[] = []
        const combined = [...empty, 1, 2, 3]
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Spread empty array should compile");
}

#[test]
fn test_spread_empty_object() {
    let source = r#"
        const empty = {}
        const combined = { ...empty, a: 1 }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Spread empty object should compile");
}

#[test]
fn test_spread_with_string_keys() {
    let source = r#"
        const obj = { ["key" .. "1"]: 1, ["key" .. "2"]: 2 }
        const combined = { ...obj }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Spread with string keys should compile");
}

#[test]
fn test_spread_in_table_constructor() {
    let source = r#"
        const keys = ["a", "b", "c"]
        const values = [1, 2, 3]
        const table = {
            ...keys.map((k, i) => [k, values[i]])
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Spread in table constructor should compile");
}

#[test]
fn test_spread_chain() {
    let source = r#"
        const a = { x: 1 }
        const b = { y: 2 }
        const c = { z: 3 }
        const combined = { ...a, ...b, ...c }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Spread chain should compile");
}
