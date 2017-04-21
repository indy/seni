#ifndef SENI_BIND_H
#define SENI_BIND_H

#include "seni_lang.h"

// a register like seni_var for holding intermediate values
extern seni_var g_reg;

extern i32 g_arg_colour;
extern i32 g_arg_from;
extern i32 g_arg_height;
extern i32 g_arg_increment;
extern i32 g_arg_position;
extern i32 g_arg_radius;
extern i32 g_arg_steps;
extern i32 g_arg_tessellation;
extern i32 g_arg_to;
extern i32 g_arg_upto;
extern i32 g_arg_width;

void interpreter_declare_keywords(word_lut *wl);

#endif
