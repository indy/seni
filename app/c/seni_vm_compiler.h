#pragma once

#include "seni_types.h"

void compiler_subsystem_startup();
void compiler_subsystem_shutdown();

seni_program *get_preamble_program();

i32 get_argument_mapping(seni_fn_info *fn_info, i32 word_lut_value);
  
seni_program *compile_program(seni_node *ast, i32 program_max_size, seni_word_lut *word_lut);

seni_program *compile_program_with_genotype(seni_node *ast, i32 program_max_size, seni_word_lut *word_lut, seni_genotype *genotype);

