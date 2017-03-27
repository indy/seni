#include <string.h>

#include "stdio.h"              /* for debug only */
#include "seni_lang_parser.h"

#include "seni_containers.h"

seni_node *parser_consume_item(seni_token **pptokens);

bool parser_string_compare(char* a, char *b)
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

seni_node *build_text_node_from_token(seni_node_type type, seni_token *token)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = type;
  node->str_value = token->str_value;

  // clear token->str_value since the seni_token is now re-using that memory
  token->str_value = NULL;
  
  return node;
}

void debug_token(seni_token *token)
{
  printf("token %d %p\n", token->type, (void *)token);
}

/* 
   the list-like nodes can allocate an array of tokens to store it's children?
 */

seni_node *parser_consume_list(seni_token **pptokens)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_LIST;

  while (1) {
    seni_token *token = *pptokens;
    if (token == NULL) {
      /* unexpected end of list */
      return NULL;
    }

    if (token->type == TOK_LIST_END) {
      *pptokens = token->next;
      return node;
    }

    seni_node *child = parser_consume_item(pptokens);
    if (child == NULL) {
      /* error? */
      return NULL;
    }

    DL_APPEND(node->children, child);
  }
}

seni_node *parser_consume_vector(seni_token **pptokens)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_VECTOR;

  while (1) {
    seni_token *token = *pptokens;
    if (token == NULL) {
      /* unexpected end of vector */
      return NULL;
    }

    if (token->type == TOK_VECTOR_END) {
      *pptokens = token->next;
      return node;
    }

    seni_node *child = parser_consume_item(pptokens);
    if (child == NULL) {
      /* error? */
      return NULL;
    }

    DL_APPEND(node->children, child);
  }
}

seni_node *parser_consume_bracket(seni_token **pptokens)
{
  seni_node *node;
  seni_node *parameter_prefix = NULL;

  while (1) {
    seni_node *c = parser_consume_item(pptokens);
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
    seni_token *token = *pptokens;
    if (token == NULL) {
      /* unexpected end of vector */
      return NULL;
    }

    if (token->type == TOK_ALTERABLE_END) {
      *pptokens = token->next;
      return node;
    }

    seni_node *child = parser_consume_item(pptokens);
    if (child == NULL) {
      /* error? */
      return NULL;
    }

    DL_APPEND(node->parameter_ast, child);
  }
}

seni_node *parser_consume_quoted_form(seni_token **pptokens)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_LIST;

  seni_node *quote_name = build_text_node_from_string(NODE_NAME, "quote");
  DL_APPEND(node->children, quote_name);

  seni_node *ws = build_text_node_from_string(NODE_WHITESPACE, " ");
  DL_APPEND(node->children, ws);

  seni_node *child = parser_consume_item(pptokens);
  DL_APPEND(node->children, child);

  return node;
}

seni_node *parser_consume_int(seni_token *token)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_INT;
  node->i32_value = token->i32_value;
  
  return node;
}

seni_node *parser_consume_float(seni_token *token)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_FLOAT;
  node->f32_value = token->f32_value;

  return node;
}

seni_node *parser_consume_boolean(bool val)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_BOOLEAN;
  node->i32_value = val;

  return node;
}

seni_node *parser_consume_name(seni_token *token)
{
  return build_text_node_from_token(NODE_NAME, token);
}

seni_node *parser_consume_string(seni_token *token)
{
  return build_text_node_from_token(NODE_STRING, token);
}

seni_node *parser_consume_label(seni_token *token)
{
  return build_text_node_from_token(NODE_LABEL, token);
}

seni_node *parser_consume_comment(seni_token *token)
{
  return build_text_node_from_token(NODE_COMMENT, token);
}

seni_node *parser_consume_whitespace(seni_token *token)
{
  return build_text_node_from_token(NODE_WHITESPACE, token);
}

seni_node *parser_consume_item(seni_token **pptokens)
{
  seni_token *token = *pptokens;
  *pptokens = token->next;

  // debug_token(token);
  
  seni_token_type token_type = token->type;
  switch (token_type) {
  case TOK_LIST_START:
    return parser_consume_list(pptokens);
    break;
  case TOK_LIST_END:
    return NULL;                /* 'mismatched closing parens' */
    break;
  case TOK_VECTOR_START:
    return parser_consume_vector(pptokens);
    break;
  case TOK_VECTOR_END:
    return NULL;                /* 'mismatched closing square brackets' */
    break;
  case TOK_ALTERABLE_START:
    return parser_consume_bracket(pptokens);
    break;
  case TOK_ALTERABLE_END:
    return NULL;                /* 'mismatched closing alterable brackets' */
    break;
  case TOK_INT:
    return parser_consume_int(token);
    break;
  case TOK_FLOAT:
    return parser_consume_float(token);
    break;
  case TOK_NAME:
    if(parser_string_compare(token->str_value, "true") == true) {
      return parser_consume_boolean(true);
    } else if(parser_string_compare(token->str_value, "false") == true) {
      return parser_consume_boolean(false);
    } else {
      return parser_consume_name(token);
    }
    break;
  case TOK_STRING:
    return parser_consume_string(token);
    break;
  case TOK_QUOTE_ABBREVIATION:
    return parser_consume_quoted_form(pptokens);
    break;
  case TOK_LABEL:
    return parser_consume_label(token);
    break;
  case TOK_COMMENT:
    return parser_consume_comment(token);
    break;
  case TOK_WHITESPACE:
    return parser_consume_whitespace(token);
    break;
  default:
    return NULL;
  };
}

char *node_type_as_string(seni_node_type type)
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

    // printf("freeing node: %s %u\n", node_type_as_string(node->type), (u32)node);
    free(node);
    
    node = next;
  }
}

seni_node *parser_parse(seni_token *tokens)
{
  seni_node *nodes = NULL;
  seni_node *node;

  seni_token **pptokens = &tokens;

  while (*pptokens != NULL) {
    node = parser_consume_item(pptokens);

    if (node == NULL) {
      // clean up and fuck off
      parser_free_nodes(nodes);
      return NULL;
    }
    
    DL_APPEND(nodes, node);
  }

  return nodes;
}
