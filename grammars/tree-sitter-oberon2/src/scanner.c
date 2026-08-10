#include <tree_sitter/parser.h>
#include <stdbool.h>

enum TokenType {
  COMMENT,
  PRAGMA,
  BRACKET_PRAGMA,
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

bool tree_sitter_oberon2_external_scanner_scan(void *payload, TSLexer *lexer, const bool *valid_symbols) {
  if (!valid_symbols[COMMENT] && !valid_symbols[PRAGMA] && !valid_symbols[BRACKET_PRAGMA]) return false;

  // The internal lexer only calls back into the external scanner once, before
  // it tries to skip whitespace itself — so this scanner must skip its own
  // leading whitespace (as `skip`, i.e. not part of the token) or it never
  // sees a comment that isn't the very next byte.
  while (is_space(lexer->lookahead)) {
    lexer->advance(lexer, true);
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
