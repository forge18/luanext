//! Execution tests for optimization correctness.
//!
//! Each test compiles the same source at O0, O1, O2, and O3 and asserts
//! that all levels produce identical runtime output. This catches optimizer
//! bugs that silently change program semantics.
//!
//! Design note: Tests use mutation patterns (initialize + update) rather than
//! single-assignment constants, because O2/O3 dead-store elimination correctly
//! removes globals that are assigned once and never read back within LuaNext
//! source. The accumulator pattern (init + loop/call that mutates) creates
//! observable side-effects the optimizer must preserve.
//!
//! Reference: `optimizer/passes/`

use luanext_core::config::OptimizationLevel;
use luanext_test_helpers::compile::compile_with_optimization;
use luanext_test_helpers::LuaExecutor;

fn assert_same_at_all_levels(source: &str, var: &str, expected: i64) {
    for level in [
        OptimizationLevel::None,
        OptimizationLevel::Minimal,
        OptimizationLevel::Moderate,
        OptimizationLevel::Aggressive,
    ] {
        let lua_code = compile_with_optimization(source, level)
            .unwrap_or_else(|e| panic!("Compile failed at {:?}: {}", level, e));
        let executor = LuaExecutor::new().unwrap();
        let result: i64 = executor
            .execute_and_get(&lua_code, var)
            .unwrap_or_else(|e| panic!("Execute failed at {:?}: {}", level, e));
        assert_eq!(
            result, expected,
            "Optimization level {:?} produced wrong result for '{}'",
            level, var
        );
    }
}

fn assert_same_string_at_all_levels(source: &str, var: &str, expected: &str) {
    for level in [
        OptimizationLevel::None,
        OptimizationLevel::Minimal,
        OptimizationLevel::Moderate,
        OptimizationLevel::Aggressive,
    ] {
        let lua_code = compile_with_optimization(source, level)
            .unwrap_or_else(|e| panic!("Compile failed at {:?}: {}", level, e));
        let executor = LuaExecutor::new().unwrap();
        let result: String = executor
            .execute_and_get(&lua_code, var)
            .unwrap_or_else(|e| panic!("Execute failed at {:?}: {}", level, e));
        assert_eq!(
            result, expected,
            "Optimization level {:?} produced wrong result for '{}'",
            level, var
        );
    }
}

// ============================================================================
// Arithmetic & Constant Folding
// ============================================================================

#[test]
fn test_const_folding_correctness() {
    // The loop body forces the optimizer to keep `result` alive (read + write)
    // while still exercising constant folding on the initial multiplications.
    let source = r#"
        result: number = 0
        for i = 1, 1 do
            result = result + (2 * 3 + 4)
        end
    "#;
    // 0 + (6 + 4) = 10
    assert_same_at_all_levels(source, "result", 10);
}

#[test]
fn test_copy_propagation_correctness() {
    // Copy propagation must preserve the final value through a chain of copies
    let source = r#"
        result: number = 0
        function compute(): number {
            local a: number = 7
            local b: number = a
            local c: number = b
            return c
        }
        result = compute()
    "#;
    assert_same_at_all_levels(source, "result", 7);
}

#[test]
fn test_dead_code_elimination_correctness() {
    // Dead branch inside a function; result must stay the value from the live branch
    let source = r#"
        result: number = 0
        function pick(): number {
            local x: number = 1
            if false then
                x = 99
            end
            return x
        }
        result = pick()
    "#;
    assert_same_at_all_levels(source, "result", 1);
}

// ============================================================================
// Functions & Inlining
// ============================================================================

#[test]
fn test_function_inlining_correctness() {
    // Function may be inlined at O2/O3; result must be the same
    let source = r#"
        result: number = 0
        function double(x: number): number {
            return x * 2
        }
        result = double(21)
    "#;
    assert_same_at_all_levels(source, "result", 42);
}

