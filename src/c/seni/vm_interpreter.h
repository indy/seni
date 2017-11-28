#pragma once

#include "types.h"

bool vm_interpret(seni_vm* vm, seni_env* env, seni_program* program);

// run the program on the vm
bool vm_run(seni_vm* vm, seni_env* env, seni_program* program);

// setup vm to invoke a single function and then stop
void vm_function_call_default_arguments(seni_vm* vm, seni_fn_info* fn_info);
void vm_function_set_argument_to_var(seni_vm* vm, seni_fn_info* fn_info, i32 iname, seni_var* src);
void vm_function_set_argument_to_f32(seni_vm* vm, seni_fn_info* fn_info, i32 iname, f32 f);
void vm_function_set_argument_to_2d(seni_vm* vm, seni_fn_info* fn_info, i32 iname, f32 x, f32 y);
void vm_function_call_body(seni_vm* vm, seni_fn_info* fn_info);
