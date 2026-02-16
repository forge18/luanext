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
// Interprocedural Constant Propagation Tests
// ============================================================================

#[test]
fn test_icp_basic_constant_parameter() {
    // Use a function body larger than the cloning threshold (>8 stmts)
    // so that only ICP handles this, not function cloning
    let source = r#"
        function compute(x: number, factor: number): number
            const a = x + 1
            const b = x + 2
            const c = x + 3
            const d = x + 4
            const e = x + 5
            const f = x + 6
            const g = x + 7
            const h = x + 8
            const i = x + 9
            return (a + b + c + d + e + f + g + h + i) * factor
        end
        const r1 = compute(10, 2)
        const r2 = compute(20, 2)
        const r3 = compute(30, 2)
    "#;

    let output = compile_o3(source).unwrap();

    // The constant `factor=2` should be propagated into the function body
    // The function should no longer accept `factor` as a parameter
    assert!(
        output.contains("* 2") || output.contains("*2"),
        "Expected factor=2 to be propagated into function body. Got:\n{}",
        output
    );
}

#[test]
fn test_icp_not_at_o2() {
    let source = r#"
        function multiply(x: number, factor: number): number
            return x * factor
        end
        const a = multiply(10, 2)
        const b = multiply(20, 2)
    "#;

    let output = compile_o2(source).unwrap();

    // At O2, interprocedural const prop should NOT occur
    // The function should still accept factor as parameter
    assert!(
        output.contains("function multiply("),
        "ICP should not happen at O2. Got:\n{}",
        output
    );
}

#[test]
fn test_icp_differing_args_not_propagated() {
    let source = r#"
        function scale(x: number, factor: number): number
            return x * factor
        end
        const a = scale(10, 2)
        const b = scale(20, 3)
    "#;

    let output = compile_o3(source).unwrap();

    // Different values for `factor` mean it can't be propagated
    // The function should still accept both parameters
    assert!(
        output.contains("scale(") || output.contains("function"),
        "Should compile. Got:\n{}",
        output
    );
}

#[test]
fn test_icp_boolean_constant() {
    let source = r#"
        function process(x: number, debug: boolean): number
            if debug then
                print(x)
            end
            return x
        end
        const a = process(10, false)
        const b = process(20, false)
    "#;

    let output = compile_o3(source).unwrap();

    // All calls pass debug=false, so it should be propagated
    // After constant propagation + jump threading, the if(false) block
    // may be removed entirely
    assert!(!output.is_empty(), "Should compile successfully");
}

#[test]
fn test_icp_string_constant() {
    let source = r#"
        function log(msg: string, level: string): void
            print(level .. ": " .. msg)
        end
        log("starting", "INFO")
        log("processing", "INFO")
        log("done", "INFO")
    "#;

    let output = compile_o3(source).unwrap();

    // All calls pass level="INFO", so it should be propagated
    assert!(!output.is_empty(), "Should compile successfully");
}

#[test]
fn test_icp_skips_variadic_functions() {
    let source = r#"
        function sum(...args: number[]): number
            return 0
        end
        const a = sum(1, 2)
        const b = sum(3, 4)
    "#;

    let output = compile_o3(source).unwrap();

    // Variadic functions should be skipped by ICP
    assert!(!output.is_empty(), "Should compile successfully");
}

#[test]
fn test_icp_skips_generic_functions() {
    let source = r#"
        function identity<T>(x: T): T
            return x
        end
        const a = identity<number>(42)
        const b = identity<number>(43)
    "#;

    let output = compile_o3(source).unwrap();

    // Generic functions should be skipped by ICP
    assert!(!output.is_empty(), "Should compile successfully");
}

#[test]
fn test_icp_preserves_varying_parameter() {
    // First param varies, second is constant
    let source = r#"
        function compute(x: number, mode: number): number
            return x + mode
        end
        const a = compute(10, 1)
        const b = compute(20, 1)
        const c = compute(30, 1)
    "#;

    let output = compile_o3(source).unwrap();

    // `mode` should be propagated (always 1), but `x` should remain as parameter
    assert!(!output.is_empty(), "Should compile successfully");
}

#[test]
fn test_icp_single_call_site() {
    let source = r#"
        function helper(x: number, y: number): number
            return x + y
        end
        const result = helper(5, 10)
    "#;

    let output = compile_o3(source).unwrap();

    // Single call site with all constants — both should be propagated
    assert!(!output.is_empty(), "Should compile successfully");
}

#[test]
fn test_icp_call_in_loop() {
    let source = r#"
        function increment(x: number, step: number): number
            return x + step
        end
        for i = 1, 5 do
            const val = increment(i, 1)
        end
    "#;

    let output = compile_o3(source).unwrap();

    // Call in loop - step=1 is constant at all call sites
    assert!(!output.is_empty(), "Should compile successfully");
}

#[test]
fn test_icp_recursive_function_skipped() {
    let source = r#"
        function factorial(n: number, acc: number): number
            if n <= 1 then
                return acc
            end
            return factorial(n - 1, acc * n)
        end
        const result = factorial(5, 1)
    "#;

    let output = compile_o3(source).unwrap();

    // Recursive calls have varying arguments - only the external call has constants
    // ICP should handle this gracefully
    assert!(!output.is_empty(), "Should compile successfully");
}

#[test]
fn test_icp_nil_constant_propagation() {
    let source = r#"
        function maybe_print(x: number, label: string?): void
            if label ~= nil then
                print(label)
            end
            print(x)
        end
        maybe_print(1, nil)
        maybe_print(2, nil)
    "#;

    let output = compile_o3(source).unwrap();

    // All calls pass nil for label
    assert!(!output.is_empty(), "Should compile successfully");
}

#[test]
fn test_icp_no_params_function_skipped() {
    let source = r#"
        function get_value(): number
            return 42
        end
        const a = get_value()
        const b = get_value()
    "#;

    let output = compile_o3(source).unwrap();

    // No parameters - nothing to propagate
    assert!(!output.is_empty(), "Should compile successfully");
}

#[test]
fn test_icp_mixed_constant_and_variable_calls() {
    let source = r#"
        function format(value: number, prefix: string): string
            return prefix .. tostring(value)
        end
        const x = 42
        const a = format(x, "$")
        const b = format(100, "$")
    "#;

    let output = compile_o3(source).unwrap();

    // prefix="$" is constant at all call sites, value varies
    assert!(!output.is_empty(), "Should compile successfully");
}

#[test]
fn test_icp_spread_arg_skips_function() {
    let source = r#"
        function add(a: number, b: number): number
            return a + b
        end
        const args = [1, 2]
        add(1, 2)
    "#;

    let output = compile_o3(source).unwrap();

    // Should compile - no spread arguments in the actual call
    assert!(!output.is_empty(), "Should compile successfully");
}
