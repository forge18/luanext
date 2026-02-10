# LuaNext TODO

## High Priority

### Enhanced Cross-File Type Support

**Rationale:** Core requirement for TypeScript-inspired type system. TypeScript does full cross-file type resolution, and it's fundamental to the value proposition.

**Estimated Effort:** 7-10 days (Phases 1-3 completed in 2 days - 2026-02-08 to 2026-02-09)

**Progress:** Phases 1, 2, 3 ✅ COMPLETE - 2026-02-09

**Phase 3 Summary (2026-02-09):**

- ✅ Dependency graph with EdgeKind enum (TypeOnly vs Value)
- ✅ Type-only cycles allowed, value cycles rejected with enhanced errors
- ✅ Lazy type resolution with circuit breaker prevents infinite recursion
- ✅ Type validation: runtime imports can't reference type-only exports
- ✅ Full CLI integration: ImportScanner detects `type` keyword
- ✅ LSP support: completion shows "(type-only import)", hover shows note
- ✅ Symbol index tracking: is_type_only field on ExportInfo/ImportInfo
- ✅ 10+ comprehensive tests covering all circular dependency scenarios
- ✅ All 446 typechecker tests pass; zero clippy warnings

**Commits:**

- `625f1cc` feat: Complete Phase 3 - Circular Type Dependencies (typechecker)
- `988ba91` feat: Phase 3 LSP Support - Type-Only Imports Visibility (lsp)
- `1f266d6` chore: Update subproject commits and main.rs (main repo)

#### Phase 1: Full Type Resolution Across Files (Days 1-3) ✅

- [x] **Enhance `resolve_import_type()` in module_phase.rs** ✅
  - [x] Implement lazy type resolution (on-demand type-checking of dependencies) - Via LazyTypeCheckCallback
  - [x] Fetch actual types from ModuleRegistry (not just Unknown fallbacks) - Proper error handling replaces Unknown fallback
  - [x] Add type validation between import and export declarations - Step 5 ✅
  - [x] Handle generic type parameter propagation across module boundaries - Step 6 ✅ (Foundation/placeholder)
  - [x] Implement proper error reporting for missing/incompatible exports - TypeCheckInProgress, ExportTypeMismatch, RuntimeImportOfTypeOnly
  - Files: `crates/luanext-typechecker/src/phases/module_phase.rs:437-485` ✅ Modified

- [x] **Type Compatibility Validation** ✅ (Step 5 - Implemented)
  - [x] Ensure type-only exports aren't imported as runtime values - Implemented via validate_import_export_compatibility()
  - [x] Generic type argument handling foundation - Step 6 ✅ (apply_type_arguments helper)
  - [x] Runtime vs type-only import distinction - ✅ Fully working

#### Phase 2: Type-Only Imports (Day 4) ✅ COMPLETE - 2026-02-09

- [x] **Codegen: Skip type-only imports** ✅
  - [x] `generate_import()` already skips `ImportClause::TypeOnly` - empty body at line 24
  - [x] No `require()` calls generated for type-only imports (verified)
  - Files: `crates/luanext-core/src/codegen/modules.rs:4-24`

- [x] **Type Checking: Validate type-only imports** ✅
  - [x] Type-only imports register as `SymbolKind::TypeAlias` in symbol table (lines 324-354)
  - [x] `validate_import_export_compatibility()` prevents runtime imports from using type-only exports
  - [x] `is_type_only_import: true` parameter correctly threaded through resolve_import_type()
  - Files: `crates/luanext-typechecker/src/phases/module_phase.rs:310-370`

- [x] **LSP: Distinguish type vs value imports** ✅
  - [x] Completion provider shows "(type-only import)" suffix for type-only imported symbols
  - [x] Hover provider shows "*Imported as type-only*" markdown note for type-only imported symbols
  - [x] Added `get_type_only_imports()` helper to completion.rs
  - [x] Added `is_type_only_import()` helper to hover.rs
  - Files:
    - `crates/luanext-lsp/src/features/edit/completion.rs` (lines 306-328)
    - `crates/luanext-lsp/src/features/navigation/hover.rs` (lines 132-156)

