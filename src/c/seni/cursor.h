#pragma once

#include "types.h"

struct seni_cursor {
  char* buffer;
  i32   buffer_size;

  char* at;
  i32   space_remaining;
};

seni_cursor* cursor_allocate(char* buffer, i32 buffer_size);
void         cursor_free(seni_cursor* cursor);

void cursor_pretty_print(seni_cursor* cursor);

void cursor_sprintf(seni_cursor* cursor, char const* fmt, ...);
bool cursor_strncpy(seni_cursor* cursor, char* c, i32 len);

void cursor_reset(seni_cursor* cursor);
void cursor_clear(seni_cursor* cursor);
void cursor_write_null(seni_cursor* cursor);

bool cursor_forward(seni_cursor* cursor, i32 amount);
bool cursor_compare(seni_cursor* cursor, char* c, i32 len);

void cursor_eat_space(seni_cursor* cursor);
void cursor_eat_nonspace(seni_cursor* cursor);

bool cursor_eat_text(seni_cursor* cursor, char* c);
i32  cursor_eat_i32(seni_cursor* cursor);
f32  cursor_eat_f32(seni_cursor* cursor);
u64  cursor_eat_u64(seni_cursor* cursor);
