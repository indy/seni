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
  word_lut *wl = (word_lut *)calloc(1, sizeof(word_lut));

  nodes = parser_parse(wl, "hello");
  assert_parser_node_txt(nodes, NODE_NAME, "hello", wl);
  word_lookup_free_words(wl);
  parser_free_nodes(nodes);


  nodes = parser_parse(wl, "5");
  assert_parser_node_i32(nodes, NODE_INT, 5);
  word_lookup_free_words(wl);
  parser_free_nodes(nodes);

  nodes = parser_parse(wl, "(4)");
  assert_parser_node_raw(nodes, NODE_LIST);
  word_lookup_free_words(wl);
  parser_free_nodes(nodes);
  
  nodes = parser_parse(wl, "true");
  assert_parser_node_i32(nodes, NODE_BOOLEAN, true);
  word_lookup_free_words(wl);
  parser_free_nodes(nodes);
  
  nodes = parser_parse(wl, "false");
  assert_parser_node_i32(nodes, NODE_BOOLEAN, false);
  word_lookup_free_words(wl);
  parser_free_nodes(nodes);
  
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
  word_lookup_free_words(wl);
  parser_free_nodes(nodes);
  
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
  word_lookup_free_words(wl);
  parser_free_nodes(nodes);
  
  nodes = parser_parse(wl, ";[add 9 8 (foo)]");
  assert_parser_node_str(nodes, NODE_COMMENT, ";[add 9 8 (foo)]");
  TEST_ASSERT_NULL(nodes->next);
  word_lookup_free_words(wl);
  parser_free_nodes(nodes);
  
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
  word_lookup_free_words(wl);
  parser_free_nodes(nodes);
  
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
  word_lookup_free_words(wl);
  parser_free_nodes(nodes);
  
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
  word_lookup_free_words(wl);
  parser_free_nodes(nodes);
  
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
  word_lookup_free_words(wl);
  parser_free_nodes(nodes);
  
  free(wl);
}

void test_lang_env(void)
{
  seni_env *env, *env2;
  seni_var *var, *var2;

  env_allocate_pools();
  env = get_initial_env();

  var = add_var(env, 1);
  var->type = NODE_INT;
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

void assert_seni_var(seni_var *var, seni_node_type type)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, var->type, parser_node_type_name(var->type));
}

void assert_seni_var_i32(seni_var *var, seni_node_type type, i32 i)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, var->type, parser_node_type_name(var->type));
  TEST_ASSERT_EQUAL(i, var->value.i);
}

void assert_seni_var_f32(seni_var *var, seni_node_type type, f32 f)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, var->type, parser_node_type_name(var->type));
  TEST_ASSERT_EQUAL_FLOAT(f, var->value.f);
}

word_lut *setup_interpreter_wl()
{
  word_lut *wl = (word_lut *)calloc(1, sizeof(word_lut));
  word_lookup_add_reserved_words(wl);

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
  word_lookup_free_words(wl);
  word_lookup_free_reserved_words(wl);
  parser_free_nodes(ast);
  free(wl);
}

void add_binding_i32(word_lut *wl, seni_env *env, char *name, i32 i)
{
  // add a foo binding to env
  i32 name_index = word_lookup_or_add(wl, name, strlen(name));
  seni_var *v = add_var(env, name_index);
  v->type = NODE_INT;
  v->value.i = i;
}

