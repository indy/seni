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

typedef enum { RGB, HSL, HSLuv, LAB, HSV, XYZ } sen_colour_format;

struct sen_colour {
  sen_colour_format format;
  f32               element[4];
};

void colour_set(sen_colour* out, sen_colour_format format, f32 e0, f32 e1, f32 e2, f32 alpha);
sen_colour* colour_clone_as(sen_colour* out, sen_colour* in, sen_colour_format new_format);
