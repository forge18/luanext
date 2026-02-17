//! Execution tests for optional chaining operators:
//! - `obj?.prop`       (OptionalMember)
//! - `arr?.[0]`        (OptionalIndex)
//! - `fn?.()`          (OptionalCall)
//! - `obj?.::method()` (OptionalMethodCall)

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

#[test]
fn test_optional_member_nil() {
    // obj?.prop returns nil when obj is nil
    let source = r#"
        obj: { x: number } | nil = nil
        result: number | nil = obj?.x
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    // result should be nil - execute_ok just checks no error
    let is_nil = !executor.execute_ok(&format!("{}\nassert(result ~= nil)", lua_code));
    assert!(is_nil, "result should be nil when obj is nil");
}

#[test]
fn test_optional_member_value() {
    // obj?.prop returns the value when obj is not nil
    let source = r#"
        const pt = {x = 42, y = 10}
        result: number = pt?.x
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_optional_index_nil() {
    // arr?.[0] returns nil when arr is nil
    let source = r#"
        arr: number[] | nil = nil
        result: number | nil = arr?.[1]
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let is_nil = !executor.execute_ok(&format!("{}\nassert(result ~= nil)", lua_code));
    assert!(is_nil, "result should be nil when arr is nil");
}

#[test]
fn test_optional_index_value() {
    // arr?.[1] returns value when arr is not nil
    let source = r#"
        const nums = [10, 20, 30]
        result: number = nums?.[2]
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 20);
}

#[test]
fn test_optional_call_nil() {
    // fn?.() does nothing and returns nil when fn is nil
    let source = r#"
        fn: (() => number) | nil = nil
        result: number | nil = fn?.()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let is_nil = !executor.execute_ok(&format!("{}\nassert(result ~= nil)", lua_code));
    assert!(is_nil, "result should be nil when fn is nil");
}

#[test]
fn test_optional_call_value() {
    // fn?.() calls the function and returns its value when fn is not nil
    let source = r#"
        const get_answer = () => 42
        result: number = get_answer?.()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_optional_method_nil() {
    // nil?.::method() returns nil without error
    let source = r#"
        class Counter {
            count: number = 0
            constructor() {
                self.count = 0
            }
            increment(): number {
                self.count = self.count + 1
                return self.count
            }
        }
        c: Counter | nil = nil
        result: number | nil = c?.::increment()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let is_nil = !executor.execute_ok(&format!("{}\nassert(result ~= nil)", lua_code));
    assert!(is_nil, "result should be nil when object is nil");
}

#[test]
fn test_optional_method_value() {
    // obj?.::method() calls method and returns value when obj is not nil
    let source = r#"
        class Greeter {
            name: string
            constructor(n: string) {
                self.name = n
            }
            greet(): string {
                return "Hello, " .. self.name
            }
        }
        const g = new Greeter("World")
        result: string = g?.::greet()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "Hello, World");
}

#[test]
fn test_optional_chained() {
    // a?.b stops at nil, no nil dereference error
    let source = r#"
        const obj = {x = 99}
        const no_obj: {x: number} | nil = nil

        r1: number = obj?.x
        r2: number | nil = no_obj?.x
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let r1: i64 = executor.execute_and_get(&lua_code, "r1").unwrap();
    assert_eq!(r1, 99);

    let is_nil = !executor.execute_ok(&format!("{}\nassert(r2 ~= nil)", lua_code));
    assert!(is_nil, "r2 should be nil");
}

#[test]
fn test_optional_with_null_coalescing() {
    // obj?.prop ?? "default" - use default when nil chain
    let source = r#"
        const obj = {name = "Alice"}
        const no_obj: {name: string} | nil = nil

        r1: string = obj?.name ?? "unknown"
        r2: string = no_obj?.name ?? "unknown"
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let r1: String = executor.execute_and_get(&lua_code, "r1").unwrap();
    let r2: String = executor.execute_and_get(&lua_code, "r2").unwrap();
    assert_eq!(r1, "Alice");
    assert_eq!(r2, "unknown");
}
