#include "colour.h"

#include "keyword_iname.h"
#include "mathutil.h"

#include <stdlib.h>
// float.h for FLT_MAX
 #include <float.h>
#include <math.h>

#define COLOUR_UNIT_ANGLE (360.0f / 12.0f)
#define COLOUR_COMPLIMENTARY_ANGLE (COLOUR_UNIT_ANGLE * 6.0f)
#define COLOUR_TRIAD_ANGLE (COLOUR_UNIT_ANGLE * 4)

//  http://www.brucelindbloom.com/index.html?Equations.html
//  http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html

// we're using an sRGB working space with a D65 reference white


// https://uk.mathworks.com/help/images/ref/whitepoint.html
// the D65 whitepoint
#define WHITEPOINT_0 0.9504f
#define WHITEPOINT_1 1.0f
#define WHITEPOINT_2 1.0888f

#define CIE_EPSILON 0.008856f
#define CIE_KAPPA 903.3f

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
#endif

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

// http://www.brucelindbloom.com/index.html?Equations.html
// inverse sRGB companding
//
f32 colour_to_axis(f32 component)
{
  f32 temp;
  if (component > 0.04045f) {
    temp = powf((component + 0.055f) / 1.055f, 2.4f);
  } else {
    temp = component / 12.92f;
  }

  return temp;
}

f32 axis_to_colour(f32 a)
{
  if (a > 0.0031308f) {
    return (1.055f * powf(a, 1.0f / 2.4f)) - 0.055f;
  } else {
    return a * 12.92f;
  }
}

seni_colour *xyz_from_rgb(seni_colour *col)
{
  f32 r = colour_to_axis(col->element[0]);
  f32 g = colour_to_axis(col->element[1]);
  f32 b = colour_to_axis(col->element[2]);

  // multiply by matrix
  // see http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html
  // sRGB colour space with D65 reference white
  //
  col->format = XYZ;
  col->element[0] = (r * 0.4124f) + (g * 0.3576f) + (b * 0.1805f);
  col->element[1] = (r * 0.2126f) + (g * 0.7152f) + (b * 0.0722f);
  col->element[2] = (r * 0.0193f) + (g * 0.1192f) + (b * 0.9505f);
  
  return col;
}

seni_colour *rgb_from_xyz(seni_colour *col)
{
  f32 xx = col->element[0];
  f32 yy = col->element[1];
  f32 zz = col->element[2];

  f32 r = (xx *  3.2406f) + (yy * -1.5372f) + (zz * -0.4986f);
  f32 g = (xx * -0.9689f) + (yy *  1.8758f) + (zz *  0.0415f);
  f32 b = (xx *  0.0557f) + (yy * -0.2040f) + (zz *  1.0570f);

  col->format = RGB;
  col->element[0] = axis_to_colour(r);
  col->element[1] = axis_to_colour(g);
  col->element[2] = axis_to_colour(b);

  return col;
}

f32 axis_to_LAB_component(f32 a)
{
  if (a > CIE_EPSILON) {
    return (f32)cbrt(a);        // cube root
  } else {
    return ((CIE_KAPPA * a) + 16.0f) / 116.0f;
  }
}

