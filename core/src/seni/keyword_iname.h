#pragma once

#include "config.h"

#define REGISTER_KEYWORD(_, x) x,
typedef enum {
  // just a way of making sure inames start at the correct value
  INAME_DONT_USE_THIS = KEYWORD_START - 1,
#include "keywords.h"
  INAME_NUMBER_OF_KNOWN_WORDS
} sen_keyword_iname;
#undef REGISTER_KEYWORD
