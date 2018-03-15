#pragma once

#include "types.h"

void compiler_subsystem_startup();
void compiler_subsystem_shutdown();

senie_program* get_preamble_program();

i32 get_argument_mapping(senie_fn_info* fn_info, i32 word_lut_value);

senie_program* compile_program(senie_node* ast, i32 program_max_size, senie_word_lut* word_lut);

senie_program* compile_program_with_genotype(senie_node*     ast,
                                             i32             program_max_size,
                                             senie_word_lut* word_lut,
                                             senie_genotype* genotype);
