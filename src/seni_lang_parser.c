#include <string.h>
#include <stdlib.h>
#include <inttypes.h>
#include <stdio.h>              /* for debug only */

#include "seni_lang_parser.h"
#include "seni_containers.h"

seni_node *consume_item();

char* remaining;                /* global */

char* chars_whitespace = " \t\n,";
char* chars_digit = "0123456789";
char* chars_alpha = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ+-*/<>=";
char* chars_symbol = "-!@#$%^&*<>?";

char MINUS = '-';
char PERIOD = '.';

bool contains(char c, char *p)
{
  while(*p != 0) {
    if (*p == c) {
      return true;
    }
    p++;
  }
  
  return false;
}

bool is_whitespace(char c)
{
  return contains(c, chars_whitespace);
}

bool is_digit(char c)
{
  return contains(c, chars_digit);
}

bool is_alpha(char c)
{
  return contains(c, chars_alpha);
}

bool is_symbol(char c)
{
  return contains(c, chars_symbol);
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
  size_t i = 0;
  char c = s[i];

  while(c != 0) {
    if (!is_alpha(c) && !is_digit(c) && !is_symbol(c)) {
      break;
    }
    i++;
    c = s[i];
  }

  return c != 0 && s[i] == ':';
}

bool has_period(char *s)
{
  size_t i = 0;
  char c = s[i];

  while (c != 0) {
    if (c == PERIOD) {
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

bool string_compare(char* a, char *b)
{
#if defined(_WIN32)
  return _stricmp(a, b) == 0;
#else
  return strcasecmp(a, b) == 0;
#endif
}

seni_node *build_text_node_from_string(seni_node_type type, char *string)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  size_t len = strlen(string);

  node->type = type;
  node->str_value = (char *)malloc(sizeof(char) * (len + 1));
  strncpy(node->str_value, string, len);
  node->str_value[len] = '\0';
  
  return node;
}

seni_node *build_text_node_of_length(seni_node_type type, size_t len)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = type;

  char *str = (char *)malloc(sizeof(char) * (len + 1));
  strncpy(str, remaining, len);
  str[len] = '\0';

  remaining += len;
  
  node->str_value = str;
  
  return node;
}

seni_node *consume_list()
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_LIST;

  remaining++; // (

  while (1) {
    if (is_list_end(*remaining)) {
      remaining++; // )
      return node;
    }

    seni_node *child = consume_item();
    if (child == NULL) {
      /* error? */
      return NULL;
    }

    DL_APPEND(node->children, child);
  }
}

seni_node *consume_vector()
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_VECTOR;

  remaining++; // [
  
  while (1) {
    if (is_vector_end(*remaining)) {
      remaining++; // ]
      return node;
    }

    seni_node *child = consume_item();
    if (child == NULL) {
      /* error? */
      return NULL;
    }

    DL_APPEND(node->children, child);
  }
}

seni_node *consume_bracket()
{
  seni_node *node;
  seni_node *parameter_prefix = NULL;
  seni_node *c;
  
  remaining++; // {
  
  while (1) {
    c = consume_item();
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
    if (is_alterable_end(*remaining)) {
      remaining++; // }
      return node;
    }

    seni_node *child = consume_item();
    if (child == NULL) {
      /* error? */
      return NULL;
    }

    DL_APPEND(node->parameter_ast, child);
  }
}

seni_node *consume_quoted_form()
{
  remaining++;
  
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_LIST;

  seni_node *quote_name = build_text_node_from_string(NODE_NAME, "quote");
  DL_APPEND(node->children, quote_name);

  seni_node *ws = build_text_node_from_string(NODE_WHITESPACE, " ");
  DL_APPEND(node->children, ws);

  seni_node *child = consume_item();
  DL_APPEND(node->children, child);

  return node;
}

seni_node *consume_int()
{
  char *end_ptr;
  
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_INT;
  node->i32_value = (i32)strtoimax(remaining, &end_ptr, 10);

  remaining = end_ptr;
  
  return node;
}

seni_node *consume_float()
{
  char *end_ptr;
  
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_FLOAT;
  node->f32_value = (f32)strtof(remaining, &end_ptr);

  remaining = end_ptr;
  
  return node;
}


