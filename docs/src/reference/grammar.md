# Grammar Reference

Formal grammar specification for LuaNext syntax in EBNF notation.

## Notation

- `::=` — Definition
- `|` — Alternative
- `()` — Grouping
- `[]` — Optional (zero or one)
- `{}` — Repetition (zero or more)
- `<>` — Non-terminal
- `""` — Terminal (keyword or operator)
- `/* */` — Comment

## Program Structure

```ebnf
<program> ::= {<statement>}

<statement> ::=
    | <variable_declaration>
    | <function_declaration>
    | <class_declaration>
    | <interface_declaration>
    | <enum_declaration>
    | <type_alias>
    | <namespace_declaration>
    | <import_statement>
    | <export_statement>
    | <expression_statement>
    | <if_statement>
    | <while_statement>
    | <for_statement>
    | <do_statement>
    | <return_statement>
    | <break_statement>
    | <continue_statement>
    | <try_statement>
    | <throw_statement>
    | <match_statement>
    | <block>
```

## Declarations

### Variable Declaration

```ebnf
<variable_declaration> ::=
    | "const" <identifier> [":" <type>] "=" <expression>
    | "local" <identifier> [":" <type>] ["=" <expression>]
```

### Function Declaration

```ebnf
<function_declaration> ::=
    "function" <identifier> <type_parameters> "(" <parameter_list> ")" [":" <type>] ["throws" <type_list>]
        <block>
    "end"

<parameter_list> ::= [<parameter> {"," <parameter>} ["," "..." <identifier> ":" <type> "[]"]]

<parameter> ::= <identifier> ["?" | "!"] ":" <type> ["=" <expression>]
```

### Class Declaration

```ebnf
<class_declaration> ::=
    {<decorator>}
    "class" <identifier> <type_parameters> ["(" <primary_constructor_params> ")"] ["extends" <type>] ["implements" <type_list>]
        {<class_member>}
    "end"

<primary_constructor_params> ::= <constructor_param> {"," <constructor_param>}

<constructor_param> ::= [<visibility>] <identifier> ":" <type>

<class_member> ::=
    | <property>
    | <method>
    | <constructor>
    | <getter>
    | <setter>
    | <operator_overload>

<property> ::= {<decorator>} [<visibility>] ["readonly"] <identifier> ":" <type> ["=" <expression>]

<method> ::= {<decorator>} [<visibility>] ["override"] ["final"] "function" <identifier> <type_parameters> "(" <parameter_list> ")" [":" <type>] <block> "end"

<constructor> ::= "constructor" "(" <parameter_list> ")" <block> "end"

<getter> ::= {<decorator>} [<visibility>] "get" <identifier> "(" ")" ":" <type> <block> "end"

<setter> ::= {<decorator>} [<visibility>] "set" <identifier> "(" <identifier> ":" <type> ")" <block> "end"

<operator_overload> ::= {<decorator>} "operator" <overloadable_op> "(" <parameter_list> ")" ":" <type> <block> "end"

<overloadable_op> ::= "+" | "-" | "*" | "/" | "%" | "^" | "//" | ".." | "==" | "~=" | "<" | "<=" | ">" | ">=" | "&" | "|" | "~" | "<<" | ">>" | "[]" | "()" | "#" | "-" | "not"
```

### Interface Declaration

```ebnf
<interface_declaration> ::=
    "interface" <identifier> <type_parameters> ["extends" <type_list>]
        {<interface_member>}
    "end"

<interface_member> ::=
    | <property_signature>
    | <method_signature>
    | <index_signature>
    | <call_signature>
    | <method_with_body>

<property_signature> ::= ["readonly"] <identifier> ["?"] ":" <type>

<method_signature> ::= "function" <identifier> <type_parameters> "(" <parameter_list> ")" ":" <type>

<method_with_body> ::= "function" <identifier> <type_parameters> "(" <parameter_list> ")" ":" <type> <block> "end"

<index_signature> ::= "[" <identifier> ":" <type> "]" ":" <type>

<call_signature> ::= "(" <parameter_list> ")" ":" <type>
```

### Enum Declaration

```ebnf
<enum_declaration> ::=
    "enum" <identifier>
        <enum_members>
    "end"

<enum_members> ::= <enum_variant> {"," <enum_variant>} [","]

<enum_variant> ::= <identifier> ["=" <expression>]
```

### Type Alias

```ebnf
<type_alias> ::= "type" <identifier> <type_parameters> "=" <type>
```

## Types

