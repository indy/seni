#include "colour.h"

#include "mathutil.h"

#include <stdlib.h>
// float.h for FLT_MAX
#include <float.h>
#include <math.h>

typedef struct seni_colour_64 {
  seni_colour_format format;
  f64                element[4];
} seni_colour_64;

const f64 ref_u = 0.19783000664283680764;
const f64 ref_v = 0.46831999493879100370;

//  http://www.brucelindbloom.com/index.html?Equations.html
//  http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html

// we're using an sRGB working space with a D65 reference white

// https://uk.mathworks.com/help/images/ref/whitepoint.html
// the D65 whitepoint
#define WHITEPOINT_0 0.9504
#define WHITEPOINT_1 1.0
#define WHITEPOINT_2 1.0888

#define CIE_EPSILON 0.008856
#define CIE_KAPPA 903.3

// intent from the CIE
//
// #define CIE_EPSILON (216.0f / 24389.0f)
// #define CIE_KAPPA (24389.0f / 27.0f)

// RGB to XYZ (M)
// 0.4124564  0.3575761  0.1804375
// 0.2126729  0.7151522  0.0721750
// 0.0193339  0.1191920  0.9503041

// XYZ to RBG (M)^-1
//  3.2404542 -1.5371385 -0.4985314
// -0.9692660  1.8760108  0.0415560
//  0.0556434 -0.2040259  1.0572252

#ifdef SENI_BUILD_WASM
#include <webassembly.h>
#define powf Math_pow
#define pow Math_pow
#endif

void colour_set(seni_colour *out, seni_colour_format format, f32 e0, f32 e1, f32 e2, f32 alpha) {
  out->format     = format;
  out->element[0] = e0;
  out->element[1] = e1;
  out->element[2] = e2;
  out->element[3] = alpha;
}

seni_colour_64 *colour_64_from_colour(seni_colour_64 *out, seni_colour *in) {
  out->format     = in->format;
  out->element[0] = (f64)(in->element[0]);
  out->element[1] = (f64)(in->element[1]);
  out->element[2] = (f64)(in->element[2]);
  out->element[3] = (f64)(in->element[3]);

  return out;
}

seni_colour *colour_from_colour_64(seni_colour *out, seni_colour_64 *in) {
  out->format     = in->format;
  out->element[0] = (f32)(in->element[0]);
  out->element[1] = (f32)(in->element[1]);
  out->element[2] = (f32)(in->element[2]);
  out->element[3] = (f32)(in->element[3]);

  return out;
}

seni_colour *colour_clone(seni_colour *out, seni_colour *in) {
  out->format     = in->format;
  out->element[0] = in->element[0];
  out->element[1] = in->element[1];
  out->element[2] = in->element[2];
  out->element[3] = in->element[3];

  return out;
}

// http://www.brucelindbloom.com/index.html?Equations.html
// inverse sRGB companding
//
f64 colour_to_axis(f64 component) {
  f64 temp;
  if (component > 0.04045) {
    temp = pow((component + 0.055) / 1.055, 2.4);
  } else {
    temp = component / 12.92;
  }

  return temp;
}

f64 axis_to_colour(f64 a) {
  if (a > 0.0031308) {
    return (1.055 * pow(a, 1.0 / 2.4)) - 0.055;
  } else {
    return a * 12.92;
  }
}

seni_colour_64 *xyz_from_rgb(seni_colour_64 *col) {
  f64 r = colour_to_axis(col->element[0]);
  f64 g = colour_to_axis(col->element[1]);
  f64 b = colour_to_axis(col->element[2]);

  // multiply by matrix
  // see http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html
  // sRGB colour space with D65 reference white
  //
  col->format = XYZ;
  col->element[0] =
      (r * 0.41239079926595948129) + (g * 0.35758433938387796373) + (b * 0.18048078840183428751);
  col->element[1] =
      (r * 0.21263900587151035754) + (g * 0.71516867876775592746) + (b * 0.07219231536073371500);
  col->element[2] =
      (r * 0.01933081871559185069) + (g * 0.11919477979462598791) + (b * 0.95053215224966058086);

  return col;
}

