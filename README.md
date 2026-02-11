# LuaNext

[![Pre-Alpha](https://img.shields.io/badge/status-pre--alpha-red.svg)](https://github.com/forge18/luanext)

A typed superset of Lua with gradual typing, inspired by TypeScript's approach to JavaScript.

## Overview

LuaNext aims to bring static type checking to Lua while maintaining its simplicity and allowing gradual adoption. Write type-safe Lua code that compiles to plain Lua, with zero runtime overhead.

**Warning: This project is in pre-alpha development. The code is not ready for production use.**

## Features (Planned)

- **Gradual Typing** - Add types at your own pace, from none to full coverage
- **TypeScript-Inspired** - Familiar syntax for developers coming from TypeScript
- **Zero Runtime Cost** - Types are erased at compile time
- **Lua Compatibility** - Compiles to clean, readable Lua (5.1-5.4)
- **Rich Type System** - Interfaces, unions, generics, and more
- **Optional Features** - Enable OOP, functional programming, or decorators as needed
- **LSP Support** - Full language server with autocomplete, diagnostics, and more
- **Multi-File Compilation** - Compile entire projects with automatic dependency ordering
- **Circular Dependency Detection** - Catch import cycles before compilation
- **Glob Pattern Support** - Use wildcards like `src/**/*.luax` to select files

## Project Status

**Current Status: Pre-Alpha**

This project is in early development. Core infrastructure is being built.

## Architecture

LuaNext is built in Rust with a focus on modularity and testability:

```
luanext/
├── crates/
│   ├── luanext-core/    # Compiler core (lexer, parser, type checker, codegen)
│   ├── luanext-cli/     # Command-line interface
│   └── luanext-lsp/     # Language Server Protocol implementation
```

**Design Principles:**
- Dependency injection for testability
- Trait-based abstractions for flexibility
- Comprehensive error handling with detailed diagnostics
- Zero runtime overhead - all types erased at compile time

## Development

### Prerequisites

- Rust 1.70+
- Cargo

### Building

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Run linter
cargo clippy

# Format code
cargo fmt
```

### Running Tests

```bash
# Run all tests
cargo test --all

# Run tests for specific crate
cargo test -p luanext-core
```

## Contributing

LuaNext is under active development. Contributions are welcome!

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Please ensure:
- All tests pass (`cargo test`)
- Code is formatted (`cargo fmt`)
- No clippy warnings (`cargo clippy`)

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- Inspired by [TypeScript](https://www.typescriptlang.org/) and [Teal](https://github.com/teal-language/tl)
- Built with [Rust](https://www.rust-lang.org/) for performance and safety
- Uses [Tower LSP](https://github.com/ebkalderon/tower-lsp) for language server implementation

---

**Status:** 🔧 Pre-Alpha - Under Development

Built with ❤️ by the LuaNext team

