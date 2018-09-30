#pragma once

#include "types.h"

void path_linear(sen_vm* vm, i32 fn, i32 steps, f32 t_start, f32 t_end, f32 a_x,
                 f32 a_y, f32 b_x, f32 b_y, i32 mapping);
void path_circle(sen_vm* vm, i32 fn, i32 steps, f32 t_start, f32 t_end,
                 f32 pos_x, f32 pos_y, f32 radius, i32 mapping);
void path_spline(sen_vm* vm, i32 fn, i32 steps, f32 t_start, f32 t_end,
                 f32* coords, i32 mapping);
void path_bezier(sen_vm* vm, i32 fn, i32 steps, f32 t_start, f32 t_end,
                 f32* coords, i32 mapping);
