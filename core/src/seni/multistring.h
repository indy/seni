#pragma once

#include "types.h"

struct sen_multistring {
  i32 buffer_size;

  char* buffer;
  char* cursor;
};

sen_multistring* multistring_allocate(i32 buffer_size);
void             multistring_free(sen_multistring* multistring);
void             multistring_reset(sen_multistring* multistring);
bool multistring_add(sen_multistring* mb, sen_string_ref* string_ref, char* string, i32 len);

struct sen_string_ref {
  char* c; // this is a pointer into sen_multistring->buffer
  i32   len;
};
