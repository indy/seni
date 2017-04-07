#include "seni_interp.h"

f32 map_linear(f32 x)
{
  return x;
}

f32 map_quick_ease(f32 x)
{
  f32 x2 = x * x;
  f32 x3 = x * x * x;

  return (3 * x2) - (2 * x3);
}

f32 map_slow_ease_in(f32 x)
{
  f32 s = (f32)sin(x * PI_BY_2);
  return s * s * s * s;
}

f32 map_slow_ease_in_ease_out(f32 x)
{
  return x - ((f32)sin(x * TAU) / TAU);
}

