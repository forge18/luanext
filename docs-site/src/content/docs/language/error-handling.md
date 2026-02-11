# Error Handling

LuaNext provides structured exception handling with try/catch/finally blocks, typed error handling, error chaining, and try expressions. This goes beyond Lua's traditional `pcall`/`xpcall` approach.

## Syntax

```lua
-- Try/catch/finally statement
try
    -- code that might throw
catch [pattern: Type]
    -- error handling
[finally
    -- cleanup code]
end

-- Throw statement
throw expression

-- Rethrow
rethrow

-- Try expression
try expression catch error => fallback

-- Error chain operator
expression1 !! expression2

-- Throws clause
function name(params): ReturnType throws ErrorType1, ErrorType2
    -- function body
end
```

## Examples

### Basic Try/Catch

```lua
function divide(a: number, b: number): number
    if b == 0 then
        throw "Division by zero"
    end
    return a / b
end

try
    const result = divide(10, 0)
    print(result)
catch error
    print("Error: " .. error)
end
```

Compiles to:

```lua
local function divide(a, b)
    if b == 0 then
        error("Division by zero")
    end
    return a / b
end

local success, result = pcall(function()
    local result = divide(10, 0)
    print(result)
end)

if not success then
    local error = result
    print("Error: " .. error)
end
```

### Typed Catch Clauses

Catch specific error types:

```lua
class ValidationError
    message: string

    constructor(message: string)
        self.message = message
    end
end

class NetworkError
    message: string
    code: number

    constructor(message: string, code: number)
        self.message = message
        self.code = code
    end
end

function processRequest(data: string): void
    if data == "" then
        throw ValidationError.new("Data cannot be empty")
    end

    if not networkAvailable() then
        throw NetworkError.new("Network unavailable", 503)
    end

    -- Process data...
end

try
    processRequest("")
catch error: ValidationError
    print(`Validation error: ${error.message}`)
catch error: NetworkError
    print(`Network error (${error.code}): ${error.message}`)
catch error
    print(`Unknown error: ${error}`)
end
```

### Multi-Typed Catch

Catch multiple error types in one clause:

```lua
try
    riskyOperation()
catch error: ValidationError | NetworkError
    print(`Known error: ${error.message}`)
catch error
    print(`Unknown error: ${error}`)
end
```

### Finally Block

Code that always executes, whether an error occurred or not:

```lua
const file = openFile("data.txt")

try
    processFile(file)
catch error
    print(`Error processing file: ${error}`)
finally
    closeFile(file)  -- Always executed
end
```

### Throw Statement

Throw any value as an error:

```lua
-- Throw string
throw "Something went wrong"

-- Throw number
throw 404

-- Throw object
throw {message = "Error", code = 500}

-- Throw custom error class
throw ValidationError.new("Invalid input")
```

### Rethrow

Rethrow the current error in a catch block:

```lua
try
    performOperation()
catch error
    logError(error)
    rethrow  -- Re-throw the same error
end
```

### Try Expression

Compact error handling for expressions:

```lua
-- Basic try expression
const result: number = try parseNumber(input) catch _ => 0

-- With error variable
const result: string = try readFile("config.json") catch error => "default config"

-- Nested try expressions
const value: number = try parseInt(try readFile("num.txt") catch _ => "0") catch _ => 0
```

Compiles to:

```lua
local result
local success, value = pcall(function() return parseNumber(input) end)
if success then
    result = value
else
    result = 0
end
```

### Error Chain Operator

The error chain operator `!!` provides a shorthand for try expressions:

```lua
-- Error chain (try expression shorthand)
const result = riskyOperation() !! fallbackValue

-- Equivalent to:
const result = try riskyOperation() catch _ => fallbackValue

-- Chaining multiple operations
const value = operation1() !! operation2() !! defaultValue
```

### Throws Clause

Document which errors a function can throw:

```lua
function readFile(path: string): string throws FileNotFoundError, PermissionError
    if not fileExists(path) then
        throw FileNotFoundError.new(path)
    end

    if not hasPermission(path) then
        throw PermissionError.new(path)
    end

    return loadFile(path)
end

function processFile(path: string): void throws FileNotFoundError, PermissionError
    const content = readFile(path)  -- Can propagate throws
    print(content)
end
```

The `throws` clause is documentation—it doesn't enforce catching at compile time, but helps with:

- Documentation and intent
- IDE autocomplete and warnings
- Static analysis tools

### Multiple Catch Clauses

Handle different error types differently:

```lua
try
    complexOperation()
catch error: ValidationError
    print("Validation failed: " .. error.message)
    return false
catch error: NetworkError
    print("Network issue: " .. error.message)
    retryLater()
catch error: DatabaseError
    print("Database error: " .. error.message)
    rollbackTransaction()
catch error
    print("Unexpected error: " .. tostring(error))
    logToFile(error)
end
```

### Nested Try/Catch

Try blocks can be nested:

```lua
try
    const data = fetchData()

    try
        validateData(data)
    catch error: ValidationError
        print("Validation failed, using defaults")
        data = getDefaultData()
    end

    processData(data)
catch error: NetworkError
    print("Network error: " .. error.message)
catch error
    print("Unexpected error: " .. tostring(error))
end
```

### Error with Context

