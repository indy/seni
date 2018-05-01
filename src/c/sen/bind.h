#pragma once

#include "types.h"

void declare_bindings(sen_word_lut* wl, sen_env* e);

// get the wlut indexes for the native functions that construct colours
i32 get_colour_constructor_start();
i32 get_colour_constructor_end();
