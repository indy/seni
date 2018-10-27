#include "multistring.h"
#include "config.h"

#include <stdlib.h>

sen_multistring* multistring_allocate(i32 buffer_size) {
  sen_multistring* mb = (sen_multistring*)calloc(1, sizeof(sen_multistring));

  mb->buffer_size = buffer_size;
  mb->buffer      = (char*)calloc(buffer_size, sizeof(char));

  mb->cursor = mb->buffer;

  return mb;
}

void multistring_free(sen_multistring* multistring) {
  free(multistring->buffer);
  free(multistring);
}

void multistring_reset(sen_multistring* multistring) {
  multistring->cursor = multistring->buffer;
}

bool multistring_add(sen_multistring* mb, sen_string_ref* string_ref, char* string, i32 len) {
  // string_ref already allocated

  if ((mb->cursor + len + 1) > (mb->buffer + mb->buffer_size)) {
    SEN_ERROR("not enough capacity in multistring buffer");
    return false;
  }

  // update the string_ref
  string_ref->c   = mb->cursor;
  string_ref->len = len;

  // copy to the multistring;
  for (i32 i = 0; i < len; i++) {
    *(mb->cursor)++ = *string++;
  }
  *(mb->cursor)++ = '\0';

  return true;
}
