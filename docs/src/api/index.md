# Rust API Documentation

LuaNext provides comprehensive API documentation for Rust developers integrating the compiler into their projects.

## Available Crates

### Core Compiler

#### [luanext-core](./luanext_core/)
The main compiler implementation including:
- **Lexer** — Tokenization and scanning
- **Parser** — Syntax parsing and AST generation
- **Type Checker** — Type inference and validation
- **Code Generator** — Lua code generation

Use this crate when building tools that need full compilation capabilities.

```rust
use luanext_core::{Lexer, Parser, TypeChecker, CodeGenerator};

let source = "local x: number = 42";
let tokens = Lexer::new(source).tokenize()?;
let ast = Parser::new(tokens).parse()?;
let checked = TypeChecker::new().check(&ast)?;
let lua_code = CodeGenerator::new().generate(&checked)?;
```

#### [luanext-parser](./luanext_parser/)
Standalone parser for parsing LuaNext syntax without type checking.

Use this for:
- Syntax validation without compilation
- Building IDE features
- Creating custom analyses

```rust
use luanext_parser::{Lexer, Parser};

let source = "function greet(name: string): string
    return 'Hello, ' .. name
end";
let ast = Parser::parse(source)?;
```

#### [luanext-typechecker](./luanext_typechecker/)
Standalone type checker for analyzing LuaNext types independently.

Use this for:
- Type analysis tools
- LSP type information
- Static analysis

### Command-Line Interface

#### [luanext-cli](./luanext_cli/)
The `luanext` command-line compiler tool.

For end users, prefer using the CLI directly. For library integration, use `luanext-core`.

### Language Server Protocol

#### [luanext-lsp](./luanext_lsp/)
Complete LSP implementation providing:
- Code completion
- Type information on hover
- Diagnostics and error reporting
- Go to definition
- References finding
- Symbol renaming

Use this to integrate LuaNext support into any editor via LSP.

### Utilities

#### [luanext-sourcemap](./luanext_sourcemap/)
Source map generation and management for debugging compiled Lua code.

Use this to map stack traces back to original LuaNext source lines.

#### [luanext-test-helpers](./luanext_test_helpers/)
Testing utilities for LuaNext development.

This crate is primarily for internal use in LuaNext development.

## Integration Examples

### Using the Compiler as a Library

```rust
use luanext_core::{Lexer, Parser, TypeChecker, CodeGenerator, Config};

fn compile_luanext(source: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Tokenize
    let tokens = Lexer::new(source).tokenize()?;

    // Parse
    let ast = Parser::new(tokens).parse()?;

    // Type check
    let checked = TypeChecker::new().check(&ast)?;

    // Generate Lua code
    let lua_code = CodeGenerator::new().generate(&checked)?;

    Ok(lua_code)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let luanext_code = r#"
        interface User {
            name: string,
            age: number
        }

        function createUser(name: string, age: number): User
            return { name = name, age = age }
        end
    "#;

    let lua_code = compile_luanext(luanext_code)?;
    println!("{}", lua_code);
    Ok(())
}
```

### Parsing Without Type Checking

```rust
use luanext_parser::{Lexer, Parser};

fn check_syntax(source: &str) -> bool {
    match Lexer::new(source).tokenize() {
        Ok(tokens) => Parser::new(tokens).parse().is_ok(),
        Err(_) => false,
    }
}
```

### Using the LSP Server

For editor integration, use the LSP server directly:

```bash
luanext-lsp --stdio
```

The LSP server communicates via stdin/stdout and is compatible with any editor that supports the Language Server Protocol.

## Documentation

Each crate has detailed documentation accessible via:

```bash
cargo doc --open
```

This opens the full API documentation in your browser.

## Feature Flags

Some crates support optional features:

```toml
[dependencies]
luanext-core = { version = "0.1", features = ["sourcemaps", "optimizations"] }
```

Check individual crate documentation for available features.

## Versioning

LuaNext follows semantic versioning (MAJOR.MINOR.PATCH):

- **MAJOR** — Breaking API changes
- **MINOR** — New features (backward compatible)
- **PATCH** — Bug fixes

Check crate-specific documentation for deprecation notices.

## Getting Help

- **API Questions?** Check crate documentation via `cargo doc`
- **Issues?** Report on [GitHub](https://github.com/forge18/luanext/issues)
- **Contributing?** See [CONTRIBUTING.md](../../CONTRIBUTING.md)
