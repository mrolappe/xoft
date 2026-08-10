#include <tree_sitter/parser.h>
#include <stdbool.h>

enum TokenType {
  COMMENT,
  PRAGMA,
  BRACKET_PRAGMA,
  ASSEMBLER_BODY,
};

void *tree_sitter_oberon2_external_scanner_create(void) { return NULL; }
void tree_sitter_oberon2_external_scanner_destroy(void *payload) {}
unsigned tree_sitter_oberon2_external_scanner_serialize(void *payload, char *buffer) { return 0; }
void tree_sitter_oberon2_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {}

// Oberon-2 report §3.6: comments are arbitrary character sequences opened by
// "(*" and closed by "*)", and may be nested. A "(*$" opener is, per D1, the
// same bracket lexically, just reported as a distinct node kind (pragma).
static bool is_space(int32_t c) {
  return c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\v' || c == '\f';
}

static bool is_ident_char(int32_t c) {
  return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9');
}

bool tree_sitter_oberon2_external_scanner_scan(void *payload, TSLexer *lexer, const bool *valid_symbols) {
  if (!valid_symbols[COMMENT] && !valid_symbols[PRAGMA] && !valid_symbols[BRACKET_PRAGMA] &&
      !valid_symbols[ASSEMBLER_BODY]) return false;

  // The internal lexer only calls back into the external scanner once, before
  // it tries to skip whitespace itself — so this scanner must skip its own
  // leading whitespace (as `skip`, i.e. not part of the token) or it never
  // sees a comment that isn't the very next byte.
  while (is_space(lexer->lookahead)) {
    lexer->advance(lexer, true);
  }

  if (valid_symbols[ASSEMBLER_BODY]) {
    // STJ-Oberon `ASSEMBLER ... END` statement: the body is raw M68K text
    // (opcodes with size suffixes, "(A0,D0.L)" addressing, "#" immediates,
    // "D0-A7" register ranges) that doesn't tokenize as Oberon, so it's
    // scanned as one opaque span, the same technique as comment/pragma
    // above. Unlike comments, this body doesn't nest, so the only thing to
    // detect is the closing "END" — as a whole word (not a substring of an
    // opcode/operand), confirmed against the corpus.
    if (lexer->eof(lexer)) return false;
    bool prev_is_ident = is_ident_char(lexer->lookahead);
    lexer->advance(lexer, false);

    for (;;) {
      if (lexer->eof(lexer)) return false;

      if (!prev_is_ident && lexer->lookahead == 'E') {
        lexer->mark_end(lexer);
        lexer->advance(lexer, false);
        if (!lexer->eof(lexer) && lexer->lookahead == 'N') {
          lexer->advance(lexer, false);
          if (!lexer->eof(lexer) && lexer->lookahead == 'D') {
            lexer->advance(lexer, false);
            if (lexer->eof(lexer) || !is_ident_char(lexer->lookahead)) {
              lexer->result_symbol = ASSEMBLER_BODY;
              return true;
            }
          }
        }
        prev_is_ident = true;
        continue;
      }

      prev_is_ident = is_ident_char(lexer->lookahead);
      lexer->advance(lexer, false);
    }
  }

  if (lexer->lookahead == '<') {
    // AmigaOberon/Oberon-A dialect extension (not in normative EBNF): a
    // "<* ... *>" bracket pragma, confirmed via corpus grep to hold either
    // bare compiler-switch flags ("<* STANDARD- *>") or "$"-prefixed
    // sub-pragmas ("<*$LongVars-*>") — same lexical family as "(*$...*)"
    // but a different delimiter, and never seen nested.
    lexer->advance(lexer, false);
    if (lexer->lookahead != '*') return false;
    lexer->advance(lexer, false);

    for (;;) {
      if (lexer->eof(lexer)) return false;

      if (lexer->lookahead == '*') {
        lexer->advance(lexer, false);
        if (lexer->lookahead == '>') {
          lexer->advance(lexer, false);
          break;
        }
        continue;
      }

      lexer->advance(lexer, false);
    }

    lexer->result_symbol = BRACKET_PRAGMA;
    lexer->mark_end(lexer);
    return true;
  }

  if (lexer->lookahead != '(') return false;
  lexer->advance(lexer, false);
  if (lexer->lookahead != '*') return false;
  lexer->advance(lexer, false);

  bool is_pragma = lexer->lookahead == '$';

  unsigned depth = 1;
  for (;;) {
    if (lexer->eof(lexer)) return false;

    if (lexer->lookahead == '(') {
      lexer->advance(lexer, false);
      if (lexer->lookahead == '*') {
        lexer->advance(lexer, false);
        depth++;
      }
      continue;
    }

    if (lexer->lookahead == '*') {
      lexer->advance(lexer, false);
      if (lexer->lookahead == ')') {
        lexer->advance(lexer, false);
        depth--;
        if (depth == 0) break;
      }
      continue;
    }

    lexer->advance(lexer, false);
  }

  lexer->result_symbol = is_pragma ? PRAGMA : COMMENT;
  lexer->mark_end(lexer);
  return true;
}
