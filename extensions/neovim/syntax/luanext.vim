" Vim syntax file for LuaNext
" Language: LuaNext
" Maintainer: LuaNext Team
" Latest Revision: 2026-02-08

if exists("b:current_syntax")
  finish
endif

" Load Lua syntax as base
runtime! syntax/lua.vim
unlet b:current_syntax

" TypeScript-style type annotations
syn keyword luanextType type interface namespace enum declare
syn keyword luanextKeyword import export from as default readonly
syn keyword luanextModifier public private protected static abstract
syn keyword luanextBuiltin string number boolean unknown void never any null undefined
syn keyword luanextStorage const let var

" Type operators
syn match luanextTypeOperator "[:|&?]"
syn match luanextArrow "=>"
syn match luanextNullCheck "??"
syn match luanextOptionalChain "?\\."

" Generics
syn region luanextGeneric start="<" end=">" contains=luanextType,luanextBuiltin,luanextGeneric

" Decorators
syn match luanextDecorator "@\w\+"

" Comments (TypeScript-style triple-slash directives)
syn match luanextDirective "///\s*<.*>" contains=luanextType

" Highlighting
hi def link luanextType Type
hi def link luanextKeyword Keyword
hi def link luanextModifier StorageClass
hi def link luanextBuiltin Type
hi def link luanextStorage StorageClass
hi def link luanextTypeOperator Operator
hi def link luanextArrow Operator
hi def link luanextNullCheck Operator
hi def link luanextOptionalChain Operator
hi def link luanextDecorator PreProc
hi def link luanextDirective SpecialComment

let b:current_syntax = "luanext"
