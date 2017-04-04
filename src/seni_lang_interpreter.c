#include <stdio.h>
#include "seni_lang_interpreter.h"

seni_var *eval(seni_env *env, seni_node *expr);

typedef struct keyword {
  seni_var *(*function_ptr)(seni_env *, seni_node *);
  char *name;
} keyword;

keyword g_keyword[MAX_KEYWORD_LOOKUPS];

int g_error = 0;
word_lut *g_wl = NULL;
// a register like seni_var for holding intermediate values
seni_var g_reg;


seni_var_type node_type_to_var_type(seni_node_type type)
{
  switch(type) {
  case NODE_INT:
    return VAR_INT;
  case NODE_FLOAT:
    return VAR_FLOAT;
  case NODE_BOOLEAN:
    return VAR_BOOLEAN;
  case NODE_NAME:
    return VAR_NAME;
  default:
    return VAR_INT;
  }
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

void safe_seni_var_copy(seni_var *dest, seni_var *src)
{
  dest->type = src->type;
  if (src->type == VAR_FLOAT) {
    dest->value.f = src->value.f;
  } else if (src->type == VAR_FN) {
    dest->value.p = src->value.p;
  } else {
    dest->value.i = src->value.i;
  }
}

seni_var *eval_keyword_plus(seni_env *env, seni_node *expr)
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

    if (all_ints && v->type == VAR_FLOAT) {
      // first time a non-int has occurred
      all_ints = false;
      fresult = (f32)iresult;
    }

    if (all_ints && v->type == VAR_INT) {
      iresult += v->value.i;
    } else {
      if (v->type == VAR_INT) {
        fresult += (f32)v->value.i;
      } else if (v->type == VAR_FLOAT){
        fresult += v->value.f;
      } else {
        // error: incompatible node type
        return NULL;
      }
    }

    sibling = safe_next(sibling);
  }

  if (all_ints) {
    g_reg.type = VAR_INT;
    g_reg.value.i = iresult;
  } else {
    g_reg.type = VAR_FLOAT;
    g_reg.value.f = fresult;
  }
  
  return &g_reg;
}

seni_var *eval_keyword_minus(seni_env *env, seni_node *expr)
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
      g_reg.type = VAR_INT;
      g_reg.value.i = -iresult;
    } else {
      g_reg.type = VAR_FLOAT;
      g_reg.value.f = -fresult;
    }
    return &g_reg;
  }
  
  seni_var *v;

  while (sibling != NULL) {
    
    v = eval(env, sibling);

    if (all_ints && v->type == VAR_FLOAT) {
      // first time a non-int has occurred
      all_ints = false;
      fresult = (f32)iresult;
    }

    if (all_ints && v->type == VAR_INT) {
      iresult -= v->value.i;
    } else {
      if (v->type == VAR_INT) {
        fresult -= (f32)v->value.i;
      } else if (v->type == VAR_FLOAT){
        fresult -= v->value.f;
      } else {
        // error: incompatible node type
        return NULL;
      }
    }

    sibling = safe_next(sibling);
  }

  if (all_ints) {
    g_reg.type = VAR_INT;
    g_reg.value.i = iresult;
  } else {
    g_reg.type = VAR_FLOAT;
    g_reg.value.f = fresult;
  }

  return &g_reg;
}

seni_var *eval_keyword_multiply(seni_env *env, seni_node *expr)
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

    if (all_ints && v->type == VAR_FLOAT) {
      // first time a non-int has occurred
      all_ints = false;
      fresult = (f32)iresult;
    }

    if (all_ints && v->type == VAR_INT) {
      iresult *= v->value.i;
    } else {
      if (v->type == VAR_INT) {
        fresult *= (f32)v->value.i;
      } else if (v->type == VAR_FLOAT){
        fresult *= v->value.f;
      } else {
        // error: incompatible node type
        return NULL;
      }
    }

    sibling = safe_next(sibling);
  }

  if (all_ints) {
    g_reg.type = VAR_INT;
    g_reg.value.i = iresult;
  } else {
    g_reg.type = VAR_FLOAT;
    g_reg.value.f = fresult;
  }

  return &g_reg;
}

