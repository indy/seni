#include "seni_mathutil.h"

// TODO: bezierCoordinates
// TODO: quadraticCoordinates

// stb.h translations
f32 smooth_step(f32 t)
{
   return (3 - 2*t)*(t*t);
}

f32 cubic_bezier_1d(f32 t, f32 p0, f32 p1, f32 p2, f32 p3)
{
   f32 it = 1-t;
   return it*it*it*p0 + 3*it*it*t*p1 + 3*it*t*t*p2 + t*t*t*p3;
}

f64 linear_remap(f64 x, f64 x_min, f64 x_max, f64 out_min, f64 out_max)
{
   return lerp(unlerp(x,x_min,x_max),out_min,out_max);
}
// end of stb.h translations

f32 mc_m(f32 xa, f32 ya, f32 xb, f32 yb)
{
  return (ya - yb) / (xa - xb);
}

f32 mc_c(f32 xa, f32 ya, f32 m)
{
  return ya - (m * xa);
}

f32 length_v2(v2 v)
{
  return (f32)sqrt((v.x * v.x) + (v.y * v.y));
}

v2 sub_v2(v2 a, v2 b)
{
  v2 ret;

  ret.x = a.x - b.x;
  ret.y = a.y - b.y;

  return ret;
}

f32 distance_v2(v2 a, v2 b)
{
  v2 diff = sub_v2(a, b);
  f32 dist = length_v2(diff);

  return dist;
}

v2 normalize(v2 v)
{
  f32 len = length_v2(v);
  v2 ret;
  
  ret.x = v.x / len;
  ret.y = v.y / len;
  
  return ret;
}

v2 normal(f32 x1, f32 y1, f32 x2, f32 y2)
{
  f32 dx = x2 - x1;
  f32 dy = -(y2 - y1);
  v2 v;

  v.x = dx;
  v.y = dy;
  
  return normalize(v);
}

v2 opposite_normal(v2 n)
{
  n.x = -n.x;
  n.y = -n.y;
  return n;
}

f32 quadratic_point(f32 a, f32 b, f32 c, f32 t)
{
  f32 r = ((b - a) - 0.5f * (c - a)) / (0.5f * (0.5f - 1));
  f32 s = c - a - r;

  return (r * t * t) + (s * t) + a;
}

f32 bezier_point(f32 a, f32 b, f32 c, f32 d, f32 t)
{
  f32 t1 = 1 - t;

  return (a * t1 * t1 * t1) +
    (3 * b * t * t1 * t1) +
    (3 * c * t * t * t1) +
    (d * t * t * t);
}

f32 bezier_tangent(f32 a, f32 b, f32 c, f32 d, f32 t)
{
  return (3 * t * t * (-a + 3 * b - 3 * c + d) +
          6 * t * (a - 2 * b + c) +
          3 * (-a + b));
}

