#ifndef SENI_LANG_PARSER_H
#define SENI_LANG_PARSER_H

#include "seni_types.h"
#include "seni_lang_word_lookup.h"

typedef enum {
  NODE_LIST = 0,
  NODE_VECTOR,
  NODE_INT,
  NODE_FLOAT,
  NODE_NAME,
  NODE_LABEL,
  NODE_STRING,
  NODE_BOOLEAN,
  NODE_LAMBDA,
  NODE_SPECIAL,
  NODE_COLOUR,
  NODE_WHITESPACE,
  NODE_COMMENT,
  NODE_NULL
} seni_node_type;


typedef struct seni_node {
  seni_node_type type;

  union {
    i32 i;
    f32 f;
    char* s;                    /* needed for whitespace/comment nodes */
  } value;

  bool alterable;

  /* node list functionality */
  struct seni_node *children;

  // node mutate specific
  struct seni_node *parameter_ast;

  // need a place for nodes that occur within curly brackets that should
  // be ignored, e.g. the whitespace before the 2 in: (+ 1 { 2} (int))
  struct seni_node *parameter_prefix;

  /* NOTE: parameter_ast, parameter_prefix, children */
  /* for children */
  struct seni_node *prev;
  struct seni_node *next;
} seni_node;

seni_node *parser_parse(word_lookup *word_lookup, char *s);
void parser_free_nodes(seni_node *nodes);
char *parser_node_type_name(seni_node_type type);
  
#endif
