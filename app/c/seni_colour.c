#include "seni_colour.h"
#include "seni_keyword_iname.h"
#include "seni_mathutil.h"

#include <stdlib.h>
#include <math.h>

#define COLOUR_UNIT_ANGLE (360.0f / 12.0f)
#define COLOUR_COMPLIMENTARY_ANGLE (COLOUR_UNIT_ANGLE * 6.0f)
#define COLOUR_TRIAD_ANGLE (COLOUR_UNIT_ANGLE * 4)


#ifdef SENI_BUILD_WASM
#include <webassembly.h>
#define powf Math_pow
#endif

seni_colour *colour_construct(seni_colour_format format, f32 e0, f32 e1, f32 e2, f32 alpha)
{
  seni_colour *colour = (seni_colour *)calloc(1, sizeof(seni_colour));

  colour->format = format;
  colour->element[0] = e0;
  colour->element[1] = e1;
  colour->element[2] = e2;
  colour->element[3] = alpha;

  return colour;
}

void colour_free(seni_colour *colour)
{
  free(colour);
}

void colour_set(seni_colour *out, seni_colour_format format, f32 e0, f32 e1, f32 e2, f32 alpha)
{
  out->format = format;
  out->element[0] = e0;
  out->element[1] = e1;
  out->element[2] = e2;
  out->element[3] = alpha;
}

seni_colour *colour_clone(seni_colour *out, seni_colour *in)
{
  out->format = in->format;
  out->element[0] = in->element[0];
  out->element[1] = in->element[1];
  out->element[2] = in->element[2];
  out->element[3] = in->element[3];

  return out;
}

//  http://www.brucelindbloom.com/index.html?Equations.html

//  l 0 -> 100  lightness
//  a -128 -> +127   green -> red
//  b -128 -> +127   cyan -> yellow
f32 colour_to_axis(f32 component)
{
  f32 temp;
  if (component > 0.04045) {
    temp = powf((component + 0.055f) / 1.055f, 2.4f);
  } else {
    temp = component / 12.92f;
  }

  return temp * 100.0f;
}

seni_colour *rgb_xyz(seni_colour *out, seni_colour *in)
{
  // assumes that this is already in RGB format
  f32 rr = colour_to_axis(in->element[0]);
  f32 gg = colour_to_axis(in->element[1]);
  f32 bb = colour_to_axis(in->element[2]);

  out->format = XYZ;
  out->element[0] = (rr * 0.4124f) + (gg * 0.3576f) + (bb * 0.1805f);
  out->element[1] = (rr * 0.2126f) + (gg * 0.7152f) + (bb * 0.0722f);
  out->element[2] = (rr * 0.0193f) + (gg * 0.1192f) + (bb * 0.9505f);
  out->element[3] = in->element[3];

  return out;
}

f32 axis_to_LAB_component(f32 a)
{
  if (a > 0.008856f) {
    return powf(a, 1.0f / 3.0f);
  } else {
    return (7.787f * a) + (16.0f / 116.0f);
  }
}

seni_colour *xyz_lab(seni_colour *out, seni_colour *in)
{
  // assumes that this is already in XYZ format
  f32 xx = axis_to_LAB_component(in->element[0] / 95.047f);
  f32 yy = axis_to_LAB_component(in->element[1] / 100.000f);
  f32 zz = axis_to_LAB_component(in->element[2] / 108.883f);

  out->format = LAB;
  out->element[0] = (116.0f * yy) - 16.0f;
  out->element[1] = 500.0f * (xx - yy);
  out->element[2] = 200.0f * (yy - zz);
  out->element[3] = in->element[3];

  return out;
}

f32 axis_to_colour(f32 a)
{
  if (a > 0.0031308f) {
    return (1.055f * powf(a, 1.0f / 2.4f)) - 0.055f;
  } else {
    return a * 12.92f;
  }
}

seni_colour *xyz_rgb(seni_colour *out, seni_colour *in)
{
  f32 xx = in->element[0] / 100.0f;
  f32 yy = in->element[1] / 100.0f;
  f32 zz = in->element[2] / 100.0f;

  f32 r = (xx * 3.2406f) + (yy * -1.5372f) + (zz * -0.4986f);
  f32 g = (xx * -0.9689f) + (yy * 1.8758f) + (zz * 0.0415f);
  f32 b = (xx * 0.0557f) + (yy * -0.2040f) + (zz * 1.0570f);

  out->format = RGB;
  out->element[0] = axis_to_colour(r);
  out->element[1] = axis_to_colour(g);
  out->element[2] = axis_to_colour(b);
  out->element[3] = in->element[3];

  return out;
}

i32 max_channel(seni_colour *colour)
{
  i32 hi = colour->element[0] > colour->element[1] ? 0 : 1;
  return colour->element[2] > colour->element[hi] ? 2 : hi;
}


