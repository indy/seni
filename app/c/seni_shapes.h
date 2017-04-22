#ifndef SENI_SHAPES_H
#define SENI_SHAPES_H

#include "seni_types.h"
#include "seni_buffer.h"

void render_rect(seni_buffer *buffer,
                 f32 x, f32 y,
                 f32 width, f32 height,
                 f32 r, f32 g, f32 b, f32 a);

void render_circle(seni_buffer *buffer,
                   f32 x, f32 y,
                   f32 width, f32 height,
                   f32 r, f32 g, f32 b, f32 a,
                   i32 tessellation);


#endif
