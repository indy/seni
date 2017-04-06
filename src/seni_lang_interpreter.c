#include <stdio.h>
#include <math.h>
#include "seni_lang_interpreter.h"


#define SENI_ERROR(MSG) printf("%s\n", MSG)

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

i32 g_arg_from = 0;
i32 g_arg_to = 0;
i32 g_arg_increment = 0;
i32 g_arg_upto = 0;
i32 g_arg_steps = 0;

// which value to use
typedef enum seni_value_in_use {
  USE_I,
  USE_F,
  USE_P
} seni_value_in_use;

seni_value_in_use get_value_in_use(seni_var_type type) {
  if (type == VAR_FLOAT) {
    return USE_F;
  } else if (type == VAR_FN) {
    return USE_P;
  } else {
    return USE_I;
  }
}

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

  seni_value_in_use using = get_value_in_use(src->type);
  
  if (using == USE_I) {
    dest->value.i = src->value.i;
  } else if (using == USE_F) {
    dest->value.f = src->value.f;
  } else if (using == USE_P) {
    dest->value.p = src->value.p;
  }
}

seni_var *true_in_g_reg()
{
  g_reg.type = VAR_BOOLEAN;
  g_reg.value.i = 1;
  return &g_reg;
}

seni_var *false_in_g_reg()
{
  g_reg.type = VAR_BOOLEAN;
  g_reg.value.i = 0;
  return &g_reg;
}

i32 var_as_int(seni_var *v1)
{
  seni_value_in_use using = get_value_in_use(v1->type);

  if (using == USE_I) {
    return v1->value.i;
  } else if (using == USE_F) {
    return (i32)(v1->value.f);
  }

  return -1;
}

f32 var_as_float(seni_var *v1)
{
  seni_value_in_use using = get_value_in_use(v1->type);

  if (using == USE_I) {
    return (f32)(v1->value.i);
  } else if (using == USE_F) {
    return v1->value.f;
  }

  return -1.0f;
}

void bind_var_to_int(seni_env *env, i32 name, i32 value)
{
  seni_var *sv = add_var(env, name);
  sv->type = VAR_INT;
  sv->value.i = value;
}

void bind_var_to_float(seni_env *env, i32 name, f32 value)
{
  seni_var *sv = add_var(env, name);
  sv->type = VAR_FLOAT;
  sv->value.f = value;
}


seni_var *eval_classic_fn_plus(seni_env *env, seni_node *expr)
{
  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    SENI_ERROR("no args given to '+'");
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
      fresult = (f32)iresult + v->value.f;
    } else if (all_ints && v->type == VAR_INT) {
      iresult += v->value.i;
    } else {
      float ff = var_as_float(v);
      fresult += ff;
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

seni_var *eval_all_nodes(seni_env *env, seni_node *body)
{
  seni_var *res = NULL;

  while (body) {
    res = eval(env, body);
    body = safe_next(body);
  }

  return res;
}

seni_var *eval_classic_fn_minus(seni_env *env, seni_node *expr)
{
  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    // error: no args given to '-'
    return NULL;
  }

  seni_var *v = eval(env, sibling);
  
  i32 iresult = v->value.i;
  f32 fresult = v->value.f;

  bool all_ints = v->type == VAR_INT;
  
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
      float ff = var_as_float(v);
      fresult -= ff;
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

seni_var *eval_classic_fn_multiply(seni_env *env, seni_node *expr)
{
  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    // error: no args given to '*'
    return NULL;
  }

  seni_var *v = eval(env, sibling);

  i32 iresult = v->value.i;
  f32 fresult = v->value.f;

  bool all_ints = v->type == VAR_INT;
  
  sibling = safe_next(sibling);
  
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
      f32 ff = var_as_float(v);
      fresult *= ff;
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

seni_var *eval_classic_fn_divide(seni_env *env, seni_node *expr)
{
  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    // error: no args given to '/'
    return NULL;
  }
  seni_var *v = eval(env, sibling);
  
  bool all_ints = v->type == VAR_INT;

  i32 iresult = v->value.i;
  f32 fresult = all_ints ? (float)(v->value.i) : v->value.f;
  
  sibling = safe_next(sibling);

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

seni_var *eval_classic_fn_equality(seni_env *env, seni_node *expr)
{
  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    // error: no args given to '='
    return NULL;
  }

  seni_var *v = eval(env, sibling);

  bool using_i;
  i32 i = 0;
  f32 f = 0.0f;

  seni_value_in_use using = get_value_in_use(v->type);
  if (using == USE_I) {
    using_i = true;
    i = v->value.i;
  } else if (using == USE_F) {
    using_i = false;
    f = v->value.f;
  } else {
    // error ()
    return NULL;
  } 
  
  sibling = safe_next(sibling);

  while (sibling != NULL) {
    
    v = eval(env, sibling);

    seni_value_in_use using2 = get_value_in_use(v->type);
    if (using2 != using) {
      return false_in_g_reg();
    }

    if (using_i) {
      if (i != v->value.i) {
        return false_in_g_reg();
      }
    } else {
      if (f != v->value.f) {
        return false_in_g_reg();
      }
    }

    sibling = safe_next(sibling);
  }

  return true_in_g_reg();
}

