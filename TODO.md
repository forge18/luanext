# LuaNext TODO

## Medium Priority

### Better Caching in LSP

**Rationale:** Current LSP caching is minimal - only AST and symbol table are cached. Many expensive operations (semantic tokens, type checking, symbol lookups) are recomputed on every request. Better caching can reduce LSP response latency from 50-100ms to <10ms for cached operations.

**Estimated Effort:** 2-3 weeks

**Current State:**

- ✅ AST caching: `Document.ast: RefCell<Option<ParsedAst>>` (line 81)
- ✅ Symbol table caching: `Document.symbol_table: Option<Arc<SymbolTable>>` (line 83)
- ✅ Semantic token caching: `VersionedCache<SemanticTokens>` with incremental updates
- ✅ Type-check result caching: `VersionedCache<TypeCheckResult>` shared across hover/completion/diagnostics
- ✅ Diagnostics deduplication: `last_published_diagnostics` skips identical notifications
- ✅ Cross-file module exports cache with dependency graph cascade invalidation
- ✅ Symbol index incremental updates via snapshot diffing

**Benefits:**

- 5-10x faster hover/completion on unchanged files
- Near-instant semantic token refresh (currently ~50ms)
- Reduced CPU usage during idle editing
- Better perceived responsiveness

#### Phase 1: Infrastructure (Week 1) ✅ COMPLETE (2026-02-14)

- [x] **Generic Cache Framework**
  - [x] Created `crates/luanext-lsp/src/core/cache.rs` module with:
    - `VersionedCache<T>`: Single-value cache with version tracking (`is_valid()`, `get_if_valid()`, `set()`, `invalidate()`)
    - `BoundedPositionCache<T>`: Position-keyed LRU cache with bounded capacity (used for hover/completion)
    - `DocumentCache`: Per-document container aggregating all cache types (placeholder `()` types for Phase 1)
    - `CacheStats`: Hit/miss counters with `LUANEXT_LSP_CACHE_STATS=1` logging support
  - [x] Added `cache: RefCell<DocumentCache>` to `Document` struct with `cache()`/`cache_mut()` accessors
  - [x] `invalidate_all()` clears all caches; `partial_invalidate(start, end)` for range-based invalidation
  - Files: `crates/luanext-lsp/src/core/cache.rs`, `crates/luanext-lsp/src/core/document.rs`

- [x] **Cache Invalidation Strategy**
  - [x] On `didChange`: Full invalidation via `doc.cache.borrow_mut().invalidate_all()`
  - [x] On `didSave`: Selective invalidation (keeps caches if text unchanged, invalidates if formatter modified text)
  - [x] `partial_invalidate(start, end)`: Clears semantic tokens + diagnostics fully; position caches only within range
  - [x] LRU eviction: max 100 hover entries, 50 completion entries, 100 type info entries per document
  - **Result**: 27 new tests (25 unit + 2 integration), all 1,750 tests passing, zero clippy warnings
  - Files: `crates/luanext-lsp/src/core/cache.rs`

#### Phase 2: Semantic Token Caching (Week 1-2) ✅ COMPLETE (2026-02-14)

- [x] **Full File Semantic Tokens**
  - [x] Changed `DocumentCache.semantic_tokens` from `VersionedCache<()>` to `VersionedCache<SemanticTokens>`
  - [x] `provide_full()` checks cache first, returns cached result if `doc.version == cache.version`
  - [x] On cache miss, computes tokens via `compute_semantic_tokens()` and stores in cache
  - [x] `result_id` uses document version (deterministic, not SystemTime)
  - [x] Cache hit/miss stats tracked via `CacheStats` with `LUANEXT_LSP_CACHE_STATS=1` logging
  - [x] Invalidated on text change (not on save/format if text unchanged)
  - [x] Expected speedup: 50ms → <1ms for cache hit
  - Files: `crates/luanext-lsp/src/features/semantic/semantic_tokens.rs`, `crates/luanext-lsp/src/core/cache.rs`