i32 min_channel(seni_colour *colour)
{
  i32 low = colour->element[0] < colour->element[1] ? 0 : 1;
  return colour->element[2] < colour->element[low] ? 2 : low;
}

f32 hue(seni_colour *colour, i32 max_chan, f32 chroma)
{
  if (chroma == 0.0f) {
    return 0.0f;        // invalid hue
  }

  switch (max_chan) {
  case 0:                       // R
    return 60.0f * ((f32)fmod(colour->element[1] - colour->element[2], chroma) / 6.0f);
  case 1:                       // G
    return 60.0f * (((colour->element[2] - colour->element[0]) / chroma) + 2.0f);
  case 2:                       // B
    return 60.0f * (((colour->element[0] - colour->element[1]) / chroma) + 4.0f);
  default:
    break;
  }

  return 0.0f;            // should never get here
}

f32 abso(f32 in)
{
  return in < 0.0f ? -in : in;
}

seni_colour *rgb_hsl(seni_colour *out, seni_colour *in)
{
  i32 min_ch = min_channel(in);
  f32 min_val = in->element[min_ch];

  i32 max_ch = max_channel(in);
  f32 max_val = in->element[max_ch];

  f32 chroma = max_val - min_val;
  f32 h = hue(in, max_ch, chroma);
  // bool valid_hue = (chroma != 0.0);

  f32 lightness = 0.5f * (min_val + max_val);
  f32 saturation;
  if (chroma == 0.0f) {
    saturation = 0.0f;
  } else {
    saturation = chroma / (1.0f - abso((2.0f * lightness) - 1.0f));
  }

  out->format = HSL;
  out->element[0] = h;
  out->element[1] = saturation;
  out->element[2] = lightness;
  out->element[3] = in->element[3];

  // TODO: set valid_hue
  // return col.set('valid_hue', valid_hue);

  return out;
}

seni_colour *rgb_hsv(seni_colour *out, seni_colour *in)
{
  i32 min_ch = min_channel(in);
  f32 min_val = in->element[min_ch];

  i32 max_ch = max_channel(in);
  f32 max_val = in->element[max_ch];

  f32 chroma = max_val - min_val;
  f32 h = hue(in, max_ch, chroma);
  // bool valid_hue = (chroma != 0.0);

  f32 value = max_val;

  f32 saturation;
  if (chroma == 0.0f) {
    saturation = 0.0f;
  } else {
    saturation = chroma / value;
  }
  
  out->format = HSV;
  out->element[0] = h;
  out->element[1] = saturation;
  out->element[2] = value;
  out->element[3] = in->element[3];

  // TODO: set valid_hue
  // return col.set('valid_hue', valid_hue);

  return out;
}

seni_colour *chm_rgb(seni_colour *out, seni_colour *in, f32 chroma, f32 h, f32 m)
{
  // todo: validhue test
  //
  //if (c.get('validHue') === undefined) {
  //return construct(Format.RGB, [m, m, m, element(c, ALPHA)]);
  //}

  f32 hprime = h / 60.0f;
  f32 x = chroma * (1.0f - abso((f32)fmod(hprime, 2.0f) - 1.0f));
  f32 r = 0.0f;
  f32 g = 0.0f;
  f32 b = 0.0f;

  if (hprime < 1.0f) {
    r = chroma;
    g = x;
    b = 0.0;
  } else if (hprime < 2.0f) {
    r = x;
    g = chroma;
    b = 0.0;
  } else if (hprime < 3.0f) {
    r = 0.0;
    g = chroma;
    b = x;
  } else if (hprime < 4.0f) {
    r = 0.0;
    g = x;
    b = chroma;
  } else if (hprime < 5.0f) {
    r = x;
    g = 0.0;
    b = chroma;
  } else if (hprime < 6.0f) {
    r = chroma;
    g = 0.0;
    b = x;
  }

  out->format = RGB;
  out->element[0] = r + m;
  out->element[1] = g + m;
  out->element[2] = b + m;
  out->element[3] = in->element[3];

  return out;
}

seni_colour *hsl_rgb(seni_colour *out, seni_colour *in)
{
  f32 h = in->element[0];
  f32 s = in->element[1];
  f32 l = in->element[2];
  f32 chroma = (1.0f - abso((2.0f * l) - 1.0f)) * s;
  f32 m = l - (0.5f * chroma);

  // todo: set validhue
  // f32 col = c.set('validHue', true);

  chm_rgb(out, in, chroma, h, m);

  return out;
}

f32 lab_component_to_axis(f32 l)
{
  if (powf(l, 3.0f) > 0.008856f) {
    return powf(l, 3.0f);
  } else {
    return (l - (16.0f / 116.0f)) / 7.787f;
  }
}

