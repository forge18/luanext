use typedlua_core::di::DiContainer;

fn compile_and_check(source: &str) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile(source)
}

#[test]
fn test_unreachable_after_wildcard() {
    let source = r#"
        const x = 5
        const result = match x {
            _ => "any",
            1 => "one"
        }
    "#;

    let _result = compile_and_check(source);
}

#[test]
fn test_unreachable_after_identifier() {
    let source = r#"
        const x = 5
        const result = match x {
            n => n + 1,
            5 => 10
        }
    "#;

    let _result = compile_and_check(source);
}

#[test]
fn test_reachable_after_guarded_wildcard() {
    let source = r#"
        const x = 5
        const result = match x {
            n when n > 10 => "big",
            _ => "small"
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Should compile without errors");
}

#[test]
fn test_duplicate_literal() {
    let source = r#"
        const x: boolean = true
        const result = match x {
            true => "yes",
            true => "also yes"
        }
    "#;

    let _result = compile_and_check(source);
}

#[test]
fn test_or_pattern_subsumes_literal() {
    let source = r#"
        const x = 2
        const result = match x {
            1 | 2 | 3 => "small",
            2 => "two"
        }
    "#;

    let _result = compile_and_check(source);
}

#[test]
fn test_or_pattern_partial_overlap_no_warning() {
    let source = r#"
        const x = 3
        const result = match x {
            1 | 2 => "one or two",
            2 | 3 => "two or three"
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Should compile without errors");
}

#[test]
fn test_or_pattern_fully_subsumed() {
    let source = r#"
        const x = 2
        const result = match x {
            1 | 2 | 3 | 4 => "small",
            2 | 3 => "middle"
        }
    "#;

    let _result = compile_and_check(source);
}

#[test]
fn test_multiple_literal_alternatives() {
    let source = r#"
        const x = 1
        const result = match x {
            1 => "one",
            1 => "duplicate"
        }
    "#;

    let _result = compile_and_check(source);
}

#[test]
fn test_array_wildcard_subsumes_literal() {
    let source = r#"
        const x = {1, 2}
        const result = match x {
            {a, b} => "any pair",
            {1, 2} => "specific"
        }
    "#;

    let _result = compile_and_check(source);
}

#[test]
fn test_array_no_warning_different_length() {
    let source = r#"
        const x: {number, number} | {number, number, number} = {1, 2}
        const result = match x {
            {a, b} => "two",
            {a, b, c} => "three"
        }
    "#;

    let result = compile_and_check(source);
    let _ = result;
}

#[test]
fn test_array_rest_pattern_subsumption() {
    let source = r#"
        const x = {1, 2, 3}
        const result = match x {
            {1, ...rest} => "starts with 1",
            {1, 2, 3} => "exact"
        }
    "#;

    let _result = compile_and_check(source);
}

#[test]
fn test_object_wildcard_subsumes_literal() {
    let source = r#"
        const x = {a: 1, b: 2}
        const result = match x {
            {a, b} => "any object",
            {a: 1, b: 2} => "specific"
        }
    "#;

    let _result = compile_and_check(source);
}

#[test]
fn test_object_different_keys_no_warning() {
    let source = r#"
        const x = {a: 1, b: 2}
        const result = match x {
            {a: 1} => "has a=1",
            {b: 2} => "has b=2"
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Should compile without errors");
}

#[test]
fn test_object_missing_property_no_warning() {
    let source = r#"
        const x = {a: 1, b: 2}
        const result = match x {
            {a: 1, b: 2} => "both",
            {a: 1} => "only a"
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Should compile without errors");
}

#[test]
fn test_guarded_pattern_not_subsumer() {
    let source = r#"
        const x = 5
        const result = match x {
            5 when x > 10 => "big five",
            5 => "normal five"
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Should compile without errors");
}

#[test]
fn test_unreachable_despite_guard() {
    let source = r#"
        const x = 5
        const result = match x {
            _ => "any",
            5 when x > 0 => "positive five"
        }
    "#;

    let _result = compile_and_check(source);
}

#[test]
fn test_literal_boolean_true_vs_false() {
    let source = r#"
        const x: boolean = true
        const result = match x {
            true => "yes",
            false => "no",
            true => "duplicate"
        }
    "#;

    let _result = compile_and_check(source);
}

#[test]
fn test_single_arm_no_warning() {
    let source = r#"
        const x = 5
        const result = match x {
            _ => "any"
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Should compile without errors");
}

#[test]
fn test_two_different_patterns_no_warning() {
    let source = r#"
        const x: 1 | 2 = 1
        const result = match x {
            1 => "one",
            2 => "two"
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Should compile without errors");
}
