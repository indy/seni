#pragma once

#include "types.h"

#ifdef SENIE_BUILD_WASM
#define TIMING_UNIT f32
#else
#include "time.h"
#define TIMING_UNIT clock_t
#endif

TIMING_UNIT get_timing();
f32         timing_delta(TIMING_UNIT a, TIMING_UNIT b);
f32         timing_delta_from(TIMING_UNIT earlier);
