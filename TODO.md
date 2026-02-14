# LuaNext TODO

## Medium Priority

### Incremental Parsing

**Rationale:** Improve LSP responsiveness for large files by only re-parsing modified regions. Currently full re-tokenization and re-parsing happens on every edit, which can be slow for large files (>1000 lines).

**Estimated Effort:** 3-4 weeks implementation + 1-2 weeks polish

**Benefits:**

- 5-10x faster for small edits (typing, single-line changes)
- 2-3x faster for medium edits (multi-line paste/delete)
- Near-instant LSP updates (hover, diagnostics, completion)

#### Phase 1: Foundation (Week 1-2) ✅ COMPLETE

- [x] **Source Range Tracking**
  - [x] ✅ `Span` already has byte offsets (`start` and `end` fields at line 10-12)
  - [x] Add helper methods to `Span`: `contains(offset)`, `overlaps(other)`, `shift(delta)`
  - [x] Add `statement_ranges: Vec<(usize, Span)>` to `Program` to track statement boundaries
  - [x] Add `span()` method to all Statement variants (many already have it via pattern)
  - Files: `crates/luanext-parser/src/span.rs`, `crates/luanext-parser/src/ast/mod.rs`

- [x] **Dirty Region Calculation**
  - [x] Create new module: `crates/luanext-parser/src/incremental/mod.rs`
  - [x] Add `TextEdit { range: (u32, u32), new_text: String }` struct
  - [x] Add `DirtyRegion { modified_range: (u32, u32), affected_statements: Vec<usize>, byte_delta: i32 }`
  - [x] Implement `calculate_dirty_regions(edits: &[TextEdit], statement_ranges: &[(usize, Span)]) -> DirtyRegionSet`
  - [x] Handle overlapping edits (merge into single dirty region)
  - [x] Algorithm: Binary search to find first/last affected statement by byte offset
  - Files: New module `crates/luanext-parser/src/incremental/dirty.rs`

- [x] **Cached Statement Structure**
  - [x] Create `CachedStatement<'arena> { statement: &'arena Statement<'arena>, byte_range: (u32, u32), source_hash: u64 }`
  - [x] Create `IncrementalParseTree<'arena> { version: u64, statements: Vec<CachedStatement<'arena>>, source_hash: u64, arena: Arc<Bump> }`
  - [x] Implement hash function for statement source text using `std::collections::hash_map::DefaultHasher`
  - [x] Add `is_valid(&self, source: &str) -> bool` to verify hash matches current source
  - [x] Store arena in Arc to manage lifetime across incremental updates
  - Files: `crates/luanext-parser/src/incremental/cache.rs`

#### Phase 2: Incremental Lexing (Week 2-3) ✅ COMPLETE

**Summary**: Implemented resumable lexer that can start tokenization from arbitrary byte offsets and token stream caching infrastructure. The lexer now supports UTF-8 safe position synchronization and can lex specific byte ranges. Token streams are cached in `CachedStatement` and can be adjusted/merged after edits. All 467 tests passing (15 new incremental lexing tests).

- [x] **Resumable Lexer**
  - [x] ✅ Added `byte_to_char_index()` helper for UTF-8 safe byte offset conversion
  - [x] ✅ Added `sync_position(&mut self, byte_offset: u32)` - scan source to calculate line/column from byte
  - [x] ✅ Added `Lexer::new_at(source, handler, interner, start_byte, start_line, start_col)` constructor
  - [x] ✅ Added `tokenize_from(&mut self, byte_offset: u32) -> Result<Vec<Token>, LexerError>`
  - [x] ✅ Added `tokenize_range(&mut self, start: u32, end: u32) -> Result<Vec<Token>, LexerError>`
  - [x] ✅ Added new `LexerError` variants: `InvalidByteOffset`, `InvalidRange`
  - Files: `crates/luanext-parser/src/lexer/mod.rs`, `crates/luanext-parser/src/errors.rs`

- [x] **Token Stream Caching**
  - [x] ✅ Added `tokens: Vec<Token>` field to `CachedStatement`
  - [x] ✅ Implemented `adjust_token_offsets(tokens: &[Token], edits: &[TextEdit]) -> Vec<Token>` - shift all spans
  - [x] ✅ Implemented `merge_token_streams(cached: &[Token], new: Vec<Token>, dirty_ranges: &[(u32, u32)]) -> Vec<Token>`
  - [x] ✅ Implemented `token_spans_boundary()` and `expand_dirty_region()` for edge case handling
  - [x] ✅ Created comprehensive integration tests (15 tests covering UTF-8, resumable lexing, token caching)
  - Files: `crates/luanext-parser/src/incremental/tokens.rs`, `crates/luanext-parser/tests/incremental_lexing_tests.rs`

