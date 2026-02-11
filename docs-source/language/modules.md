# Modules

LuaNext provides a modern module system with ES6-style imports and exports. Each file is a module, and you can import/export values, types, and declarations between files.

## Syntax

### Imports

```lua
-- Default import
import defaultExport from "./module"

-- Named imports
import {export1, export2} from "./module"

-- Renamed imports
import {export1 as alias1, export2 as alias2} from "./module"

-- Namespace import
import * as name from "./module"

-- Type-only imports
import type {Type1, Type2} from "./module"

-- Mixed import
import defaultExport, {named1, named2} from "./module"
```

### Exports

```lua
-- Export declaration
export const x: number = 42
export function greet(name: string): string
    return "Hello, " .. name
end
export class User
    -- ...
end

-- Named exports
const x = 10
const y = 20
export {x, y}

-- Renamed exports
export {x as publicX, y as publicY}

-- Default export
export default function main(): void
    print("Main function")
end

-- Re-exports
export {foo, bar} from "./other"
export * from "./other"
```

## Examples

### Basic Module

**math.luax:**

```lua
export function add(a: number, b: number): number
    return a + b
end

export function subtract(a: number, b: number): number
    return a - b
end

export const PI: number = 3.14159
```

**main.luax:**

```lua
import {add, subtract, PI} from "./math"

print(add(5, 3))        -- 8
print(subtract(10, 4))  -- 6
print(PI)               -- 3.14159
```

Compiles to:

**math.lua:**

```lua
local M = {}

function M.add(a, b)
    return a + b
end

function M.subtract(a, b)
    return a - b
end

M.PI = 3.14159

return M
```

**main.lua:**

```lua
local math = require("./math")

print(math.add(5, 3))
print(math.subtract(10, 4))
print(math.PI)
```

### Default Export

**logger.luax:**

```lua
class Logger
    function log(message: string): void
        print("[LOG] " .. message)
    end

    function error(message: string): void
        print("[ERROR] " .. message)
    end
end

export default Logger
```

**main.luax:**

```lua
import Logger from "./logger"

const logger = Logger.new()
logger:log("Application started")
```

### Named Exports

**config.luax:**

```lua
export const API_URL: string = "https://api.example.com"
export const TIMEOUT: number = 5000
export const MAX_RETRIES: number = 3

export function getConfig(): {url: string, timeout: number}
    return {
        url = API_URL,
        timeout = TIMEOUT
    }
end
```

**main.luax:**

```lua
import {API_URL, TIMEOUT, getConfig} from "./config"

print(API_URL)     -- https://api.example.com
print(TIMEOUT)     -- 5000

const config = getConfig()
```

### Renamed Imports

```lua
import {add as sum, subtract as diff} from "./math"

print(sum(5, 3))   -- 8
print(diff(10, 4)) -- 6
```

### Namespace Import

Import all exports under a single namespace:

```lua
import * as math from "./math"

print(math.add(5, 3))
print(math.subtract(10, 4))
print(math.PI)
```

### Type-Only Imports

Import only types (no runtime code generated):

**types.luax:**

```lua
export interface User
    id: string
    name: string
    email: string
end

export type Status = "active" | "inactive"

export const DEFAULT_STATUS: Status = "active"
```

**main.luax:**

```lua
import type {User, Status} from "./types"
import {DEFAULT_STATUS} from "./types"

const user: User = {
    id = "user-123",
    name = "Alice",
    email = "alice@example.com"
}

const status: Status = DEFAULT_STATUS
```

Compiles to (type imports are erased):

```lua
local types = require("./types")

local user = {
    id = "user-123",
    name = "Alice",
    email = "alice@example.com"
}

local status = types.DEFAULT_STATUS
```

### Mixed Imports

Combine default and named imports:

**utils.luax:**

```lua
export function helper1(): void
    print("Helper 1")
end

export function helper2(): void
    print("Helper 2")
end

export default function main(): void
    print("Main utility")
end
```

**main.luax:**

```lua
import mainUtil, {helper1, helper2} from "./utils"

mainUtil()    -- Main utility
helper1()     -- Helper 1
helper2()     -- Helper 2
```

### Re-Exports

Re-export from another module:

**api/users.luax:**

```lua
export function getUser(id: string): User
    -- ...
end

export function createUser(data: UserData): User
    -- ...
end
```

**api/posts.luax:**

```lua
export function getPost(id: string): Post
    -- ...
end
```

**api/index.luax:**

```lua
-- Re-export everything from users and posts
export * from "./users"
export * from "./posts"

-- Or re-export specific items
export {getUser, createUser} from "./users"
export {getPost} from "./posts"
```

**main.luax:**

```lua
import {getUser, getPost} from "./api"

const user = getUser("user-123")
const post = getPost("post-456")
```

### Exporting Types

Export type declarations:

**types.luax:**