Custom error classes with context:

```lua
class HttpError
    statusCode: number
    message: string
    url: string

    constructor(statusCode: number, message: string, url: string)
        self.statusCode = statusCode
        self.message = message
        self.url = url
    end

    function toString(): string
        return `HTTP ${self.statusCode} at ${self.url}: ${self.message}`
    end
end

function fetchAPI(url: string): string throws HttpError
    const response = httpGet(url)

    if response.statusCode ~= 200 then
        throw HttpError.new(response.statusCode, response.body, url)
    end

    return response.body
end

try
    const data = fetchAPI("https://api.example.com/data")
    print(data)
catch error: HttpError
    print(error:toString())
    if error.statusCode == 404 then
        print("Resource not found")
    elseif error.statusCode >= 500 then
        print("Server error, retrying...")
    end
end
```

### Error Propagation

Errors automatically propagate up the call stack:

```lua
function innerFunction(): void
    throw "Inner error"
end

function middleFunction(): void
    innerFunction()  -- Error propagates
end

function outerFunction(): void
    try
        middleFunction()
    catch error
        print("Caught error from inner: " .. error)
    end
end

outerFunction()  -- Prints: Caught error from inner: Inner error
```

### Graceful Degradation

Use error handling for graceful degradation:

```lua
function loadConfig(): Config
    -- Try primary config
    const primary = try readFile("config.json") catch _ => nil

    if primary ~= nil then
        return parseConfig(primary)
    end

    -- Try fallback config
    const fallback = try readFile("config.default.json") catch _ => nil

    if fallback ~= nil then
        return parseConfig(fallback)
    end

    -- Use hardcoded defaults
    return getDefaultConfig()
end
```

### Resource Management

Ensure resources are cleaned up with finally:

```lua
function processDatabase(query: string): void
    const connection = openConnection()

    try
        const result = connection:execute(query)
        processResult(result)
    catch error: DatabaseError
        print("Database error: " .. error.message)
        rollback(connection)
    finally
        closeConnection(connection)  -- Always executed
    end
end
```

### Error Recovery

Recover from errors and continue:

```lua
function processBatch(items: string[]): void
    const errors: string[] = {}

    for i, item in ipairs(items) do
        try
            processItem(item)
        catch error
            table.insert(errors, `Item ${i}: ${error}`)
        end
    end

    if #errors > 0 then
        print("Encountered errors:")
        for _, error in ipairs(errors) do
            print(error)
        end
    end
end
```

## Details

### Error Types

LuaNext can throw any value:

- Strings: `throw "error message"`
- Numbers: `throw 404`
- Tables: `throw {code = 500, message = "Error"}`
- Custom classes: `throw CustomError.new()`

### Catch Pattern Matching

Catch clauses use pattern matching:

- `catch error` — Catch any error
- `catch error: Type` — Catch specific type
- `catch error: Type1 | Type2` — Catch multiple types

### Finally Execution Order

The finally block executes:

1. After try block completes successfully
2. After catch block handles an error
3. Before control flow exits (return, break, continue)

```lua
function example(): string
    try
        return "success"
    finally
        print("Cleanup")  -- Executes before return
    end
end

example()  -- Prints "Cleanup", then returns "success"
```

### Error Chain Precedence

The error chain operator `!!` has low precedence:

```lua
-- These are equivalent:
const x = a + b !! c + d
const x = (a + b) !! (c + d)

-- Parentheses for clarity:
const x = (a + b) !! c  -- Error chain after addition
```

### Try Expression vs Error Chain

Try expression and error chain are equivalent:

```lua
-- Try expression (explicit)
const x = try f() catch _ => 0

-- Error chain (concise)
const x = f() !! 0
```

Use try expression when you need the error variable:

```lua
const x = try f() catch error => log(error) or 0
```

Use error chain for simple fallbacks:

```lua
const x = f() !! 0
```

### Throws Clause Benefits

While not enforced, the `throws` clause provides:

- **Documentation** — Clear indication of possible errors
- **IDE support** — Warnings and autocomplete
- **Static analysis** — Tools can check error handling
- **Call graph** — Understand error propagation

### Performance Considerations

Exception handling uses Lua's `pcall`/`xpcall`:

- Try blocks have minimal overhead when no error occurs
- Error throwing and catching is slower than normal control flow
- Use for exceptional cases, not normal flow control

### Error vs Return

Prefer errors for exceptional conditions:

```lua
-- Good: Error for exceptional condition
function divide(a: number, b: number): number
    if b == 0 then
        throw "Division by zero"
    end
    return a / b
end

-- Also good: Return optional for expected missing values
function findUser(id: string): User | nil
    return users[id]  -- nil is expected, not exceptional
end
```

### Stack Traces

LuaNext preserves Lua's error stack traces:

```lua
function a(): void
    b()
end

function b(): void
    c()
end

function c(): void
    throw "Error in c"
end

try
    a()
catch error
    print(debug.traceback(error))
    -- Shows: c -> b -> a call chain
end
```

## See Also

- [Functions](functions.md) — Throws clause in function signatures
- [Classes](classes.md) — Custom error classes
- [Type System](type-system.md) — Union types for error types
- [Pattern Matching](pattern-matching.md) — Pattern matching in catch clauses
