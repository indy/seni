#ifndef SENI_LANG_LEXER_H
#define SENI_LANG_LEXER_H

#include "seni_types.h"

typedef enum {
  TOK_UNKNOWN = 0,
  TOK_LIST_START,
  TOK_LIST_END,
  TOK_VECTOR_START,
  TOK_VECTOR_END,
  TOK_ALTERABLE_START,
  TOK_ALTERABLE_END,
  TOK_INT,
  TOK_FLOAT,
  TOK_NAME,
  TOK_STRING,
  TOK_QUOTE_ABBREVIATION,
  TOK_LABEL,
  TOK_COMMENT,
  TOK_WHITESPACE
} seni_token_type;

typedef struct seni_token {
  seni_token_type type;
  
  i32 i32_value;
  f32 f32_value;
  char* str_value;
  char chr_value;               /* (, ), [, ] */

  char* remaining;

  struct seni_token *prev;
  struct seni_token *next;
  
} seni_token;

seni_token *tokenise(char *s);
void free_tokens(seni_token *tokens);

#endif
