/*
  Runs tests using the native compiler
*/
#include "unity/unity.h"
#include "seni.h"

#include "seni_lang.h"
#include "seni_bind.h"
#include "seni_uv_mapper.h"
#include "seni_vm.h"

#include "time.h"
#include "stdio.h"
#include <stdlib.h>
#include <string.h>

/* way of working with boolean and TEST macros */
bool test_true = true;
bool test_false = false;

/* required by unity */
void setUp(void) { }
void tearDown(void) { }

void test_mathutil(void)
{
  TEST_ASSERT_EQUAL_FLOAT(1.5f, deg_to_rad(rad_to_deg(1.5f)));
  TEST_ASSERT_EQUAL_FLOAT(0.44444f, mc_m(1.0f, 1.0f, 10.0f, 5.0f));
  TEST_ASSERT_EQUAL_FLOAT(0.55556f, mc_c(1.0f, 1.0f, 0.444444f));
}

seni_node *assert_parser_node_raw(seni_node *node, seni_node_type type)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, node_type_name(node));
  return node->next;
}

seni_node *assert_parser_node_i32(seni_node *node, seni_node_type type, i32 val)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, node_type_name(node));
  TEST_ASSERT_EQUAL(val, node->value.i);
  return node->next;
}

seni_node *assert_parser_node_f32(seni_node *node, seni_node_type type, f32 val)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, node_type_name(node));
  TEST_ASSERT_EQUAL_FLOAT(val, node->value.f);
  return node->next;
}

seni_node *assert_parser_node_str(seni_node *node, seni_node_type type, char *val)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, node_type_name(node));
  TEST_ASSERT_EQUAL_STRING(val, node->value.s);
  return node->next;
}

seni_node *assert_parser_node_txt(seni_node *node, seni_node_type type, char *val, word_lut *wlut)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, node_type_name(node));

  char *c = wlut->words[node->value.i];
  TEST_ASSERT_EQUAL_STRING(val, c);
  
  return node->next;
}

#define PARSE(EXPR) wl = wlut_allocate(); \
  nodes = parser_parse(wl, EXPR)

#define PARSE_CLEANUP wlut_free(wl); \
  parser_free_nodes(nodes)


void test_lang_parser(void)
{
  seni_node *nodes, *iter, *iter2;
  word_lut *wl;

  PARSE("hello");
  assert_parser_node_txt(nodes, NODE_NAME, "hello", wl);
  PARSE_CLEANUP;

  PARSE("5");
  assert_parser_node_i32(nodes, NODE_INT, 5);
  PARSE_CLEANUP;

  PARSE("(4)");
  assert_parser_node_raw(nodes, NODE_LIST);
  PARSE_CLEANUP;

  PARSE("true");
  assert_parser_node_i32(nodes, NODE_BOOLEAN, true);
  PARSE_CLEANUP;

  PARSE("false");
  assert_parser_node_i32(nodes, NODE_BOOLEAN, false);
  PARSE_CLEANUP;

  PARSE("(add 1 2)");
  iter = nodes->value.first_child;
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->value.first_child;
  iter = assert_parser_node_txt(iter, NODE_NAME, "add", wl);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 1);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 2);
  TEST_ASSERT_NULL(iter);
  TEST_ASSERT_NULL(nodes->next);
  PARSE_CLEANUP;

  PARSE("[add 9 8 (foo)]");
  assert_parser_node_raw(nodes, NODE_VECTOR);
  iter = nodes->value.first_child;
  iter = assert_parser_node_txt(iter, NODE_NAME, "add", wl);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 9);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 8);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_raw(iter, NODE_LIST);
  TEST_ASSERT_NULL(iter);
  TEST_ASSERT_NULL(nodes->next);
  PARSE_CLEANUP;

  PARSE(";[add 9 8 (foo)]");
  assert_parser_node_str(nodes, NODE_COMMENT, ";[add 9 8 (foo)]");
  TEST_ASSERT_NULL(nodes->next);
  PARSE_CLEANUP;

  PARSE("'(runall \"shabba\") ; woohoo");
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->value.first_child;
  iter = assert_parser_node_txt(iter, NODE_NAME, "quote", wl);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter2 = iter;
  iter = assert_parser_node_raw(iter, NODE_LIST);
  TEST_ASSERT_NULL(iter);
  iter = iter2->value.first_child;
  iter = assert_parser_node_txt(iter, NODE_NAME, "runall", wl);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_txt(iter, NODE_STRING, "shabba", wl);
  iter = nodes->next;
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_str(iter, NODE_COMMENT, "; woohoo");
  TEST_ASSERT_NULL(iter);
  PARSE_CLEANUP;

  PARSE("(fun i: 42 f: 12.34)");
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->value.first_child;
  iter = assert_parser_node_txt(iter, NODE_NAME, "fun", wl);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_txt(iter, NODE_LABEL, "i", wl);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 42);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_txt(iter, NODE_LABEL, "f", wl);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_f32(iter, NODE_FLOAT, 12.34f);
  TEST_ASSERT_NULL(iter);
  TEST_ASSERT_NULL(nodes->next);
  PARSE_CLEANUP;

  PARSE("(a 1) (b 2)");
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->value.first_child;
  iter = assert_parser_node_txt(iter, NODE_NAME, "a", wl);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 1);
  TEST_ASSERT_NULL(iter);
  iter = nodes->next;
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  assert_parser_node_raw(iter, NODE_LIST);
  iter = iter->value.first_child;
  iter = assert_parser_node_txt(iter, NODE_NAME, "b", wl);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 2);
  TEST_ASSERT_NULL(iter);
  PARSE_CLEANUP;

  PARSE("(a {[1 2]})");
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->value.first_child;
  iter = assert_parser_node_txt(iter, NODE_NAME, "a", wl);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter2 = iter; // the vector
  iter = assert_parser_node_raw(iter, NODE_VECTOR);
  TEST_ASSERT_NULL(iter);
  TEST_ASSERT_EQUAL(test_true, iter2->alterable);
  TEST_ASSERT_NULL(nodes->next);
  PARSE_CLEANUP;
}