seni_colour *lab_xyz(seni_colour *out, seni_colour *in)
{
  f32 refX = 95.047f;
  f32 refY = 100.000f;
  f32 refZ = 108.883f;

  f32 y = (in->element[0] + 16.0f) / 116.0f;
  f32 x = (in->element[1] / 500.0f) + y;
  f32 z = y - (in->element[2] / 200.0f);

  f32 xx = lab_component_to_axis(x);
  f32 yy = lab_component_to_axis(y);
  f32 zz = lab_component_to_axis(z);

  out->format = XYZ;
  out->element[0] = refX * xx;
  out->element[1] = refY * yy;
  out->element[2] = refZ * zz;
  out->element[3] = in->element[3];

  return out;
}

seni_colour *hsv_rgb(seni_colour *out, seni_colour *in)
{
  f32 h = in->element[0];
  f32 s = in->element[1];
  f32 v = in->element[2];
  f32 chroma = v * s;
  f32 m = v - chroma;

  chm_rgb(out, in, chroma, h, m);

  return out;
}

seni_colour *colour_clone_as(seni_colour *out, seni_colour *in, seni_colour_format new_format)
{
  switch(in->format) {
  case LAB:
    switch(new_format) {
    case RGB:
      return xyz_rgb(out, lab_xyz(out, in));
      break;
    case HSV:
      return rgb_hsv(out, xyz_rgb(out, lab_xyz(out, in)));
      break;
    case HSL:
      return rgb_hsl(out, xyz_rgb(out, lab_xyz(out, in)));
      break;
    case LAB:
      return colour_clone(out, in);
      break;
    default:
    SENI_ERROR("unknown colour format %d", new_format);
    break;
    }
    break;
  case HSV:
    switch(new_format) {
    case RGB:
      return hsv_rgb(out, in);
      break;
    case HSV:
      return colour_clone(out, in);
      break;
    case HSL:
      return rgb_hsl(out, hsv_rgb(out, in));
      break;
    case LAB:
      return xyz_lab(out, rgb_xyz(out, hsv_rgb(out, in)));
      break;
    default:
    SENI_ERROR("unknown colour format %d", new_format);
    break;
    }
    break;
  case HSL:
    switch(new_format) {
    case RGB:
      return hsl_rgb(out, in);
      break;
    case HSV:
      return rgb_hsv(out, hsl_rgb(out, in));
      break;
    case HSL:
      return colour_clone(out, in);
      break;
    case LAB:
      return xyz_lab(out, rgb_xyz(out, hsl_rgb(out, in)));
      break;
    default:
    SENI_ERROR("unknown colour format %d", new_format);
    break;
    }    
    break;
  case RGB:
    switch(new_format) {
    case RGB:
      return colour_clone(out, in);
      break;
    case HSV:
      return rgb_hsv(out, in);
      break;
    case HSL:
      return rgb_hsl(out, in);
      break;
    case LAB:
      return xyz_lab(out, rgb_xyz(out, in));
      break;
    default:
    SENI_ERROR("unknown colour format %d", new_format);
    break;
    }    
    break;
  default:
    SENI_ERROR("unknown colour format %d", in->format);
    break;
  }

  return out;
}

seni_colour *add_angle_to_hsl(seni_colour *out, seni_colour *in, f32 delta)
{
  i32 H = 0;

  // rotate the hue by the given delta
  colour_clone_as(out, in, HSL);
  out->element[H] = (f32)fmod(out->element[H] + delta, 360.0f);

  return out;
}

// Return the 2 colours either side of this that are 'ang' degrees away
//
void pair(seni_colour *out0, seni_colour *out1, seni_colour *in, f32 ang)
{
  add_angle_to_hsl(out0, in, -ang);
  add_angle_to_hsl(out1, in, ang);
}

// Returns the colour at the opposite end of the wheel
//
seni_colour *complementary(seni_colour *out, seni_colour *in)
{
  return add_angle_to_hsl(out, in, COLOUR_COMPLIMENTARY_ANGLE);
}

// Returns the 2 colours next to a complementary colour.
// e.g. if the input colour is at the 12 o'clock position,
// this will return the 5 o'clock and 7 o'clock colours
//
void split_complementary(seni_colour *out0, seni_colour *out1, seni_colour *in)
{
  seni_colour tmp;
  pair(out0, out1, add_angle_to_hsl(&tmp, in, COLOUR_COMPLIMENTARY_ANGLE), COLOUR_UNIT_ANGLE);
}

// Returns the adjacent colours.
// e.g. given a colour at 3 o'clock this will return the
// colours at 2 o'clock and 4 o'clock
//
void analagous(seni_colour *out0, seni_colour *out1, seni_colour *in)
{
  pair(out0, out1, in, COLOUR_UNIT_ANGLE);
}

