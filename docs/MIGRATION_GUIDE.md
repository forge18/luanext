# Migration Guide: Enhanced Cross-File Type Support

This guide helps you migrate existing LuaNext code to use the new cross-file type resolution features added in February 2026.

## Table of Contents

- [Overview](#overview)
- [Breaking Changes](#breaking-changes)
- [New Features](#new-features)
- [Migration Steps](#migration-steps)
- [Common Patterns](#common-patterns)
- [Performance Considerations](#performance-considerations)
- [Troubleshooting](#troubleshooting)

## Overview

The enhanced cross-file type support brings TypeScript-like module features to LuaNext:

- ✅ **Type-only imports** - Import types without runtime overhead
- ✅ **Circular type dependencies** - Mutually referential types across files
- ✅ **Re-exports** - Re-export symbols from other modules
- ✅ **Export all** - Wildcard re-exports (`export *`)
- ✅ **Better error messages** - Clear suggestions for fixing circular dependencies

**Backwards Compatibility**: All existing code continues to work. These features are opt-in enhancements.

## Breaking Changes

### Circular Value Dependencies Now Error

**Before**: Circular value imports would cause runtime errors (stack overflow or undefined behavior)

**After**: Compiler detects and rejects circular value imports at compile-time

**Example**:

```typescript
// module-a.luax
import { funcB } from './module-b'  // Error: Circular dependency
export function funcA() { return funcB() }

// module-b.luax
import { funcA } from './module-a'  // Error: Circular dependency
export function funcB() { return funcA() }
```

**Error Message**:

```
Error: Circular value dependency detected:
  module-a.luax → module-b.luax → module-a.luax

This creates a runtime deadlock. Consider using type-only imports:
  import type { Foo } from './module'
```

**Migration**: Use type-only imports or dependency injection to break the cycle (see examples below)

## New Features

### 1. Type-Only Imports

Import types without generating runtime code:

```typescript
// Before (generates require() call even if only used for types)
import { User } from './types'

function processUser(user: User): void {
  print(user.name)
}

// After (no require() call - types erased at runtime)
import type { User } from './types'

function processUser(user: User): void {
  print(user.name)
}
```

**Benefits**:
- Smaller bundle size (no unnecessary require() calls)
- Faster startup (fewer modules loaded)
- Enables circular type dependencies

### 2. Circular Type Dependencies

Mutually referential types now work across files:

```typescript
// user.luax
import type { Post } from './post'

export interface User {
  id: number
  posts: Post[]
}

// post.luax
import type { User } from './user'

export interface Post {
  id: number
  author: User
}
```

This was not possible before and would cause a compilation error.

### 3. Re-Exports

Create barrel exports to simplify imports:

```typescript
// database/index.luax
export { User, findUser } from './user-model'
export { Post, findPost } from './post-model'

// app.luax - cleaner imports
import { User, Post, findUser, findPost } from './database'
```

### 4. Export All (Wildcard)

Re-export all symbols from a module:

```typescript
// utils/index.luax
export * from './string-utils'
export * from './number-utils'
export * from './array-utils'
```

## Migration Steps

### Step 1: Identify Type-Only Imports

Find imports that are only used in type positions:

**Pattern to look for**:

```typescript
import { Foo } from './module'

// Foo only used here (type position):
function bar(x: Foo): void { }

// Not used here (value position):
const instance = new Foo()  // This would be a value usage
```

**Migration**:

```typescript
// Change to type-only import
import type { Foo } from './module'
```

**Benefits**:
- Reduces bundle size
- Enables circular type dependencies if needed

### Step 2: Fix Circular Dependencies

If you have circular dependencies causing runtime errors, convert to type-only imports:

**Before** (runtime error):

```typescript
// service-a.luax
import { ServiceB } from './service-b'

export class ServiceA {
  constructor(private b: ServiceB) {}
}

// service-b.luax
import { ServiceA } from './service-a'  // Circular!

export class ServiceB {
  constructor(private a: ServiceA) {}
}
```

**After** (works correctly):

```typescript
// service-a.luax
import type { ServiceB } from './service-b'  // Type-only

export class ServiceA {
  constructor(private b: ServiceB) {}
}

// service-b.luax
import type { ServiceA } from './service-a'  // Type-only

export class ServiceB {
  constructor(private a: ServiceA) {}
}

// main.luax - create instances without circular dependency
import { ServiceA } from './service-a'
import { ServiceB } from './service-b'

const b = new ServiceB(null)  // Temporary null
const a = new ServiceA(b)
b.a = a  // Wire up after construction
```

### Step 3: Create Barrel Exports

Simplify your import structure using barrel exports:

**Before**:

```typescript
// app.luax
import { User } from './models/user'
import { Post } from './models/post'
import { Comment } from './models/comment'
import { validateUser } from './validators/user-validator'
import { validatePost } from './validators/post-validator'
```

**After**:

```typescript
// models/index.luax
export { User } from './user'
export { Post } from './post'
export { Comment } from './comment'

// validators/index.luax
export { validateUser } from './user-validator'
export { validatePost } from './post-validator'

// app.luax - much cleaner!
import { User, Post, Comment } from './models'
import { validateUser, validatePost } from './validators'
```

### Step 4: Use Export All for Utilities

For utility modules, use `export *`:

**Before**:

```typescript
// utils/index.luax
export { capitalize, trim } from './string-utils'
export { clamp, round } from './number-utils'
export { first, last } from './array-utils'
// Need to update this file every time you add a utility!
```

**After**:

```typescript
// utils/index.luax
export * from './string-utils'
export * from './number-utils'
export * from './array-utils'
// Automatically exports everything - no maintenance needed
```

## Common Patterns

### Pattern 1: Type-Only Re-Exports

Separate types from implementations:

```typescript
// api/types.luax
export interface ApiRequest { ... }
export interface ApiResponse { ... }

// api/client.luax
import type { ApiRequest, ApiResponse } from './types'

export class ApiClient {
  async request(req: ApiRequest): Promise<ApiResponse> { ... }
}

// api/index.luax - clean public API
export type { ApiRequest, ApiResponse } from './types'
export { ApiClient } from './client'

// app.luax
import type { ApiRequest, ApiResponse } from './api'
import { ApiClient } from './api'
```

### Pattern 2: Breaking Circular Dependencies

Use interfaces and dependency injection:

```typescript
// Before (circular)
// a.luax
import { B } from './b'
export class A {
  b = new B()
}

// b.luax
import { A } from './a'
export class B {
  a = new A()
}

// After (no circular dependency)
// interfaces.luax
export interface IA { }
export interface IB { }

// a.luax
import type { IB } from './interfaces'
export class A implements IA {
  constructor(public b: IB) {}
}

// b.luax
import type { IA } from './interfaces'
export class B implements IB {
  constructor(public a: IA) {}
}

// main.luax
import { A } from './a'
import { B } from './b'

const a = new A(null)
const b = new B(a)
a.b = b
```

### Pattern 3: Forward Declarations

Use forward declarations for mutual references in one file:

```typescript
// models.luax
// Forward declarations
interface Node {}
interface Edge {}

// Full definitions
interface Node {
  id: number
  edges: Edge[]
}

interface Edge {
  from: Node
  to: Node
}

export { Node, Edge }
```

### Pattern 4: Conditional Imports

Import values only when needed at runtime:

```typescript
// config.luax
export const DEBUG = true

// logger.luax
if DEBUG then
  import { detailedLogger } from './detailed-logger'
  export { detailedLogger as logger }
else
  import { simpleLogger } from './simple-logger'
  export { simpleLogger as logger }
end
```

## Performance Considerations

### Bundle Size Reduction

Type-only imports significantly reduce bundle size:

**Before**:

```typescript
// app.luax
import { User, Post, Comment, validateUser, validatePost } from './api'

// Only using types
function display(user: User, posts: Post[]): void { ... }
```

**Generated Lua** (Before):

```lua
local api = require('./api')  -- Loads entire API module
local User = api.User
local Post = api.Post
local Comment = api.Comment
local validateUser = api.validateUser
local validatePost = api.validatePost
-- Even though we don't use the functions!
```

**After**:

```typescript
// app.luax
import type { User, Post, Comment } from './api'
import { validateUser, validatePost } from './api'

function display(user: User, posts: Post[]): void { ... }
```

**Generated Lua** (After):

```lua
local api = require('./api')
local validateUser = api.validateUser
local validatePost = api.validatePost
-- User, Post, Comment not imported (types erased)
-- Smaller bundle, faster load time
```

### Lazy Loading

Use type-only imports to enable lazy loading:

```typescript
// feature.luax
import type { HeavyModule } from './heavy-module'

let cached: HeavyModule | null = null

export function getHeavy(): HeavyModule {
  if cached == null then
    // Only load when actually needed
    import { HeavyModule } from './heavy-module'
    cached = new HeavyModule()
  end
  return cached
}
```

## Troubleshooting

### Error: Circular Value Dependency

**Problem**: You're importing a value from a module that imports from you

**Solution 1**: Use type-only imports if you only need types

```typescript
// Change this:
import { Foo } from './other'

// To this:
import type { Foo } from './other'
```

**Solution 2**: Extract shared interfaces

```typescript
// Before:
// a.luax
import { B } from './b'
export class A { b: B }

// b.luax
import { A } from './a'
export class B { a: A }

// After:
// types.luax
export interface IA { }
export interface IB { }

// a.luax
import type { IB } from './types'
export class A implements IA { b: IB }

// b.luax
import type { IA } from './types'
export class B implements IB { a: IA }
```

### Error: Export Not Found

**Problem**: Trying to import something that's not exported

**Solution**: Check the export statement in the source module

```typescript
// module.luax
export { foo }  // Forgot to export bar

// app.luax
import { foo, bar } from './module'  // Error: bar not found

// Fix: Add bar to exports
export { foo, bar }
```

### Error: Re-Export Chain Too Deep

**Problem**: Re-export chain exceeds depth limit of 10

**Solution**: Flatten your module structure or use direct imports

```typescript
// Before: a → b → c → d → e → f → g → h → i → j → k (too deep!)

// After: Flatten
// utils/index.luax
export * from './string'
export * from './number'
export * from './array'
// Direct exports, no deep chains
```

### Error: Runtime Import of Type-Only Export

**Problem**: Trying to import an interface/type alias as a value

```typescript
// types.luax
export interface User { }

// app.luax
import { User } from './types'  // Error: User is type-only

const u = new User()  // Can't instantiate an interface
```

**Solution**: Use type-only import or import the implementation

```typescript
// Fix 1: Type-only import
import type { User } from './types'

// Fix 2: Import the class implementation instead
import { UserImpl } from './user-impl'
const u = new UserImpl()
```

## Summary

The enhanced cross-file type support makes LuaNext more powerful and TypeScript-like:

✅ Use `import type` for types to reduce bundle size
✅ Circular type dependencies now work correctly
✅ Use re-exports to create clean module APIs
✅ Use `export *` for utility barrels
✅ Follow the migration steps for a smooth transition

For more details, see:
- [ARCHITECTURE.md](../crates/luanext-typechecker/docs/ARCHITECTURE.md) - Technical implementation details
- [PERFORMANCE_TESTING.md](./PERFORMANCE_TESTING.md) - Performance benchmarks and testing guide
