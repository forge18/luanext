use luanext_core::config::{CompilerConfig, OptimizationLevel};
use luanext_core::di::DiContainer;

fn compile_with_opt_level(source: &str, level: OptimizationLevel) -> Result<String, String> {
    let config = CompilerConfig::default();
    let mut container = DiContainer::production(config);
    container.compile_with_stdlib_and_optimization(source, level)
}

// ============================================================================
// Loop Unrolling Tests
// ============================================================================

#[test]
fn test_basic_loop_unrolling() {
    let source = r#"
        for i = 1, 3 do
            print(i)
        end
    "#;

    let output = compile_with_opt_level(source, OptimizationLevel::Aggressive).unwrap();

    // Loop should be unrolled into 3 print statements
    assert!(
        !output.contains("for"),
        "Loop should be unrolled and removed. Got:\n{}",
        output
    );

    // Should have 3 separate print calls with constants
    assert!(
        output.contains("print(1)"),
        "Should have print(1). Got:\n{}",
        output
    );
    assert!(
        output.contains("print(2)"),
        "Should have print(2). Got:\n{}",
        output
    );
    assert!(
        output.contains("print(3)"),
        "Should have print(3). Got:\n{}",
        output
    );
}

#[test]
fn test_loop_unrolling_with_step() {
    let source = r#"
        for i = 0, 6, 2 do
            print(i)
        end
    "#;

    let output = compile_with_opt_level(source, OptimizationLevel::Aggressive).unwrap();

    // Loop should be unrolled
    assert!(
        !output.contains("for"),
        "Loop should be unrolled. Got:\n{}",
        output
    );

    // Should have prints for 0, 2, 4, 6
    assert!(
        output.contains("print(0)"),
        "Should have print(0). Got:\n{}",
        output
    );
    assert!(
        output.contains("print(2)"),
        "Should have print(2). Got:\n{}",
        output
    );
    assert!(
        output.contains("print(4)"),
        "Should have print(4). Got:\n{}",
        output
    );
    assert!(
        output.contains("print(6)"),
        "Should have print(6). Got:\n{}",
        output
    );
}

#[test]
fn test_loop_unrolling_with_negative_step() {
    let source = r#"
        for i = 3, 1, -1 do
            print(i)
        end
    "#;

    let output = compile_with_opt_level(source, OptimizationLevel::Aggressive).unwrap();

    // Loop should be unrolled
    assert!(
        !output.contains("for"),
        "Loop should be unrolled. Got:\n{}",
        output
    );

    // Should have prints for 3, 2, 1
    assert!(
        output.contains("print(3)"),
        "Should have print(3). Got:\n{}",
        output
    );
    assert!(
        output.contains("print(2)"),
        "Should have print(2). Got:\n{}",
        output
    );
    assert!(
        output.contains("print(1)"),
        "Should have print(1). Got:\n{}",
        output
    );
}

#[test]
fn test_loop_unrolling_not_applied_for_large_loops() {
    let source = r#"
        for i = 1, 10 do
            print(i)
        end
    "#;

    let output = compile_with_opt_level(source, OptimizationLevel::Aggressive).unwrap();

    // Loop should NOT be unrolled (trip count > 4)
    assert!(
        output.contains("for"),
        "Large loop should not be unrolled. Got:\n{}",
        output
    );
}

#[test]
fn test_loop_unrolling_not_applied_with_break() {
    let source = r#"
        for i = 1, 3 do
            if i == 2 then
                break
            end
            print(i)
        end
    "#;

    let output = compile_with_opt_level(source, OptimizationLevel::Aggressive).unwrap();

    // Loop should NOT be unrolled (contains break statement)
    assert!(
        output.contains("for"),
        "Loop with break should not be unrolled. Got:\n{}",
        output
    );
}

#[test]
fn test_loop_unrolling_not_applied_with_continue() {
    let source = r#"
        for i = 1, 3 do
            if i == 2 then
                continue
            end
            print(i)
        end
    "#;

    let output = compile_with_opt_level(source, OptimizationLevel::Aggressive).unwrap();

    // Loop should NOT be unrolled (contains continue statement)
    assert!(
        output.contains("for"),
        "Loop with continue should not be unrolled. Got:\n{}",
        output
    );
}

#[test]
fn test_loop_unrolling_not_applied_with_return() {
    let source = r#"
        function test(): void {
            for i = 1, 3 do
                if i == 2 then
                    return
                end
                print(i)
            end
        }
    "#;

    let output = compile_with_opt_level(source, OptimizationLevel::Aggressive).unwrap();

    // Loop should NOT be unrolled (contains return statement)
    assert!(
        output.contains("for"),
        "Loop with return should not be unrolled. Got:\n{}",
        output
    );
}

