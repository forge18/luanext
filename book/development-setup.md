# Development Setup Guide

**Version:** 1.0
**Last Updated:** 2026-02-08

This guide covers everything you need to set up a complete development environment for LuaNext, from initial installation through advanced debugging and profiling workflows.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Clone and Build](#clone-and-build)
- [Running Tests](#running-tests)
- [Debugging Techniques](#debugging-techniques)
- [IDE Setup](#ide-setup)
- [Common Development Tasks](#common-development-tasks)
- [Troubleshooting](#troubleshooting)
- [Running Benchmarks](#running-benchmarks)
- [Profiling with Flamegraph](#profiling-with-flamegraph)
- [Advanced Topics](#advanced-topics)

---

## Prerequisites

### Required Tools

**Rust 1.70 or later:**

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version

# Update Rust if already installed
rustup update
```

**Git:**

```bash
# macOS (via Homebrew)
brew install git

# Ubuntu/Debian
sudo apt-get install git

# Verify installation
git --version
```

### Optional Tools

**For LSP/VS Code Extension Development:**

- Node.js 16+ and npm
- VS Code with rust-analyzer extension

**For Profiling:**

- `cargo-flamegraph` for CPU profiling
- `cargo-tarpaulin` for code coverage
- `criterion` for benchmarking (included in dev dependencies)

---

## Clone and Build

### 1. Clone the Repository

```bash
# Clone via HTTPS
git clone https://github.com/forge18/luanext.git
cd luanext

# Or clone via SSH
git clone git@github.com:forge18/luanext.git
cd luanext
```

### 2. Workspace Structure

LuaNext uses a Cargo workspace with multiple crates:

```
luanext/
├── crates/
│   ├── luanext-core/          # Compiler core (lexer, parser, codegen)
│   ├── luanext-cli/           # Command-line interface
│   ├── luanext-lsp/           # Language Server Protocol
│   ├── luanext-parser/        # Parser (git submodule)
│   ├── luanext-typechecker/   # Type checker (git submodule)
│   ├── luanext-sourcemap/     # Source map generation
│   ├── luanext-runtime/       # Runtime utilities
│   └── luanext-test-helpers/  # Test utilities
├── editors/vscode/            # VS Code extension
└── docs/                      # Documentation
```

### 3. Initialize Submodules

```bash
# Initialize git submodules for parser and typechecker
git submodule update --init --recursive
```

### 4. Build the Project

```bash
# Build all crates in debug mode (faster compilation)
cargo build

# Build in release mode (optimized, slower compilation)
cargo build --release

# Build specific crate
cargo build -p luanext-cli
cargo build -p luanext-core

# Build all crates with all features
cargo build --all-features
```

### 5. Verify Installation

```bash
# Run the CLI in development mode
cargo run -- --version

# Or use the release binary
./target/release/luanext --version
```

---

## Running Tests

### Basic Test Commands

```bash
# Run all tests in workspace
cargo test --all

# Run tests with output (shows println! statements)
cargo test --all -- --nocapture

# Run specific crate tests
cargo test -p luanext-core
cargo test -p luanext-cli
cargo test -p luanext-lsp

# Run specific test by name
cargo test test_lexer_basic
cargo test parse_function

# Run tests matching pattern
cargo test lexer::
cargo test type_checker::
```

### Test Organization

**Unit Tests:**

- Located in same file as code with `#[cfg(test)]`
- Fast, focused on individual functions/modules

**Integration Tests:**

- Located in `crates/*/tests/` directories
- Test public API and cross-module interactions

**Documentation Tests:**

- Embedded in doc comments
- Run with `cargo test --doc`

### Advanced Testing

```bash
# Run tests in parallel (default)
cargo test --all

# Run tests serially (useful for debugging)
cargo test --all -- --test-threads=1

# Run only integration tests
cargo test --test '*'

# Run tests for all features
cargo test --all-features

# Run tests with specific features
cargo test --features "debug"
```

### Test Coverage

Generate coverage reports using `cargo-tarpaulin`:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage for entire workspace
cargo tarpaulin --all-features --workspace

# Generate coverage with configuration file
cargo tarpaulin --config-file tarpaulin.toml --verbose

# Output formats: HTML, XML, LCOV
cargo tarpaulin --out Html --out Xml --out Lcov

# Check coverage threshold (configured at 70%)
cargo tarpaulin --fail-under 70.0
```

Coverage reports are generated in:

- `tarpaulin-report.html` - Interactive HTML report
- `tarpaulin-report.xml` - Cobertura XML format
- `lcov.info` - LCOV format for CI tools

---

## Debugging Techniques

### 1. Logging with `tracing`

LuaNext uses the `tracing` crate for structured logging. Control verbosity with the `RUST_LOG` environment variable.

```bash
# Error-level logging only
RUST_LOG=error cargo run -- file.luax

# Info-level logging (default)
RUST_LOG=info cargo run -- file.luax

# Debug-level logging (detailed)
RUST_LOG=debug cargo run -- file.luax

# Trace-level logging (very detailed)
RUST_LOG=trace cargo run -- file.luax

# Filter by module
RUST_LOG=luanext_core::parser=debug cargo run -- file.luax
RUST_LOG=luanext_cli=trace,luanext_core=debug cargo run

# Filter by crate
RUST_LOG=luanext_cli=debug cargo run -- file.luax
```

**Adding debug logging to code:**

```rust
use tracing::{debug, info, warn, error, trace};

// Different severity levels
trace!("Very detailed trace information");
debug!("Debug information: variable = {}", value);
info!("General information");
warn!("Warning: potential issue");
error!("Error occurred: {}", error_msg);

// Structured logging with fields
debug!(file = %path.display(), line = line_num, "Parsing file");
```

### 2. Debugger Setup (LLDB/GDB)

**macOS (LLDB):**

```bash
# Build with debug symbols
cargo build

# Launch with LLDB
lldb target/debug/luanext

# Set breakpoint
(lldb) breakpoint set --name main
(lldb) breakpoint set --file parser.rs --line 42

# Run with arguments
(lldb) process launch -- file.luax --out-dir dist

# Common commands
(lldb) run                  # Start execution
(lldb) continue             # Continue after breakpoint
(lldb) step                 # Step into
(lldb) next                 # Step over
(lldb) finish               # Step out
(lldb) print variable       # Print variable
(lldb) frame variable       # Print all locals
(lldb) bt                   # Backtrace
```

**Linux (GDB):**

```bash
# Build with debug symbols
cargo build

# Launch with GDB
gdb target/debug/luanext

# Set breakpoint
(gdb) break main
(gdb) break parser.rs:42

# Run with arguments
(gdb) run file.luax --out-dir dist

# Common commands
(gdb) run                   # Start execution
(gdb) continue              # Continue after breakpoint
(gdb) step                  # Step into
(gdb) next                  # Step over
(gdb) finish                # Step out
(gdb) print variable        # Print variable
(gdb) info locals           # Print all locals
(gdb) backtrace             # Backtrace
```

### 3. Debug Builds vs Release Builds

```bash
# Debug build (default)
# - Includes debug symbols
# - No optimizations
# - Fast compilation
# - Slow runtime
cargo build

# Release build
# - Optimized code
# - Strips debug symbols (with strip = true)
# - Slow compilation
# - Fast runtime
cargo build --release

# Release with debug symbols (for profiling)
cargo build --release --profile release-with-debug
```

### 4. CLI Debug Flags

LuaNext CLI includes several debugging options:

```bash
# Show diagnostic codes
luanext file.luax --diagnostics

# Disable optimizations for debugging
luanext file.luax --no-optimize

# Disable specific optimizations
luanext file.luax --no-tree-shake
luanext file.luax --no-scope-hoist

# Disable caching to force full rebuild
luanext file.luax --no-cache

# Enable optimizer profiling
luanext file.luax --optimize --profile-optimizer

# Force full type check (disable incremental)
luanext file.luax --force-full-check
```

### 5. Panic Debugging

```bash
# Set panic to abort (shows better backtraces)
export RUST_BACKTRACE=1
cargo run -- file.luax

# Full backtrace with all frames
export RUST_BACKTRACE=full
cargo run -- file.luax

# Colored output
export RUST_BACKTRACE=1
export CARGO_TERM_COLOR=always
cargo run -- file.luax
```

---

## IDE Setup

### Visual Studio Code

**Recommended Extensions:**

- **rust-analyzer** - LSP client for Rust (best Rust support)
- **CodeLLDB** - Native debugger for Rust
- **Better TOML** - TOML syntax highlighting
- **Markdown All in One** - Documentation editing

**Installation:**

```bash
# Install via VS Code Extensions Marketplace or command line
code --install-extension rust-lang.rust-analyzer
code --install-extension vadimcn.vscode-lldb
code --install-extension tamasfe.even-better-toml
code --install-extension yzhang.markdown-all-in-one
```

**Configuration:**

Create `.vscode/settings.json` in project root:

```json
{
  "rust-analyzer.cargo.allFeatures": true,
  "rust-analyzer.cargo.buildScripts.enable": true,
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": ["--", "-D", "warnings"],
  "rust-analyzer.procMacro.enable": true,
  "editor.formatOnSave": true,
  "editor.rulers": [100],
  "files.trimTrailingWhitespace": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.tabSize": 4
  }
}
```

**Debugging Configuration:**

Create `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug CLI",
      "cargo": {
        "args": ["build", "-p", "luanext-cli"]
      },
      "args": ["test.luax"],
      "cwd": "${workspaceFolder}",
      "env": {
        "RUST_LOG": "debug"
      }
    },
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Tests",
      "cargo": {
        "args": ["test", "--no-run", "--all"]
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

### JetBrains CLion / IntelliJ IDEA

**Setup:**

1. Install CLion or IntelliJ IDEA with Rust plugin
2. Open `luanext` directory
3. CLion will automatically detect Cargo workspace
4. Configure run configurations for tests and CLI

**Run Configuration for CLI:**

- Command: `run`
- Options: `--package luanext-cli`
- Arguments: `file.luax --out-dir dist`
- Environment: `RUST_LOG=debug`

**Run Configuration for Tests:**

- Command: `test`
- Options: `--all`

---

## Common Development Tasks

### Code Formatting

```bash
# Format all code in workspace
cargo fmt

# Check formatting without modifying files
cargo fmt --all -- --check

# Format specific crate
cargo fmt -p luanext-core
```

**Pre-commit Hook:**

LuaNext includes a pre-commit hook configuration that enforces formatting. The hook is currently disabled (`.git/hooks/pre-commit.disabled`) but can be enabled:

```bash
# Enable pre-commit hook
mv .git/hooks/pre-commit.disabled .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

The hook runs:

- `cargo fmt --check` (enforced, fails on formatting issues)
- `cargo clippy -- -D warnings` (optional)
- `cargo test --quiet` (optional)

### Linting with Clippy

```bash
# Run clippy on all crates
cargo clippy --all-targets --all-features -- -D warnings

# Run clippy on specific crate
cargo clippy -p luanext-core -- -D warnings

# Auto-fix some warnings
cargo clippy --fix --all-targets --all-features

# Allow specific warnings temporarily (not recommended)
cargo clippy --all -- -W clippy::pedantic
```

**Clippy Configuration:**

LuaNext enforces `-D warnings` (deny all warnings). Do not use `#[allow(clippy::...)]` attributes unless absolutely necessary. Fix the underlying issue instead.

### Building Documentation

```bash
# Build documentation for all crates
cargo doc --all --no-deps

# Build and open in browser
cargo doc --all --no-deps --open

# Include private items
cargo doc --all --document-private-items

# Build docs with all features
cargo doc --all-features --no-deps --open
```

### Cleaning Build Artifacts

```bash
# Clean all build artifacts
cargo clean

# Clean specific package
cargo clean -p luanext-core

# Remove cache files
rm -rf .luanext-cache/
rm -rf crates/luanext-cli/.luanext-cache/
```

### Working with Git Submodules

```bash
# Update all submodules to latest commit
git submodule update --remote

# Update specific submodule
git submodule update --remote crates/luanext-parser

# Check submodule status
git submodule status

# Pull changes including submodules
git pull --recurse-submodules
```

---

## Troubleshooting

### Build Failures

**Issue: "could not find `luanext-parser` in registry"**

```bash
# Solution: Initialize submodules
git submodule update --init --recursive
cargo clean
cargo build
```

**Issue: "linker `cc` not found"**

```bash
# macOS: Install Xcode Command Line Tools
xcode-select --install

# Ubuntu/Debian: Install build essentials
sudo apt-get install build-essential

# Fedora/RHEL
sudo dnf install gcc
```

**Issue: Out-of-date dependencies**

```bash
# Update dependencies
cargo update

# Force rebuild
cargo clean
cargo build
```

### Test Failures

**Issue: Tests pass locally but fail in CI**

- Check `RUST_LOG` environment variable differences
- Ensure deterministic test data
- Check for race conditions (use `--test-threads=1`)

**Issue: Snapshot tests fail**

LuaNext uses `insta` for snapshot testing:

```bash
# Review and update snapshots
cargo insta review

# Accept all snapshot changes
cargo insta accept

# Reject all snapshot changes
cargo insta reject
```

### Performance Issues

**Issue: Slow compilation**

```bash
# Use faster linker (macOS)
brew install michaeleisel/zld/zld
export RUSTFLAGS="-C link-arg=-fuse-ld=/usr/local/bin/zld"

# Use faster linker (Linux)
sudo apt-get install lld
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"

# Use incremental compilation (enabled by default in debug)
export CARGO_INCREMENTAL=1

# Reduce parallel jobs to avoid memory issues
cargo build -j 2
```

**Issue: Slow test execution**

```bash
# Run tests in parallel (default)
cargo test --all --release

# Profile specific test
RUST_LOG=trace cargo test test_name -- --nocapture
```

---

## Running Benchmarks

LuaNext includes comprehensive benchmarks using the `criterion` crate.

### Basic Benchmark Commands

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench lexer_bench
cargo bench --bench parser_bench
cargo bench --bench type_checker_bench
cargo bench --bench parallel_optimization

# Run specific benchmark
cargo bench synthetic_exprs

# Quick benchmark (fewer samples, faster)
cargo bench -- --quick

# Save baseline for comparison
cargo bench -- --save-baseline main
```

### Benchmark Organization

Benchmarks are located in `crates/*/benches/`:

- `lexer_bench.rs` - Tokenization performance
- `parser_bench.rs` - Parsing performance
- `type_checker_bench.rs` - Type checking performance
- `parallel_optimization.rs` - Parallel compilation benchmarks

### Comparing Benchmark Results

```bash
# Install benchcmp
cargo install benchcmp

# Run baseline
cargo bench -- --save-baseline before

# Make changes and run again
cargo bench -- --save-baseline after

# Compare results
benchcmp before.txt after.txt
```

### Benchmark Results

Results are stored in `target/criterion/`:

- `report/index.html` - Interactive HTML report
- `report/data/` - Raw benchmark data
- `plots/` - Performance plots

Open the HTML report:

```bash
open target/criterion/report/index.html
```

---

## Profiling with Flamegraph

Flame graphs visualize where your program spends time, making hotspots immediately obvious.

### Installation

```bash
# Install cargo-flamegraph
cargo install flamegraph

# macOS: May need to disable System Integrity Protection for profiling
# See: https://github.com/flamegraph-rs/flamegraph#macos

# Linux: Install perf
sudo apt-get install linux-tools-common linux-tools-$(uname -r)
```

### Generating Flame Graphs

```bash
# Profile a benchmark
cargo flamegraph --bench type_checker_bench

# Profile with specific benchmark
cargo flamegraph --bench type_checker_bench -- synthetic_exprs/100

# Profile the CLI
cargo flamegraph -- file.luax --out-dir dist

# Custom output file
cargo flamegraph -o my_profile.svg --bench parser_bench

# Control sampling frequency (higher = more samples = more accurate)
cargo flamegraph --freq 99 --bench type_checker_bench

# Profile with release optimizations
cargo flamegraph --release --bench type_checker_bench
```

### Interpreting Flame Graphs

**Reading the graph:**

- **X-axis:** Alphabetical ordering (not time!)
- **Y-axis:** Stack depth (call chain)
- **Width:** Time spent in function (wider = more time)
- **Colors:** Random or categorical (for visual distinction)

**Common hotspots to look for:**

```
TypeChecker::infer_expression     # Expression type inference
TypeEnvironment::lookup_type      # Type lookups
SymbolTable::lookup               # Symbol table operations
Type::clone                       # Type cloning overhead
GenericInstantiation::instantiate # Generic instantiation
```

### Alternative Profiling Tools

**macOS Instruments:**

```bash
# Build release binary
cargo build --release

# Launch Instruments Time Profiler
instruments -t "Time Profiler" target/release/luanext file.luax

# Or open Instruments.app manually:
# Xcode → Open Developer Tool → Instruments
# Select "Time Profiler" template
# Choose binary: target/release/luanext
```

**Linux perf:**

```bash
# Record profile
perf record -g cargo run --release -- file.luax

# Generate report
perf report

# Convert to flame graph
perf script | stackcollapse-perf | flamegraph > perf.svg
```

**Heap profiling with dhat:**

The `dhat` crate is included in dev dependencies:

```rust
// Add to benchmark or test
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    let _profiler = dhat::Profiler::new_heap();

    // ... code to profile ...

} // Profiler prints report on drop
```

---

## Advanced Topics

### Incremental Compilation Cache

LuaNext caches compilation results for faster rebuilds:

```bash
# Cache location
.luanext-cache/
└── manifest.bin         # Serialized cache manifest

# Disable cache for testing
luanext file.luax --no-cache

# Clear cache
rm -rf .luanext-cache/

# Force full recompilation
luanext file.luax --force-full-check
```

### Parallel Compilation

LuaNext uses `rayon` for parallel compilation:

```bash
# Enable parallel optimization (default)
luanext src/**/*.luax --optimize

# Disable for benchmarking
luanext src/**/*.luax --optimize --no-parallel-optimization

# Control rayon thread pool size
RAYON_NUM_THREADS=4 luanext src/**/*.luax --optimize
```

### Memory Profiling

Track memory allocations and leaks:

```bash
# Using valgrind (Linux)
valgrind --leak-check=full cargo run -- file.luax

# Using heaptrack (Linux)
heaptrack cargo run -- file.luax
heaptrack --analyze heaptrack.*.gz

# Using Instruments (macOS)
# Open Instruments.app → Allocations template
# Profile target/release/luanext
```

### Fuzzing

Fuzz testing to discover crashes and panics:

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run fuzzer (if fuzz targets exist)
cargo fuzz run fuzz_parser

# Run with specific corpus
cargo fuzz run fuzz_parser corpus/
```

### Cross-Compilation

Build for different platforms:

```bash
# Install target
rustup target add x86_64-pc-windows-gnu
rustup target add aarch64-apple-darwin

# Cross-compile
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target aarch64-apple-darwin
```

### LSP Development

When working on the Language Server:

```bash
# Build LSP server
cargo build -p luanext-lsp

# Test LSP server
cargo test -p luanext-lsp

# Run LSP server manually (for debugging)
cargo run -p luanext-lsp

# Rebuild VS Code extension with updated LSP
cd editors/vscode
npm install
npm run compile
code --install-extension luanext-*.vsix --force
```

### Performance Targets

From `crates/luanext-typechecker/docs/PROFILING.md`:

| Operation | Target | Current | Status |
|-----------|--------|---------|--------|
| 100 expressions | < 2ms | ~1.3ms | ✅ |
| 1000 expressions | < 15ms | ~10.7ms | ✅ |
| 5000 expressions | < 80ms | ~55ms | ✅ |
| Symbol lookup | < 100ns | - | ⏳ |
| Type lookup | < 200ns | - | ⏳ |
| Generic instantiation | < 10μs | - | ⏳ |

---

## Additional Resources

**Documentation:**

- [Architecture Guide](ARCHITECTURE.md) - System design and component overview
- [Contributing Guide](../CONTRIBUTING.md) - Contribution workflow and standards
- [Language Specification](designs/language-spec.md) - LuaNext language reference
- [Profiling Guide](../crates/luanext-typechecker/docs/PROFILING.md) - Detailed profiling instructions

**External Resources:**

- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust fundamentals
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) - Practical Rust examples
- [Criterion.rs Guide](https://bheisler.github.io/criterion.rs/book/) - Benchmarking guide
- [tracing Documentation](https://docs.rs/tracing/) - Structured logging

---

## Getting Help

**Questions or issues?**

- Check existing [GitHub Issues](https://github.com/forge18/luanext/issues)
- Review [Contributing Guide](../CONTRIBUTING.md)
- Ask in discussions or open a new issue

**Found a bug?**

1. Search existing issues
2. Create minimal reproduction case
3. Open issue with:
   - LuaNext version (`luanext --version`)
   - Rust version (`rustc --version`)
   - Operating system
   - Steps to reproduce
   - Expected vs actual behavior

---

**Happy developing!**
