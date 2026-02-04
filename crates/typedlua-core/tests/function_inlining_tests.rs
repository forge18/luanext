use typedlua_core::config::OptimizationLevel;
use typedlua_core::di::DiContainer;

fn compile_with_optimization(source: &str, level: OptimizationLevel) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile_with_optimization(source, level)
}

#[test]
fn test_simple_function_inlining() {
    let source = r#"
        function add(a: number, b: number): number {
            return a + b
        }

        const x = add(1, 2)
        print(x)
    "#;

    let output = compile_with_optimization(source, OptimizationLevel::O2).unwrap();
    println!("Generated output (O2):\n{}", output);
}

#[test]
fn test_large_function_not_inlined() {
    let source = r#"
        function large(a: number, b: number): number {
            const t1 = a + 1
            const t2 = b + 2
            const t3 = t1 * 2
            const t4 = t2 * 3
            return t3 + t4
        }

        const x = large(1, 2)
    "#;

    let output = compile_with_optimization(source, OptimizationLevel::O2).unwrap();
    println!("Large function output:\n{}", output);
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

        const x = factorial(5)
    "#;

    let output = compile_with_optimization(source, OptimizationLevel::O2).unwrap();
    println!("Recursive function output:\n{}", output);
    assert!(
        output.contains("function"),
        "Recursion should prevent inlining"
    );
}

#[test]
fn test_single_use_function_inlined() {
    let source = r#"
        function id(x: number): number {
            return x
        }

        const x = id(42)
    "#;

    let output = compile_with_optimization(source, OptimizationLevel::O2).unwrap();
    println!("Single use output:\n{}", output);
    assert!(
        !output.contains("function id"),
        "Single use should be inlined"
    );
}

#[test]
fn test_method_inlining() {
    let source = r#"
        class Math {
            public double(x: number): number {
                return x * 2
            }
        }

        const m = new Math()
        const x = m.double(21)
    "#;

    let output = compile_with_optimization(source, OptimizationLevel::O2).unwrap();
    println!("Method inlining output:\n{}", output);
}

#[test]
fn test_tail_call_optimized() {
    let source = r#"
        function tailAdd(n: number, acc: number): number {
            if n <= 0 {
                return acc
            }
            return tailAdd(n - 1, acc + n)
        }

        const x = tailAdd(100, 0)
    "#;

    let output = compile_with_optimization(source, OptimizationLevel::O2).unwrap();
    println!("Tail call output:\n{}", output);
}

#[test]
fn test_constant_propagation_with_inlining() {
    let source = r#"
        function addConst(a: number): number {
            return a + 42
        }

        const x = addConst(8)
    "#;

    let output = compile_with_optimization(source, OptimizationLevel::O2).unwrap();
    println!("Constant propagation output:\n{}", output);
}

#[test]
fn test_inline_chain() {
    let source = r#"
        function f1(x: number): number { return x + 1 }
        function f2(x: number): number { return f1(x) }
        function f3(x: number): number { return f2(x) }

        const x = f3(0)
    "#;

    let output = compile_with_optimization(source, OptimizationLevel::O2).unwrap();
    println!("Inline chain output:\n{}", output);
}

#[test]
fn test_getter_inlining() {
    let source = r#"
        class Point {
            private _x: number = 0

            public get x(): number {
                return self._x
            }
        }

        const p = new Point()
        const v = p.x
    "#;

    let output = compile_with_optimization(source, OptimizationLevel::O2).unwrap();
    println!("Getter inlining output:\n{}", output);
}

#[test]
fn test_setter_inlining() {
    let source = r#"
        class Container {
            private _value: number = 0

            public set value(v: number) {
                self._value = v
            }

            public get(): number {
                return self._value
            }
        }

        const c = new Container()
        c.value = 42
        const v = c.get()
    "#;

    let output = compile_with_optimization(source, OptimizationLevel::O2).unwrap();
    println!("Setter inlining output:\n{}", output);
}

#[test]
fn test_closure_inlining() {
    let source = r#"
        function makeAdder(n: number): () => number {
            return () => n + 1
        }

        const add5 = makeAdder(5)
        const x = add5()
    "#;

    let output = compile_with_optimization(source, OptimizationLevel::O2).unwrap();
    println!("Closure inlining output:\n{}", output);
}

#[test]
fn test_loop_function_inlining() {
    let source = r#"
        function scale(x: number): number {
            return x * 2
        }

        const arr = [1, 2, 3, 4, 5]
        for i in arr {
            const s = scale(i)
        }
    "#;

    let output = compile_with_optimization(source, OptimizationLevel::O2).unwrap();
    println!("Loop function output:\n{}", output);
}

#[test]
fn test_nested_function_inlining() {
    let source = r#"
        function outer(x: number): number {
            function inner(y: number): number {
                return y * 2
            }
            return inner(x) + 1
        }

        const x = outer(5)
    "#;

    let output = compile_with_optimization(source, OptimizationLevel::O2).unwrap();
    println!("Nested function output:\n{}", output);
}

#[test]
fn test_conditional_inlining() {
    let source = r#"
        function choose(a: number, b: number): number {
            if a > b {
                return a
            }
            return b
        }

        const x = choose(3, 7)
    "#;

    let output = compile_with_optimization(source, OptimizationLevel::O2).unwrap();
    println!("Conditional inlining output:\n{}", output);
}

#[test]
fn test_inlining_with_side_effects() {
    let source = r#"
        let counter = 0
        function getAndIncrement(): number {
            counter = counter + 1
            return counter
        }

        const x = getAndIncrement()
        const y = getAndIncrement()
    "#;

    let output = compile_with_optimization(source, OptimizationLevel::O2).unwrap();
    println!("Side effects output:\n{}", output);
    assert!(
        output.contains("counter"),
        "Side effects should prevent aggressive inlining"
    );
}
