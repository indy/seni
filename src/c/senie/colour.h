#pragma once

#include "config.h"
#include "types.h"

// |--------+-----------+-------------+-------------|
// | format | element 0 | element 1   | element 2   |
// |--------+-----------+-------------+-------------|
// | RGB    | R 0..1    | G 0..1      | B 0..1      |
// | HSL    | H 0..360  | S 0..1      | L 0..1      |
// | HSLuv  | H 0..360  | S 0..100    | L 0..100    |
// | LAB    | L 0..100  | A -128..128 | B -128..128 |
// |--------+-----------+-------------+-------------|

typedef enum { RGB, HSL, HSLuv, LAB, HSV, XYZ } senie_colour_format;

struct senie_colour {
  senie_colour_format format;
  f32                 element[4];
};

void colour_set(senie_colour* out, senie_colour_format format, f32 e0, f32 e1, f32 e2, f32 alpha);
senie_colour* colour_clone_as(senie_colour* out, senie_colour* in, senie_colour_format new_format);
