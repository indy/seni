#pragma once

#include "types.h"

struct senie_multistring {
  i32 buffer_size;

  char* buffer;
  char* cursor;
};

senie_multistring* multistring_allocate(i32 buffer_size);
void               multistring_free(senie_multistring* multistring);
void               multistring_reset(senie_multistring* multistring);
bool multistring_add(senie_multistring* mb, senie_string_ref* string_ref, char* string, i32 len);

struct senie_string_ref {
  char* c; // this is a pointer into senie_multistring->buffer
  i32   len;
};
