#include "parametric.h"

#include "keyword_iname.h"
#include "mathutil.h"

#include <math.h>

f32 seni_parametric(f32  val,
                    f32  from_a,
                    f32  from_b,
                    f32  to_a,
                    f32  to_b,
                    i32  mapping,
                    bool clamping) {
  f32 from_m = mc_m(from_a, 0.0f, from_b, 1.0f);
  f32 from_c = mc_c(from_a, 0.0f, from_m);

  f32 to_m = mc_m(0.0f, to_a, 1.0f, to_b);
  f32 to_c = mc_c(0.0f, to_a, to_m);

  f32 from_interp = (from_m * val) + from_c;
  f32 to_interp   = from_interp;

  if (mapping == INAME_LINEAR) {
    to_interp = from_interp;
  } else if (mapping == INAME_QUICK) {
    to_interp = map_quick_ease(from_interp);
  } else if (mapping == INAME_SLOW_IN) {
    to_interp = map_slow_ease_in(from_interp);
  } else { // INAME_slow_in_out
    to_interp = map_slow_ease_in_ease_out(from_interp);
  }

  f32 res = (to_m * to_interp) + to_c;

  if (clamping) {
    res = from_interp < 0.0f ? to_a : (from_interp > 1.0f) ? to_b : res;
  }

  return res;
}

f32 seni_parametric_cos(f32 amplitude, f32 frequency, f32 t) {
  return amplitude * (f32)cos(t * frequency);
}

f32 seni_parametric_sin(f32 amplitude, f32 frequency, f32 t) {
  return amplitude * (f32)sin(t * frequency);
}

void seni_parametric_bezier(f32* outx, f32* outy, f32* coords, f32 t) {
  *outx = bezier_point(coords[0], coords[2], coords[4], coords[6], t);
  *outy = bezier_point(coords[1], coords[3], coords[5], coords[7], t);
}

void seni_parametric_bezier_tangent(f32* outx, f32* outy, f32* coords, f32 t) {
  *outx = bezier_tangent(coords[0], coords[2], coords[4], coords[6], t);
  *outy = bezier_tangent(coords[1], coords[3], coords[5], coords[7], t);
}

void seni_parametric_circle(f32* outx, f32* outy, f32* position, f32 radius, f32 t) {
  f32 angle = t * TAU;

  *outx = ((f32)sin(angle) * radius) + position[0];
  *outy = ((f32)cos(angle) * radius) + position[1];
}
