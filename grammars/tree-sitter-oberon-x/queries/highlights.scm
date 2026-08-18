; Adapted from tree-sitter-oberon-07 (geekstakulus, MIT) for the node names
; produced by this grammar (tree-sitter-oberon2 / viegasfh, MIT). See NOTICE.

; -- Names

(module
  (module_header (ident) @namespace)
  (module_footer (ident) @namespace))

(const_decl
  (ident_def (ident) @constant))

(type_decl
  (ident_def (ident) @type.definition))

(field_list
  (ident_list
    (ident_def (ident) @property)))

(variable_decl
  (ident_list
    (ident_def (ident) @variable)))

(designator
  (qualident (ident) @variable)
  (selector (ident) @property))

(assignment
  (designator (qualident) @variable))

(fp_section
  (ident) @variable
  (formal_type (qualident) @type))

(formal_params
  (qualident) @type)

; -- Types

(type) @type

(qualident
  (ident) @type.builtin
  (#any-of? @type.builtin
    "CHAR" "BYTE" "BOOLEAN" "INTEGER" "SHORTINT" "LONGINT" "LONGREAL" "REAL" "SET"))

; -- Calls

(procedure_call
  (designator (qualident (ident) @function.call)))

(procedure_call
  (designator
    (qualident (ident) @namespace)
    (selector (ident) @function.call)))

(procedure_call
  (designator (qualident) @function.builtin)
  (#any-of? @function.builtin
    "ABS" "ASH" "CAP" "CHR" "ENTIER" "LEN" "LONG" "MAX" "MIN" "ODD" "ORD" "SHORT" "SIZE"
    "COPY" "DEC" "EXCL" "HALT" "INC" "ASSERT" "INCL" "NEW"))

(procedure_decl
  (procedure_heading
    (ident_def (ident) @function))
  (ident) @function)

; -- Keywords

[
  (kModule)
  (kImport)
] @include

(kReturn) @keyword.return

[
  (kIf)
  (kThen)
  (kElse)
  (kElseif)
  (kCase)
] @conditional

[
  (kFor)
  (kTo)
  (kBy)
  (kWhile)
  (kRepeat)
  (kUntil)
  (kDo)
] @repeat

[
  (kIn)
  (kIs)
  (kOf)
  (kEnd)
  (kVar)
  (kType)
  (kArray)
  (kBegin)
  (kConst)
  (kRecord)
  (kPointer)
  (kProcedure)
] @keyword

[
  (kDiv)
  (kOr)
  (kMod)
] @keyword.operator

; -- Punctuation & operators

[ "(" ")" "[" "]" "{" "}" ] @punctuation.bracket
[ ";" "," ":" ".." "." ] @punctuation.delimiter

[
  "&" "~" "=" "#" "<" "<=" ">" ">=" ":="
  (kPlus)
  (kStar)
  (kMinus)
  (kSlash)
] @operator

; -- Literals and builtin constants

(kNil) @constant.builtin
[ (kTrue) (kFalse) ] @boolean

(number) @number
(string) @string
(comment) @comment
(pragma) @comment

(ERROR) @error
