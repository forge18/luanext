# LuaNext TODO

> Last updated: 2026-02-08
> Generated from comprehensive codebase review

---

## Priority 1: Critical Issues

### ~~🔴 Production Panic Calls~~ ✅ FIXED

**Audit result:** All listed `panic!()` calls are inside `#[cfg(test)]` modules (test assertions, not production code).

- [x] ~~Convert type system panics~~ — All panics are test-only assertions
- [x] Audit critical `.unwrap()`/`.expect()` calls in production code
- [x] Fix CRITICAL: Lexer `.chars().next().unwrap()` on user input (5 locations in `main.rs`)
- [x] Fix CRITICAL: `class_decl.primary_constructor.unwrap()` in `type_checker.rs:1384`
- [x] Fix HIGH: DI `.unwrap()` without message in `main.rs:1180`
- [x] Fix MEDIUM: Iterator `.unwrap()` after length checks (5 locations)
- **Status:** Critical unwraps fixed; DI `.expect()` calls with clear messages retained (Rust convention for programming errors)

---

## Priority 2: Incomplete Features

### 🟡 Devirtualization Optimization (Fully Stubbed)

- [ ] Re-implement devirtualization pass with arena-allocated AST
  - `crates/luanext-core/src/optimizer/devirtualization.rs:583-584`
- [ ] Connect existing ClassHierarchy analysis to optimization pass
- [ ] Remove or update reference to `devirtualization.rs.pre_arena` (line 5)
- [ ] Restore O3-level virtual method call optimization
- **Impact:** Major O3 optimization completely disabled

### 🟡 Rich Enum Optimization (Detection Only)

- [ ] Implement transformations for rich enums in optimization pass
  - `crates/luanext-core/src/optimizer/rich_enum_optimization.rs:50`
- [ ] Currently only detects, doesn't optimize
- [ ] Design and implement enum field access optimization
- [ ] Add enum constructor inlining
- **Impact:** O2-level optimization not working

### 🟡 Export Declaration Handling (Partial)

- [ ] Handle Function exports in type checker
  - `crates/luanext-typechecker/src/core/type_checker.rs:1065`
- [ ] Handle Class exports
- [ ] Handle Variable exports
- [ ] Handle Enum exports
- [ ] Currently only TypeAlias exports work
- **Impact:** Export type checking incomplete

### 🟡 Module Scope Hoisting (Disabled)

- [ ] Implement full module hoisting check
  - `crates/luanext-core/src/codegen/scope_hoisting.rs:931-936`
- [ ] `is_module_fully_hoistable()` always returns false
- [ ] Design safe criteria for module wrapper elimination
- [ ] Add tests for fully hoistable module optimization
- **Impact:** Missing optimization opportunity

### 🟡 Class Code Generation (Simplified Stub)

- [ ] Complete class codegen implementation
  - `crates/luanext-core/src/codegen/mod.rs:715-729`
- [ ] Generate methods properly
- [ ] Generate constructor
- [ ] Implement inheritance handling
- [ ] Currently only generates basic table structure
- **Impact:** Class features partially working

---

## Priority 3: Analysis-Only Optimization Passes

These passes perform analysis but don't apply transformations:

### 🟢 Tail Call Optimization

- [ ] Decide: Document as analysis-only OR implement transformations
  - `crates/luanext-core/src/optimizer/passes/tail_call_optimization.rs:26, 49`
- [ ] Lua runtime handles TCO automatically
- [ ] Consider if explicit optimization adds value
- **Impact:** Low (Lua handles this)

### 🟢 Table Preallocation

- [ ] Implement codegen integration for size hints
  - `crates/luanext-core/src/optimizer/passes/table_preallocation.rs:18, 47-48`
- [ ] Generate `table.create(size)` calls with collected size info
- [ ] Currently collects data but doesn't use it
- **Impact:** Medium (performance opportunity)

---

