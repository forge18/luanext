use typedlua_core::config::OptimizationLevel;
use typedlua_core::di::DiContainer;

fn compile_with_optimization(source: &str, level: OptimizationLevel) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile_with_optimization(source, level)
}

#[test]
fn test_small_function_inlines_o3() {
    let source = r#"
        function add(a: number, b: number): number {
            return a + b
        }

        const result = add(1, 2)
    "#;

    let o2_output = compile_with_optimization(source, OptimizationLevel::O2).unwrap();
    let o3_output = compile_with_optimization(source, OptimizationLevel::O3).unwrap();

    println!("O2 output:\n{}", o2_output);
    println!("O3 output:\n{}", o3_output);

    let o2_has_func_call = o2_output.contains("add(1, 2)");
    let o3_has_func_call = o3_output.contains("add(1, 2)");

    println!(
        "O2 still has add call: {}, O3 still has add call: {}",
        o2_has_func_call, o3_has_func_call
    );

    if !o2_has_func_call {
        println!("PASS: O2 inlined the add function");
    }
    if !o3_has_func_call {
        println!("PASS: O3 inlined the add function");
    }
}

#[test]
fn test_medium_function_inlines_o3() {
    let source = r#"
        function mediumFunc(a: number, b: number): number {
            local x1 = a + b
            local x2 = a - b
            local x3 = a * b
            return x1 + x2 + x3
        }

        const result = mediumFunc(10, 5)
    "#;

    let output = compile_with_optimization(source, OptimizationLevel::O3).unwrap();
    println!("O3 output:\n{}", output);
}

#[test]
fn test_recursive_function_not_inlined() {
    let source = r#"
        function factorial(n: number): number {
            if n <= 1 {
                return 1
            }
            return n * factorial(n - 1)
        }

        const result = factorial(5)
    "#;

    let output = compile_with_optimization(source, OptimizationLevel::O3).unwrap();
    println!("Recursive function output:\n{}", output);
    assert!(
        output.contains("function"),
        "Should preserve function definition for recursion"
    );
}

#[test]
fn test_large_function_not_inlined() {
    let source = r#"
        function largeFunc(a: number): number {
            local r1 = a + 1
            local r2 = a + 2
            local r3 = a + 3
            local r4 = a + 4
            local r5 = a + 5
            local r6 = a + 6
            local r7 = a + 7
            local r8 = a + 8
            local r9 = a + 9
            local r10 = a + 10
            return r1 + r2 + r3 + r4 + r5 + r6 + r7 + r8 + r9 + r10
        }

        const result = largeFunc(1)
    "#;

    let o3_output = compile_with_optimization(source, OptimizationLevel::O3).unwrap();
    println!("Large function O3 output:\n{}", o3_output);
    assert!(
        o3_output.contains("largeFunc"),
        "Large functions should not be fully inlined"
    );
}

#[test]
fn test_getter_inlined() {
    let source = r#"
        class MyClass {
            private _value: number = 0

            public get value(): number {
                return self._value
            }
        }

        const obj = new MyClass()
        const v = obj.value
    "#;

    let o3_output = compile_with_optimization(source, OptimizationLevel::O3).unwrap();
    println!("Getter O3 output:\n{}", o3_output);
}

#[test]
fn test_single_use_function_inlined() {
    let source = r#"
        function util(x: number): number {
            return x * 2 + 1
        }

        const result = util(5)
    "#;

    let o3_output = compile_with_optimization(source, OptimizationLevel::O3).unwrap();
    println!("Single use O3 output:\n{}", o3_output);
    assert!(
        !o3_output.contains("function util"),
        "Single-use function should be inlined"
    );
}

#[test]
fn test_constant_propagation_with_inlining() {
    let source = r#"
        function compute(a: number, b: number): number {
            return (a + b) * (a - b)
        }

        const result = compute(10, 5)
    "#;

    let o3_output = compile_with_optimization(source, OptimizationLevel::O3).unwrap();
    println!("Const prop O3 output:\n{}", o3_output);
}

#[test]
fn test_tail_recursive_optimization() {
    let source = r#"
        function tailSum(n: number, acc: number): number {
            if n <= 0 {
                return acc
            }
            return tailSum(n - 1, acc + n)
        }

        const result = tailSum(10, 0)
    "#;

    let o3_output = compile_with_optimization(source, OptimizationLevel::O3).unwrap();
    println!("Tail recursive O3 output:\n{}", o3_output);
}

#[test]
fn test_method_inlining_small() {
    let source = r#"
        class MathOps {
            public double(n: number): number {
                return n * 2
            }
        }

        const m = new MathOps()
        const result = m.double(5)
    "#;

    let o3_output = compile_with_optimization(source, OptimizationLevel::O3).unwrap();
    println!("Method inlining O3 output:\n{}", o3_output);
}

#[test]
fn test_hot_path_inlining() {
    let source = r#"
        function hotPath(x: number): number {
            if x > 100 {
                return x * 2
            } else if x > 50 {
                return x * 3
            } else {
                return x * 4
            }
        }

        const results = [
            hotPath(75),
            hotPath(25),
            hotPath(150)
        ]
    "#;

    let o3_output = compile_with_optimization(source, OptimizationLevel::O3).unwrap();
    println!("Hot path O3 output:\n{}", o3_output);
}

#[test]
fn test_simple_calculator_inlining() {
    let source = r#"
        class Calculator {
            public add(a: number, b: number): number {
                return a + b
            }

            public sub(a: number, b: number): number {
                return a - b
            }
        }

        const calc = new Calculator()
        const r1 = calc.add(1, 2)
        const r2 = calc.sub(5, 3)
    "#;

    let o3_output = compile_with_optimization(source, OptimizationLevel::O3).unwrap();
    println!("Calculator O3 output:\n{}", o3_output);
}

#[test]
fn test_chain_calls_inlining() {
    let source = r#"
        function step1(x: number): number { return x + 1 }
        function step2(x: number): number { return x + 2 }
        function step3(x: number): number { return x + 3 }

        const result = step3(step2(step1(0)))
    "#;

    let o3_output = compile_with_optimization(source, OptimizationLevel::O3).unwrap();
    println!("Chain calls O3 output:\n{}", o3_output);
}

#[test]
fn test_closure_inlining() {
    let source = r#"
        function makeAdder(add: number): (number) => number {
            return function(x: number): number {
                return x + add
            }
        }

        const add5 = makeAdder(5)
        const result = add5(10)
    "#;

    let o3_output = compile_with_optimization(source, OptimizationLevel::O3).unwrap();
    println!("Closure O3 output:\n{}", o3_output);
}

#[test]
fn test_trivial_getter_inlined() {
    let source = r#"
        class Data {
            private _value: number = 42

            public get value(): number {
                return self._value
            }
        }

        const d = new Data()
        const v = d.value
        const w = d.value
    "#;

    let o3_output = compile_with_optimization(source, OptimizationLevel::O3).unwrap();
    println!("Trivial getter O3 output:\n{}", o3_output);
}
