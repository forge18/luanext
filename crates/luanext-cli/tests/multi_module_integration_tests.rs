//! End-to-End Multi-Module Integration Tests
//!
//! Tests the full compilation pipeline for multi-file projects with:
//! - Cross-file imports
//! - Circular type dependencies (should succeed)
//! - Circular value dependencies (should fail)
//! - Re-export chains
//! - Type-only imports and exports
//! - Mixed scenarios

use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Get the luanext CLI binary for testing
fn luanext_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("luanext"))
}

/// Create a test file in the temp directory
/// If name doesn't have an extension, defaults to .luax
fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
    let name_with_ext = if name.contains('.') {
        name.to_string()
    } else {
        format!("{}.luax", name)
    };
    let file_path = dir.join(&name_with_ext);
    fs::write(&file_path, content).unwrap();
    file_path
}

/// Assert that compilation succeeded and output files exist
fn assert_compilation_success(temp_dir: &Path, expected_outputs: &[&str]) {
    for output in expected_outputs {
        let output_path = temp_dir.join(output);
        assert!(
            output_path.exists(),
            "Expected output file not found: {}",
            output
        );
    }
}

/// Assert that compilation failed with expected error message
fn assert_compilation_error(output: std::process::Output, expected_error: &str) {
    assert!(
        !output.status.success(),
        "Expected compilation to fail, but it succeeded"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected_error),
        "Expected error message not found.\nExpected: {}\nActual stderr:\n{}",
        expected_error,
        stderr
    );
}

/// Get all .luax files in a directory (recursive), sorted deterministically
fn get_all_luax_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();
    collect_luax_files_recursive(dir, &mut files);
    files.sort();
    files
}

fn collect_luax_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_luax_files_recursive(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "luax") {
            files.push(path);
        }
    }
}

/// Compile all .luax files in a directory (with cache disabled to isolate multi-module logic)
fn compile_all_in_dir(dir: &Path) {
    let files = get_all_luax_files(dir);
    let mut cmd = luanext_cmd();
    // Disable cache to avoid StringInterner key mismatches in tests
    cmd.arg("--no-cache");
    for file in &files {
        cmd.arg(file.to_str().unwrap());
    }
    // Use timeout to prevent indefinite hangs during testing
    cmd.timeout(std::time::Duration::from_secs(60));
    let output = cmd.output().expect("Failed to execute luanext");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "Compilation failed for {} files in {:?}\nFiles: {:?}\nstderr:\n{}\nstdout:\n{}",
            files.len(),
            dir,
            files,
            stderr,
            stdout
        );
    }
}

// ============================================================================
// BASIC CROSS-FILE IMPORT TESTS (5 tests)
// ============================================================================

#[test]
fn test_simple_cross_file_import() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(
        temp_dir.path(),
        "utils",
        r#"
        export function add(a: number, b: number): number {
            return a + b
        }
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import { add } from './utils'
        const result = add(5, 3)
        print(result)
    "#,
    );

    // Just verify compilation succeeds without checking output files
    let mut cmd = luanext_cmd();
    let utils_file = temp_dir.path().join("utils.luax");
    let main_file = temp_dir.path().join("main.luax");

    cmd.arg(utils_file.to_str().unwrap())
        .arg(main_file.to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn test_type_only_import() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(
        temp_dir.path(),
        "types",
        r#"
        export interface User {
            name: string
            age: number
        }
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import type { User } from './types'
        function greet(user: User): string
            return "Hello " .. user.name
        end
    "#,
    );

    compile_all_in_dir(temp_dir.path());

    // Verify type-only import doesn't generate require() for types
    let main_lua = fs::read_to_string(temp_dir.path().join("main.lua")).unwrap();
    assert!(!main_lua.contains("require"));
}

#[test]
fn test_diamond_dependency() {
    let temp_dir = TempDir::new().unwrap();

    // Common module that both A and B depend on
    create_test_file(
        temp_dir.path(),
        "common",
        r#"
        export const VERSION = "1.0.0"
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "module_a",
        r#"
        import { VERSION } from './common'
        export const a_value = VERSION .. "-a"
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "module_b",
        r#"
        import { VERSION } from './common'
        export const b_value = VERSION .. "-b"
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import { a_value } from './module_a'
        import { b_value } from './module_b'
        print(a_value)
        print(b_value)
    "#,
    );

    compile_all_in_dir(temp_dir.path());
}

