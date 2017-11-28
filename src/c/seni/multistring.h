#pragma once

#include "types.h"

struct seni_multistring {
  i32 buffer_size;

  char* buffer;
  char* cursor;
};

seni_multistring* multistring_allocate(i32 buffer_size);
void              multistring_free(seni_multistring* multistring);
void              multistring_reset(seni_multistring* multistring);
bool multistring_add(seni_multistring* mb, seni_string_ref* string_ref, char* string, i32 len);

struct seni_string_ref {
  char* c; // this is a pointer into seni_multistring->buffer
  i32   len;
};
