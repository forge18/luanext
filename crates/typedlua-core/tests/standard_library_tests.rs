use typedlua_core::di::DiContainer;

fn compile_and_check(source: &str) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile_with_stdlib(source)
}

#[test]
fn test_print_function() {
    let source = r#"
        print("Hello, World!")
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "print should work");
}

#[test]
fn test_typeof_function() {
    let source = r#"
        const n: number = 42
        const t = typeof(n)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "typeof should work");
}

#[test]
fn test_tonumber_function() {
    let source = r#"
        const s = "123"
        const n = tonumber(s)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "tonumber should work");
}

#[test]
fn test_tostring_function() {
    let source = r#"
        const n = 42
        const s = tostring(n)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "tostring should work");
}

#[test]
fn test_error_function() {
    let source = r#"
        error("Test error")
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "error should work");
}

#[test]
fn test_ipairs_function() {
    let source = r#"
        const arr = [1, 2, 3]
        for i, v in ipairs(arr) {
            print(i, v)
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "ipairs should work");
}

#[test]
fn test_pairs_function() {
    let source = r#"
        const obj = { a: 1, b: 2, c: 3 }
        for k, v in pairs(obj) {
            print(k, v)
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "pairs should work");
}

#[test]
fn test_next_function() {
    let source = r#"
        const obj = { a: 1, b: 2 }
        const k, v = next(obj)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "next should work");
}

#[test]
fn test_rawget_function() {
    let source = r#"
        const obj = { a: 1 }
        const v = rawget(obj, "a")
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "rawget should work");
}

#[test]
fn test_rawset_function() {
    let source = r#"
        const obj = {}
        rawset(obj, "a", 1)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "rawset should work");
}

#[test]
fn test_rawlen_function() {
    let source = r#"
        const arr = [1, 2, 3]
        const len = rawlen(arr)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "rawlen should work");
}

#[test]
fn test_table_insert() {
    let source = r#"
        const arr: number[] = []
        table.insert(arr, 1)
        table.insert(arr, 2)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "table.insert should work");
}

#[test]
fn test_table_remove() {
    let source = r#"
        const arr = [1, 2, 3]
        const v = table.remove(arr)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "table.remove should work");
}

#[test]
fn test_table_concat() {
    let source = r#"
        const arr = ["a", "b", "c"]
        const s = table.concat(arr, ",")
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "table.concat should work");
}

#[test]
fn test_table_sort() {
    let source = r#"
        const arr = [3, 1, 2]
        table.sort(arr)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "table.sort should work");
}

#[test]
fn test_math_abs() {
    let source = r#"
        const v = math.abs(-5)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "math.abs should work");
}

#[test]
fn test_math_floor() {
    let source = r#"
        const v = math.floor(3.7)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "math.floor should work");
}

#[test]
fn test_math_ceil() {
    let source = r#"
        const v = math.ceil(3.2)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "math.ceil should work");
}

#[test]
fn test_math_max() {
    let source = r#"
        const v = math.max(1, 2, 3, 4, 5)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "math.max should work");
}

#[test]
fn test_math_min() {
    let source = r#"
        const v = math.min(1, 2, 3, 4, 5)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "math.min should work");
}

#[test]
fn test_math_random() {
    let source = r#"
        const v = math.random()
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "math.random should work");
}

#[test]
fn test_math_sqrt() {
    let source = r#"
        const v = math.sqrt(16)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "math.sqrt should work");
}

#[test]
fn test_string_len() {
    let source = r#"
        const s = "hello"
        const len = #s
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "string length should work");
}

#[test]
fn test_string_sub() {
    let source = r#"
        const s = "hello"
        const sub = string.sub(s, 1, 3)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "string.sub should work");
}

#[test]
fn test_string_upper() {
    let source = r#"
        const s = "hello"
        const upper = string.upper(s)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "string.upper should work");
}

#[test]
fn test_string_lower() {
    let source = r#"
        const s = "HELLO"
        const lower = string.lower(s)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "string.lower should work");
}

#[test]
fn test_string_gmatch() {
    let source = r#"
        const s = "hello world"
        for word in string.gmatch(s, "%S+") {
            print(word)
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "string.gmatch should work");
}

#[test]
fn test_coroutine_create() {
    let source = r#"
        const co = coroutine.create(() => {
            print("running")
        })
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "coroutine.create should work");
}

#[test]
fn test_coroutine_resume() {
    let source = r#"
        const co = coroutine.create(() => {
            return 42
        })
        coroutine.resume(co)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "coroutine.resume should work");
}

#[test]
fn test_coroutine_yield() {
    let source = r#"
        const co = coroutine.create(() => {
            coroutine.yield(1)
            return 2
        })
        coroutine.resume(co)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "coroutine.yield should work");
}

#[test]
fn test_debug_getinfo() {
    let source = r#"
        function foo() end
        const info = debug.getinfo(foo)
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "debug.getinfo should work");
}

#[test]
fn test_debug_traceback() {
    let source = r#"
        const tb = debug.traceback()
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "debug.traceback should work");
}
