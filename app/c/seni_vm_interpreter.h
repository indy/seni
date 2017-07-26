#ifndef SENI_VM_INTERPRETER
#define SENI_VM_INTERPRETER

#include "seni_lang.h"

bool vm_invoke_no_arg_function(seni_vm *vm, seni_fn_info *fn_info);

// setup vm to invoke a single function and then stop
void vm_setup_function_invoke(seni_vm *vm, seni_fn_info *fn_info);
// run the vm after it's been setup to invoke a single function
bool vm_function_invoke(seni_vm *vm);

bool vm_interpret(seni_vm *vm, seni_env *env, seni_program *program);


#endif
