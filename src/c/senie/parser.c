#include "parser.h"

#include "lang.h"
#include "strtof.h"

#include "../lib/utlist.h"

#include <stdlib.h>
#include <string.h>

#include "multistring.h"
#include "pool_macro.h"

void node_cleanup(senie_node* node) {
  node->alterable         = 0;
  node->src               = NULL;
  node->src_len           = 0;
  node->value.first_child = NULL; // empty the value union
  node->parameter_ast     = NULL;
  node->parameter_prefix  = NULL;
}

SENIE_POOL(senie_node, node)

struct senie_node_pool* g_node_pool;

void parser_subsystem_startup() { g_node_pool = node_pool_allocate(1, 1000, 20); }

void parser_subsystem_shutdown() { node_pool_free(g_node_pool); }

senie_node* node_get_from_pool() {
  senie_node* node = node_pool_get(g_node_pool);
  RETURN_IF_NULL(node, "node_get_from_pool: OH NO NODE IS NULL");

  return node;
}

void node_return_to_pool(senie_node* node) {
  node_cleanup(node);
  node_pool_return(g_node_pool, node);
}

senie_node* eat_item();

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

bool is_label(char* s, size_t word_len) { return s[word_len] == ':'; }

bool has_period(char* s) {
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

char* find_next(char* s, char target) {
  while (*s != 0) {
    if (*s == target) {
      return s;
    }
    s++;
  }
  return NULL;
}

/* returns 0 if not found */
i32 lookup_name(senie_string_ref* string_refs,
                i32               word_count,
                i32               offset,
                char*             string,
                size_t            len) {
  i32               i          = 0;
  senie_string_ref* string_ref = string_refs;

  for (i = 0; i < word_count; i++) {
    char* name  = string_ref->c;
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

i32 word_lut_lookup_(senie_word_lut* word_lut, char* string, size_t len) {
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

i32 word_lut_lookup_or_add(senie_word_lut* word_lut, char* string, size_t len) {
  i32 iname = word_lut_lookup_(word_lut, string, len);
  if (iname != -1) {
    return iname;
  }

  // the string is not in the lookup table, so add it
  bool res = wlut_add_word(word_lut, string, len);
  if (res == false) {
    SENIE_ERROR("word_lut_lookup_or_add failed");
    return 0;
  }

  return word_lut->word_count - 1;
}

senie_node*
build_text_lookup_node_from_string(senie_word_lut* word_lut, senie_node_type type, char* string) {
  senie_node* node = node_get_from_pool();
  RETURN_IF_NULL(node, "build_text_lookup_node_from_string: NULL node");

  size_t len = strlen(string);
  i32    k   = word_lut_lookup_or_add(word_lut, string, len);
  if (k == -1) {
    return NULL;
  }

  node->type    = type;
  node->value.i = k;

  return node;
}

senie_node* build_text_lookup_node_of_length(senie_word_lut* word_lut,
                                             char**          src,
                                             senie_node_type type,
                                             size_t          len) {
  senie_node* node = node_get_from_pool();
  RETURN_IF_NULL(node, "build_text_lookup_node_of_length: NULL node");

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
senie_node* build_text_node_of_length(char** src, senie_node_type type, size_t len) {
  senie_node* node = node_get_from_pool();
  RETURN_IF_NULL(node, "build_text_node_of_length: NULL node");

  node->type    = type;
  node->src     = *src;
  node->src_len = (i32)len;

  *src += len;

  return node;
}

senie_node* eat_list(senie_word_lut* word_lut, char** src) {
  senie_node* node = node_get_from_pool();
  RETURN_IF_NULL(node, "eat_list: NULL node");

  node->type              = NODE_LIST;
  node->value.first_child = NULL;

  (*src)++; // (

  while (1) {
    if (is_list_end(**src)) {
      (*src)++; // )
      return node;
    }

    senie_node* child = eat_item(word_lut, src);
    RETURN_IF_NULL(child, "unable to eat element of list");

    DL_APPEND(node->value.first_child, child);
  }
}

senie_node* eat_vector(senie_word_lut* word_lut, char** src) {
  senie_node* node = node_get_from_pool();
  RETURN_IF_NULL(node, "eat_vector: NULL node");

  node->type              = NODE_VECTOR;
  node->value.first_child = NULL;

  (*src)++; // [

  while (1) {
    if (is_vector_end(**src)) {
      (*src)++; // ]
      return node;
    }

    senie_node* child = eat_item(word_lut, src);
    RETURN_IF_NULL(child, "unable to eat element of vector");

    DL_APPEND(node->value.first_child, child);
  }
}

senie_node* eat_alterable(senie_word_lut* word_lut, char** src) {
  senie_node* node;
  senie_node* parameter_prefix = NULL;
  senie_node* c;

  (*src)++; // {

  while (1) {
    c = eat_item(word_lut, src);
    RETURN_IF_NULL(c, "unable to eat element of alterable");

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
    SENIE_ERROR("non-mutable node within curly brackets: %s", node_type_name(node));
    return NULL;
  }

  while (1) {
    if (is_alterable_end(**src)) {
      (*src)++; // }
      return node;
    }

    senie_node* child = eat_item(word_lut, src);
    RETURN_IF_NULL(child, "unable to eat element of bracket");

    DL_APPEND(node->parameter_ast, child);
  }
}

senie_node* eat_quoted_form(senie_word_lut* word_lut, char** src) {
  (*src)++; // '

  senie_node* node = node_get_from_pool();
  RETURN_IF_NULL(node, "eat_quoted_form: NULL node");

  node->type = NODE_LIST;

  senie_node* quote_name = build_text_lookup_node_from_string(word_lut, NODE_NAME, "quote");
  RETURN_IF_NULL(quote_name, "eat_quoted_form: quote_name");
  DL_APPEND(node->value.first_child, quote_name);

  char*       wst = " ";
  senie_node* ws  = build_text_node_of_length(&wst, NODE_WHITESPACE, 1);
  RETURN_IF_NULL(ws, "eat_quoted_form: build_text_node_of_length");
  DL_APPEND(node->value.first_child, ws);

  senie_node* child = eat_item(word_lut, src);
  RETURN_IF_NULL(child, "eat_quoted_form: eat_item");
  DL_APPEND(node->value.first_child, child);

  return node;
}

senie_node* eat_float(char** src) {
  char* end_ptr;

  senie_node* node = node_get_from_pool();
  RETURN_IF_NULL(node, "eat_float: NULL node");

  node->type    = NODE_FLOAT;
  node->value.f = (f32)senie_strtof(*src, &end_ptr);
  node->src     = *src;
  node->src_len = (i32)(end_ptr - *src);

  *src = end_ptr;

  return node;
}

senie_node* eat_name(senie_word_lut* word_lut, char** src) {
  size_t i   = 0;
  char*  rem = *src;

  while (rem[i]) {
    char c = rem[i];
    if (!is_alpha(c) && !is_digit(c) && !is_symbol(c)) {
      break;
    }
    i++;
  }

  senie_node* node = build_text_lookup_node_of_length(word_lut, src, NODE_NAME, i);
  return node;
}

senie_node* eat_string(senie_word_lut* word_lut, char** src) {
  (*src)++; // skip the first \"

  char* next_quote = find_next(*src, '\"');
  RETURN_IF_NULL(next_quote, "eat_string: cannot find closing quote");

  size_t string_len = next_quote - *src;

  senie_node* node = build_text_lookup_node_of_length(word_lut, src, NODE_STRING, string_len);
  RETURN_IF_NULL(node, "eat_string");

  (*src)++; // skip the second \"

  return node;
}

senie_node* eat_label(senie_word_lut* word_lut, char** src) {
  size_t i   = 0;
  char*  rem = *src;

  while (rem[i]) {
    char c = rem[i];
    if (!is_alpha(c) && !is_digit(c) && !is_symbol(c)) {
      break;
    }
    i++;
  }

  // read the label name - the ':' character
  senie_node* node = build_text_lookup_node_of_length(word_lut, src, NODE_LABEL, i);
  RETURN_IF_NULL(node, "eat_label: build_text_lookup_node_of_length");

  if (**src != ':') {
    return NULL;
  }

  (*src)++; /* the remaining should skip past the ':' */

  return node;
}

senie_node* eat_comment(char** src) {
  size_t i   = 0;
  char*  rem = *src;

  while (rem[i]) {
    char c = rem[i];
    if (is_newline(c)) {
      break;
    }
    i++;
  }

  senie_node* node = build_text_node_of_length(src, NODE_COMMENT, i);
  RETURN_IF_NULL(node, "eat_comment: build_text_node_of_length");

  if (is_newline(*rem)) {
    (*src)++; /* skip past the newline */
  }

  return node;
}

senie_node* eat_whitespace(char** src) {
  size_t i   = 0;
  char*  rem = *src;
  char   c   = rem[i];

  while (c) {
    if (!is_whitespace(c)) {
      break;
    }
    i++;
    c = rem[i];
  }

  senie_node* node = build_text_node_of_length(src, NODE_WHITESPACE, i);

  return node;
}

senie_node* eat_item(senie_word_lut* word_lut, char** src) {
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

      char*  s        = *src;
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

void parser_return_nodes_to_pool(senie_node* nodes) {
  senie_node* node = nodes;
  senie_node* next;

  while (node != NULL) {
    if (node->type == NODE_LIST && node->value.first_child) {
      parser_return_nodes_to_pool(node->value.first_child);
    }
    if (node->type == NODE_VECTOR && node->value.first_child) {
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

senie_node* parser_parse(senie_word_lut* word_lut, char* s) {
  RETURN_IF_NULL(s, "parser_parse: s");

  // clear out any words defined by previous scripts
  wlut_reset_words(word_lut);

  char** src = &s;

  senie_node* nodes = NULL;
  senie_node* node;

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
