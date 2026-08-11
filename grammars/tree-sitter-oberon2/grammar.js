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
  // Oberon-A/AmigaOberon/voc dialect extension (round 21, corpus-confirmed across 72
  // oberon-a + 35 amiga-oberon-31 + 13 voc files): underscore as an identifier
  // continuation character, e.g. `TYPE_HGROUP`, `SM_MINSIZE` — C-header-derived Amiga API
  // constant names. Without it "TYPE_HGROUP" lexes as keyword `TYPE` followed by an
  // unparseable `_HGROUP`, since `_` isn't a valid token-start on its own.
  identifier = seq(letter, repeat(choice(letter, digit, '_'))),


  // scale_factor = ("E" | "D") ["+" | "-"] digit {digit}
  // docs/language-baseline.md's ScaleFactor requires a sign and >=1 exponent digit whenever
  // a scale factor appears at all, but corpus usage (oberon-a, amiga-oberon-31) diverges in
  // two ways confirmed via grep: the sign is often omitted even with exponent digits present
  // (`9.22337177E18`), and AmigaOberon's "D" (LONGREAL literal) marker is consistently used
  // bare, with no sign or digits at all (`3.141592653589793D`). Both the sign and the
  // digit{digit} tail are made optional here to cover both without over-narrowing.
  scale_factor = seq(
    choice('E', 'D'), optional(seq(optional(choice('+', '-')), digit, repeat(digit)))
  ),

  // real = digit {digit} "." {digit} [scale_factor]
  // The report allows zero digits after the "." (bare "2."), but no real-world corpus code
  // relies on that, and it makes "2." ambiguous with the range operator's leading digit in
  // "2..4" — the maximal-munch lexer would greedily swallow the first "." into `real`,
  // leaving one "." where `element`'s ".." literal needs two. Requiring at least one digit
  // after the "." disambiguates without an external scanner.
  real = seq(
    digit, repeat(digit), '.',
    digit, repeat(digit), optional(scale_factor)
  );

