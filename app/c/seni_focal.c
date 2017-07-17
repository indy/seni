#include "seni_focal.h"
#include "seni_interp.h"
#include "seni_mathutil.h"
#include "seni_config.h"
#include "seni_keyword_iname.h"

// TODO: find out how small this can be and place it somewhere more accessible for other code
#define TINY_FLOAT 0.000001f

f32 focal_point(f32 x, f32 y, f32 distance, i32 mapping, f32 centre_x, f32 centre_y)
{
  f32 d = distance_v2(x, y, centre_x, centre_y);

  if (d < TINY_FLOAT) {
    return 1.0f;
  }

  f32 res = seni_interp(d, 0.0f, distance, 1.0f, 0.0f, mapping, true);
  
  return res;
}

f32 focal_hline(f32 y, f32 distance, i32 mapping, f32 centre_y)
{
  f32 d = centre_y - y;
  d = absf(d);

  if (d < TINY_FLOAT) {
    return 1.0f;
  }

  f32 res = seni_interp(d, 0.0f, distance, 1.0f, 0.0f, mapping, true);
  
  return res;
}

f32 focal_vline(f32 x, f32 distance, i32 mapping, f32 centre_x)
{
  f32 d = centre_x - x;
  d = absf(d);

  if (d < TINY_FLOAT) {
    return 1.0f;
  }

  f32 res = seni_interp(d, 0.0f, distance, 1.0f, 0.0f, mapping, true);
  
  return res;
}


