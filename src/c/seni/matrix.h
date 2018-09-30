#pragma once

#include "types.h"

struct sen_matrix {
  f32 m[16];
};

void matrix_copy(sen_matrix* out, sen_matrix* a);
void matrix_identity(sen_matrix* out);
void matrix_ortho(sen_matrix* out, f32 left, f32 right, f32 bottom, f32 top,
                  f32 near, f32 far);
void matrix_multiply(sen_matrix* out, sen_matrix* a, sen_matrix* b);
void matrix_scale(sen_matrix* out, sen_matrix* a, f32 x, f32 y, f32 z);
void matrix_translate(sen_matrix* out, sen_matrix* a, f32 x, f32 y, f32 z);
void matrix_rotate_z(sen_matrix* out, sen_matrix* a, f32 rad);

void matrix_transform_vec2(f32* outx, f32* outy, sen_matrix* m, f32 x, f32 y);
void matrix_transform_vec3(f32* outx, f32* outy, f32* outz, sen_matrix* m,
                           f32 x, f32 y, f32 z);

#define MATRIX_STACK_SIZE 16

struct sen_matrix_stack {
  // stack
  sen_matrix* stack;
  i32         stack_size;

  i32 sp;

  sen_matrix* wip_transform; // a matrix for performing calculations
};

sen_matrix_stack* matrix_stack_allocate();
void              matrix_stack_free(sen_matrix_stack* matrix_stack);
void              matrix_stack_reset(sen_matrix_stack* matrix_stack);

sen_matrix* matrix_stack_push(sen_matrix_stack* matrix_stack);
sen_matrix* matrix_stack_pop(sen_matrix_stack* matrix_stack);
sen_matrix* matrix_stack_peek(sen_matrix_stack* matrix_stack);

// modify the top of the matrix stack
// (note: the matrix_stack cannot be empty)
//
void matrix_stack_scale(sen_matrix_stack* matrix_stack, f32 sx, f32 sy);
void matrix_stack_translate(sen_matrix_stack* matrix_stack, f32 tx, f32 ty);
void matrix_stack_rotate(sen_matrix_stack* matrix_stack, f32 a);

void matrix_stack_transform_vec2(f32* outx, f32* outy,
                                 sen_matrix_stack* matrix_stack, f32 x, f32 y);
void matrix_stack_transform_vec3(f32* outx, f32* outy, f32* outz,
                                 sen_matrix_stack* matrix_stack, f32 x, f32 y,
                                 f32 z);
