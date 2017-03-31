#include <stdio.h>
#include "seni_lang_interpreter.h"

seni_var *eval(seni_env *env, seni_node *expr);
  
/* the single seni_var allocated before any parsing */
#define SCRATCH_ID 0

int g_error = 0;
word_lookup *g_wl = NULL;
// seni_var g_var;

#define RESERVED_WORD_PLUS (RESERVED_WORD_START + 0)


bool is_reserved_word(seni_var *var)
{
  return (var->type == NODE_NAME && var->value.i >= RESERVED_WORD_START);
}

seni_node *safe_next(seni_node *expr)
{
  seni_node *sibling = expr->next;
  while(sibling && (sibling->type == NODE_WHITESPACE ||
                    sibling->type == NODE_COMMENT)) {
    sibling = sibling->next;
  }

  return sibling;
}

seni_var *eval_reserved_plus(seni_env *env, seni_node *expr)
{
  // expr is the NAME: +
  seni_node *sibling = safe_next(expr);
  i32 sum = 0;
  seni_var *v;

  while (sibling != NULL) {
    
    v = eval(env, sibling);
    if (v->type != NODE_INT) {
      printf("foook - only dealing with ints at the moment\n");
      return NULL;
    }
    sum += v->value.i;
    sibling = safe_next(sibling);
  }

  v = lookup_var(env, SCRATCH_ID);
  v->type = NODE_INT;
  v->value.i = sum;
  return v;
}

seni_var *eval_list(seni_env *env, seni_node *expr)
{
  seni_var *var = eval(env, expr->children);
  if (!is_reserved_word(var)) {
    printf("fuuck - only reserved words are functions at the moment\n");
    return NULL;
  }

  switch(var->value.i) {
  case RESERVED_WORD_PLUS :
    return eval_reserved_plus(env, expr->children);
    break;
  };
  
  return var;
}

seni_var *eval(seni_env *env, seni_node *expr)
{
  seni_var *v = NULL;
  
  if (expr == NULL) {
    // in case of non-existent else clause in if statement
    printf("TODO: can we get here?\n");
    return NULL;
  }

  if (expr->type == NODE_INT) {
    v = lookup_var(env, SCRATCH_ID);
    v->type = expr->type;
    v->value.i = expr->value.i;
    return v;
  }
  
  if (expr->type == NODE_FLOAT) {
    v = lookup_var(env, SCRATCH_ID);
    v->type = expr->type;
    v->value.f = expr->value.f;
    return v;
  }
  
  if (expr->type == NODE_BOOLEAN) {
    v = lookup_var(env, SCRATCH_ID);
    v->type = expr->type;
    v->value.i = expr->value.i;
    return v;
  }

  if (expr->type == NODE_NAME) {
    if (expr->value.i >= RESERVED_WORD_START) {
      v = lookup_var(env, SCRATCH_ID);
      v->type = expr->type;
      v->value.i = expr->value.i;
      return v;
    }
    v = lookup_var(env, expr->value.i);
    return v;
  }

  if (expr->type == NODE_LABEL || expr->type == NODE_STRING) {
      v = lookup_var(env, SCRATCH_ID);
      v->type = expr->type;
      v->value.i = expr->value.i;
      return v;
  }

  if (expr->type == NODE_LIST) {
    return eval_list(env, expr);
  }

  return NULL;
}


  
seni_var *evaluate(seni_env *env, word_lookup *wl, seni_node *ast)
{
  g_wl = wl;
  g_error = 0;

  return eval(env, ast);
}
