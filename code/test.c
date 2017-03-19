/*
  Runs tests using the native compiler
*/
#include "unity/unity.h"
#include "seni.h"

void test_mathutil(void)
{
  TEST_ASSERT_EQUAL_FLOAT(1.5f, deg2rad(rad2deg(1.5f)));
  TEST_ASSERT_EQUAL_FLOAT(0.44444f, mc_m(1.0f, 1.0f, 10.0f, 5.0f));
  TEST_ASSERT_EQUAL_FLOAT(0.55556f, mc_c(1.0f, 1.0f, 0.444444f));
}

void test_interp(void)
{
  TEST_ASSERT_EQUAL_FLOAT(1.5f, map_linear(1.5f));
}

int main(void)
{
  UNITY_BEGIN();
  RUN_TEST(test_mathutil);
  RUN_TEST(test_interp);
  return UNITY_END();
}
