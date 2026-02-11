# Installation

LuaNext provides pre-built binaries for Windows, macOS, and Linux.

## Download Pre-built Binary

Download the latest release for your platform from the [GitHub Releases](https://github.com/forge18/luanext/releases) page.

### Linux

```bash
# Download the latest release (replace VERSION with actual version, e.g., v1.0.0)
curl -L https://github.com/forge18/luanext/releases/latest/download/luanext-linux-x86_64.tar.gz -o luanext.tar.gz

# Extract
tar -xzf luanext.tar.gz

# Move to PATH
sudo mv luanext /usr/local/bin/

# Verify
luanext --version
```

### macOS

```bash
# Download the latest release (replace VERSION with actual version, e.g., v1.0.0)
curl -L https://github.com/forge18/luanext/releases/latest/download/luanext-macos-x86_64.tar.gz -o luanext.tar.gz

# Extract
tar -xzf luanext.tar.gz

# Move to PATH
sudo mv luanext /usr/local/bin/

# Verify
luanext --version
```

**macOS ARM64 (Apple Silicon):**

```bash
curl -L https://github.com/forge18/luanext/releases/latest/download/luanext-macos-aarch64.tar.gz -o luanext.tar.gz
tar -xzf luanext.tar.gz
sudo mv luanext /usr/local/bin/
```

### Windows

1. Download `luanext-windows-x86_64.zip` from the [Releases page](https://github.com/forge18/luanext/releases/latest)
2. Extract the archive
3. Add the extracted directory to your system PATH:
   - Right-click "This PC" → Properties → Advanced System Settings
   - Click "Environment Variables"
   - Edit the "Path" variable and add the directory containing `luanext.exe`
4. Open a new terminal and verify:

```bash
luanext --version
```

## Build from Source

If pre-built binaries aren't available for your platform, or you want to build from the latest source:

### Prerequisites

- **Rust 1.70 or later** — [Install Rust](https://rustup.rs/)
- **Git** — For cloning the repository

### Build Steps

```bash
# Clone the repository
git clone https://github.com/forge18/luanext
cd luanext

# Build in release mode
cargo build --release

# The binary will be at target/release/luanext
```

### Install to PATH

**Linux / macOS:**

```bash
sudo cp target/release/luanext /usr/local/bin/
```

Or add to PATH temporarily:

```bash
export PATH="$PATH:$(pwd)/target/release"
```

Add to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.) to make it permanent.

**Windows:**

Copy `target/release/luanext.exe` to a directory in your PATH, or add `target\release` to your system PATH environment variable.

## Verify Installation

Check that LuaNext is installed correctly:

```bash
luanext --version
```

You should see output like:

```text
luanext 1.0.0
```

## Install Lua Runtime

LuaNext compiles to Lua, so you need a Lua interpreter to run the compiled code.

### Option 1: System Package Manager

**Ubuntu/Debian:**

```bash
sudo apt install lua5.4
```

**macOS (Homebrew):**

```bash
brew install lua
```

**Windows (Chocolatey):**

```bash
choco install lua
```

**Windows (Scoop):**

```bash
scoop install lua
```

### Option 2: Depot (Recommended)

[Depot](https://github.com/forge18/depot) is a Lua version manager and package manager that makes it easy to install and switch between Lua versions:

```bash
# Install Depot (download from releases or build from source)
# See: https://github.com/forge18/depot

# Install Lua 5.4
depot install lua@5.4
depot use lua@5.4

# Verify
lua -v
```

### Verify Lua Installation

```bash
lua -v
```

You should see output like:

```text
Lua 5.4.7  Copyright (C) 1994-2024 Lua.org, PUC-Rio
```

## Install LSP (Optional)

For IDE support, install the Language Server:

```bash
# If you built from source
cargo install --path crates/luanext-lsp

# Or download the pre-built binary from releases
# Place luanext-lsp in your PATH
```

Then configure your editor (see [Editor Setup](editor-setup.md)).

## Next Steps

- [Quick Start](quick-start.md) — Write your first LuaNext program
- [Editor Setup](editor-setup.md) — Configure VS Code for LuaNext
- [Project Setup](project-setup.md) — Create a multi-file project

## Troubleshooting

### Command Not Found

If `luanext` is not recognized:

1. Ensure the binary is in a directory listed in your PATH:

   ```bash
   echo $PATH  # Linux/macOS
   echo %PATH% # Windows
   ```

2. Verify the binary is executable (Linux/macOS):

   ```bash
   chmod +x /usr/local/bin/luanext
   ```

### Permission Denied (Linux/macOS)

If you get "permission denied" when copying to `/usr/local/bin`:

```bash
sudo cp luanext /usr/local/bin/
```

### macOS Security Warning

On macOS, you may see a security warning when first running the binary. To allow it:

1. Go to System Preferences → Security & Privacy
2. Click "Allow Anyway" next to the blocked message
3. Run `luanext --version` again

Or remove the quarantine attribute:

```bash
xattr -d com.apple.quarantine /usr/local/bin/luanext
```

### Rust Build Errors

If building from source fails:

1. Update Rust to the latest stable version:

   ```bash
   rustup update stable
   ```

2. Clear the build cache and try again:

   ```bash
   cargo clean
   cargo build --release
   ```

### Lua Not Found

After installing Lua, if `lua` command is not found, ensure it's in your PATH or use the full path to the Lua binary.
