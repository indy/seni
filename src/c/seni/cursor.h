#pragma once

#include "types.h"

struct sen_cursor {
  char* buffer;
  i32   buffer_size;

  char* at;
  i32   space_remaining;
};

sen_cursor* cursor_allocate(char* buffer, i32 buffer_size);
void        cursor_free(sen_cursor* cursor);

void cursor_pretty_print(sen_cursor* cursor);

void cursor_sprintf(sen_cursor* cursor, char const* fmt, ...);
bool cursor_strncpy(sen_cursor* cursor, char* c, i32 len);

void cursor_reset(sen_cursor* cursor);
void cursor_clear(sen_cursor* cursor);
void cursor_write_null(sen_cursor* cursor);

bool cursor_forward(sen_cursor* cursor, i32 amount);
bool cursor_compare(sen_cursor* cursor, char* c, i32 len);

void cursor_eat_space(sen_cursor* cursor);
void cursor_eat_nonspace(sen_cursor* cursor);

bool cursor_eat_text(sen_cursor* cursor, char* c);
i32  cursor_eat_i32(sen_cursor* cursor);
f32  cursor_eat_f32(sen_cursor* cursor);
u64  cursor_eat_u64(sen_cursor* cursor);
