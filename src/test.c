/*
  Runs tests using the native compiler
*/
#include "unity/unity.h"
#include "seni.h"

#include "seni_lang_parser.h"

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
  TEST_ASSERT_EQUAL(val, node->i32_value);
  return node->next;
}

seni_node *assert_parser_node_f32(seni_node *node, seni_node_type type, f32 val)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, parser_node_type_name(node->type));
  TEST_ASSERT_EQUAL_FLOAT(val, node->f32_value);
  return node->next;
}

seni_node *assert_parser_node_str(seni_node *node, seni_node_type type, char *val)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, parser_node_type_name(node->type));
  TEST_ASSERT_EQUAL_STRING(val, node->str_value);
  return node->next;
}

seni_node *assert_parser_node_chr(seni_node *node, seni_node_type type, char val)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, parser_node_type_name(node->type));
  TEST_ASSERT_EQUAL(val, node->chr_value);
  return node->next;
}

void test_lang_parser(void)
{
  seni_node *nodes, *iter, *iter2;

  nodes = parser_parse("(add 1 2)");
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->children;
  iter = assert_parser_node_str(iter, NODE_NAME, "add");
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 1);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 2);
  TEST_ASSERT_NULL(iter);
  TEST_ASSERT_NULL(nodes->next);
  parser_free_nodes(nodes);


  nodes = parser_parse("[add 9 8 (foo)]");
  assert_parser_node_raw(nodes, NODE_VECTOR);
  iter = nodes->children;
  iter = assert_parser_node_str(iter, NODE_NAME, "add");
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 9);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 8);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_raw(iter, NODE_LIST);
  TEST_ASSERT_NULL(iter);
  TEST_ASSERT_NULL(nodes->next);
  parser_free_nodes(nodes);


  nodes = parser_parse(";[add 9 8 (foo)]");
  assert_parser_node_str(nodes, NODE_COMMENT, ";[add 9 8 (foo)]");
  TEST_ASSERT_NULL(nodes->next);
  parser_free_nodes(nodes);


  nodes = parser_parse("'(runall \"shabba\") ; woohoo");
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->children;
  iter = assert_parser_node_str(iter, NODE_NAME, "quote");
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter2 = iter;
  iter = assert_parser_node_raw(iter, NODE_LIST);
  TEST_ASSERT_NULL(iter);
  iter = iter2->children;
  iter = assert_parser_node_str(iter, NODE_NAME, "runall");
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_str(iter, NODE_STRING, "shabba");
  iter = nodes->next;
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_str(iter, NODE_COMMENT, "; woohoo");
  TEST_ASSERT_NULL(iter);
  parser_free_nodes(nodes);


  nodes = parser_parse("(fun i: 42 f: 12.34)");
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->children;
  iter = assert_parser_node_str(iter, NODE_NAME, "fun");
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_str(iter, NODE_LABEL, "i");
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 42);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_str(iter, NODE_LABEL, "f");
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_f32(iter, NODE_FLOAT, 12.34f);
  TEST_ASSERT_NULL(iter);
  TEST_ASSERT_NULL(nodes->next);
  parser_free_nodes(nodes);


  nodes = parser_parse("(a 1) (b 2)");
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->children;
  iter = assert_parser_node_str(iter, NODE_NAME, "a");
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 1);
  TEST_ASSERT_NULL(iter);
  iter = nodes->next;
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  assert_parser_node_raw(iter, NODE_LIST);
  iter = iter->children;
  iter = assert_parser_node_str(iter, NODE_NAME, "b");
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 2);
  TEST_ASSERT_NULL(iter);
  parser_free_nodes(nodes);


  nodes = parser_parse("(a {[1 2]})");
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->children;
  iter = assert_parser_node_str(iter, NODE_NAME, "a");
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter2 = iter; // the vector
  iter = assert_parser_node_raw(iter, NODE_VECTOR);
  TEST_ASSERT_NULL(iter);
  TEST_ASSERT(test_true, iter2->alterable);
  TEST_ASSERT_NULL(nodes->next);
  parser_free_nodes(nodes);
}

int main(void)
{
  UNITY_BEGIN();
  RUN_TEST(test_mathutil);
  RUN_TEST(test_interp);
  RUN_TEST(test_uthash);
  RUN_TEST(test_lang_parser);
  return UNITY_END();
}
