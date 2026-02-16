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
- [~] Write basic execution tests in `crates/luanext-core/tests/execution_tests.rs`
  - File created with all 45 test cases (9 categories)
  - **PAUSED:** Using return table pattern as workaround until typed globals implemented
  - Test categories: arithmetic, functions, control flow, tables, types, classes, string interpolation, destructuring, match
- [ ] Create cross-version tests in `crates/luanext-core/tests/cross_version_execution_tests.rs`
  - Test same code across Lua 5.1, 5.2, 5.3, 5.4
  - Verify bitwise ops, string interpolation, etc. work on all versions
- [ ] Add module execution tests in `crates/luanext-core/tests/module_execution_tests.rs`
  - Test multi-file compilation with imports/exports
  - Verify type-only imports are erased (no runtime overhead)
  - Test re-exports execute correctly

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

- [ ] Optimization correctness tests (16 tests across O0, O1, O2, O3)
  - Constant folding, DCE, inlining, loop optimizations, LTO
- [ ] Document version-specific quirks in `docs/LUA_VERSION_COMPATIBILITY.md`
  - Lua 5.1: no bitwise ops, no integers, module() function
  - Lua 5.2: bit32 library, _ENV, goto statement
  - Lua 5.3: native bitwise ops, integers, utf8 library
  - Lua 5.4: to-be-closed vars, const vars, generational GC
- [ ] Create `docs/TESTING.md` - comprehensive testing guide
- [ ] Create `docs/BENCHMARKING.md` - benchmarking guide
