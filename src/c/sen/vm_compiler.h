#pragma once

#include "types.h"

struct sen_compiler_config {
  i32           program_max_size;
  sen_word_lut* word_lut;
  i32           vary;
};

void compiler_subsystem_startup();
void compiler_subsystem_shutdown();

sen_program* get_preamble_program();

sen_program* compile_program(sen_program* program, sen_node* ast);

// just like compile_program except that it binds initial_value to INAME_GEN_INITIAL
sen_program* compile_program_for_trait(sen_program* program,
                                       sen_node*    ast,
                                       sen_node*    gen_initial_value,
                                       i32          vary);

sen_program*
compile_program_with_genotype(sen_program* program, sen_node* ast, sen_genotype* genotype);
