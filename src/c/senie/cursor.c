#include "cursor.h"

#include "config.h"
#include "printf.h"
#include "strtof.h"

#include <inttypes.h>
#include <stdarg.h>
#include <stdlib.h>
#include <string.h>

senie_cursor* cursor_allocate(char* buffer, i32 buffer_size) {
  senie_cursor* cursor = (senie_cursor*)calloc(1, sizeof(senie_cursor));

  cursor->buffer = buffer;
  cursor->at     = buffer;

  cursor->buffer_size     = buffer_size;
  cursor->space_remaining = buffer_size;

  return cursor;
}

void cursor_free(senie_cursor* cursor) {
  cursor->buffer = NULL; // don't free the buffer memory, cursor doesn't own it
  cursor->at     = NULL;
  free(cursor);
}

void cursor_pretty_print(senie_cursor* cursor) {
  SENIE_PRINT("[buffer length: %d] %s", cursor->buffer_size, cursor->buffer);
  SENIE_PRINT("[at length: %d] %s", cursor->space_remaining, cursor->at);
}

void cursor_sprintf(senie_cursor* cursor, char const* fmt, ...) {
  va_list va;
  va_start(va, fmt);
  int len = senie_vsprintf(cursor->at, cursor->space_remaining, fmt, va);
  va_end(va);

  cursor->space_remaining -= len;
  if (cursor->space_remaining > 0) {
    cursor->at += len;
  } else {
    SENIE_ERROR("senie_cursor: buffer is full");
  }
}

bool cursor_strncpy(senie_cursor* cursor, char* c, i32 len) {
  if (cursor->space_remaining - len < 0) {
    return false;
  }

  strncpy(cursor->at, c, len);

  cursor->at += len;
  cursor->space_remaining -= len;

  return true;
}

void cursor_reset(senie_cursor* cursor) {
  cursor->at              = cursor->buffer;
  cursor->space_remaining = cursor->buffer_size;
}

void cursor_clear(senie_cursor* cursor) {
  char* c = cursor->buffer;
  for (i32 i = 0; i < cursor->buffer_size; i++) {
    *c++ = 0;
  }
  cursor_reset(cursor);
}

void cursor_write_null(senie_cursor* cursor) {
  if (cursor->space_remaining < cursor->buffer_size) {
    *(cursor->at++) = '\0';
    cursor->space_remaining--;
  }
}

bool cursor_forward(senie_cursor* cursor, i32 amount) {
  if (cursor->space_remaining - amount < 0) {
    return false;
  }

  cursor->at += amount;
  cursor->space_remaining -= amount;

  return true;
}

bool cursor_compare(senie_cursor* cursor, char* c, i32 len) {
  return (strncmp(cursor->at, c, len) == 0);
}

void cursor_eat_space(senie_cursor* cursor) {
  while (*cursor->at && *cursor->at == ' ') {
    cursor->at++;
    cursor->space_remaining--;
  }
}

void cursor_eat_nonspace(senie_cursor* cursor) {
  while (*cursor->at && *cursor->at != ' ') {
    cursor->at++;
    cursor->space_remaining--;
  }
}

bool cursor_eat_text(senie_cursor* cursor, char* c) {
  i32 len = (i32)strlen(c);

  if (cursor_compare(cursor, c, len)) {
    cursor_forward(cursor, len);
    return true;
  }

  return false;
}

i32 cursor_eat_i32(senie_cursor* cursor) {
  cursor_eat_space(cursor);
  i32 i = (i32)atoi(cursor->at);
  cursor_eat_nonspace(cursor);

  return i;
}

f32 cursor_eat_f32(senie_cursor* cursor) {
  char* end_ptr;

  cursor_eat_space(cursor);

  f32 f   = senie_strtof(cursor->at, &end_ptr);
  i32 len = (i32)(end_ptr - cursor->at);

  cursor_forward(cursor, len);
  cursor_eat_nonspace(cursor);

  return f;
}

u64 cursor_eat_u64(senie_cursor* cursor) {
  cursor_eat_space(cursor);
  u64 l = (u64)atol(cursor->at);
  cursor_eat_nonspace(cursor);

  return l;
}
