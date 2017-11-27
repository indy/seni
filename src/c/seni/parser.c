#include "parser.h"

#include "lang.h"
#include "strtof.h"

#include "../lib/utlist.h"

#include <stdlib.h>
#include <string.h>

#include "multistring.h"
#include "pool_macro.h"

void node_cleanup(seni_node *node) {
  node->alterable         = 0;
  node->src               = NULL;
  node->src_len           = 0;
  node->value.first_child = NULL; // empty the value union
  node->parameter_ast     = NULL;
  node->parameter_prefix  = NULL;
}

SENI_POOL(seni_node, node)

struct seni_node_pool *g_node_pool;

void parser_subsystem_startup() { g_node_pool = node_pool_allocate(1, 1000, 20); }

void parser_subsystem_shutdown() { node_pool_free(g_node_pool); }

seni_node *node_get_from_pool() {
  seni_node *node = node_pool_get(g_node_pool);

  if (node == NULL) {
    SENI_ERROR("OH NO NODE IS NULL");
    return NULL;
  }

  return node;
}

void node_return_to_pool(seni_node *node) {
  node_cleanup(node);
  node_pool_return(g_node_pool, node);
}

seni_node *eat_item();

bool is_minus(char c) { return c == '-'; }

bool is_period(char c) { return c == '.'; }

bool is_whitespace(char c) { return c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == ','; }

bool is_digit(char c) { return c >= '0' && c <= '9'; }

