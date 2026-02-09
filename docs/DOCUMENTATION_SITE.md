# Documentation Site Implementation Guide

> **STATUS:** All user-facing documentation (Phase 2) is COMPLETE ✅
>
> - 25 content files + SUMMARY.md created in `docs-site/src/`
> - All critical corrections applied (LuaJIT removal, config option fixes, file extension fixes)
> - Ready for mdBook integration (Phase 3 pending)

## Executive Summary

**Tool:** mdBook (Rust-native static site generator)
**Hosting:** GitHub Pages at `https://forge18.github.io/luanext/`
**Architecture:** Hybrid content strategy (technical docs in `/docs/`, user guides in `docs-site/src/`)
**Versioning:** Included in Phase 1 (version selector + automated release deployment)
**Timeline:** 3 weeks (~30-40 hours)
**Cost:** $0/year

### Key Decisions

1. **Hybrid Content Strategy:** Keep technical/contributor docs in `/docs/` as source of truth, create new user-facing guides in `docs-site/src/`. This avoids duplication and preserves Git history.

2. **GitHub Pages Default URL:** Using `forge18.github.io/luanext` initially (no custom domain). Can add custom domain later without breaking links.

3. **Versioning in Phase 1:** Implementing version selector and automated versioned docs workflow from day one to ensure smooth v1.0 launch.

## Context

The LuaNext project currently has comprehensive documentation (23 markdown files across `/docs/`, `/docs/designs/`, and root-level docs like `README.md`, `CONTRIBUTING.md`, etc.) but lacks a user-friendly documentation website. The goal is to create a GitBook-like documentation site that:

1. Provides a clean, searchable web interface for existing documentation
2. Auto-deploys via GitHub Actions whenever documentation changes
3. Integrates with the existing CI/CD pipeline
4. Supports versioning for upcoming v1.0 release
5. Minimizes maintenance overhead and build times

**Why this matters:** A polished documentation site is critical for user adoption and contributor onboarding. Currently, users must navigate markdown files in GitHub, which lacks search, navigation, and professional presentation.

## Why mdBook?

After evaluating alternatives (VitePress, Docusaurus, Sphinx), **mdBook is the optimal choice** because:

- **Rust-native:** Zero Node.js dependency, aligns with project ecosystem
- **Fast:** 2-5 second builds (vs 30-60s for VitePress)
- **Battle-tested:** Used by Rust Book, Cargo Book, rustc dev guide
- **GitBook-like:** Clean UI, sidebar navigation, built-in search, dark mode
- **Low maintenance:** Single `book.toml` config, no framework dependencies
- **Free hosting:** GitHub Pages provides CDN, HTTPS, custom domains at zero cost

## Project Structure

```text
luanext/
├── docs-site/                    # NEW: mdBook documentation site
│   ├── book.toml                # mdBook configuration
│   ├── theme/                   # Custom styling (optional)
│   │   ├── css/custom.css      # Brand colors, fonts
│   │   └── index.hbs           # Custom header/footer, version selector
│   └── src/
│       ├── SUMMARY.md          # Table of contents (defines sidebar)
│       ├── index.md            # Landing page
│       ├── guide/              # User guides (NEW content)
│       │   ├── installation.md
│       │   ├── quick-start.md
│       │   └── cli-reference.md
│       ├── language/           # Language reference (NEW content)
│       │   ├── type-system.md
│       │   ├── syntax.md
│       │   └── oop.md
│       ├── contributing/       # Contributor getting started (NEW content)
│       │   └── setup.md
│       └── api/                # Link to rustdoc
│           └── index.md
├── docs/                       # EXISTING: Technical docs (kept as-is)
│   ├── ARCHITECTURE.md
│   ├── IMPLEMENTATION.md
│   ├── SECURITY.md
│   ├── designs/
│   └── ...
├── scripts/
│   └── build-api-docs.sh       # NEW: Build and integrate rustdoc
└── .github/workflows/
    └── docs.yml                # NEW: Documentation CI/CD
```

## Hybrid Content Strategy

### Technical Docs Remain in `/docs/`

These files stay in `/docs/` as the single source of truth:

- `docs/ARCHITECTURE.md` - Internal architecture
- `docs/IMPLEMENTATION.md` - Implementation details
- `docs/designs/*.md` - Design documents
- `docs/SECURITY.md` - Security documentation
- `docs/BENCHMARKS.md` - Performance benchmarks
- All other technical/contributor-focused documentation