void test_lang_env(void)
{
  seni_env *env, *env2;
  seni_var *var, *var2;

  env_allocate_pools();
  env = get_initial_env(NULL);

  var = get_binded_var(env, 1);
  var->type = VAR_INT;
  var->value.i = 42;

  /* basic lookup */
  var2 = lookup_var(env, 1);
  TEST_ASSERT_EQUAL(42, var2->value.i);

  /* lookup an outer scope */
  env2 = push_scope(env);
  var2 = lookup_var(env2, 1);
  TEST_ASSERT_EQUAL(42, var2->value.i);  

  /* redefine current scope */
  var2 = get_binded_var(env2, 1);
  var2->value.i = 100;
  var2 = lookup_var(env2, 1);
  TEST_ASSERT_EQUAL(100, var2->value.i);

  /* pop scope and get back previous value */
  env2 = pop_scope(env2);
  var2 = lookup_var(env2, 1);
  TEST_ASSERT_EQUAL(42, var2->value.i);

  env_free_pools();
}

void assert_seni_var(seni_var *var, seni_var_type type)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, var->type, var_type_name(var));
}

void assert_seni_var_i32(seni_var *var, seni_var_type type, i32 i)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, var->type, var_type_name(var));
  TEST_ASSERT_EQUAL(i, var->value.i);
}

void assert_seni_var_vec(seni_var *var)
{
  TEST_ASSERT_EQUAL_MESSAGE(VAR_VEC_HEAD, var->type, var_type_name(var));
}

void assert_seni_var_vec_i32(seni_var *var, i32 i)
{
  TEST_ASSERT_EQUAL_MESSAGE(VAR_INT, var->type, var_type_name(var));
  assert_seni_var_i32(var, VAR_INT, i);
}

void assert_seni_var_f32(seni_var *var, seni_var_type type, f32 f)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, var->type, var_type_name(var));
  TEST_ASSERT_EQUAL_FLOAT(f, var->value.f);
}

void assert_seni_var_f32_within(seni_var *var, seni_var_type type, f32 f, f32 tolerance)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, var->type, var_type_name(var));
  TEST_ASSERT_FLOAT_WITHIN(tolerance, f, var->value.f);
}