seni_colour_64 *rgb_from_xyz(seni_colour_64 *col) {
  f64 xx = col->element[0];
  f64 yy = col->element[1];
  f64 zz = col->element[2];

  f64 r = (xx * 3.24096994190452134377) + (yy * -1.53738317757009345794) +
          (zz * -0.49861076029300328366);
  f64 g = (xx * -0.96924363628087982613) + (yy * 1.87596750150772066772) +
          (zz * 0.04155505740717561247);
  f64 b = (xx * 0.05563007969699360846) + (yy * -0.20397695888897656435) +
          (zz * 1.05697151424287856072);

  col->format     = RGB;
  col->element[0] = axis_to_colour(r);
  col->element[1] = axis_to_colour(g);
  col->element[2] = axis_to_colour(b);

  return col;
}

f64 axis_to_LAB_component(f64 a) {
  if (a > CIE_EPSILON) {
    return cbrt(a); // cube root
  } else {
    return ((CIE_KAPPA * a) + 16.0) / 116.0;
  }
}

seni_colour_64 *lab_from_xyz(seni_colour_64 *col) {
  f64 xr = col->element[0] / WHITEPOINT_0;
  f64 yr = col->element[1] / WHITEPOINT_1;
  f64 zr = col->element[2] / WHITEPOINT_2;

  f64 fx = axis_to_LAB_component(xr);
  f64 fy = axis_to_LAB_component(yr);
  f64 fz = axis_to_LAB_component(zr);

  col->format     = LAB;
  col->element[0] = (116.0 * fy) - 16.0;
  col->element[1] = 500.0 * (fx - fy);
  col->element[2] = 200.0 * (fy - fz);

  return col;
}

i32 max_channel(seni_colour_64 *colour) {
  i32 hi = colour->element[0] > colour->element[1] ? 0 : 1;
  return colour->element[2] > colour->element[hi] ? 2 : hi;
}

i32 min_channel(seni_colour_64 *colour) {
  i32 low = colour->element[0] < colour->element[1] ? 0 : 1;
  return colour->element[2] < colour->element[low] ? 2 : low;
}

// http://www.rapidtables.com/convert/color/rgb-to-hsl.htm
f64 hue(seni_colour_64 *colour, i32 max_chan, f64 chroma) {
  if (chroma == 0.0) {
    return 0.0; // invalid hue
  }

  f64 angle = 0.0;

  switch (max_chan) {
  case 0: // R
    angle = 60.0 * ((f32)fmod((colour->element[1] - colour->element[2]) / chroma, 6.0));
    break;
  case 1: // G
    angle = 60.0 * (((colour->element[2] - colour->element[0]) / chroma) + 2.0);
    break;
  case 2: // B
    angle = 60.0 * (((colour->element[0] - colour->element[1]) / chroma) + 4.0);
    break;
  default:
    break;
  }

  while (angle < 0.0) {
    angle += 360.0;
  }

  return angle;
}

f64 abso(f64 in) { return in < 0.0 ? -in : in; }

// http://www.rapidtables.com/convert/color/rgb-to-hsl.htm
seni_colour_64 *hsl_from_rgb(seni_colour_64 *col) {
  i32 min_ch  = min_channel(col);
  f64 min_val = col->element[min_ch];

  i32 max_ch  = max_channel(col);
  f64 max_val = col->element[max_ch];

  f64 delta = max_val - min_val;

  f64 h = hue(col, max_ch, delta);

  f64 lightness = 0.5 * (min_val + max_val);

  f64 saturation;
  if (delta == 0.0) {
    saturation = 0.0;
  } else {
    saturation = delta / (1.0 - abso((2.0 * lightness) - 1.0));
  }

  col->format     = HSL;
  col->element[0] = h;
  col->element[1] = saturation;
  col->element[2] = lightness;

  return col;
}

