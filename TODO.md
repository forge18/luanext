# TypedLua TODO

## Current Focus

### Phase 6: Project Rename (TypedLua → LuaNext)

**Goal:** Rename all project references from TypedLua to LuaNext.

**Subtasks:**

#### Phase 6.1: Cargo Workspace & Crate Names

- [ ] Update `Cargo.toml` workspace name and `members` paths
- [ ] Update `crates/typedlua-core/Cargo.toml` package name → `luanext-core`
- [ ] Update `crates/typedlua-cli/Cargo.toml` package name → `luanext-cli`
- [ ] Update `crates/typedlua-runtime/Cargo.toml` package name → `luanext-runtime`
- [ ] Update `crates/typedlua-typechecker/Cargo.toml` package name → `luanext-typechecker`
- [ ] Update `crates/typedlua-parser/Cargo.toml` package name → `luanext-parser`
- [ ] Update `crates/typedlua-lsp/Cargo.toml` package name → `luanext-lsp`
- [ ] Run `cargo metadata` to verify workspace resolves

#### Phase 6.2: Rust Module & Crate Names

- [ ] Rename `typedlua_core` → `luanext_core` in lib.rs of each crate
- [ ] Rename `typedlua_cli` → `luanext_cli` in lib.rs
- [ ] Rename `typedlua_runtime` → `luanext_runtime` in lib.rs
- [ ] Rename `typedlua_typechecker` → `luanext_typechecker` in lib.rs
- [ ] Rename `typedlua_parser` → `luanext_parser` in lib.rs
- [ ] Rename `typedlua_lsp` → `luanext_lsp` in lib.rs
- [ ] Update all `use` statements across codebase (grep for `typedlua::`)
- [ ] Update `cargo.toml` dependencies to use new crate names
- [ ] Run `cargo check --workspace` to find all remaining references

#### Phase 6.3: CLI Binary Name

- [ ] Rename binary in `crates/typedlua-cli/Cargo.toml`: `[[bin]]` name `typedlua` → `luanext`
- [ ] Update scripts that invoke `typedlua` command
- [ ] Update VSCode extension to spawn `luanext` instead of `typedlua`
- [ ] Update CI/CD workflows that use CLI

#### Phase 6.4: VSCode Extension Rename

- [ ] Rename extension in `editors/vscode/package.json` name/displayName
- [ ] Update extension ID from `typedlua` to `luanext`
- [ ] Update README.md in editors/vscode
- [ ] Update marketplace descriptions

#### Phase 6.5: Documentation Updates

- [ ] Update main README.md title and references
- [ ] Update `docs/README.md`
- [ ] Update `docs/ARCHITECTURE.md`
- [ ] Rename `docs/designs/TypedLua-Design.md` → `LuaNext-Design.md`
- [ ] Update all design docs with new name
- [ ] Update `CONTRIBUTING.md`
- [ ] Update `CHANGELOG.md` header

#### Phase 6.6: Source Code References

- [ ] Search for string literals "TypedLua" in source code
- [ ] Update welcome messages, error messages, --help output
- [ ] Update internal constants/enum variants if any
- [ ] Update comments that reference TypedLua

#### Phase 6.7: GitHub & Publishing

- [ ] Rename GitHub repository from `typedlua` to `luanext`
- [ ] Update package.json version for VSCode extension

#### Phase 6.8: Verification

- [ ] Run `cargo build --release` for all crates
- [ ] Run `cargo test --workspace`
- [ ] Test CLI: `luanext --version`, `luanext --help`
- [ ] Test VSCode extension loads correctly
- [ ] Test LSP functionality
- [ ] Update any local development instructions

**Status:** Not started. Requires coordination with crates.io publishing and GitHub repo rename.
