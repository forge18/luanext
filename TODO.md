# LuaNext TODO

## Low Priority

### Optimizer O2/O3 Passes

- [ ] Implement remaining O2 passes (function inlining, loop optimization, etc.)
- [ ] Implement O3 passes (aggressive inlining, devirtualization, etc.)

### Error Messages

- [x] Improve type mismatch error messages with suggestions (Infrastructure complete: type_suggestions.rs, type_formatter.rs)
- [x] Add "did you mean?" suggestions for typos (Implemented for undefined variables with fuzzy matching)
- [ ] Better error recovery in parser (ParserError::new() constructor added; next: add suggestion field to ParserError, contextual hints in consume() for missing end/then/do/)/]/})

### Incremental Parsing

- [x] Dirty region detection (`DirtyRegionSet`, `is_statement_clean`) with proper zero-length insertion handling
- [x] `parse_incremental()` with dirty region support: cumulative byte delta adjustment, region-scoped lexing/parsing, clean statement reuse with adjusted byte ranges
- [x] Multi-arena consolidation with GC (max 3 arenas, periodic consolidation every 10 parses)
- [x] Integration tests: 10 realistic edit scenarios (typing, deletion, paste, comment/uncomment, undo/redo, format)
- [ ] LSP integration: wire incremental parsing into document sync handler

### Testing/Benchmarking Lua

- [ ] Consider a testing strategy for Lua code that results from the compilation process
- [ ] Consider a benchmarking strategy for Lua code that results from the compilation process
