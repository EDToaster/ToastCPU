; Tree-sitter highlight queries for ToastLang (.tl)
; Capture names follow Helix's highlight scopes.

; ---------------------------------------------------------------------------
; Declarations (matched by field so they win over the generic `word` rule)
; ---------------------------------------------------------------------------
(function_definition name: (identifier) @function)
(struct_definition   name: (identifier) @type)
(module              name: (identifier) @namespace)
(using               name: (identifier) @namespace)
(global              name: (identifier) @variable)
(inline              name: (identifier) @constant)
(struct_member       name: (identifier) @variable.other.member)
(let                 binding: (identifier) @variable.parameter)
(ptr_expression      target: (identifier) @variable)

; ---------------------------------------------------------------------------
; Types
; ---------------------------------------------------------------------------
(type_identifier) @type
(generic_type (type_identifier) @type.parameter)

((type_identifier) @type.builtin
 (#match? @type.builtin "^(u16|bool)$"))

; ---------------------------------------------------------------------------
; Words / calls and field access
; ---------------------------------------------------------------------------
(word) @function
(struct_access) @variable.other.member

; ---------------------------------------------------------------------------
; Literals
; ---------------------------------------------------------------------------
(number) @constant.numeric
(char) @constant.character
(string) @string
(boolean) @constant.builtin.boolean
(comment) @comment

; ---------------------------------------------------------------------------
; Keywords
; ---------------------------------------------------------------------------
"fn" @keyword.function
["struct" "global" "inline"] @keyword.storage.type
"let" @keyword.storage
["mod" "using"] @keyword.control.import
["if" "else"] @keyword.control.conditional
["while" "unroll"] @keyword.control.repeat
"return" @keyword.control.return
["as" "sizeof" "ptr"] @keyword.operator

; ---------------------------------------------------------------------------
; Operators and punctuation
; ---------------------------------------------------------------------------
[
  "||" "&&" "!"
  "++" "--"
  "+" "-"
  "|" "&" "~"
  ">>>" ">>" "<<" "^"
  "=" "<=" ">=" "<" ">"
  "??"
  "*" "$" "->"
] @operator

["(" ")" "{" "}" "[" "]"] @punctuation.bracket
