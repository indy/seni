#include "parser.h"

#include "lang.h"
#include "strtof.h"

#include "../lib/utlist.h"

#include <stdlib.h>
#include <string.h>

#include "multistring.h"
#include "pool_macro.h"

void node_cleanup(sen_node* node) {
  node->alterable         = 0;
  node->src               = NULL;
  node->src_len           = 0;
  node->value.first_child = NULL; // empty the value union
  node->parameter_ast     = NULL;
  node->parameter_prefix  = NULL;
}

SEN_POOL(sen_node, node)

struct sen_node_pool* g_node_pool;

void parser_subsystem_startup() {
  g_node_pool = node_pool_allocate(1, 1000, 20);
}

void parser_subsystem_shutdown() { node_pool_free(g_node_pool); }

sen_node* node_get_from_pool() {
  sen_node* node = node_pool_get(g_node_pool);
  RETURN_IF_NULL(node, "node_get_from_pool: OH NO NODE IS NULL");

  return node;
}

void node_return_to_pool(sen_node* node) {
  node_cleanup(node);
  node_pool_return(g_node_pool, node);
}

sen_result_node eat_item();

bool is_minus(char c) { return c == '-'; }

bool is_period(char c) { return c == '.'; }

bool is_whitespace(char c) {
  return c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == ',';
}

bool is_digit(char c) { return c >= '0' && c <= '9'; }

bool is_alpha(char c) {
  return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z');
}

bool is_symbol(char c) {
  return c == '+' || c == '-' || c == '*' || c == '/' || c == '=' || c == '!' ||
         c == '@' || c == '#' || c == '$' || c == '%' || c == '^' || c == '&' ||
         c == '<' || c == '>' || c == '?';
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

sen_result_i32 lookup_name(sen_string_ref* string_refs, i32 word_count,
                           i32 offset, char* string, size_t len) {
  i32             i          = 0;
  sen_string_ref* string_ref = string_refs;

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
      i32 res = i + offset;
      return result_i32_ok(res);
    }

    string_ref++;
  }

  return result_i32_error(ERROR_WLUT_LOOKUP_FAILED);
}

sen_result_i32 word_lut_lookup_(sen_word_lut* word_lut, char* string,
                                size_t len) {
  sen_result_i32 result_i32;

  result_i32 = lookup_name(word_lut->native_ref, word_lut->native_count,
                           NATIVE_START, string, len);
  if (is_result_i32_ok(result_i32)) {
    return result_i32;
  }

  result_i32 = lookup_name(word_lut->keyword_ref, word_lut->keyword_count,
                           KEYWORD_START, string, len);
  if (is_result_i32_ok(result_i32)) {
    return result_i32;
  }

  result_i32 = lookup_name(word_lut->word_ref, word_lut->word_count, WORD_START,
                           string, len);
  if (is_result_i32_ok(result_i32)) {
    return result_i32;
  }

  return result_i32_error(ERROR_WLUT_LOOKUP_FAILED);
}

sen_result_i32 word_lut_lookup_or_add(sen_word_lut* word_lut, char* string,
                                      size_t len) {
  sen_result_i32 result_i32 = word_lut_lookup_(word_lut, string, len);
  if (result_i32.error == NONE) {
    return result_i32;
  }

  // the string is not in the lookup table, so add it
  bool added = wlut_add_word(word_lut, string, len);
  if (added == false) {
    return result_i32_error(ERROR_WLUT_ADD_FAILED);
  }

  i32 res = word_lut->word_count - 1;
  return result_i32_ok(res);
}

sen_result_node build_text_lookup_node_from_string(sen_word_lut* word_lut,
                                                   sen_node_type type,
                                                   char*         string) {
  sen_result_node result_node;

  sen_node* node = node_get_from_pool();
  if (node == NULL) {
    return result_node_error(ERROR_NULL_NODE);
  }

  size_t         len        = strlen(string);
  sen_result_i32 result_i32 = word_lut_lookup_or_add(word_lut, string, len);
  if (result_i32.error != NONE) {
    result_node.error = result_i32.error;
    return result_node;
  }

  node->type    = type;
  node->value.i = result_i32.result;

  return result_node_ok(node);
}

sen_result_node build_text_lookup_node_of_length(sen_word_lut* word_lut,
                                                 char** src, sen_node_type type,
                                                 size_t len) {
  sen_result_node result_node;

  sen_node* node = node_get_from_pool();
  if (node == NULL) {
    return result_node_error(ERROR_NULL_NODE);
  }

  sen_result_i32 result_i32 = word_lut_lookup_or_add(word_lut, *src, len);
  if (result_i32.error != NONE) {
    result_node.error = result_i32.error;
    return result_node;
  }

  node->type    = type;
  node->value.i = result_i32.result;
  node->src     = *src;
  node->src_len = (i32)len;

  *src += len;

  return result_node_ok(node);
}

