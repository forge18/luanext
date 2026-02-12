-- LuaNext plugin for ZeroBrane Studio
-- Provides syntax highlighting, LSP integration, and IDE features for LuaNext (.luax) files

local id = ID("luanext.luanext")

-- Plugin configuration
local config = {
  lspPath = "luanext-lsp",
  enableLSP = true,
  enableTypeChecking = true,
  strictNullChecks = true,
  autoComplete = true,
  showParameterHints = true,
  showTypeHints = true,
  formatOnSave = false,
  indentSize = 4,
  useTabs = false,
  target = "5.4",
  optimizationLevel = "auto",
}

-- Merge user configuration
local function loadConfig()
  local userConfig = ide.config.luanext or {}
  for k, v in pairs(userConfig) do
    config[k] = v
  end
end

-- LSP client state
local lspClient = nil
local lspProcess = nil

-- Start the language server
local function startLSP()
  if not config.enableLSP then
    return
  end

  -- Check if LSP is already running
  if lspClient then
    return
  end

  DisplayOutput("[LuaNext] Starting language server: " .. config.lspPath .. "\n")

  -- Start LSP process
  local cmd = config.lspPath
  lspProcess = wx.wxProcess()
  lspProcess:Redirect()

  local pid = wx.wxExecute(cmd, wx.wxEXEC_ASYNC, lspProcess)

  if pid == -1 then
    DisplayOutputLn("[LuaNext] Error: Failed to start language server")
    return
  end

  DisplayOutputLn("[LuaNext] Language server started (PID: " .. pid .. ")")

  -- Initialize LSP client
  lspClient = {
    pid = pid,
    process = lspProcess,
    nextId = 1,
    pendingRequests = {},
  }

  -- Send initialize request
  sendLSPRequest("initialize", {
    processId = wx.wxGetProcessId(),
    rootUri = "file://" .. ide:GetProject(),
    capabilities = {
      textDocument = {
        completion = { dynamicRegistration = false },
        hover = { dynamicRegistration = false },
        signatureHelp = { dynamicRegistration = false },
        definition = { dynamicRegistration = false },
        references = { dynamicRegistration = false },
        documentSymbol = { dynamicRegistration = false },
        formatting = { dynamicRegistration = false },
      }
    },
    initializationOptions = {
      checkOnSave = config.enableTypeChecking,
      strictNullChecks = config.strictNullChecks,
    }
  })
end

-- Stop the language server
local function stopLSP()
  if not lspClient then
    return
  end

  DisplayOutputLn("[LuaNext] Stopping language server...")

  -- Send shutdown request
  sendLSPRequest("shutdown", {})

  -- Kill process
  if lspClient.process then
    lspClient.process:Kill(lspClient.pid, wx.wxSIGTERM)
  end

  lspClient = nil
  lspProcess = nil
end

-- Send LSP request
function sendLSPRequest(method, params, callback)
  if not lspClient then
    return
  end

  local id = lspClient.nextId
  lspClient.nextId = id + 1

  local request = {
    jsonrpc = "2.0",
    id = id,
    method = method,
    params = params
  }

  -- Store callback
  if callback then
    lspClient.pendingRequests[id] = callback
  end

  -- Send request (simplified - real implementation would use JSON encoding)
  local json = require("dkjson")
  local requestStr = json.encode(request)
  local stream = lspClient.process:GetOutputStream()

  if stream then
    stream:Write("Content-Length: " .. #requestStr .. "\r\n\r\n" .. requestStr)
  end
end

-- Compile current file
local function compileFile()
  local editor = ide:GetEditor()
  if not editor then
    return
  end

  local filePath = ide:GetDocument(editor):GetFilePath()
  if not filePath or not filePath:match("%.luax$") then
    DisplayOutputLn("[LuaNext] Error: Current file is not a LuaNext file (.luax)")
    return
  end

  DisplayOutputLn("[LuaNext] Compiling: " .. filePath)

  -- Run luanext compiler
  local cmd = "luanext " .. filePath
  local output = wx.wxExecute(cmd, wx.wxEXEC_SYNC)

  if output == 0 then
    DisplayOutputLn("[LuaNext] Compilation successful")
  else
    DisplayOutputLn("[LuaNext] Compilation failed")
  end
end

-- Type check current file
local function typeCheckFile()
  local editor = ide:GetEditor()
  if not editor then
    return
  end

  local filePath = ide:GetDocument(editor):GetFilePath()
  if not filePath or not filePath:match("%.luax$") then
    return
  end

  DisplayOutputLn("[LuaNext] Type checking: " .. filePath)

  -- Run type checker
  local cmd = "luanext --no-emit " .. filePath
  local output = wx.wxExecute(cmd, wx.wxEXEC_SYNC)

  if output == 0 then
    DisplayOutputLn("[LuaNext] Type check passed")
  else
    DisplayOutputLn("[LuaNext] Type check failed")
  end
end

-- Plugin initialization
return {
  name = "LuaNext support",
  description = "Adds LuaNext language support with LSP integration",
  author = "LuaNext Team",
  version = 0.1,

  onRegister = function()
    loadConfig()

    -- Register file extension
    ide:AddInterpreter("luanext", {
      name = "LuaNext",
      description = "LuaNext (.luax) files",
      api = {"baselib", "luanext"},
      fext = "luax",
      fprojext = "luax",
      frun = function(fname, env)
        -- Compile and run
        local luaFile = fname:gsub("%.luax$", ".lua")
        compileFile()
        if wx.wxFileExists(luaFile) then
          return ide:GetInterpreter("lua"):frun(luaFile, env)
        end
      end,
      hasdebugger = false,
      fattachdebug = function() end,
      scratchextloop = true,
    })

    -- Register menu items
    local menu = ide:FindTopMenu("&Project")
    if menu then
      menu:Append(id, "Compile LuaNext File\tCtrl-Shift-B")
      menu:Append(id+1, "Type Check File\tCtrl-T")
      menu:Append(id+2, "Restart LuaNext LSP\tCtrl-Shift-L")
    end

    -- Start LSP on startup
    startLSP()

    DisplayOutputLn("[LuaNext] Plugin loaded")
  end,

  onUnRegister = function()
    stopLSP()
  end,

  onMenuCompile = function()
    compileFile()
  end,

  onMenuTypeCheck = function()
    typeCheckFile()
  end,

  onMenuRestartLSP = function()
    stopLSP()
    startLSP()
  end,

  onEditorLoad = function(editor)
    local doc = ide:GetDocument(editor)
    if doc and doc:GetFilePath():match("%.luax$") then
      -- Set up editor for LuaNext
      editor:SetIndent(config.indentSize)
      editor:SetUseTabs(config.useTabs)

      DisplayOutputLn("[LuaNext] File loaded: " .. doc:GetFilePath())
    end
  end,

  onEditorSave = function(editor)
    local doc = ide:GetDocument(editor)
    if doc and doc:GetFilePath():match("%.luax$") then
      if config.enableTypeChecking then
        typeCheckFile()
      end

      if config.formatOnSave then
        -- TODO: Implement formatting via LSP
      end
    end
  end,

  onEditorCharAdded = function(editor, event)
    local char = event:GetKey()

    -- Auto-completion triggers
    if config.autoComplete and (char == string.byte('.') or char == string.byte(':')) then
      editor:AutoCompShow(0, "")
    end

    -- Parameter hints
    if config.showParameterHints and char == string.byte('(') then
      -- TODO: Request signature help from LSP
    end
  end,
}