```ebnf
<type> ::=
    | <primary_type>
    | <union_type>
    | <intersection_type>
    | <function_type>
    | <conditional_type>

<primary_type> ::=
    | <type_reference>
    | <literal_type>
    | <array_type>
    | <tuple_type>
    | <object_type>
    | <parenthesized_type>
    | <typeof_type>
    | <template_literal_type>

<type_reference> ::= <identifier> [<type_arguments>]

<literal_type> ::= <string_literal> | <number_literal> | "true" | "false" | "nil"

<array_type> ::= <type> "[" "]"

<tuple_type> ::= "[" <type_list> "]"

<object_type> ::= "{" {<type_member> ","} "}"

<type_member> ::=
    | <property_signature>
    | <method_signature>
    | <index_signature>

<union_type> ::= <type> "|" <type> {"|" <type>}

<intersection_type> ::= <type> "&" <type> {"&" <type>}

<function_type> ::= "(" <parameter_list> ")" "=>" <type>

<conditional_type> ::= <type> "extends" <type> "?" <type> ":" <type>

<typeof_type> ::= "typeof" <expression>

<template_literal_type> ::= "`" {<template_literal_element>} "`"

<template_literal_element> ::= <string_chars> | "${" <type> "}"

<type_parameters> ::= "<" <type_parameter_list> ">"

<type_parameter_list> ::= <type_parameter> {"," <type_parameter>}

<type_parameter> ::= <identifier> ["extends" <type>] ["=" <type>]

<type_arguments> ::= "<" <type_list> ">"

<type_list> ::= <type> {"," <type>}
```

## Expressions

```ebnf
<expression> ::=
    | <assignment_expression>
    | <conditional_expression>
    | <binary_expression>
    | <unary_expression>
    | <postfix_expression>
    | <primary_expression>

<primary_expression> ::=
    | <identifier>
    | <literal>
    | <function_expression>
    | <arrow_function>
    | <class_expression>
    | <array_literal>
    | <table_literal>
    | <template_string>
    | <parenthesized_expression>
    | <new_expression>

<assignment_expression> ::= <expression> <assignment_op> <expression>

<assignment_op> ::= "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "^=" | "//=" | "..=" | "&=" | "|=" | "<<=" | ">>="

<conditional_expression> ::= <expression> "?" <expression> ":" <expression>

<binary_expression> ::= <expression> <binary_op> <expression>

<binary_op> ::=
    | "or" | "and"
    | "==" | "~=" | "<" | "<=" | ">" | ">="
    | "|" | "~" | "&" | "<<" | ">>"
    | ".." | "+" | "-" | "*" | "/" | "//" | "%" | "^"
    | "??" | "|>" | "!!"
    | "instanceof"

<unary_expression> ::= <unary_op> <expression>

<unary_op> ::= "not" | "#" | "-" | "~"

<postfix_expression> ::=
    | <expression> "." <identifier>
    | <expression> "?." <identifier>
    | <expression> "[" <expression> "]"
    | <expression> "?[" <expression> "]"
    | <expression> "(" [<argument_list>] ")"
    | <expression> "?(" [<argument_list>] ")"
    | <expression> ":" <identifier> "(" [<argument_list>] ")"
    | <expression> "?:" <identifier> "(" [<argument_list>] ")"
    | <expression> "as" <type>

<arrow_function> ::= "(" <parameter_list> ")" "=>" (<expression> | <block>)

<new_expression> ::= <type> "." "new" "(" [<argument_list>] ")"

<argument_list> ::= <expression> {"," <expression>}
```

## Statements

### Control Flow

```ebnf
<if_statement> ::=
    "if" <expression> "then"
        <block>
    {"elseif" <expression> "then" <block>}
    ["else" <block>]
    "end"

<while_statement> ::= "while" <expression> "do" <block> "end"

<for_statement> ::=
    | "for" <identifier> [":" <type>] "=" <expression> "," <expression> ["," <expression>] "do" <block> "end"
    | "for" <identifier_list> "in" <expression_list> "do" <block> "end"

<do_statement> ::= "do" <block> "end"
```

### Pattern Matching

```ebnf
<match_statement> ::=
    "match" <expression>
        {"|" <pattern> ["if" <expression>] "->" (<expression> | <block> "end")}
    "end"

<pattern> ::=
    | <identifier_pattern>
    | <literal_pattern>
    | <wildcard_pattern>
    | <array_pattern>
    | <object_pattern>
    | <or_pattern>

<identifier_pattern> ::= <identifier>

<literal_pattern> ::= <literal>

<wildcard_pattern> ::= "_"

<array_pattern> ::= "[" [<pattern_list>] ["," "..." <identifier>] "]"

<object_pattern> ::= "{" [<object_pattern_field> {"," <object_pattern_field>}] "}"

<object_pattern_field> ::= <identifier> ["=" <pattern>]

<or_pattern> ::= <pattern> "|" <pattern>

<pattern_list> ::= <pattern> {"," <pattern>}
```