seni_colour_64 *hsv_from_rgb(seni_colour_64 *col) {
  i32 min_ch  = min_channel(col);
  f64 min_val = col->element[min_ch];

  i32 max_ch  = max_channel(col);
  f64 max_val = col->element[max_ch];

  f64 chroma = max_val - min_val;
  f64 h      = hue(col, max_ch, chroma);
  // bool valid_hue = (chroma != 0.0);

  f64 value = max_val;

  f64 saturation;
  if (chroma == 0.0) {
    saturation = 0.0;
  } else {
    saturation = chroma / value;
  }

  col->format     = HSV;
  col->element[0] = h;
  col->element[1] = saturation;
  col->element[2] = value;

  // TODO: set valid_hue
  // return col.set('valid_hue', valid_hue);

  return col;
}

seni_colour_64 *rgb_from_chm(seni_colour_64 *col, f64 chroma, f64 h, f64 m) {
  // todo: validhue test
  //
  // if (c.get('validHue') === undefined) {
  // return construct(Format.RGB, [m, m, m, element(c, ALPHA)]);
  //}

  f64 hprime = h / 60.0;
  f64 x      = chroma * (1.0 - abso((f64)fmod(hprime, 2.0) - 1.0));
  f64 r      = 0.0;
  f64 g      = 0.0;
  f64 b      = 0.0;

  if (hprime < 1.0) {
    r = chroma;
    g = x;
    b = 0.0;
  } else if (hprime < 2.0) {
    r = x;
    g = chroma;
    b = 0.0;
  } else if (hprime < 3.0) {
    r = 0.0;
    g = chroma;
    b = x;
  } else if (hprime < 4.0) {
    r = 0.0;
    g = x;
    b = chroma;
  } else if (hprime < 5.0) {
    r = x;
    g = 0.0;
    b = chroma;
  } else if (hprime < 6.0) {
    r = chroma;
    g = 0.0;
    b = x;
  }

  col->format     = RGB;
  col->element[0] = r + m;
  col->element[1] = g + m;
  col->element[2] = b + m;

  return col;
}

seni_colour_64 *rgb_from_hsl(seni_colour_64 *col) {
  f64 h      = col->element[0];
  f64 s      = col->element[1];
  f64 l      = col->element[2];
  f64 chroma = (1.0 - abso((2.0 * l) - 1.0)) * s;
  f64 m      = l - (0.5 * chroma);

  // todo: set validhue
  // f64 col = c.set('validHue', true);

  return rgb_from_chm(col, chroma, h, m);
}

f64 lab_component_to_axis(f64 l) {
  if (pow(l, 3.0) > CIE_EPSILON) {
    return pow(l, 3.0);
  } else {
    return ((116.0 * l) - 16.0) / CIE_KAPPA;
  }
}

seni_colour_64 *xyz_from_lab(seni_colour_64 *col) {
  f64 fy = (col->element[0] + 16.0) / 116.0;
  f64 fz = fy - (col->element[2] / 200.0);
  f64 fx = (col->element[1] / 500.0) + fy;

  f64 xr = lab_component_to_axis(fx);
  f64 yr = 0.0;
  if (col->element[0] > (CIE_EPSILON * CIE_KAPPA)) {
    yr = ((col->element[0] + 16.0) / 116.0);
    yr = yr * yr * yr;
  } else {
    yr = col->element[0] / CIE_KAPPA;
  }
  f64 zr = lab_component_to_axis(fz);

  col->format     = XYZ;
  col->element[0] = WHITEPOINT_0 * xr;
  col->element[1] = WHITEPOINT_1 * yr;
  col->element[2] = WHITEPOINT_2 * zr;

  return col;
}

seni_colour_64 *rgb_from_hsv(seni_colour_64 *col) {
  f64 h      = col->element[0];
  f64 s      = col->element[1];
  f64 v      = col->element[2];
  f64 chroma = v * s;
  f64 m      = v - chroma;

  return rgb_from_chm(col, chroma, h, m);
}

