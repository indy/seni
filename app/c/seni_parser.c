#include "seni_parser.h"
#include "seni_strtof.h"

#include "utlist.h"

#include <stdlib.h>
#include <string.h>

seni_node *consume_item();

bool is_minus(char c)
{
  return c == '-';
}

bool is_period(char c)
{
  return c == '.';
}

bool is_whitespace(char c)
{
  return c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == ',';
}

bool is_digit(char c)
{
  return c >= '0' && c <= '9';
}

bool is_alpha(char c)
{
  return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z');
}

bool is_symbol(char c)
{
  return c == '+' || c == '-' || c == '*' || c == '/' || c == '=' ||
    c == '!' || c == '@' || c == '#' || c == '$' || c == '%' ||
    c == '^' || c == '&' || c == '<' || c == '>' || c == '?';
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

bool is_label(char *s, size_t word_len)
{
  return s[word_len] == ':';
}

bool has_period(char *s)
{
  size_t i = 0;
  char c = s[i];

  while (c != 0) {
    if (is_period(c)) {
      return true;
    }
    if (is_whitespace(c)) {
      return false;
    }
    i++;
    c = s[i];
  }

  return false;
}

char *find_next(char *s, char target)
{
  while (*s != 0){
    if (*s == target) {
      return s;
    }
    s++;
  }
  return NULL;
}

void string_copy_len(char **dst, char *src, size_t len)
{
  char *c = (char *)calloc(len + 1, sizeof(char));
  strncpy(c, src, len);
  c[len] = '\0';

  *dst = c;
}

/* returns 0 if not found */
i32 lookup_name(char **words, i32 word_count, i32 offset, char *string, size_t len)
{
  i32 i = 0;
  for (i = 0; i < word_count; i++) {
    char *name = words[i];
    bool found = true;
    /* can't use string_compare since 'string' could be a substring */
    size_t j = 0;
    for (j = 0; j < len; j++) {
      if (name[j] == '\0' || (name[j] != string[j])) {
        found = false;
        break;
      }
    }
    /* searched all of 'string' and the early exit wasn't triggered */
    if (name[j] == '\0' && found) {
      return i + offset;
    }
  }

  return -1;  
}

i32 wlut_lookup_(seni_word_lut *wlut, char *string, size_t len)
{
  i32 native = lookup_name(wlut->native, wlut->native_count, NATIVE_START, string, len);
  if (native != -1) {
    return native;
  }

  i32 keyword = lookup_name(wlut->keyword, wlut->keyword_count, KEYWORD_START, string, len);
  if (keyword != -1) {
    return keyword;
  }

  i32 word = lookup_name(wlut->word, wlut->word_count, WORD_START, string, len);
  if (word != -1) {
    return word;
  }

  return -1;
}

i32 wlut_lookup_or_add(seni_word_lut *wlut, char *string, size_t len)
{
  i32 iname = wlut_lookup_(wlut, string, len);
  if (iname != -1) {
    return iname;
  }

  /* string is not in the table and there's no room for another entry */
  if (wlut->word_count >= MAX_WORD_LOOKUPS) {
    return -1;
  }

  // the string is not in the lookup table, so add it
  string_copy_len(&(wlut->word[wlut->word_count]), string, len);
  wlut->word_count++;

  return wlut->word_count - 1;
}

seni_node *build_text_lookup_node_from_string(seni_word_lut *wlut, seni_node_type type, char *string)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  size_t len = strlen(string);

  i32 k = wlut_lookup_or_add(wlut, string, len);
  if (k == -1) {
    return NULL;
  }

  node->type = type;
  node->value.i = k;
  
  return node;
}

seni_node *build_text_lookup_node_of_length(seni_word_lut *wlut, char **src, seni_node_type type, size_t len)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));

  i32 k = wlut_lookup_or_add(wlut, *src, len);
  if (k == -1) {
    return NULL;
  }

  node->type = type;
  node->value.i = k;

  node->src = *src;
  node->src_len = (i32)len;
  
  *src += len;

  return node;
}

// allocate memory for comments and whitespace rather than using the lookup table
//
seni_node *build_text_node_of_length(char **src, seni_node_type type, size_t len)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = type;

  char *str = (char *)calloc(len + 1, sizeof(char));
  strncpy(str, *src, len);
  str[len] = '\0';

  node->src = *src;
  node->src_len = (i32)len;

  *src += len;
  
  node->value.s = str;
  
  return node;
}

