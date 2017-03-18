#ifndef SENI_MATHUTIL
#define SENI_MATHUTIL

#include "seni_types.h"
#include <math.h>

#define PI 3.14159265358979323846f
#define TAU 6.283185307179586f
#define PI_BY_2 1.5707963267948966f

#define deg2rad(a)  ((a)*(PI/180))
#define rad2deg(a)  ((a)*(180/PI))

f32 mc_m(f32 xa, f32 ya, f32 xb, f32 yb);
f32 mc_c(f32 xa, f32 ya, f32 m);

v2 normal(f32 x1, f32 y1, f32 x2, f32 y2);
v2 opposite_normal(v2 n);

#endif /* SENI_MATHUTIL */
