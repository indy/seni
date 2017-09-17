#pragma once

#include "seni_types.h"

struct seni_multistring_buffer {
  i32 buffer_size;

  char *buffer;
  char *cursor;
};

seni_multistring_buffer *multistring_buffer_allocate(i32 buffer_size);
void multistring_buffer_free(seni_multistring_buffer *multistring_buffer);
void multistring_buffer_reset(seni_multistring_buffer *multistring_buffer);
bool multistring_add(seni_multistring_buffer *mb, seni_string_ref *string_ref, char *string, i32 len);

struct seni_string_ref {
  char *c;                      // this is a pointer into seni_multistring_buffer->buffer
  i32 len;
};
