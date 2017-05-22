#ifndef SENI_LANG_H
#define SENI_LANG_H

#include "seni_config.h"
#include "seni_types.h"
#include "seni_buffer.h"

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
    struct seni_node *first_child;  /* list node */
  } value;

  bool alterable;

  // node mutate specific
  struct seni_node *parameter_ast;

  // need a place for nodes that occur within curly brackets that should
  // be ignored, e.g. the whitespace before the 2 in: (+ 1 { 2} (int))
  struct seni_node *parameter_prefix;

  /* for parameter_ast, parameter_prefix, first_child */
  struct seni_node *prev;
  struct seni_node *next;
} seni_node;

// start at 128 just to make it easier to spot mistakes when transforming seni_node_type -> seni_var_type
typedef enum {
  VAR_INT = 128, // value.i
  VAR_FLOAT,     // value.f
  VAR_BOOLEAN,   // value.i
  VAR_NAME,      // word_lut[value.i]
  VAR_FN,        // pointer to seni_node: value.n
  VAR_VEC_HEAD,  // pointer to vec_rc is in value.v
  VAR_VEC_RC,    // pointer to first vector element is in value.v
} seni_var_type;

/*
  NODE_FN,
  NODE_SPECIAL,
  NODE_COLOUR,

  NODE_NULL
*/

// which value to use
typedef enum {
  USE_I,                        // integer
  USE_F,                        // float
  USE_N,                        // pointer to seni_node
  USE_V                         // pointer to seni_var
} seni_value_in_use;

typedef struct seni_var {
  i32 id;
  seni_var_type type;

  /* no char* in this union since I don't think we're ever going to have a pointer to a string */
  union {
    i32 i;
    f32 f;
    seni_node *n;
    struct seni_var *v;
  } value;

#ifdef SENI_DEBUG_MODE  
  i32 debug_id;
  bool debug_allocatable; 
#endif

  // reference count for VAR_VEC_RC
  i32 ref_count;
  bool allocated;

  /* for linked list used by the pool and for elements in a vector */
  struct seni_var *prev;
  struct seni_var *next;
} seni_var;


typedef struct seni_env {
  struct seni_env *outer;
  seni_var *vars;

  // every seni_env will have a pointer to the buffer used for rendering vertices
  seni_buffer *buffer;

  /* for linked list used by the pool */
  struct seni_env *prev;
  struct seni_env *next;
  
} seni_env;


typedef struct seni_debug_info {
  i32 num_var_allocated;
  i32 num_env_allocated;

  i32 num_var_available;

  i32 var_get_count;
  i32 var_return_count;
} seni_debug_info;

void fill_debug_info(seni_debug_info *debug_info);

// word lookup
word_lut *wlut_allocate();
void      wlut_free(word_lut *wlut);
i32       wlut_lookup_or_add(word_lut *wlut, char *string, size_t len);
char     *wlut_lookup(word_lut *wlut, i32 index);

// parser
seni_node *parser_parse(word_lut *wlut, char *s);
void       parser_free_nodes(seni_node *nodes);

char      *node_type_name(seni_node *node);
char      *var_type_name(seni_var *var);

// env
//int env_debug_available_env();
//int env_debug_available_var();

void env_allocate_pools(void);
void env_free_pools(void);
seni_env *get_initial_env();

seni_env *push_scope(seni_env *env);
seni_env *pop_scope(seni_env *outer);
seni_var *get_binded_var(seni_env *env, i32 var_id);
seni_var *lookup_var(seni_env *env, i32 var_id);

seni_var *append_to_vector(seni_var *vec, seni_var *val);
// interpreter

// helpers used by bounded functions
seni_var *false_in_reg(seni_var *reg);
seni_var *true_in_reg(seni_var *reg);

i32 var_vector_length(seni_var *var);
  
i32 var_as_int(seni_var *var);
f32 var_as_float(seni_var *var);
void var_as_vec2(f32* out0, f32* out1, seni_var *var);
void var_as_vec4(f32* out0, f32* out1, f32* out2, f32* out3, seni_var *var);

void bool_as_var(seni_var *out, bool b);
void i32_as_var(seni_var *out, i32 i);
void f32_as_var(seni_var *out, f32 f);


seni_var *bind_var(seni_env *env, i32 name, seni_var *var);
seni_var *bind_var_to_int(seni_env *env, i32 name, i32 value);
seni_var *bind_var_to_float(seni_env *env, i32 name, f32 value);
seni_var *eval(seni_env *env, seni_node *expr);
seni_var *eval_all_nodes(seni_env *env, seni_node *body);
seni_node *safe_next(seni_node *expr);
seni_value_in_use get_value_in_use(seni_var_type type);
void safe_var_copy(seni_var *dest, seni_var *src);
void safe_var_move(seni_var *dest, seni_var *src);
void add_named_parameters_to_env(seni_env *env, seni_node *named_params);

// getting value from named parameter lists
bool has_named_node(seni_node *params, i32 name);
seni_node *get_named_node(seni_node *params, i32 name);
seni_var *get_named_var(seni_env *env, seni_node *params, i32 name);
f32 get_named_f32(seni_env *env, seni_node *params, i32 name, f32 default_value);
i32 get_named_i32(seni_env *env, seni_node *params, i32 name, i32 default_value);
void get_named_vec2(seni_env *env, seni_node *params, i32 name, f32 *out0, f32 *out1);
void get_named_vec4(seni_env *env, seni_node *params, i32 name, f32 *out0, f32 *out1, f32 *out2, f32 *out3);

void declare_keyword(word_lut *wlut, char *name, seni_var *(*function_ptr)(seni_env *, seni_node *));
void declare_common_arg(word_lut *wlut, char *name, i32 *global_value);
seni_var *evaluate(seni_env *env, seni_node *ast, bool hygenic_scope);

// debugging
void debug_var_info(seni_env *env);
void debug_reset();
void pretty_print_seni_var(seni_var *var, char* msg);
void pretty_print_seni_node(seni_node *node, char* msg);
  
#endif
