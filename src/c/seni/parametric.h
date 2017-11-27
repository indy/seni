#pragma once

#include "types.h"

f32 seni_parametric(f32  val,
                    f32  from_a,
                    f32  from_b,
                    f32  to_a,
                    f32  to_b,
                    i32  mapping,
                    bool clamping);

f32  seni_parametric_cos(f32 amplitude, f32 frequency, f32 t);
f32  seni_parametric_sin(f32 amplitude, f32 frequency, f32 t);
void seni_parametric_bezier(f32 *outx, f32 *outy, f32 *coords, f32 t);
void seni_parametric_bezier_tangent(f32 *outx, f32 *outy, f32 *coords, f32 t);
void seni_parametric_circle(f32 *outx, f32 *outy, f32 *position, f32 radius, f32 t);
