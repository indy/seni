/*
  Runs tests using the native compiler
*/
#include "unity/unity.h"
#include "seni.h"

#include "seni_lang_lexer.h"

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

char *pp_token(seni_token_type type)
{
  switch(type) {
  case TOK_UNKNOWN: return "TOK_UNKNOWN";
  case TOK_LIST_START: return "TOK_LIST_START";
  case TOK_LIST_END: return "TOK_LIST_END";
  case TOK_VECTOR_START: return "TOK_VECTOR_START";
  case TOK_VECTOR_END: return "TOK_VECTOR_END";
  case TOK_ALTERABLE_START: return "TOK_ALTERABLE_START";
  case TOK_ALTERABLE_END: return "TOK_ALTERABLE_END";
  case TOK_INT: return "TOK_INT";
  case TOK_FLOAT: return "TOK_FLOAT";
  case TOK_NAME: return "TOK_NAME";
  case TOK_STRING: return "TOK_STRING";
  case TOK_QUOTE_ABBREVIATION: return "TOK_QUOTE_ABBREVIATION";
  case TOK_LABEL: return "TOK_LABEL";
  case TOK_COMMENT: return "TOK_COMMENT";
  case TOK_WHITESPACE: return "TOK_WHITESPACE";
  };
  return "FOOK";
}

void test_lang_lexer(void)
{
  seni_token *tokens = tokenise("'(runall \"shabba\") ; woohoo");
  seni_token *iter = tokens;
  i32 i = 0;
  while (iter != NULL) {
    printf("%d: %s %d %.2f %s %c\n", i++, pp_token(iter->type), iter->i32_value, iter->f32_value, iter->str_value, iter->chr_value);
    iter = iter->next;
  }
  free_tokens(tokens);

  TEST_ASSERT_EQUAL(test_true, is_label("foo:"));
  TEST_ASSERT_EQUAL(test_false, is_label("foo"));
}

int main(void)
{
  UNITY_BEGIN();
  RUN_TEST(test_mathutil);
  RUN_TEST(test_interp);
  RUN_TEST(test_uthash);
  RUN_TEST(test_lang_lexer);
  return UNITY_END();
}