#### Phase 3: Incremental Parsing (Week 3-4) ✅ COMPLETE

**Summary**: Implemented full incremental parsing with multi-arena caching, statement-level reuse, and garbage collection. The parser now has three optimization paths: (1) first parse, (2) no edits (full reuse), and (3) partial edits (hybrid cached + newly parsed). Statement cloning approach (via `Statement::clone()`) proved cleaner than lifetime transmutation. All 513 tests passing with zero clippy warnings.

- [x] **Statement-Level Caching**
  - [x] ✅ Added `Parser::parse_incremental(&mut self, prev_tree: Option<&IncrementalParseTree>, edits: &[TextEdit], source: &str)` method
  - [x] ✅ Implemented three optimization paths:
    1. **Fast path #1**: No previous tree → full parse with new `IncrementalParseTree`
    2. **Fast path #2**: No edits + hash match → reuse entire tree (clone statement values)
    3. **Incremental path**: Mixed clean/dirty statements → separate reusable from dirty, re-parse dirty regions, build hybrid statement list
  - [x] ✅ Clean/dirty separation via `is_statement_clean()` + `CachedStatement::is_valid()`
  - [x] ✅ Fallback to full parse when all statements dirty
  - [x] ✅ Return `Program` with mixed cached + newly parsed statements
  - [x] ✅ Created 4 integration tests in `crates/luanext-parser/tests/incremental_parsing_tests.rs`
  - Files: `crates/luanext-parser/src/parser/mod.rs` (lines 154-315)

- [x] **Offset Adjustment Algorithm**
  - [x] ✅ Implemented `is_statement_clean(stmt: &Statement, edits: &[TextEdit]) -> bool` for overlap detection
  - [x] ✅ Replaced deep span adjustment with simple validation approach (check if span overlaps any edit)
  - [x] ✅ Hash validation via `CachedStatement::is_valid(source: &str)` - recomputes hash and compares
  - [x] ✅ **Design decision**: No offset adjustment in cached statements - use original spans, only reuse if clean
  - [x] ✅ Simplified `adjustment.rs` to 77 lines (was complex deep-adjustment algorithm)
  - Files: `crates/luanext-parser/src/incremental/adjustment.rs` (completely rewritten)

- [x] **Arena Handling**
  - [x] ✅ **Implemented multi-arena approach** with max 3 arenas + periodic consolidation
    - Old statements stay in old arenas (kept alive via `Arc<Bump>` in `IncrementalParseTree`)
    - New statements allocated in new arena each parse
    - Each `CachedStatement` tracks `arena_generation: usize`
    - `IncrementalParseTree` holds `arenas: Vec<Arc<Bump>>` for multi-generation tracking
  - [x] ✅ Implemented `collect_garbage()` with two consolidation triggers:
    1. Arena count > 3 → immediate consolidation
    2. Every 10 parses → periodic consolidation
  - [x] ✅ Implemented `consolidate_all()` - clones all statements to new arena, drops old arenas
  - [x] ✅ Implemented `drop_unreferenced_arenas()` - normal GC for unused arenas
  - [x] ✅ **Statement cloning approach**: Completed `clone_statement_to_arena()` using `Statement::clone()` + `unsafe transmute` for lifetime casting
  - [x] ✅ Handles all 29+ Statement variants via derived Clone trait
  - [x] ✅ Added 2 new integration tests: `test_arena_consolidation_trigger()`, `test_periodic_consolidation_every_10_parses()`
  - [x] ✅ **Token stream caching**: Implemented - tokens cached per statement, reused for clean statements, extracted for dirty statements
  - [x] ✅ **Region-specific parsing**: Implemented `parse_statement_at_offset()` with proper byte offset seeking
  - [x] ✅ **Performance benchmarking**: Created comprehensive criterion benchmarks measuring single-char edits, line deletions, and multi-line pastes across 100/500/1000 line files
  - Files: `crates/luanext-parser/src/incremental/cache.rs` (lines 206-234), `crates/luanext-parser/src/parser/mod.rs` (lines 154-335), `crates/luanext-parser/benches/incremental_bench.rs`

**Phase 3 is COMPLETE** - All core functionality implemented and working including region-specific parsing and performance validation.

#### Phase 4: LSP Integration (Week 4-5) ✅ COMPLETE

**Summary**: Integrated incremental parsing infrastructure into LSP Document and DocumentManager with intelligent heuristics, comprehensive metrics, and thorough test coverage. The LSP now builds TextEdit structs from LSP content changes, analyzes edit patterns to decide parse strategy, tracks performance metrics, and passes edits to the incremental parser. All 3,138 tests passing with zero warnings.