- [x] **Incremental Semantic Token Updates**
  - [x] Created `crates/luanext-lsp/src/features/semantic/incremental.rs` with:
    - `TokenTextEdit`: Describes text edits with line/char positions and new text dimensions
    - `update_semantic_tokens()`: Adjusts token positions for single-line edits without full recompute
  - [x] Reconstructs absolute positions, applies char delta, re-encodes as deltas
  - [x] Falls back to full recompute for multi-line edits or structural changes
  - [x] Integrated into `DocumentManager.change()`: tries incremental update before invalidation
  - [x] `provide_full_delta()` wired up with cached tokens for computing deltas
  - **Result**: 18 new tests (8 unit + 10 integration), all 3,230 tests passing, zero clippy warnings
  - Files: `crates/luanext-lsp/src/features/semantic/incremental.rs`, `crates/luanext-lsp/src/core/document.rs`, `crates/luanext-lsp/tests/semantic_cache_tests.rs`

#### Phase 3: Hover & Completion Caching (Week 2) ✅ COMPLETE (2026-02-14)

- [x] **Hover Information Cache**
  - [x] Changed `hover_cache: BoundedPositionCache<()>` to `BoundedPositionCache<Hover>` in `DocumentCache`
  - [x] `provide_impl()` checks cache first, returns cached `Hover` on hit (clone to release RefMut)
  - [x] Cache key: `Position` + `document.version` (via `BoundedPositionCache` version tracking)
  - [x] Max entries: 100 per document (LRU eviction, inherited from Phase 1 infrastructure)
  - [x] Invalidated on text change via `invalidate_all()` in `DocumentManager::change()`
  - [x] Caches all hover types: keywords, builtin types, and symbol type info
  - [x] Cache stats tracked via `CacheStats` with `LUANEXT_LSP_CACHE_STATS=1` logging
  - [x] Expected speedup: 20-30ms → <1ms for repeated hovers
  - **Result**: 3 new unit tests, 2 new integration tests, all 856 lib tests passing
  - Files: `crates/luanext-lsp/src/features/navigation/hover.rs`, `crates/luanext-lsp/src/core/cache.rs`

- [x] **Completion Items Cache**
  - [x] Changed `completion_cache: BoundedPositionCache<()>` to `BoundedPositionCache<Vec<CompletionItem>>`
  - [x] `provide_with_workspace()` checks cache first for expensive contexts only
  - [x] Context-aware: caches Statement, MemberAccess, MethodCall (type-checking heavy)
  - [x] Skips caching for TypeAnnotation, Decorator, Import (cheap hardcoded lists)
  - [x] Max entries: 50 per document (LRU eviction)
  - [x] Cache stats only recorded for cacheable contexts
  - [x] Expected speedup: 30-50ms → <1ms for cached completions
  - **Result**: 4 new unit tests, 2 new integration tests, all 856 lib tests passing, zero clippy warnings
  - Files: `crates/luanext-lsp/src/features/edit/completion.rs`, `crates/luanext-lsp/src/core/cache.rs`

#### Phase 4: Type Checking & Diagnostics Cache (Week 2-3) ✅ COMPLETE (2026-02-14)

- [x] **Type Information Cache**
  - [x] Added `TypeCheckResult` and `CachedSymbolInfo` structs to `cache.rs` — owned types extracted from arena-scoped type checker
  - [x] Added `type_check_result: VersionedCache<TypeCheckResult>` to `DocumentCache`
  - [x] Created `DiagnosticsProvider::ensure_type_checked()` — single entry point for type checking, checks cache first, runs lex→parse→typecheck on miss
  - [x] Refactored `hover.rs` to use `ensure_type_checked()` instead of running its own type checker
  - [x] Refactored `completion.rs` `complete_symbols()` to use `ensure_type_checked()` instead of running its own type checker
  - [x] `complete_members()` still uses its own type checker (needs `Type<'arena>` for member iteration)
  - [x] Updated `document.rs` cache invalidation: `type_check_result.invalidate()` on `didChange`
  - [x] Expected speedup: 3-5x type-check passes per edit → 1 pass (first feature request), subsequent features ~free
  - **Result**: 6 new unit tests, all 3,240 tests passing, zero clippy warnings
  - Files: `crates/luanext-lsp/src/core/cache.rs`, `crates/luanext-lsp/src/core/diagnostics.rs`, `crates/luanext-lsp/src/features/navigation/hover.rs`, `crates/luanext-lsp/src/features/edit/completion.rs`, `crates/luanext-lsp/src/core/document.rs`

