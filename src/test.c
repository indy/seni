/*
  Runs tests using the native compiler
*/
#include "unity/unity.h"
#include "seni.h"

#include "seni_lang_word_lookup.h"
#include "seni_lang_parser.h"
#include "seni_lang_env.h"
#include "seni_lang_interpreter.h"

#include "stdio.h"

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
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, parser_node_type_name(node->type));
  return node->next;
}

seni_node *assert_parser_node_i32(seni_node *node, seni_node_type type, i32 val)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, parser_node_type_name(node->type));
  TEST_ASSERT_EQUAL(val, node->value.i);
  return node->next;
}

seni_node *assert_parser_node_f32(seni_node *node, seni_node_type type, f32 val)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, parser_node_type_name(node->type));
  TEST_ASSERT_EQUAL_FLOAT(val, node->value.f);
  return node->next;
}

seni_node *assert_parser_node_str(seni_node *node, seni_node_type type, char *val)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, parser_node_type_name(node->type));
  TEST_ASSERT_EQUAL_STRING(val, node->value.s);
  return node->next;
}

seni_node *assert_parser_node_txt(seni_node *node, seni_node_type type, char *val, word_lut *wlut)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, parser_node_type_name(node->type));

  char *c = wlut->words[node->value.i];
  TEST_ASSERT_EQUAL_STRING(val, c);
  
  return node->next;
}

void test_lang_parser(void)
{
  seni_node *nodes, *iter, *iter2;
  word_lut *wl;

  {
    wl = word_lookup_allocate();
    nodes = parser_parse(wl, "hello");
    assert_parser_node_txt(nodes, NODE_NAME, "hello", wl);
    word_lookup_free(wl);
    parser_free_nodes(nodes);
  }

  {
    wl = word_lookup_allocate();
    nodes = parser_parse(wl, "5");
    assert_parser_node_i32(nodes, NODE_INT, 5);
    word_lookup_free(wl);
    parser_free_nodes(nodes);
  }

  {
    wl = word_lookup_allocate();
    nodes = parser_parse(wl, "(4)");
    assert_parser_node_raw(nodes, NODE_LIST);
    word_lookup_free(wl);
    parser_free_nodes(nodes);
  }

  {
    wl = word_lookup_allocate();
    nodes = parser_parse(wl, "true");
    assert_parser_node_i32(nodes, NODE_BOOLEAN, true);
    word_lookup_free(wl);
    parser_free_nodes(nodes);
  }

  {
    wl = word_lookup_allocate();
    nodes = parser_parse(wl, "false");
    assert_parser_node_i32(nodes, NODE_BOOLEAN, false);
    word_lookup_free(wl);
    parser_free_nodes(nodes);
  }

  {
    wl = word_lookup_allocate();
    nodes = parser_parse(wl, "(add 1 2)");
    iter = nodes->children;
    assert_parser_node_raw(nodes, NODE_LIST);
    iter = nodes->children;
    iter = assert_parser_node_txt(iter, NODE_NAME, "add", wl);
    iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
    iter = assert_parser_node_i32(iter, NODE_INT, 1);
    iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
    iter = assert_parser_node_i32(iter, NODE_INT, 2);
    TEST_ASSERT_NULL(iter);
    TEST_ASSERT_NULL(nodes->next);
    word_lookup_free(wl);
    parser_free_nodes(nodes);
  }

  {
    wl = word_lookup_allocate();
    nodes = parser_parse(wl, "[add 9 8 (foo)]");
    assert_parser_node_raw(nodes, NODE_VECTOR);
    iter = nodes->children;
    iter = assert_parser_node_txt(iter, NODE_NAME, "add", wl);
    iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
    iter = assert_parser_node_i32(iter, NODE_INT, 9);
    iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
    iter = assert_parser_node_i32(iter, NODE_INT, 8);
    iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
    iter = assert_parser_node_raw(iter, NODE_LIST);
    TEST_ASSERT_NULL(iter);
    TEST_ASSERT_NULL(nodes->next);
    word_lookup_free(wl);
    parser_free_nodes(nodes);
  }

  {
    wl = word_lookup_allocate();
    nodes = parser_parse(wl, ";[add 9 8 (foo)]");
    assert_parser_node_str(nodes, NODE_COMMENT, ";[add 9 8 (foo)]");
    TEST_ASSERT_NULL(nodes->next);
    word_lookup_free(wl);
    parser_free_nodes(nodes);
  }

  {
    wl = word_lookup_allocate();
    nodes = parser_parse(wl, "'(runall \"shabba\") ; woohoo");
    assert_parser_node_raw(nodes, NODE_LIST);
    iter = nodes->children;
    iter = assert_parser_node_txt(iter, NODE_NAME, "quote", wl);
    iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
    iter2 = iter;
    iter = assert_parser_node_raw(iter, NODE_LIST);
    TEST_ASSERT_NULL(iter);
    iter = iter2->children;
    iter = assert_parser_node_txt(iter, NODE_NAME, "runall", wl);
    iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
    iter = assert_parser_node_txt(iter, NODE_STRING, "shabba", wl);
    iter = nodes->next;
    iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
    iter = assert_parser_node_str(iter, NODE_COMMENT, "; woohoo");
    TEST_ASSERT_NULL(iter);
    word_lookup_free(wl);
    parser_free_nodes(nodes);
  }

  {
    wl = word_lookup_allocate();
    nodes = parser_parse(wl, "(fun i: 42 f: 12.34)");
    assert_parser_node_raw(nodes, NODE_LIST);
    iter = nodes->children;
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
    word_lookup_free(wl);
    parser_free_nodes(nodes);
  }

  {
    wl = word_lookup_allocate();
    nodes = parser_parse(wl, "(a 1) (b 2)");
    assert_parser_node_raw(nodes, NODE_LIST);
    iter = nodes->children;
    iter = assert_parser_node_txt(iter, NODE_NAME, "a", wl);
    iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
    iter = assert_parser_node_i32(iter, NODE_INT, 1);
    TEST_ASSERT_NULL(iter);
    iter = nodes->next;
    iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
    assert_parser_node_raw(iter, NODE_LIST);
    iter = iter->children;
    iter = assert_parser_node_txt(iter, NODE_NAME, "b", wl);
    iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
    iter = assert_parser_node_i32(iter, NODE_INT, 2);
    TEST_ASSERT_NULL(iter);
    word_lookup_free(wl);
    parser_free_nodes(nodes);
  }

  {
    wl = word_lookup_allocate();
    nodes = parser_parse(wl, "(a {[1 2]})");
    assert_parser_node_raw(nodes, NODE_LIST);
    iter = nodes->children;
    iter = assert_parser_node_txt(iter, NODE_NAME, "a", wl);
    iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
    iter2 = iter; // the vector
    iter = assert_parser_node_raw(iter, NODE_VECTOR);
    TEST_ASSERT_NULL(iter);
    TEST_ASSERT_EQUAL(test_true, iter2->alterable);
    TEST_ASSERT_NULL(nodes->next);
    word_lookup_free(wl);
    parser_free_nodes(nodes);
  }
}

