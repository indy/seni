#pragma once

#include "types.h"

bool unparse(senie_cursor*   cursor,
             senie_word_lut* word_lut,
             senie_node*     ast,
             senie_genotype* genotype);