seni_node *consume_name_or_boolean()
{
  size_t i = 0;

  while(remaining[i]) {
    char c = remaining[i];
    if (!is_alpha(c) && !is_digit(c) && !is_symbol(c)) {
      break;
    }
    i++;
  }

  seni_node *node = build_text_node_of_length(NODE_NAME, i);

  if (string_compare(node->str_value, "true") == true) {
    node->type = NODE_BOOLEAN;
    free(node->str_value);
    node->i32_value = true;
  } else if (string_compare(node->str_value, "false") == true) {
    node->type = NODE_BOOLEAN;
    free(node->str_value);
    node->i32_value = false;
  }

  return node;
}

seni_node *consume_boolean(bool val)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_BOOLEAN;
  node->i32_value = val;

  return node;
}

seni_node *consume_string()
{
  remaining++; // skip the first \"

  char *next_quote = find_next(remaining, '\"');
  if (next_quote == NULL) {
    return NULL;
  }

  size_t string_len = next_quote - remaining;

  seni_node *node = build_text_node_of_length(NODE_STRING, string_len);

  remaining++; // skip the second \"
  
  return node;
}

seni_node *consume_label()
{
  size_t i = 0;

  while(remaining[i]) {
    char c = remaining[i];
    if (!is_alpha(c) && !is_digit(c) && !is_symbol(c)) {
      break;
    }
    i++;
  }

  // read the label name - the ':' character
  seni_node *node = build_text_node_of_length(NODE_LABEL, i);

  if (*remaining != ':') {
    return NULL;
  }

  remaining += 1;        /* the remaining should skip past the ':' */

  return node;
}

seni_node *consume_comment()
{
  size_t i = 0;

  while (remaining[i]) {
    char c = remaining[i];
    if (is_newline(c)) {
      break;
    }
    i++;
  }

  seni_node *node = build_text_node_of_length(NODE_COMMENT, i);

  if (is_newline(*remaining)) {
    remaining += 1;        /* skip past the newline */
  }
    
  return node;
}

seni_node *consume_whitespace()
{
  size_t i = 0;
  char c = remaining[i];
  
  while(c) {
    if (!is_whitespace(c)) {
      break;
    }
    i++;
    c = remaining[i];
  }

  seni_node *node = build_text_node_of_length(NODE_WHITESPACE, i);

  return node;
}

seni_node *consume_item()
{
  char c = *remaining;

  if (is_whitespace(c)) {
    return consume_whitespace();
  }

  if (is_quote_abbreviation(c)) {
    return consume_quoted_form();
  }

  if (is_list_start(c)) {
    return consume_list();
  }

  if (is_list_end(c)) {
    return NULL;                /* 'mismatched closing parens' */
  }

  if (is_vector_start(c)) {
    return consume_vector();
  }

  if (is_vector_end(c)) {
    return NULL;                /* 'mismatched closing square brackets' */
  }

  if (is_alterable_start(c)) {
    return consume_bracket();
  }

  if (is_alterable_end(c)) {
    return NULL;                /* 'mismatched closing alterable brackets' */
  }

  if (is_quoted_string(c)) {
    return consume_string();
  }

  if (is_alpha(c)) {
    if (!(c == MINUS && *(remaining + 1) != 0 && is_digit(remaining[1]))) {
      if (is_label(remaining)) {
        return consume_label();
      } else {
        return consume_name_or_boolean();
      }
    }
  }
  
  if (is_digit(c) || c == MINUS || c == PERIOD) {
    if (has_period(remaining)) {
      return consume_float();
    } else {
      return consume_int();
    }
  }

  if (is_comment(c)) {
    return consume_comment();
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
  case NODE_LAMBDA: return "NODE_LAMBDA";
  case NODE_SPECIAL: return "NODE_SPECIAL";
  case NODE_COLOUR: return "NODE_COLOUR";
  case NODE_WHITESPACE: return "NODE_WHITESPACE";
  case NODE_COMMENT: return "NODE_COMMENT";
  case NODE_NULL: return "NODE_NULL";
  };
  return "";
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
    
    if (node->str_value != NULL) {
      free(node->str_value);
    }

    // printf("freeing node: %s %u\n", parser_node_type_name(node->type), (u32)node);
    free(node);
    
    node = next;
  }
}

seni_node *parser_parse(char *s)
{
  if (s == NULL) {
    return NULL;
  }

  remaining = s;

  seni_node *nodes = NULL;
  seni_node *node;

  while(*remaining) {
    node = consume_item();

    if (node == NULL) {
      // clean up and fuck off
      parser_free_nodes(nodes);
      return NULL;
    }
    
    DL_APPEND(nodes, node);
  }

  return nodes;
}