void assert_seni_var_bool(seni_var *var, bool b)
{
  TEST_ASSERT_EQUAL_MESSAGE(VAR_BOOLEAN, var->type, var_type_name(var));
  TEST_ASSERT_EQUAL(b ? 1 : 0, var->value.i);
}

void assert_seni_var_true(seni_var *var)
{
  TEST_ASSERT_EQUAL_MESSAGE(VAR_BOOLEAN, var->type, var_type_name(var));
  TEST_ASSERT_EQUAL(1, var->value.i);
}

void assert_seni_var_false(seni_var *var)
{
  TEST_ASSERT_EQUAL_MESSAGE(VAR_BOOLEAN, var->type, var_type_name(var));
  TEST_ASSERT_EQUAL(0, var->value.i);
}

word_lut *setup_interpreter_wl()
{
  word_lut *wl = wlut_allocate();
  // add keywords to the word_lut and setup function pointers within the interpreter
  interpreter_declare_keywords(wl);

  return wl;
}

seni_env *setup_basic_interpreter_env()
{
  env_allocate_pools();
  seni_env *env = get_initial_env(NULL);

  return env;
}

seni_env *setup_interpreter_env(word_lut *wl)
{
  seni_env *env = setup_basic_interpreter_env();

  /* add a temporary debug variable for testing loops etc */
  i32 index = wlut_lookup_or_add(wl, "#test", sizeof("#test"));
  seni_var *var = get_binded_var(env, index);
  var->type = VAR_INT;
  var->value.i = 0;

  return env;
}

void shutdown_interpreter_test(word_lut *wl, seni_node *ast)
{
  env_free_pools();
  wlut_free(wl);
  parser_free_nodes(ast);
}

void add_binding_i32(word_lut *wl, seni_env *env, char *name, i32 i)
{
  // add a foo binding to env
  i32 name_index = wlut_lookup_or_add(wl, name, strlen(name));
  seni_var *v = get_binded_var(env, name_index);
  v->type = VAR_INT;
  v->value.i = i;
}

void debug_memory(char *expression)
{
  seni_debug_info debug_info;
  fill_debug_info(&debug_info);
  
  TEST_ASSERT_EQUAL_MESSAGE(debug_info.num_var_allocated, debug_info.num_var_available, expression);
  TEST_ASSERT_EQUAL_MESSAGE(debug_info.var_get_count, debug_info.var_return_count, expression);
}

#define EVAL_EXPR(EXPR) debug_reset(); \
  wl = setup_interpreter_wl(); \
  env = setup_interpreter_env(wl); \
  ast = parser_parse(wl, EXPR); \
  var = evaluate(env, ast, false)

#define EVAL_MEM(EXPR) debug_reset(); \
  wl = setup_interpreter_wl(); \
  env = setup_basic_interpreter_env(); \
  ast = parser_parse(wl, EXPR); \
  var = evaluate(env, ast, true);    \
  debug_memory(EXPR)

#define DEBUG_EXPR(EXPR) debug_reset();         \
  wl = setup_interpreter_wl(); \
  env = setup_basic_interpreter_env(); \
  ast = parser_parse(wl, EXPR); \
  var = evaluate(env, ast, true);    \
  debug_var_info(env)

#define EVAL_CLEANUP shutdown_interpreter_test(wl, ast)

#define EVAL_INT(EXPR,EXPECTED) EVAL_EXPR(EXPR);  \
  assert_seni_var_i32(var, VAR_INT, EXPECTED); \
  EVAL_CLEANUP

#define EVAL_FLOAT(EXPR,EXPECTED) EVAL_EXPR(EXPR); \
  assert_seni_var_f32(var, VAR_FLOAT, EXPECTED); \
  EVAL_CLEANUP

#define EVAL_FLOAT_WITHIN(EXPR,EXPECTED,TOLERANCE) EVAL_EXPR(EXPR);  \
  assert_seni_var_f32_within(var, VAR_FLOAT, EXPECTED, TOLERANCE); \
  EVAL_CLEANUP

#define EVAL_TRUE(EXPR) EVAL_EXPR(EXPR); \
  assert_seni_var_true(var); \
  EVAL_CLEANUP