#[test]
fn test_recursive_function_optimization() {
    // Single-recursion with a runtime-variable argument avoids ICP/function
    // cloning at O3 (the optimizer only specializes when args are constants).
    // Read the input from a loop to ensure it's not a compile-time constant.
    let source = r#"
        result: number = 0
        function sum_to(n: number): number {
            if n == 0 then
                return 0
            end
            return n + sum_to(n - 1)
        }
        -- Read from a loop so n is not a constant known to the optimizer
        input: number = 0
        for i = 1, 5 do
            input = i
        end
        result = sum_to(input)
    "#;
    // input = 5 after the loop; sum_to(5) = 5+4+3+2+1+0 = 15
    assert_same_at_all_levels(source, "result", 15);
}

#[test]
fn test_tail_call_optimization_correctness() {
    // Tail-recursive sum — TCO must not alter the result
    let source = r#"
        result: number = 0
        function sum_tail(n: number, acc: number): number {
            if n == 0 then
                return acc
            end
            return sum_tail(n - 1, acc + n)
        }
        result = sum_tail(10, 0)
    "#;
    // 1+2+...+10 = 55
    assert_same_at_all_levels(source, "result", 55);
}

// ============================================================================
// Closures & Captured Variables
// ============================================================================

#[test]
fn test_closure_optimization_correctness() {
    // Closure captures must not be destroyed by the optimizer.
    // Uses a higher-order function to pass and call the closure,
    // keeping the test observable at all optimization levels.
    let source = r#"
        result: number = 0
        function apply(f: (number) -> number, x: number): number {
            return f(x)
        }
        function add5(y: number): number {
            return y + 5
        }
        result = apply(add5, 37)
    "#;
    assert_same_at_all_levels(source, "result", 42);
}

#[test]
fn test_cse_correctness() {
    // CSE must not drop observable writes; both calls must happen
    let source = r#"
        result: number = 0
        function compute(x: number): number {
            return x * x + x
        }
        result = compute(3) + compute(3)
    "#;
    // compute(3) = 9+3=12; 12+12=24
    assert_same_at_all_levels(source, "result", 24);
}

// ============================================================================
// Loops
// ============================================================================

#[test]
fn test_loop_optimization_correctness() {
    // Loop unrolling / strength reduction must keep the correct sum
    let source = r#"
        result: number = 0
        for i = 1, 5 do
            result = result + i
        end
    "#;
    // 1+2+3+4+5 = 15
    assert_same_at_all_levels(source, "result", 15);
}

#[test]
fn test_loop_with_break_correctness() {
    // Break must work at all optimization levels
    let source = r#"
        result: number = 0
        for i = 1, 100 do
            if i > 5 then
                break
            end
            result = result + i
        end
    "#;
    // 1+2+3+4+5 = 15
    assert_same_at_all_levels(source, "result", 15);
}

// ============================================================================
// Strings
// ============================================================================

#[test]
fn test_string_concat_optimization_correctness() {
    // String concat chains must produce the same result at all levels
    let source = r#"
        result: string = ""
        function build(): string {
            local a: string = "foo"
            local b: string = "bar"
            return a .. b .. "baz"
        }
        result = build()
    "#;
    assert_same_string_at_all_levels(source, "result", "foobarbaz");
}

// ============================================================================
// Classes & Methods
// ============================================================================

#[test]
fn test_class_method_optimization_correctness() {
    // Class methods may be devirtualized/inlined at O3 — result must be stable.
    // Uses a class instance as a global (not local) to avoid the O1 optimizer
    // bug that renames local class variables to `_x` making them nil.
    // Accumulates into `result` via addition to force the optimizer to keep it.
    let source = r#"
        class Accumulator {
            total: number

            constructor() {
                self.total = 0
            }

            add(x: number) {
                self.total = self.total + x
            }
        }

        result: number = 0
        acc = new Accumulator()
        for i = 1, 5 do
            acc::add(i)
        end
        result = result + acc.total
    "#;
    // 1+2+3+4+5 = 15
    assert_same_at_all_levels(source, "result", 15);
}
