# Language baseline

**Decision:** the xoft base grammar targets **Oberon-2** as defined by

> H. Mössenböck, N. Wirth: *The Programming Language Oberon-2*.
> Institut für Computersysteme, ETH Zürich, March 1995 revision.

Normative copies consulted:
- <https://ssw.jku.at/Research/Papers/Oberon2.pdf> (author's institute)
- <https://cseweb.ucsd.edu/~wgg/CSE131B/oberon2.htm> (HTML transcription; source of the
  EBNF reproduced below)

Oberon-2 is chosen because it is the common ancestor of every dialect in the corpus:
Oberon-A, AmigaOberon 3.1 and STJ-Oberon are all Oberon-2 implementations with additions.
Oberon-07 is explicitly **not** the baseline — it removes constructs (`LOOP`, `EXIT`, `WITH`,
type-bound procedure syntax differences) that the corpus uses heavily.

## Scope of the base grammar

Per decision D1 in `docs/plan.md` the grammar is Oberon-2 **plus a lexical superset** that lets
dialect files parse without `ERROR` nodes, without ascribing meaning to them:

| Construct | Origin | Grammar treatment |
|---|---|---|
| Nested comments | Oberon-2 report §3.6 (normative) | external scanner, depth-counted |
| `(*$ … *)` pragmas | Oberon-A, AmigaOberon, STJ | distinct node kind, lexically a comment |
| `INLINE` assembly blocks | Oberon-A, AmigaOberon | opaque token, contents unparsed |
| `DEFINITION` modules | STJ-Oberon | module header variant |

## Normative EBNF (Appendix B of the report)

Reproduced verbatim, with the HTML transcription's spurious spaces around `*`, `~` and `^`
removed. This is the reference for milestone M1; grammar tasks receive fragments of it.

```ebnf
Module       = MODULE ident ";" [ImportList] DeclSeq
               [BEGIN StatementSeq] END ident ".".
ImportList   = IMPORT [ident ":="] ident {"," [ident ":="] ident} ";".
DeclSeq      = { CONST {ConstDecl ";" } | TYPE {TypeDecl ";"}
                 | VAR {VarDecl ";"}} {ProcDecl ";" | ForwardDecl ";"}.
ConstDecl    = IdentDef "=" ConstExpr.
TypeDecl     = IdentDef "=" Type.
VarDecl      = IdentList ":" Type.
ProcDecl     = PROCEDURE [Receiver] IdentDef [FormalPars] ";" DeclSeq
               [BEGIN StatementSeq] END ident.
ForwardDecl  = PROCEDURE "^" [Receiver] IdentDef [FormalPars].
FormalPars   = "(" [FPSection {";" FPSection}] ")" [":" Qualident].
FPSection    = [VAR] ident {"," ident} ":" Type.
Receiver     = "(" [VAR] ident ":" ident ")".
Type         = Qualident
             | ARRAY [ConstExpr {"," ConstExpr}] OF Type
             | RECORD ["(" Qualident ")"] FieldList {";" FieldList} END
             | POINTER TO Type
             | PROCEDURE [FormalPars].
FieldList    = [IdentList ":" Type].
StatementSeq = Statement {";" Statement}.
Statement    = [ Designator ":=" Expr
             | Designator ["(" [ExprList] ")"]
             | IF Expr THEN StatementSeq {ELSIF Expr THEN StatementSeq}
               [ELSE StatementSeq] END
             | CASE Expr OF Case {"|" Case} [ELSE StatementSeq] END
             | WHILE Expr DO StatementSeq END
             | REPEAT StatementSeq UNTIL Expr
             | FOR ident ":=" Expr TO Expr [BY ConstExpr] DO StatementSeq END
             | LOOP StatementSeq END
             | WITH Guard DO StatementSeq {"|" Guard DO StatementSeq}
               [ELSE StatementSeq] END
             | EXIT
             | RETURN [Expr]
             ].
Case         = [CaseLabels {"," CaseLabels} ":" StatementSeq].
CaseLabels   = ConstExpr [".." ConstExpr].
Guard        = Qualident ":" Qualident.
ConstExpr    = Expr.
Expr         = SimpleExpr [Relation SimpleExpr].
SimpleExpr   = ["+" | "-"] Term {AddOp Term}.
Term         = Factor {MulOp Factor}.
Factor       = Designator ["(" [ExprList] ")"] | number | character | string
             | NIL | Set | "(" Expr ")" | "~" Factor.
Set          = "{" [Element {"," Element}] "}".
Element      = Expr [".." Expr].
Relation     = "=" | "#" | "<" | "<=" | ">" | ">=" | IN | IS.
AddOp        = "+" | "-" | OR.
MulOp        = "*" | "/" | DIV | MOD | "&".
Designator   = Qualident {"." ident | "[" ExprList "]" | "^"
             | "(" Qualident ")"}.
ExprList     = Expr {"," Expr}.
IdentList    = IdentDef {"," IdentDef}.
Qualident    = [ident "."] ident.
IdentDef     = ident ["*" | "-"].
```

Note that `Statement` is optional as a whole (`[ … ]`), which is how the report permits empty
statements — `BEGIN ; END` and a trailing `;` before `END` are both legal. The corpus relies on
this; a grammar that requires a statement will produce `ERROR` nodes on real files.

## Lexical rules (report §3)

```ebnf
ident       = letter {letter | digit}.
number      = integer | real.
integer     = digit {digit} | digit {hexDigit} "H".
real        = digit {digit} "." {digit} [ScaleFactor].
ScaleFactor = ("E" | "D") ["+" | "-"] digit {digit}.
hexDigit    = digit | "A" | "B" | "C" | "D" | "E" | "F".
character   = digit {hexDigit} "X".
string      = '"' {char} '"' | "'" {char} "'".
```

Identifiers and keywords are case-sensitive; reserved words are all upper case. Both `"` and
`'` delimit strings, and the opening quote must match the closing quote.

**Comments** (§3.6, verbatim): *"Comments may be inserted between any two symbols in a program.
They are arbitrary character sequences opened by the bracket `(*` and closed by `*)`. Comments
may be nested. They do not affect the meaning of a program."*

Nesting is normative, not a dialect extension, and 48 corpus files use it. A regex token cannot
express it — hence the external scanner in M1.3.

### Reserved words

Derived from the productions above (the HTML transcription's table is garbled):

```
ARRAY BEGIN BY CASE CONST DIV DO ELSE ELSIF END EXIT FOR IF IMPORT IN IS LOOP
MOD MODULE NIL OF OR POINTER PROCEDURE RECORD REPEAT RETURN THEN TO TYPE UNTIL
VAR WHILE WITH
```

`SYSTEM` is not reserved — it is an ordinary module identifier (report Appendix C), and
`SYSTEM.ADR` etc. parse as plain qualified designators. 167 Oberon-A and 169 STJ files import
it; no grammar support is needed for that, only for the dialect-specific procedures those
modules call, which is a Phase 2 catalog concern.
