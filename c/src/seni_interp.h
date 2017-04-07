#ifndef SENI_INTERP
#define SENI_INTERP

#include "seni_mathutil.h"

typedef enum RemappingFn {
  REMAP_LINEAR,
  REMAP_QUICK_EASE,
  REMAP_SLOW_EASE_IN,
  REMAP_SLOW_EASE_IN_EASE_OUT
} RemappingFn;

f32 map_linear(f32 x);
f32 map_quick_ease(f32 x);
f32 map_slow_ease_in(f32 x);
f32 map_slow_ease_in_ease_out(f32 x);

#endif /* SENI_INTERP */