seni_var *eval_classic_fn_greater(seni_env *env, seni_node *expr)
{
  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    // error: no args given to '>'
    return NULL;
  }

  seni_var *v = eval(env, sibling);

  bool prev_using_i;
  i32 prev_i = 0;
  f32 prev_f = 0.0f;

  seni_value_in_use using = get_value_in_use(v->type);
  if (using == USE_I) {
    prev_using_i = true;
    prev_i = v->value.i;
  } else if (using == USE_F) {
    prev_using_i = false;
    prev_f = v->value.f;
  } else {
    // error ()
    return NULL;
  }

  sibling = safe_next(sibling);

  while (sibling != NULL) {
    
    v = eval(env, sibling);

    seni_value_in_use using2 = get_value_in_use(v->type);
    if (using2 == USE_I) {
      if (prev_using_i) {
        if (prev_i <= v->value.i) {
          return false_in_g_reg();
        }
      } else {
        if (prev_f <= (f32)v->value.i) {
          return false_in_g_reg();
        }
      }

      prev_using_i = true;
      prev_i = v->value.i;

    } else if (using2 == USE_F) {
      if (prev_using_i) {
        if ((f32)prev_i <= v->value.f) {
          return false_in_g_reg();
        }
      } else {
        if (prev_f <= v->value.f) {
          return false_in_g_reg();
        }
      }

      prev_using_i = false;
      prev_f = v->value.f;
      
    } else {
      // error()
      return NULL;
    }

    sibling = safe_next(sibling);
  }

  return true_in_g_reg();
}

seni_var *eval_classic_fn_lesser(seni_env *env, seni_node *expr)
{
  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    // error: no args given to '<'
    return NULL;
  }

  seni_var *v = eval(env, sibling);

  bool prev_using_i;
  i32 prev_i = 0;
  f32 prev_f = 0.0f;

  seni_value_in_use using = get_value_in_use(v->type);
  if (using == USE_I) {
    prev_using_i = true;
    prev_i = v->value.i;
  } else if (using == USE_F) {
    prev_using_i = false;
    prev_f = v->value.f;
  } else {
    // error ()
    return NULL;
  }

  sibling = safe_next(sibling);

  while (sibling != NULL) {
    
    v = eval(env, sibling);

    seni_value_in_use using2 = get_value_in_use(v->type);
    if (using2 == USE_I) {
      if (prev_using_i) {
        if (prev_i >= v->value.i) {
          return false_in_g_reg();
        }
      } else {
        if (prev_f >= (f32)v->value.i) {
          return false_in_g_reg();
        }
      }

      prev_using_i = true;
      prev_i = v->value.i;

    } else if (using2 == USE_F) {
      if (prev_using_i) {
        if ((f32)prev_i >= v->value.f) {
          return false_in_g_reg();
        }
      } else {
        if (prev_f >= v->value.f) {
          return false_in_g_reg();
        }
      }

      prev_using_i = false;
      prev_f = v->value.f;
      
    } else {
      // error()
      return NULL;
    }

    sibling = safe_next(sibling);
  }

  return true_in_g_reg();
}

seni_var *eval_classic_fn_vector(seni_env *env, seni_node *expr)
{
  printf("IMPLEMENT eval_classic_fn_vector\n");
  return eval_classic_fn_lesser(env, expr);
}

seni_var *eval_classic_fn_vector_append(seni_env *env, seni_node *expr)
{
  printf("IMPLEMENT eval_classic_fn_vector_append\n");
  return eval_classic_fn_lesser(env, expr);
}

seni_var *eval_classic_fn_sqrt(seni_env *env, seni_node *expr)
{
  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    // error: no args given to 'sqrt'
    return NULL;
  }

  seni_var *v = eval(env, sibling);
  f32 inp = var_as_float(v);

  g_reg.type = VAR_FLOAT;
  g_reg.value.f = (f32)sqrt(inp);

  return &g_reg;
}