### User-Facing Content in `docs-site/src/`

New content created specifically for end users:

- `src/index.md` - Landing page (extracted from README.md)
- `src/guide/` - User guides (installation, quick-start, configuration, CLI)
- `src/language/` - Language reference (type system, syntax, OOP, functional)
- `src/contributing/` - Contributor getting started (links to `/docs/` for details)
- `src/api/` - Link to rustdoc API documentation

### Benefits

- Single source of truth for technical docs (no duplication)
- Clear separation: users see guides, contributors see architecture
- Preserves Git history and existing contributor workflows
- Reduces maintenance burden (no syncing between two locations)

### Example SUMMARY.md Structure

```markdown
# Summary

[Introduction](index.md)

---

# User Guide
- [Installation](guide/installation.md)
- [Quick Start](guide/quick-start.md)
- [CLI Reference](guide/cli-reference.md)

# Language Reference
- [Type System](language/type-system.md)
- [Syntax](language/syntax.md)
- [OOP Features](language/oop.md)

---

# Technical Documentation
- [Architecture](../docs/ARCHITECTURE.md)
- [Implementation](../docs/IMPLEMENTATION.md)
- [Security](../docs/SECURITY.md)
- [Design Documents](../docs/designs/)

# Contributing
- [Getting Started](contributing/setup.md)
- [Code Style](contributing/code-style.md)

# API Reference
- [Rust API Docs](api/index.md)
```

## Implementation Steps

### Week 1: Foundation & Structure

1. **Install mdBook and plugins locally**

   ```bash
   cargo install mdbook mdbook-mermaid mdbook-linkcheck
   ```

2. **Initialize docs-site structure**

   ```bash
   mdbook init docs-site
   mkdir -p docs-site/src/{guide,language,contributing,api}
   mkdir -p docs-site/theme/css
   ```

3. **Create `book.toml` with full configuration**
   - Metadata (title, authors, description)
   - Search settings (boost-title, limit-results)
   - Theme settings (light/dark mode)
   - Git integration ("Edit this page" links)
   - Preprocessors (mermaid, linkcheck)

4. **Create `SUMMARY.md` with complete outline**
   - Introduction (landing page)
   - User Guide section (installation, quick-start, CLI)
   - Language Reference section (type system, syntax, OOP)
   - Contributing section (setup, links to `/docs/`)
   - API Documentation section (link to rustdoc)
   - External links to technical docs in `/docs/`

5. **Create user-facing content (NEW content, not migrated)**
   - `src/index.md` - Landing page with project overview
   - `src/guide/installation.md` - Installation instructions
   - `src/guide/quick-start.md` - Quick start tutorial
   - `src/language/type-system.md` - Type system overview
   - `src/contributing/setup.md` - Getting started (links to `/docs/ARCHITECTURE.md`)

6. **Set up references to technical docs**
   - Add links in `SUMMARY.md` to `/docs/ARCHITECTURE.md`, `/docs/IMPLEMENTATION.md`, etc.
   - These will render in mdBook via relative path includes

7. **Test local build**

   ```bash
   cd docs-site && mdbook serve --open
   ```

8. **Create `.github/workflows/docs.yml`**
   - Build job (install mdBook, build site, upload artifact)
   - Deploy job (deploy to GitHub Pages on main branch)
   - Validate job (link check on PRs)

### Week 2: Integration & Versioning

1. **Create `scripts/build-api-docs.sh`**
   - Generate rustdoc: `cargo doc --no-deps --all-features --workspace`
   - Copy to `docs-site/book/api/`
   - Create `src/api/index.md` with links to generated docs

2. **Update CI workflow for API docs**
    - Add cargo doc step to build job
    - Copy rustdoc output to book directory

3. **Add custom theme styling**
    - Create `theme/css/custom.css` with LuaNext brand colors
    - Improve code block styling
    - Add callout box styles (warning, info, tip)

4. **Implement version selector UI**
    - Create `theme/index.hbs` with version dropdown HTML
    - Add JavaScript for version switching
    - Style dropdown to match theme

5. **Implement versioned docs workflow**
    - Add `deploy-version` job to `docs.yml` (triggered by release events)
    - Checkout `gh-pages` branch
    - Copy built docs to `v{VERSION}/` directory
    - Generate/update `versions.json`
    - Commit and push to `gh-pages`

