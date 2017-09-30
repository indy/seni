#pragma once

#include "types.h"

void declare_bindings(seni_word_lut *wl, seni_env *e);

// get the wlut indexes for the native functions that construct colours
i32 get_colour_constructor_start();
i32 get_colour_constructor_end();
