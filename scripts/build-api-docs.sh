#!/usr/bin/env bash
#
# Build Rust API documentation (rustdoc) and copy to mdBook docs site
#
# This script generates Rust API documentation for all workspace crates
# and copies the output to the mdBook documentation site at docs-site/book/api/
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo -e "${YELLOW}Building Rust API documentation...${NC}"

# Generate rustdoc for all workspace crates
# --no-deps: Don't generate docs for dependencies
# --all-features: Enable all feature flags
# --workspace: Document all crates in the workspace
cd "$PROJECT_ROOT"
cargo doc --no-deps --all-features --workspace

echo -e "${GREEN}✓ Rustdoc generated${NC}"

# Create output directory
API_OUTPUT_DIR="$PROJECT_ROOT/docs-site/book/api"
mkdir -p "$API_OUTPUT_DIR"

echo -e "${YELLOW}Copying documentation to $API_OUTPUT_DIR...${NC}"

# Copy all generated docs to the mdBook directory
# This includes all HTML, CSS, JavaScript, and other static files
if [ -d "$PROJECT_ROOT/target/doc" ]; then
    # Remove old docs if they exist
    if [ -d "$API_OUTPUT_DIR" ]; then
        rm -rf "$API_OUTPUT_DIR"/*
    fi

    # Copy new docs
    cp -r "$PROJECT_ROOT/target/doc"/* "$API_OUTPUT_DIR/"
    echo -e "${GREEN}✓ Documentation copied${NC}"
else
    echo -e "${RED}Error: No documentation generated at $PROJECT_ROOT/target/doc${NC}"
    exit 1
fi

# Verify the copy was successful
if [ -f "$API_OUTPUT_DIR/index.html" ]; then
    echo -e "${GREEN}✓ API documentation ready at $API_OUTPUT_DIR${NC}"
    echo -e "${GREEN}Total files: $(find "$API_OUTPUT_DIR" -type f | wc -l)${NC}"
else
    echo -e "${RED}Error: Documentation index not found${NC}"
    exit 1
fi

echo -e "${GREEN}Done!${NC}"
