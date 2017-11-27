#include "focal.h"

#include "config.h"
#include "keyword_iname.h"
#include "mathutil.h"
#include "parametric.h"

// TODO: find out how small this can be and place it somewhere more accessible
// for other code
#define SENI_EPSILON 0.000001f

f32 focal_point(f32 x, f32 y, f32 distance, i32 mapping, f32 centre_x, f32 centre_y) {
  f32 d = distance_v2(x, y, centre_x, centre_y);

  if (d < SENI_EPSILON) {
    return 1.0f;
  }

  f32 res = seni_parametric(d, 0.0f, distance, 1.0f, 0.0f, mapping, true);

  return res;
}

f32 focal_hline(f32 y, f32 distance, i32 mapping, f32 centre_y) {
  f32 d = centre_y - y;
  d     = absf(d);

  if (d < SENI_EPSILON) {
    return 1.0f;
  }

  f32 res = seni_parametric(d, 0.0f, distance, 1.0f, 0.0f, mapping, true);

  return res;
}

f32 focal_vline(f32 x, f32 distance, i32 mapping, f32 centre_x) {
  f32 d = centre_x - x;
  d     = absf(d);

  if (d < SENI_EPSILON) {
    return 1.0f;
  }

  f32 res = seni_parametric(d, 0.0f, distance, 1.0f, 0.0f, mapping, true);

  return res;
}