void test_lang_interpreter(void)
{
  word_lut *wl = NULL;
  seni_env *env = NULL;

  {
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    seni_node *ast = parser_parse(wl, "42");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_i32(var, NODE_INT, 42);

    shutdown_interpreter_test(wl, ast);
  }

  {
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    seni_node *ast = parser_parse(wl, "12.34");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_f32(var, NODE_FLOAT, 12.34f);

    shutdown_interpreter_test(wl, ast);
  }

  {
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    seni_node *ast = parser_parse(wl, "true");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_i32(var, NODE_BOOLEAN, 1);

    shutdown_interpreter_test(wl, ast);
  }

  {
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    seni_node *ast = parser_parse(wl, "false");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_i32(var, NODE_BOOLEAN, 0);

    shutdown_interpreter_test(wl, ast);
  }  

  {
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    // add a foo binding to env
    add_binding_i32(wl, env, "foo", 31);

    seni_node *ast = parser_parse(wl, "foo");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_i32(var, NODE_INT, 31);

    shutdown_interpreter_test(wl, ast);
  }


  {
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    seni_node *ast = parser_parse(wl, "+");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_i32(var, NODE_NAME, 0 + RESERVED_WORD_START);

    shutdown_interpreter_test(wl, ast);
  }

  {
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    seni_node *ast = parser_parse(wl, "(+ 10 1)");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_i32(var, NODE_INT, 11);

    shutdown_interpreter_test(wl, ast);
  }

  { // convert result to float if any arg is a float
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    seni_node *ast = parser_parse(wl, "(+ 10.0 1)");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_f32(var, NODE_FLOAT, 11.0);

    shutdown_interpreter_test(wl, ast);
  }  

  {
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    seni_node *ast = parser_parse(wl, "(+ 10 1.0)");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_f32(var, NODE_FLOAT, 11.0);

    shutdown_interpreter_test(wl, ast);
  }
    
  {
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    seni_node *ast = parser_parse(wl, "(+ 3 4 5 6)");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_i32(var, NODE_INT, 18);

    shutdown_interpreter_test(wl, ast);
  }

  {
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    seni_node *ast = parser_parse(wl, "(+ (+ 1 2) (+ 3 4))");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_i32(var, NODE_INT, 10);

    shutdown_interpreter_test(wl, ast);
  }

  {
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    seni_node *ast = parser_parse(wl, "(+ (+ 1 2) (+ 3.0 4))");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_f32(var, NODE_FLOAT, 10.0);

    shutdown_interpreter_test(wl, ast);
  }  

  {
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    seni_node *ast = parser_parse(wl, "(- 100 20)");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_i32(var, NODE_INT, 80);

    shutdown_interpreter_test(wl, ast);
  }

  { // - with one arg = negation
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    seni_node *ast = parser_parse(wl, "(- 59)");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_i32(var, NODE_INT, -59);

    shutdown_interpreter_test(wl, ast);
  }

  {
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    seni_node *ast = parser_parse(wl, "(- 100.0 20)");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_f32(var, NODE_FLOAT, 80.0);

    shutdown_interpreter_test(wl, ast);
  }

  {
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    seni_node *ast = parser_parse(wl, "(- 100 20.0)");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_f32(var, NODE_FLOAT, 80.0);

    shutdown_interpreter_test(wl, ast);
  }  

  {
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    seni_node *ast = parser_parse(wl, "(- 100.0 20.0)");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_f32(var, NODE_FLOAT, 80.0);

    shutdown_interpreter_test(wl, ast);
  }  

  {
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    seni_node *ast = parser_parse(wl, "(* 6 5)");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_i32(var, NODE_INT, 30);

    shutdown_interpreter_test(wl, ast);
  }

  {
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    seni_node *ast = parser_parse(wl, "(/ 16.0 2.0)");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_f32(var, NODE_FLOAT, 8.0f);

    shutdown_interpreter_test(wl, ast);
  }

  {
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    seni_node *ast = parser_parse(wl, "(define num 10) (+ num num)");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_i32(var, NODE_INT, 20);

    shutdown_interpreter_test(wl, ast);
  }

  {
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    seni_node *ast = parser_parse(wl, "(define num 10.0) (+ num num)");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_f32(var, NODE_FLOAT, 20.0f);

    shutdown_interpreter_test(wl, ast);
  }

  {
    wl = setup_interpreter_wl();
    env = setup_interpreter_env();

    seni_node *ast = parser_parse(wl, "(define num (* 2 3.0)) (+ num num num)");
    seni_var *var = evaluate(env, wl, ast);
  
    assert_seni_var_f32(var, NODE_FLOAT, 18.0f);

    shutdown_interpreter_test(wl, ast);
  }

}

int main(void)
{
  UNITY_BEGIN();
#if 1
  RUN_TEST(test_mathutil);
  RUN_TEST(test_lang_parser);
  RUN_TEST(test_lang_env);
#endif  
  RUN_TEST(test_lang_interpreter);
  return UNITY_END();
}