## Priority 4: LSP Feature Gaps

### 🟢 Import Navigation Incomplete

- [ ] Implement namespace import navigation
  - `crates/luanext-lsp/src/features/navigation/definition.rs:100-102`
- [ ] Handle type-only imports (line 108)
- [ ] Add property access resolution for namespace imports
- [ ] Currently returns None for both cases
- **Impact:** Go-to-definition limited for imports

### 🟢 Type Inference Gaps

- [ ] Report errors for unimplemented expression types instead of returning Unknown
  - `crates/luanext-typechecker/src/visitors/inference.rs:701-704`
- [ ] Identify which expression types are unhandled
- [ ] Implement missing type inference cases or document as unsupported
- **Impact:** Silent type safety degradation

### 🟢 Type Alias Resolution

- [ ] Implement type alias resolution to underlying types
  - `crates/luanext-typechecker/src/core/type_compat.rs:179`
- [ ] Currently noted as "Ideally we would resolve..."
- [ ] Add tests for type alias compatibility checking
- **Impact:** Type compatibility checking incomplete

---

## Priority 5: Code Quality & Polish

### 🔵 Source Map Quality

- [ ] Implement name mappings during source map merge
  - `crates/luanext-core/src/codegen/sourcemap.rs:174-175`
- [ ] Currently sets `name_index` to None
- [ ] Improve debugging experience with proper name mappings
- **Impact:** Degraded debugging experience

### 🔵 Debug Print Cleanup

- [ ] Remove or gate debug eprintln! statements
  - `crates/luanext-core/src/codegen/scope_hoisting.rs:77-81`
- [ ] Use feature flag or logging framework
- [ ] Clean up temporary debugging code
- **Impact:** Code cleanliness

### 🔵 Declaration Checking Improvements

- [ ] Replace placeholder object types for interfaces with proper interface types
  - `crates/luanext-typechecker/src/phases/declaration_checking_phase.rs:260`
- [ ] Ensure full interface semantics are captured
- **Impact:** Minor - current approach may miss edge cases

### 🔵 Performance Optimization

- [ ] Reduce clone overhead in symbol table lookup
  - `crates/luanext-lsp/src/features/navigation/hover.rs:57`
- [ ] Use references instead of cloning symbols
- [ ] Profile hot paths for unnecessary allocations
- **Impact:** LSP responsiveness

### 🔵 Code Maintainability

- [ ] Simplify generic specialization context lifetime management
  - `crates/luanext-core/src/optimizer/passes/generic_specialization.rs:725-726`
- [ ] Refactor temporary context workaround
- [ ] Document arena lifetime constraints
- **Impact:** Code clarity

### 🔵 Dead Code Cleanup

- [ ] Remove unused variable `_table_count`
  - `crates/luanext-core/src/optimizer/passes/table_preallocation.rs:41`
- [ ] Audit other prefixed-underscore variables
- [ ] Clean up analysis results that aren't leveraged
- **Impact:** Code cleanliness

---

## Statistics

**Total Tasks:** 47

**By Priority:**

- Priority 1 (Critical): ~~11 tasks~~ → 5 remaining (6 fixed)
- Priority 2 (Incomplete Features): 18 tasks
- Priority 3 (Analysis-Only): 5 tasks
- Priority 4 (LSP Gaps): 8 tasks
- Priority 5 (Code Quality): 5 tasks

**By Impact:**

- 🔴 Critical: ~~3 major issues~~ → 1 remaining (panics resolved)
- 🟡 High: 5 incomplete features
- 🟢 Medium: 6 feature gaps
- 🔵 Low: 6 polish items

---

## Notes

- This TODO was generated from a comprehensive codebase review on 2026-02-08
- Comment quality in the codebase is **excellent** - maintain current standards
- Test coverage appears comprehensive
- Architecture is well-documented with module-level docs
- Focus on Priority 1 items first for stability and correctness
