#ifndef SENI_VM_COMPILER
#define SENI_VM_COMPILER

#include "seni_lang.h"

i32 get_argument_mapping(seni_fn_info *fn_info, i32 wlut_value);
  
seni_program *compile_program(seni_node *ast, i32 program_max_size, seni_word_lut *word_lut);

#endif