seni_colour *lab_from_xyz(seni_colour *col)
{
  f32 xr = col->element[0] / WHITEPOINT_0;
  f32 yr = col->element[1] / WHITEPOINT_1;
  f32 zr = col->element[2] / WHITEPOINT_2;
  
  f32 fx = axis_to_LAB_component(xr);
  f32 fy = axis_to_LAB_component(yr);
  f32 fz = axis_to_LAB_component(zr);

  col->format = LAB;
  col->element[0] = (116.0f * fy) - 16.0f;
  col->element[1] = 500.0f * (fx - fy);
  col->element[2] = 200.0f * (fy - fz);

  return col;
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

// http://www.rapidtables.com/convert/color/rgb-to-hsl.htm
f32 hue(seni_colour *colour, i32 max_chan, f32 chroma)
{
  if (chroma == 0.0f) {
    return 0.0f;        // invalid hue
  }

  f32 angle = 0.0f;
  
  switch (max_chan) {
  case 0:                       // R
    angle  = 60.0f * ((f32)fmod((colour->element[1] - colour->element[2]) / chroma, 6.0f));
    break;
  case 1:                       // G
    angle = 60.0f * (((colour->element[2] - colour->element[0]) / chroma) + 2.0f);
    break;
  case 2:                       // B
    angle = 60.0f * (((colour->element[0] - colour->element[1]) / chroma) + 4.0f);
    break;
  default:
    break;
  }

  while (angle < 0.0f) {
    angle += 360.0f;
  }

  return angle;
}

f32 abso(f32 in)
{
  return in < 0.0f ? -in : in;
}

// http://www.rapidtables.com/convert/color/rgb-to-hsl.htm
seni_colour *hsl_from_rgb(seni_colour *col)
{
  i32 min_ch = min_channel(col);
  f32 min_val = col->element[min_ch];

  i32 max_ch = max_channel(col);
  f32 max_val = col->element[max_ch];

  f32 delta = max_val - min_val;

  
  f32 h = hue(col, max_ch, delta);


  f32 lightness = 0.5f * (min_val + max_val);

  f32 saturation;
  if (delta == 0.0f) {
    saturation = 0.0f;
  } else {
    saturation = delta / (1.0f - abso((2.0f * lightness) - 1.0f));
  }

  col->format = HSL;
  col->element[0] = h;
  col->element[1] = saturation;
  col->element[2] = lightness;

  return col;
}

seni_colour *hsv_from_rgb(seni_colour *col)
{
  i32 min_ch = min_channel(col);
  f32 min_val = col->element[min_ch];

  i32 max_ch = max_channel(col);
  f32 max_val = col->element[max_ch];

  f32 chroma = max_val - min_val;
  f32 h = hue(col, max_ch, chroma);
  // bool valid_hue = (chroma != 0.0);

  f32 value = max_val;

  f32 saturation;
  if (chroma == 0.0f) {
    saturation = 0.0f;
  } else {
    saturation = chroma / value;
  }
  
  col->format = HSV;
  col->element[0] = h;
  col->element[1] = saturation;
  col->element[2] = value;

  // TODO: set valid_hue
  // return col.set('valid_hue', valid_hue);

  return col;
}

seni_colour *rgb_from_chm(seni_colour *col, f32 chroma, f32 h, f32 m)
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

  col->format = RGB;
  col->element[0] = r + m;
  col->element[1] = g + m;
  col->element[2] = b + m;

  return col;
}

seni_colour *rgb_from_hsl(seni_colour *col)
{
  f32 h = col->element[0];
  f32 s = col->element[1];
  f32 l = col->element[2];
  f32 chroma = (1.0f - abso((2.0f * l) - 1.0f)) * s;
  f32 m = l - (0.5f * chroma);

  // todo: set validhue
  // f32 col = c.set('validHue', true);

  return rgb_from_chm(col, chroma, h, m);
}

f32 lab_component_to_axis(f32 l)
{
  if (powf(l, 3.0f) > CIE_EPSILON) {
    return powf(l, 3.0f);
  } else {
    return ((116.0f * l) - 16.0f) / CIE_KAPPA;
  }
}

seni_colour *xyz_from_lab(seni_colour *col)
{
  f32 fy = (col->element[0] + 16.0f) / 116.0f;
  f32 fz = fy - (col->element[2] / 200.0f);
  f32 fx = (col->element[1] / 500.0f) + fy;

  f32 xr = lab_component_to_axis(fx);
  f32 yr = 0.0f;
  if (col->element[0] > (CIE_EPSILON * CIE_KAPPA)) {
    yr = ((col->element[0] + 16.0f) / 116.0f);
    yr = yr * yr * yr;
  } else {
    yr = col->element[0] / CIE_KAPPA;
  }
  f32 zr = lab_component_to_axis(fz);

  col->format = XYZ;
  col->element[0] = WHITEPOINT_0 * xr;
  col->element[1] = WHITEPOINT_1 * yr;
  col->element[2] = WHITEPOINT_2 * zr;

  return col;
}

