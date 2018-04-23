#pragma once

#include "types.h"

struct senie_compiler_config {
  i32             program_max_size;
  senie_word_lut* word_lut;
  i32             vary;
};

void compiler_subsystem_startup();
void compiler_subsystem_shutdown();

senie_program* get_preamble_program();

i32 get_argument_mapping(senie_fn_info* fn_info, i32 word_lut_value);
i32 get_global_mapping(senie_program* program, i32 word_lut_value);

senie_program* compile_program(senie_node* ast, senie_compiler_config* compiler_config);

// just like compile_program except that it binds initial_value to INAME_GEN_INITIAL
senie_program* compile_program_for_trait(senie_node*            ast,
                                         senie_compiler_config* compiler_config,
                                         senie_node*            gen_initial_value);

senie_program* compile_program_with_genotype(senie_node*            ast,
                                             senie_compiler_config* compiler_config,
                                             senie_genotype*        genotype);
