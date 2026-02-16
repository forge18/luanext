# LuaNext TODO

## High Priority

### Typed Global Variables Feature

Enable typed global variable declarations without `local` keyword for cleaner test syntax and general use.

**Syntax:** `x: number = 10` (creates global variable with type annotation)

**Current workaround:** Using return table pattern in execution tests

- [ ] **Parser Changes** (~100 lines, moderate complexity)
  - Add `Global` variant to `VariableKind` enum in `crates/luanext-parser/src/ast/statement.rs`
  - Implement two-token lookahead in `parse_statement()` to detect `Identifier Colon` pattern
  - Add checkpoint/backtrack logic (similar to `try_parse_arrow_function()`)
  - Disambiguate: `x: number = 10` (global) vs `x : (...)` (type assertion) vs `x = 10` (assignment)

- [ ] **Type Checker Changes** (~10 lines, trivial)
  - Update match statements in `check_variable_declaration()` to handle `VariableKind::Global`
  - No scope handling changes needed (globals already work)

- [ ] **Codegen Changes** (~15 lines, simple)
  - Conditional `local` keyword emission: only emit when `kind != VariableKind::Global`
  - Update `generate_variable_declaration()` in `crates/luanext-core/src/codegen/statements.rs`

- [ ] **Testing** (~200 lines)
  - Parser tests for typed global syntax
  - Type checker validation tests
  - Codegen output verification (ensure no `local` prefix for globals)
  - LSP feature tests (completion, hover, rename)

- [ ] **Documentation**
  - Update language guide with typed global syntax
  - Add examples showing difference between local and global declarations

**Estimated effort:** 3-5 days

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
