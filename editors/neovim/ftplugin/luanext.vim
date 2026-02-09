" Filetype plugin for LuaNext
" Language: LuaNext
" Maintainer: LuaNext Team

if exists("b:did_ftplugin")
  finish
endif
let b:did_ftplugin = 1

" Use Lua-style comments
setlocal commentstring=--\ %s
setlocal comments=:--

" Indentation settings
setlocal expandtab
setlocal shiftwidth=4
setlocal softtabstop=4
setlocal tabstop=4

" Format options
setlocal formatoptions-=t
setlocal formatoptions+=croql

" Keyword characters (include @ for decorators)
setlocal iskeyword+=@-@

" Undo settings
let b:undo_ftplugin = "setlocal commentstring< comments< expandtab< shiftwidth< softtabstop< tabstop< formatoptions< iskeyword<"