#[test]
fn test_deep_dependency_chain() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(
        temp_dir.path(),
        "d",
        r#"
        export function d_func(): string {
            return "d"
        }
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "c",
        r#"
        import { d_func } from './d'
        export function c_func(): string {
            return d_func() .. "-c"
        }
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "b",
        r#"
        import { c_func } from './c'
        export function b_func(): string {
            return c_func() .. "-b"
        }
    "#,
    );
    create_test_file(
        temp_dir.path(),
        "a",
        r#"
        import { b_func } from './b'
        print(b_func())
    "#,
    );

    compile_all_in_dir(temp_dir.path());

    assert_compilation_success(temp_dir.path(), &["d.lua", "c.lua", "b.lua", "a.lua"]);
}

#[test]
fn test_multiple_imports_from_same_module() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(
        temp_dir.path(),
        "math",
        r#"
        export function add(a: number, b: number): number {
            return a + b
        }
        export function subtract(a: number, b: number): number {
            return a - b
        }
        export function multiply(a: number, b: number): number {
            return a * b
        }
    "#,
    );
    create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import { add, subtract, multiply } from './math'
        print(add(10, 5))
        print(subtract(10, 5))
        print(multiply(10, 5))
    "#,
    );

    compile_all_in_dir(temp_dir.path());

    assert_compilation_success(temp_dir.path(), &["math.lua", "main.lua"]);
}

// ============================================================================
// CIRCULAR TYPE REFERENCE TESTS (8 tests - should PASS)
// ============================================================================

#[test]
fn test_circular_type_dependency_interfaces() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(
        temp_dir.path(),
        "user",
        r#"
        import type { Post } from './post'

        export interface User {
            id: number
            posts: Post[]
        }
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "post",
        r#"
        import type { User } from './user'

        export interface Post {
            id: number
            author: User
        }
    "#,
    );
    create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import type { User } from './user'
        import type { Post } from './post'

        function processUser(user: User): number
            return user.id
        end
        function processPost(post: Post): number
            return post.id
        end
    "#,
    );

    compile_all_in_dir(temp_dir.path());
}

#[test]
fn test_circular_type_with_reexport() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(
        temp_dir.path(),
        "base",
        r#"
        import type { Extended } from './extended'
        export interface Base {
            extended: Extended
        }
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "extended",
        r#"
        export type { Base } from './base'
        export interface Extended {
            value: string
        }
    "#,
    );
    create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import type { Base } from './base'
        import type { Extended } from './extended'
        function processBase(b: Base): number
            return 1
        end
    "#,
    );

    compile_all_in_dir(temp_dir.path());
}

#[test]
fn test_three_way_type_cycle() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(
        temp_dir.path(),
        "a",
        r#"
        import type { B } from './b'
        export interface A {
            b: B
        }
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "b",
        r#"
        import type { C } from './c'
        export interface B {
            c: C
        }
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "c",
        r#"
        import type { A } from './a'
        export interface C {
            a: A
        }
    "#,
    );
    create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import type { A } from './a'
        function processA(a: A): number
            return 1
        end
    "#,
    );

    compile_all_in_dir(temp_dir.path());
}

#[test]
fn test_mixed_type_only_and_value_imports() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(
        temp_dir.path(),
        "config",
        r#"
        export interface ConfigType {
            debug: boolean
        }
        export const defaultConfig: ConfigType = { debug = false }
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "app",
        r#"
        import type { ConfigType } from './config'
        import { defaultConfig } from './config'
        export function getConfig(): ConfigType {
            return defaultConfig
        }
    "#,
    );
    create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import { getConfig } from './app'
        const cfg = getConfig()
        print(cfg.debug)
    "#,
    );

    compile_all_in_dir(temp_dir.path());
}

#[test]
fn test_forward_class_declarations() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(
        temp_dir.path(),
        "node",
        r#"
        import type { Edge } from './edge'
        export class Node {
            edges: Edge[] = {}
        }
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "edge",
        r#"
        import type { Node } from './node'
        export class Edge {
            source: Node
            target: Node
        }
    "#,
    );
    create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import type { Node } from './node'
        import type { Edge } from './edge'
        function processNode(n: Node): number
            return 1
        end
    "#,
    );

    compile_all_in_dir(temp_dir.path());
}

