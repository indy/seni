#ifndef SENI_SHAPES_H
#define SENI_SHAPES_H

#include "seni_types.h"
#include "seni_render_packet.h"
#include "seni_matrix.h"
#include "seni_colour.h"
#include "seni_lang.h"
  
void seni_shapes_init_globals();

void render_line(seni_render_data *render_data,
                 seni_matrix *matrix,
                 f32 from_x, f32 from_y, f32 to_x, f32 to_y,
                 f32 width,
                 seni_colour *colour);

void render_rect(seni_render_data *render_data,
                 seni_matrix *matrix,
                 f32 x, f32 y,
                 f32 width, f32 height,
                 seni_colour *colour);

void render_circle(seni_render_data *render_data,
                   seni_matrix *matrix,
                   f32 x, f32 y,
                   f32 width, f32 height,
                   seni_colour *colour,
                   i32 tessellation);

void render_circle_slice(seni_render_data *render_data,
                         seni_matrix *matrix,
                         f32 x, f32 y,
                         f32 width, f32 height,
                         seni_colour *colour,
                         i32 tessellation,
                         f32 angle_start, f32 angle_end,
                         f32 inner_width, f32 inner_height);

void render_poly(seni_render_data *render_data,
                 seni_matrix *matrix,
                 seni_var *coords,
                 seni_var *colours);

void render_quadratic(seni_render_data *render_data,
                      seni_matrix *matrix,
                      f32 *coords,
                      f32 line_width_start, f32 line_width_end, i32 line_width_mapping,
                      f32 t_start, f32 t_end,
                      seni_colour *colour,
                      i32 tessellation,
                      i32 brush, i32 brush_subtype);

void render_bezier(seni_render_data *render_data,
                   seni_matrix *matrix,
                   f32 *coords,
                   f32 line_width_start, f32 line_width_end, i32 line_width_mapping,
                   f32 t_start, f32 t_end,
                   seni_colour *colour,
                   i32 tessellation,
                   i32 brush, i32 brush_subtype);

void render_bezier_bulging(seni_render_data *render_data,
                           seni_matrix *matrix,
                           f32 *coords,
                           f32 line_width,
                           f32 t_start, f32 t_end,
                           seni_colour *colour,
                           i32 tessellation,
                           i32 brush, i32 brush_subtype);

void render_stroked_bezier(seni_render_data *render_data,
                           seni_matrix *matrix,
                           f32 *coords,
                           seni_colour *colour, i32 tessellation,
                           f32 stroke_line_width_start, f32 stroke_line_width_end, f32 stroke_noise,
                           i32 stroke_tessellation, f32 colour_volatility, f32 seed,
                           i32 line_width_mapping, i32 brush, i32 brush_subtype);

void render_stroked_bezier_rect(seni_render_data *render_data,
                                seni_matrix *matrix,
                                f32 *position, f32 width, f32 height, f32 volatility, f32 overlap, f32 iterations,
                                f32 seed,
                                i32 tessellation, i32 stroke_tessellation, f32 stroke_noise,
                                seni_colour *colour, f32 colour_volatility,
                                i32 brush, i32 brush_subtype);
#endif
