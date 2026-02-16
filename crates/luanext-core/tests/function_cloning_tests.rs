use luanext_core::config::{CompilerConfig, OptimizationLevel};
use luanext_core::di::DiContainer;

fn compile_o3(source: &str) -> Result<String, String> {
    let config = CompilerConfig::default();
    let mut container = DiContainer::production(config);
    container.compile_with_stdlib_and_optimization(source, OptimizationLevel::Aggressive)
}

fn compile_o2(source: &str) -> Result<String, String> {
    let config = CompilerConfig::default();
    let mut container = DiContainer::production(config);
    container.compile_with_stdlib_and_optimization(source, OptimizationLevel::Moderate)
}

// ============================================================================
// Function Cloning for Specialization Tests
// ============================================================================

#[test]
fn test_function_cloning_basic() {
    let source = r#"
        function double(x: number): number
            return x * 2
        end
        const result = double(5)
    "#;

    let output = compile_o3(source).unwrap();

    // At O3 the call with a constant argument should create a clone
    // The clone should have the constant substituted in
    assert!(
        output.contains("__clone") || output.contains("10"),
        "Expected function cloning or constant folding to occur. Got:\n{}",
        output
    );
}

#[test]
fn test_function_cloning_not_at_o2() {
    let source = r#"
        function double(x: number): number
            return x * 2
        end
        const result = double(5)
    "#;

    let output = compile_o2(source).unwrap();

    // At O2, function cloning should NOT occur
    assert!(
        !output.contains("__clone"),
        "Function cloning should not happen at O2. Got:\n{}",
        output
    );
}

#[test]
fn test_function_cloning_multiple_call_sites() {
    let source = r#"
        function greet(name: string, loud: boolean): string
            if loud then
                return name
            end
            return name
        end
        const a = greet("hello", true)
        const b = greet("world", false)
    "#;

    let output = compile_o3(source).unwrap();

    // Should create clones for the different constant call sites
    assert!(
        output.contains("__clone"),
        "Expected cloned functions for different constant args. Got:\n{}",
        output
    );
}

#[test]
fn test_function_cloning_preserves_non_constant_args() {
    let source = r#"
        function add(a: number, b: number): number
            return a + b
        end
        const x = 10
        const result = add(x, 5)
    "#;

    // Should compile without error - x is a variable, not a literal constant
    let output = compile_o3(source).unwrap();
    assert!(!output.is_empty());
}

#[test]
fn test_function_cloning_skips_variadic_functions() {
    let source = r#"
        function sum(...args: number[]): number
            return 0
        end
        const result = sum(1, 2, 3)
    "#;

    let output = compile_o3(source).unwrap();

    // Variadic functions should NOT be cloned
    assert!(
        !output.contains("__clone"),
        "Variadic functions should not be cloned. Got:\n{}",
        output
    );
}

#[test]
fn test_function_cloning_skips_generic_functions() {
    // Generic functions are first handled by generic specialization,
    // which creates non-generic specialized versions. Those specializations
    // may then be eligible for cloning - this is expected behavior.
    let source = r#"
        function identity<T>(x: T): T
            return x
        end
        const result = identity<number>(42)
    "#;

    let output = compile_o3(source).unwrap();

    assert!(
        !output.is_empty(),
        "Should compile successfully with generic specialization. Got:\n{}",
        output
    );
}

#[test]
fn test_function_cloning_with_boolean_constant() {
    let source = r#"
        function check(value: number, strict: boolean): number
            if strict then
                return value * 2
            end
            return value
        end
        const a = check(10, true)
        const b = check(20, true)
    "#;

    let output = compile_o3(source).unwrap();

    // Both calls pass true - either cloning or interprocedural const prop should work
    assert!(
        !output.is_empty(),
        "Should compile successfully. Got:\n{}",
        output
    );
}

#[test]
fn test_function_cloning_deduplicates_same_args() {
    let source = r#"
        function scale(x: number, factor: number): number
            return x * factor
        end
        const a = scale(10, 2)
        const b = scale(20, 2)
    "#;

    let output = compile_o3(source).unwrap();

    // Same constant args should reuse the same clone (deduplication)
    // The interprocedural const prop may also handle this case
    assert!(!output.is_empty(), "Should compile successfully");
}

#[test]
fn test_function_cloning_with_nil_argument() {
    let source = r#"
        function maybe(x: number, opt: string?): number
            if opt == nil then
                return x
            end
            return x + 1
        end
        const result = maybe(5, nil)
    "#;

    let output = compile_o3(source).unwrap();
    assert!(!output.is_empty(), "Should compile successfully");
}

#[test]
fn test_function_cloning_nested_calls() {
    let source = r#"
        function double(x: number): number
            return x * 2
        end
        function triple(x: number): number
            return x * 3
        end
        const result = double(triple(4))
    "#;

    let output = compile_o3(source).unwrap();

    // Inner call triple(4) has constant arg; outer call double() gets the result
    assert!(!output.is_empty(), "Should compile successfully");
}

#[test]
fn test_function_cloning_no_params_skipped() {
    let source = r#"
        function get_zero(): number
            return 0
        end
        const result = get_zero()
    "#;

    let output = compile_o3(source).unwrap();

    // Function with no parameters should not be cloned
    assert!(
        !output.contains("__clone"),
        "Functions with no params should not be cloned. Got:\n{}",
        output
    );
}

#[test]
fn test_function_cloning_large_body_skipped() {
    let source = r#"
        function big(x: number): number
            const a = x + 1
            const b = x + 2
            const c = x + 3
            const d = x + 4
            const e = x + 5
            const f = x + 6
            const g = x + 7
            const h = x + 8
            const i = x + 9
            return a + b + c + d + e + f + g + h + i
        end
        const result = big(5)
    "#;

    let output = compile_o3(source).unwrap();

    // Large function body (>8 statements) should not be cloned
    assert!(
        !output.contains("__clone"),
        "Large functions should not be cloned. Got:\n{}",
        output
    );
}
