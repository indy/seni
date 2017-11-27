#include "mathutil.h"

#include <math.h>

i32 floor_f32(f32 a) {
  i32 ai = (i32)a;
  return (a < ai) ? ai - 1 : ai;
}

i32 max_i32(i32 a, i32 b) { return a > b ? a : b; }

i32 min_i32(i32 a, i32 b) { return a < b ? a : b; }

f32 map_linear(f32 x) { return x; }

f32 map_quick_ease(f32 x) {
  f32 x2 = x * x;
  f32 x3 = x * x * x;

  return (3 * x2) - (2 * x3);
}

f32 map_slow_ease_in(f32 x) {
  f32 s = (f32)sin(x * PI_BY_2);
  return s * s * s * s;
}

f32 map_slow_ease_in_ease_out(f32 x) { return x - ((f32)sin(x * TAU) / TAU); }

// TODO: bezierCoordinates
// TODO: quadraticCoordinates

// stb.h translations
f32 smooth_step(f32 t) { return (3 - 2 * t) * (t * t); }

f32 cubic_bezier_1d(f32 t, f32 p0, f32 p1, f32 p2, f32 p3) {
  f32 it = 1 - t;
  return it * it * it * p0 + 3 * it * it * t * p1 + 3 * it * t * t * p2 + t * t * t * p3;
}

f64 linear_remap(f64 x, f64 x_min, f64 x_max, f64 out_min, f64 out_max) {
  return lerp(unlerp(x, x_min, x_max), out_min, out_max);
}
// end of stb.h translations

f32 mc_m(f32 xa, f32 ya, f32 xb, f32 yb) { return (ya - yb) / (xa - xb); }

f32 mc_c(f32 xa, f32 ya, f32 m) { return ya - (m * xa); }

f32 length_v2(f32 x, f32 y) { return (f32)sqrt((x * x) + (y * y)); }

f32 distance_v2(f32 ax, f32 ay, f32 bx, f32 by) {
  f32 xdiff = ax - bx;
  f32 ydiff = ay - by;

  f32 dist = length_v2(xdiff, ydiff);

  return dist;
}

void normalize(f32 *outx, f32 *outy, f32 x, f32 y) {
  f32 len = length_v2(x, y);

  *outx = x / len;
  *outy = y / len;
}

void normal(f32 *outx, f32 *outy, f32 x1, f32 y1, f32 x2, f32 y2) {
  f32 dx = x2 - x1;
  f32 dy = y2 - y1;

  normalize(outx, outy, -dy, dx);
}

void opposite_normal(f32 *outx, f32 *outy, f32 x, f32 y) {
  *outx = -x;
  *outy = -y;
}

f32 quadratic_point(f32 a, f32 b, f32 c, f32 t) {
  f32 r = ((b - a) - 0.5f * (c - a)) / (0.5f * (0.5f - 1));
  f32 s = c - a - r;

  return (r * t * t) + (s * t) + a;
}

f32 bezier_point(f32 a, f32 b, f32 c, f32 d, f32 t) {
  f32 t1 = 1 - t;

  return (a * t1 * t1 * t1) + (3 * b * t * t1 * t1) + (3 * c * t * t * t1) + (d * t * t * t);
}

f32 bezier_tangent(f32 a, f32 b, f32 c, f32 d, f32 t) {
  return (3 * t * t * (-a + 3 * b - 3 * c + d) + 6 * t * (a - 2 * b + c) + 3 * (-a + b));
}
