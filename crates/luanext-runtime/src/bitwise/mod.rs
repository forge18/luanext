//! Bitwise operation helpers for Lua versions that lack native bitwise operators.

pub fn for_lua51() -> &'static str {
    LUA51_BITWISE_HELPERS
}

pub fn for_lua52() -> &'static str {
    LUA52_BIT32_POLYFILL
}

pub fn for_lua53_54() -> &'static str {
    ""
}

const LUA51_BITWISE_HELPERS: &str = r#"-- Bitwise operation helpers for Lua 5.1
local function _bit_band(a, b)
    local result = 0
    local bitval = 1
    while a > 0 and b > 0 do
        if a % 2 == 1 and b % 2 == 1 then
            result = result + bitval
        end
        bitval = bitval * 2
        a = math.floor(a / 2)
        b = math.floor(b / 2)
    end
    return result
end

local function _bit_bor(a, b)
    local result = 0
    local bitval = 1
    while a > 0 or b > 0 do
        if a % 2 == 1 or b % 2 == 1 then
            result = result + bitval
        end
        bitval = bitval * 2
        a = math.floor(a / 2)
        b = math.floor(b / 2)
    end
    return result
end

local function _bit_bxor(a, b)
    local result = 0
    local bitval = 1
    while a > 0 or b > 0 do
        if (a % 2) ~= (b % 2) then
            result = result + bitval
        end
        bitval = bitval * 2
        a = math.floor(a / 2)
        b = math.floor(b / 2)
    end
    return result
end

local function _bit_bnot(a)
    local result = 0
    local bitval = 1
    for i = 0, 31 do
        if a % 2 == 0 then
            result = result + bitval
        end
        bitval = bitval * 2
        a = math.floor(a / 2)
    end
    return result
end

local function _bit_lshift(a, b)
    return math.floor(a) * (2 ^ b)
end

local function _bit_rshift(a, b)
    return math.floor(math.floor(a) / (2 ^ b))
end
"#;

const LUA52_BIT32_POLYFILL: &str = r#"-- bit32 polyfill for Lua 5.2 compatibility
-- Provides the bit32 library API using pure Lua arithmetic.
-- On a real Lua 5.2 runtime, this shadows the built-in bit32 (same API).
rawset(_G, "bit32", {})
local bit32 = rawget(_G, "bit32")
function bit32.band(a, b)
    local result = 0
    local bitval = 1
    while a > 0 and b > 0 do
        if a % 2 == 1 and b % 2 == 1 then
            result = result + bitval
        end
        bitval = bitval * 2
        a = math.floor(a / 2)
        b = math.floor(b / 2)
    end
    return result
end
function bit32.bor(a, b)
    local result = 0
    local bitval = 1
    while a > 0 or b > 0 do
        if a % 2 == 1 or b % 2 == 1 then
            result = result + bitval
        end
        bitval = bitval * 2
        a = math.floor(a / 2)
        b = math.floor(b / 2)
    end
    return result
end
function bit32.bxor(a, b)
    local result = 0
    local bitval = 1
    while a > 0 or b > 0 do
        if (a % 2) ~= (b % 2) then
            result = result + bitval
        end
        bitval = bitval * 2
        a = math.floor(a / 2)
        b = math.floor(b / 2)
    end
    return result
end
function bit32.bnot(a)
    local result = 0
    local bitval = 1
    for i = 0, 31 do
        if a % 2 == 0 then
            result = result + bitval
        end
        bitval = bitval * 2
        a = math.floor(a / 2)
    end
    return result
end
function bit32.lshift(a, b)
    return math.floor(a) * (2 ^ b)
end
function bit32.rshift(a, b)
    return math.floor(math.floor(a) / (2 ^ b))
end
"#;
