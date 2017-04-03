#ifndef SENI_LANG_ENV_H
#define SENI_LANG_ENV_H

#include "seni_types.h"
#include "seni_containers.h"
#include "seni_lang_parser.h"

// start at 128 just to make it easier to spot mistakes when transforming seni_node_type -> seni_var_type
typedef enum {
  VAR_INT = 128, // value.i
  VAR_FLOAT,     // value.f
  VAR_BOOLEAN,   // value.i
  VAR_NAME,      // word_lut[value.i]
  VAR_EXPR       // TODO: pointer to seni_node
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

int env_debug_available_env();
int env_debug_available_var();

void env_allocate_pools(void);
void env_free_pools(void);
seni_env *get_initial_env();

seni_env *push_scope(seni_env *env);
seni_env *pop_scope(seni_env *outer);
seni_var *add_var(seni_env *env, i32 var_id);
seni_var *lookup_var(seni_env *env, i32 var_id);

#endif
