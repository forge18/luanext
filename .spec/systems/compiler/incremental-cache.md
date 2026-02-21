# Incremental Cache

CacheManager, manifest, serializable types, and invalidation.

## Overview

The compilation cache stores type-checked module data to disk, enabling incremental compilation. Unchanged modules skip parsing, type checking, and codegen entirely.

**Files**: `crates/luanext-core/src/cache/`

## Cache Location

```text
.luanext-cache/
    manifest.bin           -- CacheManifest (binary serialization)
    modules/
        <hash>.bin         -- CachedModule per file
```

## Cache Version

```rust
const CACHE_VERSION: u32 = 2;
```

Version 2 added `serializable_exports` field. Cache files from older versions are automatically invalidated.

`[NOTE: BEHAVIOR]` When adding new fields to `CachedModule`, use `#[serde(default)]` on `Option` fields for backward compatibility with old cache files. Bump `CACHE_VERSION` for structural changes.

## CacheManager

**File**: `manager.rs`

Coordinates all caching operations:

| Method | Purpose |
| ------ | ------- |
| `load_manifest()` | Read manifest from disk |
| `save_manifest()` | Write manifest to disk |
| `is_valid(path, hash)` | Check if cached module is still valid |
| `store_module(module)` | Write a CachedModule to cache |
| `load_module(path)` | Read a CachedModule from cache |

## CacheManifest

**File**: `manifest.rs`

Tracks all cached modules and their metadata:

```rust
struct CacheManifest {
    version: u32,
    entries: HashMap<PathBuf, CacheEntry>,
}

struct CacheEntry {
    hash: Blake3Hash,
    timestamp: SystemTime,
    module_path: PathBuf,
}
```

## CachedModule

**File**: `module.rs`

```rust
struct CachedModule {
    path: PathBuf,
    source: String,
    exports: ModuleExports,
    serializable_exports: Option<SerializableModuleExports>,
    hash: Blake3Hash,
    // ... additional metadata
}
```

`CachedModule::new()` takes 6 arguments (expanded from original to include `serializable_exports`).

## SerializableType Hierarchy

**File**: `serializable_types.rs`

The AST `Type<'arena>` uses arena-allocated references (`&'arena`) which cannot be serialized directly. `SerializableType` is an owned parallel hierarchy:

```rust
enum SerializableType {
    Primitive(PrimitiveType),
    Reference { name: String, type_arguments: Option<Vec<SerializableType>> },
    Union(Vec<SerializableType>),
    Intersection(Vec<SerializableType>),
    Object(SerializableObjectType),
    Array(Box<SerializableType>),
    Tuple(Vec<SerializableType>),
    Function(SerializableFunctionType),
    Literal(SerializableLiteral),
    Nullable(Box<SerializableType>),
    // ... other variants
}
```

Key differences from `Type<'arena>`:

- Uses `Vec` instead of `&'arena [T]`
- Uses `Box` instead of `&'arena T`
- Uses `String` instead of `StringId` (interner not available during deserialization)
- Derives `Serialize` and `Deserialize`

## Hash Computation

**File**: `hash.rs`

Uses Blake3 hashing for file content:

```rust
type Blake3Hash = [u8; 32];

fn compute_hash(source: &str) -> Blake3Hash;
fn compute_hash_with_deps(source: &str, deps: &[Blake3Hash]) -> Blake3Hash;
```

The hash includes:
- Source file content
- Dependency hashes (transitive)
- Compiler configuration hash

## Invalidation Engine

**File**: `invalidation.rs`

A cached module is invalidated when:

1. **File changed**: Source hash differs from cached hash
2. **Dependency changed**: Any imported module's hash changed
3. **Config changed**: Compiler options changed (target, strict mode, etc.)
4. **Version mismatch**: `CACHE_VERSION` doesn't match

## Incremental Type Checking

**File**: `crates/luanext-typechecker/src/incremental.rs`

The `IncrementalChecker` wraps the cache system for type checking:

```rust
struct IncrementalChecker {
    cache: CompilationCache,
}

struct CompilationCache {
    entries: HashMap<PathBuf, CacheEntry>,
}
```

### DeclarationHash

Tracks hash of declaration signatures for fine-grained invalidation:

- If a file's implementation changes but its exported API is unchanged, dependents don't need re-checking
- Uses `DeclarationId` hashing for exported symbol signatures

### Integration in CLI

1. Before type checking: check cache validity for each module
2. Cache hit → load `CachedModule`, skip type checking
3. Cache miss → full type check, then store result in cache
4. After all modules checked → save manifest

## StringInterner Serialization

The `StringInterner` is serialized alongside the cache:

```rust
// Save
let strings = interner.to_strings();
// Load
let interner = StringInterner::from_strings(strings);
```

This ensures `StringId` values from cached modules resolve correctly.

## Cross-References

- [Incremental Parsing](incremental-parsing.md) — parser-level incremental support
- [Module Resolution](../features/module-resolution.md) — module registry state
- [CLI](../tooling/cli.md) — cache integration in compilation pipeline
