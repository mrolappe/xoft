# Language baseline

**Decision:** the xoft base grammar targets **Oberon-2** as defined by

> H. Mössenböck, N. Wirth: *The Programming Language Oberon-2*.
> Institut für Computersysteme, ETH Zürich, March 1995 revision.

Normative copies consulted:
- <https://ssw.jku.at/Research/Papers/Oberon2.pdf> (author's institute)
- <https://cseweb.ucsd.edu/~wgg/CSE131B/oberon2.htm> (HTML transcription; source of the
  EBNF reproduced below)

Oberon-2 is chosen as the baseline because it is the common ancestor of most of the corpus, but
**not uniformly all of it** — this was asserted without checking and is corrected here (M1.4,
round 8). Oberon-A and STJ-Oberon are confirmed Oberon-2 implementations by corpus usage, not
just by their own documentation: both use type-bound procedures (receivers), record type
extension, and `WITH` regularly (11/237 and 68/306 files respectively for receivers alone).
AmigaOberon 3.1 uses none of these almost at all (1/122 files for receivers, 2/122 for record
extension, 1/122 for `WITH`) despite being a much larger corpus fraction than that would predict.

**Confirmed against the primary source**, not just corpus inference:

> A+L AG / Fridtjof Siebert: *Amiga Oberon Compiler Handbuch*. © 1990.
> <https://archive.org/details/amiga-oberon>

The manual's own bibliography cites only `[nw:or]: Niklaus Wirth, Revised Oberon Report, ETH
Institut für Informatik` — the **original** Oberon report, never Mössenböck & Wirth's *The
Programming Language Oberon-2*. Its foreword describes "Oberon" (singular, Wirth's language,
already including type extension as a base-language feature per the original report) as the
implemented language, not Oberon-2's later additions (type-bound procedures/receivers, `WITH`
multi-guard, open arrays as pointer base type). "Amiga Oberon 2.0" appearing throughout the text
(e.g. "Anhang A: Amiga Oberon 2.0") is the **product's own version number**, not a language-spec
reference — ruled out explicitly, since it reads exactly like an Oberon-2 citation out of
context.

AmigaOberon's real extensions beyond the original report are its own Amiga/C-interop additions
— `STRUCT`, `{base,-N}` brace-annotated procedures, the `INLINE` pseudo-procedure, `*`/`:`
import-rename variants — not Oberon-2 ones (see the dialect table below and
`docs/progress/m1-grammar.md`'s M1.4 section). This matters because it changes what "in scope
for D1's lexical superset" even means for AmigaOberon-only constructs — see `NEXT.md`'s open
scoping question.

Oberon-07 is explicitly **not** the baseline — it removes constructs (`LOOP`, `EXIT`, `WITH`,
type-bound procedure syntax differences) that the corpus uses heavily.

## Scope of the base grammar

Per decision D1 in `docs/plan.md` the grammar is Oberon-2 **plus a lexical superset** that lets
dialect files parse without `ERROR` nodes, without ascribing meaning to them:

| Construct | Origin | Grammar treatment |
|---|---|---|
| Nested comments | Oberon-2 report §3.6 (normative) | external scanner, depth-counted |
| `(*$ … *)` pragmas | Oberon-A, AmigaOberon, STJ | distinct node kind, lexically a comment |
| `SYSTEM.INLINE(...)` | STJ only (confirmed by corpus grep — not Oberon-A/AmigaOberon as previously guessed) | **not opaque, not a block** — an ordinary procedure call. Needed no grammar rule; was blocked by an unrelated hex-integer-literal token bug, fixed in M1.4. See `docs/progress/m1-grammar.md`. |
| `DEFINITION` modules | STJ-Oberon | module header variant |
| `IMPORT ident * := M` (re-export marker), `IMPORT ident: M` (colon rename) | AmigaOberon only, likely pre-Oberon-2 heritage (see above) | widened `import` rule, M1.4 |
| `<* ... *>` bracket pragmas | AmigaOberon, STJ (212/792 files) | **not yet in grammar** — different delimiter from `(*$…*)`. Triaged, not scoped. See `NEXT.md`. |
| `STRUCT` record variant | AmigaOberon only (43/792 files) | **not yet in grammar** — C-interop type, not `RECORD`. Likely a base-language difference, not a "Oberon-2 + lexical extras" addition — see the AmigaOberon-heritage note above. Triaged, not scoped. See `NEXT.md`. |
| `PROCEDURE ... *{base,-N}(...)`, `param{N}` brace annotations | AmigaOberon only (42/792 files) | **not yet in grammar** — library-vector-offset metadata. Triaged, not scoped. See `NEXT.md`. |
| `ASSEMBLER` blocks | STJ only (32/792 files) | **not yet in grammar**, real syntax unconfirmed. Triaged, not scoped. See `NEXT.md`. |

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
