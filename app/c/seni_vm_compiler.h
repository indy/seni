#ifndef SENI_VM_COMPILER
#define SENI_VM_COMPILER

#include "seni_lang.h"

i32 get_argument_mapping(seni_fn_info *fn_info, i32 wlut_value);
  
void compiler_compile(seni_node *ast, seni_program *program);

#endif
