#pragma once

#include "config.h"
#include "types.h"

typedef enum {
  COLOUR_FN_UNKNOWN = 0,
  COLOUR_FN_PROCEDURAL,
  COLOUR_FN_BEZIER,
  COLOUR_FN_QUADRATIC
} sen_colour_fn_type;

struct sen_colour_fn_state {
  sen_colour_fn_type type;
  f32                a[4];
  f32                b[4];
  f32                c[4];
  f32                d[4];
  f32                alpha;
};

sen_colour* complementary(sen_colour* out, sen_colour* in);
void split_complementary(sen_colour* out0, sen_colour* out1, sen_colour* in);
void analagous(sen_colour* out0, sen_colour* out1, sen_colour* in);
void triad(sen_colour* out0, sen_colour* out1, sen_colour* in);

void get_colour_presets(f32* a, f32* b, f32* c, f32* d, i32 preset);
void colour_procedural(sen_colour* out, sen_colour_fn_state* colour_fn_state,
                       f32 t);
void colour_bezier(sen_colour* out, sen_colour_fn_state* colour_fn_state,
                   f32 t);
