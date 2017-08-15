#pragma once

#include "seni_types.h"

struct seni_text_buffer {
  char *buffer;
  i32 buffer_size;

  char *cursor;
  i32 current_size;
  
};

seni_text_buffer *text_buffer_allocate(char *buffer, i32 buffer_size);
void text_buffer_free(seni_text_buffer *text_buffer);

void text_buffer_pretty_print(seni_text_buffer *text_buffer);

void text_buffer_sprintf(seni_text_buffer *text_buffer, char const * fmt, ... );

void text_buffer_reset(seni_text_buffer *text_buffer);
void text_buffer_clear(seni_text_buffer *text_buffer);
void text_buffer_write_null(seni_text_buffer *text_buffer);

bool text_buffer_forward(seni_text_buffer *text_buffer, i32 amount);
bool text_buffer_compare(seni_text_buffer *text_buffer, char *c, i32 len);

void text_buffer_eat_space(seni_text_buffer *text_buffer);
void text_buffer_eat_nonspace(seni_text_buffer *text_buffer);

bool text_buffer_eat_text(seni_text_buffer *text_buffer, char *c);
i32 text_buffer_eat_i32(seni_text_buffer *text_buffer);
f32 text_buffer_eat_f32(seni_text_buffer *text_buffer);
u64 text_buffer_eat_u64(seni_text_buffer *text_buffer);
