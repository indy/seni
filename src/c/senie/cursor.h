#pragma once

#include "types.h"

struct senie_cursor {
  char* buffer;
  i32   buffer_size;

  char* at;
  i32   space_remaining;
};

senie_cursor* cursor_allocate(char* buffer, i32 buffer_size);
void          cursor_free(senie_cursor* cursor);

void cursor_pretty_print(senie_cursor* cursor);

void cursor_sprintf(senie_cursor* cursor, char const* fmt, ...);
bool cursor_strncpy(senie_cursor* cursor, char* c, i32 len);

void cursor_reset(senie_cursor* cursor);
void cursor_clear(senie_cursor* cursor);
void cursor_write_null(senie_cursor* cursor);

bool cursor_forward(senie_cursor* cursor, i32 amount);
bool cursor_compare(senie_cursor* cursor, char* c, i32 len);

void cursor_eat_space(senie_cursor* cursor);
void cursor_eat_nonspace(senie_cursor* cursor);

bool cursor_eat_text(senie_cursor* cursor, char* c);
i32  cursor_eat_i32(senie_cursor* cursor);
f32  cursor_eat_f32(senie_cursor* cursor);
u64  cursor_eat_u64(senie_cursor* cursor);