### Error Handling

```ebnf
<try_statement> ::=
    "try"
        <block>
    {<catch_clause>}
    ["finally" <block>]
    "end"

<catch_clause> ::= "catch" [<identifier> [":" <type>]] <block>

<throw_statement> ::= "throw" <expression>
```

### Module System

```ebnf
<import_statement> ::=
    | "import" "{" <import_specifier_list> "}" "from" <string_literal>
    | "import" "*" "as" <identifier> "from" <string_literal>
    | "import" <identifier> "from" <string_literal>

<import_specifier_list> ::= <import_specifier> {"," <import_specifier>}

<import_specifier> ::= <identifier> ["as" <identifier>] | "type" <identifier>

<export_statement> ::=
    | "export" <declaration>
    | "export" "{" <export_specifier_list> "}"
    | "export" "*" "from" <string_literal>

<export_specifier_list> ::= <export_specifier> {"," <export_specifier>}

<export_specifier> ::= <identifier> ["as" <identifier>]
```

## Literals

```ebnf
<literal> ::=
    | <nil_literal>
    | <boolean_literal>
    | <number_literal>
    | <string_literal>
    | <function_literal>

<nil_literal> ::= "nil"

<boolean_literal> ::= "true" | "false"

<number_literal> ::= <decimal_literal> | <hex_literal> | <binary_literal>

<decimal_literal> ::= <digit> {<digit>} ["." {<digit>}] [<exponent>]

<hex_literal> ::= "0x" <hex_digit> {<hex_digit>}

<binary_literal> ::= "0b" <binary_digit> {<binary_digit>}

<exponent> ::= ("e" | "E") ["+" | "-"] <digit> {<digit>}

<string_literal> ::= '"' {<string_char>} '"' | "'" {<string_char>} "'"

<template_string> ::= "`" {<template_element>} "`"

<template_element> ::= <string_char> | "${" <expression> "}"

<array_literal> ::= "{" [<expression_list>] "}"

<table_literal> ::= "{" [<field_list>] "}"

<field_list> ::= <field> {"," <field>} [","]

<field> ::=
    | "[" <expression> "]" "=" <expression>
    | <identifier> "=" <expression>
    | <expression>

<expression_list> ::= <expression> {"," <expression>}
```

## Decorators

```ebnf
<decorator> ::= "@" <identifier> ["(" [<argument_list>] ")"]
```

## Identifiers

```ebnf
<identifier> ::= <letter> {<letter> | <digit> | "_"}

<letter> ::= "a" | "b" | ... | "z" | "A" | "B" | ... | "Z"

<digit> ::= "0" | "1" | ... | "9"

<hex_digit> ::= <digit> | "a" | ... | "f" | "A" | ... | "F"

<binary_digit> ::= "0" | "1"
```

## Operator Precedence

From highest to lowest:

1. Member access (`.`, `[]`, `:`)
2. Function call (`()`)
3. Unary (`not`, `-`, `#`, `~`)
4. Exponentiation (`^`)
5. Multiplicative (`*`, `/`, `%`, `//`)
6. Additive (`+`, `-`)
7. Concatenation (`..`)
8. Bitwise shift (`<<`, `>>`)
9. Bitwise AND (`&`)
10. Bitwise XOR (`~`)
11. Bitwise OR (`|`)
12. Comparison (`<`, `<=`, `>`, `>=`, `==`, `~=`, `instanceof`)
13. Logical AND (`and`)
14. Logical OR (`or`)
15. Null coalesce (`??`)
16. Ternary (`? :`)
17. Pipe (`|>`)
18. Error chain (`!!`)
19. Assignment (`=`, `+=`, etc.)

## Comments

```ebnf
<comment> ::=
    | "--" {<any_char>} <newline>          /* Single-line comment */
    | "--[[" {<any_char>} "]]"             /* Multi-line comment */
    | "---" {<any_char>} <newline>         /* Documentation comment */
```

## Whitespace

Whitespace (spaces, tabs, newlines) is ignored except in string literals and to separate tokens.

## See Also

- [Language Features](../language/basics.md) — Syntax examples
- [Operators](../language/operators.md) — Operator details
- [Keywords](keywords.md) — Reserved keywords