seni_colour *rgb_from_hsv(seni_colour *col)
{
  f32 h = col->element[0];
  f32 s = col->element[1];
  f32 v = col->element[2];
  f32 chroma = v * s;
  f32 m = v - chroma;

  return rgb_from_chm(col, chroma, h, m);
}


// the luv and hsluv code is based on https://github.com/hsluv/hsluv-c
// which uses the MIT License:

// # The MIT License (MIT)

// Copyright © 2015 Alexei Boronine (original idea, JavaScript implementation)  
// Copyright © 2015 Roger Tallada (Obj-C implementation)  
// Copyright © 2017 Martin Mitáš (C implementation, based on Obj-C implementation)  

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
  f32 a;
  f32 b;
};

void get_bounds(f32 l, Bounds bounds[6])
{
  f32 tl = l + 16.0f;
  f32 sub1 = (tl * tl * tl) / 1560896.0f;
  f32 sub2 = sub1 > CIE_EPSILON ? sub1 : (l / CIE_KAPPA);
  int channel;
  int t;

  f32 m[3][3];
  m[0][0] =  3.24096994190452134377f; m[0][1] = -1.53738317757009345794f; m[0][2] = -0.49861076029300328366f;
  m[1][0] = -0.96924363628087982613f; m[1][1] =  1.87596750150772066772f; m[1][2] =  0.04155505740717561247f;
  m[2][0] =  0.05563007969699360846f; m[2][1] = -0.20397695888897656435f; m[2][2] =  1.05697151424287856072f;
  
  
  for(channel = 0; channel < 3; channel++) {
    f32 m1 = m[channel][0];
    f32 m2 = m[channel][1];
    f32 m3 = m[channel][2];

    for (t = 0; t < 2; t++) {
      f32 top1 = (284517.0f * m1 - 94839.0f * m3) * sub2;
      f32 top2 = (838422.0f * m3 + 769860.0f * m2 + 731718.0f * m1) * l * sub2 - 769860.0f * t * l;
      f32 bottom = (632260.0f * m3 - 126452.0f * m2) * sub2 + 126452.0f * t;

      bounds[channel * 2 + t].a = top1 / bottom;
      bounds[channel * 2 + t].b = top2 / bottom;
    }
  }
}

f32 intersect_line_line(const Bounds* line1, const Bounds* line2)
{
  return (line1->b - line2->b) / (line2->a - line1->a);
}

f32 dist_from_pole(f32 x, f32 y)
{
  return sqrtf(x * x + y * y);
}

f32 ray_length_until_intersect(f32 theta, const Bounds* line)
{
  return line->b / (sinf(theta) - line->a * cosf(theta));
}

f32 max_safe_chroma_for_l(f32 l)
{
  f32 min_len = FLT_MAX;
  Bounds bounds[6];
  int i;

  get_bounds(l, bounds);
  for(i = 0; i < 6; i++) {
    f32 m1 = bounds[i].a;
    f32 b1 = bounds[i].b;

    /* x where line intersects with perpendicular running though (0, 0) */
    Bounds line2;
    line2.a = -1.0f / m1;
    line2.b = 0.0f;
        
    f32 x = intersect_line_line(&bounds[i], &line2);
    f32 distance = dist_from_pole(x, b1 + x * m1);

    if(distance >= 0.0f && distance < min_len)
      min_len = distance;
  }

  return min_len;
}

