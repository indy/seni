#include "seni_bind_core.h"
#include "seni_bind.h"

#include "seni_vm.h"

#include <stdio.h>
#include <math.h>

seni_var *var_as_int_or_float(bool is_int, i32 i, f32 f)
{
  if (is_int) {
    g_reg.type = VAR_INT;
    g_reg.value.i = i;
  } else {
    g_reg.type = VAR_FLOAT;
    g_reg.value.f = f;
  }

  return &g_reg;
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

  return var_as_int_or_float(all_ints, iresult, fresult);
}

seni_var *eval_classic_fn_minus(seni_env *env, seni_node *expr)
{
  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    SENI_ERROR("no args given to '-'");
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
    return var_as_int_or_float(all_ints, -iresult, -fresult);
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

  return var_as_int_or_float(all_ints, iresult, fresult);
}

seni_var *eval_classic_fn_multiply(seni_env *env, seni_node *expr)
{
  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    SENI_ERROR("no args given to '*'");
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

  return var_as_int_or_float(all_ints, iresult, fresult);
}

seni_var *eval_classic_fn_divide(seni_env *env, seni_node *expr)
{
  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    SENI_ERROR("no args given to '/'");
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
        SENI_ERROR("incompatible node type");
        return NULL;
      }
    }

    sibling = safe_next(sibling);
  }

  return var_as_int_or_float(all_ints, iresult, fresult);
}

seni_var *eval_classic_fn_equality(seni_env *env, seni_node *expr)
{
  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    SENI_ERROR("no args given to '='");
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
    SENI_ERROR("()");
    return NULL;
  } 
  
  sibling = safe_next(sibling);

  while (sibling != NULL) {
    
    v = eval(env, sibling);

    seni_value_in_use using2 = get_value_in_use(v->type);
    if (using2 != using) {
      return false_in_reg(&g_reg);
    }

    if (using_i) {
      if (i != v->value.i) {
        return false_in_reg(&g_reg);
      }
    } else {
      if (f != v->value.f) {
        return false_in_reg(&g_reg);
      }
    }

    sibling = safe_next(sibling);
  }

  return true_in_reg(&g_reg);
}

seni_var *eval_classic_fn_greater(seni_env *env, seni_node *expr)
{
  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    SENI_ERROR("no args given to '>'");
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
    SENI_ERROR("()");
    return NULL;
  }

  sibling = safe_next(sibling);

  while (sibling != NULL) {
    
    v = eval(env, sibling);

    seni_value_in_use using2 = get_value_in_use(v->type);
    if (using2 == USE_I) {
      if (prev_using_i) {
        if (prev_i <= v->value.i) {
          return false_in_reg(&g_reg);
        }
      } else {
        if (prev_f <= (f32)v->value.i) {
          return false_in_reg(&g_reg);
        }
      }

      prev_using_i = true;
      prev_i = v->value.i;

    } else if (using2 == USE_F) {
      if (prev_using_i) {
        if ((f32)prev_i <= v->value.f) {
          return false_in_reg(&g_reg);
        }
      } else {
        if (prev_f <= v->value.f) {
          return false_in_reg(&g_reg);
        }
      }

      prev_using_i = false;
      prev_f = v->value.f;
      
    } else {
      SENI_ERROR("()");
      return NULL;
    }

    sibling = safe_next(sibling);
  }

  return true_in_reg(&g_reg);
}

seni_var *eval_classic_fn_lesser(seni_env *env, seni_node *expr)
{
  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    SENI_ERROR("no args given to '<'");
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
    SENI_ERROR("()");
    return NULL;
  }

  sibling = safe_next(sibling);

  while (sibling != NULL) {
    
    v = eval(env, sibling);

    seni_value_in_use using2 = get_value_in_use(v->type);
    if (using2 == USE_I) {
      if (prev_using_i) {
        if (prev_i >= v->value.i) {
          return false_in_reg(&g_reg);
        }
      } else {
        if (prev_f >= (f32)v->value.i) {
          return false_in_reg(&g_reg);
        }
      }

      prev_using_i = true;
      prev_i = v->value.i;

    } else if (using2 == USE_F) {
      if (prev_using_i) {
        if ((f32)prev_i >= v->value.f) {
          return false_in_reg(&g_reg);
        }
      } else {
        if (prev_f >= v->value.f) {
          return false_in_reg(&g_reg);
        }
      }

      prev_using_i = false;
      prev_f = v->value.f;
      
    } else {
      SENI_ERROR("()");
      return NULL;
    }

    sibling = safe_next(sibling);
  }

  return true_in_reg(&g_reg);
}