#define EVAL_FALSE(EXPR) EVAL_EXPR(EXPR); \
  assert_seni_var_false(var); \
  EVAL_CLEANUP

#define EVAL_NAME(EXPR,EXPECTED) EVAL_EXPR(EXPR); \
  assert_seni_var_i32(var, VAR_NAME, EXPECTED); \
  EVAL_CLEANUP

#define SENI_ASSERT_VEC() assert_seni_var_vec(var); \
  var = var->value.v; \
  var = var->value.v

#define SENI_ASSERT_VEC_INT(EXPECTED) assert_seni_var_vec_i32(var, EXPECTED); \
  var = var->next

#define EVAL_DECL   word_lut *wl = NULL; \
  seni_env *env = NULL; \
  seni_node *ast = NULL; \
  seni_var *var = NULL

void test_lang_interpret_basic(void)
{
  EVAL_DECL;

  // basic eval
  EVAL_INT("42", 42);
  EVAL_FLOAT("12.34", 12.34f);
  EVAL_TRUE("true");
  EVAL_FALSE("false");
}

void test_lang_interpret_math(void)
{
  EVAL_DECL;

  EVAL_INT("(+ 10 1)", 11);
  EVAL_FLOAT("(+ 10.0 1)", 11.0f);
  EVAL_FLOAT("(+ 10 1.0)", 11.0f);
  EVAL_INT("(+ 3 4 5 6)", 18);
  EVAL_INT("(+ (+ 1 2) (+ 3 4))", 10);
  EVAL_FLOAT("(+ (+ 1 2) (+ 3.0 4))", 10.0f);
  EVAL_INT("(- 100 20)", 80);
  EVAL_INT("(- (+ 50 50) 20)", 80);
  EVAL_INT("(- 59)", -59);
  EVAL_INT("(- (+ 50 9))", -59);
  EVAL_FLOAT("(- 100.0 20)", 80.0f);
  EVAL_FLOAT("(- 100 20.0)", 80.0f);
  EVAL_FLOAT("(- 100.0 20.0)", 80.0f);
  EVAL_INT("(* 6 5)", 30);
  EVAL_INT("(* (* 2 3) 5)", 30);
  EVAL_FLOAT("(/ 16.0 2.0)", 8.0f);

  EVAL_FLOAT("(sqrt 144)", 12.0f);
  EVAL_FLOAT("(sqrt 144.0)", 12.0f);
  EVAL_FLOAT("(sqrt (+ 100 44))", 12.0f);

  EVAL_INT("(mod 10 3)", 1);
  EVAL_INT("(mod 11 3)", 2);
}

void test_lang_interpret_comparison(void)
{
  EVAL_DECL;

  EVAL_TRUE("(= 16.0 16.0)");
  EVAL_FALSE("(= 16.0 99.0)");
  EVAL_FALSE("(= 16.0 16)");
  EVAL_TRUE("(= 6 6)");
  EVAL_FALSE("(= 6 26)");
  EVAL_TRUE("(> 6 2)");
  EVAL_FALSE("(> 6 6)");
  EVAL_FALSE("(> 6 26)");
  EVAL_TRUE("(> 1000 100 10 1)");
  EVAL_TRUE("(> 6 2.0)");
  EVAL_TRUE("(< 7 10)");
  EVAL_FALSE("(< 7 5)");
}

void test_lang_interpret_define(void)
{
  EVAL_DECL;

  EVAL_INT("(define num 10) (+ num num)", 20);
  EVAL_FLOAT("(define num 10.0) (+ num num)", 20.0f);
  EVAL_FLOAT("(define num (* 2 3.0)) (+ num num num)", 18.0f);
  // multiple defines
  EVAL_INT("(define a 10 b 5 c 2) (+ a b c)", 17);
}

void test_lang_interpret_function(void)
{
  EVAL_DECL;

  EVAL_INT("(fn (a) 42) (+ (a) (a))", 84);
  EVAL_INT("(fn (a) 12 34 55 42) (+ (a) (a))", 84);
  EVAL_INT("(fn (foo b: 1 c: 2) (+ b c)) (foo)", 3);
  EVAL_INT("(fn (foo b: 1 c: 2) (+ b c)) (foo b: 10 c: 100)", 110);
  EVAL_FLOAT("(fn (foo b: 1 c: 2) (+ b c)) (foo c: 30 b: 5.6)", 35.6f);
  EVAL_INT("(define b 10)(fn (foo b: 1) (+ b b)) (foo b: (+ b b))", 40);
}  

