/*
  Runs tests using the native compiler
*/
#include "unity/unity.h"
#include "seni.h"

#include "seni_lang_parser.h"
#include "seni_lang_env.h"

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

void test_interp(void)
{
  TEST_ASSERT_EQUAL_FLOAT(1.5f, map_linear(1.5f));
}

my_struct *users = NULL;    /* important! initialize to NULL */

void test_uthash(void)
{
  int user_id = 42;
  f32 fvalue = 99.88f;
  char* name = "hello";

  
  my_struct *s;

  s = malloc(sizeof(my_struct));
  s->id = user_id;
  s->ff = fvalue;
  strcpy(s->name, name);
  HASH_ADD_INT( users, id, s );  /* id: name of key field */

  my_struct *t;

  HASH_FIND_INT( users, &user_id, t );  /* t: output pointer */
  
  TEST_ASSERT_EQUAL(fvalue, t->ff);
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

seni_node *assert_parser_node_txt(seni_node *node, seni_node_type type, char *val, parser_info *parser_info)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, parser_node_type_name(node->type));

  char *c = parser_info->name_lookup[node->value.i];
  TEST_ASSERT_EQUAL_STRING(val, c);
  
  return node->next;
}

#define MAX_NAMES 64
char *name_lookup[MAX_NAMES];

void teardown_parser(parser_info *parser_info, seni_node *nodes)
{
  for( int i = 0; i < MAX_NAMES; i++) {
    if (parser_info->name_lookup[i]) {
      free(parser_info->name_lookup[i]);
    }
    parser_info->name_lookup[i] = 0;      
  }
  parser_info->name_lookup_count = 0;
  parser_free_nodes(nodes);
}

void test_lang_parser(void)
{
  seni_node *nodes, *iter, *iter2;
  parser_info *pi = (parser_info *)calloc(1, sizeof(parser_info));

  pi->name_lookup = name_lookup;
  pi->name_lookup_count = 0;
  pi->name_lookup_max = MAX_NAMES;
  for( int i = 0; i < MAX_NAMES; i++) {
    pi->name_lookup[i] = 0;      
  }

  pi = parser_parse(pi, "hello");
  nodes = pi->nodes;
  assert_parser_node_txt(nodes, NODE_NAME, "hello", pi);
  teardown_parser(pi, nodes);

  pi = parser_parse(pi, "5");
  nodes = pi->nodes;
  assert_parser_node_i32(nodes, NODE_INT, 5);
  teardown_parser(pi, nodes);

  pi = parser_parse(pi, "(4)");
  nodes = pi->nodes;
  assert_parser_node_raw(nodes, NODE_LIST);
  teardown_parser(pi, nodes);
  
  pi = parser_parse(pi, "true");
  nodes = pi->nodes;
  assert_parser_node_i32(nodes, NODE_BOOLEAN, true);
  teardown_parser(pi, nodes);

  pi = parser_parse(pi, "false");
  nodes = pi->nodes;
  assert_parser_node_i32(nodes, NODE_BOOLEAN, false);
  teardown_parser(pi, nodes);

  pi = parser_parse(pi, "(add 1 2)");
  nodes = pi->nodes;
  iter = nodes->children;
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->children;
  iter = assert_parser_node_txt(iter, NODE_NAME, "add", pi);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 1);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 2);
  TEST_ASSERT_NULL(iter);
  TEST_ASSERT_NULL(nodes->next);
  teardown_parser(pi, nodes);

  pi = parser_parse(pi, "[add 9 8 (foo)]");
  nodes = pi->nodes;
  assert_parser_node_raw(nodes, NODE_VECTOR);
  iter = nodes->children;
  iter = assert_parser_node_txt(iter, NODE_NAME, "add", pi);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 9);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 8);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_raw(iter, NODE_LIST);
  TEST_ASSERT_NULL(iter);
  TEST_ASSERT_NULL(nodes->next);
  teardown_parser(pi, nodes);
 
  pi = parser_parse(pi, ";[add 9 8 (foo)]");
  nodes = pi->nodes;
  assert_parser_node_str(nodes, NODE_COMMENT, ";[add 9 8 (foo)]");
  TEST_ASSERT_NULL(nodes->next);
  teardown_parser(pi, nodes);

  pi = parser_parse(pi, "'(runall \"shabba\") ; woohoo");
  nodes = pi->nodes;
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->children;
  iter = assert_parser_node_txt(iter, NODE_NAME, "quote", pi);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter2 = iter;
  iter = assert_parser_node_raw(iter, NODE_LIST);
  TEST_ASSERT_NULL(iter);
  iter = iter2->children;
  iter = assert_parser_node_txt(iter, NODE_NAME, "runall", pi);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_txt(iter, NODE_STRING, "shabba", pi);
  iter = nodes->next;
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_str(iter, NODE_COMMENT, "; woohoo");
  TEST_ASSERT_NULL(iter);
  teardown_parser(pi, nodes);

  pi = parser_parse(pi, "(fun i: 42 f: 12.34)");
  nodes = pi->nodes;
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->children;
  iter = assert_parser_node_txt(iter, NODE_NAME, "fun", pi);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_txt(iter, NODE_LABEL, "i", pi);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 42);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_txt(iter, NODE_LABEL, "f", pi);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_f32(iter, NODE_FLOAT, 12.34f);
  TEST_ASSERT_NULL(iter);
  TEST_ASSERT_NULL(nodes->next);
  teardown_parser(pi, nodes);

  pi = parser_parse(pi, "(a 1) (b 2)");
  nodes = pi->nodes;
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->children;
  iter = assert_parser_node_txt(iter, NODE_NAME, "a", pi);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 1);
  TEST_ASSERT_NULL(iter);
  iter = nodes->next;
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  assert_parser_node_raw(iter, NODE_LIST);
  iter = iter->children;
  iter = assert_parser_node_txt(iter, NODE_NAME, "b", pi);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 2);
  TEST_ASSERT_NULL(iter);
  teardown_parser(pi, nodes);

  pi = parser_parse(pi, "(a {[1 2]})");
  nodes = pi->nodes;
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->children;
  iter = assert_parser_node_txt(iter, NODE_NAME, "a", pi);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter2 = iter; // the vector
  iter = assert_parser_node_raw(iter, NODE_VECTOR);
  TEST_ASSERT_NULL(iter);
  TEST_ASSERT_EQUAL(test_true, iter2->alterable);
  TEST_ASSERT_NULL(nodes->next);
  teardown_parser(pi, nodes);

  free(pi);
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

int main(void)
{
  UNITY_BEGIN();
  RUN_TEST(test_mathutil);
  RUN_TEST(test_interp);
  RUN_TEST(test_uthash);
  RUN_TEST(test_lang_parser);
  RUN_TEST(test_lang_env);
  return UNITY_END();
}