- [x] **Symbol Index: Track type-only imports/exports** ✅
  - [x] Added `is_type_only: bool` field to `ExportInfo` struct
  - [x] Added `is_type_only: bool` field to `ImportInfo` struct
  - [x] Added `is_declaration_type_only()` helper to determine type-only exports
  - [x] Implemented `ImportClause::TypeOnly` handling in `index_imports()`
  - [x] Updated all 9 test creations of ExportInfo/ImportInfo with new fields
  - Files: `crates/luanext-lsp/src/core/analysis/symbol_index.rs`

- [x] **Fixed all compiler warnings** ✅
  - [x] Changed `arena` parameter to `_arena` and added `#[allow(dead_code)]` to `apply_type_arguments`
  - [x] Removed all `needless_return` statements - replaced with expression-based returns
  - [x] Fixed `needless_borrow` in validate_import_export_compatibility calls
  - Files: `crates/luanext-typechecker/src/phases/module_phase.rs`
  - **All 441 tests pass; zero clippy warnings; no regressions**

#### Phase 3: Circular Type Dependencies & Forward Declarations (Days 5-6) ✅ COMPLETE - 2026-02-09

- [x] **Dependency Graph: Separate type and value edges** ✅
  - [x] Add `EdgeKind` enum (TypeOnly, Value) to DependencyGraph
  - [x] Track which imports are type-only vs runtime
  - [x] Allow cycles in type-only imports
  - [x] Error only on runtime circular dependencies
  - [x] 10 comprehensive unit tests (type cycles, value cycles, mixed scenarios, filtering)
  - Files: `crates/luanext-typechecker/src/module_resolver/dependency_graph.rs` ✅ Modified

- [x] **Better Error Messages** ✅
  - [x] Show full cycle path when runtime circular dependency detected
  - [x] Suggest using `import type` to break cycles
  - [x] Differentiate between type cycles (OK) and value cycles (ERROR)
  - [x] Enhanced error shows actionable before/after example
  - Files: `crates/luanext-typechecker/src/module_resolver/error.rs` ✅ Modified

- [x] **CLI Integration** ✅
  - [x] ImportScanner detects `type` keyword in import statements
  - [x] parse_import_statement returns (String, EdgeKind) tuples
  - [x] analyze_dependencies passes edge kinds to dependency graph
  - Files: `crates/luanext-cli/src/main.rs` ✅ Modified

- [x] **Type Checker Integration** ✅
  - [x] TypedDependency struct tracks dependencies with edge kinds
  - [x] check_import_statement uses Vec<TypedDependency>
  - [x] resolve_import_type determines and tracks edge kinds
  - [x] TypeCheckerState and TypeChecker updated
  - Files: `crates/luanext-typechecker/src/phases/module_phase.rs`, `module_resolver/mod.rs`, `state/type_checker_state.rs`, `core/type_checker.rs` ✅ Modified

- [x] **Forward Type Declarations** ✅ COMPLETE - 2026-02-09
  - [x] Implement forward declarations for interfaces (empty interface body)
  - [x] Implement forward declarations for classes (empty class body, no modifiers)
  - [x] Parser detects: `interface Foo {}` and `class Bar {}`
  - [x] Type checker registers forward-declared names for mutual references
  - [x] Protection: modifiers (abstract, final), decorators, inheritance prevent forward declaration
  - [x] All 446 typechecker tests pass; zero clippy warnings

#### Phase 4: Re-exports (Days 7-8) ✅ PARTIAL - Phase 4.1-4.2 COMPLETE - 2026-02-09

**Phase 4.1-4.2 Summary (2026-02-09):**

- ✅ Robust re-export resolution with cycle detection and lazy callbacks
- ✅ Critical codegen bug fix - re-exported symbols now accessible to importers
- ✅ Error types for circular re-exports, depth limits, type-only validation
- ✅ All 446 tests pass; zero regressions

**Commits:**

- `170dc32` feat: Phase 4.1-4.2 - Robust re-export resolution and critical codegen bug fix

- [x] **Transitive Type Resolution**
  - [x] Resolve types through re-export chains via `resolve_re_export()`
  - [x] Handle `export { Foo } from './module'` (named re-exports) ✅
  - [ ] Handle `export * from './module'` (export all, both values and types) - Phase 4.4
  - [ ] Handle `export type { Bar } from './module'` (type-only re-exports) - Phase 4.1 TODO
  - [x] Prevent circular re-exports (detect cycles during resolution) ✅
  - [ ] Cache re-export resolution to avoid redundant lookups - Phase 4.5
  - Files: `crates/luanext-typechecker/src/phases/module_phase.rs:227-295` ✅