seni_var *eval_keyword_divide(seni_env *env, seni_node *expr)
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

    if (all_ints && v->type == VAR_FLOAT) {
      // first time a non-int has occurred
      all_ints = false;
    }

    if (all_ints && v->type == VAR_INT) {
      iresult /= v->value.i;
      // keep a track of the floating point equivalent in case
      // a later seni_node evals to NODE_FLOAT. We don't want
      // to lose precision by casting the i32 result to f32.
      fresult /= (f32)v->value.i; 
    } else {
      if (v->type == VAR_INT) {
        fresult /= (f32)v->value.i;
      } else if (v->type == VAR_FLOAT){
        fresult /= v->value.f;
      } else {
        // error: incompatible node type
        return NULL;
      }
    }

    sibling = safe_next(sibling);
  }

  if (all_ints) {
    g_reg.type = VAR_INT;
    g_reg.value.i = iresult;
  } else {
    g_reg.type = VAR_FLOAT;
    g_reg.value.f = fresult;
  }

  return &g_reg;
}

seni_var *eval_keyword_define(seni_env *env, seni_node *expr)
{
  // (define num 10)

  // char *a1 = word_lookup_i32(g_wl, expr->value.i);
  
  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    // error: no args given to 'define'
    return NULL;
  }

  // get the binding name
  seni_node *name = sibling;
  // this should be NODE_NAME
  //char *a2 = word_lookup_i32(g_wl, name->value.i);

  // get the value
  sibling = safe_next(sibling);
  seni_var *v = eval(env, sibling);

  // add the name/value binding to the current env
  seni_var *env_var = add_var(env, name->value.i);
  safe_seni_var_copy(env_var, v);

  return env_var;
}

seni_var *eval_keyword_fn(seni_env *env, seni_node *expr)
{
  // (fn (a) 42)
  // (fn (add a: 0 b: 0) (+ a b))
  
  seni_node *fn_keyword = expr;
  //char *fn_keyword_c = word_lookup_i32(g_wl, fn_keyword->value.i); // should be 'fn'
  
  seni_node *def_list = safe_next(fn_keyword);
  if (!def_list || def_list->type != NODE_LIST) {
    // error: no name+parameter list given
    // printf("error: no name+parameter list given\n");
  }

  seni_node *fn_name = def_list->children;
  //char *fn_name_c = word_lookup_i32(g_wl, fn_name->value.i); // the name of the function

  
  // todo: parse the args ???

  seni_var *env_var = add_var(env, fn_name->value.i);
  env_var->type = VAR_FN;
  env_var->value.p = expr;
  
  return env_var;
}

seni_var *eval_fn(seni_env *env, seni_node *expr)
{
  seni_node *name = expr->children;

  // look up the name in the env
  seni_var *var = lookup_var(env, name->value.i);  // var == fn (foo b: 1 c: 2) (+ b c)

  // should be of type VAR_FN
  if (var->type != VAR_FN) {
    // error: eval_fn - function invocation leads to non-fn binding
  }

  seni_env *fn_env = push_scope(env);
  
  seni_node *fn_expr = var->value.p;

  // fn_expr points to the 'fn' keyword

  seni_node *fn_name_and_args_list = safe_next(fn_expr); // (foo b: 1 c: 2)
  
  seni_node *fn_args = safe_next(fn_name_and_args_list->children); // b: 1 c: 2

  // Add the default parameter bindings to the function's locally scoped env
  //
  while (fn_args != NULL) {
    // fn_args points to the binding symbol e.g. b
    seni_node *arg_binding = fn_args;

    // fn_args points to the expr that evaluates to the default value to assign to the binding symbol
    fn_args = safe_next(fn_args);
    seni_var *default_value = eval(fn_env, fn_args);

    // set this parameter's default value
    seni_var *fn_parameter = add_var(fn_env, arg_binding->value.i);
    safe_seni_var_copy(fn_parameter, default_value);

    fn_args = safe_next(fn_args);
  }

  // Add the invoked parameter bindings to the function's locally scoped env
  //
  seni_node *invoke_node = safe_next(name);
  while (invoke_node != NULL) {
    seni_node *arg_binding = invoke_node;

    invoke_node = safe_next(invoke_node);
    seni_var *invoke_parameter_value = eval(env, invoke_node); // note: eval using the original outer scope

    seni_var *invoke_parameter = add_var(fn_env, arg_binding->value.i);
    safe_seni_var_copy(invoke_parameter, invoke_parameter_value);

    invoke_node = safe_next(invoke_node);
  }

  seni_node *fn_body = safe_next(fn_name_and_args_list);
  seni_var *res = NULL;
  
  while (fn_body) {
    res = eval(fn_env, fn_body);
    fn_body = safe_next(fn_body);
  }

  pop_scope(fn_env);
  
  return res;
}

