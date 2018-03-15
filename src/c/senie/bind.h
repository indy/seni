#pragma once

#include "types.h"

void declare_bindings(senie_word_lut* wl, senie_env* e);

// get the wlut indexes for the native functions that construct colours
i32 get_colour_constructor_start();
i32 get_colour_constructor_end();