void test_lang_interpret_if(void)
{
  EVAL_DECL;

  EVAL_INT("(if true 3 4)", 3);
  EVAL_INT("(if true 3)", 3);
  EVAL_INT("(if false 3 4)", 4);

  EVAL_INT("(if (> 100 1) 5 6)", 5);
  EVAL_INT("(if (> 1 100) 5 6)", 6);
}  

/* temporary, only used for testing */
void test_lang_interpret_setq(void)
{
  EVAL_DECL;

  EVAL_INT("(define x 7) (setq x 4)", 4);
  EVAL_INT("(setq #test 5) #test", 5);
}  

void test_lang_interpret_loop(void)
{
  EVAL_DECL;

  // #test is a special global for unit testing that we've defined
  // in setup_interpreter_env

  // basic from, to and upto
  EVAL_INT("(loop (x from: 0 to: 5) (setq #test x)) #test", 4);
  EVAL_INT("(loop (x from: 0 upto: 5) (setq #test x)) #test", 5);
  EVAL_INT("(loop (x to: 8) (setq #test x)) #test", 7);
  EVAL_INT("(loop (x upto: 8) (setq #test x)) #test", 8);
  EVAL_INT("(setq #test 1)(loop (x from: 0 to: 5) (setq #test (+ #test #test))) #test", 32);
  EVAL_INT("(loop (x from: 1 to: 4) (setq #test (+ #test x))) #test", 6);
  EVAL_INT("(loop (x from: 1 upto: 4) (setq #test (+ #test x))) #test", 10);

  // change increment
  EVAL_INT("(loop (x from: 0 to: 10 increment: 2) (setq #test (+ #test x))) #test", 20);

  // steps
  EVAL_FLOAT_WITHIN("(loop (x from: 0   to: 10 steps: 3) (setq #test (+ #test x))) #test",
                    0.00f + 3.333f + 6.666f, 0.1f);
  EVAL_FLOAT_WITHIN("(loop (x from: 0 upto: 10 steps: 3) (setq #test (+ #test x))) #test",
                    0.00f + 5.0f + 10.0f, 0.1f);
  //  EVAL_FLOAT("(loop (x from: 0 upto: 10 steps: 3) (setq #test (+ #test x))) #test", 0.00f + 5.0f + 10.0f);
}

void test_lang_interpret_vector(void)
{
  EVAL_DECL;
#if 1
  {
    EVAL_EXPR("(define x [])");

    SENI_ASSERT_VEC();
    TEST_ASSERT_NULL(var);
    
    EVAL_CLEANUP;
  }
#endif
  
  {
    EVAL_EXPR("(define x [3])");

    SENI_ASSERT_VEC();
    SENI_ASSERT_VEC_INT(3);
    TEST_ASSERT_NULL(var);

    EVAL_CLEANUP;
  }

  {
    EVAL_EXPR("(define y [8 7 6])");

    SENI_ASSERT_VEC();
    SENI_ASSERT_VEC_INT(8);
    SENI_ASSERT_VEC_INT(7);
    SENI_ASSERT_VEC_INT(6);
    TEST_ASSERT_NULL(var);

    EVAL_CLEANUP;
  }
  
  {
    EVAL_EXPR("(define y []) (vector/append y 5)");

    SENI_ASSERT_VEC();
    SENI_ASSERT_VEC_INT(5);
    TEST_ASSERT_NULL(var);

    EVAL_CLEANUP;
  }

  {
    EVAL_EXPR("(define y [3]) (vector/append y 4)");

    SENI_ASSERT_VEC();
    SENI_ASSERT_VEC_INT(3);
    SENI_ASSERT_VEC_INT(4);
    TEST_ASSERT_NULL(var);

    EVAL_CLEANUP;
  }

  {
    EVAL_EXPR("(define y [4 5]) (vector/append y 6)");

    SENI_ASSERT_VEC();
    SENI_ASSERT_VEC_INT(4);
    SENI_ASSERT_VEC_INT(5);
    SENI_ASSERT_VEC_INT(6);
    TEST_ASSERT_NULL(var);

    EVAL_CLEANUP;
  }

  {
    EVAL_EXPR("(define y [8 7 6]) (vector/append y 5)");

    SENI_ASSERT_VEC();
    SENI_ASSERT_VEC_INT(8);
    SENI_ASSERT_VEC_INT(7);
    SENI_ASSERT_VEC_INT(6);
    SENI_ASSERT_VEC_INT(5);
    TEST_ASSERT_NULL(var);

    EVAL_CLEANUP;
  }
}