f32 max_chroma_for_lh(f32 l, f32 h)
{
  f32 min_len = FLT_MAX;
  f32 hrad = h * 0.01745329251994329577f; /* (2 * pi / 260) */
  Bounds bounds[6];
  int i;

  get_bounds(l, bounds);
  for(i = 0; i < 6; i++) {
    f32 l2 = ray_length_until_intersect(hrad, &bounds[i]);

    if(l2 >= 0.0f  &&  l2 < min_len)
      min_len = l2;
  }
  return min_len;
}

/* http://en.wikipedia.org/wiki/CIELUV
 * In these formulas, Yn refers to the reference white point. We are using
 * illuminant D65, so Yn (see refY in Maxima file) equals 1. The formula is
 * simplified accordingly.
 */
f32 y2l(f32 y)
{
  if(y <= CIE_EPSILON)
    return y * CIE_KAPPA;
  else
    return 116.0f * (f32)cbrt(y) - 16.0f;
}

f32 l2y(f32 l)
{
  if(l <= 8.0f) {
    return l / CIE_KAPPA;
  } else {
    f32 x = (l + 16.0f) / 116.0f;
    return (x * x * x);
  }
}

const f32 ref_u = 0.19783000664283680764f;
const f32 ref_v = 0.46831999493879100370f;

seni_colour* luv_from_xyz(seni_colour* col)
{
  f32 var_u = (4.0f * col->element[0]) / (col->element[0] + (15.0f * col->element[1]) + (3.0f * col->element[2]));
  f32 var_v = (9.0f * col->element[1]) / (col->element[0] + (15.0f * col->element[1]) + (3.0f * col->element[2]));
  f32 l = y2l(col->element[1]);
  f32 u = 13.0f * l * (var_u - ref_u);
  f32 v = 13.0f * l * (var_v - ref_v);

  col->element[0] = l;
  if(l < 0.00000001f) {
    col->element[1] = 0.0f;
    col->element[2] = 0.0f;
  } else {
    col->element[1] = u;
    col->element[2] = v;
  }

  return col;
}

seni_colour* xyz_from_luv(seni_colour* col)
{
  if(col->element[0] <= 0.00000001f) {
    col->element[0] = 0.0f;
    col->element[1] = 0.0f;
    col->element[2] = 0.0f;
    return col;
  }

  f32 var_u = col->element[1] / (13.0f * col->element[0]) + ref_u;
  f32 var_v = col->element[2] / (13.0f * col->element[0]) + ref_v;
  f32 y = l2y(col->element[0]);
  f32 x = -(9.0f * y * var_u) / ((var_u - 4.0f) * var_v - var_u * var_v);
  f32 z = (9.0f * y - (15.0f * var_v * y) - (var_v * x)) / (3.0f * var_v);
    
  col->element[0] = x;
  col->element[1] = y;
  col->element[2] = z;

  return col;
}

seni_colour* lch_from_luv(seni_colour* col)
{
  f32 l = col->element[0];
  f32 u = col->element[1];
  f32 v = col->element[2];
  f32 h;
  f32 c = sqrtf(u * u + v * v);

  if(c < 0.00000001f) {
    h = 0.0f;
  } else {
    h = atan2f(v, u) * 57.29577951308232087680f;  /* (180 / pi) */
    if(h < 0.0f)
      h += 360.0f;
  }

  col->element[0] = l;
  col->element[1] = c;
  col->element[2] = h;

  return col;
}

seni_colour* luv_from_lch(seni_colour* col)
{
  f32 hrad = col->element[2] * 0.01745329251994329577f;  /* (pi / 180.0) */
  f32 u = cosf(hrad) * col->element[1];
  f32 v = sinf(hrad) * col->element[1];

  col->element[1] = u;
  col->element[2] = v;

  return col;
}

