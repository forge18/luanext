use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// Helper to create typedlua command using the non-deprecated macro approach
fn luanext_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("typedlua"))
}

/// Test basic compilation of a simple file
#[test]
fn test_compile_simple_file() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.luax");

    fs::write(
        &input_file,
        r#"
        const x: number = 42
        const y: string = "hello"
    "#,
    )
    .unwrap();

    luanext_cmd()
        .arg(input_file.to_str().unwrap())
        .assert()
        .success();
}

/// Test compilation with type errors
#[test]
fn test_compile_with_type_error() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("error.luax");

    fs::write(
        &input_file,
        r#"
        const x: number = "not a number"
    "#,
    )
    .unwrap();

    let output = luanext_cmd()
        .arg(input_file.to_str().unwrap())
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "Compilation should fail for type error"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Type mismatch"),
        "Error should mention 'Type mismatch', got: {}",
        stderr
    );
    assert!(
        stderr.contains("error"),
        "Error should contain 'error', got: {}",
        stderr
    );
}

/// Test output directory option
#[test]
fn test_output_directory() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.luax");
    let output_dir = temp_dir.path().join("out");

    fs::write(
        &input_file,
        r#"
        const message: string = "hello world"
        print(message)
    "#,
    )
    .unwrap();

    luanext_cmd()
        .arg(input_file.to_str().unwrap())
        .arg("--out-dir")
        .arg(output_dir.to_str().unwrap())
        .assert()
        .success();

    assert!(output_dir.exists());
    assert!(output_dir.join("test.lua").exists());
}

/// Test multiple input files
#[test]
fn test_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = temp_dir.path().join("file1.luax");
    let file2 = temp_dir.path().join("file2.luax");

    fs::write(&file1, "const a: number = 1").unwrap();
    fs::write(&file2, "const b: string = \"test\"").unwrap();

    luanext_cmd()
        .arg(file1.to_str().unwrap())
        .arg(file2.to_str().unwrap())
        .assert()
        .success();
}

/// Test --no-emit flag
#[test]
fn test_no_emit_flag() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.luax");
    let output_file = temp_dir.path().join("test.lua");

    fs::write(
        &input_file,
        r#"
        const x: number = 42
    "#,
    )
    .unwrap();

    luanext_cmd()
        .arg(input_file.to_str().unwrap())
        .arg("--no-emit")
        .assert()
        .success();

    // Output file should not exist
    assert!(!output_file.exists());
}

/// Test Lua 5.1 target
#[test]
fn test_lua51_target() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.luax");

    fs::write(
        &input_file,
        r#"
        const x: number = 42
    "#,
    )
    .unwrap();

    luanext_cmd()
        .arg(input_file.to_str().unwrap())
        .arg("--target")
        .arg("5.1")
        .arg("--no-emit")
        .assert()
        .success();
}

/// Test function compilation
#[test]
fn test_function_compilation() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("func.luax");
    let output_file = temp_dir.path().join("func.lua");

    fs::write(
        &input_file,
        r#"
        function add(a: number, b: number): number
            return a + b
        end

        const result: number = add(5, 3)
    "#,
    )
    .unwrap();

    luanext_cmd()
        .arg(input_file.to_str().unwrap())
        .assert()
        .success();

    let output = fs::read_to_string(&output_file).unwrap();
    assert!(
        output.contains("function add"),
        "Generated Lua should contain 'function add'"
    );
    assert!(
        output.contains("return"),
        "Generated Lua should contain 'return' statement"
    );
    assert!(
        output.contains("a + b"),
        "Generated Lua should preserve the addition expression"
    );
    assert!(
        output.contains("result"),
        "Generated Lua should contain the const variable name"
    );
    assert!(
        !output.contains(": number"),
        "Type annotations should be stripped from generated Lua"
    );
    assert!(
        !output.contains(": string"),
        "Type annotations should be stripped from generated Lua"
    );
}

/// Test class compilation
#[test]
fn test_class_compilation() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("class.luax");
    let output_file = temp_dir.path().join("class.lua");

    fs::write(
        &input_file,
        r#"
        class Point {
            public x: number = 0
            public y: number = 0
        }
    "#,
    )
    .unwrap();

    luanext_cmd()
        .arg(input_file.to_str().unwrap())
        .assert()
        .success();

    assert!(
        output_file.exists(),
        "Output file should exist after successful compilation"
    );

    let output = fs::read_to_string(&output_file).unwrap();
    assert!(
        output.contains("Point"),
        "Generated Lua should contain class name 'Point'"
    );
    assert!(
        output.contains("x"),
        "Generated Lua should contain field 'x'"
    );
    assert!(
        output.contains("y"),
        "Generated Lua should contain field 'y'"
    );
    assert!(
        !output.contains("public"),
        "Access modifiers should be stripped from generated Lua"
    );
    assert!(
        !output.contains(": number"),
        "Type annotations should be stripped from generated Lua"
    );
}

/// Test interface type checking
#[test]
fn test_interface_type_checking() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("interface.luax");

    fs::write(
        &input_file,
        r#"
        interface User {
            name: string
            age: number
        }
    "#,
    )
    .unwrap();

    luanext_cmd()
        .arg(input_file.to_str().unwrap())
        .arg("--no-emit")
        .assert()
        .success();
}

/// Test invalid interface usage
#[test]
fn test_invalid_interface() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("bad_interface.luax");

    fs::write(
        &input_file,
        r#"
        interface User {
            name: string
            age: number
        }

        const user: User = {
            name = "Alice"
            -- missing age field
        }
    "#,
    )
    .unwrap();

    let output = luanext_cmd()
        .arg(input_file.to_str().unwrap())
        .arg("--no-emit")
        .arg("--no-cache")
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "Compilation should fail for missing interface field"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Type mismatch"),
        "Error should mention 'Type mismatch' for interface mismatch, got: {}",
        stderr
    );
    assert!(
        stderr.contains("User"),
        "Error should reference the interface name 'User', got: {}",
        stderr
    );
}

/// Test --version flag
#[test]
fn test_version_flag() {
    luanext_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("typedlua"));
}

/// Test --help flag
#[test]
fn test_help_flag() {
    luanext_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("TypedLua"));
}

/// Test nonexistent file error
#[test]
fn test_nonexistent_file() {
    luanext_cmd().arg("nonexistent.luax").assert().failure();
}

/// Test source map generation
#[test]
fn test_source_map_generation() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.luax");
    let source_map_file = temp_dir.path().join("test.lua.map");

    fs::write(
        &input_file,
        r#"
        const x: number = 42
        const y: string = "hello"
    "#,
    )
    .unwrap();

    luanext_cmd()
        .arg(input_file.to_str().unwrap())
        .arg("--source-map")
        .assert()
        .success();

    assert!(source_map_file.exists());
}

/// Test pretty printing
#[test]
fn test_pretty_output() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.luax");

    fs::write(
        &input_file,
        r#"
        const x: number = "wrong type"
    "#,
    )
    .unwrap();

    luanext_cmd()
        .arg(input_file.to_str().unwrap())
        .arg("--pretty")
        .assert()
        .failure();
}
