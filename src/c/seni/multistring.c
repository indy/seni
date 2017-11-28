#include "multistring.h"
#include "config.h"

#include <stdlib.h>

seni_multistring* multistring_allocate(i32 buffer_size) {
  seni_multistring* mb = (seni_multistring*)calloc(1, sizeof(seni_multistring));

  mb->buffer_size = buffer_size;
  mb->buffer      = (char*)calloc(buffer_size, sizeof(char));

  mb->cursor = mb->buffer;

  return mb;
}

void multistring_free(seni_multistring* multistring) {
  free(multistring->buffer);
  free(multistring);
}

void multistring_reset(seni_multistring* multistring) { multistring->cursor = multistring->buffer; }

bool multistring_add(seni_multistring* mb, seni_string_ref* string_ref, char* string, i32 len) {
  // string_ref already allocated

  if ((mb->cursor + len + 1) > (mb->buffer + mb->buffer_size)) {
    SENI_ERROR("not enough capacity in multistring buffer");
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