void test_lang_env(void)
{
  seni_env *env, *env2;
  seni_var *var, *var2;

  env_allocate_pools();
  env = get_initial_env();

  var = add_var(env, 1);
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
  var2 = add_var(env2, 1);
  var2->value.i = 100;
  var2 = lookup_var(env2, 1);
  TEST_ASSERT_EQUAL(100, var2->value.i);

  /* pop scope and get back previous value */
  env2 = pop_scope(env2);
  var2 = lookup_var(env2, 1);
  TEST_ASSERT_EQUAL(42, var2->value.i);

  env_free_pools();
}

char *var_type_name(seni_var_type type)
{
  switch(type) {
  case VAR_INT:
    return "VAR_INT";
  case VAR_FLOAT:
    return "VAR_FLOAT";
  case VAR_BOOLEAN:
    return "VAR_BOOLEAN";
  case VAR_NAME:
    return "VAR_NAME";
  case VAR_FN:
    return "VAR_FN";
  }

  return "unknown var type";
}

void assert_seni_var(seni_var *var, seni_var_type type)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, var->type, var_type_name(var->type));
}

void assert_seni_var_i32(seni_var *var, seni_var_type type, i32 i)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, var->type, var_type_name(var->type));
  TEST_ASSERT_EQUAL(i, var->value.i);
}

void assert_seni_var_f32(seni_var *var, seni_var_type type, f32 f)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, var->type, var_type_name(var->type));
  TEST_ASSERT_EQUAL_FLOAT(f, var->value.f);
}

void assert_seni_var_true(seni_var *var)
{
  TEST_ASSERT_EQUAL_MESSAGE(VAR_BOOLEAN, var->type, var_type_name(var->type));
  TEST_ASSERT_EQUAL(1, var->value.i);
}

void assert_seni_var_false(seni_var *var)
{
  TEST_ASSERT_EQUAL_MESSAGE(VAR_BOOLEAN, var->type, var_type_name(var->type));
  TEST_ASSERT_EQUAL(0, var->value.i);
}

word_lut *setup_interpreter_wl()
{
  word_lut *wl = word_lookup_allocate();
  // add keywords to the word_lut and setup function pointers within the interpreter
  interpreter_declare_keywords(wl);

  return wl;
}

seni_env *setup_interpreter_env()
{
  env_allocate_pools();
  seni_env *env = get_initial_env();
  add_var(env, 0);

  return env;
}

void shutdown_interpreter_test(word_lut *wl, seni_node *ast)
{
  env_free_pools();
  word_lookup_free(wl);
  parser_free_nodes(ast);
}

void add_binding_i32(word_lut *wl, seni_env *env, char *name, i32 i)
{
  // add a foo binding to env
  i32 name_index = word_lookup_or_add(wl, name, strlen(name));
  seni_var *v = add_var(env, name_index);
  v->type = VAR_INT;
  v->value.i = i;
}

#define EVAL_EXPR(EXPR) wl = setup_interpreter_wl(); \
  env = setup_interpreter_env(); \
  ast = parser_parse(wl, EXPR); \
  var = evaluate(env, wl, ast)

#define EVAL_CLEANUP shutdown_interpreter_test(wl, ast)

#define EVAL_INT(EXPR,EXPECTED) EVAL_EXPR(EXPR); \
  assert_seni_var_i32(var, VAR_INT, EXPECTED); \
  EVAL_CLEANUP

#define EVAL_FLOAT(EXPR,EXPECTED) EVAL_EXPR(EXPR); \
  assert_seni_var_f32(var, VAR_FLOAT, EXPECTED); \
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
  EVAL_NAME("+", 0 + KEYWORD_START);
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

int main(void)
{
  UNITY_BEGIN();
  RUN_TEST(test_mathutil);
  RUN_TEST(test_lang_parser);
  RUN_TEST(test_lang_env);
  RUN_TEST(test_lang_interpret_basic);
  RUN_TEST(test_lang_interpret_math);
  RUN_TEST(test_lang_interpret_comparison);
  RUN_TEST(test_lang_interpret_define);
  RUN_TEST(test_lang_interpret_function);
  return UNITY_END();
}