// Returns the 2 colours that will result in all 3 colours
// being evenly spaced around the colour wheel.
// e.g. given 12 o'clock this will return 4 o'clock and 8 o'clock
//
void triad(seni_colour *out0, seni_colour *out1, seni_colour *in)
{
  pair(out0, out1, in, COLOUR_TRIAD_ANGLE);
}


void get_colour_presets(f32 *a, f32 *b, f32 *c, f32 *d, i32 preset)
{
  switch(preset) {
  case INAME_CHROME:
    a[0] = 0.5f; a[1] = 0.5f; a[2] = 0.5f;
    b[0] = 0.5f; b[1] = 0.5f; b[2] = 0.5f;
    c[0] = 1.0f; c[1] = 1.0f; c[2] = 1.0f;
    d[0] = 0.0f; d[1] = 0.1f; d[2] = 0.2f;
    break;
  case INAME_HOTLINE_MIAMI:
    a[0] = 0.5f; a[1] = 0.5f; a[2] = 0.5f;
    b[0] = 0.5f; b[1] = 0.5f; b[2] = 0.5f;
    c[0] = 2.0f; c[1] = 1.0f; c[2] = 0.0f;
    d[0] = 0.5f; d[1] = 0.2f; d[2] = 0.25f;
    break;
  case INAME_KNIGHT_RIDER:
    a[0] = 0.5f; a[1] = 0.5f; a[2] = 0.5f;
    b[0] = 0.5f; b[1] = 0.5f; b[2] = 0.5f;
    c[0] = 1.0f; c[1] = 0.7f; c[2] = 0.4f;
    d[0] = 0.0f; d[1] = 0.15f; d[2] = 0.2f;
    break;
  case INAME_MARS:
    a[0] = 0.8f; a[1] = 0.5f; a[2] = 0.4f;
    b[0] = 0.2f; b[1] = 0.4f; b[2] = 0.2f;
    c[0] = 2.0f; c[1] = 1.0f; c[2] = 1.0f;
    d[0] = 0.0f; d[1] = 0.25f; d[2] = 0.25f;
    break;
  case INAME_RAINBOW:
    a[0] = 0.5f; a[1] = 0.5f; a[2] = 0.5f;
    b[0] = 0.5f; b[1] = 0.5f; b[2] = 0.5f;
    c[0] = 1.0f; c[1] = 1.0f; c[2] = 1.0f;
    d[0] = 0.0f; d[1] = 3.33f; d[2] = 6.67f;
    break;
  case INAME_ROBOCOP:
    a[0] = 0.5f; a[1] = 0.5f; a[2] = 0.5f;
    b[0] = 0.5f; b[1] = 0.5f; b[2] = 0.5f;
    c[0] = 1.0f; c[1] = 1.0f; c[2] = 1.0f;
    d[0] = 0.3f; d[1] = 0.2f; d[2] = 0.2f;
    break;
  case INAME_TRANSFORMERS:
    a[0] = 0.5f; a[1] = 0.5f; a[2] = 0.5f;
    b[0] = 0.5f; b[1] = 0.5f; b[2] = 0.5f;
    c[0] = 1.0f; c[1] = 1.0f; c[2] = 0.5f;
    d[0] = 0.8f; d[1] = 0.9f; d[2] = 0.3f;
    break;
  }
}

void colour_procedural(seni_colour *out, seni_colour_fn_state *colour_fn_state, f32 t)
{
  f32 *a = colour_fn_state->a;
  f32 *b = colour_fn_state->b;
  f32 *c = colour_fn_state->c;
  f32 *d = colour_fn_state->d;

  out->format = RGB;
  out->element[0] = a[0] + b[0] * (f32)cos(TAU * (c[0] * t + d[0]));
  out->element[1] = a[1] + b[1] * (f32)cos(TAU * (c[1] * t + d[1]));
  out->element[2] = a[2] + b[2] * (f32)cos(TAU * (c[2] * t + d[2]));
  out->element[3] = colour_fn_state->alpha;
}

void colour_bezier(seni_colour *out, seni_colour_fn_state *colour_fn_state, f32 t)
{
  f32 *a = colour_fn_state->a;
  f32 *b = colour_fn_state->b;
  f32 *c = colour_fn_state->c;
  f32 *d = colour_fn_state->d;

  // assuming that seni_bind is using RGB colour space
  // todo: experiment with different colour spaces
  out->format = RGB;
  out->element[0] = bezier_point(a[0], b[0], c[0], d[0], t);
  out->element[1] = bezier_point(a[1], b[1], c[1], d[1], t);
  out->element[2] = bezier_point(a[2], b[2], c[2], d[2], t);
  out->element[3] = bezier_point(a[3], b[3], c[3], d[3], t);
}