- [x] **Document Manager Integration** ✅ COMPLETE
  - [x] ✅ Added `incremental_tree: RefCell<Option<IncrementalParseTree<'static>>>` to `Document` struct
  - [x] ✅ Modified `Document::get_or_parse_ast()` to delegate to `parse_with_edits(&[])`
  - [x] ✅ Created `parse_with_edits()` method that accepts `&[TextEdit]` and uses incremental parsing when edits provided
  - [x] ✅ Updated `DocumentManager::change()` to build `TextEdit` from `params.content_changes`
  - [x] ✅ Converts LSP `Range` to byte offsets using existing `position_to_offset()`
  - [x] ✅ Passes edits to `parse_incremental()` via `parse_with_edits()` instead of always doing full parse
  - [x] ✅ Stores resulting `IncrementalParseTree` back in document via `unsafe transmute` to 'static lifetime
  - [x] ✅ Clears incremental tree on cache clear (used on full document replacement)
  - [x] ✅ Implements fallback to full parse on parse error (try incremental, catch error, fallback to full)
  - [x] ✅ Arena handling: Wrapped `Bump` in `Arc` before parser use to avoid borrow checker issues
  - [x] ✅ Created 5 integration tests in `crates/luanext-lsp/tests/incremental_lsp_tests.rs`
  - Files: `crates/luanext-lsp/src/core/document.rs` (lines 77-195, 273-330)

- [x] **Optimize Common Edit Patterns** ✅ COMPLETE
  - [x] ✅ Created `ParseStrategyAnalyzer` with intelligent edit pattern detection
  - [x] ✅ Implemented `SingleLineOptimized` strategy: single-line edits <100 chars
  - [x] ✅ Implemented `AppendOnlyOptimized` strategy: typing at end of file
  - [x] ✅ Implemented heuristic thresholds:
    - `edit.text.len() > 1000` → FullParse (large paste)
    - `dirty_regions.len() > 10` → FullParse (too fragmented)
    - `affected_ratio > 50%` → FullParse (via `DirtyRegionSet::should_use_incremental()`)
  - [x] ✅ Added `ParseMetrics` with atomic counters:
    - `incremental_parse_count`, `full_parse_count`
    - `avg_incremental_time_ms`, `avg_full_parse_time_ms`
    - `incremental_ratio` (percentage of incremental parses)
  - [x] ✅ Integrated into `DocumentManager::change()` with strategy selection and timing
  - [x] ✅ Environment variable support: `LUANEXT_LSP_PARSE_STATS`, `LUANEXT_DISABLE_INCREMENTAL`, tuning vars
  - Files: `crates/luanext-lsp/src/core/heuristics.rs`, `crates/luanext-lsp/src/core/metrics.rs`, `crates/luanext-lsp/src/core/document.rs`, `crates/luanext-parser/src/incremental/dirty.rs`

- [x] **Testing & Benchmarking** ✅ COMPLETE
  - [x] ✅ Created 5 LSP integration tests verifying incremental infrastructure doesn't break normal parsing
  - [x] ✅ Existing parser benchmarks cover incremental parsing performance (see `BENCHMARK_RESULTS.md`)
  - [x] ✅ Added 12 unit tests for dirty region calculation in `dirty.rs`:
    - Edit at file start/end
    - Edit deletes entire statement
    - Edit deletes multiple statements
    - Multiple non-overlapping edits
    - Multiple overlapping edits merge
    - Adjacent edits merge
    - Three-way edit merge
    - Edit ends/starts at statement boundary
    - Zero-length edit insertion
    - Full statement replacement
    - Ratio calculation tests (affected_ratio, should_use_incremental)
  - [x] ✅ Added 10 unit tests for offset adjustment in `adjustment.rs`:
    - Statement clean after insertion
    - Statement dirty after overlapping deletion
    - Statement clean before edit
    - Statement dirty when edit inside
    - Sequential edits overlap detection
    - Replacement edit overlap
    - Edit exactly at statement end (no overlap)
    - Edit one char before statement (no overlap)
    - Empty edits - all statements clean
    - Large edit affects all statements
  - [x] ✅ Created 10 heuristics unit tests in `parse_heuristics_tests.rs`:
    - Single-line edit detection
    - Large edit forces full parse
    - Append-only detection (empty document and at end)
    - Many edits forces full parse
    - Full document replacement
    - Config from environment variables
    - Multiline edit uses standard incremental
    - Medium edit under threshold
    - Single-line edit over 100 chars
  - [x] ✅ Created 10 integration tests for edit scenarios in `incremental_edit_scenarios.rs`:
    - Type single character
    - Delete single line
    - Paste multiline code
    - Undo/redo sequence
    - Format document
    - Incremental typing sequence
    - Delete entire function
    - Insert new function
    - Comment out code
    - Uncomment code
  - [x] ✅ Created 12 LSP integration tests in `incremental_document_manager_tests.rs`:
    - Document parsing (single/multiple/empty statements, caching, interfaces)
    - Metrics tracking (initial state, after parse)
    - Heuristics (single-line, large edit, append, many edits, full replacement)
  - [x] ✅ Extended benchmarks in `incremental_bench.rs` with 6 new benchmarks:
    - Very large file (10,000 lines)
    - Small file (10 lines)
    - Append to end (typing at EOF)
    - Edit at start of file
    - Multi-statement deletion (delete 10 lines)
    - Format document (whitespace changes)
  - Files: `crates/luanext-parser/src/incremental/dirty.rs`, `crates/luanext-parser/src/incremental/adjustment.rs`, `crates/luanext-lsp/tests/parse_heuristics_tests.rs`, `crates/luanext-parser/tests/incremental_edit_scenarios.rs`, `crates/luanext-lsp/tests/incremental_document_manager_tests.rs`, `crates/luanext-parser/benches/incremental_bench.rs`

