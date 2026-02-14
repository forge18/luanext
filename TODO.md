# LuaNext TODO

## Low Priority

### Optimizer O2/O3 Passes

- [ ] Implement remaining O2 passes (function inlining, loop optimization, etc.)
- [ ] Implement O3 passes (aggressive inlining, devirtualization, etc.)

### Error Messages

- [x] Improve type mismatch error messages with suggestions (Infrastructure complete: type_suggestions.rs, type_formatter.rs)
- [x] Add "did you mean?" suggestions for typos (Implemented for undefined variables with fuzzy matching)
- [ ] Better error recovery in parser (Infrastructure ready, integration pending)

### Testing/Benchmarking Lua

- [ ] Consider a testing strategy for Lua code that results from the compilation process
- [ ] Consider a benchmarking strategy for Lua code that results from the compilation process
