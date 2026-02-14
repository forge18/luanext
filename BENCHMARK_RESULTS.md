# Incremental Parsing Benchmark Results

## Summary

Incremental parsing is **2.7-3.3x faster** than full parsing across all edit types and file sizes.

## Results

### Single Character Edit

| File Size | Full Parse | Incremental | Speedup |
|-----------|-----------|-------------|---------|
| 100 lines | 49.2 µs | 15.2 µs | **3.2x** |
| 500 lines | 235.5 µs | 70.4 µs | **3.3x** |
| 1000 lines | 462.5 µs | 148.7 µs | **3.1x** |

### Line Deletion

| File Size | Full Parse | Incremental | Speedup |
|-----------|-----------|-------------|---------|
| 100 lines | 48.9 µs | 18.2 µs | **2.7x** |
| 500 lines | 236.3 µs | 78.9 µs | **3.0x** |
| 1000 lines | 466.8 µs | 160.8 µs | **2.9x** |

### Multi-line Paste (3 lines)

| File Size | Full Parse | Incremental | Speedup |
|-----------|-----------|-------------|---------|
| 100 lines | 48.4 µs | 14.9 µs | **3.3x** |
| 500 lines | 235.9 µs | 71.3 µs | **3.3x** |
| 1000 lines | 468.8 µs | 148.2 µs | **3.2x** |

## Optimization History

### Round 2: Partial Re-Lexing (current)
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
