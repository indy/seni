#include "seni_interp.h"
#include "seni_mathutil.h"

f32 seni_interp_cos(f32 amplitude, f32 frequency, f32 t)
{
  return amplitude * (f32)cos(t * frequency);
}

f32 seni_interp_sin(f32 amplitude, f32 frequency, f32 t)
{
  return amplitude * (f32)sin(t * frequency);
}

v2 seni_interp_bezier(f32 *coords, f32 t)
{
  v2 ret;

  ret.x = bezier_point(coords[0], coords[2], coords[4], coords[6], t);
  ret.y = bezier_point(coords[1], coords[3], coords[5], coords[7], t);

  return ret;
}

v2 seni_interp_bezier_tangent(f32 *coords, f32 t)
{
  v2 ret;

  ret.x = bezier_tangent(coords[0], coords[2], coords[4], coords[6], t);
  ret.y = bezier_tangent(coords[1], coords[3], coords[5], coords[7], t);

  return ret;
}

v2 seni_interp_circle(f32 *position, f32 radius, f32 t)
{
  v2 ret;
  f32 angle = t * TAU;

  ret.x = ((f32)sin(angle) * radius) + position[0];
  ret.y = ((f32)cos(angle) * radius) + position[1];

  return ret;
}
