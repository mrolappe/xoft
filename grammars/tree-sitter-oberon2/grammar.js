const
  letter = /[a-zA-Z]/,
  digit = /[0-9]/,
  hex_digit = /[0-9A-F]/,

  // string = '"' {char} '"' | "'" {char} "'" | digit {hex_digit} "X"
  // The report allows either quote character to delimit a string (docs/language-baseline.md
  // §3), not just double quotes — confirmed against real corpus usage (e.g. AmigaOberon's
  // 'KICK'/'PREF' FourCC tags, Prefs.mod) which uses multi-character single-quoted strings,
  // not just single-character CHAR literals.
  string_literal = choice(
    /"[^"\n]*"/,
    /'[^'\n]*'/,
    seq(digit, repeat(hex_digit), 'X')
  ),

  // ident = letter {letter | digit}
  identifier = seq(letter, repeat(choice(letter, digit))),


  // scale_factor = "E" ["+" | "-"] digit {digit}
  scale_factor = seq('E', choice('+', '-'), digit, repeat(digit)),

  // real = digit {digit} "." {digit} [scale_factor]
  real = seq(
    digit, repeat(digit), '.', 
    repeat(digit), optional(scale_factor)
  );

module.exports = grammar({
  name: 'oberon2',

  externals: $ => [$.comment, $.pragma, $.bracket_pragma, $.assembler_body],

  extras: $ => [$.comment, $.pragma, $.bracket_pragma, /\s/],

  word: $ => $.ident,

  // procedure_decl and definition_proc_decl share the "procedure_heading ';'" prefix;
  // GLR resolves which one matched by whether a procedure_body actually follows.
  conflicts: $ => [[$.procedure_decl, $.definition_proc_decl]],

  rules: {

    // module = "MODULE" ident ";"
    // [import_list]
    // declaration_seq
    // ["BEGIN" statement_seq]
    // "END" ident "."
    //
    // definition module (STJ-Oberon) = "DEFINITION" ident ";"
    // [import_list] declaration_seq "END" ident "."
    // — an interface: procedure declarations are headings only, no body.
    module: $ => choice(
      seq(
        $.module_header,
        optional($.import_list),
        optional($.const_decls),
        optional($.type_decls),
        optional($.variable_decls),
        repeat($.procedure_decls),
        optional(seq(
          $.kBegin,
          optional($.statement_seq)
        )),
        $.module_footer
      ),
      seq(
        $.definition_header,
        optional($.import_list),
        optional($.const_decls),
        optional($.type_decls),
        optional($.variable_decls),
        repeat($.definition_proc_decl),
        $.module_footer
      )
    ),
    module_header: $ => seq($.kModule, $.ident, ';'),
    definition_header: $ => seq($.kDefinition, $.ident, ';'),
    module_footer: $ => seq($.kEnd, $.ident, '.'),

    // definition_proc_decl = procedure_heading ";"
    definition_proc_decl: $ => seq($.procedure_heading, ';'),

    // import_list = "IMPORT" import {"," import}
    import_list: $ => seq(
      $.kImport,
      $.import,
      repeat(seq(
        ",",
        $.import
      )),
      ';'
    ),

    // import = ident [":=" ident]
    // AmigaOberon dialect extension (not in normative EBNF, confirmed via corpus):
    // "*" re-export marker after the local alias, and ":" as an alternate
    // rename operator alongside ":=".
    import: $ => seq(
      $.ident,
      optional('*'),
      optional(seq(
        choice(':=', ':'),
        $.ident
      ))
    ),

    // the following declarations are a bit of a hack
    // because of the empty string problem with the previous
    // one

    // const_decls = "CONST" {const_decl ";"}
    const_decls: $ => seq(
      $.kConst, 
      repeat(seq(
        $.const_decl, ';'
      ))
    ),

    // type_decls = "TYPE" {type_decl ";"}
    type_decls: $=> seq(
      $.kType, 
      repeat(seq(
        $.type_decl, ';'
      ))
    ),

    // variable_decls =["VAR" {variable_decl ";"}]
    variable_decls: $ => seq(
      $.kVar, 
      repeat(seq(
        $.variable_decl, ';'
      ))
    ),

    // procedure_decls = (procedure_decl | forward_decl) ";" | definition_proc_decl
    // AmigaOberon dialect extension (not in normative EBNF, confirmed via corpus): a
    // bodiless procedure heading (no "BEGIN...END", no "^" forward marker either) inside
    // an ordinary MODULE, used by the Interfaces/*.mod system-call wrappers. Structurally
    // identical to `definition_proc_decl` (normally only legal inside DEFINITION modules),
    // so reused as-is rather than duplicating the node.
    procedure_decls: $ => choice(
      seq($.procedure_decl, ';'),
      seq($.forward_decl, ';'),
      $.definition_proc_decl
    ),

    // const_decl = ident_def "=" const_expresion
    const_decl: $ => seq(
      $.ident_def, '=', $.const_expression
    ),

    // const_expression = expression
    const_expression: $ =>  $.expression,

    // type_decl = ident_def "=" (qualident | struct_type)
    type_decl: $ => seq(
      $.ident_def, '=', $.type
    ),

    // struct_type = array_type | record_type | pointer_type | procedure_type
    struct_type: $ => choice(
      $.array_type,
      $.record_type,
      $.pointer_type,
      $.procedure_type
    ),

    // array_type = "ARRAY" length {"," length} "OF" type
    // The length list is optional in practice: corpus files use a length-less
    // `ARRAY OF Type` as a pointer's base type (e.g. `POINTER TO ARRAY OF INTEGER`),
    // the same shorthand formal_type already has for formal parameters.
    array_type: $ => seq(
      $.kArray,
      optional(seq($.length, repeat(seq(',', $.length)))),
      $.kOf, $.type
    ),

    // length = const_expression
    length: $ => $.const_expression,

    // record_type = "RECORD" ["(" base_type ")"] [field_list_seq] "END"
    record_type: $ => seq(
      $.kRecord,
      optional(seq(
        '(',
        $.base_type,
        ')'
      )),
      optional($.field_list_seq),
      $.kEnd
    ),

    // base_type = qualident
    base_type: $ => $.qualident,

    // field_list_seq = field_list {";" field_list}
    // FieldList is itself optional in the EBNF ([...]), same shape as StatementSeq — the
    // corpus relies on a trailing ";" before "END" (e.g. voc's Printer.Mod). Same two-branch
    // fix as statement_seq: every branch must still consume at least one token, so the
    // "totally empty" case is expressed by omitting field_list_seq at the call site
    // (already optional($.field_list_seq) in record_type), not by this rule matching nothing.
    field_list_seq: $ => choice(
      seq($.field_list, repeat(seq(';', optional($.field_list)))),
      repeat1(seq(';', optional($.field_list)))
    ),

    // field_list = [ident_list ":" type]
    field_list: $ => seq(
      $.ident_list, ':', $.type
    ),

    // ident_list = ident_def {"," ident_def}
    ident_list: $ => seq(
      $.ident_def, repeat(seq(',', $.ident_def))
    ),

    // pointer_type = "POINTER" "TO" type
    pointer_type: $ => seq(
      $.kPointer, $.kTo, $.type
    ),

    // procedure_type = "PROCEDURE" [formal_params]
    procedure_type: $ => seq(
      $.kProcedure, optional($.formal_params)
    ),

    // formal_params = "(" [fp_section {";" fp_section}] ")" [":" qualident]
    formal_params: $ => seq(
      "(",
      optional(seq(
        $.fp_section, repeat(seq(';', $.fp_section))
      )),
      ")",
      optional(seq(
        ':', $.qualident
      ))
    ),

    // fp_section = ["VAR"] ident [param_offset] {"," ident [param_offset]} ":" formal_type
    fp_section: $ => seq(
      optional($.kVar),
      $.ident, optional($.param_offset),
      repeat(seq(',', $.ident, optional($.param_offset))),
      ':',
      $.formal_type
    ),

    // param_offset = "{" integer "}"
    // AmigaOberon dialect extension: per-parameter vector-offset metadata paired with
    // a procedure's vector_offset, e.g. `PROCEDURE Foo(x{2}: LONGINT)`.
    param_offset: $ => seq('{', $.integer, '}'),

    // formal_type = {"ARRAY" "OF"} (qualident | procedure_type)
    // The report's FPSection uses full Type (Qualident | ARRAY OF Type | RECORD... |
    // POINTER TO Type | PROCEDURE [FormalPars]); this grammar's formal_type has always been
    // narrower than that. Widened only for the case the corpus actually uses — a PROCEDURE
    // type as a callback parameter (voc's MultiArrays.Mod) — not the full Type recursion.
    formal_type: $ => seq(
      repeat(seq($.kArray, $.kOf)),
      choice($.qualident, $.procedure_type)
    ),

    // qualident = [ident "."] ident
    qualident: $ => prec.left(choice(
      seq($.ident),
      seq(field("qualifier", $.ident), '.', field("property", $.ident)),
    )),

    // ident_def = ident ["*" | "-"]
    ident_def: $ => seq(
      $.ident, optional(choice($.kStar, $.kMinus))
    ),

    // variable_decl = ident_list ":" type
    variable_decl: $ => seq(
      $.ident_list, ':', $.type
    ),

    // type = qualident | struct_type
    type: $ => choice(
      $.qualident,
      $.struct_type
    ),

    // procedure_decl = procedure_heading ";" procedure_body ident
    procedure_decl: $ => seq(
      $.procedure_heading, ';', $.procedure_body, $.ident
    ),

    // procedure_heading = "PROCEDURE" [receiver] ident_def [vector_offset] [formal_params]
    procedure_heading: $ => seq(
      $.kProcedure, optional($.receiver), $.ident_def, optional($.vector_offset), optional($.formal_params)
    ),

    // vector_offset = "{" ident "," "-" integer "}"
    // AmigaOberon dialect extension (not in normative EBNF, confirmed via corpus): a
    // library base-relative vector-offset annotation on a procedure heading, e.g.
    // `PROCEDURE Foo*{base,-54}(...)`. The base name varies ("base", "cwBase", ...);
    // the offset is always negative and may be decimal or hex.
    vector_offset: $ => seq(
      '{', $.ident, ',', '-', $.integer, '}'
    ),

    // receiver = "(" ["VAR"] ident ":" ident ")"
    receiver: $ => seq(
      '(', optional($.kVar), $.ident, ':', $.ident, ')'
    ),

    // forward_decl = "PROCEDURE" "^" [receiver] ident_def [formal_params]
    forward_decl: $ => seq(
      $.kProcedure, '^', optional($.receiver), $.ident_def, optional($.formal_params)
    ),

    // procedure_body = declaration_seq ["BEGIN" statement_seq] "END"
    // RETURN is an ordinary statement (see `statement`), not modeled here —
    // the report's "RETURN only at the end" restriction isn't reflected in
    // the EBNF's Statement production and the corpus uses RETURN mid-body
    // (early return inside IF branches).
    procedure_body: $ => seq(
      //$.declaration_seq,
      optional($.const_decls),
      optional($.type_decls),
      optional($.variable_decls),
      repeat($.procedure_decls),
      optional(seq($.kBegin, optional($.statement_seq))),
      $.kEnd
    ),

    // expression = simple_expression [relation simple_expression]
    expression: $ => seq(
      $.simple_expression,
      optional(seq($.relation, $.simple_expression))
    ),

    // relation = "=" | "#" | "<" | "<=" | ">" | ">=" | "IN" | "IS"
    relation: $ => choice(
      '=',
      '#',
      '<',
      '<=',
      '>',
      '>=',
      $.kIn,
      $.kIs
    ),

    // simple_expression = ["+" | "-"] term {add_operator term}
    simple_expression: $ => seq(
      optional(choice(
        $.kPlus,
        $.kMinus
      )),
      $.term,
      repeat(seq(
        $.add_operator, $.term
      ))
    ),

    // add_operator = "+" | "-" | "OR"
    add_operator: $ => choice(
      $.kPlus,
      $.kMinus,
      $.kOr
    ),

    // term = factor {mul_operator factor}
    term: $ => seq(
      $.factor, repeat(seq($.mul_operator, $.factor))
    ),

    // mul_operator = "*" | "/" | "DIV" | "MOD" | "&"
    mul_operator: $ => choice(
      $.kStar,
      $.kSlash,
      $.kDiv,
      $.kMod,
      '&'
    ),

    // factor = number | string | "NIL" | "TRUE" | "FALSE" |
    //          set | designator [actual_params] | "(" expression ")" | "~" factor
    factor: $ => choice(
      $.number,
      $.string,
      $.kNil,
      $.kTrue,
      $.kFalse,
      $.set,
      seq($.designator, optional($.actual_params)),
      seq('(', $.expression, ')'),
      seq('~', $.factor)
    ),

    // designator = qualident {selector}
    designator: $ => prec.left(seq(
      $.qualident,
      repeat($.selector)
    )),

    // selector = "." ident | "[" expression_list "]" | "^" | "(" qualident ")"
    selector: $ => choice(
      seq('.', $.ident),
      seq('[', $.expression_list, ']'),
      '^',
      seq('(', $.qualident, ')')
    ),

    // set = "{" [element {"," element}] "}"
    set: $ => seq(
      '{',
      optional(seq(
        $.element, repeat(seq(',', $.element))
      )),
      '}'
    ),

    // element = expression [".." expression]
    element: $ => seq(
      $.expression,
      optional(seq(
        '..', $.expression
      ))
    ),

    // expression_list = expression {"," expression}
    expression_list: $ => seq(
      $.expression, repeat(seq(',', $.expression))
    ),

    // actual_params = "(" [expression_list] ")"
    actual_params: $ => seq(
      '(', optional($.expression_list), ')'
    ),

    // statement = assignment | procedure_call | if_statement | case_statement |
    //              while_statement | repeat_statement | for_statement |
    //              loop_statement | with_statement | exit_statement |
    //              return_statement
    statement: $ => choice(
      $.assignment,
      $.procedure_call,
      $.if_statement,
      $.case_statement,
      $.while_statement,
      $.repeat_statement,
      $.for_statement,
      $.loop_statement,
      $.with_statement,
      $.exit_statement,
      $.return_statement,
      $.assembler_statement
    ),

    // assignment = designator ":=" expression
    assignment: $ => seq(
      $.designator, ':=', $.expression
    ),

    // procedure_call = designator [actual_params]
    procedure_call: $ => seq(
      $.designator, optional($.actual_params)
    ),

    // statement_seq = statement {";" statement}
    // Statement is itself optional in the EBNF ([...]), so every element of
    // the sequence may be empty — an empty statement isn't a kind of
    // statement, it's the absence of one, so it isn't its own node kind.
    // (statement_seq itself must still consume at least one token — a
    // wholly empty sequence is expressed by omitting it at the call site,
    // e.g. `optional($.statement_seq)` — tree-sitter rejects any rule that
    // can match the empty string.)
    statement_seq: $ => choice(
      seq($.statement, repeat(seq(';', optional($.statement)))),
      repeat1(seq(';', optional($.statement)))
    ),

    // if_statement = "IF" expression "THEN" statement_seq
    //                {"ELSIF" expression "THEN" statement_seq}
    //                ["ELSE" statement_seq] "END"
    if_statement: $ => seq(
      $.kIf, $.expression, $.kThen, optional($.statement_seq),
      repeat(seq($.kElseif, $.expression, $.kThen, optional($.statement_seq))),
      optional(seq($.kElse, optional($.statement_seq))),
      $.kEnd
    ),

    // case_statement = "CASE" expression "OF" case {"|" case} "END"
    case_statement: $ => seq(
      $.kCase, $.expression, $.kOf, optional($.case_clause), repeat(seq('|', optional($.case_clause))), $.kEnd
    ),

    // case = [case_label_list ":" statement_sequence]
    case_clause: $ => seq($.case_label_list, ':', optional($.statement_seq)),

    // case_label_list = label_range {"," label_range}
    case_label_list: $ => seq(
      $.label_range,
      repeat(seq(',', $.label_range))
    ),

    // label_range = label [".." label]
    label_range: $ => seq(
      $.label, optional(seq('..', $.label))
    ),

    // label = integer | string | qualident
    label: $ => choice(
      $.integer,
      $.string,
      $.qualident
    ),

    // while_statement = "WHILE" expression "DO" statement_sequence
    //                   {"ELSIF" expression "DO" statement_sequence} "END"
    while_statement: $ => seq(
      $.kWhile, $.expression, $.kDo, optional($.statement_seq),
      repeat(seq($.kElseif, $.expression, $.kDo, optional($.statement_seq))),
      $.kEnd
    ),

    // repeat_statement = "REPEAT" statement_seq "UNTIL" expression
    repeat_statement: $ => seq(
      $.kRepeat, optional($.statement_seq), $.kUntil, $.expression
    ),

    // for_statement = "FOR" ident ":=" expression "TO" expression ["BY" const_expression]
    //                 "DO" statement_seq "END"
    for_statement: $ => seq(
      $.kFor, $.ident, ':=', $.expression, $.kTo, $.expression,
      optional(seq($.kBy, $.const_expression)),
      $.kDo, optional($.statement_seq), $.kEnd
    ),

    // loop_statement = "LOOP" statement_seq "END"
    loop_statement: $ => seq(
      $.kLoop, optional($.statement_seq), $.kEnd
    ),

    // exit_statement = "EXIT"
    exit_statement: $ => $.kExit,

    // return_statement = "RETURN" [expression]
    return_statement: $ => seq(
      $.kReturn, optional($.expression)
    ),

    // with_statement = "WITH" guard "DO" statement_seq
    //                  {"|" guard "DO" statement_seq}
    //                  ["ELSE" statement_seq] "END"
    with_statement: $ => seq(
      $.kWith, $.with_arm, repeat(seq('|', $.with_arm)),
      optional(seq($.kElse, optional($.statement_seq))),
      $.kEnd
    ),

    with_arm: $ => seq(
      $.guard, $.kDo, optional($.statement_seq)
    ),

    // guard = qualident ":" qualident
    guard: $ => seq(
      $.qualident, ':', $.qualident
    ),

    // assembler_statement = "ASSEMBLER" assembler_body "END"
    // STJ-Oberon dialect extension (not in normative EBNF, confirmed via corpus): raw M68K
    // assembly embedded as a statement inside a procedure body. assembler_body is an opaque
    // external token (src/scanner.c) since its content — opcodes, "(A0,D0.L)" addressing,
    // "#" immediates, "D0-A7" register ranges — doesn't tokenize as Oberon.
    assembler_statement: $ => seq(
      $.kAssembler, $.assembler_body, $.kEnd
    ),

    string: $ => token(string_literal),
    // number = integer | real
    number: $ => choice($.integer, token(real)),

    // integer = digit {digit} | digit {hex_digit} "H"
    integer: $ => choice(
      token(seq(digit, repeat(digit))),
      token(seq(digit, repeat(hex_digit), 'H'))
    ),
    
    // mathematical operators
    kPlus:  $ => '+',
    kStar:  $ => '*',
    kMinus: $ => '-',
    kSlash: $ => '/',

    // keywords
    kBy: $ => 'BY',
    kDo: $ => 'DO',
    kIf: $ => 'IF',
    kIn: $ => 'IN',
    kIs: $ => 'IS',
    kOf: $ => 'OF',
    kOr: $ => 'OR',
    kTo: $ => 'TO',

    kDiv: $ => 'DIV',
    kEnd: $ => 'END',
    kFor: $ => 'FOR',
    kMod: $ => 'MOD',
    kNil: $ => 'NIL',
    kVar: $ => 'VAR',
    kExit: $ => 'EXIT',
    kLoop: $ => 'LOOP',
    kWith: $ => 'WITH',

    kCase: $ => 'CASE',
    kElse: $ => 'ELSE',
    kThen: $ => 'THEN',
    kTrue: $ => 'TRUE',
    kType: $ => 'TYPE',

    kArray: $ => 'ARRAY',
    kBegin: $ => 'BEGIN',
    kConst: $ => 'CONST',
    kFalse: $ => 'FALSE',
    kUntil: $ => 'UNTIL',
    kWhile: $ => 'WHILE',

    kDefinition: $ => 'DEFINITION',
    kElseif: $ => 'ELSIF',
    kImport: $ => 'IMPORT',
    kModule: $ => 'MODULE',
    kRecord: $ => 'RECORD',
    kRepeat: $ => 'REPEAT',
    kReturn: $ => 'RETURN',

    kPointer: $ => 'POINTER',

    kProcedure: $ => 'PROCEDURE',

    kAssembler: $ => 'ASSEMBLER',

    ident: $ => token(identifier),
  }
});
