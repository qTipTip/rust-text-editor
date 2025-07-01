(line_comment) @comment
(block_comment) @comment

(string_literal) @string
(raw_string_literal) @string
(char_literal) @string

(integer_literal) @number
(float_literal) @number

"fn" @keyword
"let" @keyword
"if" @keyword
"else" @keyword
"for" @keyword
"while" @keyword
"loop" @keyword
"match" @keyword
"return" @keyword

; Function definitions
(function_item
  name: (identifier) @function)

; Function calls
(call_expression
  function: (identifier) @function)

; Method calls
(call_expression
  function: (field_expression
    field: (field_identifier) @function.method))