#[test]
fn test_type_alias_circular_reference() {
    let temp_dir = TempDir::new().unwrap();

    // Note: Using interfaces instead of type aliases because
    // `export type X = Y` has ambiguous parsing (the `type` keyword
    // is consumed as the type-only modifier). This test verifies
    // circular type-only dependencies with interfaces work correctly.
    create_test_file(
        temp_dir.path(),
        "types_a",
        r#"
        import type { TypeB } from './types_b'
        export interface TypeA {
            b: TypeB
        }
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "types_b",
        r#"
        import type { TypeA } from './types_a'
        export interface TypeB {
            a: TypeA
        }
    "#,
    );
    create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import type { TypeA } from './types_a'
        function processA(x: TypeA): number
            return 1
        end
    "#,
    );

    compile_all_in_dir(temp_dir.path());
}

// ============================================================================
// CIRCULAR VALUE REFERENCE TESTS (4 tests - should FAIL)
// ============================================================================

#[test]
fn test_circular_value_dependency_error() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(
        temp_dir.path(),
        "a",
        r#"
        import { foo } from './b'
        export const bar = foo + 1
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "b",
        r#"
        import { bar } from './a'
        export const foo = bar + 1
    "#,
    );

    let mut cmd = luanext_cmd();
    cmd.arg("--no-cache");
    cmd.arg(temp_dir.path().join("a.luax").to_str().unwrap());
    cmd.arg(temp_dir.path().join("b.luax").to_str().unwrap());
    let output = cmd.output().unwrap();

    assert_compilation_error(output, "Circular runtime dependency");
}

#[test]
fn test_three_way_value_cycle() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(
        temp_dir.path(),
        "x",
        r#"
        import { y_val } from './y'
        export const x_val = y_val + 1
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "y",
        r#"
        import { z_val } from './z'
        export const y_val = z_val + 1
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "z",
        r#"
        import { x_val } from './x'
        export const z_val = x_val + 1
    "#,
    );

    let mut cmd = luanext_cmd();
    cmd.arg("--no-cache");
    cmd.arg(temp_dir.path().join("x.luax").to_str().unwrap());
    cmd.arg(temp_dir.path().join("y.luax").to_str().unwrap());
    cmd.arg(temp_dir.path().join("z.luax").to_str().unwrap());
    let output = cmd.output().unwrap();

    assert_compilation_error(output, "Circular runtime dependency");
}

#[test]
fn test_self_import_error() {
    let temp_dir = TempDir::new().unwrap();

    let self_import_file = create_test_file(
        temp_dir.path(),
        "self",
        r#"
        import { foo } from './self'
        export const foo = 42
    "#,
    );

    let mut cmd = luanext_cmd();
    cmd.arg("--no-cache");
    cmd.arg(self_import_file.to_str().unwrap());
    let output = cmd.output().unwrap();

    // Self-import creates a circular runtime dependency
    assert_compilation_error(output, "Circular runtime dependency");
}

#[test]
fn test_mixed_cycle_type_and_value_error() {
    let temp_dir = TempDir::new().unwrap();

    // A imports type from B (OK), but B imports value from A (ERROR)
    create_test_file(
        temp_dir.path(),
        "mod_a",
        r#"
        import type { TypeB } from './mod_b'
        export interface TypeA {
            b: TypeB
        }
        export const value_a = 1
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "mod_b",
        r#"
        import { value_a } from './mod_a'
        export interface TypeB {
            x: number
        }
        export const value_b = value_a + 1
    "#,
    );

    let mut cmd = luanext_cmd();
    cmd.arg("--no-cache");
    cmd.arg(temp_dir.path().join("mod_a.luax").to_str().unwrap());
    cmd.arg(temp_dir.path().join("mod_b.luax").to_str().unwrap());
    let output = cmd.output().unwrap();

    // Mixed cycle has a value edge (B→A), so it should be detected as circular
    assert_compilation_error(output, "Circular runtime dependency");
}

// ============================================================================
// RE-EXPORT CHAIN TESTS (7 tests)
// ============================================================================

#[test]
fn test_single_level_reexport() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(
        temp_dir.path(),
        "original",
        r#"
        export interface User {
            name: string
        }
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "reexporter",
        r#"
        export { User } from './original'
    "#,
    );
    create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import type { User } from './reexporter'
        function processUser(user: User): string
            return user.name
        end
    "#,
    );

    compile_all_in_dir(temp_dir.path());
}

