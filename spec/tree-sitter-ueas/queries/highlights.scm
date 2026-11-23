(line_comment) @comment
(block_comment) @comment

(string_literal) @string
(int_literal) @number
(real_literal) @number.float
(bool_literal) @boolean
(none_literal) @constant.builtin

(algorithm_decl name: (identifier) @function)
(record_decl (identifier) @type.definition)
(type_expr (identifier) @type)
(decorator "@" @attribute (identifier) @attribute)
