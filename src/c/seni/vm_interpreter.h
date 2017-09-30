#pragma once

#include "types.h"

// run the program on the vm
bool vm_run(seni_vm *vm, seni_env *env, seni_program *program);

// run the vm to invoke a no arg function
bool vm_setup_and_call_function(seni_vm *vm, seni_fn_info *fn_info);
// setup vm to invoke a single function and then stop
void vm_setup_function(seni_vm *vm, seni_fn_info *fn_info);
// run the vm after it's been setup to invoke a single function
bool vm_call_function(seni_vm *vm);

