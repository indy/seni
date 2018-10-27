#pragma once

#define RESULT_STRUCT(TYPE, NAME)                                       \
  struct sen_result_##NAME {                                            \
    TYPE      result;                                                   \
    sen_error error;                                                    \
  };                                                                    \
                                                                        \
  typedef struct sen_result_##NAME sen_result_##NAME;                   \
                                                                        \
  bool              is_result_##NAME##_error(sen_result_##NAME result); \
  bool              is_result_##NAME##_ok(sen_result_##NAME result);    \
  sen_result_##NAME result_##NAME##_error(sen_error error);             \
  sen_result_##NAME result_##NAME##_ok(TYPE val);

#define RESULT_STRUCT_FUNCTIONS(TYPE, NAME)                                                \
  bool is_result_##NAME##_error(sen_result_##NAME result) { return result.error != NONE; } \
  bool is_result_##NAME##_ok(sen_result_##NAME result) { return result.error == NONE; }    \
  sen_result_##NAME result_##NAME##_error(sen_error error) {                               \
    sen_result_##NAME result;                                                              \
    result.error = error;                                                                  \
    return result;                                                                         \
  }                                                                                        \
  sen_result_##NAME result_##NAME##_ok(TYPE val) {                                         \
    sen_result_##NAME result;                                                              \
    result.result = val;                                                                   \
    result.error  = NONE;                                                                  \
    return result;                                                                         \
  }

#define OPTION_STRUCT(TYPE, NAME)                                      \
  struct sen_option_##NAME {                                           \
    TYPE some;                                                         \
    bool has_some;                                                     \
  };                                                                   \
                                                                       \
  typedef struct sen_option_##NAME sen_option_##NAME;                  \
                                                                       \
  bool              is_option_##NAME##_some(sen_option_##NAME result); \
  bool              is_option_##NAME##_none(sen_option_##NAME result); \
  sen_option_##NAME option_##NAME##_some(TYPE val);                    \
  sen_option_##NAME option_##NAME##_none();

#define OPTION_STRUCT_FUNCTIONS(TYPE, NAME)                                           \
  bool is_option_##NAME##_some(sen_option_##NAME option) { return option.has_some; }  \
  bool is_option_##NAME##_none(sen_option_##NAME option) { return !option.has_some; } \
  sen_option_##NAME option_##NAME##_some(TYPE val) {                                  \
    sen_option_##NAME option;                                                         \
    option.some     = val;                                                            \
    option.has_some = true;                                                           \
    return option;                                                                    \
  }                                                                                   \
  sen_option_##NAME option_##NAME##_none() {                                          \
    sen_option_##NAME option;                                                         \
    option.has_some = false;                                                          \
    return option;                                                                    \
  }