#[test]
fn test_multilevel_reexport_chain() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(
        temp_dir.path(),
        "original",
        r#"
        export interface Data {
            value: string
        }
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "level1",
        r#"
        export { Data } from './original'
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "level2",
        r#"
        export { Data } from './level1'
    "#,
    );
    create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import type { Data } from './level2'
        function processData(d: Data): string
            return d.value
        end
    "#,
    );

    compile_all_in_dir(temp_dir.path());
}

#[test]
fn test_reexport_with_alias() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(
        temp_dir.path(),
        "source",
        r#"
        export function original_name(): string {
            return "test"
        }
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "public",
        r#"
        export { original_name as exported_name } from './source'
    "#,
    );
    create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import { exported_name } from './public'
        print(exported_name())
    "#,
    );

    compile_all_in_dir(temp_dir.path());
}

#[test]
fn test_export_star_from_module() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(
        temp_dir.path(),
        "utils",
        r#"
        export function add(a: number, b: number): number {
            return a + b
        }
        export function subtract(a: number, b: number): number {
            return a - b
        }
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "index",
        r#"
        export * from './utils'
    "#,
    );
    create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import { add, subtract } from './index'
        print(add(10, 5))
        print(subtract(10, 5))
    "#,
    );

    compile_all_in_dir(temp_dir.path());
}

#[test]
fn test_type_only_reexport_chain() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(
        temp_dir.path(),
        "types",
        r#"
        export interface Config {
            debug: boolean
        }
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "types_api",
        r#"
        export type { Config } from './types'
    "#,
    );
    create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import type { Config } from './types_api'
        function processConfig(cfg: Config): boolean
            return cfg.debug
        end
    "#,
    );

    compile_all_in_dir(temp_dir.path());

    // Verify type-only re-export generates no runtime code
    let index_lua = fs::read_to_string(temp_dir.path().join("types_api.lua")).unwrap();
    assert!(
        index_lua.trim().is_empty() || !index_lua.contains("require"),
        "Type-only re-export should not generate runtime code"
    );
}

// ============================================================================
// TYPE-ONLY IMPORT/EXPORT VALIDATION TESTS (4 tests)
// ============================================================================

#[test]
fn test_export_type_generates_no_runtime_code() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(
        temp_dir.path(),
        "types",
        r#"
        export interface User {
            name: string
        }
        export interface ID {
            value: number
        }
    "#,
    );
    create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import type { User, ID } from './types'
        function processUser(user: User): string
            return user.name
        end
    "#,
    );

    compile_all_in_dir(temp_dir.path());

    // Verify types.lua doesn't contain function implementations or require() calls
    // (type-only modules may still have minimal boilerplate like module return)
    let types_lua = fs::read_to_string(temp_dir.path().join("types.lua")).unwrap();
    assert!(
        !types_lua.contains("function") && !types_lua.contains("require"),
        "Type-only exports should not generate function implementations or require() calls, got:\n{}",
        types_lua
    );
}

#[test]
fn test_import_type_no_require_call() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(
        temp_dir.path(),
        "config",
        r#"
        export interface AppConfig {
            version: string
        }
    "#,
    );
    create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import type { AppConfig } from './config'
        function processConfig(cfg: AppConfig): string
            return cfg.version
        end
    "#,
    );

    compile_all_in_dir(temp_dir.path());

    // Verify no require() for type-only import
    let main_lua = fs::read_to_string(temp_dir.path().join("main.lua")).unwrap();
    assert!(
        !main_lua.contains("require('config')") && !main_lua.contains("require(\"config\")"),
        "import type should not generate require() call"
    );
}

#[test]
fn test_reexported_type_preserves_type_only_nature() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(
        temp_dir.path(),
        "base_types",
        r#"
        export interface BaseType {
            id: number
        }
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "api_types",
        r#"
        export type { BaseType } from './base_types'
    "#,
    );
    create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import type { BaseType } from './api_types'
        function processBase(bt: BaseType): number
            return bt.id
        end
    "#,
    );

    compile_all_in_dir(temp_dir.path());

    // api_types.lua should be empty or minimal
    let api_types_lua = fs::read_to_string(temp_dir.path().join("api_types.lua")).unwrap();
    assert!(
        api_types_lua.trim().is_empty(),
        "Re-exported types should not generate runtime code"
    );

    // main.lua should not require api_types
    let main_lua = fs::read_to_string(temp_dir.path().join("main.lua")).unwrap();
    assert!(
        !main_lua.contains("require('api_types')") && !main_lua.contains("require(\"api_types\")"),
        "Type-only re-import should not generate require() call"
    );
}