- [x] **Diagnostics Deduplication**
  - [x] Added `last_published_diagnostics: Option<Vec<Diagnostic>>` to `DocumentCache`
  - [x] `publish_diagnostics()` in `message_handler.rs` compares new diagnostics with last published
  - [x] Skips `PublishDiagnostics` notification when diagnostics are identical
  - [x] `last_published_diagnostics` intentionally NOT invalidated on `didChange` — stores "what the client last saw"
  - [x] Reduces LSP traffic and eliminates red squiggle flickering
  - Files: `crates/luanext-lsp/src/message_handler.rs`, `crates/luanext-lsp/src/core/cache.rs`

- [x] **Bug Fix: Use-after-free in AST caching (SIGSEGV)**
  - [x] `parse_full()` and `parse_with_edits()` were taking `&program` (reference to stack-local `Program`) and transmuting to `'static`
  - [x] After function return, stack local was dropped → dangling reference → SIGSEGV on access
  - [x] Fixed by allocating `Program` in arena via `arena.alloc(program)` so reference lives as long as `Arc<Bump>`
  - [x] 3 previously-crashing cross-file completion tests now pass
  - Files: `crates/luanext-lsp/src/core/document.rs`

#### Phase 5: Cross-File Caching (Week 3) ✅ COMPLETE (2026-02-14)

- [x] **Module Type Exports Cache**
  - [x] Added `ModuleExportsEntry` struct with `symbols: HashMap<String, CachedSymbolInfo>`, `version`, `content_hash`
  - [x] Added `ModuleDependencyGraph` with forward (`dependencies`) and reverse (`dependents`) edge tracking
  - [x] `get_transitive_dependents()` BFS with visited set for cycle-safe cascade invalidation
  - [x] Added `module_exports_cache: HashMap<String, ModuleExportsEntry>` to `DocumentManager`
  - [x] `get_module_exports()` checks cache validity by version; `ensure_module_exports_cached()` lazy-computes via `ensure_type_checked()`
  - [x] `extract_dependencies()` scans AST Import/Export statements to build dependency edges
  - [x] Cascade invalidation in `change()`: when exports change, invalidates all transitive dependents' `module_exports_cache` + `type_check_result`
  - [x] Cascade cleanup in `close()`: removes module from caches, invalidates dependents, clears dependency graph
  - [x] `content_hash()` utility using `std::hash::DefaultHasher` (no new dependencies)
  - **Result**: 8 new unit tests (dependency graph, module exports, content hash), all tests passing, zero clippy warnings
  - Files: `crates/luanext-lsp/src/core/cache.rs`, `crates/luanext-lsp/src/core/document.rs`

- [x] **Symbol Index Incremental Updates**
  - [x] Added `DocumentSymbolSnapshot` with hashed fingerprints for exports, imports, and workspace symbols
  - [x] Replaced `index_exports()`, `index_imports()`, `index_workspace_symbols()` with `_to` variants that write to external collections
  - [x] Rewrote `update_document()` with snapshot + diff: build temp collections → hash into snapshot → compare with old → apply only changed categories
  - [x] `update_document()` now returns `bool` (`exports_changed`) used by `DocumentManager` for cascade invalidation
  - [x] `clear_document()` also removes document snapshot
  - [x] Hashing helpers: `hash_export_info()`, `hash_import_infos()`, `hash_workspace_symbols()` using `DefaultHasher`
  - [x] Expected speedup: Index update from O(n) to O(changed categories) — unchanged exports/imports/workspace symbols are not re-inserted
  - **Result**: 5 new unit tests (no-change, export added, export removed, body-only change, snapshot cleanup), all tests passing
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

- [ ] String pattern matching improvements
- [ ] Type assertions with runtime checks

### Optimizer O2/O3 Passes

- [ ] Implement remaining O2 passes (function inlining, loop optimization, etc.)
- [ ] Implement O3 passes (aggressive inlining, devirtualization, etc.)

### Error Messages

- [ ] Improve type mismatch error messages with suggestions
- [ ] Add "did you mean?" suggestions for typos
- [ ] Better error recovery in parser

### Testing/Benchmarking Lua

- [ ] Consider a testing strategy for Lua code that results from the compilation process
- [ ] Consider a benchmarking strategy for Lua code that results from the compilation process
