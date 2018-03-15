#pragma once

#include "types.h"

void path_linear(senie_vm* vm,
                 i32       fn,
                 i32       steps,
                 f32       t_start,
                 f32       t_end,
                 f32       a_x,
                 f32       a_y,
                 f32       b_x,
                 f32       b_y);
void path_circle(senie_vm* vm,
                 i32       fn,
                 i32       steps,
                 f32       t_start,
                 f32       t_end,
                 f32       pos_x,
                 f32       pos_y,
                 f32       radius);
void path_spline(senie_vm* vm, i32 fn, i32 steps, f32 t_start, f32 t_end, f32* coords);
void path_bezier(senie_vm* vm, i32 fn, i32 steps, f32 t_start, f32 t_end, f32* coords);