- [x] **Type Checker Integration**
  - [x] `resolve_re_export()` walks export chain to find original type ✅
  - [x] Update ModuleRegistry via lazy callbacks for uncompiled dependencies ✅
  - [x] Handle mixed imports/exports (some re-exported, some local) ✅
  - [ ] Handle type-only re-exports validation - Phase 4.1 TODO
  - Files: `crates/luanext-typechecker/src/phases/module_phase.rs` ✅

- [x] **Codegen: Re-export support**
  - [x] Generate proper re-export code in bundle mode ✅
  - [x] Handle re-exports in require mode (passthrough to source module) ✅
  - [x] **CRITICAL FIX**: Add re-exported symbols to self.exports (was missing) ✅
  - [ ] Inline re-exports for bundler optimization - Phase 4.5
  - Files: `crates/luanext-core/src/codegen/modules.rs:171-238` ✅

- [ ] **LSP Support** - Phase 4.3
  - [ ] Go-to-definition follows re-export chains to original definition
  - [ ] Hover shows original module name with re-export note
  - [ ] Completion includes re-exported symbols with source info
  - [ ] Find references finds uses of re-exported symbols

#### Phase 5: Integration & Testing (Days 9-10)

- [ ] **End-to-End Tests**
  - [ ] Multi-file project with complex type dependencies
  - [ ] Circular type reference tests (should pass)
  - [ ] Circular value reference tests (should error)
  - [ ] Re-export chain tests (single level, multi-level, circular)
  - [ ] Type-only import/export tests
  - [ ] Mixed scenarios (import, re-export, import again)

- [ ] **LSP Testing**
  - [ ] Go-to-definition across files with type resolution
  - [ ] Hover shows correct types for cross-file imports
  - [ ] Completion works for cross-file types (including re-exported)
  - [ ] Find references across files through re-exports
  - [ ] Rename refactoring across files and re-exports

- [ ] **Performance Testing**
  - [ ] Large projects (100+ files) compile in reasonable time (<5 seconds for clean build)
  - [ ] LSP remains responsive with many cross-file references (<100ms for hover/completion)
  - [ ] Incremental compilation works with cross-file changes
  - [ ] Deep re-export chains don't cause performance degradation

- [ ] **Documentation**
  - [ ] Document cross-file type resolution in ARCHITECTURE.md
  - [ ] Add examples of type-only imports and re-exports
  - [ ] Document circular dependency handling
  - [ ] Explain re-export resolution strategy
  - [ ] Add migration guide for existing code

---

## Medium Priority

### Incremental Parsing

**Rationale:** Improve LSP responsiveness for large files by only re-parsing modified regions. Currently full re-tokenization and re-parsing happens on every edit, which can be slow for large files (>1000 lines).

**Estimated Effort:** 3-4 weeks implementation + 1-2 weeks polish

**Benefits:**

- 5-10x faster for small edits (typing, single-line changes)
- 2-3x faster for medium edits (multi-line paste/delete)
- Near-instant LSP updates (hover, diagnostics, completion)

#### Phase 1: Foundation (Week 1-2)

- [ ] **Source Range Tracking**
  - [ ] ✅ `Span` already has byte offsets (`start` and `end` fields at line 10-12)
  - [ ] Add helper methods to `Span`: `contains(offset)`, `overlaps(other)`, `shift(delta)`
  - [ ] Add `statement_ranges: Vec<(usize, Span)>` to `Program` to track statement boundaries
  - [ ] Add `span()` method to all Statement variants (many already have it via pattern)
  - Files: `crates/luanext-parser/src/span.rs`, `crates/luanext-parser/src/ast/mod.rs`

- [ ] **Dirty Region Calculation**
  - [ ] Create new module: `crates/luanext-parser/src/incremental/mod.rs`
  - [ ] Add `TextEdit { range: (u32, u32), new_text: String }` struct
  - [ ] Add `DirtyRegion { modified_range: (u32, u32), affected_statements: Vec<usize>, byte_delta: i32 }`
  - [ ] Implement `calculate_dirty_regions(edits: &[TextEdit], statement_ranges: &[(usize, Span)]) -> DirtyRegionSet`
  - [ ] Handle overlapping edits (merge into single dirty region)
  - [ ] Algorithm: Binary search to find first/last affected statement by byte offset
  - Files: New module `crates/luanext-parser/src/incremental/dirty.rs`

