#pragma once

#include "types.h"

void shapes_subsystem_startup();

void render_line(senie_render_data* render_data,
                 senie_matrix*      matrix,
                 f32                from_x,
                 f32                from_y,
                 f32                to_x,
                 f32                to_y,
                 f32                width,
                 senie_colour*      colour);

void render_rect(senie_render_data* render_data,
                 senie_matrix*      matrix,
                 f32                x,
                 f32                y,
                 f32                width,
                 f32                height,
                 senie_colour*      colour);

void render_circle(senie_render_data* render_data,
                   senie_matrix*      matrix,
                   f32                x,
                   f32                y,
                   f32                width,
                   f32                height,
                   senie_colour*      colour,
                   i32                tessellation);

void render_circle_slice(senie_render_data* render_data,
                         senie_matrix*      matrix,
                         f32                x,
                         f32                y,
                         f32                width,
                         f32                height,
                         senie_colour*      colour,
                         i32                tessellation,
                         f32                angle_start,
                         f32                angle_end,
                         f32                inner_width,
                         f32                inner_height);

void render_poly(senie_render_data* render_data,
                 senie_matrix*      matrix,
                 senie_var*         coords,
                 senie_var*         colours);

void render_quadratic(senie_render_data* render_data,
                      senie_matrix*      matrix,
                      f32*               coords,
                      f32                line_width_start,
                      f32                line_width_end,
                      i32                line_width_mapping,
                      f32                t_start,
                      f32                t_end,
                      senie_colour*      colour,
                      i32                tessellation,
                      i32                brush,
                      i32                brush_subtype);

void render_bezier(senie_render_data* render_data,
                   senie_matrix*      matrix,
                   f32*               coords,
                   f32                line_width_start,
                   f32                line_width_end,
                   i32                line_width_mapping,
                   f32                t_start,
                   f32                t_end,
                   senie_colour*      colour,
                   i32                tessellation,
                   i32                brush,
                   i32                brush_subtype);

void render_bezier_bulging(senie_render_data* render_data,
                           senie_matrix*      matrix,
                           f32*               coords,
                           f32                line_width,
                           f32                t_start,
                           f32                t_end,
                           senie_colour*      colour,
                           i32                tessellation,
                           i32                brush,
                           i32                brush_subtype);

void render_stroked_bezier(senie_render_data* render_data,
                           senie_matrix*      matrix,
                           f32*               coords,
                           senie_colour*      colour,
                           i32                tessellation,
                           f32                stroke_line_width_start,
                           f32                stroke_line_width_end,
                           f32                stroke_noise,
                           i32                stroke_tessellation,
                           f32                colour_volatility,
                           f32                seed,
                           i32                line_width_mapping,
                           i32                brush,
                           i32                brush_subtype);

void render_stroked_bezier_rect(senie_render_data* render_data,
                                senie_matrix*      matrix,
                                f32*               position,
                                f32                width,
                                f32                height,
                                f32                volatility,
                                f32                overlap,
                                f32                iterations,
                                f32                seed,
                                i32                tessellation,
                                i32                stroke_tessellation,
                                f32                stroke_noise,
                                senie_colour*      colour,
                                f32                colour_volatility,
                                i32                brush,
                                i32                brush_subtype);