void test_lang_interpret_mem(void)
{
  EVAL_DECL;

  // tests the memory allocations for any leaks or double-free

  {
    EVAL_MEM("(fn (some-fn) (define a [3 4])(setq a 11))");
    EVAL_CLEANUP;
  }
  
  {
    EVAL_MEM("(define b [2 3]) (fn (some-fn) (define a [1 2 3])(setq a b))");
    EVAL_CLEANUP;
  }

  {
    EVAL_MEM("(fn (some-fn)(define a [7 8]) (define b [2 3]) (setq a b))(some-fn)");
    EVAL_CLEANUP;
  }

  {
    EVAL_MEM("(define g 0) (loop (x from: 0 upto: 10 steps: 3) (setq g (+ g x)))");
    EVAL_CLEANUP;
  }

  {
    EVAL_MEM("(define b [2 3]) (fn (some-fn) (define a [1 2 3]) (vector/append b a))");
    EVAL_CLEANUP;
  }

  {
    EVAL_MEM("(define b [2 3]) (fn (some-fn) (define a 1) (vector/append b a))");
    EVAL_CLEANUP;
  }  
}

void debug_lang_interpret_mem(void)
{
  EVAL_DECL;

  {
    DEBUG_EXPR("(fn (some-fn) (define a [3 4])(setq a 11))");
    EVAL_CLEANUP;
  }

  {
    DEBUG_EXPR("(define b [2 3]) (fn (some-fn) (define a [1 2 3])(setq a b))");
    EVAL_CLEANUP;
  }

  {
    DEBUG_EXPR("(fn (some-fn)(define a [7 8]) (define b [2 3]) (setq a b))(some-fn)");
    EVAL_CLEANUP;
  }

  {
    DEBUG_EXPR("(define g 0) (loop (x from: 0 upto: 10 steps: 3) (setq g (+ g x)))");
    EVAL_CLEANUP;
  }

  {
    DEBUG_EXPR("(define b [2 3]) (fn (some-fn) (define a [1 2 3]) (vector/append b a))");
    EVAL_CLEANUP;
  }

  {
    DEBUG_EXPR("(define b [2 3]) (fn (some-fn) (define a 1) (vector/append b a))");
    EVAL_CLEANUP;
  }

  {
    DEBUG_EXPR("(define b [2 3]) (fn (some-fn) (define b 1) (+ b b)) (some-fn)");
    EVAL_CLEANUP;
  }
}

void test_uv_mapper(void)
{
  init_uv_mapper();

  seni_uv_mapping *flat = get_uv_mapping(BRUSH_FLAT, 0);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 1.0f, flat->width_scale);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 2.0f / 1024.0f, flat->map[0].x);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 1.0f / 1024.0f, flat->map[0].y);
  
  TEST_ASSERT_NULL(get_uv_mapping(BRUSH_FLAT, 1)); // out of range

  seni_uv_mapping *c = get_uv_mapping(BRUSH_C, 8);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 1.1f, c->width_scale);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 326.0f / 1024.0f, c->map[0].x);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 556.0f / 1024.0f, c->map[0].y);

  free_uv_mapper();
}

// --------------------------------------------------

// debug version of VM_COMPILE - prints the bytecode
//
#define DVM_COMPILE(EXPR) word_lut *wl = NULL;            \
  seni_node *ast = NULL;                                  \
  seni_program *prog = NULL;                              \
  seni_virtual_machine *vm = NULL;                        \
  debug_reset();                                          \
  wl = setup_interpreter_wl();                            \
  ast = parser_parse(wl, EXPR);                           \
  prog = program_allocate(256);                           \
  compiler_compile(ast, prog, wl);                        \
  vm = virtual_machine_construct(STACK_SIZE,MEMORY_SIZE); \
  printf("%s\n", EXPR);                                   \
  program_pretty_print(prog);

