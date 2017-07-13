#ifndef SENI_VM_INTERPRETER
#define SENI_VM_INTERPRETER

#include "seni_lang.h"

// functions used by the interpreter and the native bindings during run-time

void      var_move(seni_var *dest, seni_var *src);
bool      var_copy(seni_vm *vm, seni_var *dest, seni_var *src);
bool      var_copy_onto_junk(seni_vm *vm, seni_var *dest, seni_var *src);

seni_var *var_get_from_heap(seni_vm *vm);
void      var_return_to_heap(seni_vm *vm,  seni_var *var);
bool      vector_ref_count_decrement(seni_vm *vm, seni_var *vec_head);
void      vector_ref_count_increment(seni_vm *vm, seni_var *vec_head);

void      vector_construct(seni_vm *vm, seni_var *head);
bool      append_to_vector(seni_vm *vm, seni_var *head, seni_var *val);
void      append_to_vector_i32(seni_vm *vm, seni_var *head, i32 val);
void      append_to_vector_f32(seni_vm *vm, seni_var *head, f32 val);
void      append_to_vector_u64(seni_vm *vm, seni_var *head, u64 val);
void      append_to_vector_col(seni_vm *vm, seni_var *head, seni_colour *col);

bool      vm_invoke_no_arg_function(seni_vm *vm, seni_fn_info *fn_info);

bool      vm_interpret(seni_vm *vm, seni_program *program);


#endif
