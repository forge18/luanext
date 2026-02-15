# LuaNext TODO

## Low Priority

### Optimizer O2/O3 Passes

#### Missing O2 Passes (Moderate Optimizations)

- [ ] Jump threading - optimize conditional branches with known values
- [ ] Common subexpression elimination (CSE) - eliminate duplicate computations
- [ ] Copy propagation - replace variable uses with their values
- [ ] Peephole optimization - small local code improvements
- [ ] Branch prediction hints - annotate likely/unlikely branches for Lua VM
- [ ] Sparse conditional constant propagation (SCCP) - combine constant folding with dead code elimination

#### Missing O3 Passes (Aggressive Optimizations)

- [ ] Loop unrolling - duplicate loop bodies for small iteration counts
- [ ] Loop fusion - merge adjacent loops with same iteration space
- [ ] Loop fission/distribution - split loops to improve cache locality
- [ ] Function cloning for specialization - duplicate functions for different call contexts
- [ ] Escape analysis - stack-allocate tables that don't escape
- [ ] Interprocedural constant propagation - propagate constants across function boundaries
- [ ] Scalar replacement of aggregates - replace table accesses with local variables

#### Advanced Infrastructure (Required for Some O2/O3 Passes)

- [ ] Control Flow Graph (CFG) construction
  - Required for: jump threading, SCCP, advanced dead code elimination
  - Implementation: Build basic blocks and edges from AST

- [ ] Dominance analysis
  - Required for: advanced loop optimizations, SSA construction
  - Implementation: Compute dominator tree from CFG

- [ ] Static Single Assignment (SSA) form
  - Required for: aggressive constant propagation, copy propagation, CSE
  - Implementation: Insert φ-functions at join points

- [ ] Alias analysis
  - Required for: escape analysis, scalar replacement
  - Implementation: Track which expressions may alias same memory

- [ ] Side-effect analysis
  - Required for: function cloning, interprocedural optimizations
  - Implementation: Track which functions have observable side effects

#### Performance/Profiling Guided Optimizations

- [ ] Profile-Guided Optimization (PGO)
  - Collect runtime profiling data
  - Use hot path information to guide inlining/specialization decisions

- [ ] Link-Time Optimization (LTO)
  - Cross-module optimizations using cached type information
  - Already have infrastructure via `CacheManager` and `ModuleRegistry`

### Testing/Benchmarking Lua

- [ ] Consider a testing strategy for Lua code that results from the compilation process
- [ ] Consider a benchmarking strategy for Lua code that results from the compilation process