// --------------------------------------------------

// eval version of VM_COMPILE - evals and compares result to an int
//
#define EVM_COMPILE(EXPR) word_lut *wl = NULL;              \
  seni_node *ast = NULL;                                    \
  seni_program *prog = NULL;                                \
  seni_virtual_machine *vm = NULL;                          \
  debug_reset();                                            \
  wl = setup_interpreter_wl();                              \
  ast = parser_parse(wl, EXPR);                             \
  prog = program_allocate(256);                             \
  compiler_compile(ast, prog, wl);                          \
  vm = virtual_machine_construct(STACK_SIZE,MEMORY_SIZE);   \
  vm_interpret(vm, prog)


#define VM_TEST_INT(RES) assert_seni_var_i32(stack_pop(vm), VAR_INT, RES)
#define VM_TEST_BOOL(RES) assert_seni_var_bool(stack_pop(vm), RES)

#define VM_CLEANUP shutdown_interpreter_test(wl, ast);  \
  program_free(prog);                                   \
  virtual_machine_free(vm)


// COMPILE macros that eval and compare results
//

#if 1
#define VM_COMPILE_INT(EXPR,RES) {EVM_COMPILE(EXPR);VM_TEST_INT(RES);VM_CLEANUP;}
#define VM_COMPILE_BOOL(EXPR,RES) {EVM_COMPILE(EXPR);VM_TEST_BOOL(RES);VM_CLEANUP;}
#else
// COMPILE macros that print out bytecode
//
#define VM_COMPILE_INT(EXPR,_) {DVM_COMPILE(EXPR);VM_CLEANUP;}
#define VM_COMPILE_BOOL(EXPR,_) {DVM_COMPILE(EXPR);VM_CLEANUP;}
#endif
// --------------------------------------------------

void test_vm_bytecode(void)
{
  VM_COMPILE_INT("(define a 42) (define b 52) 10", 10);
  VM_COMPILE_INT("(define a 6) (define b 7) (+ a b)", 13);
  VM_COMPILE_INT("(+ 3 4)", 7);
  VM_COMPILE_INT("(- (+ 1 2) 3)", 0);
  VM_COMPILE_BOOL("(> 5 10)", false);
  VM_COMPILE_BOOL("(< 5 10)", true);
  VM_COMPILE_BOOL("(= 2 2)", true);
  VM_COMPILE_BOOL("(= 1 2)", false);
  VM_COMPILE_BOOL("(and (< 1 2) (< 3 4))", true);
  VM_COMPILE_BOOL("(and (< 1 2) (> 3 4))", false);
  VM_COMPILE_BOOL("(or (< 1 2) (> 3 4))", true);
  VM_COMPILE_BOOL("(not (> 1 10))", true);
  VM_COMPILE_BOOL("(and (or (< 1 2) (> 3 4)) (not (> 1 10)))", true);

  VM_COMPILE_INT("(if (> 400 200) 66)", 66);
  VM_COMPILE_INT("(if (> 200 100) 12 24)", 12);
  VM_COMPILE_INT("(if (< 200 100) 12 24)", 24);
  VM_COMPILE_BOOL("(if (> 400 200) (= 50 50))", true);
  VM_COMPILE_BOOL("(if (> 99 88) (= 3 4) (= 5 5))", false);
  VM_COMPILE_BOOL("(if (< 99 88) (= 3 4) (= 5 5))", true);

  VM_COMPILE_INT("(loop (x from: 0 to: 5) (+ 42 38)) 9", 9);
  VM_COMPILE_INT("(loop (x from: 0 to: 5) (loop (y from: 0 to: 5) (+ 3 4))) 9", 9);
}

