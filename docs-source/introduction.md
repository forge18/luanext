# Introduction

LuaNext is a typed superset of Lua with gradual typing, inspired by TypeScript's approach to JavaScript. It brings static type checking to Lua while maintaining its simplicity and allowing gradual adoption. Write type-safe Lua code that compiles to plain Lua, with zero runtime overhead.

## What is LuaNext?

LuaNext extends Lua with optional type annotations, interfaces, generics, and modern language features. The type system is erased at compile time, producing clean, readable Lua code that runs on any Lua 5.1–5.4 interpreter.

## Key Features

- **Gradual Typing** — Add types at your own pace, from none to full coverage
- **Zero Runtime Cost** — Types are erased at compile time
- **Lua Compatibility** — Compiles to clean, readable Lua (5.1-5.4)
- **Rich Type System** — Interfaces, unions, generics, conditional types, and more
- **Optional Features** — Enable OOP, functional programming, or decorators as needed
- **LSP Support** — Full language server with autocomplete, diagnostics, and more
- **Multi-File Compilation** — Compile entire projects with automatic dependency ordering
- **Circular Dependency Detection** — Catch import cycles before compilation
- **Incremental Compilation** — Fast rebuilds with intelligent caching
- **Source Maps** — Debug compiled Lua with original LuaNext source
- **Optional Optimizations** — Enable performance optimizations like constant folding, dead code elimination, and tail call optimization

## Example

LuaNext code:

```lua
-- Variable declarations with types
const PI: number = 3.14159
local radius: number = 5

-- Interfaces for table shapes
interface Point {
    x: number,
    y: number
}

-- Functions with type signatures
function calculateArea(r: number): number
    return PI * r * r
end

-- Type inference
const area = calculateArea(radius)  -- inferred as number

print("Area:", area)
```

Compiles to clean Lua:

```lua
local PI = 3.14159
local radius = 5

local function calculateArea(r)
    return PI * r * r
end

local area = calculateArea(radius)

print("Area:", area)
```

## Why LuaNext?

**Catch bugs before runtime** — The type checker finds type errors, nil access, and mismatched function signatures before your code runs.

**Better IDE support** — Autocomplete, go-to-definition, inline documentation, and refactoring powered by the LSP.

**Gradual adoption** — Start with plain Lua and add types incrementally. No need to rewrite everything at once.

**No runtime cost** — Types are stripped during compilation. Your Lua code runs exactly as fast as hand-written Lua.

**Modern language features** — Classes, interfaces, enums, pattern matching, decorators, generics, and more.

## Optional Optimizations

LuaNext includes optional compiler optimizations that can improve runtime performance while maintaining correctness. These are disabled by default but can be enabled per-project:

**Constant Folding & Propagation** — Evaluate constant expressions at compile time and propagate known values through the code.

**Dead Code Elimination** — Remove unreachable code, unused variables, and redundant operations.

**Tail Call Optimization** — Transform tail-recursive calls into loops to prevent stack overflow.

**Table Preallocation** — Pre-size tables when the number of elements is known at compile time.

**Rich Enum Optimization** — Optimize enum value representations and eliminate unnecessary type checks.

**Function Inlining** — Inline small functions to reduce call overhead (configurable threshold).

These optimizations are carefully designed to preserve your code's semantics while improving performance. Enable them with the `--optimize` flag or configure individual passes in your project settings.

## The LuaNext Ecosystem

LuaNext is part of a broader ecosystem of Lua/LuaNext developer tools. Each tool works standalone on plain Lua but gains capabilities when used together.

### Developer Tools

**[Depot](https://github.com/forge18/depot)** — Lua Package Manager

- Local, project-scoped dependency management (like npm/cargo for Lua)
- Lua version manager (manage 5.1, 5.3, 5.4 installs)
- Lockfile support for reproducible builds
- SemVer version resolution
- LuaRocks compatible upstream package source
- Supply chain security (BLAKE3 checksums, sandboxed builds)
- Watch mode for automatic rebuilds on file changes

**[Lintomatic](https://github.com/forge18/lintomatic)** — Linter for Lua and LuaNext

- 100+ built-in rules across 8 categories
- Rust backend for performance
- Lua plugin API for custom rules
- Type-aware linting when used with LuaNext
- Auto-fix support
- Works with both plain Lua and LuaNext

**[Canary](https://github.com/forge18/canary)** — Test Framework

- Rust-powered test runner with Vitest-inspired API
- Built-in mocking and spying
- Snapshot testing
- Coverage collection
- Parallel test execution
- Works with both plain Lua and LuaNext

**[Wayfinder](https://github.com/forge18/wayfinder)** — Debugger (DAP)

- Debug Adapter Protocol implementation for Lua
- Works with PUC Lua 5.1-5.4 and LuaNext
- Source map support for LuaNext debugging
- Breakpoints, stepping, stack inspection, variable watches
- IDE integration (VS Code, Neovim, JetBrains)
- Works with both plain Lua and LuaNext

### Core Libraries

These standalone crates are available for building ecosystem tools:

**[luanext-parser](https://github.com/forge18/luanext-parser)** — Parser Library

- Lexer and parser for LuaNext (.luax) files
- Arena-allocated AST (`bumpalo::Bump`)
- String interner for efficient symbol handling
- Full error recovery and span tracking

**[luanext-typechecker](https://github.com/forge18/luanext-typechecker)** — Type Checker Library

- Complete type checking for LuaNext
- Type inference, generics, conditional types
- Diagnostic reporting
- Module resolution and dependency tracking

**[luanext-sourcemap](https://github.com/forge18/luanext-sourcemap)** — Source Map Library

- Source map generation and consumption
- Maps compiled Lua positions back to LuaNext source
- Used by Wayfinder for debug source mapping

**[luanext-lsp](https://github.com/forge18/luanext-lsp)** — Language Server Protocol

- Full LSP implementation for LuaNext
- Completion, hover, go-to-definition, references, rename, formatting, diagnostics
- Semantic tokens, inlay hints, code actions
- Powers the VS Code extension

These crates are published separately so ecosystem tools can depend on them without pulling in the full compiler.

## Project Status

**Current Status: Pre-Alpha**

The compiler is actively in development with core features implemented. We're refining the type system, improving performance, and expanding the ecosystem.

**Next Steps: Beta Release**

- Type system refinements and edge case handling
- Performance optimization and profiling
- Expanded documentation and examples
- Community feedback and testing

## Community

- **GitHub:** [github.com/forge18/luanext](https://github.com/forge18/luanext)
- **Issues:** [Report bugs and feature requests](https://github.com/forge18/luanext/issues)
- **Discussions:** [Ask questions and share ideas](https://github.com/forge18/luanext/discussions)

## License

LuaNext is open source software released under the MIT License.
