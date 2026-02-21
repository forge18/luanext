# Modules

Import and export syntax, re-exports, and type-only imports/exports.

## Overview

LuaNext has a TypeScript-inspired module system. Each file is a module. Symbols are private by default — only exported symbols are accessible from other modules.

**Parser**: `crates/luanext-parser/src/ast/statement.rs` (`ImportDeclaration`, `ExportDeclaration`)
**Codegen**: `crates/luanext-core/src/codegen/modules.rs`

## Imports

```rust
struct ImportDeclaration<'arena> {
    clause: ImportClause<'arena>,
    source: String,           // module path
    span: Span,
}
```

### Import Clause Types

```rust
enum ImportClause<'arena> {
    Default(Ident),                              // import x from "mod"
    Named(&'arena [ImportSpecifier]),             // import { x, y } from "mod"
    Namespace(Ident),                            // import * as mod from "mod"
    TypeOnly(&'arena [ImportSpecifier]),          // import type { T } from "mod"
    Mixed { default: Ident, named: &'arena [ImportSpecifier] },
                                                 // import x, { y } from "mod"
}
```

### Import Specifier

```rust
struct ImportSpecifier {
    imported: Ident,       // original name in source module
    local: Option<Ident>,  // local alias (if renamed)
    span: Span,
}
```

### Syntax Examples

```lua
import defaultExport from "./module"
import { foo, bar as baz } from "./module"
import * as utils from "./utils"
import type { MyType, Config } from "./types"
import defaultExport, { named1, named2 } from "./module"
```

### Module Path Syntax

- Relative: `"./sibling"`, `"../parent"`, `"./sub/deep"`
- Package: `"some-package"`
- Standard library: `"@std/math"`

## Exports

```rust
struct ExportDeclaration<'arena> {
    kind: ExportKind<'arena>,
    span: Span,
}

enum ExportKind<'arena> {
    Declaration(&'arena Statement<'arena>),     // export function foo() {}
    Named {                                      // export { x, y }
        specifiers: &'arena [ExportSpecifier],
        source: Option<String>,                  // re-export source
        is_type_only: bool,
    },
    Default(&'arena Expression<'arena>),         // export default expr
    All {                                         // export * from "mod"
        source: String,
        is_type_only: bool,
    },
}

struct ExportSpecifier {
    local: Ident,            // local name
    exported: Option<Ident>, // external name (if renamed)
    span: Span,
}
```

### Syntax Examples

```lua
-- Export declaration
export function greet(name: string): string { ... }
export class MyClass { ... }
export const PI: number = 3.14

-- Named exports
export { foo, bar }
export { internal as external }

-- Default export
export default createApp()

-- Re-exports
export { name } from "./other"
export * from "./utils"
export type { MyType } from "./types"
```

## Type-Only Imports and Exports

Type-only imports are erased during codegen — they produce no Lua output.

```lua
import type { Config, Options } from "./config"
export type { Result, Error }
```

### Validation

- Type-only imports cannot be used as runtime values
- Attempting to call a type-only imported function produces an error: `RuntimeImportOfTypeOnly`
- Re-exports preserve the type-only flag from the source

`[NOTE: BEHAVIOR]` Codegen emits an empty body for type-only imports (no `require()` call generated).

## Re-exports

Re-exports make symbols from one module available through another:

```lua
-- Named re-export
export { helper } from "./internal"

-- Wildcard re-export
export * from "./utils"

-- Type-only re-export
export type { Config } from "./config"
```

### Re-export Resolution

Re-exports are resolved with:

- **Cycle detection** via `HashSet` of visited modules
- **Depth limit** of 10 hops in the re-export chain
- **Type-only validation**: type-only re-exports cannot be imported as values

Error types: `CircularReExport`, `ReExportChainTooDeep`, `TypeOnlyReExportAsValue`

`[NOTE: BEHAVIOR]` The codegen for `generate_re_export()` adds symbols to `self.exports` — this was a critical bug fix (previously re-exported symbols were not added to the exports table).

## Codegen: Lua Output

### Require Mode (default)

Each import becomes a `require()` call:

```lua
local module = require("./module")
local foo = module.foo
local bar = module.bar
```

Exports are returned as a table:

```lua
local _M = {}
_M.greet = greet
_M.MyClass = MyClass
return _M
```

### Bundle Mode

All modules in a single file with an internal `__require` mechanism.

## Namespaces

```rust
struct NamespaceDeclaration {
    path: Vec<Ident>,
    span: Span,
}
```

File-based namespace declarations: `namespace App.Models`

Sets the namespace prefix for the file's exports.

## Cross-References

- [Module Resolution](module-resolution.md) — path resolution, dependency graph, module registry
- [AST](../language/ast.md) — ImportDeclaration, ExportDeclaration nodes
- [Codegen Statements](../compiler/codegen-statements.md) — import/export code generation
- [Optimizer LTO](../compiler/optimizer-lto.md) — dead import/export elimination
