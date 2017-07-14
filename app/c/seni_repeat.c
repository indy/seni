#include "seni_repeat.h"
#include "seni_vm_interpreter.h"
#include "seni_matrix.h"
#include "seni_mathutil.h"


void flip(seni_vm *vm, seni_fn_info *fn_info, f32 sx, f32 sy)
{
  seni_matrix_stack *matrix_stack = vm->matrix_stack;
  
  matrix_stack_push(matrix_stack);
  vm_invoke_no_arg_function(vm, fn_info);
  matrix_stack_pop(matrix_stack);

  matrix_stack_push(matrix_stack);
  matrix_stack_scale(matrix_stack, sx, sy);
  vm_invoke_no_arg_function(vm, fn_info);
  matrix_stack_pop(matrix_stack);
}

void repeat_symmetry_vertical(seni_vm *vm, i32 draw)
{
  seni_program *program = vm->program;
  seni_fn_info *fn_info = &(program->fn_info[draw]);

  flip(vm, fn_info, -1.0f, 1.0f);
}

void repeat_symmetry_horizontal(seni_vm *vm, i32 draw)
{
  seni_program *program = vm->program;
  seni_fn_info *fn_info = &(program->fn_info[draw]);

  flip(vm, fn_info, 1.0f, -1.0f);
}

void repeat_symmetry_4(seni_vm *vm, i32 draw)
{
  seni_program *program = vm->program;
  seni_fn_info *fn_info = &(program->fn_info[draw]);

  seni_matrix_stack *matrix_stack = vm->matrix_stack;
  
  matrix_stack_push(matrix_stack);
  flip(vm, fn_info, -1.0f, 1.0f);
  matrix_stack_pop(matrix_stack);

  matrix_stack_push(matrix_stack);
  matrix_stack_scale(matrix_stack, 1.0f, -1.0f);
  flip(vm, fn_info, -1.0f, 1.0f);  
  matrix_stack_pop(matrix_stack);
}

void repeat_symmetry_8(seni_vm *vm, i32 draw)
{
  seni_matrix_stack *matrix_stack = vm->matrix_stack;
  
  matrix_stack_push(matrix_stack);
  repeat_symmetry_4(vm, draw);
  matrix_stack_pop(matrix_stack);

  matrix_stack_push(matrix_stack);
  matrix_stack_rotate(matrix_stack, PI_BY_2);
  repeat_symmetry_4(vm, draw);
  matrix_stack_pop(matrix_stack);
}

void repeat_rotate(seni_vm *vm, i32 draw, i32 copies)
{
  seni_program *program = vm->program;
  seni_fn_info *fn_info = &(program->fn_info[draw]);

  seni_matrix_stack *matrix_stack = vm->matrix_stack;

  f32 delta = TAU / (f32)copies;

  for(i32 i = 0; i < copies; i++) {
    matrix_stack_push(matrix_stack);
    matrix_stack_rotate(matrix_stack, delta * (f32)i);
    vm_invoke_no_arg_function(vm, fn_info);    
    matrix_stack_pop(matrix_stack);
  }
}

void repeat_rotate_mirrored(seni_vm *vm, i32 draw, i32 copies)
{
  seni_program *program = vm->program;
  seni_fn_info *fn_info = &(program->fn_info[draw]);

  seni_matrix_stack *matrix_stack = vm->matrix_stack;

  f32 delta = TAU / (f32)copies;
  i32 i;

  for(i = 0; i < copies; i++) {
    matrix_stack_push(matrix_stack);
    matrix_stack_rotate(matrix_stack, delta * (f32)i);
    vm_invoke_no_arg_function(vm, fn_info);    
    matrix_stack_pop(matrix_stack);
  }

  matrix_stack_push(matrix_stack);
  matrix_stack_scale(matrix_stack, -1.0f, 1.0f);
  for(i = 0; i < copies; i++) {
    matrix_stack_push(matrix_stack);
    matrix_stack_rotate(matrix_stack, delta * (f32)i);
    vm_invoke_no_arg_function(vm, fn_info);    
    matrix_stack_pop(matrix_stack);
  }
  matrix_stack_pop(matrix_stack);
}