bool is_alpha(char c) { return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z'); }

bool is_symbol(char c) {
  return c == '+' || c == '-' || c == '*' || c == '/' || c == '=' || c == '!' || c == '@' ||
         c == '#' || c == '$' || c == '%' || c == '^' || c == '&' || c == '<' || c == '>' ||
         c == '?';
}

bool is_list_start(char c) { return c == '('; }

bool is_list_end(char c) { return c == ')'; }

bool is_vector_start(char c) { return c == '['; }

bool is_vector_end(char c) { return c == ']'; }

bool is_alterable_start(char c) { return c == '{'; }

bool is_alterable_end(char c) { return c == '}'; }

bool is_quoted_string(char c) { return c == '"'; }

bool is_quote_abbreviation(char c) { return c == '\''; }

bool is_comment(char c) { return c == ';'; }

bool is_newline(char c) { return c == '\n'; }

bool is_label(char *s, size_t word_len) { return s[word_len] == ':'; }

bool has_period(char *s) {
  size_t i = 0;
  char   c = s[i];

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

char *find_next(char *s, char target) {
  while (*s != 0) {
    if (*s == target) {
      return s;
    }
    s++;
  }
  return NULL;
}

/* returns 0 if not found */
i32 lookup_name(seni_string_ref *string_refs,
                i32              word_count,
                i32              offset,
                char *           string,
                size_t           len) {
  i32              i          = 0;
  seni_string_ref *string_ref = string_refs;

  for (i = 0; i < word_count; i++) {
    char *name  = string_ref->c;
    bool  found = true;

    // can also compare len with string_ref->len for an early test exit

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

    string_ref++;
  }

  return -1;
}

i32 word_lut_lookup_(seni_word_lut *word_lut, char *string, size_t len) {
  i32 native = lookup_name(word_lut->native_ref, word_lut->native_count, NATIVE_START, string, len);
  if (native != -1) {
    return native;
  }

  i32 keyword =
      lookup_name(word_lut->keyword_ref, word_lut->keyword_count, KEYWORD_START, string, len);
  if (keyword != -1) {
    return keyword;
  }

  i32 word = lookup_name(word_lut->word_ref, word_lut->word_count, WORD_START, string, len);
  if (word != -1) {
    return word;
  }

  return -1;
}

i32 word_lut_lookup_or_add(seni_word_lut *word_lut, char *string, size_t len) {
  i32 iname = word_lut_lookup_(word_lut, string, len);
  if (iname != -1) {
    return iname;
  }

  // the string is not in the lookup table, so add it
  bool res = wlut_add_word(word_lut, string, len);
  if (res == false) {
    SENI_ERROR("word_lut_lookup_or_add failed");
    return 0;
  }

  return word_lut->word_count - 1;
}

seni_node *
build_text_lookup_node_from_string(seni_word_lut *word_lut, seni_node_type type, char *string) {
  seni_node *node = node_get_from_pool();
  size_t     len  = strlen(string);

  i32 k = word_lut_lookup_or_add(word_lut, string, len);
  if (k == -1) {
    return NULL;
  }

  node->type    = type;
  node->value.i = k;

  return node;
}

seni_node *build_text_lookup_node_of_length(seni_word_lut *word_lut,
                                            char **        src,
                                            seni_node_type type,
                                            size_t         len) {
  seni_node *node = node_get_from_pool();

  i32 k = word_lut_lookup_or_add(word_lut, *src, len);
  if (k == -1) {
    return NULL;
  }

  node->type    = type;
  node->value.i = k;

  node->src     = *src;
  node->src_len = (i32)len;

  *src += len;

  return node;
}

// allocate memory for comments and whitespace rather than using the lookup
// table
//
seni_node *build_text_node_of_length(char **src, seni_node_type type, size_t len) {
  seni_node *node = node_get_from_pool();
  node->type      = type;

  node->src     = *src;
  node->src_len = (i32)len;

  *src += len;

  return node;
}

seni_node *eat_list(seni_word_lut *word_lut, char **src) {
  seni_node *node         = node_get_from_pool();
  node->type              = NODE_LIST;
  node->value.first_child = NULL;

  (*src)++; // (

  while (1) {
    if (is_list_end(**src)) {
      (*src)++; // )
      return node;
    }

    seni_node *child = eat_item(word_lut, src);
    if (child == NULL) {
      SENI_ERROR("unable to eat element of list");
      return NULL;
    }

    DL_APPEND(node->value.first_child, child);
  }
}

seni_node *eat_vector(seni_word_lut *word_lut, char **src) {
  seni_node *node = node_get_from_pool();
  node->type      = NODE_VECTOR;

  (*src)++; // [

  while (1) {
    if (is_vector_end(**src)) {
      (*src)++; // ]
      return node;
    }

    seni_node *child = eat_item(word_lut, src);
    if (child == NULL) {
      SENI_ERROR("unable to eat element of vector");
      return NULL;
    }

    DL_APPEND(node->value.first_child, child);
  }
}

seni_node *eat_alterable(seni_word_lut *word_lut, char **src) {
  seni_node *node;
  seni_node *parameter_prefix = NULL;
  seni_node *c;

  (*src)++; // {

  while (1) {
    c = eat_item(word_lut, src);
    if (c == NULL) {
      SENI_ERROR("unable to eat element of alterable");
      return NULL;
    }

    if (c->type == NODE_COMMENT || c->type == NODE_WHITESPACE) {
      DL_APPEND(parameter_prefix, c);
    } else {
      node                   = c;
      node->alterable        = 1;
      node->parameter_prefix = parameter_prefix;
      break;
    }
  }

  if (node->type != NODE_INT && node->type != NODE_FLOAT && node->type != NODE_NAME &&
      node->type != NODE_LIST && node->type != NODE_VECTOR) {
    SENI_ERROR("non-mutable node within curly brackets: %s", node_type_name(node));
    return NULL;
  }

  while (1) {
    if (is_alterable_end(**src)) {
      (*src)++; // }
      return node;
    }

    seni_node *child = eat_item(word_lut, src);
    if (child == NULL) {
      SENI_ERROR("unable to eat element of bracket");
      return NULL;
    }

    DL_APPEND(node->parameter_ast, child);
  }
}

seni_node *eat_quoted_form(seni_word_lut *word_lut, char **src) {
  (*src)++; // '

  seni_node *node = node_get_from_pool();
  node->type      = NODE_LIST;

  seni_node *quote_name = build_text_lookup_node_from_string(word_lut, NODE_NAME, "quote");
  DL_APPEND(node->value.first_child, quote_name);

  char *     wst = " ";
  seni_node *ws  = build_text_node_of_length(&wst, NODE_WHITESPACE, 1);
  DL_APPEND(node->value.first_child, ws);

  seni_node *child = eat_item(word_lut, src);
  DL_APPEND(node->value.first_child, child);

  return node;
}

seni_node *eat_float(char **src) {
  char *end_ptr;

  seni_node *node = node_get_from_pool();
  node->type      = NODE_FLOAT;
  node->value.f   = (f32)seni_strtof(*src, &end_ptr);

  node->src     = *src;
  node->src_len = (i32)(end_ptr - *src);

  *src = end_ptr;

  return node;
}

seni_node *eat_name(seni_word_lut *word_lut, char **src) {
  size_t i   = 0;
  char * rem = *src;

  while (rem[i]) {
    char c = rem[i];
    if (!is_alpha(c) && !is_digit(c) && !is_symbol(c)) {
      break;
    }
    i++;
  }

  seni_node *node = build_text_lookup_node_of_length(word_lut, src, NODE_NAME, i);

  return node;
}

seni_node *eat_string(seni_word_lut *word_lut, char **src) {
  (*src)++; // skip the first \"

  char *next_quote = find_next(*src, '\"');
  if (next_quote == NULL) {
    return NULL;
  }

  size_t string_len = next_quote - *src;

  seni_node *node = build_text_lookup_node_of_length(word_lut, src, NODE_STRING, string_len);

  (*src)++; // skip the second \"

  return node;
}

seni_node *eat_label(seni_word_lut *word_lut, char **src) {
  size_t i   = 0;
  char * rem = *src;

  while (rem[i]) {
    char c = rem[i];
    if (!is_alpha(c) && !is_digit(c) && !is_symbol(c)) {
      break;
    }
    i++;
  }

  // read the label name - the ':' character
  seni_node *node = build_text_lookup_node_of_length(word_lut, src, NODE_LABEL, i);

  if (**src != ':') {
    return NULL;
  }

  (*src)++; /* the remaining should skip past the ':' */

  return node;
}

seni_node *eat_comment(char **src) {
  size_t i   = 0;
  char * rem = *src;

  while (rem[i]) {
    char c = rem[i];
    if (is_newline(c)) {
      break;
    }
    i++;
  }

  seni_node *node = build_text_node_of_length(src, NODE_COMMENT, i);

  if (is_newline(*rem)) {
    (*src)++; /* skip past the newline */
  }

  return node;
}

seni_node *eat_whitespace(char **src) {
  size_t i   = 0;
  char * rem = *src;
  char   c   = rem[i];

  while (c) {
    if (!is_whitespace(c)) {
      break;
    }
    i++;
    c = rem[i];
  }

  seni_node *node = build_text_node_of_length(src, NODE_WHITESPACE, i);

  return node;
}

seni_node *eat_item(seni_word_lut *word_lut, char **src) {
  char c = **src;

  if (is_whitespace(c)) {
    return eat_whitespace(src);
  }

  if (is_quote_abbreviation(c)) {
    return eat_quoted_form(word_lut, src);
  }

  if (is_list_start(c)) {
    return eat_list(word_lut, src);
  }

  if (is_list_end(c)) {
    return NULL; /* 'mismatched closing parens' */
  }

  if (is_vector_start(c)) {
    return eat_vector(word_lut, src);
  }

  if (is_vector_end(c)) {
    return NULL; /* 'mismatched closing square brackets' */
  }

  if (is_alterable_start(c)) {
    return eat_alterable(word_lut, src);
  }

  if (is_alterable_end(c)) {
    return NULL; /* 'mismatched closing alterable brackets' */
  }

  if (is_quoted_string(c)) {
    return eat_string(word_lut, src);
  }

  if (is_alpha(c) || is_minus(c) || is_symbol(c)) {
    // doesn't begin with -[0..9]
    if (!(is_minus(c) && *(*src + 1) != 0 && is_digit(*(*src + 1)))) {

      char * s        = *src;
      size_t word_len = 0;

      while (*s != 0) {
        if (!is_alpha(*s) && !is_digit(*s) && !is_symbol(*s)) {
          break;
        }
        word_len++;
        s++;
      }

      if (is_label(*src, word_len)) {
        return eat_label(word_lut, src);
      }

      return eat_name(word_lut, src);
    }
  }

  if (is_digit(c) || is_minus(c) || is_period(c)) {
    return eat_float(src);
  }

  if (is_comment(c)) {
    return eat_comment(src);
  }
  return NULL;
}

void parser_return_nodes_to_pool(seni_node *nodes) {
  seni_node *node = nodes;
  seni_node *next;

  while (node != NULL) {
    if (node->type == NODE_LIST && node->value.first_child) {
      parser_return_nodes_to_pool(node->value.first_child);
    }
    if (node->parameter_ast) {
      parser_return_nodes_to_pool(node->parameter_ast);
    }
    if (node->parameter_prefix) {
      parser_return_nodes_to_pool(node->parameter_prefix);
    }

    next = node->next;

    node_return_to_pool(node);

    node = next;
  }
}

seni_node *parser_parse(seni_word_lut *word_lut, char *s) {
  if (s == NULL) {
    return NULL;
  }

  // clear out any words defined by previous scripts
  wlut_reset_words(word_lut);

  char **src = &s;

  seni_node *nodes = NULL;
  seni_node *node;

  while (**src) {
    node = eat_item(word_lut, src);
    if (node == NULL) {
      // clean up
      parser_return_nodes_to_pool(nodes);
      return NULL;
    }

    DL_APPEND(nodes, node);
  }

  // node_pool_pretty_print(g_node_pool);

  // NOTE: not strictly a tree as the ast root could have siblings
  return nodes;
}
