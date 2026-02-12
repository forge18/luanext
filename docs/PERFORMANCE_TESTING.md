# LuaNext Performance Testing Guide

## Overview

This document describes the performance testing infrastructure for LuaNext's cross-file type support (Phase 5 of Enhanced Cross-File Type Support).

## Performance Targets

Based on TODO.md Phase 5 requirements:

1. **Large Project Compilation**: Projects with 100+ files should compile in <5 seconds (clean build)
2. **LSP Responsiveness**: Hover and completion requests should respond in <100ms with many cross-file references
3. **Incremental Compilation**: Cross-file changes should trigger minimal recompilation
4. **Re-export Chain Performance**: Deep re-export chains should not cause significant performance degradation

## Benchmark Suite

### 1. Cross-File Performance (`cross_file_performance.rs`)

**Location**: `crates/luanext-cli/benches/cross_file_performance.rs`

**Tests**:
- `large_project_compilation`: Measures compilation time for projects with 50-200 files
- `incremental_compilation`: Measures incremental rebuild time after single file change
- `reexport_chains`: Tests performance with re-export chains of depth 1-10
- `cross_file_type_resolution`: Measures type resolution with varying numbers of cross-file references

**Run**:
```bash
cd crates/luanext-cli
cargo bench --bench cross_file_performance
```

**Expected Results**:
- 100 files: <5 seconds clean build
- 200 files: <10 seconds clean build
- Single file change: <1 second incremental rebuild
- Re-export depth 10: <2x slowdown vs depth 1

### 2. Cache Performance (`cache_performance.rs`)

**Location**: `crates/luanext-cli/benches/cache_performance.rs`

**Tests**:
- `cache_hit_vs_miss`: Compares cached vs uncached compilation
- `incremental_after_edit`: Measures incremental compilation after file edits
- `cache_scaling`: Tests cache effectiveness with different project sizes
- `dependency_invalidation`: Tests selective invalidation (root vs leaf modules)
- `reexport_caching`: Measures re-export chain caching efficiency

**Run**:
```bash
cd crates/luanext-cli
cargo bench --bench cache_performance
```

**Expected Results**:
- Cache hit: 10-50x faster than cache miss
- Root dependency edit: Invalidates all dependents
- Leaf module edit: Only recompiles that module
- Re-export chains: Cached effectively (no repeated resolution)

### 3. LSP Responsiveness (`lsp_responsiveness.rs`)

**Location**: `crates/luanext-lsp/benches/lsp_responsiveness.rs`

**Tests**:
- `lsp_hover`: Hover performance with 10-100 cross-file references
- `lsp_completion`: Completion with varying numbers of imported symbols
- `lsp_parsing`: Document parsing performance
- `lsp_symbol_table`: Symbol table generation
- `lsp_reexport_resolution`: Re-export traversal in LSP
- `lsp_incremental_updates`: Document update performance

**Run**:
```bash
cd crates/luanext-lsp
cargo bench --bench lsp_responsiveness
```

**Expected Results**:
- Hover: <100ms even with 100 cross-file references
- Completion: <100ms with 100 imported symbols
- Parsing: <50ms for realistic module
- Symbol table: <30ms for typical module

## Running All Benchmarks

Use the provided script to run all benchmarks and generate a report:

```bash
./scripts/run_performance_tests.sh
```

This will:
1. Build all benchmarks in release mode
2. Run each benchmark suite sequentially
3. Generate a timestamped report in `target/performance-reports/`
4. Validate all performance targets
5. Create detailed HTML reports via Criterion

## Interpreting Results

### Criterion Output

Criterion provides detailed statistical analysis including:
- Mean execution time
- Standard deviation
- Outlier detection
- Performance regression warnings

### HTML Reports

After running benchmarks, view detailed reports at:
```
file://target/criterion/report/index.html
```

These include:
- Time series graphs
- Distribution plots
- Comparison with previous runs
- Performance trends

### Performance Regression Detection

Criterion automatically detects performance regressions:
- **Green**: Performance improved or stable
- **Yellow**: Minor performance change (within noise)
- **Red**: Significant performance regression (>5% slower)

If you see red warnings:
1. Check if the regression is real (re-run to eliminate noise)
2. Investigate code changes that may have caused it
3. Use `git bisect` to find the culprit commit
4. Profile with `cargo flamegraph` to find hotspots

