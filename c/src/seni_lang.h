#ifndef SENI_LANG_H
#define SENI_LANG_H

#include "seni_types.h"
#include "seni_containers.h"

// 2 << 7 == 128
#define MAX_WORD_LOOKUPS (2 << 7)
#define MAX_KEYWORD_LOOKUPS MAX_WORD_LOOKUPS
#define KEYWORD_START MAX_WORD_LOOKUPS

/* word lookup table */
typedef struct word_lut {
  // filled in by interpreter: add_keywords_to_word_lookup
  char *keywords[MAX_KEYWORD_LOOKUPS];  
  i32 keywords_count;
  
  char *words[MAX_WORD_LOOKUPS];
  i32 words_count;
} word_lut;

typedef enum {
  NODE_LIST = 0,
  NODE_VECTOR,
  NODE_INT,
  NODE_FLOAT,
  NODE_NAME,
  NODE_LABEL,
  NODE_STRING,
  NODE_BOOLEAN,
  NODE_WHITESPACE,
  NODE_COMMENT
} seni_node_type;

typedef struct seni_node {
  seni_node_type type;

  union {
    i32 i;
    f32 f;
    char* s;                     /* needed for whitespace/comment nodes */
    struct seni_node *children;  /* list node */
  } value;

  bool alterable;

  // node mutate specific
  struct seni_node *parameter_ast;

  // need a place for nodes that occur within curly brackets that should
  // be ignored, e.g. the whitespace before the 2 in: (+ 1 { 2} (int))
  struct seni_node *parameter_prefix;

  /* for parameter_ast, parameter_prefix, children */
  struct seni_node *prev;
  struct seni_node *next;
} seni_node;

// start at 128 just to make it easier to spot mistakes when transforming seni_node_type -> seni_var_type
typedef enum {
  VAR_INT = 128, // value.i
  VAR_FLOAT,     // value.f
  VAR_BOOLEAN,   // value.i
  VAR_NAME,      // word_lut[value.i]
  VAR_FN         // pointer to seni_node
} seni_var_type;

/*
  NODE_FN,
  NODE_SPECIAL,
  NODE_COLOUR,

  NODE_NULL
*/

typedef struct seni_var {
  seni_var_type type;

  /* no char* in this union since I don't think we're ever going to have a pointer to a string */
  union {
    i32 i;
    f32 f;
    seni_node *p;
  } value;

  /* for hashing */
  i32 id;                    /* key */
  UT_hash_handle hh;         /* makes this structure hashable */

  /* for linked list used by the pool */
  struct seni_var *prev;
  struct seni_var *next;

} seni_var;


typedef struct seni_env {
  struct seni_env *outer;
  seni_var *vars;

  /* for linked list used by the pool */
  struct seni_env *prev;
  struct seni_env *next;
  
} seni_env;


// word lookup
word_lut *word_lookup_allocate();
void word_lookup_free(word_lut *wlut);
i32 word_lookup_or_add(word_lut *wlut, char *string, size_t len);
// char *word_lookup_i32(word_lut *wlut, i32 index);

// parser
seni_node *parser_parse(word_lut *wlut, char *s);
void parser_free_nodes(seni_node *nodes);
char *parser_node_type_name(seni_node_type type);

// env
//int env_debug_available_env();
//int env_debug_available_var();

void env_allocate_pools(void);
void env_free_pools(void);
seni_env *get_initial_env();

seni_env *push_scope(seni_env *env);
seni_env *pop_scope(seni_env *outer);
seni_var *add_var(seni_env *env, i32 var_id);
seni_var *lookup_var(seni_env *env, i32 var_id);

// interpreter
void interpreter_declare_keywords(word_lut *wl);
seni_var *evaluate(seni_env *env, word_lut *wl, seni_node *ast);

#endif
