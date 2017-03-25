#include <stdlib.h>
#include <inttypes.h>

#include "seni_lang_lexer.h"
#include "string.h"
#include "seni_containers.h"

char* chars_whitespace = " \t\n,";
char* chars_digit = "0123456789";
char* chars_alpha = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ+-*/<>=";
char* chars_symbol = "-!@#$%^&*<>?";
size_t chars_whitespace_len;
size_t chars_digit_len;
size_t chars_alpha_len;
size_t chars_symbol_len;

char MINUS = '-';
char PERIOD = '.';

bool contains(char c, char *set, size_t len)
{
  char *p = set;

  for( size_t i = 0; i < len; i++) {
    if (*p == c) {
      return true;
    }
    p++;
  }
  
  return false;
}

bool is_whitespace(char c)
{
  return contains(c, chars_whitespace, chars_whitespace_len);
}

bool is_digit(char c)
{
  return contains(c, chars_digit, chars_digit_len);
}

bool is_alpha(char c)
{
  return contains(c, chars_alpha, chars_alpha_len);
}

bool is_symbol(char c)
{
  return contains(c, chars_symbol, chars_symbol_len);
}

bool is_list_start(char c)
{
  return c == '(';
}

bool is_list_end(char c)
{
  return c == ')';
}

bool is_vector_start(char c)
{
  return c == '[';
}

bool is_vector_end(char c)
{
  return c == ']';
}

bool is_alterable_start(char c)
{
  return c == '{';
}

bool is_alterable_end(char c)
{
  return c == '}';
}

bool is_quoted_string(char c)
{
  return c == '"';
}

bool is_quote_abbreviation(char c)
{
  return c == '\'';
}

bool is_comment(char c)
{
  return c == ';';
}

bool is_newline(char c)
{
  return c == '\n';
}

bool is_label(char *s)
{
  size_t len = strlen(s);
  size_t i;
  char c;

  for (i = 0; i < len; i++) {
    c = s[i];
    if (!is_alpha(c) && !is_digit(c) && !is_symbol(c)) {
      break;
    }
  }

  return i < len && s[i] == ':';
}

bool has_period(char *s)
{
  size_t len = strlen(s);
  size_t i;
  char c;

  for (i = 0; i < len; i++) {
    c = s[i];
    if (c == PERIOD) {
      return true;
    }
    if (is_whitespace(c)) {
      return false;
    }
  }

  return false;
}

seni_token *consume_int(char *s)
{
  char *end_ptr;
  seni_token *ptoken = (seni_token *)calloc(1, sizeof(seni_token));

  ptoken->type = TOK_INT;
  ptoken->i32_value = (i32)strtoimax(s, &end_ptr, 10);
  ptoken->remaining = end_ptr;

  return ptoken;
}


seni_token *consume_float(char *s)
{
  char *end_ptr;
  seni_token *ptoken = (seni_token *)calloc(1, sizeof(seni_token));

  ptoken->type = TOK_FLOAT;
  ptoken->f32_value = (f32)strtof(s, &end_ptr);
  ptoken->remaining = end_ptr;

  return ptoken;
}

seni_token *build_single_char_token(seni_token_type type, char *s)
{
  seni_token *ptoken = (seni_token *)calloc(1, sizeof(seni_token));

  ptoken->type = type;
  ptoken->chr_value = *s;
  ptoken->remaining = s + 1;
  
  return ptoken;
}

seni_token *build_string_token(seni_token_type type, char *s, size_t len)
{
  seni_token *ptoken = (seni_token *)calloc(1, sizeof(seni_token));

  char *str = (char *)malloc(sizeof(char) * (len + 1));
  strncpy(str, s, len);
  str[len] = '\0';
  
  ptoken->type = type;
  ptoken->str_value = str;
  ptoken->remaining = s + len;
  
  return ptoken;
}

seni_token *consume_unknown(char *s)
{
  return build_single_char_token(TOK_UNKNOWN, s);
}

seni_token *consume_list_start(char *s)
{
  return build_single_char_token(TOK_LIST_START, s);
}

seni_token *consume_list_end(char *s)
{
  return build_single_char_token(TOK_LIST_END, s);
}

seni_token *consume_vector_start(char *s)
{
  return build_single_char_token(TOK_VECTOR_START, s);
}

seni_token *consume_vector_end(char *s)
{
  return build_single_char_token(TOK_VECTOR_END, s);
}

seni_token *consume_alterable_start(char *s)
{
  return build_single_char_token(TOK_ALTERABLE_START, s);
}

seni_token *consume_alterable_end(char *s)
{
  return build_single_char_token(TOK_ALTERABLE_END, s);
}

seni_token *consume_quote_abbreviation(char *s)
{
  return build_single_char_token(TOK_QUOTE_ABBREVIATION, s);  
}

char *find_next(char *s, char target)
{
  size_t len = strlen(s);

  for( size_t i = 0; i < len; i++){
    if (*s == target) {
      return s;
    }
    s++;
  }
  return NULL;
}

seni_token *consume_string(char *s)
{
  char *c = s + 1; // skip the first \"
  char *next_quote = find_next(c, '\"');
  if (next_quote == NULL) {
    return NULL;
  }

  size_t string_len = next_quote - c;

  seni_token *token = build_string_token(TOK_STRING, c, string_len);
  token->remaining = next_quote + 1;

  return token;
}

