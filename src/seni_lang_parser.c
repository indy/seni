#include <string.h>
#include <stdlib.h>
#include <inttypes.h>
#include <stdio.h>              /* for debug only */

#include "seni_lang_parser.h"
#include "seni_containers.h"

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
  return (c == ' ' || c == '\t' || c == '\n' || c == ',') ? true : false;
}

bool is_digit(char c)
{
  return (c >= '0' && c <= '9') ? true : false;
}

bool is_alpha(char c)
{
  return ((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')) ? true : false;
}

bool is_symbol(char c)
{
  return (c == '+' || c == '-' || c == '*' || c == '/' || c == '=' ||
          c == '!' || c == '@' || c == '#' || c == '$' || c == '%' ||
          c == '^' || c == '&' || c == '<' || c == '>' || c == '?') ? true : false;
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

bool is_boolean_true(char *s, size_t word_len)
{
  return word_len == 4 && s[0] == 't' && s[1] == 'r' && s[2] == 'u' && s[3] == 'e';
}

bool is_boolean_false(char *s, size_t word_len)
{
  return word_len == 5 && s[0] == 'f' && s[1] == 'a' && s[2] == 'l' && s[3] == 's' && s[4] == 'e';
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

seni_node *build_text_lookup_node_from_string(word_lut *wlut, seni_node_type type, char *string)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  size_t len = strlen(string);

  i32 k = word_lookup_or_add(wlut, string, len);
  if (k == -1) {
    return NULL;
  }

  node->type = type;
  node->value.i = k;
  
  return node;
}

seni_node *build_text_lookup_node_of_length(word_lut *wlut, char **src, seni_node_type type, size_t len)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));

  i32 k = word_lookup_or_add(wlut, *src, len);
  if (k == -1) {
    return NULL;
  }

  node->type = type;
  node->value.i = k;
  *src += len;

  return node;
}

// allocate memory for comments and whitespace rather than using the lookup table
//
seni_node *build_text_node_of_length(char **src, seni_node_type type, size_t len)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = type;

  char *str = (char *)malloc(sizeof(char) * (len + 1));
  strncpy(str, *src, len);
  str[len] = '\0';

  *src += len;
  
  node->value.s = str;
  
  return node;
}

seni_node *consume_list(word_lut *wlut, char **src)
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
      /* error? */
      return NULL;
    }

    DL_APPEND(node->children, child);
  }
}

seni_node *consume_vector(word_lut *wlut, char **src)
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
      /* error? */
      return NULL;
    }

    DL_APPEND(node->children, child);
  }
}

seni_node *consume_bracket(word_lut *wlut, char **src)
{
  seni_node *node;
  seni_node *parameter_prefix = NULL;
  seni_node *c;
  
  (*src)++; // {
  
  while (1) {
    c = consume_item(wlut, src);
    if (c == NULL) {
      /* error? */
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

  /* TODO: sanity check the parameter prefixes */
  /* 
  prefixParameters.forEach(pp => node.addParameterNodePrefix(pp));

  if (nodeType !== NodeType.BOOLEAN &&
      nodeType !== NodeType.INT &&
      nodeType !== NodeType.FLOAT &&
      nodeType !== NodeType.NAME &&
      nodeType !== NodeType.STRING &&
      nodeType !== NodeType.LIST &&
      nodeType !== NodeType.VECTOR) {
    console.log('whooops', tokens, node);
    return {error: `non-mutable node within curly brackets ${nodeType}`};
  }
  */
  
  while (1) {
    if (is_alterable_end(**src)) {
      (*src)++; // }
      return node;
    }

    seni_node *child = consume_item(wlut, src);
    if (child == NULL) {
      /* error? */
      return NULL;
    }

    DL_APPEND(node->parameter_ast, child);
  }
}

seni_node *consume_quoted_form(word_lut *wlut, char **src)
{
  (*src)++; // '
  
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_LIST;

  seni_node *quote_name = build_text_lookup_node_from_string(wlut, NODE_NAME, "quote");
  DL_APPEND(node->children, quote_name);

  char *wst = " ";
  seni_node *ws = build_text_node_of_length(&wst, NODE_WHITESPACE, 1);
  DL_APPEND(node->children, ws);

  seni_node *child = consume_item(wlut, src);
  DL_APPEND(node->children, child);

  return node;
}

seni_node *consume_int(char **src)
{
  char *end_ptr;
  
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_INT;
  node->value.i = (i32)strtoimax(*src, &end_ptr, 10);

  *src = end_ptr;
  
  return node;
}

seni_node *consume_float(char **src)
{
  char *end_ptr;
  
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_FLOAT;
  node->value.f = (f32)strtof(*src, &end_ptr);

  *src = end_ptr;
  
  return node;
}

seni_node *consume_boolean(char **src, bool val)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_BOOLEAN;
  node->value.i = val;

  if (val == true) {
    (*src) += 4;                /* 'true' */
  } else {
    (*src) += 5;                /* 'false' */
  }
  
  return node;
}


seni_node *consume_name(word_lut *wlut, char **src)
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

seni_node *consume_string(word_lut *wlut, char **src)
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

seni_node *consume_label(word_lut *wlut, char **src)
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

seni_node *consume_item(word_lut *wlut, char **src)
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
    return consume_bracket(wlut, src);
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

      if (is_boolean_true(*src, word_len)) {
        return consume_boolean(src, true);
      }

      if (is_boolean_false(*src, word_len)) {
        return consume_boolean(src, false);
      } 

      return consume_name(wlut, src);
    }
  }
  
  if (is_digit(c) || is_minus(c) || is_period(c)) {
    if (has_period(*src)) {
      return consume_float(src);
    } else {
      return consume_int(src);
    }
  }

  if (is_comment(c)) {
    return consume_comment(src);
  }
  return NULL;
}

char *parser_node_type_name(seni_node_type type)
{
  switch(type) {
  case NODE_LIST: return "NODE_LIST";
  case NODE_VECTOR: return "NODE_VECTOR";
  case NODE_INT: return "NODE_INT";
  case NODE_FLOAT: return "NODE_FLOAT";
  case NODE_NAME: return "NODE_NAME";
  case NODE_LABEL: return "NODE_LABEL";
  case NODE_STRING: return "NODE_STRING";
  case NODE_BOOLEAN: return "NODE_BOOLEAN";
  case NODE_WHITESPACE: return "NODE_WHITESPACE";
  case NODE_COMMENT: return "NODE_COMMENT";
  default: return "unknown node type";
  };
}

void parser_free_nodes(seni_node *nodes)
{
  seni_node *node = nodes;
  seni_node *next;

  while(node != NULL) {
    if (node->children) {
      parser_free_nodes(node->children);
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

    // printf("freeing node: %s %u\n", parser_node_type_name(node->type), (u32)node);
    free(node);
    
    node = next;
  }
}

seni_node *parser_parse(word_lut *wlut, char *s)
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
      // clean up and fuck off
      parser_free_nodes(nodes);
      return NULL;
    }

    DL_APPEND(nodes, node);
  }

  // NOTE: not strictly a tree as the ast root could have siblings
  return nodes;
}
