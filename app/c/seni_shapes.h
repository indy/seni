#ifndef SENI_SHAPES_H
#define SENI_SHAPES_H

#include "seni_types.h"
#include "seni_buffer.h"

void render_line(seni_buffer *buffer,
                 f32 from_x, f32 from_y, f32 to_x, f32 to_y,
                 f32 width,
                 rgba col);

void render_rect(seni_buffer *buffer,
                 f32 x, f32 y,
                 f32 width, f32 height,
                 rgba col);

void render_circle(seni_buffer *buffer,
                   f32 x, f32 y,
                   f32 width, f32 height,
                   rgba col,
                   i32 tessellation);


#endif
