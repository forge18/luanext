# Quick Start

This guide will walk you through creating your first LuaNext program, compiling it, and running it with Lua.

## Your First LuaNext File

Create a file named `hello.luax`:

```lua
-- hello.luax
const message: string = "Hello, LuaNext!"
print(message)
```

LuaNext files use the `.luax` extension to distinguish them from plain Lua files.

## Compile and Run

Compile the file to Lua:

```bash
luanext hello.luax
```

This creates `hello.lua` in the same directory. Run it with Lua:

```bash
lua hello.lua
```

Output:

```text
Hello, LuaNext!
```

## Type Safety in Action

Let's see how types catch bugs. Create `calculator.luax`:

```lua
-- calculator.luax
function add(a: number, b: number): number
    return a + b
end

const result = add(5, 10)
print("5 + 10 =", result)

-- This will cause a type error:
const wrong = add("5", 10)
```

Compile it:

```bash
luanext calculator.luax
```

LuaNext catches the error:

```text
error[E0308]: type mismatch
 --> calculator.luax:9:21
  |
9 | const wrong = add("5", 10)
  |                   ^^^ expected number, found string
```

Fix the error by using a number instead:

```lua
const correct = add(5, 10)
```

## Working with Interfaces

Interfaces define the shape of tables. Create `person.luax`:

```lua
-- person.luax
interface Person {
    name: string,
    age: number,
    email?: string  -- optional field
}

function greet(person: Person): void
    print("Hello, " .. person.name .. "!")
    if person.email then
        print("Email: " .. person.email)
    end
end

const alice: Person = {
    name = "Alice",
    age = 30,
    email = "[email protected]"
}

const bob: Person = {
    name = "Bob",
    age = 25
    -- email is optional
}

greet(alice)
greet(bob)
```

Compile and run:

```bash
luanext person.luax
lua person.lua
```

Output:

```text
Hello, Alice!
Email: [email protected]
Hello, Bob!
```

## Type Inference

LuaNext infers types when you don't explicitly provide them. Create `inference.luax`:

```lua
-- inference.luax
-- Type inferred as number
const x = 42

-- Type inferred as (number, number) -> number
function multiply(a, b)
    return a * b
end

-- Type inferred as number
const result = multiply(x, 2)

print("Result:", result)
```

Even without type annotations, LuaNext knows `x` is a `number` and `multiply` returns a `number`. If you try to pass a string, you'll get a type error:

```lua
const wrong = multiply("hello", 2)  -- Error: expected number, found string
```

## Gradual Typing

You can mix typed and untyped code. LuaNext uses `unknown` for values without type information:

```lua
-- gradual.luax
-- Explicitly typed
const typed: number = 42

-- Inferred type
const inferred = 100

-- No type information (unknown)
local dynamic

dynamic = "string"
dynamic = 123
dynamic = { x = 1 }

-- To use dynamic values, narrow the type first
if type(dynamic) == "number" then
    const safe: number = dynamic
    print("It's a number:", safe)
end
```

## Compiling Multiple Files

Create two files:

**math.luax:**

```lua
-- math.luax
export function square(x: number): number
    return x * x
end

export function cube(x: number): number
    return x * x * x
end
```

**main.luax:**

```lua
-- main.luax
import { square, cube } from "./math"

print("Square of 5:", square(5))
print("Cube of 3:", cube(3))
```

Compile both files:

```bash
luanext main.luax math.luax
```

LuaNext automatically determines the correct compilation order based on imports.

Run the main file:

```bash
lua main.lua
```

Output:

```text
Square of 5: 25
Cube of 3: 27
```

## Specifying Output Directory

By default, compiled Lua files are created next to the source files. Use `--out-dir` to specify a different location:

```bash
luanext hello.luax --out-dir dist
```

This creates `dist/hello.lua`.

## Bundling Output

To combine multiple files into a single output file, use `--out-file`:

```bash
luanext main.luax math.luax --out-file bundle.lua
```

This creates a single `bundle.lua` file with all code.

## Targeting Different Lua Versions

LuaNext supports Lua 5.1, 5.2, 5.3, and 5.4. Specify the target version with `--target`:

```bash
luanext hello.luax --target 5.1
```

Default is Lua 5.4.

## Generating Source Maps

Source maps allow debuggers to map compiled Lua back to the original LuaNext source:

```bash
luanext hello.luax --source-map
```

This creates `hello.lua` and `hello.lua.map`.

## Watch Mode

Automatically recompile files when they change:

```bash
luanext hello.luax --watch
```

The compiler will watch `hello.luax` and recompile whenever you save changes.

## Next Steps

- [Editor Setup](editor-setup.md) — Configure VS Code for autocomplete and diagnostics
- [Project Setup](project-setup.md) — Create a `luanext.config.yaml` for larger projects
- [Type System](../language/type-system.md) — Learn about LuaNext's type system
- [CLI Reference](../reference/cli.md) — Complete command-line reference