// the luv and hsluv code is based on https://github.com/hsluv/hsluv-c
// which uses the MIT License:

// # The MIT License (MIT)

// Copyright © 2015 Alexei Boronine (original idea, JavaScript implementation)
// Copyright © 2015 Roger Tallada (Obj-C implementation)
// Copyright © 2017 Martin Mitáš (C implementation, based on Obj-C
// implementation)

// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the “Software”),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included
// in all copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
// THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
// IN THE SOFTWARE.

typedef struct Bounds_tag Bounds;
struct Bounds_tag {
  f64 a;
  f64 b;
};

void get_bounds(f64 l, Bounds bounds[6]) {
  f64 tl   = l + 16.0;
  f64 sub1 = (tl * tl * tl) / 1560896.0;
  f64 sub2 = sub1 > CIE_EPSILON ? sub1 : (l / CIE_KAPPA);
  int channel;
  int t;

  f64 m[3][3];
  m[0][0] = 3.24096994190452134377;
  m[0][1] = -1.53738317757009345794;
  m[0][2] = -0.49861076029300328366;
  m[1][0] = -0.96924363628087982613;
  m[1][1] = 1.87596750150772066772;
  m[1][2] = 0.04155505740717561247;
  m[2][0] = 0.05563007969699360846;
  m[2][1] = -0.20397695888897656435;
  m[2][2] = 1.05697151424287856072;

  for (channel = 0; channel < 3; channel++) {
    f64 m1 = m[channel][0];
    f64 m2 = m[channel][1];
    f64 m3 = m[channel][2];

    for (t = 0; t < 2; t++) {
      f64 top1   = (284517.0 * m1 - 94839.0 * m3) * sub2;
      f64 top2   = (838422.0 * m3 + 769860.0 * m2 + 731718.0 * m1) * l * sub2 - 769860.0 * t * l;
      f64 bottom = (632260.0 * m3 - 126452.0 * m2) * sub2 + 126452.0 * t;

      bounds[channel * 2 + t].a = top1 / bottom;
      bounds[channel * 2 + t].b = top2 / bottom;
    }
  }
}

f64 intersect_line_line(const Bounds *line1, const Bounds *line2) {
  return (line1->b - line2->b) / (line2->a - line1->a);
}

f64 dist_from_pole(f64 x, f64 y) { return sqrt(x * x + y * y); }

f64 ray_length_until_intersect(f64 theta, const Bounds *line) {
  return line->b / (sin(theta) - line->a * cos(theta));
}

f64 max_safe_chroma_for_l(f64 l) {
  f64    min_len = FLT_MAX;
  Bounds bounds[6];
  int    i;

  get_bounds(l, bounds);
  for (i = 0; i < 6; i++) {
    f64 m1 = bounds[i].a;
    f64 b1 = bounds[i].b;

    /* x where line intersects with perpendicular running though (0, 0) */
    Bounds line2;
    line2.a = -1.0 / m1;
    line2.b = 0.0;

    f64 x        = intersect_line_line(&bounds[i], &line2);
    f64 distance = dist_from_pole(x, b1 + x * m1);

    if (distance >= 0.0 && distance < min_len)
      min_len = distance;
  }

  return min_len;
}

f64 max_chroma_for_lh(f64 l, f64 h) {
  f64    min_len = FLT_MAX;
  f64    hrad    = h * 0.01745329251994329577; /* (2 * pi / 260) */
  Bounds bounds[6];
  int    i;

  get_bounds(l, bounds);
  for (i = 0; i < 6; i++) {
    f64 l2 = ray_length_until_intersect(hrad, &bounds[i]);

    if (l2 >= 0.0 && l2 < min_len)
      min_len = l2;
  }
  return min_len;
}

/* http://en.wikipedia.org/wiki/CIELUV
 * In these formulas, Yn refers to the reference white point. We are using
 * illuminant D65, so Yn (see refY in Maxima file) equals 1. The formula is
 * simplified accordingly.
 */