module.exports = grammar({
  name: 'oberon2',

  externals: $ => [$.comment, $.pragma, $.bracket_pragma, $.assembler_body],

  // \s alone doesn't cover U+00A0 (non-breaking space): confirmed via corpus grep, several
  // AmigaOberon files (BasicTypes.mod, Lists.mod, FArrays.mod) use a literal NBSP byte as
  // inter-token whitespace after a procedure heading's ";", not just inside comment prose.
  extras: $ => [$.comment, $.pragma, $.bracket_pragma, /[\s\u00a0]/],

  word: $ => $.ident,

  // procedure_decl and definition_proc_decl share the "procedure_heading ';'" prefix;
  // GLR resolves which one matched by whether a procedure_body actually follows.
  conflicts: $ => [
    [$.procedure_decl, $.definition_proc_decl],
    [$.selector, $.actual_params],
    // "PROCEDURE -ident" is ambiguous between voc's external_proc_decl (round 20) and
    // STJ-Oberon's trap-bound procedure_heading (round 22) until the tokens after the
    // heading (a trailing string vs. ";"/trap_offset) disambiguate which dialect it is.
    [$.external_proc_decl, $.kMinus]
  ],

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
        repeat(choice($.const_decls, $.type_decls, $.variable_decls)),
        repeat($.procedure_decls),
        // AmigaOberon dialect extension (not in normative EBNF, confirmed via corpus): a
        // module-level "CLOSE" section after "BEGIN", holding a finalizer statement sequence
        // run on module unload.
        optional(seq(
          $.kBegin,
          optional($.statement_seq),
          optional(seq($.kClose, optional($.statement_seq)))
        )),
        $.module_footer
      ),
      seq(
        $.definition_header,
        optional($.import_list),
        repeat(choice($.const_decls, $.type_decls, $.variable_decls)),
        repeat($.definition_proc_decl),
        $.module_footer
      )
    ),
    // Oberon-A dialect extension (round 21, corpus-confirmed in Classface.mod,
    // Obsolete/BoopsiUtil.mod, Obsolete/RexxUtil.mod): a module-level external object-file
    // name, e.g. `MODULE [4] Classface ["Classface.o"];` — same bracketed-string shape
    // `external_code_names` already gives a procedure heading, reused here rather than
    // duplicating the node.
    module_header: $ => seq(
      $.kModule, optional($.sysflag), $.ident, optional($.external_code_names), ';'
    ),

    // sysflag = "[" integer "]"
    // Oberon-A dialect extension (not in normative EBNF, confirmed via corpus and docs/OC.doc):
    // a "system flag" marking a MODULE, POINTER, RECORD or PROCEDURE as following a foreign
    // (non-Oberon) calling/layout convention — 1=Modula-2, 2=C, 3=BCPL, 4=Assembly. Placed
    // directly after the keyword it modifies, before whatever normally follows.
    sysflag: $ => seq('[', $.integer, ']'),
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
      $.definition_proc_decl,
      $.external_proc_decl
    ),

    // external_proc_decl = "PROCEDURE" "-" ident_def [formal_params] string ";"
    // vishap oberon compiler (voc) dialect extension (not in normative EBNF, confirmed via
    // corpus, 56 occurrences across oocX11.Mod/oocXYplane.Mod/oocXutil.Mod/
    // oocwrapperlibc.Mod/ulmSysStat.Mod): a bodiless procedure heading marked with a leading
    // "-", implemented by a literal C-source string trailing the heading instead of a
    // BEGIN...END body, e.g. `PROCEDURE -sprntf(s: ARRAY OF CHAR): INTEGER
    // "sprintf((char*)s)";`. voc splices the string into its generated C output at each
    // call site. No receiver form found in the corpus.
    external_proc_decl: $ => seq(
      $.kProcedure, '-', $.ident_def, optional($.formal_params), $.string, ';'
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
      $.procedure_type,
      $.amiga_struct_type
    ),

    // amiga_struct_type = "STRUCT" ["(" field_list ")"] [field_list_seq] "END"
    // AmigaOberon dialect extension (not in normative EBNF — AmigaOberon is based on
    // Wirth's original Oberon report, not Oberon-2, see docs/language-baseline.md): a
    // C-interop struct type for foreign (non-GC-tracked) Amiga library structures, always
    // paired with UNTRACED POINTER. Structurally close to record_type, but the parenthesized
    // "base" slot is a single embedded named field (ident ":" type), C-struct-embedding
    // style, not a bare base-type reference — confirmed via corpus (OberonLib.mod's `Node =
    // STRUCT (dummy: CommonNode) succ: NodePtr; ... END`). Reuses field_list_seq's own
    // leading-";" continuation branch for what follows the parenthesized field (confirmed by
    // OberonLib.mod's `MemElement = STRUCT (node: MinNode); size: LONGINT; ... END`), so no
    // separate rule is needed for the "; more fields" tail.
    amiga_struct_type: $ => seq(
      $.kStruct,
      optional(seq('(', $.field_list, ')')),
      optional($.field_list_seq),
      $.kEnd
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
      optional($.sysflag),
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
    // STJ-Oberon dialect extension (confirmed via corpus, e.g. DEF/BINTREE.DEF,
    // DEF/CDCL.DEF, DEF/STACK.DEF): a DEFINITION module's RECORD body may list its
    // type-bound procedure headings directly as field_list items, interleaved with
    // ordinary fields, instead of declaring them separately at module level (the
    // module-level `definition_proc_decl` shape). Reuses `procedure_heading` bare
    // (not `definition_proc_decl`) since `field_list_seq` already supplies the ";"
    // separator between items.
    field_list: $ => choice(
      seq($.ident_list, ':', $.type),
      $.procedure_heading
    ),

    // ident_list = ident_def {"," ident_def}
    ident_list: $ => seq(
      $.ident_def, repeat(seq(',', $.ident_def))
    ),

    // pointer_type = ["UNTRACED"] "POINTER" "TO" type | "BPOINTER" "TO" type
    // AmigaOberon dialect extension (not in normative EBNF, confirmed via corpus): "UNTRACED"
    // marks a pointer as not tracked/scanned by the garbage collector, always paired in the
    // corpus with an amiga_struct_type (or otherwise foreign-layout) target, e.g. `UNTRACED
    // POINTER TO TypDesc`. "BPOINTER" is AmigaDOS's own BCPL-relative pointer type — a
    // distinct keyword replacing "POINTER" entirely rather than modifying it (confirmed via
    // corpus, e.g. Dos.mod's `FileLockPtr* = BPOINTER TO FileLock;`, never "BPOINTER POINTER
    // TO").
    pointer_type: $ => choice(
      seq(optional($.kUntraced), $.kPointer, optional($.sysflag), $.kTo, $.type),
      seq($.kBPointer, $.kTo, $.type)
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
      $.ident, optional(choice($.param_offset, $.reg_spec)),
      repeat(seq(',', $.ident, optional(choice($.param_offset, $.reg_spec)))),
      ':',
      $.formal_type
    ),

    // param_offset = "{" integer "}" [".."]
    // AmigaOberon dialect extension: per-parameter vector-offset metadata paired with
    // a procedure's vector_offset, e.g. `PROCEDURE Foo(x{2}: LONGINT)`. Trailing ".." is
    // reg_spec's varargs marker, sibling here too (e.g. `data{9}..: SYSTEM.ADDRESS`).
    param_offset: $ => seq('{', $.integer, '}', optional('..')),

    // reg_spec = "[" integer "]" [".."]
    // Oberon-A dialect extension (docs/OC.doc "RegPars"): square-bracket sibling of
    // param_offset — the CPU register (0..15: D0-D7, A0-A7) a library-call/external-code
    // parameter is passed in. A trailing ".." marks the (always last) parameter as a
    // variable-length argument list.
    reg_spec: $ => seq('[', $.integer, ']', optional('..')),

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
    // AmigaOberon dialect extension (not in normative EBNF, confirmed via corpus, always
    // exactly one identifier, never a comma list): a variable's identifier may carry an
    // absolute hardware-address annotation "[" integer "]" mapping it onto a fixed memory
    // location (custom-chip register), e.g. `Ciapra[0BFE001H]: SHORTSET;`. Modeled as a
    // sibling addressed_ident alternative rather than folded into the shared ident_list,
    // which record fields and formal parameters also use and never carry this.
    variable_decl: $ => seq(
      choice($.ident_list, $.addressed_ident), ':', $.type
    ),

    // addressed_ident = ident_def address
    addressed_ident: $ => seq($.ident_def, $.address),

    // address = "[" integer "]"
    address: $ => seq('[', $.integer, ']'),

    // type = qualident | struct_type
    type: $ => choice(
      $.qualident,
      $.struct_type
    ),

    // procedure_decl = procedure_heading ";" procedure_body ident
    procedure_decl: $ => seq(
      $.procedure_heading, ';', $.procedure_body, $.ident
    ),

    // procedure_heading = "PROCEDURE" ["*" | "-" | "~"] [sysflag] [receiver] ident_def
    //                     [vector_offset | square_vector_offset | external_code_names]
    //                     [formal_params] [trap_offset]
    // The "*" right after PROCEDURE (before sysflag/receiver/ident) is Oberon-A's
    // "assignable procedure" mark (docs/OC.doc "AssignableProcs"): it allows a procedure to
    // be assigned to a procedure variable without being exported, e.g. `PROCEDURE* [0] Foo`.
    // The "-" in the same slot is STJ-Oberon's (confirmed via corpus, e.g. LIBRARY.PRJ/
    // BIOS.MOD, GEMDOS.MOD, XBIOS.MOD) marker for a procedure bound directly to a GEMDOS/
    // BIOS/XBIOS trap, always paired with a trailing `trap_offset`. The "~" is STJ-Oberon's
    // sibling mark on a *nested* (locally-declared, inside another procedure's body)
    // procedure (confirmed via corpus, e.g. LIBRARY.PRJ/TASK.MOD, PROCLIST.MOD): every
    // corpus occurrence assigns the nested procedure's ident to a procedure variable, so
    // it plays the same "assignable" role "*" plays at module level, just spelled
    // differently since nested procedures can't carry an export mark.
    procedure_heading: $ => seq(
      $.kProcedure, optional(choice($.kStar, $.kMinus, '~')), optional($.sysflag),
      optional($.receiver), $.ident_def,
      optional(choice(
        $.vector_offset, $.square_vector_offset, $.external_code_names,
        $.curly_external_code_names
      )),
      optional($.formal_params),
      optional($.trap_offset)
    ),

    // trap_offset = integer "," integer
    // STJ-Oberon dialect extension (not in normative EBNF, confirmed via corpus): the
    // GEMDOS/BIOS/XBIOS trap number and function number of a "PROCEDURE-"-marked system-call
    // binding, e.g. `PROCEDURE- Bconout*(Char,Device : INTEGER) 3,13;`.
    trap_offset: $ => seq($.integer, ',', $.integer),

    // vector_offset = "{" ident "," "-" integer "}"
    // AmigaOberon dialect extension (not in normative EBNF, confirmed via corpus): a
    // library base-relative vector-offset annotation on a procedure heading, e.g.
    // `PROCEDURE Foo*{base,-54}(...)`. The base name varies ("base", "cwBase", ...);
    // the offset is always negative and may be decimal or hex.
    vector_offset: $ => seq(
      '{', $.ident, ',', '-', $.integer, '}'
    ),

    // square_vector_offset = "[" ident "," ["-"] integer "]"
    // Oberon-A dialect extension (docs/OC.doc "LibCalls"): square-bracket sibling of
    // vector_offset for an Amiga library-call heading, e.g. `PROCEDURE Foo*[base,-6](...)`.
    // Unlike vector_offset the leading "-" is optional in the compiler's own grammar.
    square_vector_offset: $ => seq(
      '[', $.ident, ',', optional('-'), $.integer, ']'
    ),

    // external_code_names = "[" string {"," string} "]"
    // Oberon-A dialect extension (docs/OC.doc "ExternalCode"): the linker symbol name(s) of
    // an externally-compiled procedure, e.g. `PROCEDURE Foo* ["_Foo"](...)`.
    external_code_names: $ => seq(
      '[', $.string, repeat(seq(',', $.string)), ']'
    ),

    // curly_external_code_names = "{" string {"," string} "}"
    // AmigaOberon dialect extension (not in normative EBNF, confirmed via corpus): curly-brace
    // sibling of external_code_names, e.g. `PROCEDURE Foo*{"Foo.Bar"}(...)`. Distinguishable
    // from vector_offset (also "{"-led) by its first token being a string, not an ident.
    curly_external_code_names: $ => seq(
      '{', $.string, repeat(seq(',', $.string)), '}'
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
    // Nested declarations are deliberately narrower than `procedure_decls` (round 21):
    // baseline DeclSeq (docs/language-baseline.md) only nests `ProcDecl ";" | ForwardDecl
    // ";"`, never a bodyless heading. Corpus-confirmed, e.g. Oberon-A's Amiga library-interface
    // files declare dozens of consecutive bodyless `definition_proc_decl`s at MODULE level, and
    // that ambiguity is genuinely unresolvable until a real "END" is reached — with
    // `definition_proc_decl` included here too, GLR must keep every possible nesting of N
    // consecutive bodyless headings live simultaneously (is proc 2 nested in proc 1's body, or
    // proc 1's sibling? proc 3 in proc 1, proc 2, or a sibling of both?), a combinatorial
    // (Catalan-like) blowup that overwhelms the parser past ~7 consecutive bodyless procedures
    // in a row (confirmed by direct bisection, not guessed). Since a bodyless heading is never
    // legitimately nested inside another procedure's body anyway (it's a top-level-only library
    // stub, not a local declaration), excluding it here removes the ambiguity entirely without
    // losing any real construct: every bodyless heading always has a genuine, nearby "END" to
    // anchor against.
    procedure_body: $ => seq(
      //$.declaration_seq,
      repeat(choice($.const_decls, $.type_decls, $.variable_decls)),
      repeat(choice(seq($.procedure_decl, ';'), seq($.forward_decl, ';'))),
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
    // STJ-Oberon (Atari ST) also accepts "AND" as a textual synonym for "&" —
    // confirmed via corpus (both spellings coexist in the same files) and the
    // compiler's own embedded keyword table; a lexical dialect extension per
    // D1, not a structural one, so no scoping question.
    mul_operator: $ => choice(
      $.kStar,
      $.kSlash,
      $.kDiv,
      $.kMod,
      '&',
      $.kAnd
    ),

    // factor = number | string | "NIL" | "TRUE" | "FALSE" |
    //          set | designator [actual_params] | "(" expression ")" | "~" factor
    // STJ-Oberon also accepts "NOT" as a textual synonym for "~" (same corpus
    // evidence as "AND" above, often used together). designator already folds
    // actual_params into its own repeat (see below), so factor no longer needs a
    // separate trailing slot for it.
    // Oberon-A dialect extension (round 21, corpus evidence across 14 oberon-a files,
    // e.g. OC.mod's `template = "NS=..." (* comment *) ",FORCE/S";`): adjacent string
    // literals concatenate, C-style, used to spread long CONST strings and data tables
    // across lines with a per-line comment. A single string factor is unaffected (still
    // exactly one $.string child); this only adds the 2+-strings case, which was
    // previously an ERROR.
    factor: $ => choice(
      $.number,
      seq($.string, repeat($.string)),
      $.kNil,
      $.kTrue,
      $.kFalse,
      $.set,
      $.typed_set,
      $.designator,
      seq('(', $.expression, ')'),
      seq('(', $.assignment, ')'),
      seq('~', $.factor),
      seq($.kNot, $.factor)
    ),

    // designator = qualident {selector}
    // The report keeps a designator's trailing call ("(" [ExpList] ")", ActualParameters)
    // separate from selector's type guard ("(" qualident ")"), bolted on only once at the
    // very end by factor/procedure_call. But a single bare-identifier argument is exactly
    // the same token sequence either way, and real Oberon-2 compilers resolve which one it
    // is via the symbol table (is the name a type?) — information this syntax-only grammar
    // doesn't have. Corpus evidence (AmigaOberon's COMPLEX.mod, VECTOR.mod, SecureDos.mod)
    // needs guards and calls to freely interleave and chain, e.g. `n(COMPLEX).Norm()`
    // (guard, then field, then call) — so actual_params joins selector in one repeating
    // choice here instead of a single trailing slot that can't be followed by more
    // selectors. See the `conflicts` entry pairing them: the ambiguous case (parenthesized
    // single qualident) is genuinely undecidable without semantic info, so GLR explores
    // both and keeps whichever lets the rest of the input parse.
    designator: $ => prec.left(seq(
      $.qualident,
      repeat(choice($.selector, $.actual_params))
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

    // typed_set = qualident set
    // AmigaOberon dialect extension (not in normative EBNF, confirmed via corpus): a
    // type-qualified set constructor for the dialect's fixed-width SET types, e.g.
    // LONGSET{1, 2..4} / SHORTSET{}.
    typed_set: $ => seq($.qualident, $.set),

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

    // assignment = designator ":=" (expression | assignment)
    // STJ-Oberon dialect extension (docs/STJ-OBN.TXT "Assigment expressions", "extended
    // mode"): the RHS may itself be another assignment, chaining without parens (e.g.
    // `a := b := proc();`, confirmed straight from the compiler manual). Parenthesized, an
    // assignment can also appear nested inside a larger expression (factor's "(" assignment
    // ")" alternative below) — confirmed via corpus, e.g. LIBRARY.PRJ/MODELLIS.MOD's
    // `IF (answer := self.First()) = NIL THEN`.
    assignment: $ => seq(
      $.designator, ':=', choice($.assignment, $.expression)
    ),

    // procedure_call = designator [actual_params]
    // actual_params already folds into designator's own repeat (see designator).
    procedure_call: $ => $.designator,

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

    // case_statement = "CASE" expression "OF" case {"|" case} ["ELSE" statement_seq] "END"
    case_statement: $ => seq(
      $.kCase, $.expression, $.kOf, optional($.case_clause), repeat(seq('|', optional($.case_clause))),
      optional(seq($.kElse, optional($.statement_seq))),
      $.kEnd
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

    // label = ConstExpr, per the normative baseline (this grammar had narrowed it to just
    // integer/string/qualident, missing arithmetic on named constants). Confirmed corpus
    // need, not a dialect extension: SYSTEM.PRJ/OCASSEMB.MOD's negated condition-code
    // constants (`-FNE: RETURN FEQ;`) and SYSTEM.PRJ/OCASSOPT.MOD's offset labels
    // (`Expr.Set-1, Expr.Set+1..Expr.DynArr:`) both need more than a bare qualident.
    // const_expression already covers integer/string/qualident too, so this replaces
    // rather than extends the old choice.
    label: $ => $.const_expression,

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

    // return_statement = "RETURN" ["^"] [expression]
    // The "^" is an STJ-Oberon dialect extension (not in normative EBNF, confirmed via
    // corpus, e.g. LIBRARY.PRJ/TASK.MOD, OBJFILE.MOD, LISTVIEW.MOD): always immediately
    // after "RETURN", before the optional expression.
    return_statement: $ => seq(
      $.kReturn, optional('^'), optional($.expression)
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
    // AmigaOberon dialect extension (not in normative EBNF, confirmed via corpus): a "U"
    // suffix as a sibling of "H", denoting an unsigned hex literal (e.g. 016C0U) — used
    // throughout for raw machine-code words passed to SYSTEM.INLINE and hex bit-mask
    // constants.
    integer: $ => choice(
      token(seq(digit, repeat(digit))),
      token(seq(digit, repeat(hex_digit), 'H')),
      token(seq(digit, repeat(hex_digit), 'U'))
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

    kAnd: $ => 'AND',
    kDiv: $ => 'DIV',
    kEnd: $ => 'END',
    kFor: $ => 'FOR',
    kMod: $ => 'MOD',
    kNil: $ => 'NIL',
    kNot: $ => 'NOT',
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
    kClose: $ => 'CLOSE',
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
    kUntraced: $ => 'UNTRACED',
    kBPointer: $ => 'BPOINTER',
    kStruct: $ => 'STRUCT',

    kProcedure: $ => 'PROCEDURE',

    kAssembler: $ => 'ASSEMBLER',

    ident: $ => token(identifier),
  }
});
