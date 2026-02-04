use typedlua_core::di::DiContainer;

fn compile_and_generate(source: &str) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile(source)
}

#[test]
fn test_single_line_template_no_dedenting() {
    let source = r#"
        const msg = `Hello World`
    "#;

    let result = compile_and_generate(source);
    match &result {
        Ok(output) => {
            println!("Generated code:\n{}", output);
            assert!(
                output.contains(r#""Hello World""#),
                "Single-line should not be dedented"
            );
        }
        Err(e) => {
            panic!("Should compile successfully: {}", e);
        }
    }
}

#[test]
fn test_basic_multi_line_dedenting() {
    let source = r#"
        const sql = `
            SELECT *
            FROM users
            WHERE id = 1
        `
    "#;

    let result = compile_and_generate(source);
    match &result {
        Ok(output) => {
            println!("Generated code:\n{}", output);
            assert!(
                output.contains("SELECT *")
                    && output.contains("FROM users")
                    && output.contains("WHERE id = 1"),
                "Should dedent and trim blank lines. Got: {}",
                output
            );
        }
        Err(e) => {
            panic!("Should compile successfully: {}", e);
        }
    }
}

#[test]
fn test_preserve_relative_indentation() {
    let source = r#"
        const html = `
            <div>
              <h1>Title</h1>
              <p>
                Content
              </p>
            </div>
        `
    "#;

    let result = compile_and_generate(source);
    match &result {
        Ok(output) => {
            println!("Generated code:\n{}", output);
            assert!(
                output.contains("<div>") && output.contains("  <h1>"),
                "Should preserve relative indentation"
            );
        }
        Err(e) => {
            panic!("Should compile successfully: {}", e);
        }
    }
}

#[test]
fn test_trim_leading_trailing_blank_lines() {
    let source = r#"
        const text = `

            Line 1
            Line 2

        `
    "#;

    let result = compile_and_generate(source);
    match &result {
        Ok(output) => {
            println!("Generated code:\n{}", output);
            assert!(
                output.contains("Line 1") && output.contains("Line 2"),
                "Should trim leading/trailing blank lines. Got: {}",
                output
            );
        }
        Err(e) => {
            panic!("Should compile successfully: {}", e);
        }
    }
}

#[test]
fn test_preserve_blank_lines_in_middle() {
    let source = r#"
        const text = `
            Line 1

            Line 2
        `
    "#;

    let result = compile_and_generate(source);
    match &result {
        Ok(output) => {
            println!("Generated code:\n{}", output);
            assert!(
                output.contains("Line 1") && output.contains("Line 2"),
                "Should preserve blank lines in middle. Got: {}",
                output
            );
        }
        Err(e) => {
            panic!("Should compile successfully: {}", e);
        }
    }
}

#[test]
fn test_sql_query_example() {
    let source = r#"
        function getUser(id: number): string {
            return `
                SELECT name, email
                FROM users
                WHERE id = ${id}
                ORDER BY name
            `
        }
    "#;

    let result = compile_and_generate(source);
    match &result {
        Ok(output) => {
            println!("Generated code:\n{}", output);
            assert!(
                output.contains("SELECT name, email"),
                "Should have dedented SQL"
            );
            assert!(output.contains("FROM users"), "Should have FROM");
        }
        Err(e) => {
            panic!("Should compile successfully: {}", e);
        }
    }
}

#[test]
fn test_html_template_example() {
    let source = r#"
        function render(title: string): string {
            return `
                <div class="container">
                  <h1>${title}</h1>
                  <p>Welcome!</p>
                </div>
            `
        }
    "#;

    let result = compile_and_generate(source);
    match &result {
        Ok(output) => {
            println!("Generated code:\n{}", output);
            assert!(
                output.contains(r#"<div class=\"container">"#),
                "Should have dedented HTML"
            );
            assert!(output.contains("  <h1>"), "Should preserve relative indent");
        }
        Err(e) => {
            panic!("Should compile successfully: {}", e);
        }
    }
}

#[test]
fn test_json_example() {
    let source = r#"
        function makeJSON(name: string): string {
            return `
                {
                  "name": "${name}",
                  "active": true
                }
            `
        }
    "#;

    let result = compile_and_generate(source);
    match &result {
        Ok(output) => {
            println!("Generated code:\n{}", output);
            assert!(
                output.contains("{") && output.contains("name"),
                "Should have dedented JSON. Got: {}",
                output
            );
        }
        Err(e) => {
            panic!("Should compile successfully: {}", e);
        }
    }
}

#[test]
fn test_all_whitespace_template() {
    let source = r#"
        const empty = `


        `
    "#;

    let result = compile_and_generate(source);
    match &result {
        Ok(output) => {
            println!("Generated code:\n{}", output);
            assert!(
                output.contains(r#""""#),
                "All-whitespace should become empty"
            );
        }
        Err(e) => {
            panic!("Should compile successfully: {}", e);
        }
    }
}

#[test]
fn test_first_line_has_content() {
    let source = r#"
        const msg = `Hello
            World`
    "#;

    let result = compile_and_generate(source);
    match &result {
        Ok(output) => {
            println!("Generated code:\n{}", output);
            assert!(output.contains("Hello"), "Should have Hello");
            assert!(output.contains("World"), "Should have World");
        }
        Err(e) => {
            panic!("Should compile successfully: {}", e);
        }
    }
}

#[test]
fn test_tabs_and_spaces_mixed_indentation() {
    let source = "
        const text = `
\t\t\tLine 1
            Line 2
        `
    ";

    let result = compile_and_generate(source);
    match &result {
        Ok(output) => {
            println!("Generated code:\n{}", output);
            assert!(
                output.contains("Line 1") && output.contains("Line 2"),
                "Should handle mixed tabs/spaces. Got: {}",
                output
            );
        }
        Err(e) => {
            panic!("Should compile successfully: {}", e);
        }
    }
}
