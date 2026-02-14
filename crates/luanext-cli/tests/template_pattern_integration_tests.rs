use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

// Helper to create luanext command
fn luanext_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("luanext"))
}

/// Helper to compile a luanext source string and return the generated Lua
fn compile_source(source: &str) -> String {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.luax");

    fs::write(&input_file, source).unwrap();

    let output = luanext_cmd()
        .arg(input_file.to_str().unwrap())
        .arg("--emit")
        .arg("lua")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Compilation should succeed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn test_basic_url_pattern() {
    let lua = compile_source(
        r#"
        const url = "https://example.com/api/users"
        const result = match url {
          `https://${host}/${path}` => `Matched: ${host} / ${path}`,
          _ => "No match"
        }
        print(result)
    "#,
    );

    // Verify string.match is generated
    assert!(
        lua.contains("string.match"),
        "Should contain string.match call"
    );
    // Verify correct pattern with delimiters
    assert!(
        lua.contains("^https://([^/]+)/(.+)$"),
        "Should have delimiter-aware pattern"
    );
    // Verify capture variables are created
    assert!(lua.contains("__capture_1"), "Should have __capture_1");
    assert!(lua.contains("__capture_2"), "Should have __capture_2");
    // Verify nil check for match success
    assert!(
        lua.contains("__capture_1 ~= nil"),
        "Should check if match succeeded"
    );
    // Verify bindings from captures to user variables
    assert!(lua.contains("local host = __capture_1"), "Should bind host");
    assert!(lua.contains("local path = __capture_2"), "Should bind path");
}

#[test]
fn test_escape_special_characters() {
    let lua = compile_source(
        r#"
        const log = "[ERROR] Failed"
        match log {
          `[ERROR] ${msg}` => msg,
          _ => ""
        }
    "#,
    );

    // Square brackets should be escaped with %
    assert!(
        lua.contains("%[ERROR%]"),
        "Should escape square brackets in pattern"
    );
}

#[test]
fn test_percent_character_escaping() {
    let lua = compile_source(
        r#"
        const status = "100% complete"
        match status {
          `100% ${msg}` => msg,
          _ => ""
        }
    "#,
    );

    // % should be escaped as %%
    assert!(lua.contains("100%% "), "Should escape % character");
}

#[test]
fn test_multiple_delimiters() {
    let lua = compile_source(
        r#"
        const date = "2026-02-14"
        match date {
          `${year}-${month}-${day}` => year,
          _ => ""
        }
    "#,
    );

    // Should generate pattern with delimiter-aware captures
    // First two use [^-]+ (stop at -), last uses .+ (greedy)
    assert!(
        lua.contains("([^%-]+)%-([^%-]+)%-(.+)"),
        "Should have delimiter-aware captures for -"
    );
}

#[test]
fn test_single_capture() {
    let lua = compile_source(
        r#"
        const input = "Hello"
        match input {
          `${content}` => content,
          _ => ""
        }
    "#,
    );

    // Single capture should be greedy
    assert!(lua.contains("^(.+)$"), "Should have greedy single capture");
}

#[test]
fn test_with_guard_clause() {
    let lua = compile_source(
        r#"
        const input = "age:25"
        match input {
          `age:${val}` when tonumber(val) != nil => val,
          _ => ""
        }
    "#,
    );

    // Guard should be in the condition
    assert!(
        lua.contains("tonumber(val) ~= nil"),
        "Should include guard in condition"
    );
}

#[test]
fn test_prefix_pattern() {
    let lua = compile_source(
        r#"
        match "error: timeout" {
          `error: ${msg}` => msg,
          _ => ""
        }
    "#,
    );

    assert!(lua.contains("^error: (.+)$"), "Should match prefix pattern");
}

#[test]
fn test_compilation_success_complex() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.luax");

    fs::write(
        &input_file,
        r#"
        function parseUrl(url: string): string {
          return match url {
            `https://${host}/${path}` => `HTTPS: ${host}`,
            `http://${host}/${path}` => `HTTP: ${host}`,
            `ftp://${host}/${path}` => `FTP: ${host}`,
            _ => "Unknown protocol"
          }
        }

        const result = parseUrl("https://example.com/api")
        print(result)
    "#,
    )
    .unwrap();

    luanext_cmd()
        .arg(input_file.to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn test_mixed_with_other_patterns() {
    let lua = compile_source(
        r#"
        const value = "error: fail"
        match value {
          `error: ${msg}` => msg,
          "success" => "ok",
          _ => "unknown"
        }
    "#,
    );

    assert!(lua.contains("string.match"), "Should have template pattern");
    assert!(
        lua.contains("== \"success\""),
        "Should have literal string pattern"
    );
}

#[test]
fn test_nested_template_patterns() {
    let lua = compile_source(
        r#"
        const url = "https://api.example.com/v1/users"
        match url {
          `https://${rest}` => match rest {
            `${host}/v1/${endpoint}` => endpoint,
            _ => "unknown"
          },
          _ => "not https"
        }
    "#,
    );

    // Should have two separate string.match calls
    let match_count = lua.matches("string.match").count();
    assert!(match_count >= 2, "Should have multiple string.match calls");
}

#[test]
fn test_type_error_non_string_value() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.luax");

    fs::write(
        &input_file,
        r#"
        const x: number = 42
        match x {
          `${y}` => y
        }
    "#,
    )
    .unwrap();

    let output = luanext_cmd()
        .arg(input_file.to_str().unwrap())
        .output()
        .unwrap();

    // Should fail type checking
    assert!(!output.status.success(), "Should fail with type error");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("Template pattern can only match string values"),
        "Should have appropriate error message"
    );
}

#[test]
fn test_adjacent_captures_error() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.luax");

    fs::write(
        &input_file,
        r#"
        match "test" {
          `${a}${b}` => a
        }
    "#,
    )
    .unwrap();

    let output = luanext_cmd()
        .arg(input_file.to_str().unwrap())
        .output()
        .unwrap();

    // Should fail parsing
    assert!(!output.status.success(), "Should fail with parse error");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("Adjacent template pattern captures"),
        "Should have adjacent captures error"
    );
}

#[test]
fn test_expression_in_capture_error() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.luax");

    fs::write(
        &input_file,
        r#"
        match "test" {
          `${x + y}` => x
        }
    "#,
    )
    .unwrap();

    let output = luanext_cmd()
        .arg(input_file.to_str().unwrap())
        .output()
        .unwrap();

    // Should fail parsing
    assert!(!output.status.success(), "Should fail with parse error");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("Template pattern captures must be simple identifiers"),
        "Should require simple identifiers"
    );
}
