# Testing Guide

This document provides comprehensive guidance on testing practices in the LuaNext compiler project.

## Table of Contents

1. [Testing Philosophy](#testing-philosophy)
2. [Test Organization](#test-organization)
3. [Unit Testing Patterns](#unit-testing-patterns)
4. [Integration Testing](#integration-testing)
5. [Test Data Management](#test-data-management)
6. [Snapshot Testing with Insta](#snapshot-testing-with-insta)
7. [Property-Based Testing](#property-based-testing)
8. [Running Tests](#running-tests)
9. [Code Coverage](#code-coverage)
10. [Test Helpers and Utilities](#test-helpers-and-utilities)
11. [CI/CD Testing](#cicd-testing)

---

## Testing Philosophy

LuaNext follows a pragmatic testing approach that balances thorough coverage with maintainability:

### Core Principles

- **70% Coverage Target**: The project enforces a minimum 70% line coverage threshold via CI
- **Unit Tests Over Integration Tests**: Prefer unit tests in the same file as implementation (`#[cfg(test)]`)
- **Fast Feedback**: Tests should run quickly to enable rapid development cycles
- **No Flaky Tests**: Tests must be deterministic and reliable
- **Fix, Don't Delete**: Never delete failing tests—fix the code or update the test with an explanation

### Test Categories

```
Unit Tests (Primary)
├─ Inline tests (#[cfg(test)] modules)
├─ Fast, focused tests for individual components
└─ Co-located with implementation code

Integration Tests (Secondary)
├─ Tests in tests/ directories
├─ End-to-end compiler pipeline tests
└─ CLI command tests

Stress Tests
├─ Large-scale input handling (10K+ elements)
├─ Deep nesting (500+ levels)
└─ Performance benchmarks
```

---

## Test Organization

### Directory Structure

```
crates/
├─ luanext-core/
│  ├─ src/
│  │  ├─ codegen/
│  │  │  ├─ mod.rs            # Contains #[cfg(test)] inline tests
│  │  │  ├─ snapshots/        # Insta snapshot files
│  │  │  └─ ...
│  │  ├─ optimizer/
│  │  │  ├─ devirtualization.rs  # Contains #[cfg(test)] tests
│  │  │  └─ ...
│  │  └─ di.rs                # Contains #[cfg(test)] DI tests
│  └─ tests/                  # Integration tests (100+ files)
│     ├─ pattern_matching_tests.rs
│     ├─ stress_tests.rs
│     ├─ optimizer_integration_tests.rs
│     └─ ...
├─ luanext-cli/
│  └─ tests/
│     ├─ cli_features_tests.rs
│     ├─ integration_tests.rs
│     └─ watch_mode_tests.rs
└─ luanext-lsp/
   └─ tests/
      ├─ di_tests.rs
      └─ features_integration_test.rs
```

### File Naming Conventions

- Unit tests: `#[cfg(test)] mod tests { ... }` in source files
- Integration tests: `*_tests.rs` or `*_test.rs` in `tests/` directories
- Snapshot files: Auto-generated in `snapshots/` directories
- Test helpers: `test_utils/`, `test_helpers/` modules

---

## Unit Testing Patterns

### Inline Tests with #[cfg(test)]

Unit tests are co-located with implementation code using `#[cfg(test)]` modules:

```rust
// crates/luanext-core/src/optimizer/devirtualization.rs
pub struct ClassHierarchy {
    // ... implementation
}

impl ClassHierarchy {
    pub fn build<'arena>(program: &Program<'arena>) -> Self {
        // ... implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_hierarchy_basic() {
        let input = r#"
            class Base { }
            class Child extends Base { }
        "#;
        let hierarchy = ClassHierarchy::build(&parse(input));
        assert_eq!(hierarchy.children_count(), 1);
    }

    #[test]
    fn test_final_class_detection() {
        let input = "final class Sealed { }";
        let hierarchy = ClassHierarchy::build(&parse(input));
        assert!(hierarchy.is_final("Sealed"));
    }
}
```

### Dependency Injection for Testability

Use the `DiContainer` pattern for testable services:

```rust
// crates/luanext-core/src/di.rs
impl DiContainer {
    pub fn test_default() -> Self {
        let config = CompilerConfig::default();
        let diagnostics = Arc::new(CollectingDiagnosticHandler::new());
        let fs = Arc::new(MockFileSystem::new());
        Self::test(config, diagnostics, fs)
    }

    pub fn test_with_config(config: CompilerConfig) -> Self {
        let diagnostics = Arc::new(CollectingDiagnosticHandler::new());
        let fs = Arc::new(MockFileSystem::new());
        Self::test(config, diagnostics, fs)
    }
}
```

### Common Test Helper Pattern

```rust
fn compile_and_check(source: &str) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile(source)
}

#[test]
fn test_type_inference() {
    let source = r#"
        const x = 42
        const y = "hello"
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Type inference should succeed");
}
```

---

## Integration Testing

### CLI Integration Tests

Integration tests for the CLI use `assert_cmd` and `tempfile`:

```rust
// crates/luanext-cli/tests/cli_features_tests.rs
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn luanext_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("luanext"))
}

#[test]
fn test_compile_with_config_file() {
    let temp_dir = TempDir::new().unwrap();

    // Create config
    let config = r#"
compilerOptions:
  target: "5.1"
  outDir: "./output"
  strict: true
"#;
    fs::write(temp_dir.path().join("luanext.config.yaml"), config).unwrap();

    // Create source file
    fs::write(temp_dir.path().join("test.luax"), "const x: number = 42").unwrap();

    luanext_cmd()
        .current_dir(&temp_dir)
        .arg("--project")
        .arg("luanext.config.yaml")
        .arg("test.luax")
        .assert()
        .success();

    assert!(temp_dir.path().join("output/test.lua").exists());
}

#[test]
fn test_error_reporting() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("error.luax");

    fs::write(&input_file, "const x: number = \"wrong\"").unwrap();

    let output = luanext_cmd().arg(&input_file).output().unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Type mismatch"));
}
```

### Compiler Pipeline Tests

Full pipeline tests in `luanext-core/tests/`:

```rust
// crates/luanext-core/tests/pattern_matching_tests.rs
use luanext_core::di::DiContainer;

fn compile_and_check(source: &str) -> Result<String, String> {
    let mut container = DiContainer::test_default();
    container.compile(source)
}

#[test]
fn test_simple_literal_match() {
    let source = r#"
        const x = 5
        const result = match x {
            1 => "one",
            2 => "two",
            5 => "five",
            _ => "other"
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Simple literal match should compile");
    let output = result.unwrap();
    assert!(output.contains("(function()"), "Should generate an IIFE");
}

#[test]
fn test_match_with_guard() {
    let source = r#"
        const x: number = 15
        const result = match x {
            n if n > 10 => "big"
            n if n > 5 => "medium"
            _ => "small"
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok(), "Match with guard should compile");
}
```

### Stress and Scale Tests

```rust
// crates/luanext-core/tests/stress_tests.rs
#[test]
fn test_large_array_literal_10k() {
    let mut input = String::from("const arr: number[] = {");
    for i in 0..10000 {
        if i > 0 {
            input.push_str(", ");
        }
        input.push_str(&i.to_string());
    }
    input.push('}');

    assert!(
        lex_and_parse(&input),
        "Should parse large array literal with 10K elements"
    );
}

#[test]
fn test_deeply_nested_expressions() {
    // Test with 500 levels - no stack overflow
    let mut input = String::from("const x: number = ");
    for _ in 0..500 {
        input.push('(');
    }
    input.push_str("42");
    for _ in 0..500 {
        input.push(')');
    }

    assert!(
        lex_and_parse(&input),
        "Should parse extremely deep nesting (500 levels)"
    );
}
```

---

## Test Data Management

### Inline Test Data

Prefer inline test data using raw string literals:

```rust
#[test]
fn test_class_with_methods() {
    let source = r#"
        class Calculator {
            add(a: number, b: number): number {
                return a + b
            }
        }
    "#;

    let result = compile_and_check(source);
    assert!(result.is_ok());
}
```

### Temporary Files with TempDir

For file-based tests, use `tempfile::TempDir`:

```rust
use tempfile::TempDir;
use std::fs;

#[test]
fn test_multi_file_compilation() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple test files
    fs::write(temp_dir.path().join("module1.luax"), "export const x = 42").unwrap();
    fs::write(temp_dir.path().join("module2.luax"), "import { x } from './module1'").unwrap();

    // Run compilation
    let result = compile_project(temp_dir.path());
    assert!(result.is_ok());

    // TempDir is automatically cleaned up when it goes out of scope
}
```

### Mock File System for Unit Tests

```rust
use luanext_core::fs::MockFileSystem;

#[test]
fn test_module_resolution() {
    let mut fs = MockFileSystem::new();
    fs.add_file("/src/main.luax", "import { foo } from './lib'");
    fs.add_file("/src/lib.luax", "export const foo = 42");

    let resolver = ModuleResolver::new(Arc::new(fs));
    let resolved = resolver.resolve("/src/main.luax", "./lib").unwrap();
    assert_eq!(resolved, "/src/lib.luax");
}
```

---

## Snapshot Testing with Insta

LuaNext uses [Insta](https://insta.rs/) for snapshot testing of code generation:

### Basic Snapshot Testing

```rust
// crates/luanext-core/src/codegen/mod.rs
#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn snapshot_variable_declarations() {
        let source = r#"
            const x: number = 42
            const y: string = "hello"
            const z: boolean = true
        "#;

        let output = generate_code(source);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn snapshot_arrow_function() {
        let source = "const add = (a: number, b: number) => a + b";
        let output = generate_code(source);
        insta::assert_snapshot!(output);
    }
}
```

### Snapshot Files

Snapshots are stored in `snapshots/` directories:

```
crates/luanext-core/src/codegen/snapshots/
├─ luanext_core__codegen__tests__snapshot_arrays_and_objects.snap
├─ luanext_core__codegen__tests__snapshot_arrow_function.snap
├─ luanext_core__codegen__tests__snapshot_control_flow.snap
└─ ...
```

### Reviewing Snapshot Changes

```bash
# Review snapshot changes
cargo insta review

# Accept all changes
cargo insta accept

# Reject all changes
cargo insta reject

# Test with inline snapshots
cargo insta test
```

### Best Practices

- Use descriptive test names (they become snapshot file names)
- Review snapshot diffs carefully in PRs
- Commit snapshot files to version control
- Use snapshots for stable outputs (codegen, error messages)

---

## Property-Based Testing

LuaNext includes property-based tests using [proptest](https://proptest-rs.github.io/proptest/):

### Setup

Property tests are configured in `Cargo.toml`:

```toml
[dev-dependencies]
proptest = "1.5"
```

### Regression Files

Proptest stores failing cases in regression files:

```
crates/luanext-core/tests/property_tests.proptest-regressions
```

These files should be committed to version control to ensure regressions don't reoccur.

### Example Property Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_lexer_never_panics(s in "\\PC*") {
        // Property: Lexer should never panic on any input
        let _ = lex(s);
    }

    #[test]
    fn test_roundtrip_numbers(n in any::<f64>()) {
        // Property: Number parsing should roundtrip
        let source = format!("const x = {}", n);
        let parsed = parse_and_extract_number(&source);
        prop_assert_eq!(parsed, n);
    }
}
```

---

## Running Tests

### Basic Test Commands

```bash
# Run all tests
cargo test --all

# Run tests for specific crate
cargo test -p luanext-core
cargo test -p luanext-cli

# Run specific test
cargo test test_pattern_matching

# Run with output
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored

# Run tests in parallel (default)
cargo test -- --test-threads=8

# Run tests sequentially
cargo test -- --test-threads=1
```

### Integration Tests Only

```bash
# Run only integration tests
cargo test --test '*'

# Run specific integration test file
cargo test --test cli_features_tests
```

### Debug Tests

```bash
# Show test output
cargo test -- --nocapture --test-threads=1

# Run specific test with logging
RUST_LOG=debug cargo test test_name -- --nocapture
```

### Watch Mode

```bash
# Install cargo-watch
cargo install cargo-watch

# Run tests on file changes
cargo watch -x test

# Run specific tests on changes
cargo watch -x "test pattern_matching"
```

---

## Code Coverage

### Tarpaulin Configuration

Coverage is configured in `tarpaulin.toml`:

```toml
[typedlua_coverage]
workspace = true
all-targets = true
all-features = true
test-timeout = "120s"

# Exclude patterns
exclude-files = [
    "fuzz/*",
    "*/tests/*",
    "*/test_*.rs",
]

# 70% threshold
fail-under = 70.0

# Use LLVM for accuracy
engine = "Llvm"

# Output formats
out = ["Xml", "Html", "Lcov"]

include-tests = false
run-types = ["Tests", "Lib"]
```

### Running Coverage Locally

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Run coverage with script
./scripts/coverage.sh

# Open HTML report automatically
./scripts/coverage.sh --open

# Run coverage manually
cargo tarpaulin --config-file tarpaulin.toml --verbose
```

### Coverage Reports

Tarpaulin generates three report formats:

1. **XML** (`tarpaulin-report.xml`): For CI integration, Codecov
2. **HTML** (`tarpaulin-report.html`): Interactive browser view
3. **LCOV** (`lcov.info`): For IDE integration

### CI Coverage Enforcement

The CI pipeline enforces 70% coverage:

```yaml
# .github/workflows/ci.yml
- name: Check coverage threshold
  run: |
    COVERAGE=$(grep -o 'line-rate="[0-9.]*"' tarpaulin-report.xml | head -1 | cut -d'"' -f2)
    COVERAGE_PCT=$(echo "$COVERAGE * 100" | bc)
    echo "Line coverage: ${COVERAGE_PCT}%"

    if (( $(echo "$COVERAGE_PCT < 70.0" | bc -l) )); then
      echo "ERROR: Coverage ${COVERAGE_PCT}% is below threshold of 70%"
      exit 1
    fi
    echo "✅ Coverage ${COVERAGE_PCT}% meets threshold of 70%"
```

### Improving Coverage

Focus on these areas for coverage improvements:

1. **Error paths**: Test error conditions and edge cases
2. **Match arms**: Ensure all enum variants are covered
3. **Conditional branches**: Test both true and false paths
4. **Loop variations**: Test empty, single, and multiple iterations

---

## Test Helpers and Utilities

### Test Helper Crates

```
crates/luanext-test-helpers/
└─ Common utilities shared across test suites
```

### Common Helper Patterns

```rust
// Helper: Create test container with default config
fn test_container() -> DiContainer {
    DiContainer::test_default()
}

// Helper: Compile source and unwrap result
fn must_compile(source: &str) -> String {
    compile_and_check(source).expect("Compilation should succeed")
}

// Helper: Expect compilation failure
fn must_fail(source: &str) -> String {
    compile_and_check(source).expect_err("Compilation should fail")
}

// Helper: Parse and return AST
fn parse(source: &str) -> Program {
    let arena = Bump::new();
    let (interner, common_ids) = StringInterner::new_with_common_identifiers();
    let handler = Arc::new(CollectingDiagnosticHandler::new());

    let mut lexer = Lexer::new(source, handler.clone(), &interner);
    let tokens = lexer.tokenize().unwrap();

    let mut parser = Parser::new(tokens, handler, &interner, &common_ids, &arena);
    parser.parse().unwrap()
}
```

---

## CI/CD Testing

### GitHub Actions Workflow

```yaml
# .github/workflows/ci.yml
jobs:
  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Run tests
        run: cargo test --all --verbose

  fmt:
    name: Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - name: Check formatting
        run: cargo fmt --all -- --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      - name: Generate coverage
        run: cargo tarpaulin --config-file tarpaulin.toml --verbose
      - name: Upload to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: ./tarpaulin-report.xml,./lcov.info
```

### Multi-Platform Testing

```yaml
build:
  name: Build
  runs-on: ${{ matrix.os }}
  strategy:
    matrix:
      os: [ubuntu-latest, macos-latest, windows-latest]
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - name: Build
      run: cargo build --release --verbose
```

---

## Best Practices Summary

### DO

✅ Write tests before fixing bugs (TDD for bug fixes)
✅ Use `#[cfg(test)]` for unit tests in the same file
✅ Use descriptive test names that explain what's being tested
✅ Test error paths and edge cases
✅ Use `assert_cmd` for CLI integration tests
✅ Use `tempfile::TempDir` for file-based tests
✅ Review snapshot changes carefully
✅ Commit regression files to version control
✅ Run `cargo clippy` and `cargo fmt` before committing

### DON'T

❌ Delete failing tests without explanation
❌ Use `#[allow(clippy::...)]` to suppress warnings
❌ Skip error case testing
❌ Write flaky or non-deterministic tests
❌ Hard-code absolute paths in tests
❌ Leave commented-out test code
❌ Use `unwrap()` without context in test failures
❌ Test implementation details instead of behavior

---

## Additional Resources

- **Rust Testing Book**: https://doc.rust-lang.org/book/ch11-00-testing.html
- **Insta Documentation**: https://insta.rs/
- **Proptest Guide**: https://proptest-rs.github.io/proptest/
- **Cargo Tarpaulin**: https://github.com/xd009642/tarpaulin
- **Assert CMD**: https://docs.rs/assert_cmd/

For project-specific testing patterns, see:
- `/Users/forge18/Repos/luanext/crates/luanext-core/tests/` (100+ integration tests)
- `/Users/forge18/Repos/luanext/crates/luanext-cli/tests/` (CLI tests)
- `/Users/forge18/Repos/luanext/CLAUDE.md` (Project guidelines)

Agent is calibrated...