seni_colour* lch_from_hsluv(seni_colour* col)
{
  f32 h = col->element[0];
  f32 s = col->element[1];
  f32 l = col->element[2];
  f32 c;

  if(l > 99.9999999f || l < 0.00000001f)
    c = 0.0f;
  else
    c = max_chroma_for_lh(l, h) / 100.0f * s;

  if (s < 0.00000001f)
    h = 0.0f;

  col->element[0] = l;
  col->element[1] = c;
  col->element[2] = h;

  return col;
}

seni_colour* hsluv_from_lch(seni_colour* col)
{
  f32 l = col->element[0];
  f32 c = col->element[1];
  f32 h = col->element[2];
  f32 s;

  if(l > 99.9999999f || l < 0.00000001f)
    s = 0.0f;
  else
    s = c / max_chroma_for_lh(l, h) * 100.0f;

  if (c < 0.00000001f)
    h = 0.0f;

  col->format = HSLuv;
  col->element[0] = h;
  col->element[1] = s;
  col->element[2] = l;

  return col;
}

seni_colour* xyz_from_hsluv(seni_colour* hsluv)
{
  return xyz_from_luv(luv_from_lch(lch_from_hsluv(hsluv)));
}

seni_colour* hsluv_from_xyz(seni_colour* xyz)
{
  return hsluv_from_lch(lch_from_luv(luv_from_xyz(xyz)));
}

seni_colour *colour_clone_as(seni_colour *out, seni_colour *in, seni_colour_format new_format)
{
  if (out != in) {
    colour_clone(out, in);
  }
  
  switch(out->format) {
  case HSL:
    switch(new_format) {
    case HSL:
      return out;
    case HSLuv:
      return hsluv_from_xyz(xyz_from_rgb(rgb_from_hsl(out)));
    case HSV:
      return hsv_from_rgb(rgb_from_hsl(out));
    case LAB:
      return lab_from_xyz(xyz_from_rgb(rgb_from_hsl(out)));
    case RGB:
      return rgb_from_hsl(out);
    default:
      SENI_ERROR("unknown colour format %d", new_format);
      break;
    }    
    break;
  case HSLuv:
    switch(new_format) {
    case HSL:
      return hsl_from_rgb(rgb_from_xyz(xyz_from_hsluv(out)));
    case HSLuv:
      return out;
    case HSV:
      return hsv_from_rgb(rgb_from_xyz(xyz_from_hsluv(out)));
    case LAB:
      return lab_from_xyz(xyz_from_hsluv(out));
    case RGB:
      return rgb_from_xyz(xyz_from_hsluv(out));
    default:
      SENI_ERROR("unknown colour format %d", new_format);
      break;
    }    
    break;
  case HSV:
    switch(new_format) {
    case HSL:
      return hsl_from_rgb(rgb_from_hsv(out));
    case HSLuv:
      return hsluv_from_xyz(xyz_from_rgb(rgb_from_hsv(out)));
    case HSV:
      return out;
    case LAB:
      return lab_from_xyz(xyz_from_rgb(rgb_from_hsv(out)));
    case RGB:
      return rgb_from_hsv(out);
    default:
      SENI_ERROR("unknown colour format %d", new_format);
      break;
    }
    break;
  case LAB:
    switch(new_format) {
    case HSL:
      return hsl_from_rgb(rgb_from_xyz(xyz_from_lab(out)));
    case HSLuv:
      return hsluv_from_xyz(xyz_from_lab(out));
    case HSV:
      return hsv_from_rgb(rgb_from_xyz(xyz_from_lab(out)));
    case LAB:
      return out;
    case RGB:
      return rgb_from_xyz(xyz_from_lab(out));
    default:
      SENI_ERROR("unknown colour format %d", new_format);
      break;
    }
    break;
  case RGB:
    switch(new_format) {
    case HSL:
      return hsl_from_rgb(out);
    case HSLuv:
      return hsluv_from_xyz(xyz_from_rgb(out));
    case HSV:
      return hsv_from_rgb(out);
    case LAB:
      return lab_from_xyz(xyz_from_rgb(out));
    case RGB:
      return out;
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