seni_var *eval_classic_fn_vector_append(seni_env *env, seni_node *expr)
{
  // (define v [1])(vector/append v 3)

  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    SENI_ERROR("no args given to 'vector/append'");
    return NULL;
  }

  seni_var *vec_head = eval(env, sibling);
  if (vec_head->type != VAR_VEC_HEAD) {
    SENI_ERROR("first argument of vector/append should be a vector");
    return NULL;
  }

  sibling = safe_next(sibling);
  seni_var *val = eval(env, sibling);

  // add val to the end of vec
  vec_head = append_to_vector(vec_head, val);
  if (vec_head == NULL) {
    return NULL;
  }

  return vec_head;
}

seni_var *eval_classic_fn_sqrt(seni_env *env, seni_node *expr)
{
  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    SENI_ERROR("no args given to 'sqrt'");
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
    SENI_ERROR("no args given to 'sqrt'");
    return NULL;
  }

  seni_var *v;

  v = eval(env, sibling);
  i32 i1 = var_as_int(v);

  sibling = safe_next(sibling);
  
  v = eval(env, sibling);
  i32 i2 = var_as_int(v);

  g_reg.type = VAR_FLOAT;
  g_reg.value.f = (float)(i1 % i2);

  return &g_reg;
}

seni_var *eval_keyword_define(seni_env *env, seni_node *expr)
{
  // (define num 10)

  seni_node *sibling = safe_next(expr);
  if (sibling == NULL) {
    SENI_ERROR("no args given to 'define'");
    return NULL;
  }

  seni_var *env_var = NULL;
  
  while (sibling) {
    // get the binding name
    seni_node *name = sibling;
    sibling = safe_next(sibling);

    // get the value
    seni_var *v = eval(env, sibling);
    sibling = safe_next(sibling);

    // add the name/value binding to the current env
    env_var = bind_var(env, name->value.i, v);
  }

  return env_var;
}