seni_node *consume_list(seni_word_lut *wlut, char **src)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_LIST;

  (*src)++; // (

  while (1) {
    if (is_list_end(**src)) {
      (*src)++; // )
      return node;
    }

    seni_node *child = consume_item(wlut, src);
    if (child == NULL) {
      SENI_ERROR("unable to consume element of list");
      return NULL;
    }

    DL_APPEND(node->value.first_child, child);
  }
}

seni_node *consume_vector(seni_word_lut *wlut, char **src)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_VECTOR;

  (*src)++; // [
  
  while (1) {
    if (is_vector_end(**src)) {
      (*src)++; // ]
      return node;
    }

    seni_node *child = consume_item(wlut, src);
    if (child == NULL) {
      SENI_ERROR("unable to consume element of vector");
      return NULL;
    }

    DL_APPEND(node->value.first_child, child);
  }
}

seni_node *consume_alterable(seni_word_lut *wlut, char **src)
{
  seni_node *node;
  seni_node *parameter_prefix = NULL;
  seni_node *c;
  
  (*src)++; // {
  
  while (1) {
    c = consume_item(wlut, src);
    if (c == NULL) {
      SENI_ERROR("unable to consume element of alterable");
      return NULL;
    }

    if (c->type == NODE_COMMENT || c->type == NODE_WHITESPACE) {
      DL_APPEND(parameter_prefix, c);
    } else {
      node = c;
      node->alterable = true;
      node->parameter_prefix = parameter_prefix;
      break;
    }
  }

  if (node->type != NODE_INT
      && node->type != NODE_FLOAT
      && node->type != NODE_NAME
      && node->type != NODE_LIST
      && node->type != NODE_VECTOR)
    {
      SENI_ERROR("non-mutable node within curly brackets: %s", node_type_name(node));
      return NULL;
    }
  
  while (1) {
    if (is_alterable_end(**src)) {
      (*src)++; // }
      return node;
    }

    seni_node *child = consume_item(wlut, src);
    if (child == NULL) {
      SENI_ERROR("unable to consume element of bracket");
      return NULL;
    }

    DL_APPEND(node->parameter_ast, child);
  }
}

seni_node *consume_quoted_form(seni_word_lut *wlut, char **src)
{
  (*src)++; // '
  
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_LIST;

  seni_node *quote_name = build_text_lookup_node_from_string(wlut, NODE_NAME, "quote");
  DL_APPEND(node->value.first_child, quote_name);

  char *wst = " ";
  seni_node *ws = build_text_node_of_length(&wst, NODE_WHITESPACE, 1);
  DL_APPEND(node->value.first_child, ws);

  seni_node *child = consume_item(wlut, src);
  DL_APPEND(node->value.first_child, child);

  return node;
}

seni_node *consume_float(char **src)
{
  char *end_ptr;
  
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_FLOAT;
  node->value.f = (f32)seni_strtof(*src, &end_ptr);

  node->src = *src;
  node->src_len = (i32)(end_ptr - *src);

  *src = end_ptr;
  
  return node;
}

seni_node *consume_name(seni_word_lut *wlut, char **src)
{
  size_t i = 0;
  char *rem = *src;

  while(rem[i]) {
    char c = rem[i];
    if (!is_alpha(c) && !is_digit(c) && !is_symbol(c)) {
      break;
    }
    i++;
  }

  seni_node *node = build_text_lookup_node_of_length(wlut, src, NODE_NAME, i);

  return node;
}

seni_node *consume_string(seni_word_lut *wlut, char **src)
{
  (*src)++; // skip the first \"

  char *next_quote = find_next(*src, '\"');
  if (next_quote == NULL) {
    return NULL;
  }

  size_t string_len = next_quote - *src;

  seni_node *node = build_text_lookup_node_of_length(wlut, src, NODE_STRING, string_len);

  (*src)++; // skip the second \"
  
  return node;
}