#### Phase 5: Polish & Optimization (Week 5-6) ✅ COMPLETE

**Summary**: Completed all performance tuning, edge case handling, and documentation tasks. Replaced `DefaultHasher` with `FxHasher` (~2-4x faster hashing), added `incremental-parsing` feature flag as safety valve, added debug logging (`LUANEXT_DEBUG_INCREMENTAL=1`), fixed `Arc<Bump>` → `Rc<Bump>` for correctness, and created comprehensive documentation. All edge cases handled via fallback-to-full-parse strategy with proper logging. Benchmarks show 2.5-3.0x speedup maintained across all edit scenarios.

- [x] **Performance Tuning**
  - [x] ✅ Replaced `DefaultHasher` with `FxHasher` for ~2-4x faster source hashing (cache.rs, parser/mod.rs)
  - [x] ✅ Added `incremental-parsing` feature flag (default on) — compile-time disable: `--no-default-features --features typed`
  - [x] ✅ Added debug logging: `LUANEXT_DEBUG_INCREMENTAL=1` prints parse decisions to stderr via `debug_incremental!` macro
  - [x] ✅ Fixed `Arc<Bump>` → `Rc<Bump>` (correctness: `Bump` is not `Sync`, avoids clippy `arc_with_non_send_sync`)
  - [x] ✅ Benchmarked: 2.5-3.0x speedup for single-statement edits, 1.4-1.5x for all-dirty scenarios (see `BENCHMARK_RESULTS.md`)

- [x] **Edge Cases**
  - [x] ✅ Multi-statement edits: Handled by `DirtyRegionSet::calculate()` — marks all overlapping statements dirty
  - [x] ✅ Statement boundary changes: Handled by full-reparse fallback (all dirty → full parse)
  - [x] ✅ Multi-line constructs: Statement-level overlap detection catches all edits within constructs
  - [x] ✅ Fallback to full reparse: `tracing::warn!` in LSP document.rs on `parse_with_edits()` error
  - [x] ✅ Unicode: Byte offsets work correctly — Rust UTF-8 strings + LSP position-to-offset conversion

- [x] **Documentation**
  - [x] ✅ Updated `architecture.md` with "Incremental Parsing" section (architecture flow, caching, arenas, performance, debug logging, file map)
  - [x] ✅ Enhanced doc comments on key algorithms: `DirtyRegionSet::calculate()`, `is_statement_clean()`, `adjust_span()`, `parse_incremental()`
  - [x] ✅ Created `docs/incremental-parsing.md` design document (design decisions, arena handling, performance, troubleshooting, file map)
  - [x] ✅ Added rustdoc examples on `TextEdit`, `DirtyRegionSet::calculate()`, `IncrementalParseTree` (all doctests pass)

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

### Type Checker Bug Fixes

- [x] ✅ **Fixed function call argument type validation** (2026-02-12)
  - **Issue**: `test_type_mismatch_in_function_call` was passing when it should fail - type checker wasn't catching `greet(123)` when `greet(name: string): void`
  - **Root cause**: `is_assignable_with_env_recursive()` inserted type pairs into visited set before checking, causing false positive cycle detection
  - **Fix**: Duplicated literal/primitive checking logic into `is_assignable_with_env_recursive()` (lines 174-189 in `type_compat.rs`)
  - **Result**: All 1778 tests passing (1464 lib + 314 integration), zero new clippy warnings
  - File: `crates/luanext-typechecker/src/core/type_compat.rs`

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
