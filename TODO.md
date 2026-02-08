# LuaNext TODO

> Last updated: 2026-02-08
> Generated from comprehensive codebase review

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
- Priority 2 (Incomplete Features): ~~18 tasks~~ → 0 remaining (ALL COMPLETED: devirtualization: 4, rich enum: 4, export handling: 5, module hoisting: 4, class codegen: 5)
- Priority 3 (Analysis-Only): ~~5 tasks~~ → 0 remaining (ALL COMPLETED: tail call optimization documented, table preallocation implemented)
- Priority 4 (LSP Gaps): 8 tasks
- Priority 5 (Code Quality): 5 tasks

**By Impact:**

- 🔴 Critical: ~~3 major issues~~ → 1 remaining (panics resolved)
- 🟡 High: ~~5 incomplete features~~ → 0 remaining (ALL COMPLETED: devirtualization, rich enum, export handling, module hoisting, class codegen)
- 🟢 Medium: 6 feature gaps
- 🔵 Low: 6 polish items

---

## Notes

- This TODO was generated from a comprehensive codebase review on 2026-02-08
- Comment quality in the codebase is **excellent** - maintain current standards
- Test coverage appears comprehensive
- Architecture is well-documented with module-level docs
- Focus on Priority 1 items first for stability and correctness
