#pragma once

#include "types.h"

bool vm_interpret(senie_vm* vm, senie_env* env, senie_program* program);

// run the program on the vm
bool vm_run(senie_vm* vm, senie_env* env, senie_program* program);

// setup vm to invoke a single function and then stop
void vm_function_call_default_arguments(senie_vm* vm, senie_fn_info* fn_info);
void vm_function_set_argument_to_var(senie_vm*      vm,
                                     senie_fn_info* fn_info,
                                     i32            iname,
                                     senie_var*     src);
void vm_function_set_argument_to_f32(senie_vm* vm, senie_fn_info* fn_info, i32 iname, f32 f);
void vm_function_set_argument_to_2d(senie_vm* vm, senie_fn_info* fn_info, i32 iname, f32 x, f32 y);
void vm_function_call_body(senie_vm* vm, senie_fn_info* fn_info);