6. **Test complete workflow on feature branch**
    - Push to trigger CI build
    - Verify artifact generation
    - Test link checking
    - Validate all paths resolve correctly

### Week 3: Launch & Validation

1. **Enable GitHub Pages**
    - Repository Settings → Pages → Source: "GitHub Actions"
    - Enforce HTTPS (enabled automatically)
    - Wait for first deployment

2. **Run comprehensive link validation**

    ```bash
    cd docs-site && mdbook-linkcheck
    ```

3. **Validate live deployment**
    - All pages load without 404 errors
    - Search returns accurate results
    - Sidebar navigation works correctly
    - Code syntax highlighting works
    - Dark mode toggle functions
    - "Edit this page" links resolve
    - API docs render correctly
    - Mobile responsive design displays properly

4. **Test versioning on pre-release**
    - Create test release tag (v0.9.0-alpha or similar)
    - Verify versioned docs deploy to `/v0.9/`
    - Test version selector switches between latest and v0.9
    - Validate frozen snapshot (no updates on subsequent commits)

5. **Tune search configuration**
    - Test common search queries
    - Adjust boost weights if needed
    - Verify search index size <500KB

6. **Create contributor documentation**
    - Add section to `CONTRIBUTING.md` about documentation updates
    - Document hybrid structure (user guides vs technical docs)
    - Explain when to edit `/docs/` vs `docs-site/src/`

7. **Performance validation**
    - Verify page load <1 second
    - Confirm CI build <2 minutes
    - Check search latency

## mdBook Configuration (`book.toml`)

```toml
[book]
title = "LuaNext Documentation"
authors = ["LuaNext Contributors"]
description = "A typed superset of Lua with gradual typing"
language = "en"
multilingual = false
src = "src"

[build]
build-dir = "book"
create-missing = true

[output.html]
default-theme = "light"
preferred-dark-theme = "navy"
git-repository-url = "https://github.com/forge18/luanext"
git-repository-icon = "fa-github"
edit-url-template = "https://github.com/forge18/luanext/edit/main/docs-site/src/{path}"
site-url = "/luanext/"

[output.html.search]
enable = true
limit-results = 30
teaser-word-count = 30
use-boolean-and = true
boost-title = 2
boost-hierarchy = 1
boost-paragraph = 1
expand = true
heading-split-level = 3

[output.html.fold]
enable = true
level = 1

[output.html.playground]
editable = false
copyable = true
copy-js = true
line-numbers = true

[preprocessor.links]

[preprocessor.index]

# Optional: Mermaid diagrams
[preprocessor.mermaid]
command = "mdbook-mermaid"

# Optional: Link checking
[output.linkcheck]
```

## CI/CD Pipeline (`.github/workflows/docs.yml`)

```yaml
name: Documentation

on:
  push:
    branches: [main]
    paths:
      - 'docs/**'
      - 'docs-site/**'
      - 'README.md'
      - 'CONTRIBUTING.md'
      - '.github/workflows/docs.yml'
  pull_request:
    branches: [main]
    paths:
      - 'docs/**'
      - 'docs-site/**'
  workflow_dispatch:
  release:
    types: [published]

env:
  MDBOOK_VERSION: '0.4.40'
  RUST_BACKTRACE: 1

jobs:
  build:
    name: Build Documentation
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Rust dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install mdBook
        run: |
          mkdir -p ~/mdbook
          curl -sSL https://github.com/rust-lang/mdBook/releases/download/v${MDBOOK_VERSION}/mdbook-v${MDBOOK_VERSION}-x86_64-unknown-linux-gnu.tar.gz | tar -xz -C ~/mdbook
          echo "$HOME/mdbook" >> $GITHUB_PATH

      - name: Install mdBook plugins
        run: |
          cargo install mdbook-mermaid --version 0.14.0
          cargo install mdbook-linkcheck --version 0.7.7

      - name: Build Rust API documentation
        run: |
          cargo doc --no-deps --all-features --workspace
          mkdir -p docs-site/book/api
          cp -r target/doc/* docs-site/book/api/

      - name: Build mdBook
        run: |
          cd docs-site
          mdbook build

      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: docs-site/book

  deploy:
    name: Deploy to GitHub Pages
    if: github.ref == 'refs/heads/main' && github.event_name == 'push'
    needs: build
    runs-on: ubuntu-latest
    permissions:
      pages: write
      id-token: write
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4

  validate:
    name: Validate Documentation
    if: github.event_name == 'pull_request'
    needs: build
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install mdBook
        run: |
          mkdir -p ~/mdbook
          curl -sSL https://github.com/rust-lang/mdBook/releases/download/v${MDBOOK_VERSION}/mdbook-v${MDBOOK_VERSION}-x86_64-unknown-linux-gnu.tar.gz | tar -xz -C ~/mdbook
          echo "$HOME/mdbook" >> $GITHUB_PATH

      - name: Check for broken links
        run: |
          cd docs-site
          mdbook-linkcheck
```

