# Error Codes Reference

Complete reference for LuaNext compiler error codes and diagnostics.

## Overview

LuaNext error codes follow the format `E####` where `####` is a four-digit number. Errors are categorized by type.

Enable error codes in output:

```bash
luanext --diagnostics main.luax
```

Or in configuration:

```yaml
compilerOptions:
  diagnostics: true
```

## Type Errors (E0001-E0999)

### E0001: Type Mismatch

**Description:** Expected one type, found another.

**Example:**

```lua
const x: number = "hello"  -- E0001: Type mismatch
```

**Fix:** Ensure types match or use type assertion:

```lua
const x: number = 42  -- ✅ OK
const y = "hello" as unknown as number  -- Type assertion (use with caution)
```

### E0002: Undefined Variable

**Description:** Variable used before declaration.

**Example:**

```lua
print(x)  -- E0002: Undefined variable 'x'
const x = 5
```

**Fix:** Declare before use:

```lua
const x = 5
print(x)  -- ✅ OK
```

### E0003: Cannot Assign to Const

**Description:** Attempting to reassign an immutable variable.

**Example:**

```lua
const PI = 3.14
PI = 3  -- E0003: Cannot assign to const variable
```

**Fix:** Use `local` for mutable variables:

```lua
local count = 0
count = 5  -- ✅ OK
```

### E0004: Null Reference

**Description:** Possible nil access without check (strict null checks enabled).

**Example:**

```lua
function findUser(id: string): User | nil
    return users[id]
end

const user = findUser("123")
print(user.name)  -- E0004: Possible nil reference
```

**Fix:** Check for nil first:

```lua
const user = findUser("123")
if user ~= nil then
    print(user.name)  -- ✅ OK
end
```

### E0005: Property Does Not Exist

**Description:** Accessing non-existent property on a type.

**Example:**

```lua
interface User
    name: string
end

const user: User = {name = "Alice"}
print(user.age)  -- E0005: Property 'age' does not exist on type 'User'
```

**Fix:** Add property to interface or check existence:

```lua
interface User
    name: string
    age?: number  -- Optional property
end
```

## Function Errors (E1000-E1999)

### E1001: Argument Count Mismatch

**Description:** Wrong number of arguments passed to function.

**Example:**

```lua
function add(a: number, b: number): number
    return a + b
end

add(5)  -- E1001: Expected 2 arguments, got 1
```

**Fix:** Provide correct number of arguments:

```lua
add(5, 3)  -- ✅ OK
```

### E1002: Invalid Return Type

**Description:** Function returns wrong type.

**Example:**

```lua
function getName(): string
    return 42  -- E1002: Cannot return number from function returning string
end
```

**Fix:** Return correct type:

```lua
function getName(): string
    return "Alice"  -- ✅ OK
end
```

### E1003: Missing Return Statement

**Description:** Function with non-void return type doesn't always return.

**Example:**

```lua
function divide(a: number, b: number): number
    if b ~= 0 then
        return a / b
    end
    -- E1003: Not all code paths return a value
end
```

**Fix:** Ensure all paths return:

```lua
function divide(a: number, b: number): number
    if b ~= 0 then
        return a / b
    end
    return 0  -- ✅ OK
end
```

## Class Errors (E2000-E2999)

### E2001: Cannot Extend Sealed Class

**Description:** Attempting to extend a class marked as sealed.

**Example:**

```lua
@sealed
class FinalClass end

class Derived extends FinalClass end  -- E2001: Cannot extend sealed class
```

**Fix:** Don't extend sealed classes or remove `@sealed`.

### E2002: Property Already Declared

**Description:** Duplicate property in class.

**Example:**

```lua
class User
    name: string
    name: string  -- E2002: Property 'name' already declared
end
```

**Fix:** Remove duplicate:

```lua
class User
    name: string  -- ✅ OK
end
```

### E2003: Must Override

**Description:** Abstract method not implemented.

**Example:**

```lua
abstract class Animal
    abstract function speak(): void
end

class Dog extends Animal end  -- E2003: Must override abstract method 'speak'
```

**Fix:** Implement abstract methods:

```lua
class Dog extends Animal
    override function speak(): void
        print("Woof!")
    end
end
```

## Module Errors (E3000-E3999)

### E3001: Module Not Found

**Description:** Cannot find imported module.

**Example:**

```lua
import {helper} from "./missing"  -- E3001: Module './missing' not found
```

**Fix:** Check file path and ensure file exists:

```lua
import {helper} from "./utils"  -- ✅ OK (if utils.luax exists)
```

### E3002: Circular Dependency

**Description:** Modules import each other, creating a cycle.

**Example:**

```lua
-- a.luax
import {b} from "./b"

-- b.luax
import {a} from "./a"  -- E3002: Circular dependency detected
```

**Fix:** Refactor to break cycle:

```lua
-- Create shared.luax with common code
-- a.luax imports shared
-- b.luax imports shared
```

### E3003: Export Not Found

**Description:** Trying to import non-existent export.

**Example:**

```lua
-- utils.luax
export function add(a: number, b: number): number
    return a + b
end

-- main.luax
import {multiply} from "./utils"  -- E3003: Export 'multiply' not found
```

**Fix:** Import existing exports:

```lua
import {add} from "./utils"  -- ✅ OK
```

## Generic Errors (E4000-E4999)

### E4001: Type Argument Mismatch

**Description:** Wrong type arguments for generic.

**Example:**

```lua
type Box<T extends number> = {value: T}

type StringBox = Box<string>  -- E4001: string doesn't satisfy constraint 'extends number'
```

**Fix:** Use compatible type:

```lua
type NumberBox = Box<number>  -- ✅ OK
```

### E4002: Cannot Infer Type Argument

**Description:** Generic type argument cannot be inferred.

**Example:**

```lua
function identity<T>(x: T): T
    return x
end

const result = identity()  -- E4002: Cannot infer type argument for T
```

**Fix:** Provide explicit type or argument:

```lua
const result = identity<number>(42)  -- ✅ OK
const result2 = identity(42)  -- ✅ OK (infers number)
```

## Pattern Matching Errors (E5000-E5999)

### E5001: Non-Exhaustive Match

**Description:** Match expression doesn't handle all cases.

**Example:**

```lua
enum Status { Pending, Active, Completed }

function handle(status: Status): string
    return match status
        | Status.Pending -> "waiting"
        | Status.Active -> "processing"
        -- E5001: Non-exhaustive match, missing Status.Completed
    end
end
```

**Fix:** Handle all cases or add wildcard:

```lua
function handle(status: Status): string
    return match status
        | Status.Pending -> "waiting"
        | Status.Active -> "processing"
        | Status.Completed -> "done"  -- ✅ OK
    end
end
```

### E5002: Unreachable Pattern

**Description:** Pattern will never be matched.

**Example:**

```lua
match x
    | _ -> "any"
    | 42 -> "forty-two"  -- E5002: Unreachable pattern
end
```

**Fix:** Reorder patterns (specific before general):

```lua
match x
    | 42 -> "forty-two"
    | _ -> "any"  -- ✅ OK
end
```

## Decorator Errors (E6000-E6999)

### E6001: Invalid Decorator Target

**Description:** Decorator applied to invalid target.

**Example:**

```lua
@decorator
const x = 5  -- E6001: Cannot apply decorator to const variable
```

**Fix:** Apply decorators to valid targets (classes, methods, properties):

```lua
class User
    @decorator
    name: string  -- ✅ OK
end
```

## Naming Convention Errors (E7000-E7999)

### E7001: Invalid Constant Name

**Description:** Const doesn't follow UPPER_SNAKE_CASE.

**Example:**

```lua
const maxSize = 100  -- E7001: Const 'maxSize' should be UPPER_SNAKE_CASE
```

**Fix:** Use correct naming:

```lua
const MAX_SIZE = 100  -- ✅ OK
```

### E7002: Invalid Type Name

**Description:** Type/interface doesn't follow PascalCase.

**Example:**

```lua
interface user_data end  -- E7002: Type 'user_data' should be PascalCase
```

**Fix:** Use PascalCase:

```lua
interface UserData end  -- ✅ OK
```

## Error Severity Levels

Errors have different severity levels:

- **Error** — Compilation fails
- **Warning** — Compilation succeeds, but issue flagged
- **Info** — Informational message

Control severity with `strictNaming`:

```yaml
compilerOptions:
  strictNaming: "error"    # Naming violations are errors
  # OR
  strictNaming: "warning"  # Naming violations are warnings
  # OR
  strictNaming: "off"      # No naming enforcement
```

## Suppressing Errors

### Type Assertions

Bypass type checking (use sparingly):

```lua
const data = externalCall() as User  -- Suppresses type errors
```

### Unknown Type

Use `unknown` for truly dynamic data:

```lua
const data: unknown = getData()

-- Must narrow type before use
if type(data) == "table" then
    print(data.value)  -- ✅ OK after narrowing
end
```

## Debugging Errors

### Pretty Diagnostics

Enable for better error messages:

```bash
luanext --pretty main.luax
```

```
error[E0001]: Type mismatch
  ┌─ main.luax:5:10
  │
5 │     const x: number = "hello"
  │              ^^^^^^   ^^^^^^^ expected number, found string
  │              │
  │              type annotation
```

### Error Codes

Show error codes for lookup:

```bash
luanext --diagnostics main.luax
```

```
main.luax:5:10: error[E0001]: Type mismatch
```

## See Also

- [CLI Reference](cli.md) — Diagnostic options
- [Configuration](configuration.md) — Error configuration
- [Type System](../language/type-system.md) — Type checking rules
