#include "parametric.h"

#include "config.h"
#include "ease.h"
#include "mathutil.h"

#include <math.h>

f32 sen_parametric(f32 val, f32 from_a, f32 from_b, f32 to_a, f32 to_b,
                   i32 mapping, bool clamping) {
  f32 from_m = mc_m(from_a, 0.0f, from_b, 1.0f);
  f32 from_c = mc_c(from_a, 0.0f, from_m);

  f32 to_m = mc_m(0.0f, to_a, 1.0f, to_b);
  f32 to_c = mc_c(0.0f, to_a, to_m);

  f32 from_interp = (from_m * val) + from_c;
  f32 to_interp   = easing(from_interp, mapping);
  f32 res         = (to_m * to_interp) + to_c;

  if (clamping) {
    res = from_interp < 0.0f ? to_a : (from_interp > 1.0f) ? to_b : res;
  }

  return res;
}

f32 sen_parametric_scalar(f32 a, f32 b, i32 mapping, bool clamping, f32 t) {
  f32 new_t = easing(t, mapping);
  f32 res = lerp(new_t, a, b);

  if (clamping) {
    res = new_t < 0.0f ? a : (new_t > 1.0f) ? b : res;
  }

  return res;
}

f32 sen_parametric_cos(f32 amplitude, f32 frequency, f32 t) {
  return amplitude * (f32)cos(t * frequency);
}

f32 sen_parametric_sin(f32 amplitude, f32 frequency, f32 t) {
  return amplitude * (f32)sin(t * frequency);
}

void sen_parametric_bezier(f32* outx, f32* outy, f32* coords, f32 t) {
  *outx = bezier_point(coords[0], coords[2], coords[4], coords[6], t);
  *outy = bezier_point(coords[1], coords[3], coords[5], coords[7], t);
}

void sen_parametric_bezier_tangent(f32* outx, f32* outy, f32* coords, f32 t) {
  *outx = bezier_tangent(coords[0], coords[2], coords[4], coords[6], t);
  *outy = bezier_tangent(coords[1], coords[3], coords[5], coords[7], t);
}

void sen_parametric_circle(f32* outx, f32* outy, f32* position, f32 radius,
                           f32 t) {
  f32 angle = t * TAU;

  *outx = ((f32)sin(angle) * radius) + position[0];
  *outy = ((f32)cos(angle) * radius) + position[1];
}

void sen_parametric_ray(f32* outx, f32* outy, f32* point, f32* direction, f32 t) {
  // direction should be a normalized vector

  *outx = point[0] + (direction[0] * t);
  *outy = point[1] + (direction[1] * t);
}