seni_node *consume_label(seni_word_lut *wlut, char **src)
{
  size_t i = 0;
  char *rem = *src;

  while(rem[i]) {
    char c = rem[i];
    if (!is_alpha(c) && !is_digit(c) && !is_symbol(c)) {
      break;
    }
    i++;
  }

  // read the label name - the ':' character
  seni_node *node = build_text_lookup_node_of_length(wlut, src, NODE_LABEL, i);

  if (**src != ':') {
    return NULL;
  }

  (*src)++;        /* the remaining should skip past the ':' */

  return node;
}

seni_node *consume_comment(char **src)
{
  size_t i = 0;
  char *rem = *src;
  
  while (rem[i]) {
    char c = rem[i];
    if (is_newline(c)) {
      break;
    }
    i++;
  }

  seni_node *node = build_text_node_of_length(src, NODE_COMMENT, i);

  if (is_newline(*rem)) {
    (*src)++;        /* skip past the newline */
  }
    
  return node;
}

seni_node *consume_whitespace(char **src)
{
  size_t i = 0;
  char *rem = *src;
  char c = rem[i];
  
  while(c) {
    if (!is_whitespace(c)) {
      break;
    }
    i++;
    c = rem[i];
  }

  seni_node *node = build_text_node_of_length(src, NODE_WHITESPACE, i);

  return node;
}

seni_node *consume_item(seni_word_lut *wlut, char **src)
{
  char c = **src;

  if (is_whitespace(c)) {
    return consume_whitespace(src);
  }

  if (is_quote_abbreviation(c)) {
    return consume_quoted_form(wlut, src);
  }

  if (is_list_start(c)) {
    return consume_list(wlut, src);
  }

  if (is_list_end(c)) {
    return NULL;                /* 'mismatched closing parens' */
  }

  if (is_vector_start(c)) {
    return consume_vector(wlut, src);
  }

  if (is_vector_end(c)) {
    return NULL;                /* 'mismatched closing square brackets' */
  }

  if (is_alterable_start(c)) {
    return consume_alterable(wlut, src);
  }

  if (is_alterable_end(c)) {
    return NULL;                /* 'mismatched closing alterable brackets' */
  }

  if (is_quoted_string(c)) {
    return consume_string(wlut, src);
  }

  if (is_alpha(c) || is_minus(c) || is_symbol(c)) {
    // doesn't begin with -[0..9]
    if (!(is_minus(c) && *(*src + 1) != 0 && is_digit(*(*src + 1)))) {

      char *s = *src;
      size_t word_len = 0;

      while(*s != 0) {
        if (!is_alpha(*s) && !is_digit(*s) && !is_symbol(*s)) {
          break;
        }
        word_len++;
        s++;
      }
      
      if (is_label(*src, word_len)) {
        return consume_label(wlut, src);
      }

      return consume_name(wlut, src);
    }
  }
  
  if (is_digit(c) || is_minus(c) || is_period(c)) {
    return consume_float(src);
  }

  if (is_comment(c)) {
    return consume_comment(src);
  }
  return NULL;
}

void parser_free_nodes(seni_node *nodes)
{
  seni_node *node = nodes;
  seni_node *next;

  while(node != NULL) {
    if (node->type == NODE_LIST && node->value.first_child) {
      parser_free_nodes(node->value.first_child);
    }
    if (node->parameter_ast) {
      parser_free_nodes(node->parameter_ast);
    }
    if (node->parameter_prefix) {
      parser_free_nodes(node->parameter_prefix);
    }
    
    next = node->next;

    if (node->type == NODE_COMMENT || node->type == NODE_WHITESPACE) {
      // freeing a pointer in a union, so make sure that the value in
      // the union only comes from the 's' component and not 'i' or 'f'
      //
      if (node->value.s != NULL) {
        free(node->value.s);
      }
    }

    // SENI_LOG("freeing node: %s %u", node_type_name(node), (u32)node);
    free(node);
    
    node = next;
  }
}

seni_node *parser_parse(seni_word_lut *wlut, char *s)
{
  if (s == NULL) {
    return NULL;
  }

  char **src = &s;

  seni_node *nodes = NULL;
  seni_node *node;

  while(**src) {
    node = consume_item(wlut, src);
    if (node == NULL) {
      // clean up
      parser_free_nodes(nodes);
      return NULL;
    }

    DL_APPEND(nodes, node);
  }

  // NOTE: not strictly a tree as the ast root could have siblings
  return nodes;
}
