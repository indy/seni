#pragma once

#define RESULT_STRUCT(TYPE, NAME) \
  struct sen_result_##NAME {      \
    TYPE      result;             \
    sen_error error;              \
  };                              \
  typedef struct sen_result_##NAME sen_result_##NAME;

#define RESULT_HELPER_FUNCTIONS(TYPE, NAME)                  \
  bool is_result_##NAME##_error(sen_result_##NAME result) {  \
    return result.error != NONE;                             \
  }                                                          \
  bool is_result_##NAME##_ok(sen_result_##NAME result) {     \
    return result.error == NONE;                             \
  }                                                          \
  sen_result_##NAME result_##NAME##_error(sen_error error) { \
    sen_result_##NAME result;                                \
    result.error = error;                                    \
    return result;                                           \
  }                                                          \
  sen_result_##NAME result_##NAME##_ok(TYPE val) {           \
    sen_result_##NAME result;                                \
    result.result = val;                                     \
    result.error  = NONE;                                    \
    return result;                                           \
  }
