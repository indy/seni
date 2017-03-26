/*
  Runs tests using the native compiler
*/
#include "unity/unity.h"
#include "seni.h"

#include "seni_lang_lexer.h"
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

seni_token *assert_lexer_token_raw(seni_token *token, seni_token_type type)
{
  TEST_ASSERT_EQUAL(type, token->type);
  return token->next;
}

seni_token *assert_lexer_token_i32(seni_token *token, seni_token_type type, i32 val)
{
  TEST_ASSERT_EQUAL(type, token->type);
  TEST_ASSERT_EQUAL(val, token->i32_value);
  return token->next;
}

seni_token *assert_lexer_token_f32(seni_token *token, seni_token_type type, f32 val)
{
  TEST_ASSERT_EQUAL(type, token->type);
  TEST_ASSERT_EQUAL_FLOAT(val, token->f32_value);
  return token->next;
}

seni_token *assert_lexer_token_str(seni_token *token, seni_token_type type, char *val)
{
  TEST_ASSERT_EQUAL(type, token->type);
  TEST_ASSERT_EQUAL_STRING(val, token->str_value);
  return token->next;
}

seni_token *assert_lexer_token_chr(seni_token *token, seni_token_type type, char val)
{
  TEST_ASSERT_EQUAL(type, token->type);
  TEST_ASSERT_EQUAL(val, token->chr_value);
  return token->next;
}

void test_lang_lexer(void)
{
  seni_token *tokens = lexer_tokenise("'(runall \"shabba\") ; woohoo");
  seni_token *iter = tokens;

  iter = assert_lexer_token_raw(iter, TOK_QUOTE_ABBREVIATION);
  iter = assert_lexer_token_raw(iter, TOK_LIST_START);
  iter = assert_lexer_token_str(iter, TOK_NAME, "runall");
  iter = assert_lexer_token_str(iter, TOK_WHITESPACE, " ");
  iter = assert_lexer_token_str(iter, TOK_STRING, "shabba");
  iter = assert_lexer_token_raw(iter, TOK_LIST_END);
  iter = assert_lexer_token_str(iter, TOK_WHITESPACE, " ");
  iter = assert_lexer_token_str(iter, TOK_COMMENT, "; woohoo");

  lexer_free_tokens(tokens);

  tokens = lexer_tokenise("(fun i: 42 f: 12.34)");
  iter = tokens;

  iter = assert_lexer_token_raw(iter, TOK_LIST_START);
  iter = assert_lexer_token_str(iter, TOK_NAME, "fun");
  iter = assert_lexer_token_str(iter, TOK_WHITESPACE, " ");
  iter = assert_lexer_token_str(iter, TOK_LABEL, "i");
  iter = assert_lexer_token_str(iter, TOK_WHITESPACE, " ");
  iter = assert_lexer_token_i32(iter, TOK_INT, 42);
  iter = assert_lexer_token_str(iter, TOK_WHITESPACE, " ");
  iter = assert_lexer_token_str(iter, TOK_LABEL, "f");
  iter = assert_lexer_token_str(iter, TOK_WHITESPACE, " ");
  iter = assert_lexer_token_f32(iter, TOK_FLOAT, 12.34f);
  iter = assert_lexer_token_raw(iter, TOK_LIST_END);

  lexer_free_tokens(tokens);


  tokens = lexer_tokenise("(a {[1 2]})");
  iter = tokens;

  iter = assert_lexer_token_raw(iter, TOK_LIST_START);
  iter = assert_lexer_token_str(iter, TOK_NAME, "a");
  iter = assert_lexer_token_str(iter, TOK_WHITESPACE, " ");
  iter = assert_lexer_token_raw(iter, TOK_ALTERABLE_START);
  iter = assert_lexer_token_raw(iter, TOK_VECTOR_START);
  iter = assert_lexer_token_i32(iter, TOK_INT, 1);
  iter = assert_lexer_token_str(iter, TOK_WHITESPACE, " ");
  iter = assert_lexer_token_i32(iter, TOK_INT, 2);
  iter = assert_lexer_token_raw(iter, TOK_VECTOR_END);
  iter = assert_lexer_token_raw(iter, TOK_ALTERABLE_END);
  iter = assert_lexer_token_raw(iter, TOK_LIST_END);
  
  lexer_free_tokens(tokens);

}

seni_node *assert_parser_node_raw(seni_node *node, seni_node_type type)
{
  TEST_ASSERT_EQUAL(type, node->type);
  return node->next;
}

seni_node *assert_parser_node_i32(seni_node *node, seni_node_type type, i32 val)
{
  TEST_ASSERT_EQUAL(type, node->type);
  TEST_ASSERT_EQUAL(val, node->i32_value);
  return node->next;
}

seni_node *assert_parser_node_f32(seni_node *node, seni_node_type type, f32 val)
{
  TEST_ASSERT_EQUAL(type, node->type);
  TEST_ASSERT_EQUAL_FLOAT(val, node->f32_value);
  return node->next;
}

seni_node *assert_parser_node_str(seni_node *node, seni_node_type type, char *val)
{
  TEST_ASSERT_EQUAL(type, node->type);
  TEST_ASSERT_EQUAL_STRING(val, node->str_value);
  return node->next;
}

seni_node *assert_parser_node_chr(seni_node *node, seni_node_type type, char val)
{
  TEST_ASSERT_EQUAL(type, node->type);
  TEST_ASSERT_EQUAL(val, node->chr_value);
  return node->next;
}

void test_lang_parser(void)
{
  seni_token *tokens = lexer_tokenise("(add 1 2)");
  seni_node *nodes = parser_parse(tokens);

  assert_parser_node_raw(nodes, NODE_LIST);

  seni_node *iter = nodes->children;
  iter = assert_parser_node_raw(iter, NODE_NAME);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 1);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_i32(iter, NODE_INT, 2);
  TEST_ASSERT_NULL(iter);

  parser_free_nodes(nodes);
}

int main(void)
{
  UNITY_BEGIN();
  RUN_TEST(test_mathutil);
  RUN_TEST(test_interp);
  RUN_TEST(test_uthash);
  RUN_TEST(test_lang_lexer);
  RUN_TEST(test_lang_parser);
  return UNITY_END();
}