/*
sen_node* build_text_node_of_length(char** src, sen_node_type type,
                                    size_t len) {
  sen_node* node = node_get_from_pool();
  RETURN_IF_NULL(node, "build_text_node_of_length: NULL node");

  node->type    = type;
  node->src     = *src;
  node->src_len = (i32)len;

  *src += len;

  return node;
}
*/

// allocate memory for comments and whitespace rather than using the lookup
// table
//
sen_result_node build_text_node_of_length(char** src, sen_node_type type,
                                          size_t len) {
  sen_node* node = node_get_from_pool();
  if (node == NULL) {
    return result_node_error(ERROR_NULL_NODE);
  }

  node->type    = type;
  node->src     = *src;
  node->src_len = (i32)len;

  *src += len;

  return result_node_ok(node);
}

sen_result_node eat_list(sen_word_lut* word_lut, char** src) {
  sen_result_node result_node;

  sen_node* node = node_get_from_pool();
  if (node == NULL) {
    return result_node_error(ERROR_NULL_NODE);
  }

  node->type              = NODE_LIST;
  node->value.first_child = NULL;

  (*src)++; // (

  while (1) {
    if (is_list_end(**src)) {
      (*src)++; // )
      return result_node_ok(node);
    }

    result_node = eat_item(word_lut, src);
    if (result_node.error == ERROR_PARSE_END_OF_INPUT) {
      result_node.error = ERROR_PARSE_EXPECTED_END_OF_LIST;
      return result_node;
    }
    if (is_result_node_error(result_node)) {
      return result_node;
    }
    sen_node* child = result_node.result;

    DL_APPEND(node->value.first_child, child);
  }
}

sen_result_node eat_vector(sen_word_lut* word_lut, char** src) {
  sen_result_node result_node;

  sen_node* node = node_get_from_pool();
  if (node == NULL) {
    return result_node_error(ERROR_NULL_NODE);
  }

  node->type              = NODE_VECTOR;
  node->value.first_child = NULL;

  (*src)++; // [

  while (1) {
    if (is_vector_end(**src)) {
      (*src)++; // ]
      return result_node_ok(node);
    }

    result_node = eat_item(word_lut, src);
    if (result_node.error == ERROR_PARSE_END_OF_INPUT) {
      result_node.error = ERROR_PARSE_EXPECTED_END_OF_VECTOR;
      return result_node;
    }
    if (is_result_node_error(result_node)) {
      return result_node;
    }
    sen_node* child = result_node.result;

    DL_APPEND(node->value.first_child, child);
  }
}

sen_result_node eat_alterable(sen_word_lut* word_lut, char** src) {
  sen_result_node result_node;

  sen_node* node;
  sen_node* parameter_prefix = NULL;
  sen_node* c;

  (*src)++; // {

  while (1) {
    result_node = eat_item(word_lut, src);
    // "unable to eat element of alterable"
    if (is_result_node_error(result_node)) {
      return result_node;
    }

    c = result_node.result;
    if (c->type == NODE_COMMENT || c->type == NODE_WHITESPACE) {
      DL_APPEND(parameter_prefix, c);
    } else {
      node                   = c;
      node->alterable        = 1;
      node->parameter_prefix = parameter_prefix;
      break;
    }
  }

  if (node->type != NODE_INT && node->type != NODE_FLOAT &&
      node->type != NODE_NAME && node->type != NODE_LIST &&
      node->type != NODE_VECTOR) {
    SEN_ERROR("non-mutable node within curly brackets: %s",
              node_type_name(node));
    return result_node_error(ERROR_PARSE_NON_MUTABLE_NODE);
  }

  while (1) {
    if (is_alterable_end(**src)) {
      (*src)++; // }
      return result_node_ok(node);
    }

    result_node = eat_item(word_lut, src);
    // "unable to eat element of bracket"
    if (is_result_node_error(result_node)) {
      return result_node;
    }

    sen_node* child = result_node.result;

    DL_APPEND(node->parameter_ast, child);
  }
}

sen_result_node eat_quoted_form(sen_word_lut* word_lut, char** src) {
  sen_result_node result_node;

  (*src)++; // '

  sen_node* node = node_get_from_pool();
  if (node == NULL) {
    return result_node_error(ERROR_NULL_NODE);
  }

  node->type = NODE_LIST;

  result_node =
      build_text_lookup_node_from_string(word_lut, NODE_NAME, "quote");

  if (is_result_node_error(result_node)) {
    return result_node;
  }

  sen_node* quote_name = result_node.result;
  DL_APPEND(node->value.first_child, quote_name);

  char* wst   = " ";
  result_node = build_text_node_of_length(&wst, NODE_WHITESPACE, 1);
  // "eat_quoted_form: build_text_node_of_length"
  if (is_result_node_error(result_node)) {
    return result_node;
  }

  sen_node* ws = result_node.result;

  DL_APPEND(node->value.first_child, ws);

  result_node = eat_item(word_lut, src);
  // "eat_quoted_form: eat_item"
  if (is_result_node_error(result_node)) {
    return result_node;
  }

  sen_node* child = result_node.result;
  DL_APPEND(node->value.first_child, child);

  return result_node_ok(node);
}

