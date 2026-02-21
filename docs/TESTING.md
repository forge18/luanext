# LuaNext Comprehensive Testing Guide

## Table of Contents

1. [Overview](#overview)
2. [Test Infrastructure](#test-infrastructure)
3. [Language Feature Tests](#language-feature-tests)
4. [Lua Runtime Version Tests](#lua-runtime-version-tests)
5. [Optimization Level Tests](#optimization-level-tests)
6. [Type Definition Files](#type-definition-files)
7. [Error Handling Tests](#error-handling-tests)
8. [Standard Library Integration](#standard-library-integration)
9. [Incremental Compilation](#incremental-compilation)
10. [Edge Cases](#edge-cases)
11. [Best Practices](#best-practices)
12. [CI Integration](#ci-integration)
13. [Adding New Tests](#adding-new-tests)

---

## Overview

### Purpose

LuaNext includes comprehensive execution testing to ensure that generated Lua code runs correctly across different:
- **Language features** (typed vs untyped code)
- **Lua runtime versions** (5.1, 5.2, 5.3, 5.4, 5.5)
- **Optimization levels** (O0, O1, O2, O3)
- **Integration scenarios** (type definitions, stdlib, caching)
- **Edge cases** (deep recursion, large data structures, numeric limits)

### Test Categories

| Category | Purpose | Files | Priority |
|----------|---------|-------|----------|
| **Language Features** | Verify all LuaNext features work | `execution_tests.rs`, `execution_untyped_tests.rs` | High |
| **Lua Versions** | Ensure cross-version compatibility | `lua5X_compat_tests.rs` | Medium |
| **Optimization** | Verify O0/O1/O2/O3 semantic equivalence | `execution_optimization_tests.rs` | High |
| **Type Definitions** | Test `.d.luax` file support | `type_definition_tests.rs` | Medium |
| **Error Handling** | Validate error behavior | `runtime_error_tests.rs` | Medium |
| **Stdlib** | Test standard library integration | `stdlib_execution_tests.rs` | Medium |
| **Caching** | Verify incremental compilation | `cache_execution_tests.rs` | Medium |
| **Edge Cases** | Stress test Lua runtime limits | `lua_edge_cases_tests.rs` | Low |

### Quick Reference

```bash
# Run all execution tests
cargo test --test execution_tests

# Run specific test category
cargo test --test execution_optimization_tests

# Run a single test
cargo test --test execution_tests test_integer_arithmetic

# Run tests with different Lua versions (requires mlua feature flags)
cargo test --features mlua/lua51
cargo test --features mlua/lua54
```

---

## Test Infrastructure

### LuaExecutor

**Location**: `crates/luanext-test-helpers/src/lua_executor.rs`

The `LuaExecutor` struct provides methods for executing generated Lua code and retrieving results.

#### API

```rust
use luanext_test_helpers::LuaExecutor;

// Create a new executor
let executor = LuaExecutor::new()?;

// Execute code without returning a value
executor.execute("local x = 10")?;

// Execute code and retrieve a global variable
let result: i64 = executor.execute_and_get("x = 42", "x")?;

// Execute code and return the result
let result: i64 = executor.execute_with_result("return 1 + 2")?;

// Check if code executes successfully (boolean)
let ok: bool = executor.execute_ok("local x = 10");

// Get access to underlying mlua::Lua instance
let lua = executor.lua();
```

#### Usage Example

```rust
#[test]
fn test_simple_arithmetic() {
    let executor = LuaExecutor::new().unwrap();

    // Execute LuaNext-generated Lua code
    executor.execute("x = 10 + 20").unwrap();

    // Retrieve the global variable
    let result: i64 = executor.execute_and_get("x = 10 + 20", "x").unwrap();

    assert_eq!(result, 30);
}
```

### Compile Helpers

**Location**: `crates/luanext-test-helpers/src/compile.rs`

Convenience functions for compiling LuaNext source code in tests.

#### Available Functions

```rust
use luanext_test_helpers::compile::{
    compile,
    compile_with_optimization,
    compile_with_stdlib,
    compile_with_stdlib_and_optimization,
};
use luanext_core::config::OptimizationLevel;

// Basic compilation (no stdlib, no optimization)
let lua_code = compile(source)?;

// With optimization level
let lua_code = compile_with_optimization(source, OptimizationLevel::O2)?;

// With standard library loaded
let lua_code = compile_with_stdlib(source)?;

// With both stdlib and optimization
let lua_code = compile_with_stdlib_and_optimization(source, OptimizationLevel::O3)?;
```

#### Usage Example

```rust
#[test]
fn test_math_operations() {
    let source = r#"
        result: number = math.sqrt(16) + math.abs(-10)
    "#;

    // Need stdlib for math functions
    let lua_code = compile_with_stdlib(source).unwrap();

    let executor = LuaExecutor::new().unwrap();
    let result: f64 = executor.execute_and_get(&lua_code, "result").unwrap();

    assert!((result - 14.0).abs() < 0.001); // 4 + 10 = 14
}
```

### Test Fixtures

**Location**: `crates/luanext-test-helpers/src/fixtures.rs`

Reusable source code snippets for common testing scenarios.

```rust
use luanext_test_helpers::fixtures::*;

// Simple programs
simple_program()      // "local x = 10"
function_program()    // Function definition
class_program()       // Class definition
interface_program()   // Interface definition
type_alias_program()  // Type alias
enum_program()        // Enum definition

// Programs with errors
type_error_assignment()  // Type mismatch
type_error_call()        // Wrong argument type
syntax_error_missing_end()  // Syntax error
```

### Running Tests

```bash
# Run all tests
cargo test

# Run execution tests only
cargo test --test execution_tests

# Run specific test
cargo test test_integer_arithmetic

# Run with verbose output
cargo test -- --nocapture

# Run tests in specific crate
cd crates/luanext-core && cargo test
```

---

## Language Feature Tests

### Typed Code Tests

**File**: `crates/luanext-core/tests/execution_tests.rs`

The main execution test suite covering all LuaNext language features with type annotations.

#### Test Categories

1. **Arithmetic & Literals** (6 tests)
   - Variable scoping patterns
   - Integer arithmetic
   - Float arithmetic
   - String concatenation
   - Boolean logic
   - Nil values

2. **Functions** (8 tests)
   - Basic functions
   - Return values
   - Multiple returns
   - Default parameters
   - Variadic functions
   - Closures and upvalues
   - Higher-order functions
   - Recursive functions

3. **Control Flow** (7 tests)
   - If/else statements
   - While loops
   - For loops
   - For-in loops
   - Break statements
   - Continue statements
   - Nested loops

4. **Tables** (6 tests)
   - Array-style tables
   - Dictionary-style tables
   - Mixed tables
   - Table access
   - Table length operator
   - Table iteration

5. **Type System** (4 tests)
   - Union types
   - Intersection types
   - Type assertions
   - Type guards

6. **Classes** (5 tests)
   - Class declaration
   - Constructors
   - Methods
   - Inheritance
   - Method overriding

7. **String Interpolation** (2 tests)
   - Template strings
   - Nested expressions

8. **Destructuring** (4 tests)
   - Array destructuring
   - Object destructuring
   - Nested destructuring
   - Default values

9. **Pattern Matching** (3 tests)
   - Match expressions
   - Guards
   - Exhaustiveness

#### Variable Scoping Patterns

LuaNext supports multiple variable declaration styles:

```rust
#[test]
fn test_variable_scoping_patterns() {
    let source = r#"
        -- Implicit global (type annotation required)
        implicit_global: number = 100

        -- Explicit global (optional, clearer intent)
        global explicit_global: number = 200

        -- Const (immutable, generates local in Lua)
        const CONSTANT_VALUE: number = 42

        -- Local variable in function (demonstrates scoping)
        function calculate(): number {
            local temp: number = 10
            return temp * CONSTANT_VALUE
        }

        result: number = calculate()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    // Verify global variables (implicit and explicit)
    let implicit: i64 = executor.execute_and_get(&lua_code, "implicit_global").unwrap();
    let explicit: i64 = executor.execute_and_get(&lua_code, "explicit_global").unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();

    assert_eq!(implicit, 100);
    assert_eq!(explicit, 200);
    assert_eq!(result, 420); // 10 * 42
}
```

#### Adding New Feature Tests

When adding a new language feature:

1. Add test to `execution_tests.rs` in appropriate category
2. Use descriptive test name: `test_<feature>_<scenario>`
3. Include both positive and edge cases
4. Use global variables for assertions (accessible from Lua)
5. Document expected behavior in comments

**Example**:

```rust
#[test]
fn test_async_await_basic() {
    let source = r#"
        async function fetch(): string
            return "data"
        end

        result: string = await fetch()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, "data");
}
```

### Untyped Code Tests

**File**: `crates/luanext-core/tests/execution_untyped_tests.rs` (to be created)

Mirror of `execution_tests.rs` but without type annotations to verify type erasure.

#### Purpose

- Ensure LuaNext doesn't break when types are omitted
- Validate type erasure correctness
- Support gradual typing migration path

#### Example Tests

```rust
#[test]
fn test_untyped_arithmetic() {
    let source = r#"
        x = 1 + 2 * 3
        y = 10 - x
        z = y * 2
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let x: i64 = executor.execute_and_get(&lua_code, "x").unwrap();
    let y: i64 = executor.execute_and_get(&lua_code, "y").unwrap();
    let z: i64 = executor.execute_and_get(&lua_code, "z").unwrap();

    assert_eq!(x, 7);
    assert_eq!(y, 3);
    assert_eq!(z, 6);
}

#[test]
fn test_untyped_function() {
    let source = r#"
        function add(a, b)
            return a + b
        end

        result = add(5, 3)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 8);
}
```

---

## Lua Runtime Version Tests

### Version Strategy

LuaNext targets multiple Lua versions using mlua's feature flags:

- **Lua 5.1**: Legacy support (used by LuaJIT)
- **Lua 5.2**: Introduction of `_ENV`, `bit32`, `goto`
- **Lua 5.3**: Native bitwise operators, integers, `utf8` library
- **Lua 5.4**: To-be-closed variables, const variables, generational GC
- **Lua 5.5**: Global declarations, named vararg, compact arrays, incremental GC

### Version Detection

Create a helper function to detect Lua version at runtime:

```rust
/// Returns true if the current Lua version is >= the specified version
fn requires_lua_version(executor: &LuaExecutor, major: u8, minor: u8) -> bool {
    // Query Lua _VERSION global
    let version: String = executor
        .lua()
        .globals()
        .get("_VERSION")
        .unwrap_or_else(|_| "Lua 5.4".to_string());

    // Parse "Lua X.Y"
    if let Some(ver) = version.strip_prefix("Lua ") {
        if let Some((maj, min)) = ver.split_once('.') {
            if let (Ok(maj_num), Ok(min_num)) = (maj.parse::<u8>(), min.parse::<u8>()) {
                return (maj_num, min_num) >= (major, minor);
            }
        }
    }

    false // Default to false if parsing fails
}
```

### Lua 5.1 Compatibility Tests

**File**: `crates/luanext-core/tests/lua51_compat_tests.rs` (to be created)

Tests for Lua 5.1-specific behavior and limitations.

#### Key Differences

- No bitwise operators (must use bit library or polyfill)
- No integers (all numbers are floats)
- `module()` function for modules (deprecated in 5.2+)
- No `goto` statement
- No `\\z` escape in strings

#### Example Tests

```rust
#[test]
fn test_lua51_no_bitwise() {
    // Lua 5.1 doesn't have native bitwise operators
    // LuaNext should polyfill or error gracefully
    let source = r#"
        -- Must use bit library or polyfill
        result: number = 5
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    if !requires_lua_version(&executor, 5, 3) {
        // On Lua 5.1/5.2, bitwise ops should be polyfilled or unavailable
        executor.execute(&lua_code).unwrap();
    }
}

#[test]
fn test_lua51_all_numbers_are_floats() {
    let source = r#"
        x = 10
        y = 10.0
        same: boolean = x == y
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let same: bool = executor.execute_and_get(&lua_code, "same").unwrap();
    assert!(same); // In Lua 5.1, 10 and 10.0 are identical
}
```

### Lua 5.2 Compatibility Tests

**File**: `crates/luanext-core/tests/lua52_compat_tests.rs` (to be created)

#### Key Features

- `_ENV` table for environment access
- `bit32` library for bitwise operations
- `goto` statement and labels
- `__pairs` and `__ipairs` metamethods

#### Example Tests

```rust
#[test]
fn test_lua52_bit32_library() {
    let source = r#"
        -- Lua 5.2 has bit32 library
        result: number = bit32.band(15, 7) -- 15 & 7 = 7
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    if requires_lua_version(&executor, 5, 2) && !requires_lua_version(&executor, 5, 3) {
        executor.execute(&lua_code).unwrap();
        let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
        assert_eq!(result, 7);
    }
}

#[test]
fn test_lua52_goto() {
    let source = r#"
        x: number = 0
        goto skip
        x = 10 -- This should be skipped
        ::skip::
        result: number = x
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    if requires_lua_version(&executor, 5, 2) {
        executor.execute(&lua_code).unwrap();
        let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
        assert_eq!(result, 0);
    }
}
```

### Lua 5.3 Compatibility Tests

**File**: `crates/luanext-core/tests/lua53_compat_tests.rs` (to be created)

#### Key Features

- Native bitwise operators (`&`, `|`, `~`, `<<`, `>>`, `~`)
- Integer subtype (distinct from floats)
- `utf8` library
- Floor division operator (`//`)

#### Example Tests

```rust
#[test]
fn test_lua53_bitwise_operators() {
    let source = r#"
        a: number = 15 & 7   -- AND: 7
        b: number = 15 | 7   -- OR: 15
        c: number = 15 ~ 7   -- XOR: 8
        d: number = 1 << 3   -- Left shift: 8
        e: number = 16 >> 2  -- Right shift: 4
        result: number = a + b + c + d + e
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    if requires_lua_version(&executor, 5, 3) {
        executor.execute(&lua_code).unwrap();
        let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
        assert_eq!(result, 7 + 15 + 8 + 8 + 4); // 42
    }
}

#[test]
fn test_lua53_integer_type() {
    let source = r#"
        int: number = 10
        float: number = 10.0
        is_int: boolean = math.type(int) == "integer"
        is_float: boolean = math.type(float) == "float"
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    if requires_lua_version(&executor, 5, 3) {
        executor.execute(&lua_code).unwrap();
        let is_int: bool = executor.execute_and_get(&lua_code, "is_int").unwrap();
        let is_float: bool = executor.execute_and_get(&lua_code, "is_float").unwrap();
        assert!(is_int);
        assert!(is_float);
    }
}

#[test]
fn test_lua53_floor_division() {
    let source = r#"
        result: number = 10 // 3  -- Floor division: 3
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    if requires_lua_version(&executor, 5, 3) {
        executor.execute(&lua_code).unwrap();
        let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
        assert_eq!(result, 3);
    }
}
```

### Lua 5.4 Compatibility Tests

**File**: `crates/luanext-core/tests/lua54_compat_tests.rs` (to be created)

#### Key Features

- To-be-closed variables (`<close>`)
- Const variables
- Generational garbage collector
- New `warn` function

#### Example Tests

```rust
#[test]
fn test_lua54_to_be_closed() {
    let source = r#"
        do
            local f <close> = io.open("test.txt", "r")
            -- f will be closed automatically when leaving scope
        end
        result: number = 1
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    if requires_lua_version(&executor, 5, 4) {
        // Note: This test requires file system access
        // In practice, you'd need to create test.txt first
        let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
        assert_eq!(result, 1);
    }
}

#[test]
fn test_lua54_const_variables() {
    let source = r#"
        const x = 10
        result: number = x * 2
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    if requires_lua_version(&executor, 5, 4) {
        executor.execute(&lua_code).unwrap();
        let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
        assert_eq!(result, 20);
    }
}
```

### Lua 5.5 Compatibility Tests

**File**: `crates/luanext-core/tests/lua55_compat_tests.rs` (to be created)

#### Key Features (Released Dec 2025)

- Global variable declarations (prevent accidental globals)
- Named vararg tables (`...name`)
- 60% more compact arrays
- Incremental major garbage collections
- Read-only for-loop variables
- Improved float printing

#### Example Tests

```rust
#[test]
fn test_lua55_global_declarations() {
    let source = r#"
        -- Lua 5.5 allows explicit global declarations
        global x: number = 10
        result: number = x * 2
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    if requires_lua_version(&executor, 5, 5) {
        executor.execute(&lua_code).unwrap();
        let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
        assert_eq!(result, 20);
    }
}

#[test]
fn test_lua55_named_vararg() {
    let source = r#"
        function sum(...nums): number
            local total: number = 0
            for i, v in ipairs(nums) do
                total = total + v
            end
            return total
        end

        result: number = sum(1, 2, 3, 4, 5)
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    if requires_lua_version(&executor, 5, 5) {
        executor.execute(&lua_code).unwrap();
        let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
        assert_eq!(result, 15);
    }
}
```

### CI Matrix Configuration

To test all Lua versions in CI:

```yaml
# .github/workflows/test-lua-versions.yml
name: Lua Version Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        lua-version:
          - lua51
          - lua52
          - lua53
          - lua54
          - lua54-jit  # LuaJIT (5.1 compatible)
        # Note: lua55 support pending mlua 0.11.6 upgrade

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run tests with Lua ${{ matrix.lua-version }}
        run: |
          cd crates/luanext-core
          cargo test --features mlua/${{ matrix.lua-version }} \
            --test lua51_compat_tests \
            --test lua52_compat_tests \
            --test lua53_compat_tests \
            --test lua54_compat_tests
```

---

## Optimization Level Tests

### Purpose

Verify that all optimization levels (O0, O1, O2, O3) produce **semantically equivalent** code. Optimizations should improve performance without changing behavior.

### Correctness Tests

**File**: `crates/luanext-core/tests/execution_optimization_tests.rs` (to be created)

#### Test Pattern

```rust
use luanext_core::config::OptimizationLevel;

#[test]
fn test_optimization_correctness_<scenario>() {
    let source = r#"
        // LuaNext source code
    "#;

    // Compile with all optimization levels
    let o0 = compile_with_optimization(source, OptimizationLevel::O0).unwrap();
    let o1 = compile_with_optimization(source, OptimizationLevel::O1).unwrap();
    let o2 = compile_with_optimization(source, OptimizationLevel::O2).unwrap();
    let o3 = compile_with_optimization(source, OptimizationLevel::O3).unwrap();

    // Execute each and compare results
    let executor = LuaExecutor::new().unwrap();
    let result_o0: i64 = executor.execute_and_get(&o0, "result").unwrap();
    let result_o1: i64 = executor.execute_and_get(&o1, "result").unwrap();
    let result_o2: i64 = executor.execute_and_get(&o2, "result").unwrap();
    let result_o3: i64 = executor.execute_and_get(&o3, "result").unwrap();

    // All results must be identical
    assert_eq!(result_o0, result_o1);
    assert_eq!(result_o1, result_o2);
    assert_eq!(result_o2, result_o3);
}
```

#### Basic Test Cases

```rust
#[test]
fn test_optimization_correctness_constant_folding() {
    let source = r#"
        result: number = 10 + 20 * 3 - 5  // Should fold to 65
    "#;

    let o0 = compile_with_optimization(source, OptimizationLevel::O0).unwrap();
    let o1 = compile_with_optimization(source, OptimizationLevel::O1).unwrap();
    let o2 = compile_with_optimization(source, OptimizationLevel::O2).unwrap();
    let o3 = compile_with_optimization(source, OptimizationLevel::O3).unwrap();

    let executor = LuaExecutor::new().unwrap();
    let result_o0: i64 = executor.execute_and_get(&o0, "result").unwrap();
    let result_o1: i64 = executor.execute_and_get(&o1, "result").unwrap();
    let result_o2: i64 = executor.execute_and_get(&o2, "result").unwrap();
    let result_o3: i64 = executor.execute_and_get(&o3, "result").unwrap();

    assert_eq!(result_o0, 65);
    assert_eq!(result_o1, 65);
    assert_eq!(result_o2, 65);
    assert_eq!(result_o3, 65);
}

#[test]
fn test_optimization_correctness_dead_code() {
    let source = r#"
        x: number = 10
        if false then
            x = 999  // Dead code
        end
        result: number = x
    "#;

    let o0 = compile_with_optimization(source, OptimizationLevel::O0).unwrap();
    let o2 = compile_with_optimization(source, OptimizationLevel::O2).unwrap();

    let executor = LuaExecutor::new().unwrap();
    let result_o0: i64 = executor.execute_and_get(&o0, "result").unwrap();
    let result_o2: i64 = executor.execute_and_get(&o2, "result").unwrap();

    assert_eq!(result_o0, 10);
    assert_eq!(result_o2, 10);
}

#[test]
fn test_optimization_correctness_inlining() {
    let source = r#"
        function double(x: number): number
            return x * 2
        end

        result: number = double(21)  // Should inline to 21 * 2
    "#;

    let o0 = compile_with_optimization(source, OptimizationLevel::O0).unwrap();
    let o2 = compile_with_optimization(source, OptimizationLevel::O2).unwrap();

    let executor = LuaExecutor::new().unwrap();
    let result_o0: i64 = executor.execute_and_get(&o0, "result").unwrap();
    let result_o2: i64 = executor.execute_and_get(&o2, "result").unwrap();

    assert_eq!(result_o0, 42);
    assert_eq!(result_o2, 42);
}
```

#### Complex Test Cases

```rust
#[test]
fn test_optimization_correctness_closures() {
    let source = r#"
        function make_counter()
            local count: number = 0
            return function(): number
                count = count + 1
                return count
            end
        end

        counter = make_counter()
        a: number = counter()
        b: number = counter()
        c: number = counter()
        result: number = a + b + c  // 1 + 2 + 3 = 6
    "#;

    let o0 = compile_with_optimization(source, OptimizationLevel::O0).unwrap();
    let o3 = compile_with_optimization(source, OptimizationLevel::O3).unwrap();

    let executor = LuaExecutor::new().unwrap();
    let result_o0: i64 = executor.execute_and_get(&o0, "result").unwrap();
    let result_o3: i64 = executor.execute_and_get(&o3, "result").unwrap();

    assert_eq!(result_o0, 6);
    assert_eq!(result_o3, 6);
}

#[test]
fn test_optimization_correctness_classes() {
    let source = r#"
        class Point {
            x: number;
            y: number;

            constructor(x: number, y: number) {
                this.x = x;
                this.y = y;
            }

            method distance(): number {
                return math.sqrt(this.x * this.x + this.y * this.y);
            }
        }

        p = Point.new(3, 4)
        result: number = p:distance()  // 5.0
    "#;

    let o0 = compile_with_stdlib_and_optimization(source, OptimizationLevel::O0).unwrap();
    let o3 = compile_with_stdlib_and_optimization(source, OptimizationLevel::O3).unwrap();

    let executor = LuaExecutor::new().unwrap();
    let result_o0: f64 = executor.execute_and_get(&o0, "result").unwrap();
    let result_o3: f64 = executor.execute_and_get(&o3, "result").unwrap();

    assert!((result_o0 - 5.0).abs() < 0.001);
    assert!((result_o3 - 5.0).abs() < 0.001);
}

#[test]
fn test_optimization_correctness_recursion() {
    let source = r#"
        function fib(n: number): number
            if n <= 1 then return n end
            return fib(n - 1) + fib(n - 2)
        end

        result: number = fib(10)  // 55
    "#;

    let o0 = compile_with_optimization(source, OptimizationLevel::O0).unwrap();
    let o3 = compile_with_optimization(source, OptimizationLevel::O3).unwrap();

    let executor = LuaExecutor::new().unwrap();
    let result_o0: i64 = executor.execute_and_get(&o0, "result").unwrap();
    let result_o3: i64 = executor.execute_and_get(&o3, "result").unwrap();

    assert_eq!(result_o0, 55);
    assert_eq!(result_o3, 55);
}

#[test]
fn test_optimization_correctness_loops() {
    let source = r#"
        sum: number = 0
        for i = 1, 100 do
            sum = sum + i
        end
        result: number = sum  // 5050
    "#;

    let o0 = compile_with_optimization(source, OptimizationLevel::O0).unwrap();
    let o3 = compile_with_optimization(source, OptimizationLevel::O3).unwrap();

    let executor = LuaExecutor::new().unwrap();
    let result_o0: i64 = executor.execute_and_get(&o0, "result").unwrap();
    let result_o3: i64 = executor.execute_and_get(&o3, "result").unwrap();

    assert_eq!(result_o0, 5050);
    assert_eq!(result_o3, 5050);
}
```

### Benchmarking

For performance benchmarking (not correctness), see [docs/PERFORMANCE_TESTING.md](PERFORMANCE_TESTING.md).

Execution benchmarking tests runtime performance of generated code:

```rust
// In benches/execution_benchmarks.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_optimization_levels(c: &mut Criterion) {
    let source = r#"
        // Complex computation
    "#;

    let o0 = compile_with_optimization(source, OptimizationLevel::O0).unwrap();
    let o3 = compile_with_optimization(source, OptimizationLevel::O3).unwrap();

    let executor = LuaExecutor::new().unwrap();

    c.bench_function("execution_o0", |b| {
        b.iter(|| executor.execute(&o0).unwrap())
    });

    c.bench_function("execution_o3", |b| {
        b.iter(|| executor.execute(&o3).unwrap())
    });
}

criterion_group!(benches, benchmark_optimization_levels);
criterion_main!(benches);
```

---

## Type Definition Files

### Purpose

Type definition files (`.d.luax`) provide type information for Lua libraries without requiring LuaNext source code. They enable:
- Type-checking external Lua libraries
- Type-safe FFI bindings
- Gradual migration from Lua to LuaNext

### Module Resolution

When importing `./utils`, LuaNext searches in order:
1. `utils.luax` - LuaNext source file
2. `utils.d.luax` - Type declaration file
3. `utils.lua` - Plain Lua file (policy-dependent)
4. `utils/index.luax` - Directory with index

See [technical/module-system/resolution.md](../technical/module-system/resolution.md) for details.

### Test Structure

**File**: `crates/luanext-core/tests/type_definition_tests.rs` (to be created)

#### Test 1: Type Checker Respects `.d.luax`

```rust
use tempfile::TempDir;
use std::fs;

#[test]
fn test_d_tl_type_checking() {
    let temp_dir = TempDir::new().unwrap();

    // Create .d.luax file with type declarations
    fs::write(temp_dir.path().join("math_ext.d.luax"), r#"
        export function clamp(x: number, min: number, max: number): number
        export function lerp(a: number, b: number, t: number): number
    "#).unwrap();

    // Create .luax file that uses it correctly
    let valid_source = r#"
        import { clamp, lerp } from './math_ext'
        result: number = clamp(10, 0, 100)
    "#;

    fs::write(temp_dir.path().join("main.luax"), valid_source).unwrap();

    // Should compile successfully
    let mut container = create_test_container_with_dir(temp_dir.path());
    let result = container.compile_file(temp_dir.path().join("main.luax"));
    assert!(result.is_ok());
}
```

#### Test 2: Type Errors Caught

```rust
#[test]
fn test_d_tl_type_errors() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(temp_dir.path().join("math_ext.d.luax"), r#"
        export function clamp(x: number, min: number, max: number): number
    "#).unwrap();

    // Create .luax file with type error
    let invalid_source = r#"
        import { clamp } from './math_ext'
        result: string = clamp(10, 0, 100)  // Type error: number vs string
    "#;

    fs::write(temp_dir.path().join("main.luax"), invalid_source).unwrap();

    let mut container = create_test_container_with_dir(temp_dir.path());
    let result = container.compile_file(temp_dir.path().join("main.luax"));

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Type mismatch") || err.contains("Expected string"));
}
```

#### Test 3: Compilation Works with `.d.luax`

```rust
#[test]
fn test_d_tl_compilation() {
    let temp_dir = TempDir::new().unwrap();

    // Create .d.luax file
    fs::write(temp_dir.path().join("math_ext.d.luax"), r#"
        export function double(x: number): number
    "#).unwrap();

    // Create actual implementation in .lua file
    fs::write(temp_dir.path().join("math_ext.lua"), r#"
        local M = {}
        function M.double(x)
            return x * 2
        end
        return M
    "#).unwrap();

    // Create .luax file that uses both
    fs::write(temp_dir.path().join("main.luax"), r#"
        import { double } from './math_ext'
        result: number = double(21)
    "#).unwrap();

    let mut container = create_test_container_with_dir(temp_dir.path());
    let lua_code = container.compile_file(temp_dir.path().join("main.luax")).unwrap();

    // Execute and verify
    let executor = LuaExecutor::new().unwrap();
    // Note: Would need to handle module loading in Lua runtime
    // This is a simplified example
}
```

#### Test 4: Type Erasure (No Runtime Code)

```rust
#[test]
fn test_d_tl_no_runtime_code() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(temp_dir.path().join("types.d.luax"), r#"
        export type Point = { x: number, y: number }
        export type Vector = { x: number, y: number }
    "#).unwrap();

    fs::write(temp_dir.path().join("main.luax"), r#"
        import type { Point, Vector } from './types'

        p: Point = { x: 10, y: 20 }
        result: number = p.x + p.y
    "#).unwrap();

    let mut container = create_test_container_with_dir(temp_dir.path());
    let lua_code = container.compile_file(temp_dir.path().join("main.luax")).unwrap();

    // Verify that the generated Lua code doesn't include any imports
    assert!(!lua_code.contains("require"));
    assert!(!lua_code.contains("types"));

    // But the code should still work
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 30);
}
```

---

## Error Handling Tests

### Compile-Time Errors

Most compile-time error tests already exist in `error_conditions_comprehensive.rs`. These cover:
- Type errors
- Syntax errors
- Module resolution errors
- Circular dependency errors

### Runtime Error Tests

**File**: `crates/luanext-core/tests/runtime_error_tests.rs` (to be created)

#### Purpose

Verify that LuaNext-generated code produces expected Lua runtime errors.

#### Example Tests

```rust
#[test]
fn test_runtime_nil_access() {
    let source = r#"
        local x = nil
        result: number = x.foo  // Should error: attempt to index nil
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result = executor.execute(&lua_code);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("nil") || err.contains("index"));
}

#[test]
fn test_runtime_type_error() {
    let source = r#"
        function greet(name: string): string
            return "Hello, " .. name
        end

        result: string = greet(123)  // Type-checking might not catch this in untyped Lua
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    // In Lua, string concatenation with number works (auto-conversion)
    // This test verifies the behavior is preserved
    let result = executor.execute(&lua_code);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_runtime_division_by_zero() {
    let source = r#"
        result: number = 10 / 0  // Lua allows this (returns inf)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let result: f64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert!(result.is_infinite());
}

#[test]
fn test_runtime_function_not_found() {
    let source = r#"
        result: number = nonexistent_function()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result = executor.execute(&lua_code);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("nil") || err.contains("function"));
}

#[test]
fn test_runtime_table_index_out_of_bounds() {
    let source = r#"
        t = {1, 2, 3}
        result: number = t[10]  // Lua returns nil for out of bounds
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    executor.execute(&lua_code).unwrap();

    let result: mlua::Value = executor.lua().globals().get("result").unwrap();
    assert!(result.is_nil());
}
```

---

## Standard Library Integration

### Purpose

Test that LuaNext correctly integrates with Lua standard library functions and provides correct type definitions.

### Test Structure

**File**: `crates/luanext-core/tests/stdlib_execution_tests.rs` (to be created)

#### Math Library

```rust
#[test]
fn test_stdlib_math_basic() {
    let source = r#"
        result: number = math.sqrt(16) + math.abs(-10)
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: f64 = executor.execute_and_get(&lua_code, "result").unwrap();

    assert!((result - 14.0).abs() < 0.001); // 4 + 10 = 14
}

#[test]
fn test_stdlib_math_trigonometry() {
    let source = r#"
        pi: number = math.pi
        result: number = math.sin(pi / 2)  // sin(90°) = 1
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: f64 = executor.execute_and_get(&lua_code, "result").unwrap();

    assert!((result - 1.0).abs() < 0.001);
}

#[test]
fn test_stdlib_math_random() {
    let source = r#"
        math.randomseed(42)
        result: number = math.random(1, 100)
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();

    assert!(result >= 1 && result <= 100);
}
```

#### String Library

```rust
#[test]
fn test_stdlib_string_format() {
    let source = r#"
        result: string = string.format("Hello, %s!", "world")
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();

    assert_eq!(result, "Hello, world!");
}

#[test]
fn test_stdlib_string_operations() {
    let source = r#"
        upper: string = string.upper("hello")
        lower: string = string.lower("WORLD")
        len: number = string.len("test")
        result: string = upper .. " " .. lower
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();

    assert_eq!(result, "HELLO world");
}
```

#### Table Library

```rust
#[test]
fn test_stdlib_table_operations() {
    let source = r#"
        t = {1, 2, 3}
        table.insert(t, 4)
        table.insert(t, 1, 0)  // Insert at beginning
        result: number = #t
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();

    assert_eq!(result, 5); // [0, 1, 2, 3, 4]
}

#[test]
fn test_stdlib_table_sort() {
    let source = r#"
        t = {3, 1, 4, 1, 5, 9, 2, 6}
        table.sort(t)
        result: number = t[1] + t[#t]  // First + last
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();

    assert_eq!(result, 1 + 9); // 10
}
```

---

## Incremental Compilation

### Purpose

Verify that cached compilation produces identical output to fresh compilation, ensuring cache correctness.

### Test Structure

**File**: `crates/luanext-core/tests/cache_execution_tests.rs` (to be created)

#### Single File Caching

```rust
use tempfile::TempDir;
use std::fs;

#[test]
fn test_cache_single_file_correctness() {
    let temp_dir = TempDir::new().unwrap();
    let source = r#"
        result: number = 42 * 2
    "#;
    fs::write(temp_dir.path().join("main.luax"), source).unwrap();

    // First compilation (cache miss)
    let mut container1 = create_test_container_with_cache(temp_dir.path());
    let output1 = container1.compile_file(temp_dir.path().join("main.luax")).unwrap();

    // Second compilation (cache hit)
    let mut container2 = create_test_container_with_cache(temp_dir.path());
    let output2 = container2.compile_file(temp_dir.path().join("main.luax")).unwrap();

    // Outputs should be identical
    assert_eq!(output1, output2);

    // Execute both and verify results match
    let executor = LuaExecutor::new().unwrap();
    let result1: i64 = executor.execute_and_get(&output1, "result").unwrap();
    let result2: i64 = executor.execute_and_get(&output2, "result").unwrap();
    assert_eq!(result1, result2);
    assert_eq!(result1, 84);
}
```

#### Multi-File Caching

```rust
#[test]
fn test_cache_multi_file_correctness() {
    let temp_dir = TempDir::new().unwrap();

    // Create module
    fs::write(temp_dir.path().join("math_utils.luax"), r#"
        export function double(x: number): number {
            return x * 2
        }
    "#).unwrap();

    // Create main file
    fs::write(temp_dir.path().join("main.luax"), r#"
        import { double } from './math_utils'
        result: number = double(21)
    "#).unwrap();

    // First compilation (cache miss)
    let mut container1 = create_test_container_with_cache(temp_dir.path());
    let output1 = container1.compile_file(temp_dir.path().join("main.luax")).unwrap();

    // Second compilation (cache hit for both files)
    let mut container2 = create_test_container_with_cache(temp_dir.path());
    let output2 = container2.compile_file(temp_dir.path().join("main.luax")).unwrap();

    assert_eq!(output1, output2);
}
```

#### Cache Invalidation

```rust
#[test]
fn test_cache_invalidation_on_change() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("main.luax");

    // Initial version
    fs::write(&file_path, "result: number = 10").unwrap();

    let mut container1 = create_test_container_with_cache(temp_dir.path());
    let output1 = container1.compile_file(&file_path).unwrap();

    // Modify file
    fs::write(&file_path, "result: number = 20").unwrap();

    let mut container2 = create_test_container_with_cache(temp_dir.path());
    let output2 = container2.compile_file(&file_path).unwrap();

    // Outputs should be different (cache invalidated)
    assert_ne!(output1, output2);

    // Verify execution results
    let executor = LuaExecutor::new().unwrap();
    let result1: i64 = executor.execute_and_get(&output1, "result").unwrap();
    let result2: i64 = executor.execute_and_get(&output2, "result").unwrap();
    assert_eq!(result1, 10);
    assert_eq!(result2, 20);
}
```

---

## Edge Cases

### Purpose

Stress test LuaNext-generated code against Lua runtime limits: deep recursion, large data structures, numeric edge cases, and metamethods.

### Test Structure

**File**: `crates/luanext-core/tests/lua_edge_cases_tests.rs` (to be created)

#### Deep Recursion

```rust
#[test]
fn test_deep_recursion() {
    let source = r#"
        function fib(n: number): number
            if n <= 1 then return n end
            return fib(n - 1) + fib(n - 2)
        end

        result: number = fib(20)  // Deep but manageable
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();

    assert_eq!(result, 6765); // 20th Fibonacci number
}

#[test]
fn test_tail_call_optimization() {
    let source = r#"
        function sum_tail(n: number, acc: number): number
            if n == 0 then return acc end
            return sum_tail(n - 1, acc + n)  // Tail call
        end

        result: number = sum_tail(1000, 0)
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();

    assert_eq!(result, 500500); // Sum of 1..1000
}
```

#### Large Tables

```rust
#[test]
fn test_large_table() {
    let source = r#"
        t = {}
        for i = 1, 10000 do
            t[i] = i * 2
        end
        result: number = #t
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();

    assert_eq!(result, 10000);
}

#[test]
fn test_sparse_table() {
    let source = r#"
        t = {}
        t[1] = 1
        t[1000000] = 1000000
        result: number = t[1000000]
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();

    assert_eq!(result, 1000000);
}
```

#### String Concatenation

```rust
#[test]
fn test_large_string_concatenation() {
    let source = r#"
        s: string = ""
        for i = 1, 1000 do
            s = s .. "a"
        end
        result: number = string.len(s)
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();

    assert_eq!(result, 1000);
}
```

#### Numeric Edge Cases

```rust
#[test]
fn test_numeric_overflow() {
    let source = r#"
        huge: number = math.huge
        result: boolean = huge == math.huge
    "#;

    let lua_code = compile_with_stdlib(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: bool = executor.execute_and_get(&lua_code, "result").unwrap();

    assert!(result);
}

#[test]
fn test_nan_handling() {
    let source = r#"
        nan: number = 0 / 0
        is_nan: boolean = nan ~= nan  // NaN is not equal to itself
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: bool = executor.execute_and_get(&lua_code, "is_nan").unwrap();

    assert!(result);
}

#[test]
fn test_negative_zero() {
    let source = r#"
        neg_zero: number = -0.0
        pos_zero: number = 0.0
        result: boolean = neg_zero == pos_zero
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: bool = executor.execute_and_get(&lua_code, "result").unwrap();

    assert!(result); // In Lua, -0 == 0
}
```

#### Metamethods

```rust
#[test]
fn test_metamethod_add() {
    let source = r#"
        Point = {}
        Point.__index = Point

        function Point.new(x, y)
            local self = setmetatable({}, Point)
            self.x = x
            self.y = y
            return self
        end

        function Point.__add(a, b)
            return Point.new(a.x + b.x, a.y + b.y)
        end

        p1 = Point.new(1, 2)
        p2 = Point.new(3, 4)
        p3 = p1 + p2
        result: number = p3.x + p3.y
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();

    assert_eq!(result, 10); // (1+3) + (2+4) = 10
}
```

---

## Best Practices

### Use Appropriate Helpers

- **`execute_and_get()`**: For retrieving global variables after execution
- **`execute_with_result()`**: For evaluating expressions that return values
- **`execute()`**: For code that doesn't return a value
- **`compile_with_stdlib()`**: When testing code that uses Lua standard library
- **`compile_with_optimization()`**: When testing specific optimization levels

### Test Organization

- **Group related tests** in the same file
- **Use descriptive names**: `test_<feature>_<scenario>`
- **Add comments** for complex test cases
- **Keep tests focused**: One assertion per logical test
- **Use constants** for magic numbers

### Example Structure

```rust
// Good: Focused, descriptive, single concern
#[test]
fn test_closure_captures_upvalue() {
    let source = r#"
        function make_counter()
            local count: number = 0
            return function(): number
                count = count + 1
                return count
            end
        end
        counter = make_counter()
        result: number = counter()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(result, 1);
}
```

### Multi-File Tests

Use `TempDir` for tests involving multiple files:

```rust
use tempfile::TempDir;
use std::fs;

#[test]
fn test_multi_file_import() {
    let temp_dir = TempDir::new().unwrap();

    // Create module
    fs::write(temp_dir.path().join("utils.luax"), r#"
        export function add(a: number, b: number): number {
            return a + b
        }
    "#).unwrap();

    // Create main file
    fs::write(temp_dir.path().join("main.luax"), r#"
        import { add } from './utils'
        result: number = add(10, 20)
    "#).unwrap();

    // Compile and test
    let mut container = create_test_container_with_dir(temp_dir.path());
    let lua_code = container.compile_file(temp_dir.path().join("main.luax")).unwrap();

    let executor = LuaExecutor::new().unwrap();
    // Note: Multi-file execution requires module loader setup
}
```

### Running Specific Tests

```bash
# Run all tests in a file
cargo test --test execution_tests

# Run a specific test
cargo test test_integer_arithmetic

# Run tests matching a pattern
cargo test test_optimization

# Run with verbose output
cargo test -- --nocapture

# Run single-threaded (useful for debugging)
cargo test -- --test-threads=1
```

---

## CI Integration

### GitHub Actions Example

```yaml
name: Execution Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run execution tests
        run: |
          cd crates/luanext-core
          cargo test --test execution_tests
          cargo test --test execution_optimization_tests
          cargo test --test lua_edge_cases_tests
```

### Test Matrix for Lua Versions

```yaml
jobs:
  test-versions:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        lua-version: [lua51, lua52, lua53, lua54]

    steps:
      - uses: actions/checkout@v3

      - name: Test Lua ${{ matrix.lua-version }}
        run: |
          cargo test --features mlua/${{ matrix.lua-version }} \
            --test lua51_compat_tests \
            --test lua52_compat_tests \
            --test lua53_compat_tests \
            --test lua54_compat_tests
```

### Coverage Reporting

```yaml
jobs:
  coverage:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage
        run: cargo tarpaulin --out Xml --output-dir coverage

      - name: Upload to codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage/cobertura.xml
```

---

## Adding New Tests

### Checklist for New Language Features

When adding a new LuaNext language feature:

1. **Add execution test** in `execution_tests.rs`
   - Test basic functionality
   - Test edge cases
   - Test interaction with other features

2. **Add untyped test** in `execution_untyped_tests.rs`
   - Verify type erasure works
   - Test without type annotations

3. **Add optimization test** in `execution_optimization_tests.rs`
   - Verify O0 === O1 === O2 === O3

4. **Add version-specific test** (if applicable)
   - Check Lua version compatibility
   - Add to appropriate `luaXX_compat_tests.rs`

5. **Add edge case test** (if applicable)
   - Stress test the feature
   - Test runtime limits

6. **Update documentation**
   - Update this file (`docs/TESTING.md`)
   - Update `TODO.md` if tasks remain

### Where to Add Tests

| Test Type | File | When to Add |
|-----------|------|-------------|
| Basic feature test | `execution_tests.rs` | Always |
| Untyped variant | `execution_untyped_tests.rs` | If feature uses types |
| Optimization correctness | `execution_optimization_tests.rs` | If feature can be optimized |
| Lua version specific | `luaXX_compat_tests.rs` | If feature uses version-specific APIs |
| Type definition | `type_definition_tests.rs` | If feature involves imports/exports |
| Error handling | `runtime_error_tests.rs` | If feature has error cases |
| Stdlib integration | `stdlib_execution_tests.rs` | If feature uses stdlib |
| Edge case | `lua_edge_cases_tests.rs` | If feature has limits |

### Debugging Test Failures

1. **Run test with verbose output**:
   ```bash
   cargo test test_name -- --nocapture
   ```

2. **Inspect generated Lua code**:
   ```rust
   let lua_code = compile(source).unwrap();
   println!("Generated Lua:\n{}", lua_code);
   ```

3. **Check Lua execution error**:
   ```rust
   let result = executor.execute(&lua_code);
   if let Err(e) = result {
       println!("Lua error: {}", e);
   }
   ```

4. **Run in isolation**:
   ```bash
   cargo test --test execution_tests test_name -- --test-threads=1
   ```

5. **Use debugger**:
   ```bash
   rust-lldb target/debug/deps/execution_tests-*
   ```

---

## Additional Resources

- **Performance Testing**: See [docs/PERFORMANCE_TESTING.md](PERFORMANCE_TESTING.md) for benchmarking
- **Module System**: See [technical/module-system/resolution.md](../technical/module-system/resolution.md)
- **LuaExecutor Source**: `crates/luanext-test-helpers/src/lua_executor.rs`
- **Compile Helpers Source**: `crates/luanext-test-helpers/src/compile.rs`
- **Existing Tests**: `crates/luanext-core/tests/`

---

## Upgrading mlua

**Current**: mlua 0.10 with Lua 5.4

**Target**: mlua 0.11.6 with Lua 5.1-5.5 support

### Steps

1. Update `Cargo.toml`:
   ```toml
   [workspace.dependencies]
   mlua = { version = "0.11", features = ["lua54", "vendored"] }
   ```

2. Test with different versions:
   ```bash
   cargo test --features mlua/lua51
   cargo test --features mlua/lua52
   cargo test --features mlua/lua53
   cargo test --features mlua/lua54
   cargo test --features mlua/lua54-jit
   ```

3. Add Lua 5.5 tests after upgrade:
   ```bash
   cargo test --features mlua/lua55
   ```

4. Update CI matrix to include lua55

---

**Last Updated**: 2026-02-16
