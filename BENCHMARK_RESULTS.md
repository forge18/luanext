# Incremental Parsing Benchmark Results

## Summary

Incremental parsing is **2.5-3.0x faster** than full parsing for single-statement edits, scaling well across file sizes. Multi-statement edits show proportionally smaller gains as expected.

## Results (Round 3 — Phase 5 Polish)

### Single Character Edit

| File Size  | Full Parse | Incremental | Speedup  |
|------------|------------|-------------|----------|
| 100 lines  | 50.5 µs    | 17.0 µs     | **3.0x** |
| 500 lines  | 235.1 µs   | 84.8 µs     | **2.8x** |
| 1000 lines | 471.7 µs   | 172.9 µs    | **2.7x** |

### Line Deletion

| File Size  | Full Parse | Incremental | Speedup  |
|------------|------------|-------------|----------|
| 100 lines  | 48.9 µs    | 20.4 µs     | **2.4x** |
| 500 lines  | 236.5 µs   | 90.8 µs     | **2.6x** |
| 1000 lines | 479.6 µs   | 189.3 µs    | **2.5x** |

### Multi-line Paste (3 lines)

| File Size  | Full Parse | Incremental | Speedup  |
|------------|------------|-------------|----------|
| 100 lines  | 50.0 µs    | 18.3 µs     | **2.7x** |
| 500 lines  | 238.8 µs   | 87.0 µs     | **2.7x** |
| 1000 lines | 475.5 µs   | 177.9 µs    | **2.7x** |

### Append to End

| File Size  | Full Parse | Incremental | Speedup  |
|------------|------------|-------------|----------|
| 100 lines  | 49.6 µs    | 17.2 µs     | **2.9x** |
| 500 lines  | 234.5 µs   | 83.4 µs     | **2.8x** |
| 1000 lines | 471.3 µs   | 170.7 µs    | **2.8x** |

### Edit at Start

| File Size  | Full Parse | Incremental | Speedup  |
|------------|------------|-------------|----------|
| 100 lines  | 50.5 µs    | 19.2 µs     | **2.6x** |
| 500 lines  | 236.1 µs   | 91.7 µs     | **2.6x** |
| 1000 lines | 470.8 µs   | 186.9 µs    | **2.5x** |

### Multi-Statement Deletion (10% of file)

| File Size  | Full Parse | Incremental | Speedup  |
|------------|------------|-------------|----------|
| 100 lines  | 48.9 µs    | 43.3 µs     | **1.1x** |
| 500 lines  | 236.3 µs   | 156.4 µs    | **1.5x** |
| 1000 lines | 468.4 µs   | 306.6 µs    | **1.5x** |

### Format Document (all statements dirty)

| File Size  | Full Parse | Incremental | Speedup  |
|------------|------------|-------------|----------|
| 100 lines  | 49.8 µs    | 36.4 µs     | **1.4x** |
| 500 lines  | 239.4 µs   | 159.3 µs    | **1.5x** |
| 1000 lines | 469.7 µs   | 324.8 µs    | **1.4x** |

### Very Large File (10,000 lines)

| Scenario        | Time     |
|-----------------|----------|
| Full parse      | 5.20 ms  |
| Incremental     | 2.40 ms  |
| **Speedup**     | **2.2x** |

### Small File (5 lines)

| Scenario        | Time     |
|-----------------|----------|
| Full parse      | 4.71 µs  |
| Incremental     | 1.89 µs  |
| **Speedup**     | **2.5x** |

## Optimization History

### Round 3: Phase 5 Polish (current)

- Replaced `DefaultHasher` with `FxHasher` for ~2-4x faster source hashing
- Added `incremental-parsing` feature flag (default on) as safety valve
- Added debug logging (`LUANEXT_DEBUG_INCREMENTAL=1`)
- Added LSP fallback warning via `tracing::warn!`
- Replaced `Arc<Bump>` with `Rc<Bump>` (correctness: `Bump` is not `Sync`)
- Added comprehensive documentation (architecture.md, design doc, rustdoc examples)
- **Result**: 2.5-3.0x speedup maintained; new benchmark scenarios added

### Round 2: Partial Re-Lexing

- Instead of lexing the entire file, only dirty byte regions are lexed via `Lexer::tokenize_range()`
- Clean statements are reused directly with their cached tokens
- **Result**: 2.7-3.3x faster than full parse (44-47% improvement over Round 1)

### Round 1: Core Bug Fixes

- **BUG 1 (Critical)**: `is_valid()` used OLD byte ranges against NEW source - all statements after edit were invalidated
- **BUG 2**: O(n^2) clean statement lookups replaced with HashSet O(1)
- **BUG 4**: Linear token scan replaced with binary search (`partition_point`)
- **BUG 5**: Redundant hash validation removed (overlap check is sufficient)
- **BUG 6**: Linear token seek in `parse_statement_at_offset()` replaced with binary search
- **Panic fix**: Reused cached statements preserve original CachedStatement instead of rebuilding
- **Result**: 1.7-1.8x faster than full parse (was 2-6x slower before fixes)
