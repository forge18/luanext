# LuaNext TODO

## High Priority

### ✅ Typed Global Variables Feature (COMPLETED 2026-02-16)

Enable typed global variable declarations without `local` keyword for cleaner test syntax and general use.

**Syntax:** Three ways to declare globals:

- `global x: number = 42` - explicit keyword with type
- `global x = 42` - explicit keyword with inference
- `x: number = 42` - implicit global (type annotation required)

**Implementation:**

- [x] **Parser Changes** - Completed with lookahead logic
  - Added `Global` variant to `VariableKind` enum in `ast/statement.rs:77-81`
  - Implemented `is_implicit_global_declaration()` with 2-token lookahead
  - Distinguishes `x: number = 42` (declaration) from `x :: T` (type assertion) from `x = 42` (assignment)
  - Parser helper methods in separate impl block

- [x] **Type Checker Changes** - Updated match statement
  - `type_checker.rs:474` - Global maps to `SymbolKind::Variable`

- [x] **Codegen Changes** - Conditional local keyword emission
  - `statements.rs:67-71` - Omits `local` for `VariableKind::Global`
  - Also updated array/object destructuring cases
  - Output: `global x = 42` → `x = 42`, `local x = 42` → `local x = 42`

- [x] **Scope Hoisting** - Globals not hoisted
  - `scope_hoisting.rs:363` - Returns false for Global kind

- [x] **LSP Support** - Updated symbol handling
  - `symbols.rs:63` - Handles Global variant

- [x] **Testing** - Comprehensive test suite
  - Created `global_variable_tests.rs` with 16 parser tests
  - All 890 existing tests pass, full codebase builds

- [ ] **Documentation** - User-facing docs needed
  - Language guide with global syntax examples
  - Difference between local/const/global declarations

## Low Priority

### Testing/Benchmarking Lua

#### Execution Testing Infrastructure

- [x] Add `mlua` crate with Lua 5.4 support (vendored feature for no system deps)
  - Note: Lua 5.5 not yet released; mlua supports up to 5.4
- [x] Create `LuaExecutor` helper in `crates/luanext-test-helpers/src/lua_executor.rs`
  - Methods: `new()`, `execute()`, `execute_and_get()`, `execute_with_result()`, `execute_ok()`, `lua()`
  - `LuaValueExt` trait for converting mlua::Value to Rust types
  - All 10 unit tests passing
- [x] **Comprehensive Execution Testing** - PHASE 1 COMPLETE (42/42 passing)
  - All 42 core execution tests passing across 9 categories
  - Fixed codegen bugs: constructor default params, match guard pre-binding
  - Fixed test syntax: `when` guards, implicit `self`, `new` keyword, `::` methods, destructuring
  - Target: 187+ tests covering all LuaNext features (Phase 2+)
  - See "Comprehensive Execution Test Expansion Plan" section below for detailed breakdown

#### Performance Benchmarking Infrastructure

- [ ] Add `criterion` crate for statistical benchmarking
- [ ] Create separate benchmark files for each optimization pass:
  - `benches/constant_folding_bench.rs` (3 benchmarks)
  - `benches/function_inlining_bench.rs` (2 benchmarks)
  - `benches/dead_code_elim_bench.rs` (2 benchmarks)
  - `benches/loop_optimizations_bench.rs` (2 benchmarks)
- [ ] Set up CI benchmark tracking in `.github/workflows/benchmarks.yml`
  - Compare PRs against main branch baseline
  - Fail on performance regressions >10%

#### Test Coverage & Documentation

- [x] Create `docs/TESTING.md` - comprehensive testing guide (completed 2026-02-16)
  - 950+ lines covering all 8 test areas
  - Includes examples, patterns, best practices, CI integration
- [ ] Create `docs/BENCHMARKING.md` - benchmarking guide
- [ ] Upgrade mlua to 0.11.6 - Support Lua 5.5
  - Current: mlua 0.10 with Lua 5.4
  - Target: mlua 0.11.6 with Lua 5.1-5.5 support
- [ ] Document version-specific quirks in `docs/LUA_VERSION_COMPATIBILITY.md`
  - Lua 5.1: no bitwise ops, no integers, module() function
  - Lua 5.2: bit32 library, _ENV, goto statement
  - Lua 5.3: native bitwise ops, integers, utf8 library
  - Lua 5.4: to-be-closed vars, const vars, generational GC
  - Lua 5.5: global declarations, named vararg, compact arrays, incremental GC (released Dec 2025)

#### Comprehensive Execution Test Expansion Plan (187+ Total Tests)

**Phase 1: Fix Failing Tests - COMPLETE (42/42 passing)**

- [x] Stdlib imports (2 tests)
- [x] Codegen fixes: string interpolation, default params, multi-assignment, array indexing, table types
- [x] Destructuring (4 tests): semicolons for `[` ambiguity, bracket arrays, VariableKind parameter, default values
- [x] Classes (5 tests): `new` keyword, `::` method calls, implicit `self`, explicit constructors for inheritance
- [x] Match (3 tests): implicit globals for executor visibility, `when` keyword for guards
- [x] Constructor default params codegen bug fixed (`classes.rs`)
- [x] Match guard pre-binding codegen bug fixed (`expressions.rs`)

**Phase 2: High Priority (~70 tests, 8 files)** - arrow functions, optional chaining, spread/rest, error handling, interfaces, enums, type aliases, decorators

**Phase 3: Medium Priority (~40 tests, 4 files)** - operator overloading, getters/setters, advanced classes, advanced control flow

**Phase 4: Lower Priority (~32 tests, 3 files)** - advanced operators, advanced patterns, module system

#### Original Execution Test Files to Create (Lower Priority)

- [ ] `execution_untyped_tests.rs` - Language features without type annotations
  - Mirror of execution_tests.rs but validates type erasure
- [ ] `execution_optimization_tests.rs` - O0/O1/O2/O3 correctness tests (16 tests)
  - Verify semantic equivalence: constant folding, DCE, inlining, loops, LTO
  - Basic cases + complex cases (closures, classes, recursion)
- [ ] `lua51_compat_tests.rs` - Lua 5.1 compatibility tests
  - No bitwise ops, all numbers are floats, module() function
- [ ] `lua52_compat_tests.rs` - Lua 5.2 compatibility tests
  - bit32 library, _ENV, goto statement
- [ ] `lua53_compat_tests.rs` - Lua 5.3 compatibility tests
  - Native bitwise ops (&, |, ~, <<, >>), integers, utf8 library, floor division
- [ ] `lua54_compat_tests.rs` - Lua 5.4 compatibility tests
  - To-be-closed variables (<close>), const vars, generational GC
- [ ] `lua55_compat_tests.rs` - Lua 5.5 compatibility tests
  - Global declarations, named vararg (...name), compact arrays
- [ ] `type_definition_tests.rs` - .d.tl file support tests
  - Type checker respects .d.tl, type errors caught, compilation works, type erasure
- [ ] `runtime_error_tests.rs` - Runtime error behavior tests
  - Nil access, division by zero, function not found, type errors
- [ ] `stdlib_execution_tests.rs` - Standard library integration tests
  - Math, string, table libraries with correct type definitions
- [ ] `cache_execution_tests.rs` - Incremental compilation correctness
  - Verify cached === fresh compilation, cache invalidation
- [ ] `lua_edge_cases_tests.rs` - Lua runtime edge cases
  - Deep recursion, large tables, string concatenation, numeric edge cases (NaN, infinity), metamethods
