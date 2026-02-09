-- Language specification for LuaNext syntax highlighting
return {
  exts = {"luax"},
  lexer = wxstc.wxSTC_LEX_LUA,

  apitype = "lua",

  linecomment = "--",
  blockcomment = {"--[[", "]]"},

  keywords = {
    -- Standard Lua keywords
    [[and break do else elseif end false for function goto if in
      local nil not or repeat return then true until while]],

    -- LuaNext-specific keywords (TypeScript-inspired)
    [[type interface namespace enum declare
      import export from as default
      readonly public private protected static abstract
      const let var implements extends
      async await]],

    -- Built-in types
    [[string number boolean unknown void never any null undefined
      object symbol bigint]],

    -- Lua standard library
    [[_G _VERSION assert collectgarbage dofile error getmetatable ipairs
      load loadfile next pairs pcall print rawequal rawget rawlen rawset
      require select setmetatable tonumber tostring type xpcall
      coroutine debug io math os package string table utf8]],
  },

  lexerstyleconvert = {
    text = {wxstc.wxSTC_LUA_IDENTIFIER,},

    lexerdef = {wxstc.wxSTC_LUA_DEFAULT,},

    comment = {wxstc.wxSTC_LUA_COMMENT,
                wxstc.wxSTC_LUA_COMMENTLINE,
                wxstc.wxSTC_LUA_COMMENTDOC,},

    stringtxt = {wxstc.wxSTC_LUA_STRING,
                  wxstc.wxSTC_LUA_CHARACTER,
                  wxstc.wxSTC_LUA_LITERALSTRING,},

    stringeol = {wxstc.wxSTC_LUA_STRINGEOL,},

    preprocessor= {wxstc.wxSTC_LUA_PREPROCESSOR,},

    operator = {wxstc.wxSTC_LUA_OPERATOR,},

    number = {wxstc.wxSTC_LUA_NUMBER,},

    keywords0 = {wxstc.wxSTC_LUA_WORD,},

    keywords1 = {wxstc.wxSTC_LUA_WORD2,},

    keywords2 = {wxstc.wxSTC_LUA_WORD3,},

    keywords3 = {wxstc.wxSTC_LUA_WORD4,},

    keywords4 = {wxstc.wxSTC_LUA_WORD5,},

    keywords5 = {wxstc.wxSTC_LUA_WORD6,},

    keywords6 = {wxstc.wxSTC_LUA_WORD7,},

    keywords7 = {wxstc.wxSTC_LUA_WORD8,},
  },

  -- Additional styling for LuaNext-specific syntax
  marksymbols = {
    -- Type annotations
    {":", wxstc.wxSTC_LUA_OPERATOR},
    -- Generics
    {"<", wxstc.wxSTC_LUA_OPERATOR},
    {">", wxstc.wxSTC_LUA_OPERATOR},
    -- Null coalescing
    {"??", wxstc.wxSTC_LUA_OPERATOR},
    -- Optional chaining
    {"?.", wxstc.wxSTC_LUA_OPERATOR},
    -- Arrow functions
    {"=>", wxstc.wxSTC_LUA_OPERATOR},
  },
}