## Profiling

For detailed performance analysis beyond benchmarks:

### 1. Flamegraph (CPU Profiling)

```bash
# Install flamegraph
cargo install flamegraph

# Profile compilation
cd crates/luanext-cli
cargo flamegraph --bench cross_file_performance

# Profile LSP
cd crates/luanext-lsp
cargo flamegraph --bench lsp_responsiveness
```

This generates `flamegraph.svg` showing where CPU time is spent.

### 2. Heap Profiling

```bash
# On macOS with Instruments
cargo instruments --bench cross_file_performance --template Allocations

# On Linux with heaptrack
heaptrack cargo bench --bench cross_file_performance
```

### 3. Perf (Linux only)

```bash
cargo build --release --benches
perf record ./target/release/deps/cross_file_performance-*
perf report
```

## Continuous Performance Monitoring

### CI Integration

The benchmark suite can be integrated into CI:

```yaml
# .github/workflows/performance.yml
name: Performance Benchmarks
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run benchmarks
        run: ./scripts/run_performance_tests.sh
      - name: Upload report
        uses: actions/upload-artifact@v3
        with:
          name: performance-report
          path: target/performance-reports/
```

### Performance Tracking

Track performance over time by:
1. Storing Criterion's baseline data in a separate branch
2. Comparing each CI run against the baseline
3. Alerting on regressions >10%

## Common Performance Issues

### Issue 1: Slow Compilation (>5s for 100 files)

**Possible Causes**:
- Inefficient type resolution algorithm
- Missing caching
- Redundant re-parsing

**Debug**:
```bash
LUANEXT_DEBUG=1 cargo run -- compile large_project/main.luax
```

**Fix**: Add caching, optimize hot paths identified via flamegraph

### Issue 2: LSP Lag (>100ms hover/completion)

**Possible Causes**:
- Re-parsing on every request
- Uncached symbol lookups
- Synchronous file I/O

**Debug**:
```bash
RUST_LOG=debug cargo run --bin luanext-lsp
```

**Fix**: Cache AST and symbol table, use async I/O

### Issue 3: Poor Incremental Compilation

**Possible Causes**:
- Over-invalidation (invalidating too many modules)
- Missing dependency tracking
- Cache key computation is too coarse

**Debug**:
- Check which modules are being recompiled
- Verify dependency graph is correct
- Add logging to cache invalidation logic

**Fix**: Fine-tune dependency tracking, implement selective invalidation

### Issue 4: Re-export Chain Slowdown

**Possible Causes**:
- Repeated resolution of same chains
- Missing memoization
- Circular reference detection is too slow

**Debug**:
```bash
# Add trace logging to re-export resolution
RUST_LOG=luanext_typechecker::phases::module_phase=trace cargo bench
```

**Fix**: Add re-export chain caching (already implemented in Phase 4.5)

## Benchmark Maintenance

### Adding New Benchmarks

1. Create benchmark file in `crates/*/benches/`
2. Add to `Cargo.toml`:
   ```toml
   [[bench]]
   name = "my_benchmark"
   harness = false
   ```
3. Use Criterion framework:
   ```rust
   use criterion::{criterion_group, criterion_main, Criterion};

   fn my_benchmark(c: &mut Criterion) {
       c.bench_function("test_name", |b| {
           b.iter(|| {
               // Code to benchmark
           })
       });
   }

   criterion_group!(benches, my_benchmark);
   criterion_main!(benches);
   ```
4. Add to `run_performance_tests.sh`
5. Document in this file

### Updating Performance Targets

When targets change (e.g., aiming for <50ms LSP response):
1. Update TODO.md
2. Update this document
3. Update validation logic in `run_performance_tests.sh`
4. Add regression tests if target is critical

## Performance Best Practices

1. **Measure before optimizing**: Profile first, optimize hot paths
2. **Benchmark variations**: Test with realistic workloads (size, complexity)
3. **Run multiple iterations**: Criterion handles this automatically
4. **Control for noise**: Close other applications, use consistent hardware
5. **Document baselines**: Store expected performance for comparison
6. **Test edge cases**: Empty files, huge files, deep nesting, circular deps
7. **Monitor memory**: Performance isn't just speed—watch heap usage too

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Flamegraph Guide](https://www.brendangregg.com/flamegraphs.html)
- LuaNext TODO.md - Phase 5: Integration & Testing
