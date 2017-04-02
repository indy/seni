#include <stdio.h>
#include "seni_lang_interpreter.h"

seni_var *eval(seni_env *env, seni_node *expr);
  
int g_error = 0;
word_lookup *g_wl = NULL;
// a register like seni_var for holding intermediate values
seni_var g_reg;

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
  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    // error: no args given to '+'
    return NULL;
  }

  i32 iresult = 0;
  f32 fresult = 0.0f;
  bool all_ints = true;
  
  seni_var *v;

  while (sibling != NULL) {
    
    v = eval(env, sibling);

    if (all_ints && v->type == NODE_FLOAT) {
      // first time a non-int has occurred
      all_ints = false;
      fresult = (f32)iresult;
    }

    if (all_ints && v->type == NODE_INT) {
      iresult += v->value.i;
    } else {
      if (v->type == NODE_INT) {
        fresult += (f32)v->value.i;
      } else if (v->type == NODE_FLOAT){
        fresult += v->value.f;
      } else {
        // error: incompatible node type
        return NULL;
      }
    }

    sibling = safe_next(sibling);
  }

  if (all_ints) {
    g_reg.type = NODE_INT;
    g_reg.value.i = iresult;
  } else {
    g_reg.type = NODE_FLOAT;
    g_reg.value.f = fresult;
  }
  
  return &g_reg;
}

seni_var *eval_reserved_minus(seni_env *env, seni_node *expr)
{
  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    // error: no args given to '-'
    return NULL;
  }
  
  i32 iresult = sibling->value.i;
  f32 fresult = sibling->value.f;

  bool all_ints = sibling->type == NODE_INT;
  
  sibling = safe_next(sibling);

  if (sibling == NULL) {
    // only 1 arg e.g. (- 42)
    // so negate it
    if (all_ints) {
      g_reg.type = NODE_INT;
      g_reg.value.i = -iresult;
    } else {
      g_reg.type = NODE_FLOAT;
      g_reg.value.f = -fresult;
    }
    return &g_reg;
  }
  
  seni_var *v;

  while (sibling != NULL) {
    
    v = eval(env, sibling);

    if (all_ints && v->type == NODE_FLOAT) {
      // first time a non-int has occurred
      all_ints = false;
      fresult = (f32)iresult;
    }

    if (all_ints && v->type == NODE_INT) {
      iresult -= v->value.i;
    } else {
      if (v->type == NODE_INT) {
        fresult -= (f32)v->value.i;
      } else if (v->type == NODE_FLOAT){
        fresult -= v->value.f;
      } else {
        // error: incompatible node type
        return NULL;
      }
    }

    sibling = safe_next(sibling);
  }

  if (all_ints) {
    g_reg.type = NODE_INT;
    g_reg.value.i = iresult;
  } else {
    g_reg.type = NODE_FLOAT;
    g_reg.value.f = fresult;
  }

  return &g_reg;
}

seni_var *eval_reserved_multiply(seni_env *env, seni_node *expr)
{
  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    // error: no args given to '*'
    return NULL;
  }

  i32 iresult = sibling->value.i;
  f32 fresult = sibling->value.f;

  bool all_ints = sibling->type == NODE_INT;
  
  sibling = safe_next(sibling);
  
  seni_var *v;

  while (sibling != NULL) {
    
    v = eval(env, sibling);

    if (all_ints && v->type == NODE_FLOAT) {
      // first time a non-int has occurred
      all_ints = false;
      fresult = (f32)iresult;
    }

    if (all_ints && v->type == NODE_INT) {
      iresult *= v->value.i;
    } else {
      if (v->type == NODE_INT) {
        fresult *= (f32)v->value.i;
      } else if (v->type == NODE_FLOAT){
        fresult *= v->value.f;
      } else {
        // error: incompatible node type
        return NULL;
      }
    }

    sibling = safe_next(sibling);
  }

  if (all_ints) {
    g_reg.type = NODE_INT;
    g_reg.value.i = iresult;
  } else {
    g_reg.type = NODE_FLOAT;
    g_reg.value.f = fresult;
  }

  return &g_reg;
}

seni_var *eval_reserved_divide(seni_env *env, seni_node *expr)
{
  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    // error: no args given to '/'
    return NULL;
  }

  bool all_ints = sibling->type == NODE_INT;

  i32 iresult = sibling->value.i;
  f32 fresult = all_ints ? (float)(sibling->value.i) : sibling->value.f;
  
  sibling = safe_next(sibling);
  
  seni_var *v;

  while (sibling != NULL) {
    
    v = eval(env, sibling);

    if (all_ints && v->type == NODE_FLOAT) {
      // first time a non-int has occurred
      all_ints = false;
    }

    if (all_ints && v->type == NODE_INT) {
      iresult /= v->value.i;
      // keep a track of the floating point equivalent in case
      // a later seni_node evals to NODE_FLOAT. We don't want
      // to lose precision by casting the i32 result to f32.
      fresult /= (f32)v->value.i; 
    } else {
      if (v->type == NODE_INT) {
        fresult /= (f32)v->value.i;
      } else if (v->type == NODE_FLOAT){
        fresult /= v->value.f;
      } else {
        // error: incompatible node type
        return NULL;
      }
    }

    sibling = safe_next(sibling);
  }

  if (all_ints) {
    g_reg.type = NODE_INT;
    g_reg.value.i = iresult;
  } else {
    g_reg.type = NODE_FLOAT;
    g_reg.value.f = fresult;
  }

  return &g_reg;
}


seni_var *eval_list(seni_env *env, seni_node *expr)
{
  seni_var *var = eval(env, expr->children);
  if (!is_reserved_word(var)) {
    printf("fuuck - only reserved words are functions at the moment\n");
    return NULL;
  }

  if (var->value.i >= RESERVED_WORD_START) {
    switch(var->value.i) {
    case RESERVED_WORD_PLUS:
      return eval_reserved_plus(env, expr->children);
    case RESERVED_WORD_MINUS:
      return eval_reserved_minus(env, expr->children);
    case RESERVED_WORD_MULTIPLY:
      return eval_reserved_multiply(env, expr->children);
    case RESERVED_WORD_DIVIDE:
      return eval_reserved_divide(env, expr->children);
    };
  }
  
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
    g_reg.type = expr->type;
    g_reg.value.i = expr->value.i;
    return &g_reg;
  }
  
  if (expr->type == NODE_FLOAT) {
    g_reg.type = expr->type;
    g_reg.value.f = expr->value.f;
    return &g_reg;
  }
  
  if (expr->type == NODE_BOOLEAN) {
    g_reg.type = expr->type;
    g_reg.value.i = expr->value.i;
    return &g_reg;
  }

  if (expr->type == NODE_NAME) {
    if (expr->value.i >= RESERVED_WORD_START) {
      g_reg.type = expr->type;
      g_reg.value.i = expr->value.i;
      return &g_reg;
    }
    v = lookup_var(env, expr->value.i);
    return v;
  }

  if (expr->type == NODE_LABEL || expr->type == NODE_STRING) {
      g_reg.type = expr->type;
      g_reg.value.i = expr->value.i;
      return &g_reg;
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