seni_var *eval_classic_fn_mod(seni_env *env, seni_node *expr)
{
  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    // error: no args given to 'sqrt'
    return NULL;
  }

  seni_var *v;

  v = eval(env, sibling);
  i32 i1 = var_as_int(v);

  sibling = safe_next(sibling);
  
  v = eval(env, sibling);
  i32 i2 = var_as_int(v);

  g_reg.type = VAR_INT;
  g_reg.value.i = i1 % i2;

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

  seni_node *fn_name = def_list->value.children;
  //char *fn_name_c = word_lookup_i32(g_wl, fn_name->value.i); // the name of the function

  
  // todo: parse the args ???

  seni_var *env_var = add_var(env, fn_name->value.i);
  env_var->type = VAR_FN;
  env_var->value.p = expr;
  
  return env_var;
}

seni_var *eval_keyword_if(seni_env *env, seni_node *expr)
{
  // if (test) then else

  seni_node *test = safe_next(expr);
  if (test == NULL) {
    return NULL;
  }

  seni_var *test_var = eval(env, test);
  if (test_var->type != VAR_BOOLEAN) {
    // error: if's test condition should evaluate to a boolean
    return NULL;
  }

  bool truthy = test_var->value.i == 1;

  seni_node *truthy_node = safe_next(test);
  seni_node *falsey_node = safe_next(truthy_node);

  if (truthy) {
    return eval(env, truthy_node);
  } else if (falsey_node) {
    return eval(env, falsey_node);
  }

  return NULL;
}

// a: 3 b: 10 c: (+ 5 6)
void add_labelled_parameters_to_env(seni_env *env, seni_node *named_args)
{
  while (named_args) {
    seni_node *name = named_args;
    if (name->type != NODE_LABEL) {
      // error: expected a label as the start of the name/value pair
      return;
    }
    i32 name_i = name->value.i;

    named_args = safe_next(named_args);
    if (named_args == NULL) {
      // error expected name value pairs
      return;
    }
    
    seni_node *value = named_args;
    seni_var *var = eval(env, value);
    seni_var *env_var = add_var(env, name_i);

    safe_seni_var_copy(env_var, var);

    named_args = safe_next(named_args);
  }
}

// does the sequence of named args contain the given name
bool has_labelled_parameter(seni_node *named_args, i32 name)
{
  // assuming we're at the first named parameter
  seni_node *node = named_args;
  
  while(node) {
    if (node->type != NODE_LABEL) {
      // error should never get here, all name/value parameters should start with a NODE_NAME
      return false;
    }

    if (node->value.i == name) {
      return true;
    }

    node = safe_next(node);     // node is at the value
    if (node == NULL) {
      // error should never get here, there should always be pairs of name/value
      return false;
    }

    node = safe_next(node);     // node is at the next named parameter
  }

  return false;
}

seni_var *eval_keyword_loop(seni_env *env, seni_node *expr)
{
  // (loop (x from: 1 to: 4) x)

  seni_var *res = NULL;

  seni_node *setup = safe_next(expr);
  if (setup->type != NODE_LIST) {
    // error loop requires a list describing it's behaviour
    return NULL;
  }

  seni_node *body = safe_next(setup);
  seni_node *var_node = setup->value.children;

  i32 var_index = var_node->value.i;

  seni_env *loop_env = push_scope(env);
  {
    seni_node *args = safe_next(var_node);

    add_labelled_parameters_to_env(loop_env, args);

    i32 increment = 1;
    i32 steps = 1;
    seni_var *from_var = NULL;
    seni_var *to_var = NULL;
    seni_var *upto_var = NULL;
    
    bool has_from = has_labelled_parameter(args, g_arg_from);
    if (has_from) {
      from_var = lookup_var(loop_env, g_arg_from);
    }

    bool has_to = has_labelled_parameter(args, g_arg_to);
    if (has_to) {
      to_var = lookup_var(loop_env, g_arg_to);
    }

    bool has_upto = has_labelled_parameter(args, g_arg_upto);
    if (has_upto) {
      upto_var = lookup_var(loop_env, g_arg_upto);
    }

    bool has_increment = has_labelled_parameter(args, g_arg_increment);
    if (has_increment) {
      seni_var *increment_var = lookup_var(loop_env, g_arg_increment);
      increment = var_as_int(increment_var);
      if (increment == 0) {
        // error: cannot have an increment of 0
        pop_scope(loop_env);
        return NULL;
      }
    }

    bool has_steps = has_labelled_parameter(args, g_arg_steps);
    if (has_steps) {
      seni_var *steps_var = lookup_var(loop_env, g_arg_steps);
      steps = var_as_int(steps_var);
      if (steps == 0) {
        // error: cannot have steps value of 0
        pop_scope(loop_env);
        return NULL;
      }
    }

    if(has_increment || !has_steps) {
      i32 from = has_from ? var_as_int(from_var) : 0;
      i32 to = has_to ? var_as_int(to_var) : 10;
      i32 upto = has_upto ? var_as_int(upto_var) : 10;

      // the default path - perform an integer based iteration
      i32 i;
      if (has_to || !has_upto) {

        if (from > to) {
          // error: from has to be less than to
          pop_scope(loop_env);
          return NULL;
        }
    
        
        for (i = from; i < to; i += increment) {
          bind_var_to_int(loop_env, var_index, i);
          res = eval_all_nodes(loop_env, body);
        }
      
      } else if (has_upto) {
        for (i = from; i <= upto; i += increment) {
          bind_var_to_int(loop_env, var_index, i);
          res = eval_all_nodes(loop_env, body);
        }
      }
      
    } else {
      // use steps and a float based iteration
      f32 from = has_from ? var_as_float(from_var) : 0.0f;
      f32 to = has_to ? var_as_float(to_var) : 10.0f;
      f32 upto = has_upto ? var_as_float(upto_var) : 10.0f;

      f32 f;
      if (has_to || !has_upto) {

        f32 step_size = (to - from) / (f32)steps;
        i32 i;
        
        for (i = 0; i < steps; i++) {
          f = from + ((f32)i * step_size);
          bind_var_to_float(loop_env, var_index, f);
          res = eval_all_nodes(loop_env, body);
        }
      
      } else if (has_upto) {

        f32 step_size = (upto - from) / (f32)(steps - 1);
        i32 i;
        
        for (i = 0; i < steps; i++) {
          f = from + ((f32)i * step_size);
          bind_var_to_float(loop_env, var_index, f);
          res = eval_all_nodes(loop_env, body);
        }
      }
    }
  }
  pop_scope(loop_env);
    
  return res;
}