## Versioned Documentation

### URL Structure

- `https://forge18.github.io/luanext/` → Latest (main branch, always current)
- `https://forge18.github.io/luanext/v1.0/` → v1.0 release (frozen snapshot)
- `https://forge18.github.io/luanext/v0.9/` → v0.9 release (frozen snapshot)

### Workflow

On release publication:

1. CI builds documentation for the release tag
2. Copies built site to `v{VERSION}/` directory in `gh-pages` branch
3. Updates `versions.json` with available versions
4. Version selector dropdown allows switching between versions

### Version Selector UI

Add to `theme/index.hbs`:

```html
<div class="version-selector">
  <select id="version-picker" onchange="switchVersion(this.value)">
    <option value="latest">Latest</option>
    <option value="v1.0">v1.0</option>
  </select>
</div>

<script>
function switchVersion(version) {
  const base = window.location.origin + '/luanext/';
  window.location.href = base + (version === 'latest' ? '' : version + '/');
}
</script>
```

## Maintenance & Contributor Workflow

### For Technical Documentation

1. Edit files in `/docs/` (architecture, implementation details, design docs)
2. mdBook automatically includes these via relative path references
3. No manual syncing required

### For User-Facing Guides

1. Edit files in `docs-site/src/` (guides, tutorials, language reference)
2. CI automatically rebuilds docs-site
3. Preview available in PR checks
4. Merging to `main` deploys to production

### Linking Between Sections

- User guides link to technical docs: `[Architecture](../../docs/ARCHITECTURE.md)`
- Technical docs can reference user guides: `[Type System Guide](../docs-site/src/language/type-system.md)`

## Search Features

mdBook includes built-in client-side search powered by elasticlunr.js:

- No backend required
- Fast client-side indexing
- Works offline
- Searches titles, headings, body text
- Keyboard navigation (Ctrl+S / Cmd+S)
- Configurable boost weights for titles/headings

## Success Criteria

### Phase 1 Completion

- [ ] Documentation site live at `https://forge18.github.io/luanext/`
- [ ] User-facing guides created in `docs-site/src/`
- [ ] Technical docs remain in `/docs/` and properly linked
- [ ] Zero broken links (validated by mdbook-linkcheck)
- [ ] Search functional and accurate
- [ ] CI/CD pipeline deploys automatically on push to main
- [ ] Build time <2 minutes
- [ ] Mobile-responsive design
- [ ] API docs integrated (rustdoc linked from site)
- [ ] Version selector UI implemented
- [ ] Versioned docs workflow tested and functional

### Post-Launch Metrics

- Page load time <1 second (First Contentful Paint)
- Search index <500KB
- No 404 errors on any internal links
- CI builds passing consistently

## Cost & Performance

### Infrastructure Costs

- **GitHub Pages:** $0/year (free for public repos)
- **mdBook:** $0 (open source)
- **Custom domain (optional):** $12/year
- **Total:** $0/year

### Build Performance

- mdBook build: ~2-5 seconds
- Rust API docs: ~30-60 seconds (with cache)
- Total CI time: ~1-2 minutes

## Alternative Considered: VitePress

**Why not VitePress:**

- Requires Node.js dependency in CI (adds 30-60s to builds)
- More complex configuration (Vue components, Vite config)
- Framework lock-in (Vue ecosystem)
- Heavier bundle size for end users
- Less aligned with Rust project ecosystem

**When to reconsider:** If you need SSR, API routes, or Vue component interactivity (not needed for documentation).

## References

- [mdBook Documentation](https://rust-lang.github.io/mdBook/)
- [The Rust Book](https://doc.rust-lang.org/book/) (mdBook example)
- [Cargo Book](https://doc.rust-lang.org/cargo/) (mdBook example)
- [GitHub Pages with Actions](https://docs.github.com/en/pages/getting-started-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site#publishing-with-a-custom-github-actions-workflow)
