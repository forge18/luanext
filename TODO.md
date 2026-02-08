# LuaNext TODO

## Current Focus

### Phase 6: Project Rename (TypedLua → LuaNext)

**Goal:** Rename all project references from TypedLua to LuaNext.

**Subtasks:**

#### Phase 6.1: Cargo Workspace & Crate Names ✓ COMPLETED

- [x] Update `Cargo.toml` workspace name and `members` paths
- [x] Update `crates/typedlua-core/Cargo.toml` package name → `luanext-core`
- [x] Update `crates/typedlua-cli/Cargo.toml` package name → `luanext-cli`
- [x] Update `crates/typedlua-runtime/Cargo.toml` package name → `luanext-runtime`
- [x] Update `crates/typedlua-typechecker/Cargo.toml` package name → `luanext-typechecker`
- [x] Update `crates/typedlua-parser/Cargo.toml` package name → `luanext-parser`
- [x] Update `crates/typedlua-lsp/Cargo.toml` package name → `luanext-lsp`
- [x] Update wayfinder, lintomatic, and depot dependencies
- [x] Run `cargo metadata` to verify workspace resolves

#### Phase 6.2: Rust Module & Crate Names ✓ COMPLETED

- [x] Rename `typedlua_core` → `luanext_core` in lib.rs of each crate
- [x] Rename `typedlua_cli` → `luanext_cli` in lib.rs
- [x] Rename `typedlua_runtime` → `luanext_runtime` in lib.rs
- [x] Rename `typedlua_typechecker` → `luanext_typechecker` in lib.rs
- [x] Rename `typedlua_parser` → `luanext_parser` in lib.rs
- [x] Rename `typedlua_lsp` → `luanext_lsp` in lib.rs
- [x] Update all `use` statements across codebase (grep for `typedlua::` and `typedlua_`)
- [x] Update `cargo.toml` dependencies to use new crate names
- [x] Run `cargo check --workspace` to find all remaining references

#### Phase 6.3: CLI Binary Name (IN PROGRESS)

- [x] Rename binary in `crates/typedlua-cli/Cargo.toml`: `[[bin]]` name `typedlua` → `luanext`
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

**Status:** Phase 6.1 and 6.2 complete. Phase 6.3 in progress.
