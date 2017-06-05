#ifndef SENI_SHAPES_H
#define SENI_SHAPES_H

#include "seni_types.h"
#include "seni_buffer.h"
#include "seni_matrix.h"
#include "seni_colour.h"

void render_line(seni_buffer *buffer,
                 seni_matrix *matrix,
                 f32 from_x, f32 from_y, f32 to_x, f32 to_y,
                 f32 width,
                 seni_colour *colour);

void render_rect(seni_buffer *buffer,
                 seni_matrix *matrix,
                 f32 x, f32 y,
                 f32 width, f32 height,
                 seni_colour *colour);

void render_circle(seni_buffer *buffer,
                   seni_matrix *matrix,
                   f32 x, f32 y,
                   f32 width, f32 height,
                   seni_colour *colour,
                   i32 tessellation);
void render_bezier(seni_buffer *buffer,
                   seni_matrix *matrix,
                   f32 *coords,
                   f32 line_width_start, f32 line_width_end, i32 line_width_mapping,
                   f32 t_start, f32 t_end,
                   seni_colour *colour,
                   i32 tessellation);

#endif