/* used for debugging only */
seni_var *eval_keyword_setq(seni_env *env, seni_node *expr)
{
  // (setq foo 4)
  seni_node *name_node = safe_next(expr);

  seni_node *value_node = safe_next(name_node);
  seni_var *value_var = eval(env, value_node);

  if (name_node->type != NODE_NAME) {
    printf("error: setq expects a name as the first arg\n");
    return NULL;
  }

  // call lookup_var since we're overwriting the existing value
  seni_var *variable = lookup_var(env, name_node->value.i);
  safe_seni_var_copy(variable, value_var);
  return variable;
}

seni_var *eval_fn(seni_env *env, seni_node *expr)
{
  seni_node *name = expr->value.children;

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
  
  seni_node *fn_args = safe_next(fn_name_and_args_list->value.children); // b: 1 c: 2

  add_labelled_parameters_to_env(fn_env, fn_args);

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

  seni_var *res = eval_all_nodes(fn_env, fn_body);

  pop_scope(fn_env);

  return res;
}

seni_var *eval_list(seni_env *env, seni_node *expr)
{
  seni_var *var = eval(env, expr->value.children);

  if (var->type == VAR_NAME && (var->value.i & KEYWORD_START)) {
    i32 i = var->value.i - KEYWORD_START;
    return (*g_keyword[i].function_ptr)(env, expr->value.children);
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

void declare_common_arg(word_lut *wlut, char *name, i32 *global_value)
{
  declare_keyword(wlut, name, NULL);
  i32 i = word_lookup_or_add(wlut, name, strlen(name));
  *global_value = i;
}

void interpreter_declare_keywords(word_lut *wlut)
{
  wlut->keywords_count = 0;

  // classic functions that don't use named arguments when invoked
  declare_keyword(wlut, "+", &eval_classic_fn_plus);
  declare_keyword(wlut, "-", &eval_classic_fn_minus);
  declare_keyword(wlut, "*", &eval_classic_fn_multiply);
  declare_keyword(wlut, "/", &eval_classic_fn_divide);
  declare_keyword(wlut, "=", &eval_classic_fn_equality);
  declare_keyword(wlut, ">", &eval_classic_fn_greater);
  declare_keyword(wlut, "<", &eval_classic_fn_lesser);
  declare_keyword(wlut, "vector", &eval_classic_fn_vector);
  declare_keyword(wlut, "vector/append", &eval_classic_fn_vector_append);
  declare_keyword(wlut, "sqrt", &eval_classic_fn_sqrt);
  declare_keyword(wlut, "mod", &eval_classic_fn_mod);

  // special functions with non-standard syntax
  declare_keyword(wlut, "define", &eval_keyword_define);
  declare_keyword(wlut, "fn", &eval_keyword_fn);
  declare_keyword(wlut, "if", &eval_keyword_if);
  declare_keyword(wlut, "loop", &eval_keyword_loop);

  declare_keyword(wlut, "setq", &eval_keyword_setq);

  declare_common_arg(wlut, "from", &g_arg_from);
  declare_common_arg(wlut, "to", &g_arg_to);
  declare_common_arg(wlut, "increment", &g_arg_increment);
  declare_common_arg(wlut, "upto", &g_arg_upto);
  declare_common_arg(wlut, "steps", &g_arg_steps);
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