f64 y2l(f64 y) {
  if (y <= CIE_EPSILON)
    return y * CIE_KAPPA;
  else
    return 116.0 * cbrt(y) - 16.0;
}

f64 l2y(f64 l) {
  if (l <= 8.0) {
    return l / CIE_KAPPA;
  } else {
    f64 x = (l + 16.0) / 116.0;
    return (x * x * x);
  }
}

seni_colour_64 *luv_from_xyz(seni_colour_64 *col) {
  f64 var_u = (4.0 * col->element[0]) /
              (col->element[0] + (15.0 * col->element[1]) + (3.0 * col->element[2]));
  f64 var_v = (9.0 * col->element[1]) /
              (col->element[0] + (15.0 * col->element[1]) + (3.0 * col->element[2]));
  f64 l = y2l(col->element[1]);
  f64 u = 13.0 * l * (var_u - ref_u);
  f64 v = 13.0 * l * (var_v - ref_v);

  // SENI_LOG("");
  // SENI_LOG("var_u - ref_u %.5f", (var_u - ref_u));
  // SENI_LOG("var_v - ref_v %.5f", (var_v - ref_v));
  // SENI_LOG("var_u %.5f, var_v %.5f", var_u, var_v);
  // SENI_LOG("");

  col->element[0] = l;
  if (l < 0.00000001) {
    col->element[1] = 0.0;
    col->element[2] = 0.0;
  } else {
    col->element[1] = u;
    col->element[2] = v;
  }

  return col;
}

seni_colour_64 *xyz_from_luv(seni_colour_64 *col) {
  if (col->element[0] <= 0.00000001) {
    col->element[0] = 0.0;
    col->element[1] = 0.0;
    col->element[2] = 0.0;
    return col;
  }

  f64 var_u = col->element[1] / (13.0 * col->element[0]) + ref_u;
  f64 var_v = col->element[2] / (13.0 * col->element[0]) + ref_v;
  f64 y     = l2y(col->element[0]);
  f64 x     = -(9.0 * y * var_u) / ((var_u - 4.0) * var_v - var_u * var_v);
  f64 z     = (9.0 * y - (15.0 * var_v * y) - (var_v * x)) / (3.0 * var_v);

  col->element[0] = x;
  col->element[1] = y;
  col->element[2] = z;

  return col;
}

seni_colour_64 *lch_from_luv(seni_colour_64 *col) {
  f64 l = col->element[0];
  f64 u = col->element[1];
  f64 v = col->element[2];
  f64 h;
  f64 c = sqrtf(u * u + v * v);

  // SENI_LOG("lch_from_luv c %.5f", c);

  if (c < 0.00000001f) {
    h = 0.0f;
  } else {
    // SENI_LOG("lch_from_luv atan2f(v, u): %.5f", atan2f(v, u));
    h = atan2f(v, u) * 57.29577951308232087680f; /* (180 / pi) */
    if (h < 0.0f)
      h += 360.0f;
  }

  col->element[0] = l;
  col->element[1] = c;
  col->element[2] = h;

  return col;
}

seni_colour_64 *luv_from_lch(seni_colour_64 *col) {
  f64 hrad = col->element[2] * 0.01745329251994329577f; /* (pi / 180.0) */
  f64 u    = cosf(hrad) * col->element[1];
  f64 v    = sinf(hrad) * col->element[1];

  col->element[1] = u;
  col->element[2] = v;

  return col;
}

seni_colour_64 *lch_from_hsluv(seni_colour_64 *col) {
  f64 h = col->element[0];
  f64 s = col->element[1];
  f64 l = col->element[2];
  f64 c;

  if (l > 99.9999999 || l < 0.00000001)
    c = 0.0;
  else
    c = max_chroma_for_lh(l, h) / 100.0 * s;

  if (s < 0.00000001)
    h = 0.0;

  col->element[0] = l;
  col->element[1] = c;
  col->element[2] = h;

  return col;
}

