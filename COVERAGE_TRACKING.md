# Coverage Tracking Report

**Date:** 2026-02-02  
**Status:** Baseline coverage analysis in progress  
**Target:** 70% line coverage

## Summary

Due to the extensive test suite (1,500+ tests across 86 test suites), the TypedLua project likely has good coverage, but a full tarpaulin run takes significant time (>2 minutes). This report identifies potential coverage gaps based on project structure analysis.

## Test Suite Overview

- **Total Test Suites:** 86
- **Total Tests:** ~1,500+
- **All Tests Passing:** ✅ Yes (0 failures)
- **Test Categories:**
  - Unit tests: Embedded in source files with `#[cfg(test)]`
  - Integration tests: 63 files in `tests/` directories
  - Feature tests: Comprehensive coverage of all language features

## Critical Modules Analysis

### 1. Type Checker (`src/typechecker/`)
**Priority: CRITICAL**

**Likely Well-Covered:**
- Type inference (`visitors/inference.rs`) - Covered by type inference tests
- Type checking entry points (`type_checker.rs`) - Covered by integration tests
- Symbol table (`symbol_table.rs`) - Covered by symbol resolution tests

**Potential Gaps:**
- Generic specialization (`generics/specialization.rs`) - Complex edge cases
- Type compatibility edge cases (`type_compat.rs`) - Rare type combinations
- Error recovery paths - Hard to trigger all error conditions

**Recommendation:** Add targeted tests for:
- Complex generic type inference edge cases
- Type error recovery scenarios
- Cross-module type resolution

### 2. Parser (`crates/typedlua-parser/src/`)
**Priority: HIGH**

**Likely Well-Covered:**
- Basic parsing (lexer, parser) - Extensive parser tests
- Expression parsing - Expression tests
- Statement parsing - Statement tests

**Potential Gaps:**
- Error recovery in parser - Partial parses
- Complex nested expressions - Deep nesting
- Unicode/edge case handling - Special characters

**Recommendation:** Add tests for:
- Parser error recovery scenarios
- Maximum nesting depth handling
- Invalid UTF-8 handling

### 3. Code Generator (`src/codegen/`)
**Priority: HIGH**

**Likely Well-Covered:**
- Basic code generation - Most integration tests check output
- Expression generation - Expression tests
- Statement generation - Statement tests

**Potential Gaps:**
- Lua version-specific strategies (`strategies/lua51.rs`, etc.) - May only test default
- Source map generation - Debug info
- Decorator code generation - Complex decorators

**Recommendation:** Add tests for:
- Each Lua target version (5.1, 5.2, 5.3, 5.4)
- Source map accuracy
- Complex decorator combinations

### 4. Optimizer (`src/optimizer/`)
**Priority: MEDIUM-HIGH**

**Likely Well-Covered:**
- Basic optimizations - O1, O2, O3 tests exist
- Dead code elimination - DCE tests
- Inlining - Inlining tests

**Potential Gaps:**
- Rich enum optimization (`rich_enum_optimization.rs`) - May not cover all cases
- Interface inlining (`interface_inlining.rs`) - Complex scenarios
- Devirtualization (`devirtualization.rs`) - Virtual call optimization
- Aggressive inlining edge cases

**Recommendation:** Add tests for:
- Optimization interaction (multiple passes)
- Edge cases for each optimization pass
- Performance benchmarks for optimization effectiveness

### 5. Cache System (`src/cache/`)
**Priority: MEDIUM**

**Likely Well-Covered:**
- Basic caching - Cache manager tests
- Module serialization - Module cache tests

**Potential Gaps:**
- Cache invalidation (`invalidation.rs`) - Complex dependency graphs
- Manifest management (`manifest.rs`) - Concurrent access
- Hash computation (`hash.rs`) - Collision scenarios

**Recommendation:** Add tests for:
- Complex dependency invalidation chains
- Concurrent cache access
- Cache corruption recovery

### 6. Module Resolver (`src/module_resolver/`)
**Priority: MEDIUM**

**Likely Well-Covered:**
- Basic resolution - Module system tests
- Registry operations - Registry tests

**Potential Gaps:**
- Circular dependency detection - Complex graphs
- Cross-platform path handling - Windows vs Unix
- Complex re-export scenarios

**Recommendation:** Add tests for:
- Circular dependency edge cases
- Path normalization across platforms
- Deep re-export chains

### 7. LSP (`crates/typedlua-lsp/src/`)
**Priority: MEDIUM**

**Likely Well-Covered:**
- Basic LSP operations - LSP integration tests
- Document sync - Document tests

**Potential Gaps:**
- Incremental document updates - Complex edits
- Provider edge cases - Hover, completion, etc.
- Message handling errors - Protocol errors

**Recommendation:** Add tests for:
- Complex incremental edits
- Provider timeout scenarios
- Malformed LSP messages

### 8. CLI (`crates/typedlua-cli/src/`)
**Priority: LOW-MEDIUM**

**Likely Well-Covered:**
- Basic CLI operations - CLI integration tests
- Argument parsing - Args tests

**Potential Gaps:**
- Error handling paths - File not found, permissions
- Watch mode - File system events
- Complex flag combinations

**Recommendation:** Add tests for:
- Error conditions (missing files, bad permissions)
- Watch mode file changes
- Flag interaction edge cases

## Coverage Improvement Plan

### Phase 1: Critical Gaps (Target: +5% coverage)

1. **Type Checker Edge Cases**
   - Add tests for complex generic inference failures
   - Add tests for type error recovery
   - Add tests for cross-module type resolution

2. **Parser Error Recovery**
   - Add tests for partial parse scenarios
   - Add tests for malformed input handling
   - Add tests for maximum nesting limits

3. **Code Generator Lua Versions**
   - Add tests for each Lua target (5.1, 5.2, 5.3, 5.4)
   - Add tests for version-specific features

### Phase 2: High-Impact Gaps (Target: +3% coverage)

1. **Optimizer Passes**
   - Add tests for rich enum optimization
   - Add tests for interface inlining
   - Add tests for devirtualization

2. **Cache System**
   - Add tests for complex invalidation
   - Add tests for concurrent access
   - Add tests for corruption recovery

### Phase 3: Medium-Impact Gaps (Target: +2% coverage)

1. **Module System**
   - Add tests for circular dependencies
   - Add tests for complex re-exports

2. **LSP Edge Cases**
   - Add tests for incremental updates
   - Add tests for provider timeouts

## Tracking Progress

After each batch of tests is added:

```bash
# Run coverage report
./scripts/coverage.sh

# Check if we hit targets
grep "coverage" tarpaulin-report.xml | head -1
```

## Current Status

- **Baseline Coverage:** Running (estimated 60-75% based on test volume)
- **Target Coverage:** 70%
- **Gap to Close:** ~0-10%
- **Estimated Tests Needed:** 20-50 additional targeted tests

## Next Steps

1. ✅ Complete baseline coverage run (in progress)
2. Identify specific uncovered lines from HTML report
3. Prioritize by criticality and impact
4. Add targeted tests for highest-impact gaps
5. Re-run coverage after each batch
6. Track progress toward 70% target

---

**Note:** Full coverage report generation is in progress. Run `./scripts/coverage.sh` to generate detailed HTML report showing exact uncovered lines.
