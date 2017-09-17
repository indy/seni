#include "seni_multistring_buffer.h"
#include "seni_config.h"

#include <stdlib.h>

seni_multistring_buffer *multistring_buffer_allocate(i32 buffer_size)
{
  seni_multistring_buffer *mb = (seni_multistring_buffer *)calloc(1, sizeof(seni_multistring_buffer));

  mb->buffer_size = buffer_size;
  mb->buffer = (char *)calloc(buffer_size, sizeof(char));

  mb->cursor = mb->buffer;
  
  return mb;
}

void multistring_buffer_free(seni_multistring_buffer *multistring_buffer)
{
  free(multistring_buffer->buffer);
  free(multistring_buffer);
}

void multistring_buffer_reset(seni_multistring_buffer *multistring_buffer)
{
  multistring_buffer->cursor = multistring_buffer->buffer;
}

bool multistring_add(seni_multistring_buffer *mb, seni_string_ref *string_ref, char *string, i32 len)
{
  // string_ref already allocated

  if ((mb->cursor + len + 1) > (mb->buffer + mb->buffer_size)) {
    SENI_ERROR("not enough capacity in multistring buffer");
    return false;
  }

  // update the string_ref
  string_ref->c = mb->cursor;
  string_ref->len = len;

  // copy to the multistring_buffer;
  for (i32 i = 0; i < len; i++) {
    *(mb->cursor)++ = *string++;
  }
  *(mb->cursor)++ = '\0';
  
  return true;
}
