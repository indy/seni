#pragma once

#include "config.h"

#define REGISTER_KEYWORD(_, x) x,
typedef enum {
  INAME_DONT_USE_THIS =
      KEYWORD_START - 1, // just a way of making sure inames start at the correct value
#include "keywords.h"
  INAME_NUMBER_OF_KNOWN_WORDS
} seni_keyword_iname;
#undef REGISTER_KEYWORD
// todo: have a check at startup to assert that INAME_NUMBER_OF_KNOWN_WORDS <
// NATIVE_START
