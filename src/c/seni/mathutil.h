#pragma once

#include "types.h"

#define PI 3.14159265358979323846f
#define TAU 6.283185307179586f
#define PI_BY_2 1.5707963267948966f

#define deg_to_rad(a) ((a) * (PI / 180.0f))
#define rad_to_deg(a) ((a) * (180.0f / PI))

#define absf(x) ((x) < 0.0f ? -(x) : (x))
#define lerp(t, a, b) ((a) + (t) * (f32)((b) - (a)))
#define unlerp(t, a, b) (((t) - (a)) / (f32)((b) - (a)))
#define clamp(x, xmin, xmax) ((x) < (xmin) ? (xmin) : (x) > (xmax) ? (xmax) : (x))

i32 floor_f32(f32 a);

i32 max_i32(i32 a, i32 b);
i32 min_i32(i32 a, i32 b);

f32 map_linear(f32 x);
f32 map_quick_ease(f32 x);
f32 map_slow_ease_in(f32 x);
f32 map_slow_ease_in_ease_out(f32 x);

f32 smooth_step(f32 t);
f32 cubic_bezier_1d(f32 t, f32 p0, f32 p1, f32 p2, f32 p3);
f64 linear_remap(f64 x, f64 x_min, f64 x_max, f64 out_min, f64 out_max);

f32 mc_m(f32 xa, f32 ya, f32 xb, f32 yb);
f32 mc_c(f32 xa, f32 ya, f32 m);

f32 length_v2(f32 x, f32 y);
f32 distance_v2(f32 ax, f32 ay, f32 bx, f32 by);

void normalize(f32 *outx, f32 *outy, f32 x, f32 y);
void normal(f32 *outx, f32 *outy, f32 x1, f32 y1, f32 x2, f32 y2);
void opposite_normal(f32 *outx, f32 *outy, f32 x, f32 y);

f32 quadratic_point(f32 a, f32 b, f32 c, f32 t);
f32 bezier_point(f32 a, f32 b, f32 c, f32 d, f32 t);
f32 bezier_tangent(f32 a, f32 b, f32 c, f32 d, f32 t);
