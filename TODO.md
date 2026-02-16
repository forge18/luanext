# LuaNext TODO

## Low Priority

### Testing/Benchmarking Lua

#### Execution Testing Infrastructure

- [ ] Add `mlua` crate with Lua 5.1-5.5 support (vendored feature for no system deps)
- [ ] Create `LuaExecutor` helper in `crates/luanext-core/tests/common/lua_executor.rs`
  - Constructors for each Lua version (5.1, 5.2, 5.3, 5.4, 5.5)
  - Methods: `execute()`, `execute_and_get()`, `execute_ok()`
  - `LuaValueExt` trait for converting mlua::Value to Rust types
- [ ] Write basic execution tests in `crates/luanext-core/tests/execution_tests.rs`
  - Compile TypedLua → Lua → execute → assert results
  - Test arithmetic, functions, strings, tables, control flow
- [ ] Create cross-version tests in `crates/luanext-core/tests/cross_version_execution_tests.rs`
  - Test same code across Lua 5.1, 5.2, 5.3, 5.4, 5.5
  - Verify bitwise ops, string interpolation, etc. work on all versions
- [ ] Add module execution tests in `crates/luanext-core/tests/module_execution_tests.rs`
  - Test multi-file compilation with imports/exports
  - Verify type-only imports are erased (no runtime overhead)
  - Test re-exports execute correctly

#### Performance Benchmarking Infrastructure

- [ ] Add `criterion` crate for statistical benchmarking
- [ ] Create main benchmark suite in `crates/luanext-core/benches/optimization_benchmarks.rs`
  - Benchmark constant folding (unoptimized vs optimized)
  - Benchmark function inlining (call overhead reduction)
  - Benchmark dead code elimination (unused code removal)
- [ ] Create separate benchmark files for each optimization pass:
  - `benches/constant_folding_bench.rs`
  - `benches/function_inlining_bench.rs`
  - `benches/dead_code_elim_bench.rs`
  - `benches/loop_optimizations_bench.rs`
- [ ] Set up CI benchmark tracking in `.github/workflows/benchmarks.yml`
  - Compare PRs against main branch baseline
  - Fail on performance regressions >10%

#### Test Coverage Strategy

- [ ] High-priority execution tests:
  - Constant folding correctness
  - Dead code elimination correctness
  - Function inlining correctness
  - Type erasure semantics preservation
  - Module system runtime behavior
- [ ] Medium-priority execution tests:
  - Control flow (match, loops, break/continue)
  - Classes (constructors, methods, inheritance)
  - String interpolation
  - Destructuring
- [ ] Document version-specific quirks (5.1 no bitwise ops, 5.3 integers, etc.)