sen_result_node eat_float(char** src) {
  char* end_ptr;

  sen_node* node = node_get_from_pool();
  if (node == NULL) {
    return result_node_error(ERROR_NULL_NODE);
  }

  node->type    = NODE_FLOAT;
  node->value.f = (f32)sen_strtof(*src, &end_ptr);
  node->src     = *src;
  node->src_len = (i32)(end_ptr - *src);

  *src = end_ptr;

  return result_node_ok(node);
}

sen_result_node eat_name(sen_word_lut* word_lut, char** src) {
  sen_result_node result_node;

  size_t i   = 0;
  char*  rem = *src;

  while (rem[i]) {
    char c = rem[i];
    if (!is_alpha(c) && !is_digit(c) && !is_symbol(c)) {
      break;
    }
    i++;
  }

  result_node = build_text_lookup_node_of_length(word_lut, src, NODE_NAME, i);
  // "eat_name: build_text_lookup_node_of_length"
  if (is_result_node_error(result_node)) {
    return result_node;
  }

  sen_node* node = result_node.result;
  return result_node_ok(node);
}

sen_result_node eat_string(sen_word_lut* word_lut, char** src) {
  sen_result_node result_node;

  (*src)++; // skip the first \"

  char* next_quote = find_next(*src, '\"');
  if (next_quote == NULL) {
    return result_node_error(ERROR_NULL_NODE);
  }

  size_t string_len = next_quote - *src;

  result_node =
      build_text_lookup_node_of_length(word_lut, src, NODE_STRING, string_len);
  // "eat_string: build_text_lookup_node_of_length"
  if (is_result_node_error(result_node)) {
    return result_node;
  }

  (*src)++; // skip the second \"

  sen_node* node = result_node.result;
  return result_node_ok(node);
}

sen_result_node eat_label(sen_word_lut* word_lut, char** src) {
  sen_result_node result_node;

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
  result_node = build_text_lookup_node_of_length(word_lut, src, NODE_LABEL, i);
  // "eat_label: build_text_lookup_node_of_length"
  if (is_result_node_error(result_node)) {
    return result_node;
  }

  if (**src != ':') {
    return result_node_error(ERROR_PARSE);
  }

  (*src)++; /* the remaining should skip past the ':' */

  sen_node* node = result_node.result;
  return result_node_ok(node);
}

sen_result_node eat_comment(char** src) {
  sen_result_node result_node;

  size_t i   = 0;
  char*  rem = *src;

  while (rem[i]) {
    char c = rem[i];
    if (is_newline(c)) {
      break;
    }
    i++;
  }

  result_node = build_text_node_of_length(src, NODE_COMMENT, i);
  // "eat_comment: build_text_node_of_length"
  if (is_result_node_error(result_node)) {
    return result_node;
  }

  if (is_newline(*rem)) {
    (*src)++; /* skip past the newline */
  }

  sen_node* node = result_node.result;
  return result_node_ok(node);
}

sen_result_node eat_whitespace(char** src) {
  sen_result_node result_node;

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

  result_node = build_text_node_of_length(src, NODE_WHITESPACE, i);
  // "eat_whitespace: build_text_node_of_length"
  if (is_result_node_error(result_node)) {
    return result_node;
  }

  sen_node* node = result_node.result;
  return result_node_ok(node);
}

sen_result_node eat_item(sen_word_lut* word_lut, char** src) {
  sen_result_node result_node;

  char c = **src;
  if (c == 0) {
    result_node.error = ERROR_PARSE_END_OF_INPUT;
    return result_node;
  }

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
    return result_node_error(ERROR_NULL_NODE);
  }

  if (is_vector_start(c)) {
    return eat_vector(word_lut, src);
  }

  if (is_vector_end(c)) {
    return result_node_error(ERROR_NULL_NODE);
  }

  if (is_alterable_start(c)) {
    return eat_alterable(word_lut, src);
  }

  if (is_alterable_end(c)) {
    return result_node_error(ERROR_NULL_NODE);
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

  return result_node_error(ERROR_NULL_NODE);
}

void parser_return_nodes_to_pool(sen_node* nodes) {
  sen_node* node = nodes;
  sen_node* next;

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

sen_result_node parser_parse(sen_word_lut* word_lut, char* s) {
  sen_result_node result_node;
  result_node.result = NULL;

  if (s == NULL) {
    return result_node_error(ERROR_PARSE_NULL_INPUT);
  }

  // clear out any words defined by previous scripts
  wlut_reset_words(word_lut);

  char** src = &s;

  sen_node* nodes = NULL;
  sen_node* node;

  while (**src) {
    result_node = eat_item(word_lut, src);
    if (result_node.error != NONE) {
      // clean up
      parser_return_nodes_to_pool(nodes);
      return result_node;
    }

    node = result_node.result;
    DL_APPEND(nodes, node);
  }

  // node_pool_pretty_print(g_node_pool);

  // NOTE: not strictly a tree as the ast root could have siblings
  return result_node_ok(nodes);
}
