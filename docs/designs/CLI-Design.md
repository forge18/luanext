# LuaNext CLI Design

**Document Version:** 0.1  
**Last Updated:** 2024-12-31

This document defines the command-line interface for the LuaNext compiler, following TypeScript's `tsc` design for familiarity.

---

## Overview

The LuaNext compiler provides a single command `tl` (short for LuaNext) that mirrors TypeScript's `tsc` command structure.

**Binary name:** `tl`

**Design philosophy:** Maximum compatibility with TypeScript workflows - developers familiar with `tsc` should feel immediately at home.

---

## Basic Usage

### Compile Project

```bash
# Compile using luanext.json in current directory
tl

# Same as above (explicit)
tl --project .

# Use specific config file
tl --project path/to/luanext.json
tl -p ./config/luanext.json

# Also accepts tsconfig.json for familiarity
tl -p tsconfig.json
```

### Compile Specific Files

```bash
# Compile single file
tl main.luax

# Compile multiple files
tl src/main.luax src/utils.luax lib/helper.luax

# Compile with glob patterns
tl src/**/*.luax
```

### Initialize Project

```bash
# Create luanext.json with defaults
tl --init

# Creates:
# {
#   "compilerOptions": {
#     "target": "lua5.4",
#     "outDir": "./dist",
#     "sourceMap": true,
#     "strictNullChecks": true,
#     "enableOOP": true,
#     "enableFP": true,
#     "enableDecorators": true,
#     "allowNonLuaNext": true
#   },
#   "include": ["src/**/*"],
#   "exclude": ["node_modules", "dist"]
# }
```

---

[continuing in next message due to length...]

**Document Version:** 0.1  
**Last Updated:** 2024-12-31
