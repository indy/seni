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

senie_program* compile_program(senie_program* program, senie_node* ast);

// just like compile_program except that it binds initial_value to INAME_GEN_INITIAL
senie_program* compile_program_for_trait(senie_program* program,
                                         senie_node*    ast,
                                         senie_node*    gen_initial_value,
                                         i32            vary);

senie_program*
compile_program_with_genotype(senie_program* program, senie_node* ast, senie_genotype* genotype);
