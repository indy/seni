#pragma once

#include "types.h"

f32  sen_parametric(f32 val, f32 from_a, f32 from_b, f32 to_a, f32 to_b, i32 mapping,
                    bool clamping);
f32  sen_parametric_scalar(f32 a, f32 b, i32 mapping, bool clamping, f32 t);
f32  sen_parametric_cos(f32 amplitude, f32 frequency, f32 t);
f32  sen_parametric_sin(f32 amplitude, f32 frequency, f32 t);
void sen_parametric_bezier(f32* outx, f32* outy, f32* coords, f32 t);
void sen_parametric_bezier_tangent(f32* outx, f32* outy, f32* coords, f32 t);
void sen_parametric_circle(f32* outx, f32* outy, f32* position, f32 radius, f32 t);
void sen_parametric_ray(f32* outx, f32* outy, f32* point, f32* direction, f32 t);
