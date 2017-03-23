#ifndef SENI_MATHUTIL
#define SENI_MATHUTIL

#include "seni_types.h"
#include <math.h>

#define PI 3.14159265358979323846f
#define TAU 6.283185307179586f
#define PI_BY_2 1.5707963267948966f

#define deg2rad(a)  ((a)*(PI/180))
#define rad2deg(a)  ((a)*(180/PI))

#define lerp(t,a,b)        ( (a) + (t) * (f32) ((b)-(a)) )
#define unlerp(t,a,b)      ( ((t) - (a)) / (f32) ((b) - (a)) )
#define clamp(x,xmin,xmax) ((x) < (xmin) ? (xmin) : (x) > (xmax) ? (xmax) : (x))

f32 smoothstep(f32 t);
f32 cubic_bezier_1d(f32 t, f32 p0, f32 p1, f32 p2, f32 p3);
f64 linear_remap(f64 x, f64 x_min, f64 x_max, f64 out_min, f64 out_max);

f32 mc_m(f32 xa, f32 ya, f32 xb, f32 yb);
f32 mc_c(f32 xa, f32 ya, f32 m);

f32 length_v2(v2 v);
v2 sub_v2(v2 a, v2 b);
f32 distance_v2(v2 a, v2 b);

v2 normalize(v2 v);
v2 normal(f32 x1, f32 y1, f32 x2, f32 y2);
v2 opposite_normal(v2 n);

f32 quadraticPoint(f32 a, f32 b, f32 c, f32 t);
f32 bezierPoint(f32 a, f32 b, f32 c, f32 d, f32 t);
f32 bezierTangent(f32 a, f32 b, f32 c, f32 d, f32 t);

#endif /* SENI_MATHUTIL */