seni_var *eval_keyword_fn(seni_env *env, seni_node *expr)
{
  // (fn (a) 42)
  // (fn (add a: 0 b: 0) (+ a b))
  
  seni_node *fn_keyword = expr;
  
  seni_node *def_list = safe_next(fn_keyword);
  if (!def_list || def_list->type != NODE_LIST) {
    SENI_ERROR("no name+parameter list given");
    // printf("error: no name+parameter list given\n");
  }

  seni_node *fn_name = def_list->value.first_child;
  
  // todo: parse the args ???

  seni_var fn_var;
  fn_var.type = VAR_FN;
  fn_var.value.n = expr;
#ifdef SENI_DEBUG_MODE
  fn_var.debug_allocatable = false;
#endif

  seni_var *env_var = bind_var(env, fn_name->value.i, &fn_var);
  
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
    SENI_ERROR("if test condition should evaluate to a boolean");
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

seni_var *eval_keyword_loop(seni_env *env, seni_node *expr)
{
  // (loop (x from: 1 to: 4) x)

  seni_var *res = NULL;

  seni_node *setup = safe_next(expr);
  if (setup->type != NODE_LIST) {
    SENI_ERROR("loop requires a list describing its behaviour");
    return NULL;
  }

  seni_node *body = safe_next(setup);
  seni_node *var_node = setup->value.first_child;

  i32 var_index = var_node->value.i;

  seni_env *loop_env = push_scope(env);
  {
    seni_node *args = safe_next(var_node);

    add_named_parameters_to_env(loop_env, args);

    f32 increment = 1.0f;
    f32 steps = 1.0f;
    seni_var *from_var = NULL;
    seni_var *to_var = NULL;
    seni_var *upto_var = NULL;
    
    bool has_from = has_named_node(args, g_arg_from);
    if (has_from) {
      from_var = lookup_var(loop_env, g_arg_from);
    }

    bool has_to = has_named_node(args, g_arg_to);
    if (has_to) {
      to_var = lookup_var(loop_env, g_arg_to);
    }

    bool has_upto = has_named_node(args, g_arg_upto);
    if (has_upto) {
      upto_var = lookup_var(loop_env, g_arg_upto);
    }

    bool has_increment = has_named_node(args, g_arg_increment);
    if (has_increment) {
      seni_var *increment_var = lookup_var(loop_env, g_arg_increment);
      increment = var_as_float(increment_var);
      if (increment == 0.0f) {
        SENI_ERROR("cannot have an increment of 0");
        pop_scope(loop_env);
        return NULL;
      }
    }

    bool has_steps = has_named_node(args, g_arg_steps);
    if (has_steps) {
      seni_var *steps_var = lookup_var(loop_env, g_arg_steps);
      steps = var_as_float(steps_var);
      if (steps == 0.0f) {
        SENI_ERROR("cannot have steps value of 0");
        pop_scope(loop_env);
        return NULL;
      }
    }

    if(has_increment || !has_steps) {
      f32 from = has_from ? var_as_float(from_var) : 0.0f;
      f32 to = has_to ? var_as_int(to_var) : 10.0f;
      f32 upto = has_upto ? var_as_int(upto_var) : 10.0f;

      // the default path - perform an integer based iteration
      f32 i;
      if (has_to || !has_upto) {

        if (from > to) {
          SENI_ERROR("from has to be less than to");
          pop_scope(loop_env);
          return NULL;
        }
    
        
        for (i = from; i < to; i += increment) {
          // todo: should each iteration of the loop have it's own env scope?
          bind_var_to_float(loop_env, var_index, i);
          res = eval_all_nodes(loop_env, body);
        }
      
      } else if (has_upto) {
        for (i = from; i <= upto; i += increment) {
          bind_var_to_float(loop_env, var_index, i);
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
    SENI_ERROR("setq expects a name as the first arg");
    return NULL;
  }

  // call lookup_var since we're overwriting the existing value
  seni_var *variable = lookup_var(env, name_node->value.i);
  safe_var_copy(variable, value_var);
  return variable;
}

/* used for debugging only */
seni_var *eval_keyword_vars(seni_env *env, seni_node *expr)
{
  // (#vars)
  safe_next(expr);

  debug_var_info(env);

  return NULL;;
}


void bind_core_declarations(seni_word_lut *wlut)
{
  // classic functions that don't use named arguments when invoked
  declare_keyword(wlut,    "+",             &eval_classic_fn_plus);
  declare_keyword(wlut,    "-",             &eval_classic_fn_minus);
  declare_keyword(wlut,    "*",             &eval_classic_fn_multiply);
  declare_keyword(wlut,    "/",             &eval_classic_fn_divide);
  declare_keyword(wlut,    "=",             &eval_classic_fn_equality);
  declare_keyword(wlut,    ">",             &eval_classic_fn_greater);
  declare_keyword(wlut,    "<",             &eval_classic_fn_lesser);
  declare_keyword(wlut,    "vector/append", &eval_classic_fn_vector_append);
  declare_keyword(wlut,    "sqrt",          &eval_classic_fn_sqrt);
  declare_keyword(wlut,    "mod",           &eval_classic_fn_mod);
  // special functions with non-standard syntax
  declare_keyword(wlut,    "define",        &eval_keyword_define);
  declare_keyword(wlut,    "fn",            &eval_keyword_fn);
  declare_keyword(wlut,    "if",            &eval_keyword_if);
  declare_keyword(wlut,    "loop",          &eval_keyword_loop);
  // for debugging
  declare_keyword(wlut,    "setq",          &eval_keyword_setq);
  declare_keyword(wlut,    "#vars",         &eval_keyword_vars);
}


void bind_vm_core_declarations(seni_word_lut *wlut)
{
  // classic functions that don't use named arguments when invoked
  declare_vm_keyword(wlut, "+", &(wlut->iname_plus));
  declare_vm_keyword(wlut, "-", &(wlut->iname_minus));
  declare_vm_keyword(wlut, "*", &(wlut->iname_mult));
  declare_vm_keyword(wlut, "/", &(wlut->iname_divide));
  declare_vm_keyword(wlut, "=", &(wlut->iname_equal));
  declare_vm_keyword(wlut, ">", &(wlut->iname_gt));
  declare_vm_keyword(wlut, "<", &(wlut->iname_lt));
  declare_vm_keyword(wlut, "vector/append", &(wlut->iname_vector_append));
  declare_vm_keyword(wlut, "sqrt", &(wlut->iname_sqrt));
  declare_vm_keyword(wlut, "mod", &(wlut->iname_mod));

  declare_vm_keyword(wlut, "and", &(wlut->iname_and));
  declare_vm_keyword(wlut, "or", &(wlut->iname_or));
  declare_vm_keyword(wlut, "not", &(wlut->iname_not));
  // TODO: on-matrix-stack
  // special functions with non-standard syntax
  declare_vm_keyword(wlut, "define", &(wlut->iname_define));
  declare_vm_keyword(wlut, "fn", &(wlut->iname_fn));
  declare_vm_keyword(wlut, "if", &(wlut->iname_if));
  declare_vm_keyword(wlut, "loop", &(wlut->iname_loop));
  // for debugging
  declare_vm_keyword(wlut, "setq", &(wlut->iname_setq));
  declare_vm_keyword(wlut, "#vars", &(wlut->iname_hash_vars));
}
