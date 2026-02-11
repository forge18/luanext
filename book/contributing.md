# Contributing to LuaNext

Thank you for your interest in contributing to LuaNext! This guide provides detailed information about our development workflow, code standards, and contribution process.

## Table of Contents

1. [Getting Started](#getting-started)
2. [PR Workflow](#pr-workflow)
3. [Code Review Process](#code-review-process)
4. [Commit Message Conventions](#commit-message-conventions)
5. [Branch Naming Conventions](#branch-naming-conventions)
6. [Pre-commit Hook Requirements](#pre-commit-hook-requirements)
7. [Issue Triage and Bug Reporting](#issue-triage-and-bug-reporting)
8. [Documentation Requirements](#documentation-requirements)

---

## Getting Started

### Prerequisites

- **Rust 1.70+** - Install via [rustup](https://rustup.rs/)
- **Cargo** - Comes with Rust
- **Git** - Version control
- **Node.js 16+** - For VS Code extension development (optional)

### Initial Setup

1. **Fork the repository** on GitHub
2. **Clone your fork**:
   ```bash
   git clone https://github.com/yourusername/luanext.git
   cd luanext
   ```

3. **Add upstream remote**:
   ```bash
   git remote add upstream https://github.com/forge18/luanext.git
   ```

4. **Build and test**:
   ```bash
   cargo build
   cargo test --all
   cargo fmt
   cargo clippy -- -D warnings
   ```

---

## PR Workflow

### 1. Fork and Branch

1. **Fork the repository** to your GitHub account
2. **Clone your fork** locally
3. **Create a feature branch** from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```

### 2. Make Changes

1. **Write your code** following our [coding standards](#pre-commit-hook-requirements)
2. **Add tests** for new functionality (target 70%+ coverage)
3. **Update documentation** as needed
4. **Run local checks**:
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test --all
   ```

### 3. Commit Your Changes

Follow our [commit message conventions](#commit-message-conventions):
```bash
git add .
git commit -m "feat: Add awesome new feature"
```

### 4. Push to Your Fork

```bash
git push origin feature/your-feature-name
```

### 5. Create Pull Request

1. Go to the [LuaNext repository](https://github.com/forge18/luanext)
2. Click "New Pull Request"
3. Select your fork and branch
4. Fill out the PR template with:
   - **What**: Brief description of changes
   - **Why**: Motivation and context
   - **How**: Implementation approach
   - **Testing**: How you tested the changes

### 6. Address Review Feedback

- Respond to reviewer comments
- Make requested changes
- Push updates to the same branch
- Re-request review when ready

---

## Code Review Process

All pull requests must pass both automated checks and manual review before merging.

### Automated Checks (CI/CD)

Our CI pipeline runs the following checks on every PR:

1. **Format Check** - `cargo fmt --all -- --check`
   - Ensures code follows Rust formatting standards
   - Must pass before manual review

2. **Clippy Linting** - `cargo clippy --all-targets --all-features -- -D warnings`
   - Enforces Rust best practices
   - No warnings allowed (treat warnings as errors)
   - No suppression via `#[allow(clippy::...)]` (except in `#[cfg(test)]` items)

3. **Test Suite** - `cargo test --all --verbose`
   - All tests must pass
   - Tests run on Ubuntu (Linux)

4. **Build Verification** - `cargo build --release --verbose`
   - Cross-platform builds (Ubuntu, macOS, Windows)
   - Ensures code compiles on all supported platforms

5. **Code Coverage** - `cargo tarpaulin`
   - Minimum 70% line coverage required
   - Coverage must meet or exceed threshold
   - See `tarpaulin.toml` for configuration

### Manual Review Checklist

Reviewers will check for:

- **Code Quality**
  - [ ] Follows Rust idioms and best practices
  - [ ] No unnecessary complexity or premature abstraction
  - [ ] Proper error handling with `Result<T, E>` (no panics)
  - [ ] Trait-based dependency injection for testability

- **Testing**
  - [ ] New features have unit tests (`#[cfg(test)]` modules)
  - [ ] Integration tests added to `tests/` directory where appropriate
  - [ ] Tests use realistic data and test behavior, not implementation
  - [ ] Coverage meets 70% threshold

- **Documentation**
  - [ ] Public APIs have doc comments (///)
  - [ ] README.md updated if needed
  - [ ] Relevant docs in `docs/` updated
  - [ ] TODO.md updated for remaining work

- **Architecture**
  - [ ] Follows existing patterns (see `crates/luanext-lsp/src/message_handler.rs` for DI example)
  - [ ] No breaking changes without discussion
  - [ ] Backward compatibility maintained (unless explicitly not required)

### Review Timeline

- **Initial review**: Within 3-5 business days
- **Follow-up reviews**: Within 1-2 business days
- **Merge**: After approval and all checks pass

---

## Commit Message Conventions

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification with project-specific patterns.

### Format

```
<type>: <subject>

[optional body]

[optional footer]
```

### Types

Based on analysis of git history, we use these commit types:

- **feat**: New feature or enhancement
  - `feat: Add cross-file rename support`
  - `feat: Implement arena pooling for long-lived processes`

- **fix**: Bug fix
  - `fix: Restore arena lifetime management in LSP`
  - `fix: Resolve parser panic on empty input`

- **refactor**: Code restructuring without changing behavior
  - `refactor: Clean up TODO.md and enhance comments`
  - `refactor: Simplify type checker error handling`

- **docs**: Documentation changes
  - `docs: Update TODO.md - mark cache tuning complete`
  - `docs: Add contributing guidelines`

- **test**: Adding or updating tests
  - `test: Add reflection v2 tests for code generation`
  - `test: Enhance method call tests`

- **perf**: Performance improvements
  - `perf: Optimize table preallocation`
  - `perf: Implement parallel parsing with rayon`

- **chore**: Build process, dependencies, tooling
  - `chore: Update dependencies in Cargo.toml`
  - `chore: Bump version to 0.2.0`

### Subject Guidelines

- Use imperative mood ("Add feature" not "Added feature")
- Start with capital letter after type
- No period at the end
- Keep under 72 characters
- Be specific but concise

### Body Guidelines

- Wrap at 72 characters
- Explain **what** and **why**, not **how**
- Use bullet points for multiple changes:
  ```
  feat: Complete rename from TypedLua to LuaNext

  - Updated all use statements across codebase
  - Renamed crate directories and Cargo.toml files
  - Updated repository URLs to luanext-* repos
  - Changed CLI binary name
  ```

### Examples from Project History

**Good commits:**
```
feat: Enhance RichEnumOptimizationPass with enum analysis and optimization

refactor: Update TODO.md and optimize table preallocation and tail call optimization passes

fix: restore arena lifetime management in LSP after reverting bad commits

docs: Mark cache tuning as complete in TODO.md
```

**Update commits** (for submodules/references):
```
Update subproject reference for luanext-lsp

Update binary manifest in typedlua-cli cache
```

---

## Branch Naming Conventions

### Standard Branches

- **main** - Production-ready code (protected)
- **feature/*** - New features
- **fix/*** - Bug fixes
- **refactor/*** - Code refactoring
- **docs/*** - Documentation updates
- **test/*** - Test additions/updates

### Branch Naming Format

```
<type>/<short-description>
```

### Examples

```bash
# Features
feature/arena-allocation
feature/incremental-compilation
feature/cross-file-rename

# Bug fixes
fix/parser-panic
fix/lsp-lifetime-management
fix/cache-serialization

# Refactoring
refactor/di-container
refactor/optimize-passes
refactor/simplify-codegen

# Documentation
docs/contributing-guide
docs/architecture-update

# Tests
test/reflection-v2
test/pattern-matching
```

### Guidelines

- Use lowercase with hyphens
- Keep names short but descriptive
- Reference issue number if applicable: `fix/123-parser-panic`
- Delete branch after PR is merged

---

## Pre-commit Hook Requirements

LuaNext uses a pre-commit hook to enforce code quality standards before commits are created.

### Enforced Checks

The pre-commit hook configuration (`.git/hooks/pre-commit-config.json`) enforces:

1. **Cargo Format** - `cargo fmt --check`
   - Required: Yes
   - Auto-fix: `cargo fmt`
   - Ensures consistent Rust formatting

2. **Cargo Clippy** - `cargo clippy -- -D warnings`
   - Required: No (but strongly recommended)
   - Fails on any warnings
   - No suppression allowed (fix the issue, don't ignore it)

3. **Cargo Test** - `cargo test --quiet`
   - Required: No
   - Runs test suite before commit
   - Helps catch regressions early

### Custom Checks

Additional validation includes:

- **No dbg!() macro** - Prevents debug macros in production code
  - Fails if `dbg!()` found in `*.rs` files (except tests)
  - Error: "Found dbg!() macro - remove before committing"

- **TODO/FIXME warning** - Warns about unresolved comments
  - Non-blocking (doesn't fail commit)
  - Reminds to address or document TODOs

### File Size Limits

- Maximum file size: 5000 KB (5 MB)
- Excludes: `*.lock`, `target/**/*`

### Running Checks Manually

Before committing, you can run these checks manually:

```bash
# Format code
cargo fmt

# Check formatting without modifying
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings

# Run all clippy targets
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --quiet

# Run everything (recommended)
cargo fmt && cargo clippy -- -D warnings && cargo test
```

### Bypassing Hooks (Not Recommended)

While the pre-commit hook can be disabled, this is **strongly discouraged**:

```bash
# Don't do this unless absolutely necessary
git commit --no-verify -m "message"
```

All PRs must pass CI checks, which enforce the same standards.

---

## Issue Triage and Bug Reporting

### Before Opening an Issue

1. **Search existing issues** - Your issue may already be reported
2. **Check documentation** - Review README.md and docs/
3. **Test with latest main** - Bug may already be fixed
4. **Verify it's reproducible** - Ensure consistent behavior

### Bug Report Template

When opening a bug report, include:

```markdown
**Description**
Clear description of the bug

**To Reproduce**
Steps to reproduce the behavior:
1. Create file 'example.luax' with: ...
2. Run command: `luanext example.luax`
3. See error: ...

**Expected Behavior**
What you expected to happen

**Actual Behavior**
What actually happened

**Environment**
- LuaNext version: (e.g., 0.1.0 or commit hash)
- OS: (e.g., Ubuntu 22.04, macOS 13.0, Windows 11)
- Rust version: (run `rustc --version`)

**Additional Context**
- Error messages (full output)
- Minimal code example
- Screenshots (if relevant)
```

### Feature Request Template

```markdown
**Feature Description**
Clear description of the proposed feature

**Use Case**
Why is this feature needed? What problem does it solve?

**Proposed Solution**
How you envision this working

**Alternatives Considered**
Other approaches you've thought about

**Additional Context**
Examples from other languages/tools, if applicable
```

### Issue Labels

Maintainers use these labels for triage:

- **bug** - Something isn't working
- **enhancement** - New feature or request
- **documentation** - Documentation improvements
- **good first issue** - Good for newcomers
- **help wanted** - Extra attention needed
- **question** - Further information requested
- **wontfix** - Will not be worked on

---

## Documentation Requirements

### When to Update Documentation

Update documentation when you:
- Add a new feature
- Change existing behavior
- Fix a bug that affects documented behavior
- Improve performance significantly
- Add new CLI options or configuration

### What to Document

1. **Code Comments**
   - Use `///` for public API documentation
   - Use `//` for implementation notes
   - Explain **why**, not **what** (code shows what)
   - Example:
     ```rust
     /// Performs incremental type checking using cached module data.
     ///
     /// This function checks if the cached type information is still valid
     /// by comparing declaration hashes. If valid, it skips re-typechecking
     /// and reuses the cached result.
     ///
     /// # Arguments
     /// * `module` - The module to type check
     /// * `cache` - The cache manager instance
     ///
     /// # Returns
     /// * `Ok(TypedModule)` - Successfully typed module
     /// * `Err(TypeCheckError)` - Type checking failed
     pub fn check_module_incremental(
         module: &Module,
         cache: &CacheManager,
     ) -> Result<TypedModule, TypeCheckError>
     ```

2. **README.md**
   - Update if adding major features
   - Keep examples current
   - Update feature list
   - Maintain accuracy of project status

3. **docs/ Directory**
   - `docs/ARCHITECTURE.md` - Architecture changes
   - `docs/designs/` - Design documents for major features
   - `docs/SECURITY.md` - Security-related changes
   - Create new docs for complex features

4. **TODO.md**
   - Mark tasks complete
   - Add new tasks discovered during work
   - Keep status current

5. **CHANGELOG.md**
   - Document user-facing changes
   - Follow [Keep a Changelog](https://keepachangelog.com/) format
   - Maintainers typically update this, but you can draft entries

### Documentation Style Guide

- **Use clear, concise language** - Avoid jargon when possible
- **Provide examples** - Show, don't just tell
- **Be accurate** - Test examples before documenting
- **Use proper Markdown** - Headers, code blocks, lists
- **Link related docs** - Help readers find more information

### Example Documentation PR Checklist

- [ ] Added/updated doc comments for public APIs
- [ ] Updated README.md if needed
- [ ] Created/updated design docs for major changes
- [ ] Updated TODO.md to reflect current status
- [ ] Verified all code examples work
- [ ] Checked for broken internal links

---

## Additional Resources

- **Project Guidelines**: See [CLAUDE.md](../CLAUDE.md)
- **Architecture**: See [docs/ARCHITECTURE.md](ARCHITECTURE.md)
- **Language Design**: See [docs/designs/language-spec.md](designs/language-spec.md)
- **Security**: See [docs/SECURITY.md](SECURITY.md)
- **Main Contributing Guide**: See [CONTRIBUTING.md](../CONTRIBUTING.md)

---

## Code of Conduct

- Be respectful and constructive in all interactions
- Focus on what is best for the project and community
- Show empathy towards other contributors
- Accept constructive criticism gracefully
- Give credit where credit is due

---

## Recognition

Contributors are recognized through:
- Git commit history
- Release notes and CHANGELOG.md
- Future CONTRIBUTORS.md file
- GitHub contributor graph

---

## Getting Help

If you have questions:
1. **Check documentation** - README.md, docs/, CONTRIBUTING.md
2. **Search issues** - Someone may have asked before
3. **Open a discussion** - Use GitHub Discussions (if enabled)
4. **Ask in PR comments** - If related to specific code

---

Thank you for contributing to LuaNext! Your contributions help make this project better for everyone.