- [ ] **Cached Statement Structure**
  - [ ] Create `CachedStatement<'arena> { statement: &'arena Statement<'arena>, byte_range: (u32, u32), source_hash: u64 }`
  - [ ] Create `IncrementalParseTree<'arena> { version: u64, statements: Vec<CachedStatement<'arena>>, source_hash: u64, arena: Arc<Bump> }`
  - [ ] Implement hash function for statement source text using `std::collections::hash_map::DefaultHasher`
  - [ ] Add `is_valid(&self, source: &str) -> bool` to verify hash matches current source
  - [ ] Store arena in Arc to manage lifetime across incremental updates
  - Files: `crates/luanext-parser/src/incremental/cache.rs`

#### Phase 2: Incremental Lexing (Week 2-3)

- [ ] **Resumable Lexer**
  - [ ] Modify `Lexer` struct: add `start_offset: u32` field (currently always starts at 0)
  - [ ] Add `Lexer::new_at(source, handler, interner, start_byte, start_line, start_col)` constructor
  - [ ] Add `sync_position(&mut self, byte_offset: u32)` - scan source to calculate line/column from byte
  - [ ] Add `tokenize_from(&mut self, byte_offset: u32) -> Result<Vec<Token>, LexerError>`
  - [ ] Add `tokenize_range(&mut self, start: u32, end: u32) -> Result<Vec<Token>, LexerError>`
  - [ ] Update `current()` method to respect `start_offset`
  - Files: `crates/luanext-parser/src/lexer/mod.rs` (lines 12-41 current Lexer struct)

- [ ] **Token Stream Caching**
  - [ ] Add `tokens: Vec<Token>` field to `CachedStatement`
  - [ ] Implement `adjust_token_offsets(tokens: &mut [Token], delta: i32)` - shift all spans
  - [ ] Create `TokenCache { ranges: Vec<(u32, u32, Vec<Token>)> }` for granular caching
  - [ ] Implement `merge_token_streams(cached: &[Token], new: Vec<Token>) -> Vec<Token>`
  - [ ] Handle edge case: tokens that span dirty region boundary (invalidate and re-lex)
  - Files: `crates/luanext-parser/src/incremental/tokens.rs`

#### Phase 3: Incremental Parsing (Week 3-4)

- [ ] **Statement-Level Caching**
  - [ ] Add `Parser::parse_incremental(&mut self, prev_tree: Option<&IncrementalParseTree>, edits: &[TextEdit])` method
  - [ ] Algorithm:
    1. Calculate dirty regions from edits
    2. Iterate through prev_tree.statements
    3. If statement outside dirty region: adjust offsets, reuse AST node
    4. If statement in dirty region: re-parse from adjusted position
    5. Collect new statements in dirty region
  - [ ] Handle new statements (insertions that create additional statements)
  - [ ] Handle deleted statements (mark range, skip in output)
  - [ ] Return `Program` with mixed cached + newly parsed statements
  - Files: `crates/luanext-parser/src/parser/mod.rs` (add after line 140)

- [ ] **Offset Adjustment Algorithm**
  - [ ] Implement `adjust_span(span: &mut Span, edit_end: u32, delta: i32)`
    - If `span.start >= edit_end`: `span.start += delta; span.end += delta`
    - If `span.end <= edit_start`: no change
    - If span overlaps edit: invalidate (return None, requires re-parse)
  - [ ] Implement `adjust_statement_offsets(stmt: &CachedStatement, edits: &[TextEdit]) -> Option<CachedStatement>`
  - [ ] Handle multiple edits: apply deltas cumulatively from left to right
  - [ ] Validate: adjusted span still within source bounds
  - [ ] Validate: adjusted hash still matches source text (detect subtle corruption)
  - Files: `crates/luanext-parser/src/incremental/offset.rs`

