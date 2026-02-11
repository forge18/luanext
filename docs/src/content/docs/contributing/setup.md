---
title: Getting Started as a Contributor
---

# Getting Started as a Contributor

Welcome to LuaNext! This guide will help you set up your development environment and get started contributing to the project.

## Development Environment Setup

### Prerequisites

- **Rust 1.70+** — Required for building the compiler
- **Cargo** — Rust's package manager (installed with Rust)
- **Git** — Version control
- **mdBook** — Documentation site generator (optional, only needed to build docs locally)

### Quick Start

1. **Clone the repository**
   ```bash
   git clone https://github.com/forge18/luanext.git
   cd luanext
   ```

2. **Install Rust** (if not already installed)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Build the project**
   ```bash
   cargo build --release
   ```

4. **Run tests**
   ```bash
   cargo test --all
   ```

5. **Install the CLI locally** (for testing)
   ```bash
   cargo install --path crates/luanext-cli
   ```

## Development Workflow

### Code Quality

Before committing, ensure code passes all checks:

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all

# Check code coverage
cargo tarpaulin --config-file tarpaulin.toml
```

The repository has pre-commit hooks that enforce these checks automatically.

### Project Structure

```
luanext/
├── crates/
│   ├── luanext-core/       # Lexer, parser, type checker, codegen
│   ├── luanext-cli/        # Command-line interface
│   ├── luanext-lsp/        # Language Server Protocol implementation
│   ├── luanext-parser/     # Parser (separate crate)
│   ├── luanext-typechecker/# Type checker (separate crate)
│   └── ...
├── docs/                   # Technical documentation (architecture, implementation)
├── docs-site/              # User-facing documentation website
│   └── src/
│       ├── getting-started/# Installation, quick-start guides
│       ├── language/       # Language reference
│       ├── guides/         # Tutorials and migration guides
│       └── reference/      # CLI, config, stdlib reference
├── editors/                # Editor integrations (VSCode, etc.)
└── scripts/                # Build and utility scripts
```

## Contributing to Documentation

The project uses a hybrid documentation structure:

### Technical Documentation (`/docs`)

For internal implementation details, architecture, design decisions:

- **Location:** `/docs/` directory
- **Audience:** Contributors and maintainers
- **Examples:** `ARCHITECTURE.md`, `IMPLEMENTATION.md`, design documents
- **Edit directly** when updating technical details

### User-Facing Documentation (`/docs-site/src`)

For guides, tutorials, and language reference:

- **Location:** `/docs-site/src/` directory
- **Audience:** End users
- **Sections:**
  - `getting-started/` — Installation, quick-start, setup guides
  - `language/` — Language reference and features
  - `guides/` — Tutorials and migration guides
  - `reference/` — API reference, CLI, configuration

### Editing Content

1. **For quick-start or language guides:**
   ```bash
   # Edit files in docs-site/src/
   editor docs-site/src/guides/your-guide.md
   ```

2. **For technical docs:**
   ```bash
   # Edit files in docs/
   editor docs/ARCHITECTURE.md
   ```

3. **Adding new pages:**
   - Create the markdown file in the appropriate directory
   - Update `docs-site/src/SUMMARY.md` with the new entry
   - Test locally (see below)

### Updating the Navigation Menu

Edit `/docs-site/src/SUMMARY.md` to add or modify documentation structure:

```markdown
# Summary

[Introduction](introduction.md)

---

# Language

- [Basics](language/basics.md)
- [Your New Guide](language/your-guide.md)

---
```

## Testing Documentation Locally

To preview documentation changes:

1. **Install mdBook** (if not already installed)
   ```bash
   cargo install mdbook mdbook-mermaid mdbook-linkcheck
   ```

2. **Serve documentation locally**
   ```bash
   cd docs-site
   mdbook serve --open
   ```

3. **View in browser** — Opens at `http://localhost:3000`

4. **Live reload** — Changes to markdown files automatically reload in browser

5. **Check for broken links** (before submitting PR)
   ```bash
   cd docs-site
   mdbook-linkcheck
   ```

## Code Submission Guidelines

### Before Submitting a PR

1. **Ensure code is formatted**
   ```bash
   cargo fmt --all
   ```

2. **Pass linter checks**
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```

3. **All tests pass**
   ```bash
   cargo test --all
   ```

4. **Update documentation**
   - If adding a new feature, update language reference or CLI docs
   - If fixing a bug, clarify in commit message
   - Keep README.md in sync with major changes

5. **Documentation links pass validation**
   ```bash
   cd docs-site && mdbook-linkcheck
   ```

### Commit Messages

Use clear, descriptive commit messages:

```
feat: add pattern matching support
fix: resolve type inference issue in generics
docs: clarify module system architecture
refactor: simplify type checking algorithm
test: add coverage for pattern matching
```

### Pull Requests

1. Create a feature branch
2. Make changes and commit
3. Push to your fork
4. Create a pull request with clear description
5. Address feedback from reviewers
6. Squash commits if requested

## Useful Commands

| Command | Purpose |
|---------|---------|
| `cargo build` | Build debug binaries |
| `cargo build --release` | Build optimized release binaries |
| `cargo test --all` | Run all tests |
| `cargo test --all -- --nocapture` | Run tests with output |
| `cargo doc --open` | Build and view API documentation |
| `cargo fmt --all` | Format all code |
| `cargo clippy` | Run linter |
| `mdbook serve --open` | Preview documentation |

## Getting Help

- **Questions?** Open a GitHub issue with the `question` label
- **Bug reports?** Use the `bug` label with a minimal reproduction
- **Feature requests?** Use the `enhancement` label with use cases
- **Documentation improvements?** Submit a PR or open an issue

## Additional Resources

For more details, see:

- **Architecture:** [/docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md)
- **Implementation:** [/docs/IMPLEMENTATION.md](../../docs/IMPLEMENTATION.md)
- **Development Setup:** [/docs/DEVELOPMENT_SETUP.md](../../docs/DEVELOPMENT_SETUP.md)
- **Testing Guide:** [/docs/TESTING.md](../../docs/TESTING.md)
- **Contributing Guidelines:** [CONTRIBUTING.md](../../CONTRIBUTING.md)
