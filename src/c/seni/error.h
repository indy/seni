#pragma once

typedef enum {
  NONE = 0,
  ERROR_PARSE,
  ERROR_PARSE_NULL_INPUT,
  ERROR_PARSE_NON_MUTABLE_NODE,
  ERROR_PARSE_END_OF_INPUT,
  ERROR_PARSE_EXPECTED_END_OF_LIST,
  ERROR_PARSE_EXPECTED_END_OF_VECTOR,
  ERROR_NULL_NODE,
  ERROR_WLUT_ADD_FAILED,
  ERROR_WLUT_LOOKUP_FAILED
} sen_error;
