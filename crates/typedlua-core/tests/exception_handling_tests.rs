use typedlua_core::di::DiContainer;

fn compile_and_check(source: &str) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile_with_stdlib(source)
}

#[test]
fn test_throw_statement() {
    let source = r#"
        throw "error message"
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Throw statement should compile");
}

#[test]
fn test_try_catch_basic() {
    let source = r#"
        try {
            throw "error"
        } catch e {
            const message = e
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Try-catch should compile");
}

#[test]
fn test_try_catch_with_finally() {
    let source = r#"
        let cleaned = false
        try {
            throw "error"
        } catch e {
            const msg = e
        } finally {
            cleaned = true
        }
        return cleaned
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Try-catch-finally should compile");
}

#[test]
fn test_nested_try_catch() {
    let source = r#"
        try {
            try {
                throw "inner"
            } catch e {
                throw "outer"
            }
        } catch e2 {
            const msg = e2
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Nested try-catch should compile");
}

#[test]
fn test_try_with_multiple_catch() {
    let source = r#"
        try {
            throw "error"
        } catch e if typeof(e) == "string" {
            const s = e
        } catch e if typeof(e) == "number" {
            const n = e
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Multiple catch clauses should compile");
}

#[test]
fn test_catch_with_type_guard() {
    let source = r#"
        try {
            throw 42
        } catch e if typeof(e) == "number" {
            const num: number = e
        } catch e {
            const msg: string = e
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Catch with type guard should compile");
}

#[test]
fn test_rethrow_exception() {
    let source = r#"
        try {
            try {
                throw "original"
            } catch e {
                throw "rethrown"
            }
        } catch e2 {
            const msg = e2
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Rethrow should compile");
}

#[test]
fn test_try_resource_pattern() {
    let source = r#"
        let acquired = false
        let released = false
        try {
            acquired = true
        } finally {
            released = true
        }
        return acquired and released
    "#;

    let result = compile_and_check(source);
    assert!(
        result.is_ok(),
        "Try-finally resource pattern should compile"
    );
}

#[test]
fn test_catch_union_type() {
    let source = r#"
        try {
            throw "error"
        } catch e: string | number {
            const value = e
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Catch with union type should compile");
}

#[test]
fn test_try_in_loop() {
    let source = r#"
        let success = false
        for i in [1, 2, 3] {
            try {
                if i == 2 {
                    throw "skip"
                }
            } catch e {
                continue
            }
        }
        success = true
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Try in loop should compile");
}

#[test]
fn test_throw_in_function() {
    let source = r#"
        function fail(msg: string): never {
            throw msg
        }

        try {
            fail("test")
        } catch e {
            const m = e
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Throw in function should compile");
}

#[test]
fn test_throw_custom_error() {
    let source = r#"
        class MyError {
            message: string
        }

        throw new MyError()
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Throw custom error should compile");
}

#[test]
fn test_try_catch_return() {
    let source = r#"
        function f(): number {
            try {
                throw "error"
            } catch e {
                return 42
            }
        }
        return f()
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Try-catch with return should compile");
}

#[test]
fn test_finally_with_return() {
    let source = r#"
        function f(): number {
            try {
                return 1
            } finally {
                const cleanup = true
            }
        }
        return f()
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Finally with return should compile");
}

#[test]
fn test_catch_in_method() {
    let source = r#"
        class Handler {
            public process(): void {
                try {
                    throw "error"
                } catch e {
                    const msg = e
                }
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Catch in method should compile");
}

#[test]
fn test_throw_expression() {
    let source = r#"
        const f = () => {
            throw "error"
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Throw in arrow function should compile");
}

#[test]
fn test_nested_finally() {
    let source = r#"
        let outer = false
        let inner = false
        try {
            try {
            } finally {
                inner = true
            }
        } finally {
            outer = true
        }
        return outer and inner
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Nested finally should compile");
}

#[test]
fn test_try_catch_types() {
    let source = r#"
        try {
            throw 123
        } catch e: number {
            const n = e
        } catch e: string {
            const s = e
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Typed catch clauses should compile");
}

#[test]
fn test_error_with_stack_trace() {
    let source = r#"
        function deepStack(): void {
            function level1(): void {
                function level2(): void {
                    throw "error at level 2"
                }
                level2()
            }
            level1()
        }

        try {
            deepStack()
        } catch e {
            const msg = e
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Stack trace in error should compile");
}