```lua
export interface Point
    x: number
    y: number
end

export type Vector = Point

export type Result<T> = {success: true, value: T} | {success: false, error: string}
```

**main.luax:**

```lua
import type {Point, Vector, Result} from "./types"

const p: Point = {x = 1, y = 2}
const v: Vector = {x = 3, y = 4}
const result: Result<number> = {success = true, value = 42}
```

### File Namespaces

Declare a namespace for the entire file:

**math.luax:**

```lua
namespace Math

export function add(a: number, b: number): number
    return a + b
end

export function subtract(a: number, b: number): number
    return a - b
end

export const PI: number = 3.14159
```

**main.luax:**

```lua
import {add, PI} from "./math"

-- Or use namespace import
import * as Math from "./math"

print(Math.add(5, 3))
print(Math.PI)
```

### Circular Dependencies

LuaNext detects circular dependencies at compile time:

**a.luax:**

```lua
import {foo} from "./b"

export function bar(): void
    print("bar")
    foo()
end
```

**b.luax:**

```lua
import {bar} from "./a"

export function foo(): void
    print("foo")
    bar()
end
```

```
Error: Circular dependency detected:
  a.luax -> b.luax -> a.luax
```

#### Breaking Circular Dependencies with Type-Only Imports

Type-only imports don't create runtime dependencies:

**a.luax:**

```lua
import type {BType} from "./b"
import {bFunction} from "./b"

export interface AType
    value: number
end

export function aFunction(b: BType): void
    print(b.name)
end
```

**b.luax:**

```lua
import type {AType} from "./a"  -- Type-only, no circular dependency

export interface BType
    name: string
end

export function bFunction(a: AType): void
    print(a.value)
end
```

This works because type-only imports are erased at runtime.

## Details

### Module Resolution

LuaNext resolves modules using the following rules:

1. **Relative paths** — Start with `./` or `../`
   - `./module` → same directory
   - `../module` → parent directory
   - `../../module` → grandparent directory

2. **Absolute paths** — Resolved from project root or node_modules

3. **File extensions** — `.luax` is implicit
   - `import {x} from "./module"` → looks for `module.luax`

### Export Restrictions

- Each file can have at most one default export
- Cannot export the same name twice
- Cannot re-export and locally export the same name

```lua
-- Error: Duplicate export
export const x = 1
export const x = 2  -- ❌ Error

-- Error: Duplicate default export
export default function foo() end
export default function bar() end  -- ❌ Error
```

### Import Restrictions

- Cannot import from a module that doesn't export the name
- Cannot import the same name twice in one import statement

```lua
-- Error: Module doesn't export 'unknownExport'
import {unknownExport} from "./module"  -- ❌ Error

-- Error: Duplicate import
import {x, x} from "./module"  -- ❌ Error
```

### Type-Only Imports

Type-only imports:

- Import types, interfaces, and type aliases
- Generate no runtime code
- Help break circular dependencies
- Clearly separate type and value imports

```lua
-- Type-only: no require() generated
import type {User} from "./types"

-- Regular import: generates require()
import {createUser} from "./api"
```

### Export All

`export * from "./module"` re-exports everything except:

- Default exports (must be explicitly re-exported)
- Type-only exports (unless using `export type * from`)

```lua
-- Re-export all values
export * from "./module"

-- Re-export all types
export type * from "./types"

-- Re-export specific default
export {default as MyDefault} from "./module"
```

### Module Compilation Modes

LuaNext supports two module compilation modes:

**1. Require Mode (default)**

Each module compiles to a file with `return M`:

```lua
local M = {}
M.foo = ...
return M
```

Imported with `require()`:

```lua
local module = require("./module")
```

**2. Bundle Mode**

All modules bundled into a single file with inline module definitions.

Configure in `luanext.config.yaml`:

```yaml
module:
  mode: "require"  # or "bundle"
```

### Module Scope

Each module has its own scope:

- Top-level variables are local to the module
- Only exported values are accessible from other modules
- Imports are available throughout the module

```lua
-- Private to this module
const PRIVATE_CONSTANT = 42

function privateHelper(): void
    -- Only accessible within this file
end

-- Public (exported)
export function publicAPI(): void
    privateHelper()  -- Can use private functions
end
```

### Module Initialization

Modules are initialized once on first import:

**counter.luax:**

```lua
print("Initializing counter module")

const count = 0

export function increment(): void
    count = count + 1
end

export function getCount(): number
    return count
end
```

**main.luax:**

```lua
import {increment, getCount} from "./counter"  -- Prints "Initializing counter module"
import {increment as inc} from "./counter"     -- Does NOT print again

increment()
print(getCount())  -- 1
inc()
print(getCount())  -- 2
```

## See Also

- [Basics](basics.md) — Variable declarations and types
- [Classes](classes.md) — Exporting classes
- [Interfaces](interfaces.md) — Exporting type definitions
- [Namespaces](../reference/configuration.md#namespaces) — File namespaces
