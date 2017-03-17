/*
  Runs tests using the native compiler
*/

#include <stdio.h>
#include "unity/unity.h"
#include "seni.h"

void test_monkey(void)
{
  TEST_ASSERT_EQUAL(4, 4);
  TEST_ASSERT_EQUAL(6, 6);
}

void test_monkey2(void)
{
  TEST_ASSERT_EQUAL(4, 4);
  TEST_ASSERT_EQUAL(6, 6);
}

int main(void)
{
  UNITY_BEGIN();
  RUN_TEST(test_monkey);
  RUN_TEST(test_monkey2);
  return UNITY_END();
}
