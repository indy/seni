#include "seni_mathutil.h"

f32 mc_m(f32 xa, f32 ya, f32 xb, f32 yb)
{
  return (ya - yb) / (xa - xb);
}

f32 mc_c(f32 xa, f32 ya, f32 m)
{
  return ya - (m * xa);
}

v2 normal(f32 x1, f32 y1, f32 x2, f32 y2)
{
  f32 dx = x2 - x1;
  f32 dy = -(y2 - y1);

  f32 len = sqrt((dy * dy) + (dx * dx));
  f32 nx = dy / len;
  f32 ny = dx / len;

  v2 ret = {nx, ny};

  return ret;
}

v2 opposite_normal(v2 n)
{
  n.x = -n.x;
  n.y = -n.y;
  return n;
}
