#include "repeat.h"

#include "keyword_iname.h"
#include "lang.h"
#include "mathutil.h"
#include "matrix.h"
#include "vm_compiler.h"
#include "vm_interpreter.h"

void flip(seni_vm *vm, seni_fn_info *fn_info, f32 sx, f32 sy, i32 *copy) {
  seni_matrix_stack *matrix_stack = vm->matrix_stack;

  matrix_stack_push(matrix_stack);
  {
    vm_function_call_default_arguments(vm, fn_info);
    vm_function_set_argument_to_f32(vm, fn_info, INAME_COPY, (f32)*copy);
    vm_function_call_body(vm, fn_info);
  }
  matrix_stack_pop(matrix_stack);
  (*copy)++;

  matrix_stack_push(matrix_stack);
  {
    matrix_stack_scale(matrix_stack, sx, sy);
    vm_function_call_default_arguments(vm, fn_info);
    vm_function_set_argument_to_f32(vm, fn_info, INAME_COPY, (f32)*copy);
    vm_function_call_body(vm, fn_info);
  }
  matrix_stack_pop(matrix_stack);
  (*copy)++;
}

void repeat_symmetry_vertical(seni_vm *vm, i32 fn, i32 *copy) {
  seni_program *program = vm->program;
  seni_fn_info *fn_info = &(program->fn_info[fn]);

  flip(vm, fn_info, -1.0f, 1.0f, copy);
}

void repeat_symmetry_horizontal(seni_vm *vm, i32 fn, i32 *copy) {
  seni_program *program = vm->program;
  seni_fn_info *fn_info = &(program->fn_info[fn]);

  flip(vm, fn_info, 1.0f, -1.0f, copy);
}

void repeat_symmetry_4(seni_vm *vm, i32 fn, i32 *copy) {
  seni_program *program = vm->program;
  seni_fn_info *fn_info = &(program->fn_info[fn]);

  seni_matrix_stack *matrix_stack = vm->matrix_stack;

  matrix_stack_push(matrix_stack);
  flip(vm, fn_info, -1.0f, 1.0f, copy);
  matrix_stack_pop(matrix_stack);

  matrix_stack_push(matrix_stack);
  matrix_stack_scale(matrix_stack, 1.0f, -1.0f);
  flip(vm, fn_info, -1.0f, 1.0f, copy);
  matrix_stack_pop(matrix_stack);
}

void repeat_symmetry_8(seni_vm *vm, i32 fn, i32 *copy) {
  seni_matrix_stack *matrix_stack = vm->matrix_stack;

  matrix_stack_push(matrix_stack);
  repeat_symmetry_4(vm, fn, copy);
  matrix_stack_pop(matrix_stack);

  matrix_stack_push(matrix_stack);
  matrix_stack_rotate(matrix_stack, PI_BY_2);
  repeat_symmetry_4(vm, fn, copy);
  matrix_stack_pop(matrix_stack);
}

void repeat_rotate(seni_vm *vm, i32 fn, i32 copies) {
  seni_program *program = vm->program;
  seni_fn_info *fn_info = &(program->fn_info[fn]);

  seni_matrix_stack *matrix_stack = vm->matrix_stack;

  f32 delta = TAU / (f32)copies;
  f32 angle;

  for (i32 i = 0; i < copies; i++) {
    angle = delta * (f32)i;
    matrix_stack_push(matrix_stack);
    matrix_stack_rotate(matrix_stack, angle);

    vm_function_call_default_arguments(vm, fn_info);
    vm_function_set_argument_to_f32(vm, fn_info, INAME_ANGLE, rad_to_deg(angle));
    vm_function_set_argument_to_f32(vm, fn_info, INAME_COPY, (f32)i);
    vm_function_call_body(vm, fn_info);

    matrix_stack_pop(matrix_stack);
  }
}

void repeat_rotate_mirrored(seni_vm *vm, i32 fn, i32 copies) {
  seni_program *program = vm->program;
  seni_fn_info *fn_info = &(program->fn_info[fn]);

  seni_matrix_stack *matrix_stack = vm->matrix_stack;

  f32 delta = TAU / (f32)copies;
  f32 angle;
  i32 i;
  i32 copy = 0;

  for (i = 0; i < copies; i++) {
    angle = delta * (f32)i;
    matrix_stack_push(matrix_stack);
    matrix_stack_rotate(matrix_stack, angle);

    vm_function_call_default_arguments(vm, fn_info);
    vm_function_set_argument_to_f32(vm, fn_info, INAME_ANGLE, rad_to_deg(angle));
    vm_function_set_argument_to_f32(vm, fn_info, INAME_COPY, (f32)copy++);
    vm_function_call_body(vm, fn_info);

    matrix_stack_pop(matrix_stack);
  }

  matrix_stack_push(matrix_stack);
  matrix_stack_scale(matrix_stack, -1.0f, 1.0f);
  for (i = 0; i < copies; i++) {
    angle = delta * (f32)i;
    matrix_stack_push(matrix_stack);
    matrix_stack_rotate(matrix_stack, angle);

    vm_function_call_default_arguments(vm, fn_info);
    vm_function_set_argument_to_f32(vm, fn_info, INAME_ANGLE, -rad_to_deg(angle));
    vm_function_set_argument_to_f32(vm, fn_info, INAME_COPY, (f32)copy++);
    vm_function_call_body(vm, fn_info);

    matrix_stack_pop(matrix_stack);
  }
  matrix_stack_pop(matrix_stack);
}
