use typedlua_core::config::OptimizationLevel;
use typedlua_core::di::DiContainer;

fn compile_with_optimization_level(
    source: &str,
    level: OptimizationLevel,
) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile_with_optimization(source, level)
}

fn compile_with_o2(source: &str) -> Result<String, String> {
    compile_with_optimization_level(source, OptimizationLevel::O2)
}

#[test]
fn test_dead_store_simple_unused_variable() {
    let source = r#"
        const unused = 42
        const x = 1
    "#;

    let output = compile_with_o2(source).unwrap();
    assert!(
        !output.contains("unused"),
        "Dead store should be eliminated: {}",
        output
    );
}

#[test]
fn test_dead_store_reassigned_variable() {
    let source = r#"
        let x = 1
        x = 2
        return x
    "#;

    let output = compile_with_o2(source).unwrap();
    println!("Dead store reassigned:\n{}", output);
    assert!(
        !output.contains("= 1"),
        "Initial assignment should be eliminated"
    );
}

#[test]
fn test_dead_store_in_loop() {
    let source = r#"
        let sum = 0
        for i in [1, 2, 3] {
            sum = sum + i
        }
        return sum
    "#;

    let output = compile_with_o2(source).unwrap();
    println!("Dead store in loop:\n{}", output);
    assert!(
        output.contains("sum"),
        "Live variable in loop should be kept"
    );
}

#[test]
fn test_dead_store_across_blocks() {
    let source = r#"
        let x = 10
        if true {
            let y = x + 1
        }
        return x
    "#;

    let output = compile_with_o2(source).unwrap();
    println!("Dead store across blocks:\n{}", output);
    assert!(!output.contains("y"), "Variable y should be eliminated");
}

#[test]
fn test_dead_store_nested_conditionals() {
    let source = r#"
        let a = 1
        if true {
            let b = a + 1
            if true {
                let c = b + 1
            }
        }
        return a
    "#;

    let output = compile_with_o2(source).unwrap();
    println!("Dead store nested:\n{}", output);
    assert!(
        !output.contains("b") && !output.contains("c"),
        "Nested dead stores should be eliminated"
    );
}

#[test]
fn test_dead_store_with_function_call() {
    let source = r#"
        let x = print("dead")
        return 42
    "#;

    let output = compile_with_o2(source).unwrap();
    println!("Dead store with function:\n{}", output);
    assert!(
        !output.contains("dead"),
        "Dead store with side effect should be kept but value unused"
    );
}

#[test]
fn test_dead_store_const_reassigned() {
    let source = r#"
        const x = 1
        const y = x + 1
        return y
    "#;

    let output = compile_with_o2(source).unwrap();
    println!("Const reassigned:\n{}", output);
    assert!(
        !output.contains("= 1"),
        "Const assignment should be eliminated if only used once"
    );
}

#[test]
fn test_dead_store_class_field() {
    let source = r#"
        class Point {
            x: number
            y: number
            unused: number
        }
        const p = new Point()
    "#;

    let output = compile_with_o2(source).unwrap();
    println!("Dead store class field:\n{}", output);
    assert!(
        output.contains("x") && output.contains("y"),
        "Used fields should be kept"
    );
}

#[test]
fn test_dead_store_for_loop() {
    let source = r#"
        for i in [1, 2, 3] {
            let temp = i * 2
        }
        return 0
    "#;

    let output = compile_with_o2(source).unwrap();
    println!("Dead store for loop:\n{}", output);
    assert!(
        !output.contains("temp"),
        "Dead store in for loop should be eliminated"
    );
}

#[test]
fn test_dead_store_while_loop() {
    let source = r#"
        let i = 0
        while i < 10 {
            let temp = i * 2
            i = i + 1
        }
        return i
    "#;

    let output = compile_with_o2(source).unwrap();
    println!("Dead store while loop:\n{}", output);
    assert!(
        !output.contains("temp"),
        "Dead store in while loop should be eliminated"
    );
}

#[test]
fn test_dead_store_return_value() {
    let source = r#"
        function f(): number {
            let x = 1
            return x
        }
        return f()
    "#;

    let output = compile_with_o2(source).unwrap();
    println!("Dead store return value:\n{}", output);
    assert!(
        output.contains("x"),
        "Dead store used as return value should be kept"
    );
}

#[test]
fn test_dead_store_parameter() {
    let source = r#"
        function f(a: number, b: number): number {
            return a
        }
        return f(1, 2)
    "#;

    let output = compile_with_o2(source).unwrap();
    println!("Dead store parameter:\n{}", output);
    assert!(output.contains("a"), "Used parameter should be kept");
}

#[test]
fn test_dead_store_unused_parameter() {
    let source = r#"
        function f(a: number, b: number): number {
            return a
        }
    "#;

    let output = compile_with_o2(source).unwrap();
    println!("Dead store unused parameter:\n{}", output);
    assert!(
        !output.contains("b"),
        "Unused parameter should be eliminated"
    );
}

#[test]
fn test_dead_store_self_modify() {
    let source = r#"
        let x = 1
        x = x + 1
        x = x * 2
        return x
    "#;

    let output = compile_with_o2(source).unwrap();
    println!("Dead store self modify:\n{}", output);
    assert!(
        output.contains("x"),
        "Self-modifying variable should be kept"
    );
}

#[test]
fn test_dead_store_complex_expression() {
    let source = r#"
        const a = 1
        const b = a + 2
        const c = b * 3
        return c
    "#;

    let output = compile_with_o2(source).unwrap();
    println!("Dead store complex expression:\n{}", output);
    assert!(
        !output.contains("a") && !output.contains("b"),
        "Intermediate values should be eliminated"
    );
}

#[test]
fn test_dead_store_closure() {
    let source = r#"
        let x = 1
        const f = () => x + 1
        return f()
    "#;

    let output = compile_with_o2(source).unwrap();
    println!("Dead store closure:\n{}", output);
    assert!(
        output.contains("x"),
        "Variable captured by closure should be kept"
    );
}

#[test]
fn test_dead_store_method_call() {
    let source = r#"
        const arr = [1, 2, 3]
        const len = arr.length
        return len
    "#;

    let output = compile_with_o2(source).unwrap();
    println!("Dead store method call:\n{}", output);
    assert!(
        output.contains("length"),
        "Method call result should be kept if used"
    );
}

#[test]
fn test_dead_store_subscript() {
    let source = r#"
        const arr = [1, 2, 3]
        const first = arr[0]
        return first
    "#;

    let output = compile_with_o2(source).unwrap();
    println!("Dead store subscript:\n{}", output);
    assert!(
        output.contains("first"),
        "Subscript result should be kept if used"
    );
}

#[test]
fn test_dead_store_table_literal() {
    let source = r#"
        const t = { a: 1, b: 2 }
        const key = "a"
        return t[key]
    "#;

    let output = compile_with_o2(source).unwrap();
    println!("Dead store table literal:\n{}", output);
    assert!(
        output.contains("a"),
        "Table literal should be kept if accessed"
    );
}