void timing(void)
{
  clock_t start, diff;
  int msec;

  {
    EVAL_DECL;
    start = clock();
    // EVAL_INT("(loop (x from: 0 to: 1000000) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1)) 4", 4);
    EVAL_INT("(loop (x from: 0 to: 10000) (loop (y from: 0 to: 1000) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (+ 3 4))) 9", 9);
    diff = clock() - start;
    msec = diff * 1000 / CLOCKS_PER_SEC;
    printf("Eval Time taken %d seconds %d milliseconds\n", msec/1000, msec%1000);
  }

  {
    start = clock();
    //VM_COMPILE_INT("(loop (x from: 0 to: 1000000) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1)) 4", 4);

    VM_COMPILE_INT("(loop (x from: 0 to: 10000) (loop (y from: 0 to: 1000) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (+ 3 4))) 9", 9);
    diff = clock() - start;
    msec = diff * 1000 / CLOCKS_PER_SEC;
    printf("VM Time taken %d seconds %d milliseconds\n", msec/1000, msec%1000);
  }
}

void test_vm_callret(void)
{
  // VM_COMPILE_INT("(define a 42) (define b 52) 10", 10);
  // VM_COMPILE_INT("(define a 6) (define b 7) (+ a b)", 13);
  // VM_COMPILE_INT("(+ 3 4)", 7);
  // VM_COMPILE_INT("(- (+ 1 2) 3)", 0);
  // VM_COMPILE_BOOL("(> 5 10)", false);
  // VM_COMPILE_BOOL("(< 5 10)", true);
  // VM_COMPILE_BOOL("(= 2 2)", true);
  // VM_COMPILE_BOOL("(= 1 2)", false);
  // VM_COMPILE_BOOL("(and (< 1 2) (< 3 4))", true);
  // VM_COMPILE_BOOL("(and (< 1 2) (> 3 4))", false);
  // VM_COMPILE_BOOL("(or (< 1 2) (> 3 4))", true);
  // VM_COMPILE_BOOL("(not (> 1 10))", true);
  // VM_COMPILE_BOOL("(and (or (< 1 2) (> 3 4)) (not (> 1 10)))", true);

  // VM_COMPILE_INT("(if (> 400 200) 66)", 66);
  // VM_COMPILE_INT("(if (> 200 100) 12 24)", 12);
  // VM_COMPILE_INT("(if (< 200 100) 12 24)", 24);
  // VM_COMPILE_BOOL("(if (> 400 200) (= 50 50))", true);
  // VM_COMPILE_BOOL("(if (> 99 88) (= 3 4) (= 5 5))", false);
  // VM_COMPILE_BOOL("(if (< 99 88) (= 3 4) (= 5 5))", true);

  // (fn (adder a: 0 b: 0) (+ a b)) (adder a: 3 b: 5)

  // VM_COMPILE_INT("(+ a b)", 0);
   VM_COMPILE_INT("(fn (adder a: 0 b: 0) (+ a b)) (adder a: 3 b: 5)", 0);
  // VM_COMPILE_INT("(fn (adder a: 0 b: 0) (+ a b)) (fn (bbb c: 0 d: 0) (+ c d))", 0);
  // VM_COMPILE_INT("(fn (adder a: 0 b: 0) (define c 5) (+ a b c)) (adder a: 3 b: 5)", 0);

  // VM_COMPILE_INT("(loop (x from: 0 to: 5) (+ 42 38)) 9", 9);
}

int main(void)
{
  // timing();
    
  UNITY_BEGIN();

  //RUN_TEST(debug_lang_interpret_mem); // for debugging/development
  
  RUN_TEST(test_mathutil);
  RUN_TEST(test_lang_parser);
  RUN_TEST(test_lang_env);
  RUN_TEST(test_uv_mapper);

  RUN_TEST(test_lang_interpret_basic);
  RUN_TEST(test_lang_interpret_math);
  RUN_TEST(test_lang_interpret_comparison);
  RUN_TEST(test_lang_interpret_define);
  RUN_TEST(test_lang_interpret_function);
  RUN_TEST(test_lang_interpret_if);
  RUN_TEST(test_lang_interpret_setq);
  RUN_TEST(test_lang_interpret_loop);
  RUN_TEST(test_lang_interpret_vector);
  RUN_TEST(test_lang_interpret_mem);
  
  // // vm
  RUN_TEST(test_vm_bytecode);

  // RUN_TEST(test_vm_callret);
  
  return UNITY_END();
}
