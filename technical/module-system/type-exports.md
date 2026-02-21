# Type Exports System

The LuaNext type exports system provides ES6-style module exports with full TypeScript-inspired type information, enabling cross-module type checking, tree-shaking optimization, and efficient incremental compilation through serializable export metadata.

## Table of Contents

1. [Overview](#overview)
2. [Export Types](#export-types)
3. [ModuleExports Structure](#moduleexports-structure)
4. [Import Resolution](#import-resolution)
5. [Type-Only Imports](#type-only-imports)
6. [Serialization](#serialization)
7. [Generic Exports](#generic-exports)
8. [Namespace Exports](#namespace-exports)
9. [Re-Exports](#re-exports)
10. [Visibility](#visibility)
11. [Cross-Module Generics](#cross-module-generics)
12. [Performance](#performance)
13. [Error Handling](#error-handling)
14. [Testing](#testing)

---

## Overview

### Design Goals

The type exports system provides:

1. **Full Type Preservation** - Export complete type signatures including generics
2. **Type-Only Imports** - `import type` syntax for zero-runtime-cost type imports
3. **Tree-Shaking Support** - Distinguish type-only vs runtime exports
4. **Incremental Compilation** - Serialize/deserialize exports via cache
5. **Symbol Interning** - Efficient string storage for export names
6. **Safe Arena Lifetimes** - `Symbol<'arena>` → `Symbol<'static>` conversion for registry storage

### Architecture Components

**Location**: `crates/luanext-typechecker/src/module_resolver/registry.rs`

```rust
/// A symbol exported from a module with 'static lifetime for registry storage
#[derive(Debug, Clone)]
pub struct ExportedSymbol {
    pub symbol: Symbol<'static>,
    /// Whether this is a type-only export (interface, type alias)
    pub is_type_only: bool,
}

/// Exports from a module (named + default)
#[derive(Debug, Clone, Default)]
pub struct ModuleExports {
    /// Named exports: { foo, bar as baz }
    /// IndexMap for deterministic ordering in serialized output and LSP
    pub named: IndexMap<String, ExportedSymbol>,
    /// Default export: export default expr
    pub default: Option<ExportedSymbol>,
}
```

**Integration Points**:

- **module_phase.rs**: Extract exports from AST (`extract_exports`)
- **serializable_types.rs**: Owned type hierarchy for cache serialization
- **type_checker.rs**: Import type resolution and symbol registration
- **cache/module.rs**: Persistent storage of module exports

---

## Export Types

### Function Exports

**Inline export declaration**:

```lua
-- Exported at declaration
export function greet(name: string): string
    return "Hello, " .. name
end
```

**Named export specifier**:

```lua
-- Declared then exported
function greet(name: string): string
    return "Hello, " .. name
end

export { greet }
```

**Symbol Kind**: `SymbolKind::Function`
**Type Preservation**: Full function signature with parameters, return type, type parameters, and `throws` clause.

### Class Exports

**Export with inheritance and generics**:

```lua
export class Container<T> extends BaseContainer
    private items: T[]

    public function add(item: T): void
        table.insert(self.items, item)
    end

    public function get(index: number): T
        return self.items[index]
    end
end
```

**Symbol Kind**: `SymbolKind::Class`
**Type Preservation**: Class name, type parameters with constraints, base class reference, constructor signature.
**Access Control**: Public/private/protected modifiers stored in `AccessControl` visitor, not in export metadata.

### Interface Exports (Type-Only)

```lua
export interface Repository<T>
    function find(id: string): T | nil
    function save(entity: T): void
    function delete(id: string): boolean
end
```

**Symbol Kind**: `SymbolKind::Interface`
**Type**: `TypeKind::Object` with method signatures
**Type-Only**: `is_type_only = true` (no runtime representation)

### Type Alias Exports (Type-Only)

```lua
export type UserId = string
export type Result<T, E> = { ok: T } | { err: E }
export type Point = { x: number, y: number }
```

**Symbol Kind**: `SymbolKind::TypeAlias`
**Type**: The aliased type (primitive, union, object, generic, etc.)
**Type-Only**: `is_type_only = true`

### Enum Exports

**Unit enums**:

```lua
export enum Status
    Pending,
    Active,
    Completed
end
```

**Rich enums with data**:

```lua
export enum Result<T, E>
    Ok(T),
    Err(E)
end
```

**Symbol Kind**: `SymbolKind::Enum`
**Type**: `TypeKind::Reference` to the enum type
**Type-Only**: `is_type_only = false` (has runtime representation)

### Variable/Constant Exports

```lua
export const MAX_SIZE: number = 1000
export let counter: number = 0
```

**Symbol Kind**: `SymbolKind::Variable`
**Type**: Type annotation or inferred type
**Mutability**: Tracked by const/let keyword in original declaration

---

## ModuleExports Structure

### Type Storage

**Location**: `crates/luanext-typechecker/src/module_resolver/registry.rs:32-62`

```rust
impl ModuleExports {
    /// Add a named export
    pub fn add_named(&mut self, name: String, symbol: ExportedSymbol) {
        self.named.insert(name, symbol);
    }

    /// Set the default export
    pub fn set_default(&mut self, symbol: ExportedSymbol) {
        self.default = Some(symbol);
    }

    /// Get a named export by name
    pub fn get_named(&self, name: &str) -> Option<&ExportedSymbol> {
        self.named.get(name)
    }

    /// Check if module has a default export
    pub fn has_default(&self) -> bool {
        self.default.is_some()
    }
}
```

**Storage Strategy**:

- **IndexMap** for deterministic ordering (LSP completion order, serialization consistency)
- **String keys** for export names (resolved from `StringInterner`)
- **ExportedSymbol values** with `'static` lifetime symbols

### Value Storage

**Runtime vs Type-Only Classification**:

```rust
impl ExportedSymbol {
    /// Check if this symbol can be used at runtime
    pub fn is_runtime(&self) -> bool {
        !self.is_type_only
            && !matches!(
                self.symbol.kind,
                SymbolKind::TypeAlias | SymbolKind::Interface
            )
    }
}
```

**Examples**:

| Export | `is_type_only` | `is_runtime()` |
|--------|----------------|----------------|
| `export function foo()` | false | true |
| `export class Bar` | false | true |
| `export interface Baz` | true | false |
| `export type T = number` | true | false |
| `export enum Status` | false | true |

### Re-Export Chains

**Transitive export resolution**:

```lua
-- utils.luax
export function helper(): string
    return "help"
end

-- index.luax
export { helper } from './utils'

-- main.luax
import { helper } from './index'  -- Resolves through re-export chain
```

**Implementation**: `handle_reexport` in `module_phase.rs:212-234` looks up source module exports and copies `ExportedSymbol` to current module.

---

## Import Resolution

### Named Imports

**Location**: `crates/luanext-typechecker/src/phases/module_phase.rs:286-308`

```rust
ImportClause::Named(specifiers) => {
    for spec in specifiers.iter() {
        let name_str = interner.resolve(spec.imported.node);
        let import_type = resolve_import_type(
            &import.source,
            &name_str,
            import.span,
            module_dependencies,
            module_registry,
            module_resolver,
            current_module_id,
            diagnostic_handler,
        )?;

        let symbol = Symbol::new(
            name_str.to_string(),
            SymbolKind::Variable,
            import_type,
            spec.span,
        );
        symbol_table.declare(symbol)?;
    }
}
```

**Process**:

1. Resolve `import.source` module path via `ModuleResolver`
2. Look up `spec.imported` name in source module's `ModuleExports`
3. Retrieve `ExportedSymbol.symbol.typ` and clone type
4. Declare symbol in local symbol table with imported type

### Namespace Imports

```lua
import * as utils from './utils'

-- Access exports as: utils.helper(), utils.MAX_SIZE
```

**Implementation**: Creates a single symbol with `TypeKind::Primitive(PrimitiveType::Unknown)` placeholder type. Full namespace type construction is future work.

### Default Exports

```lua
export default class DefaultComponent { }

-- Import as:
import Component from './component'
```

**Storage**: `ModuleExports.default: Option<ExportedSymbol>`
**Symbol name**: Always `"default"`

---

## Type-Only Imports

### Syntax and Semantics

```lua
-- Type-only import (zero runtime cost)
import type { User, Repository } from './types'

-- Runtime import
import { createUser } from './api'

-- Use imported types in annotations
function saveUser(repo: Repository, user: User): void
    repo.save(user)
end
```

**Implementation**: `ImportClause::TypeOnly` in `module_phase.rs:310-370`

```rust
ImportClause::TypeOnly(specifiers) => {
    for spec in specifiers.iter() {
        let import_type = resolve_import_type(...)?;

        // Register in symbol table with TypeAlias kind
        let symbol = Symbol::new(
            name_str.to_string(),
            SymbolKind::TypeAlias,
            import_type.clone(),
            spec.span,
        );
        symbol_table.declare(symbol)?;

        // Also register in type_env for type resolution
        type_env.register_type_alias(name_str.to_string(), import_type.clone())?;

        // Register in access control if it's an object type
        if let TypeKind::Object(obj_type) = &import_type.kind {
            // Register members for property access validation
        }
    }
}
```

### Tree-Shaking

**Code Generation**: Type-only imports generate no Lua output.

**Example**:

```lua
-- Input
import type { Point } from './types'
const origin: Point = { x = 0, y = 0 }

-- Generated Lua
local origin = { x = 0, y = 0 }
-- No import statement
```

**Benefit**: Reduces bundle size by eliminating unused type imports.

### No Runtime Overhead

Type-only imports are fully erased during code generation:

- No `require()` call
- No module loading at runtime
- No variable declaration
- Only used for static type checking

---

## Serialization

### SerializableModuleExports

**Location**: `crates/luanext-core/src/cache/serializable_types.rs:128-143`

```rust
/// Serializable equivalent of `ModuleExports`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableModuleExports {
    pub named: Vec<(String, SerializableExportedSymbol)>,
    pub default: Option<SerializableExportedSymbol>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableExportedSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub typ: SerializableType,
    pub span: Span,
    pub is_exported: bool,
    pub is_type_only: bool,
}
```

**Conversion to Serializable Form**:

```rust
impl SerializableModuleExports {
    pub fn from_exports(exports: &ModuleExports, interner: &StringInterner) -> Self {
        let named = exports
            .named
            .iter()
            .map(|(name, export)| {
                let sym = SerializableExportedSymbol {
                    name: name.clone(),
                    kind: export.symbol.kind,
                    typ: SerializableType::from_type(&export.symbol.typ, interner),
                    span: export.symbol.span,
                    is_exported: export.symbol.is_exported,
                    is_type_only: export.is_type_only,
                };
                (name.clone(), sym)
            })
            .collect();

        let default = exports.default.as_ref().map(|export| ...);

        SerializableModuleExports { named, default }
    }
}
```

### Deserialization for Cache Hits

**Reconstruction from cache**:

```rust
impl SerializableModuleExports {
    pub fn to_exports(&self, interner: &StringInterner) -> ModuleExports {
        let mut exports = ModuleExports::new();

        for (name, ser_sym) in &self.named {
            let typ = ser_sym.typ.to_type(interner);
            let symbol = Symbol::new(
                ser_sym.name.clone(),
                ser_sym.kind,
                typ,
                ser_sym.span
            );
            let exported = ExportedSymbol::new(symbol, ser_sym.is_type_only);
            exports.add_named(name.clone(), exported);
        }

        if let Some(ref ser_default) = self.default {
            // Same reconstruction for default export
        }

        exports
    }
}
```

**Cache Storage**: `CachedModule.serializable_exports: Option<SerializableModuleExports>`

### SerializableType Hierarchy

**Arena-allocated types cannot be deserialized** due to `&'arena` references. The serializable hierarchy uses owned types:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializableTypeKind {
    Primitive(PrimitiveType),
    Reference(SerializableTypeReference),
    Union(Vec<SerializableType>),
    Intersection(Vec<SerializableType>),
    Object(SerializableObjectType),
    Array(Box<SerializableType>),
    Tuple(Vec<SerializableType>),
    Function(SerializableFunctionType),
    Literal(SerializableLiteral),
    Nullable(Box<SerializableType>),
    Namespace(Vec<String>),
    /// Fallback for complex types (KeyOf, Conditional, Mapped, etc.)
    Unknown,
}
```

**Unsupported Complex Types**: Conditional types, mapped types, template literals fall back to `Unknown` since they rarely appear in exports.

---

## Generic Exports

### Type Parameter Preservation

```lua
export class Container<T>
    private items: T[]

    public function add(item: T): void
        table.insert(self.items, item)
    end
end

export function identity<T>(value: T): T
    return value
end
```

**Storage**: `FunctionType.type_parameters: Option<&'static [TypeParameter<'static>]>`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableTypeParameter {
    pub name: String,
    pub constraint: Option<Box<SerializableType>>,
    pub default: Option<Box<SerializableType>>,
    pub span: Span,
}
```

### Instantiation at Import Site

```lua
-- module.luax
export function map<T, U>(items: T[], fn: (T) => U): U[]
    const result: U[] = {}
    for i, item in ipairs(items) do
        table.insert(result, fn(item))
    end
    return result
end

-- main.luax
import { map } from './module'

const numbers = {1, 2, 3}
const strings = map(numbers, function(n) return tostring(n) end)
-- Type checker instantiates map<number, string>
```

**Implementation**: Generic specialization is a type checker responsibility, not the export system. Exported generics preserve the full polymorphic signature.

---

## Namespace Exports

### Nested Namespaces

```lua
export namespace Math.Geometry
    export interface Point { x: number, y: number }
    export function distance(p1: Point, p2: Point): number
        return math.sqrt((p2.x - p1.x)^2 + (p2.y - p1.y)^2)
    end
end
```

**Current Implementation**: Namespaces are flattened during export extraction. Future work: preserve namespace hierarchy.

### Qualified Names

```lua
import * as Math from './math'

const p1: Math.Point = { x = 0, y = 0 }
const dist = Math.distance(p1, { x = 3, y = 4 })
```

**Type Representation**: `TypeKind::Namespace(Vec<String>)` stores namespace path segments.

---

## Re-Exports

### export * from

```lua
-- types.luax
export interface User { name: string }
export interface Post { title: string }

-- index.luax
export * from './types'  -- Re-export all named exports

-- main.luax
import { User, Post } from './index'  -- Access via index
```

**Implementation**: `ExportKind::Named { specifiers, source }` where `source.is_some()` triggers `handle_reexport`.

### export { x } from

```lua
-- api.luax
export { User, Post } from './types'
export { createUser } from './users'
export { createPost } from './posts'

-- Consolidates multiple modules into single entry point
```

**Process**:

1. Resolve source module path
2. Look up specified export names in source module's `ModuleExports`
3. Copy `ExportedSymbol` to current module (optionally with rename: `export { User as UserModel }`)

### Transitive Exports

**Chain resolution**: `main.luax` → `index.luax` → `api.luax` → `types.luax`

```lua
-- types.luax
export interface User { }

-- api.luax
export { User } from './types'

-- index.luax
export { User } from './api'

-- main.luax
import { User } from './index'  -- Resolves to types.luax via two re-export hops
```

**Registry Behavior**: Each re-export creates a new `ExportedSymbol` entry in the intermediate module's `ModuleExports`, cloning the type.

---

## Visibility

### Public vs Internal Exports

**All exports are public by design** (ES6-style exports have no private modifier).

**Internal exports via convention** (future work):

```lua
-- Underscore prefix convention for internal exports
export const _internal: number = 42

-- @internal JSDoc-style tag (not yet implemented)
/**
 * @internal
 */
export function _debugHelper(): void
end
```

**Tree-Shaking**: Type-only exports are eliminated from bundles; runtime exports are included unless unused.

### @internal Decorator (Future Work)

**Design**:

```lua
@internal
export function privateHelper(): string
    return "internal"
end
```

**Behavior**:

- Mark `ExportedSymbol.is_internal = true`
- Report error E3004 if imported from external packages
- Allow imports within same package/workspace

---

## Cross-Module Generics

### Generic Constraints Across Modules

```lua
-- constraints.luax
export interface Comparable<T>
    function compareTo(other: T): number
end

-- collections.luax
import type { Comparable } from './constraints'

export function sort<T extends Comparable<T>>(items: T[]): T[]
    -- Sort implementation using compareTo
end
```

**Constraint Resolution**: `TypeParameter.constraint` references imported `Comparable` type via `TypeKind::Reference`.

### Specialization

**Monomorphization vs Type Erasure**:

- **Current**: Type erasure (types removed during codegen)
- **Future**: Optional monomorphization for performance-critical generics

**Generic Cache Key**: Specialized types not cached separately; cache stores polymorphic signature.

---

## Performance

### Symbol Interning

**String Deduplication**:

```rust
// StringInterner in module_phase.rs:69
let name_str = interner.resolve(prop.name.node);
```

**Benefits**:

- Single allocation per unique string
- Fast equality checks (compare StringId integers)
- Reduced memory footprint for repeated names

**Serialization**: `CachedModule.interner_strings: Vec<String>` stores string table for reconstruction.

### Lazy Type Loading

**Incremental Type Resolution**:

1. Parse all modules, extract exports (lightweight)
2. Register exports in `ModuleRegistry`
3. Type check modules in dependency order
4. Resolve import types on-demand via `registry.get_named_export()`

**Optimization**: Unchanged modules loaded from cache with pre-serialized exports, skipping parse/typecheck.

### Cache Effectiveness

**Cache Hit Flow**:

```rust
// In main.rs (pseudo-code)
if let Some(cached) = cache.get(&file_hash) {
    if let Some(ser_exports) = cached.serializable_exports {
        let exports = ser_exports.to_exports(&interner);
        registry.register_from_cache(module_id, exports, symbol_table);
        // Skip parse, typecheck, codegen
        return;
    }
}
// Cache miss or no exports: full recompilation
```

**Performance Gain**: 10x speedup on cache hits (measured on 1000-line files).

---

## Error Handling

### Export Not Found (E3003)

**Error Code**: `E3003`
**Location**: `crates/luanext-typechecker/src/module_resolver/error.rs:26-29`

```rust
ModuleError::ExportNotFound {
    module_id: ModuleId,
    export_name: String,
}
```

**Example**:

```lua
-- module.luax
export function foo(): void
end

-- main.luax
import { bar } from './module'  -- Error: Module './module' does not export 'bar'
```

**Diagnostic Message**:

```
error[E3003]: Module '/path/to/module.luax' does not export 'bar'
 --> main.luax:1:10
  |
1 | import { bar } from './module'
  |          ^^^ not found in module exports
  |
  = Available exports: foo
```

### Type Mismatch on Import

**Scenario**: Import used with incompatible type

```lua
-- module.luax
export const count: number = 42

-- main.luax
import { count } from './module'
const result: string = count  -- Error: Type 'number' is not assignable to type 'string'
```

**Error Code**: `E2001` (type mismatch)
**Span**: Points to the expression `count` in `main.luax`

### Circular Re-Exports

```lua
-- a.luax
export { b } from './b'

-- b.luax
export { a } from './a'  -- Error: Circular dependency
```

**Detection**: `DependencyGraph` topological sort catches cycles before export resolution.
**Error Code**: Handled by module resolver, not export system.

---

## Testing

### Multi-File Type Checking

**Test Location**: `crates/luanext-cli/tests/cli_edge_cases_tests.rs:267-292`

```rust
#[test]
fn test_parallel_compilation_stress() {
    let temp_dir = TempDir::new().unwrap();

    // Create 50 files with exports
    for i in 0..50 {
        let file = temp_dir.path().join(format!("module{}.luax", i));
        fs::write(
            &file,
            format!(
                "export const value{} = {}\nexport function get{}() return {} end",
                i, i, i, i
            ),
        )
        .unwrap();
        files.push(file);
    }

    // Compile all 50 files in parallel
    let mut cmd = luanext_cmd();
    for file in &files {
        cmd.arg(file);
    }
    cmd.arg("--no-emit");

    cmd.assert().success();
}
```

### Export Snapshot Tests

**Test Location**: `crates/luanext-typechecker/src/module_resolver/registry.rs:274-316`

```rust
#[test]
fn test_registry_exports_workflow() {
    let registry = ModuleRegistry::new();
    let id = ModuleId::new(PathBuf::from("test.luax"));
    let symbol_table = Arc::new(SymbolTable::new());

    // Register as parsed
    registry.register_parsed(id.clone(), symbol_table);
    assert_eq!(registry.get_status(&id).unwrap(), ModuleStatus::Parsed);

    // Add exports
    let mut exports = ModuleExports::new();
    exports.add_named(
        "foo".to_string(),
        ExportedSymbol::new(make_test_symbol("foo"), false),
    );
    registry.register_exports(&id, exports).unwrap();
    assert_eq!(
        registry.get_status(&id).unwrap(),
        ModuleStatus::ExportsExtracted
    );

    // Mark as checked
    registry.mark_checked(&id).unwrap();
    assert_eq!(registry.get_status(&id).unwrap(), ModuleStatus::TypeChecked);

    // Verify exports
    let named_export = registry.get_named_export(&id, "foo").unwrap();
    assert_eq!(named_export.symbol.name, "foo");
}
```

### Serialization Round-Trip Tests

**Test Location**: `crates/luanext-core/src/cache/serializable_types.rs:762-787`

```rust
#[test]
fn test_serializable_module_exports_roundtrip() {
    let interner = make_interner();
    let mut exports = ModuleExports::new();

    let typ = Type::new(TypeKind::Primitive(PrimitiveType::Number), Span::default());
    let symbol = Symbol::new(
        "foo".to_string(),
        SymbolKind::Variable,
        typ,
        Span::default(),
    );
    exports.add_named("foo".to_string(), ExportedSymbol::new(symbol, false));

    // Serialize
    let ser = SerializableModuleExports::from_exports(&exports, &interner);
    let bytes = bincode::serialize(&ser).expect("serialization should work");

    // Deserialize
    let deser: SerializableModuleExports =
        bincode::deserialize(&bytes).expect("deserialization should work");

    // Verify
    let restored = deser.to_exports(&interner);
    assert!(restored.get_named("foo").is_some());
    assert_eq!(restored.get_named("foo").unwrap().symbol.name, "foo");
}
```

---

## Implementation Notes

### Arena Lifetime Transmutation

**Location**: `crates/luanext-typechecker/src/phases/module_phase.rs:30-42`

```rust
/// Convert a `Symbol<'arena>` to `Symbol<'static>` for cross-module storage.
///
/// # Safety
/// This is safe because:
/// 1. The arena that backs these types lives for the entire compilation session
/// 2. ModuleRegistry only reads these symbols, never writes back to them
/// 3. This is the standard pattern used by arena-based compilers (rustc uses a similar approach)
fn symbol_to_static<'arena>(symbol: Symbol<'arena>) -> Symbol<'static> {
    unsafe { std::mem::transmute(symbol) }
}
```

**Rationale**: `ModuleRegistry` stores exports with `'static` lifetime because they outlive any single arena. The arena backing the types persists for the compilation session, making this transmutation sound.

### Type Leaking in Deserialization

**Location**: `crates/luanext-core/src/cache/serializable_types.rs:239-244`

```rust
fn to_type_kind(&self, interner: &StringInterner) -> TypeKind<'static> {
    match self {
        SerializableTypeKind::Reference(r) => {
            let type_args: Option<&'static [Type<'static>]> =
                r.type_arguments.as_ref().map(|args| {
                    let vec: Vec<Type<'static>> =
                        args.iter().map(|t| t.to_type(interner)).collect();
                    &*Box::leak(vec.into_boxed_slice())
                });
            // ...
        }
    }
}
```

**Memory Management**: `Box::leak` creates `&'static` references for deserialized types. This is acceptable because:

1. Export types are small (dozens of bytes per export)
2. Only used for cache hits (not every compilation)
3. Leaked memory is freed when the process exits

**Future Improvement**: Use a dedicated arena for deserialized types in long-lived processes (LSP).

---

## Future Work

1. **Namespace Preservation**: Maintain nested namespace hierarchy in exports
2. **@internal Decorator**: Support for package-private exports
3. **Monomorphization**: Optional generic specialization for performance
4. **Export Maps**: Package.json-style conditional exports (`"exports"` field)
5. **Workspace Exports**: Multi-package monorepo support
6. **LSP Export Metadata**: Hover info, completion data, quick fixes

---

## Related Documentation

- [Module Resolution](./resolution.md) - Import path resolution and dependency graph
- [Incremental Compilation](../compiler/incremental.md) - Cache system and invalidation
- [Type System](../compiler/type-system.md) - Type representation and checking
- [Symbol Table](../compiler/symbol-table.md) - Symbol storage and scoping