seni_colour_64 *hsluv_from_lch(seni_colour_64 *col) {
  f64 l = col->element[0];
  f64 c = col->element[1];
  f64 h = col->element[2];
  f64 s;

  if (l > 99.9999999 || l < 0.00000001)
    s = 0.0;
  else
    s = c / max_chroma_for_lh(l, h) * 100.0;

  if (c < 0.00000001)
    h = 0.0;

  col->format     = HSLuv;
  col->element[0] = h;
  col->element[1] = s;
  col->element[2] = l;

  return col;
}

seni_colour_64 *xyz_from_hsluv(seni_colour_64 *hsluv) {
  return xyz_from_luv(luv_from_lch(lch_from_hsluv(hsluv)));
}

seni_colour_64 *hsluv_from_xyz(seni_colour_64 *xyz) {
  return hsluv_from_lch(lch_from_luv(luv_from_xyz(xyz)));
}

seni_colour *colour_clone_as(seni_colour *out, seni_colour *in, seni_colour_format new_format) {
  if (out != in) {
    colour_clone(out, in);
  }

  if (out->format == new_format) {
    return out;
  }

  seni_colour_64 c64;
  colour_64_from_colour(&c64, out);

  switch (c64.format) {
  case HSL:
    switch (new_format) {
    case HSLuv:
      hsluv_from_xyz(xyz_from_rgb(rgb_from_hsl(&c64)));
      break;
    case HSV:
      hsv_from_rgb(rgb_from_hsl(&c64));
      break;
    case LAB:
      lab_from_xyz(xyz_from_rgb(rgb_from_hsl(&c64)));
      break;
    case RGB:
      rgb_from_hsl(&c64);
      break;
    default:
      SENI_ERROR("unknown colour format %d", new_format);
      break;
    }
    break;
  case HSLuv:
    switch (new_format) {
    case HSL:
      hsl_from_rgb(rgb_from_xyz(xyz_from_hsluv(&c64)));
      break;
    case HSV:
      hsv_from_rgb(rgb_from_xyz(xyz_from_hsluv(&c64)));
      break;
    case LAB:
      lab_from_xyz(xyz_from_hsluv(&c64));
      break;
    case RGB:
      rgb_from_xyz(xyz_from_hsluv(&c64));
      break;
    default:
      SENI_ERROR("unknown colour format %d", new_format);
      break;
    }
    break;
  case HSV:
    switch (new_format) {
    case HSL:
      hsl_from_rgb(rgb_from_hsv(&c64));
      break;
    case HSLuv:
      hsluv_from_xyz(xyz_from_rgb(rgb_from_hsv(&c64)));
      break;
    case LAB:
      lab_from_xyz(xyz_from_rgb(rgb_from_hsv(&c64)));
      break;
    case RGB:
      rgb_from_hsv(&c64);
      break;
    default:
      SENI_ERROR("unknown colour format %d", new_format);
      break;
    }
    break;
  case LAB:
    switch (new_format) {
    case HSL:
      hsl_from_rgb(rgb_from_xyz(xyz_from_lab(&c64)));
      break;
    case HSLuv:
      hsluv_from_xyz(xyz_from_lab(&c64));
      break;
    case HSV:
      hsv_from_rgb(rgb_from_xyz(xyz_from_lab(&c64)));
      break;
    case RGB:
      rgb_from_xyz(xyz_from_lab(&c64));
      break;
    default:
      SENI_ERROR("unknown colour format %d", new_format);
      break;
    }
    break;
  case RGB:
    switch (new_format) {
    case HSL:
      hsl_from_rgb(&c64);
      break;
    case HSLuv:
      hsluv_from_xyz(xyz_from_rgb(&c64));
      break;
    case HSV:
      hsv_from_rgb(&c64);
      break;
    case LAB:
      lab_from_xyz(xyz_from_rgb(&c64));
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

  colour_from_colour_64(out, &c64);

  return out;
}
