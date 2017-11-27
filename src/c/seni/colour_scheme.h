#pragma once

#include "config.h"
#include "types.h"

typedef enum {
  COLOUR_FN_UNKNOWN = 0,
  COLOUR_FN_PROCEDURAL,
  COLOUR_FN_BEZIER,
  COLOUR_FN_QUADRATIC
} seni_colour_fn_type;

struct seni_colour_fn_state {
  seni_colour_fn_type type;
  f32                 a[4];
  f32                 b[4];
  f32                 c[4];
  f32                 d[4];
  f32                 alpha;
};

seni_colour *complementary(seni_colour *out, seni_colour *in);
void         split_complementary(seni_colour *out0, seni_colour *out1, seni_colour *in);
void         analagous(seni_colour *out0, seni_colour *out1, seni_colour *in);
void         triad(seni_colour *out0, seni_colour *out1, seni_colour *in);

void get_colour_presets(f32 *a, f32 *b, f32 *c, f32 *d, i32 preset);
void colour_procedural(seni_colour *out, seni_colour_fn_state *colour_fn_state, f32 t);
void colour_bezier(seni_colour *out, seni_colour_fn_state *colour_fn_state, f32 t);