seni_var *eval_list(seni_env *env, seni_node *expr)
{
  seni_var *var = eval(env, expr->children);

  if (var->type == VAR_NAME && (var->value.i & KEYWORD_START)) {
    i32 i = var->value.i - KEYWORD_START;
    return (*g_keyword[i].function_ptr)(env, expr->children);
  }

  // user defined function
  if (var->type == VAR_FN) {
    return eval_fn(env, expr);
  }

  if (!(var->type == VAR_NAME || var->type == VAR_FN)) {
    printf("fuuck - only named functions can be invoked at the moment %d\n", var->type);
    return NULL;
  }

  return NULL;
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
    g_reg.type = node_type_to_var_type(expr->type);
    g_reg.value.i = expr->value.i;
    return &g_reg;
  }
  
  if (expr->type == NODE_FLOAT) {
    g_reg.type = node_type_to_var_type(expr->type);
    g_reg.value.f = expr->value.f;
    return &g_reg;
  }
  
  if (expr->type == NODE_BOOLEAN) {
    g_reg.type = node_type_to_var_type(expr->type);
    g_reg.value.i = expr->value.i;
    return &g_reg;
  }

  if (expr->type == NODE_NAME) {
    if (expr->value.i & KEYWORD_START) {
      g_reg.type = node_type_to_var_type(expr->type);
      g_reg.value.i = expr->value.i;
      return &g_reg;
    }
    v = lookup_var(env, expr->value.i);
    return v;
  }

  if (expr->type == NODE_LABEL || expr->type == NODE_STRING) {
      g_reg.type = node_type_to_var_type(expr->type);
      g_reg.value.i = expr->value.i;
      return &g_reg;
  }

  if (expr->type == NODE_LIST) {
    return eval_list(env, expr);
  }

  return NULL;
}

void string_copy(char **dst, char *src)
{
  size_t len = strlen(src);
  
  char *c = (char *)malloc(sizeof(char) * (len + 1));
  strncpy(c, src, len);
  c[len] = '\0';

  *dst = c;
}

// NOTE: the keyword.name is pointing to memory that's managed by the word_lut
//
void declare_keyword(word_lut *wlut, char *name, seni_var *(*function_ptr)(seni_env *, seni_node *))
{
  string_copy(&(wlut->keywords[wlut->keywords_count]), name);
  g_keyword[wlut->keywords_count].name = wlut->keywords[wlut->keywords_count];
  g_keyword[wlut->keywords_count].function_ptr = function_ptr;
  wlut->keywords_count++;

  if (wlut->keywords_count > MAX_KEYWORD_LOOKUPS) {
    // error
  }
}

void interpreter_declare_keywords(word_lut *wlut)
{
  wlut->keywords_count = 0;

  declare_keyword(wlut, "+", &eval_keyword_plus);
  declare_keyword(wlut, "-", &eval_keyword_minus);
  declare_keyword(wlut, "*", &eval_keyword_multiply);
  declare_keyword(wlut, "/", &eval_keyword_divide);
  declare_keyword(wlut, "define", &eval_keyword_define);
  declare_keyword(wlut, "fn", &eval_keyword_fn);
}
  
seni_var *evaluate(seni_env *env, word_lut *wl, seni_node *ast)
{
  g_wl = wl;
  g_error = 0;

  seni_var *res = NULL;
  for (seni_node *n = ast; n != NULL; n = safe_next(n)) {
    res = eval(env, n);
  }

  return res;
}