seni_token *consume_name(char *s)
{
  size_t len = strlen(s);
  size_t i;

  for (i = 0; i < len; i++) {
    char c = s[i];
    if (!is_alpha(c) && !is_digit(c) && !is_symbol(c)) {
      break;
    }
  }

  seni_token *token = build_string_token(TOK_NAME, s, i);
  return token;
}

seni_token *consume_whitespace(char *s)
{
  size_t len = strlen(s);
  size_t i;

  for (i = 0; i < len; i++) {
    char c = s[i];
    if (!is_whitespace(c)) {
      break;
    }
  }

  seni_token *token = build_string_token(TOK_WHITESPACE, s, i);
  return token;
}

seni_token *consume_comment(char *s)
{
  size_t len = strlen(s);
  size_t i;

  for (i = 0; i < len; i++) {
    char c = s[i];
    if (is_newline(c)) {
      break;
    }
  }

  seni_token *token = build_string_token(TOK_COMMENT, s, i);
  if (is_newline(*(token->remaining))) {
    token->remaining += 1;        /* skip past the newline */
  }
    
  return token;
}

seni_token *consume_label(char *s)
{
  size_t len = strlen(s);
  size_t i;

  for (i = 0; i < len; i++) {
    char c = s[i];
    if (!is_alpha(c) && !is_digit(c) && !is_symbol(c)) {
      break;
    }
  }

  // read the label name - the ':' character
  seni_token *token = build_string_token(TOK_LABEL, s, i);
  token->remaining += 1;        /* the remaining should skip past the ':' */
  
  return token;
}

seni_token_type next_token_type(char *s)
{
  char c = *s;

  if (is_whitespace(c)) {
    return TOK_WHITESPACE;
  }

  if (is_quote_abbreviation(c)) {
    return TOK_QUOTE_ABBREVIATION;
  }

  if (is_list_start(c)) {
    return TOK_LIST_START;
  }

  if (is_list_end(c)) {
    return TOK_LIST_END;
  }

  if (is_vector_start(c)) {
    return TOK_VECTOR_START;
  }

  if (is_vector_end(c)) {
    return TOK_VECTOR_END;
  }

  if (is_alterable_start(c)) {
    return TOK_ALTERABLE_START;
  }

  if (is_alterable_end(c)) {
    return TOK_ALTERABLE_END;
  }

  if (is_quoted_string(c)) {
    return TOK_STRING;
  }

  if (is_alpha(c)) {
    if (!(c == MINUS && strlen(s) > 1 && is_digit(s[1]))) {
      return is_label(s) ? TOK_LABEL : TOK_NAME;
    }
  }

  if (is_digit(c) || c == MINUS || c == PERIOD) {
    return has_period(s) ? TOK_FLOAT : TOK_INT;
  }

  if (is_comment(c)) {
    return TOK_COMMENT;
  }

  return TOK_UNKNOWN;  
}

seni_token *tokenise(char *s)
{
  if (s == NULL) {
    return NULL;
  }

  chars_whitespace_len = strlen(chars_whitespace);
  chars_digit_len = strlen(chars_digit);
  chars_alpha_len = strlen(chars_alpha);
  chars_symbol_len = strlen(chars_symbol);
  
  seni_token *tokens = NULL;
  seni_token *p;
  size_t len = strlen(s);

  while (len > 0) {
    switch(next_token_type(s)) {
      case TOK_WHITESPACE :
        p = consume_whitespace(s);
        break;
      case TOK_LIST_START :
        p = consume_list_start(s);
        break;
      case TOK_LIST_END :
        p = consume_list_end(s);
        break;
      case TOK_VECTOR_START :
        p = consume_vector_start(s);
        break;
      case TOK_VECTOR_END :
        p = consume_vector_end(s);
        break;
      case TOK_ALTERABLE_START :
        p = consume_alterable_start(s);
        break;
      case TOK_ALTERABLE_END :
        p = consume_alterable_end(s);
        break;
      case TOK_STRING :
        p = consume_string(s);
        break;
      case TOK_NAME :
        p = consume_name(s);
        break;
      case TOK_LABEL :
        p = consume_label(s);
        break;
      case TOK_INT :
        p = consume_int(s);
        break;
      case TOK_FLOAT :
        p = consume_float(s);
        break;
      case TOK_QUOTE_ABBREVIATION :
        p = consume_quote_abbreviation(s);
        break;
      case TOK_COMMENT :
        p = consume_comment(s);
        break;
      default:
        // read the unknown token and return it
        //const tok = consumeUnknown(s)[0];
        //return {error: `unknown token: ${tok.value}`,
        //tokens: [tok]};
        return NULL;
    };

    if(p == NULL) {
      /* TODO: ERROR */
      return NULL;
    }
      
    DL_APPEND(tokens, p);
    s = p->remaining;
    len = strlen(s);
  }
    
  return tokens;
}

void free_tokens(seni_token *tokens)
{
  seni_token *token = tokens;
  seni_token *next;

  while(token != NULL) {
    next = token->next;
    if (token->str_value != NULL) {
      free(token->str_value);
    }
    free(token);
    token = next;
  }
}
