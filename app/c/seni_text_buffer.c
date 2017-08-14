#include "seni_text_buffer.h"
#include "seni_config.h"
#include "seni_printf.h"
#include "seni_strtof.h"

#include <string.h>
#include <stdlib.h>
#include <inttypes.h>
#include <stdarg.h>

seni_text_buffer *text_buffer_construct(char *buffer, i32 buffer_size)
{
  seni_text_buffer *text_buffer = (seni_text_buffer *)calloc(1, sizeof(seni_text_buffer));

  text_buffer->buffer = buffer;
  text_buffer->cursor = buffer;

  text_buffer->buffer_size = buffer_size;
  text_buffer->current_size = buffer_size;
  
  return text_buffer;
}

void text_buffer_free(seni_text_buffer *text_buffer)
{
  free(text_buffer);
}

void text_buffer_pretty_print(seni_text_buffer *text_buffer)
{
  SENI_PRINT("[buffer length: %d] %s", text_buffer->buffer_size, text_buffer->buffer);
  SENI_PRINT("[cursor length: %d] %s", text_buffer->current_size, text_buffer->cursor);
}


void text_buffer_sprintf(seni_text_buffer *text_buffer, char const * fmt, ... )
{
  va_list va;
  va_start(va, fmt);
  int len = seni_vsprintf(text_buffer->cursor, text_buffer->current_size, fmt, va);
  va_end(va);

  text_buffer->current_size -= len;
  if (text_buffer->current_size > 0) {
    text_buffer->cursor += len;
  } else {
    SENI_ERROR("seni_text_buffer: buffer is full");
  }

}

void text_buffer_reset(seni_text_buffer *text_buffer)
{
  text_buffer->cursor = text_buffer->buffer;
  text_buffer->current_size = text_buffer->buffer_size;
}

void text_buffer_clear(seni_text_buffer *text_buffer)
{
  char *c = text_buffer->buffer;
  for(i32 i = 0; i < text_buffer->buffer_size; i++) {
    *c++ = 0;
  }
  text_buffer_reset(text_buffer);
}

bool text_buffer_forward(seni_text_buffer *text_buffer, i32 amount)
{
  if (text_buffer->current_size - amount < 0) {
    return false;
  }

  text_buffer->cursor += amount;
  text_buffer->current_size -= amount;

  return true;
}

bool text_buffer_compare(seni_text_buffer *text_buffer, char *c, i32 len)
{
  return (strncmp(text_buffer->cursor, c, len) == 0);
}

void text_buffer_eat_space(seni_text_buffer *text_buffer)
{
  while (*text_buffer->cursor && *text_buffer->cursor == ' ') {
    text_buffer->cursor++;
    text_buffer->current_size--;
  }
}

void text_buffer_eat_nonspace(seni_text_buffer *text_buffer)
{
  while (*text_buffer->cursor && *text_buffer->cursor != ' ') {
    text_buffer->cursor++;
    text_buffer->current_size--;
  }
}

bool text_buffer_eat_text(seni_text_buffer *text_buffer, char *c)
{
  i32 len = strlen(c);
  
  if (text_buffer_compare(text_buffer, c, len)) {
    text_buffer_forward(text_buffer, len);
    return true;
  }

  return false;
}

i32 text_buffer_eat_i32(seni_text_buffer *text_buffer)
{
  text_buffer_eat_space(text_buffer);
  i32 i = (i32)atoi(text_buffer->cursor);
  text_buffer_eat_nonspace(text_buffer);

  return i;
}

f32 text_buffer_eat_f32(seni_text_buffer *text_buffer)
{
  char *end_ptr;
  
  text_buffer_eat_space(text_buffer);

  f32 f = seni_strtof(text_buffer->cursor, &end_ptr);
  i32 len = end_ptr - text_buffer->cursor;

  text_buffer_forward(text_buffer, len);
  text_buffer_eat_nonspace(text_buffer);

  return f;
}

u64 text_buffer_eat_u64(seni_text_buffer *text_buffer)
{
  text_buffer_eat_space(text_buffer);
  u64 l = (u64)atol(text_buffer->cursor);
  text_buffer_eat_nonspace(text_buffer);

  return l;
}
