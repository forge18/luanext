# LuaNext TODO

## Low Priority

### Optimizer O2/O3 Passes

#### Missing O2 Passes (Moderate Optimizations)

- [x] Jump threading - optimize conditional branches with known values (10 tests)
- [x] Common subexpression elimination (CSE) - eliminate duplicate computations (12 tests)
- [x] Copy propagation - replace variable uses with their values (10 tests)
- [x] Peephole optimization - small local code improvements (8 tests)
- [x] Sparse conditional constant propagation (SCCP) - combine constant folding with dead code elimination (10 tests)

#### O3 Passes (Aggressive Optimizations)

**Completed (3/6):**

- [x] **Loop unrolling** - duplicate loop bodies for small iteration counts
  - Implementation: `crates/luanext-core/src/optimizer/passes/loop_unrolling.rs` (520 lines)
  - Tests: `crates/luanext-core/tests/loop_unrolling_tests.rs` (15 tests, all passing)
  - Registered at O3 level, safely unrolls numeric for-loops with ≤4 constant iterations
  - Includes safety checks: no break/continue/return, numeric loops only
- [x] **Function cloning for specialization** - duplicate functions for different call contexts
  - Implementation: `crates/luanext-core/src/optimizer/passes/function_cloning.rs`
  - Tests: `crates/luanext-core/tests/function_cloning_tests.rs` (12 tests, all passing)
  - Clones small functions (≤8 statements) when called with constant arguments
  - Max 4 clones per function, deduplicates identical specializations
  - Substitutes constant args into cloned body, removes from parameter list
- [x] **Interprocedural constant propagation** - propagate constants across function boundaries
  - Implementation: `crates/luanext-core/src/optimizer/passes/interprocedural_const_prop.rs`
  - Tests: `crates/luanext-core/tests/interprocedural_const_prop_tests.rs` (15 tests, all passing)
  - Analyzes all call sites; when ALL callers pass the same constant for a parameter,
    substitutes it into the function body and removes the parameter
  - Fixed-point iteration (max 3 rounds) for propagation chains
  - Skips varargs, spread arguments, generic functions, destructuring parameters

**Remaining Future Work (1/6):**

- [ ] **Scalar replacement of aggregates** - replace table accesses with local variables
  - Would require: Robust escape analysis, table access pattern detection
  - Complexity: Very High, needs comprehensive alias/escape analysis

#### Cross-Module Optimizations

- [ ] Link-Time Optimization (LTO)
  - Cross-module optimizations using cached type information
  - Already have infrastructure via `CacheManager` and `ModuleRegistry`

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
