#include "colour_scheme.h"

#include "colour.h"
#include "keyword_iname.h"
#include "mathutil.h"

#include <math.h>

#define COLOUR_UNIT_ANGLE (360.0f / 12.0f)
#define COLOUR_COMPLIMENTARY_ANGLE (COLOUR_UNIT_ANGLE * 6.0f)
#define COLOUR_TRIAD_ANGLE (COLOUR_UNIT_ANGLE * 4)

seni_colour* add_angle_to_hsluv(seni_colour* out, seni_colour* in, f32 delta) {
  i32 H = 0;

  seni_colour_format original_format = in->format;

  // rotate the hue by the given delta
  colour_clone_as(out, in, HSLuv);
  out->element[H] = (f32)fmod(out->element[H] + delta, 360.0f);

  // return the new colour in the format of the original colour
  return colour_clone_as(out, out, original_format);
}

// Return the 2 colours either side of this that are 'ang' degrees away
//
void pair(seni_colour* out0, seni_colour* out1, seni_colour* in, f32 ang) {
  add_angle_to_hsluv(out0, in, -ang);
  add_angle_to_hsluv(out1, in, ang);
}

// Returns the colour at the opposite end of the wheel
//
seni_colour* complementary(seni_colour* out, seni_colour* in) {
  return add_angle_to_hsluv(out, in, COLOUR_COMPLIMENTARY_ANGLE);
}

// Returns the 2 colours next to a complementary colour.
// e.g. if the input colour is at the 12 o'clock position,
// this will return the 5 o'clock and 7 o'clock colours
//
void split_complementary(seni_colour* out0, seni_colour* out1, seni_colour* in) {
  seni_colour tmp;
  pair(out0, out1, add_angle_to_hsluv(&tmp, in, COLOUR_COMPLIMENTARY_ANGLE), COLOUR_UNIT_ANGLE);
}

// Returns the adjacent colours.
// e.g. given a colour at 3 o'clock this will return the
// colours at 2 o'clock and 4 o'clock
//
void analagous(seni_colour* out0, seni_colour* out1, seni_colour* in) {
  pair(out0, out1, in, COLOUR_UNIT_ANGLE);
}

// Returns the 2 colours that will result in all 3 colours
// being evenly spaced around the colour wheel.
// e.g. given 12 o'clock this will return 4 o'clock and 8 o'clock
//
void triad(seni_colour* out0, seni_colour* out1, seni_colour* in) {
  pair(out0, out1, in, COLOUR_TRIAD_ANGLE);
}

void get_colour_presets(f32* a, f32* b, f32* c, f32* d, i32 preset) {
  switch (preset) {
  case INAME_CHROME:
    a[0] = 0.5f;
    a[1] = 0.5f;
    a[2] = 0.5f;
    b[0] = 0.5f;
    b[1] = 0.5f;
    b[2] = 0.5f;
    c[0] = 1.0f;
    c[1] = 1.0f;
    c[2] = 1.0f;
    d[0] = 0.0f;
    d[1] = 0.1f;
    d[2] = 0.2f;
    break;
  case INAME_HOTLINE_MIAMI:
    a[0] = 0.5f;
    a[1] = 0.5f;
    a[2] = 0.5f;
    b[0] = 0.5f;
    b[1] = 0.5f;
    b[2] = 0.5f;
    c[0] = 2.0f;
    c[1] = 1.0f;
    c[2] = 0.0f;
    d[0] = 0.5f;
    d[1] = 0.2f;
    d[2] = 0.25f;
    break;
  case INAME_KNIGHT_RIDER:
    a[0] = 0.5f;
    a[1] = 0.5f;
    a[2] = 0.5f;
    b[0] = 0.5f;
    b[1] = 0.5f;
    b[2] = 0.5f;
    c[0] = 1.0f;
    c[1] = 0.7f;
    c[2] = 0.4f;
    d[0] = 0.0f;
    d[1] = 0.15f;
    d[2] = 0.2f;
    break;
  case INAME_MARS:
    a[0] = 0.8f;
    a[1] = 0.5f;
    a[2] = 0.4f;
    b[0] = 0.2f;
    b[1] = 0.4f;
    b[2] = 0.2f;
    c[0] = 2.0f;
    c[1] = 1.0f;
    c[2] = 1.0f;
    d[0] = 0.0f;
    d[1] = 0.25f;
    d[2] = 0.25f;
    break;
  case INAME_RAINBOW:
    a[0] = 0.5f;
    a[1] = 0.5f;
    a[2] = 0.5f;
    b[0] = 0.5f;
    b[1] = 0.5f;
    b[2] = 0.5f;
    c[0] = 1.0f;
    c[1] = 1.0f;
    c[2] = 1.0f;
    d[0] = 0.0f;
    d[1] = 3.33f;
    d[2] = 6.67f;
    break;
  case INAME_ROBOCOP:
    a[0] = 0.5f;
    a[1] = 0.5f;
    a[2] = 0.5f;
    b[0] = 0.5f;
    b[1] = 0.5f;
    b[2] = 0.5f;
    c[0] = 1.0f;
    c[1] = 1.0f;
    c[2] = 1.0f;
    d[0] = 0.3f;
    d[1] = 0.2f;
    d[2] = 0.2f;
    break;
  case INAME_TRANSFORMERS:
    a[0] = 0.5f;
    a[1] = 0.5f;
    a[2] = 0.5f;
    b[0] = 0.5f;
    b[1] = 0.5f;
    b[2] = 0.5f;
    c[0] = 1.0f;
    c[1] = 1.0f;
    c[2] = 0.5f;
    d[0] = 0.8f;
    d[1] = 0.9f;
    d[2] = 0.3f;
    break;
  }
}

void colour_procedural(seni_colour* out, seni_colour_fn_state* colour_fn_state, f32 t) {
  f32* a = colour_fn_state->a;
  f32* b = colour_fn_state->b;
  f32* c = colour_fn_state->c;
  f32* d = colour_fn_state->d;

  out->format     = RGB;
  out->element[0] = a[0] + b[0] * (f32)cos(TAU * (c[0] * t + d[0]));
  out->element[1] = a[1] + b[1] * (f32)cos(TAU * (c[1] * t + d[1]));
  out->element[2] = a[2] + b[2] * (f32)cos(TAU * (c[2] * t + d[2]));
  out->element[3] = colour_fn_state->alpha;
}

void colour_bezier(seni_colour* out, seni_colour_fn_state* colour_fn_state, f32 t) {
  f32* a = colour_fn_state->a;
  f32* b = colour_fn_state->b;
  f32* c = colour_fn_state->c;
  f32* d = colour_fn_state->d;

  // assuming that seni_bind is using RGB colour space
  // todo: experiment with different colour spaces
  out->format     = RGB;
  out->element[0] = bezier_point(a[0], b[0], c[0], d[0], t);
  out->element[1] = bezier_point(a[1], b[1], c[1], d[1], t);
  out->element[2] = bezier_point(a[2], b[2], c[2], d[2], t);
  out->element[3] = bezier_point(a[3], b[3], c[3], d[3], t);
}
