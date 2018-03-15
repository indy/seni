#pragma once

#include "config.h"
#include "types.h"

typedef enum {
  COLOUR_FN_UNKNOWN = 0,
  COLOUR_FN_PROCEDURAL,
  COLOUR_FN_BEZIER,
  COLOUR_FN_QUADRATIC
} senie_colour_fn_type;

struct senie_colour_fn_state {
  senie_colour_fn_type type;
  f32                  a[4];
  f32                  b[4];
  f32                  c[4];
  f32                  d[4];
  f32                  alpha;
};

senie_colour* complementary(senie_colour* out, senie_colour* in);
void          split_complementary(senie_colour* out0, senie_colour* out1, senie_colour* in);
void          analagous(senie_colour* out0, senie_colour* out1, senie_colour* in);
void          triad(senie_colour* out0, senie_colour* out1, senie_colour* in);

void get_colour_presets(f32* a, f32* b, f32* c, f32* d, i32 preset);
void colour_procedural(senie_colour* out, senie_colour_fn_state* colour_fn_state, f32 t);
void colour_bezier(senie_colour* out, senie_colour_fn_state* colour_fn_state, f32 t);
