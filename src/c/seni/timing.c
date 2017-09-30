#include "timing.h"

#include "js_imports.h"

TIMING_UNIT get_timing()
{
#ifdef SENI_BUILD_WASM  
  return performance_now();
#else
  return clock();
#endif
}

f32 timing_delta(TIMING_UNIT earlier, TIMING_UNIT later)
{
#ifdef SENI_BUILD_WASM
  return later - earlier;
#else
  clock_t diff = later - earlier;
  int msec = diff * 1000 / CLOCKS_PER_SEC;

  return (f32)(msec);  
#endif
}

f32 timing_delta_from(TIMING_UNIT earlier)
{
  TIMING_UNIT later = get_timing();
  f32 delta = timing_delta(earlier, later);

  return delta;
}