// ============================================================================
// MIXED SCENARIO TESTS (3 tests - complex real-world patterns)
// ============================================================================

#[test]
fn test_api_layer_pattern() {
    let temp_dir = TempDir::new().unwrap();

    // Pure types module
    create_test_file(
        temp_dir.path(),
        "types",
        r#"
        export interface User {
            id: number
            name: string
        }
    "#,
    );

    // Models module that imports types and exports functions
    create_test_file(
        temp_dir.path(),
        "models",
        r#"
        import type { User } from './types'
        export function getUserName(user: User): string
            return user.name
        end
    "#,
    );

    // API module that uses both types and models
    create_test_file(
        temp_dir.path(),
        "api",
        r#"
        import type { User } from './types'
        import { getUserName } from './models'
        export function processUser(user: User): string
            return getUserName(user)
        end
    "#,
    );

    // Main application
    create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import type { User } from './types'
        import { processUser } from './api'
        import { getUserName } from './models'
        function main(): string
            return "ok"
        end
    "#,
    );

    compile_all_in_dir(temp_dir.path());

    assert_compilation_success(
        temp_dir.path(),
        &["types.lua", "models.lua", "api.lua", "main.lua"],
    );
}

#[test]
fn test_plugin_architecture_pattern() {
    let temp_dir = TempDir::new().unwrap();

    // Plugin interface definition
    create_test_file(
        temp_dir.path(),
        "plugin_interface",
        r#"
        export interface Plugin {
            name: string
            version: string
        }
    "#,
    );

    // Plugin A implementation
    create_test_file(
        temp_dir.path(),
        "plugin_a",
        r#"
        import type { Plugin } from './plugin_interface'
        export function getPluginA(): Plugin
            return { name = "Plugin A", version = "1.0" }
        end
    "#,
    );

    // Plugin B implementation
    create_test_file(
        temp_dir.path(),
        "plugin_b",
        r#"
        import type { Plugin } from './plugin_interface'
        export function getPluginB(): Plugin
            return { name = "Plugin B", version = "2.0" }
        end
    "#,
    );

    // Main app that loads plugins
    create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import { getPluginA } from './plugin_a'
        import { getPluginB } from './plugin_b'
        const a = getPluginA()
        const b = getPluginB()
        print(a.name)
        print(b.name)
    "#,
    );

    compile_all_in_dir(temp_dir.path());
}

#[test]
fn test_barrel_export_pattern() {
    let temp_dir = TempDir::new().unwrap();

    // Create utils directory
    fs::create_dir(temp_dir.path().join("utils")).unwrap();

    create_test_file(
        temp_dir.path(),
        "utils/string.luax",
        r#"
        export function toUpper(s: string): string {
            return string.upper(s)
        }
    "#,
    );

    create_test_file(
        temp_dir.path(),
        "utils/math.luax",
        r#"
        export function square(n: number): number {
            return n * n
        }
    "#,
    );

    // Barrel export
    create_test_file(
        temp_dir.path(),
        "utils/index.luax",
        r#"
        export * from './string'
        export * from './math'
    "#,
    );

    // Main that uses barrel
    create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import { toUpper, square } from './utils/index'
        print(toUpper("hello"))
        print(square(5))
    "#,
    );

    compile_all_in_dir(temp_dir.path());

    assert_compilation_success(
        temp_dir.path(),
        &[
            "utils/string.lua",
            "utils/math.lua",
            "utils/index.lua",
            "main.lua",
        ],
    );
}

// ============================================================================
// MODULE RESOLUTION ERROR TESTS (4 tests)
// ============================================================================

#[test]
fn test_module_not_found_error() {
    let temp_dir = TempDir::new().unwrap();
    let main_file = create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import { foo } from './nonexistent'
        print(foo)
    "#,
    );

    let output = luanext_cmd()
        .arg(main_file.to_str().unwrap())
        .output()
        .unwrap();

    assert_compilation_error(output, "Cannot find module");
}

#[test]
fn test_missing_export_error() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(
        temp_dir.path(),
        "module",
        r#"
        export const foo = "hello"
    "#,
    );
    let main_file = create_test_file(
        temp_dir.path(),
        "main",
        r#"
        import { bar } from './module'
        print(bar)
    "#,
    );

    let output = luanext_cmd()
        .arg(main_file.to_str().unwrap())
        .output()
        .unwrap();

    assert!(!output.status.success());
}