#[test]
fn test_loop_unrolling_with_non_constant_bounds() {
    let source = r#"
        const n = 3;
        for i = 1, n do
            print(i)
        end
    "#;

    let output = compile_with_opt_level(source, OptimizationLevel::Aggressive).unwrap();

    // Loop WILL be unrolled because constant folding runs first and turns n into 3
    // This is actually correct behavior - the optimizer sees constant bounds
    assert!(
        !output.contains("for"),
        "Loop should be unrolled after constant folding. Got:\n{}",
        output
    );
    assert!(
        output.contains("print(1)"),
        "Should have print(1). Got:\n{}",
        output
    );
}

#[test]
fn test_loop_unrolling_with_arithmetic_in_body() {
    let source = r#"
        for i = 1, 3 do
            print(i * 2)
        end
    "#;

    let output = compile_with_opt_level(source, OptimizationLevel::Aggressive).unwrap();

    // Loop should be unrolled with arithmetic substitution
    assert!(
        !output.contains("for"),
        "Loop should be unrolled. Got:\n{}",
        output
    );

    // Should have substituted values (constant folding may or may not run after)
    assert!(
        output.contains("print((1 * 2))") || output.contains("print(2)"),
        "Should have print((1 * 2)) or print(2). Got:\n{}",
        output
    );
    assert!(
        output.contains("print((2 * 2))") || output.contains("print(4)"),
        "Should have print((2 * 2)) or print(4). Got:\n{}",
        output
    );
    assert!(
        output.contains("print((3 * 2))") || output.contains("print(6)"),
        "Should have print((3 * 2)) or print(6). Got:\n{}",
        output
    );
}

#[test]
fn test_loop_unrolling_idempotence() {
    let source = r#"
        for i = 1, 3 do
            print(i)
        end
    "#;

    let output1 = compile_with_opt_level(source, OptimizationLevel::Aggressive).unwrap();

    // Compile the already-unrolled code again
    let output2 = compile_with_opt_level(&output1, OptimizationLevel::Aggressive).unwrap();

    // Should produce the same output (idempotent)
    assert_eq!(
        output1.trim(),
        output2.trim(),
        "Loop unrolling should be idempotent"
    );
}

#[test]
fn test_loop_unrolling_nested_loops() {
    let source = r#"
        for i = 1, 2 do
            for j = 1, 2 do
                print(i + j)
            end
        end
    "#;

    let output = compile_with_opt_level(source, OptimizationLevel::Aggressive).unwrap();

    // Both loops should be unrolled
    assert!(
        !output.contains("for"),
        "Both nested loops should be unrolled. Got:\n{}",
        output
    );

    // Should have 4 print statements (2x2 iterations)
    let print_count = output.matches("print(").count();
    assert_eq!(
        print_count, 4,
        "Should have 4 print statements. Got:\n{}",
        output
    );
}

#[test]
fn test_loop_unrolling_single_iteration() {
    let source = r#"
        for i = 5, 5 do
            print(i)
        end
    "#;

    let output = compile_with_opt_level(source, OptimizationLevel::Aggressive).unwrap();

    // Single iteration loop should be unrolled
    assert!(
        !output.contains("for"),
        "Single iteration loop should be unrolled. Got:\n{}",
        output
    );

    assert!(
        output.contains("print(5)"),
        "Should have print(5). Got:\n{}",
        output
    );
}

#[test]
fn test_loop_unrolling_zero_iterations() {
    let source = r#"
        for i = 10, 5 do
            print(i)
        end
    "#;

    let output = compile_with_opt_level(source, OptimizationLevel::Aggressive).unwrap();

    // Zero iteration loop should be removed entirely
    assert!(
        !output.contains("print"),
        "Zero iteration loop should have no print statements. Got:\n{}",
        output
    );
}

#[test]
fn test_generic_for_loop_not_unrolled() {
    let source = r#"
        const arr = {1, 2, 3};
        for i, v in ipairs(arr) do
            print(v)
        end
    "#;

    let output = compile_with_opt_level(source, OptimizationLevel::Aggressive).unwrap();

    // Generic for-loop should NOT be unrolled (iterator state is opaque)
    assert!(
        output.contains("for"),
        "Generic for-loop should not be unrolled. Got:\n{}",
        output
    );
}

#[test]
fn test_loop_unrolling_only_at_o3() {
    let source = r#"
        for i = 1, 3 do
            print(i)
        end
    "#;

    // Should NOT unroll at O2
    let output_o2 = compile_with_opt_level(source, OptimizationLevel::Moderate).unwrap();
    assert!(
        output_o2.contains("for"),
        "Loop should not be unrolled at O2. Got:\n{}",
        output_o2
    );

    // Should unroll at O3
    let output_o3 = compile_with_opt_level(source, OptimizationLevel::Aggressive).unwrap();
    assert!(
        !output_o3.contains("for"),
        "Loop should be unrolled at O3. Got:\n{}",
        output_o3
    );
}