- [ ] **Arena Handling**
  - [ ] **Decision: Hybrid approach** (best for LuaNext)
    - Keep old arena alive via `Arc<Bump>` in `IncrementalParseTree`
    - Create new arena for incremental parse
    - Cached statements point to old arena (lifetime 'static via Arc)
    - New statements allocated in new arena
    - Old arena dropped when all references gone (automatic via Arc)
  - [ ] Update `Document::get_or_parse_ast()` to store `Arc<Bump>` (currently at line 148)
  - [ ] Modify `IncrementalParseTree` to hold `Vec<Arc<Bump>>` for multi-generation arenas
  - [ ] Add `arena_generation: usize` to `CachedStatement` to track which arena owns it
  - [ ] Implement `collect_garbage()` to drop old arenas when no statements reference them
  - Files: `crates/luanext-parser/src/parser/mod.rs`, `crates/luanext-parser/src/incremental/cache.rs`

#### Phase 4: LSP Integration (Week 4-5)

- [ ] **Document Manager Integration**
  - [ ] Add `incremental_tree: RefCell<Option<IncrementalParseTree<'static>>>` to `Document` struct (line 77-86)
  - [ ] Modify `Document::get_or_parse_ast()` to use incremental parsing if tree exists (line 120)
  - [ ] Update `DocumentManager::change()` to build `TextEdit` from `params.content_changes` (line 239)
  - [ ] In `change()` at line 245: convert LSP `Range` to byte offsets using existing `position_to_offset()`
  - [ ] Pass edits to `Parser::parse_incremental()` instead of always doing full parse
  - [ ] Store resulting `IncrementalParseTree` back in document for next edit
  - [ ] Clear incremental tree on file close or on parse error (fallback to full reparse)
  - Files: `crates/luanext-lsp/src/core/document.rs` (lines 77-163)

- [ ] **Optimize Common Edit Patterns**
  - [ ] Detect single-line edit: `edit.range.start.line == edit.range.end.line && edit.text.len() < 100`
    - Only re-parse containing statement + next statement (for context)
    - Fastest path: ~10x faster than full parse
  - [ ] Detect append-only edit: `edit.range.start.offset == doc.text.len()`
    - Only parse new text as additional statements
    - Common when typing at end of file
  - [ ] Heuristic thresholds:
    - `edit.text.len() > 1000` → full reparse (likely paste of large code block)
    - `dirty_regions.len() > 10` → full reparse (too fragmented)
    - `affected_statements > 50%` → full reparse (not worth incremental)
  - [ ] Add metrics: `incremental_parse_count`, `full_parse_count`, `avg_parse_time_ms`
  - Files: `crates/luanext-lsp/src/core/document.rs`, new `crates/luanext-parser/src/incremental/heuristics.rs`

- [ ] **Testing & Benchmarking**
  - [ ] Unit tests for dirty region calculation:
    - Single edit in middle of file
    - Multiple overlapping edits
    - Edit at start/end of file
    - Edit that deletes entire statement
  - [ ] Unit tests for offset adjustment:
    - Insertion increases offsets
    - Deletion decreases offsets
    - Replacement (delete + insert)
    - Multiple sequential edits
  - [ ] Integration tests for edit scenarios:
    - Type single character (most common)
    - Delete line
    - Paste multi-line code
    - Undo/redo sequences
    - Format document (large structural change)
  - [ ] Benchmark suite (use criterion):
    - Small file (100 lines): measure overhead vs full parse
    - Medium file (1000 lines): measure speedup for typical edits
    - Large file (10000 lines): measure worst-case performance
    - Compare: incremental vs full parse for 1-char, 1-line, 10-line edits
  - Files: `crates/luanext-parser/tests/incremental_tests.rs`, `crates/luanext-parser/benches/incremental_bench.rs`

#### Phase 5: Polish & Optimization (Week 5-6)

- [ ] **Performance Tuning**
  - [ ] Profile with `cargo flamegraph` on real-world LuaNext files
  - [ ] Identify hotspots: likely candidates:
    - `calculate_dirty_regions()` - optimize with binary search
    - `adjust_span()` - called many times, inline and optimize
    - `hash_source()` - consider faster hash function (xxhash vs DefaultHasher)
  - [ ] Memory analysis:
    - Measure: `IncrementalParseTree` size vs original AST size
    - Target: <50% overhead (acceptable for 5-10x speed gain)
    - If overhead too high: implement LRU cache for oldest statement generations
  - [ ] Add feature flag `incremental-parsing` to disable if issues found
  - [ ] Add debug logging: `LUANEXT_DEBUG_INCREMENTAL=1` shows parse decisions

- [ ] **Edge Cases**
  - [ ] Handle edits that span multiple statements:
    - Example: Delete from middle of statement 5 to middle of statement 8
    - Solution: Mark all affected statements dirty, re-parse as single chunk
  - [ ] Handle edits that change statement boundaries:
    - Example: Add `;` splitting one statement into two
    - Example: Remove `end` merging two statements
    - Solution: Re-parse dirty region, detect statement count change, rebuild index
  - [ ] Handle edits in multi-line constructs:
    - Example: Edit inside string literal, multi-line comment, function body
    - Solution: Extend dirty region to encompass entire construct (scan for boundaries)
  - [ ] Fallback to full reparse:
    - When `parse_incremental()` returns error
    - When adjusted spans fail validation
    - When source hash mismatch detected (corruption)
    - Log warning: "Incremental parse failed, falling back to full parse: {reason}"
  - [ ] Unicode handling: ensure byte offsets work correctly with multi-byte chars
    - Rust strings are UTF-8, `char_indices()` gives byte positions ✓
    - Verify: LSP Position (line/col) → byte offset conversion handles Unicode

- [ ] **Documentation**
  - [ ] Update `ARCHITECTURE.md` section "Performance Optimization":
    - Add "Incremental Parsing" subsection
    - Explain statement-level caching strategy
    - Document dirty region algorithm
    - Show example: edit at line 500 of 1000-line file only re-parses ~3 statements
  - [ ] Add code comments to key algorithms:
    - `calculate_dirty_regions()` - explain binary search approach
    - `adjust_span()` - document the 3 cases (before/after/overlapping edit)
    - `parse_incremental()` - document the 5-step algorithm
  - [ ] Create `docs/incremental-parsing.md`:
    - Design decisions: why statement-level vs token-level
    - Arena handling: why hybrid multi-arena approach
    - Performance characteristics: when incremental is faster/slower
    - Troubleshooting: how to debug incremental parsing issues
  - [ ] Add rustdoc examples to public API:

    ```rust
    /// Parse incrementally from previous parse tree
    /// # Example
    /// ```
    /// let edit = TextEdit { range: (100, 105), new_text: "local x = 1".into() };
    /// let program = parser.parse_incremental(Some(&prev_tree), &[edit])?;
    /// ```
    ```

### Better Caching in LSP

**Rationale:** Current LSP caching is minimal - only AST and symbol table are cached. Many expensive operations (semantic tokens, type checking, symbol lookups) are recomputed on every request. Better caching can reduce LSP response latency from 50-100ms to <10ms for cached operations.

**Estimated Effort:** 2-3 weeks

**Current State:**

- ✅ AST caching: `Document.ast: RefCell<Option<ParsedAst>>` (line 81)
- ✅ Symbol table caching: `Document.symbol_table: Option<Arc<SymbolTable>>` (line 83)
- ❌ No caching for: semantic tokens, hover info, completion items, diagnostics, type info

**Benefits:**

- 5-10x faster hover/completion on unchanged files
- Near-instant semantic token refresh (currently ~50ms)
- Reduced CPU usage during idle editing
- Better perceived responsiveness

#### Phase 1: Infrastructure (Week 1)

- [ ] **Generic Cache Framework**
  - [ ] Create `crates/luanext-lsp/src/core/cache.rs` module
  - [ ] Implement `VersionedCache<T> { value: T, version: i32, last_updated: Instant }`
  - [ ] Implement `CacheKey` trait for hashable cache keys
  - [ ] Create `DocumentCache` struct to hold all per-document caches:

    ```rust
    pub struct DocumentCache {
        semantic_tokens: VersionedCache<Vec<SemanticToken>>,
        hover_cache: HashMap<Position, VersionedCache<Hover>>,
        completion_cache: HashMap<Position, VersionedCache<Vec<CompletionItem>>>,
        diagnostics: VersionedCache<Vec<Diagnostic>>,
        type_info_cache: HashMap<Position, VersionedCache<Type>>,
    }
    ```

  - [ ] Add `cache: RefCell<DocumentCache>` to `Document` struct
  - [ ] Implement `invalidate_all()` method called on text changes
  - [ ] Implement `is_valid(version: i32) -> bool` for version checking
  - Files: New `crates/luanext-lsp/src/core/cache.rs`, `crates/luanext-lsp/src/core/document.rs`

- [ ] **Cache Invalidation Strategy**
  - [ ] On `didChange`: Full invalidation (clear all caches)
  - [ ] On `didSave`: Selective invalidation (keep AST/symbols if unchanged)
  - [ ] Implement `partial_invalidate(affected_range: Range)` for incremental updates:
    - Clear hover/completion caches for positions in range
    - Keep semantic tokens if edit doesn't change token structure
    - Keep diagnostics if edit is in comment/whitespace
  - [ ] Add LRU eviction: keep max 100 hover entries, 50 completion entries per file
  - Files: `crates/luanext-lsp/src/core/cache.rs`

#### Phase 2: Semantic Token Caching (Week 1-2)

- [ ] **Full File Semantic Tokens**
  - [ ] Current: `semantic_tokens()` recomputes on every request (~50ms for 1000 lines)
  - [ ] Add `semantic_tokens: VersionedCache<SemanticTokens>` to `DocumentCache`
  - [ ] In `semantic_tokens.rs::provide()`: check cache before computing
  - [ ] Store result with document version
  - [ ] Return cached result if `doc.version == cache.version`
  - [ ] Invalidate only on text change (not on save/format)
  - [ ] Expected speedup: 50ms → <1ms for cache hit
  - Files: `crates/luanext-lsp/src/features/semantic/semantic_tokens.rs` (lines ~50-200)

- [ ] **Incremental Semantic Token Updates** (Advanced)
  - [ ] For single-line edits: only re-lex/re-tokenize affected lines
  - [ ] Implement `update_semantic_tokens(prev: &SemanticTokens, edit: TextEdit) -> SemanticTokens`
  - [ ] Adjust token positions after edit (similar to incremental parsing offset adjustment)
  - [ ] Fallback to full recompute if edit spans multiple lines or changes structure
  - Files: `crates/luanext-lsp/src/features/semantic/incremental.rs` (new)

#### Phase 3: Hover & Completion Caching (Week 2)

- [ ] **Hover Information Cache**
  - [ ] Current: `hover()` re-parses AST, re-resolves symbols on every hover (~20-30ms)
  - [ ] Add position-based cache: `HashMap<Position, VersionedCache<Hover>>`
  - [ ] Cache key: `(line, column, doc_version)`
  - [ ] TTL: 5 seconds (hover spam protection)
  - [ ] Max entries: 100 per document (LRU eviction)
  - [ ] Invalidate on text change or when `position` is in dirty region
  - [ ] Expected speedup: 20-30ms → <1ms for repeated hovers
  - Files: `crates/luanext-lsp/src/features/navigation/hover.rs`

- [ ] **Completion Items Cache**
  - [ ] Current: `completion()` rescans symbols, re-filters on every keystroke (~30-50ms)
  - [ ] Challenge: Completion context changes rapidly (trigger chars, partial identifiers)
  - [ ] Strategy: Cache member completions by type hash
    - Example: Hovering on `foo.` where `foo: Table<string, number>`
    - Cache key: `type_hash(typeof(foo))` + trigger character
    - Cache result: `Vec<CompletionItem>` for that type's members
  - [ ] Implement `CompletionContext { trigger_kind, trigger_char, partial_text }` as cache key
  - [ ] Short TTL: 2 seconds (typing is transient)
  - [ ] Max entries: 50 per document
  - [ ] Expected speedup: 30-50ms → 5-10ms (still need filtering, but pre-computed list)
  - Files: `crates/luanext-lsp/src/features/edit/completion.rs`

#### Phase 4: Type Checking & Diagnostics Cache (Week 2-3)

- [ ] **Type Information Cache**
  - [ ] Current: Type info recomputed for every hover/completion/diagnostic request
  - [ ] Add `TypeCache { exprs: HashMap<ExprId, Type>, stmts: HashMap<StmtId, TypeEnv> }`
  - [ ] Populate during type checking phase (one-time cost)
  - [ ] Store in `Document` alongside AST
  - [ ] Query cache in hover/completion providers instead of re-running type checker
  - [ ] Invalidate on AST change (same invalidation as AST cache)
  - [ ] Expected speedup: Eliminates redundant type checking (currently run 3-5x per edit)
  - Files: `crates/luanext-lsp/src/core/document.rs`, integration with type checker

- [ ] **Diagnostics Deduplication**
  - [ ] Current: Diagnostics sent on every `didChange`, even if unchanged
  - [ ] Add `last_diagnostics: VersionedCache<Vec<Diagnostic>>` to `DocumentCache`
  - [ ] Before sending diagnostics:
    - Compute hash of new diagnostic list
    - Compare with cached hash
    - If identical: skip `PublishDiagnostics` notification
  - [ ] Reduces LSP traffic and client-side UI updates
  - [ ] Particularly helpful for files with many warnings (red squiggles stop flickering)
  - Files: `crates/luanext-lsp/src/core/document.rs` (around diagnostic publishing)

#### Phase 5: Cross-File Caching (Week 3)

- [ ] **Module Type Exports Cache**
  - [ ] Current: Module exports re-resolved on every cross-file query
  - [ ] Add `ModuleExportsCache: HashMap<ModuleId, (Arc<ModuleExports>, FileHash)>`
  - [ ] Store in `DocumentManager` (shared across documents)
  - [ ] Cache key: Module path + file modification time
  - [ ] Invalidate when dependency file changes (watch `didChange` for imported files)
  - [ ] Implement cascade invalidation: if `a.lua` imports `b.lua` and `b.lua` changes → invalidate `a.lua` caches
  - [ ] Expected benefit: Multi-file hover/completion becomes practical (currently too slow)
  - Files: `crates/luanext-lsp/src/core/document.rs` (DocumentManager)

- [ ] **Symbol Index Incremental Updates**
  - [ ] Current: `SymbolIndex::update_document()` rebuilds entire index on every change
  - [ ] Implement incremental update:
    - Track which symbols changed (added/removed/modified)
    - Update only affected index entries
    - Keep unchanged symbol entries
  - [ ] Requires statement-level dirty tracking (synergy with incremental parsing)
  - [ ] Expected speedup: Index update from O(n) to O(changed statements)
  - Files: `crates/luanext-lsp/src/core/analysis/symbol_index.rs`

#### Phase 6: Testing & Optimization (Week 3)

- [ ] **Cache Effectiveness Metrics**
  - [ ] Add telemetry counters:
    - `cache_hits`, `cache_misses` per cache type
    - `avg_response_time_cached` vs `avg_response_time_uncached`
    - `memory_usage_caches` (track cache overhead)
  - [ ] Log metrics with `LUANEXT_LSP_CACHE_STATS=1` environment variable
  - [ ] Identify which caches provide most value (data-driven optimization)

- [ ] **Memory Profiling**
  - [ ] Measure memory usage with various cache sizes
  - [ ] Target: <20MB cache overhead for 100 open documents
  - [ ] Implement cache size limits:
    - Max total cache size: 50MB
    - LRU eviction when limit reached
    - Per-document limits to prevent one large file from evicting all caches
  - [ ] Profile with `cargo instruments` (macOS) or `heaptrack` (Linux)

- [ ] **Cache Correctness Testing**
  - [ ] Test: Edit document → hover → should show updated info (not stale cache)
  - [ ] Test: Rename symbol → completion cache invalidated → new name appears
  - [ ] Test: Change imported file → dependent file caches invalidated
  - [ ] Test: Rapid edits → caches don't thrash (avoid invalidate-recompute-invalidate loop)
  - [ ] Add integration tests: `crates/luanext-lsp/tests/cache_tests.rs`

- [ ] **Documentation**
  - [ ] Document caching architecture in `ARCHITECTURE.md`
  - [ ] Add section "LSP Performance Optimization" with cache strategy
  - [ ] Document cache invalidation rules
  - [ ] Add troubleshooting: "If LSP shows stale data, try restarting language server"

### Language Features

- [ ] Async/await syntax
- [ ] String pattern matching improvements
- [ ] Type assertions with runtime checks

### Optimizer O2/O3 Passes

- [ ] Implement remaining O2 passes (function inlining, loop optimization, etc.)
- [ ] Implement O3 passes (aggressive inlining, devirtualization, etc.)

### Error Messages

- [ ] Improve type mismatch error messages with suggestions
- [ ] Add "did you mean?" suggestions for typos
- [ ] Better error recovery in parser

---

## Low Priority

### Tooling

- [ ] Package manager integration
- [ ] Better VSCode extension features
- [ ] Playground/REPL

### Documentation

- [ ] Comprehensive language guide
- [ ] Migration guide from Lua
- [ ] Best practices guide
- [ ] API documentation for stdlib

---

## Completed ✅

- [x] Basic lexer, parser, type checker, codegen
- [x] LSP implementation
- [x] O1 optimizer passes
- [x] Incremental compilation with caching
- [x] Rich enums with fields and methods
- [x] Exception handling (try/catch/finally)
- [x] Operator overloading
- [x] Safe navigation (`?.`) and null coalescing (`??`)
- [x] File namespaces
- [x] Interface default methods
