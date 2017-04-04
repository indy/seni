#include "seni_lang_env.h"

/* 
   TODO: add debug function to count the number of available envs/vars
 */

#define G_ENVS_MAX 32
seni_env *g_envs;               /* doubly linked list used as a pool of seni_env structs */
i32 g_envs_used;

#define G_VARS_MAX 64
seni_var *g_vars;               /* doubly linked list used as a pool of seni_var structs */
i32 g_vars_used;


int env_debug_available_env()
{
  seni_env *seni_env;
  int count = 0;

  DL_COUNT(g_envs, seni_env, count);

  return count;
}

int env_debug_available_var()
{
  seni_var *seni_var;
  int count = 0;

  DL_COUNT(g_vars, seni_var, count);

  return count;
}

/* called once at startup */
void env_allocate_pools(void)
{
  i32 i;
  
  g_envs = NULL;
  g_envs_used = 0;
  seni_env *env;

  for (i = 0; i < G_ENVS_MAX; i++) {
    env = (seni_env *)malloc(sizeof(seni_env));
    env->outer = NULL;
    env->vars = NULL;
    env->prev = NULL;
    env->next = NULL;
    DL_APPEND(g_envs, env);
  }

  g_vars = NULL;
  g_vars_used = 0;
  seni_var *var;

  for (i = 0; i < G_VARS_MAX; i++) {
    var = (seni_var *)malloc(sizeof(seni_var));
    DL_APPEND(g_vars, var);
  }
}

/* called once at shutdown */
void env_free_pools(void)
{
  seni_env *env, *tmp;
  DL_FOREACH_SAFE(g_envs, env, tmp) {
    DL_DELETE(g_envs, env);
    free(env);
  }  

  seni_var *var, *tmp2;
  DL_FOREACH_SAFE(g_vars, var, tmp2) {
    DL_DELETE(g_vars, var);
    free(var);
  }  
}

seni_env *get_env_from_pool()
{
  seni_env *head = g_envs;

  if (head != NULL) {
    DL_DELETE(g_envs, head);

    head->outer = NULL;
    head->vars = NULL;
    head->prev = NULL;
    head->next = NULL;
  }
  
  return head;
}

void return_env_to_pool(seni_env *env)
{
  DL_APPEND(g_envs, env);
}

seni_var *get_var_from_pool()
{
  seni_var *head = g_vars;

  if (head != NULL) {
    DL_DELETE(g_vars, head);
  }
  
  return head;
}

void return_var_to_pool(seni_var *var)
{
  DL_APPEND(g_vars, var);
}

seni_env *get_initial_env()
{
  return get_env_from_pool();
}

seni_env *push_scope(seni_env *outer)
{
  seni_env *env = get_env_from_pool();
  if (env == NULL) {
    // error
    return NULL;
  }

  env->outer = outer;
  env->vars = NULL;
  
  return env;
}

seni_env *pop_scope(seni_env *env)
{
  seni_env *outer_env = env->outer;

  // return all the vars back to the pool
  seni_var *var, *tmp;

  HASH_ITER(hh, env->vars, var, tmp) {
    HASH_DEL(env->vars, var);
    return_var_to_pool(var);
  }  

  return_env_to_pool(env);
  
  return outer_env;
}

/* adds a var to the env */
seni_var *add_var(seni_env *env, i32 var_id)
{
  seni_var *var = NULL;

  // is the key already assigned to this env?
  HASH_FIND_INT(env->vars, &var_id, var);
  if (var != NULL) {
    // return the existing var so that it can be overwritten
    return var;
  }

  var = get_var_from_pool();
  if (var == NULL) {
    // error
    return NULL;
  }
  
  var->id = var_id;
  HASH_ADD_INT(env->vars, id, var);

  return var;
}

seni_var *lookup_var(seni_env *env, i32 var_id)
{
  if (env == NULL) {
    return NULL;
  }

  seni_var *var = NULL;

  HASH_FIND_INT(env->vars, &var_id, var);
  if (var != NULL) {
    // return the existing var
    return var;
  }

  return lookup_var(env->outer, var_id);
}
