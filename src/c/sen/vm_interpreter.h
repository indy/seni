#pragma once

#include "types.h"

bool vm_interpret(sen_vm* vm, sen_env* env, sen_program* program);

// run the program on the vm
bool vm_run(sen_vm* vm, sen_env* env, sen_program* program);

// setup vm to invoke a single function and then stop
void vm_function_call_default_arguments(sen_vm* vm, sen_fn_info* fn_info);
void vm_function_set_argument_to_var(sen_vm* vm, sen_fn_info* fn_info, i32 iname, sen_var* src);
void vm_function_set_argument_to_f32(sen_vm* vm, sen_fn_info* fn_info, i32 iname, f32 f);
void vm_function_set_argument_to_2d(sen_vm* vm, sen_fn_info* fn_info, i32 iname, f32 x, f32 y);
void vm_function_call_body(sen_vm* vm, sen_fn_info* fn_info);
