#pragma once

#include "types.h"

struct senie_matrix {
  f32 m[16];
};

void matrix_copy(senie_matrix* out, senie_matrix* a);
void matrix_identity(senie_matrix* out);
void matrix_ortho(senie_matrix* out, f32 left, f32 right, f32 bottom, f32 top, f32 near, f32 far);
void matrix_multiply(senie_matrix* out, senie_matrix* a, senie_matrix* b);
void matrix_scale(senie_matrix* out, senie_matrix* a, f32 x, f32 y, f32 z);
void matrix_translate(senie_matrix* out, senie_matrix* a, f32 x, f32 y, f32 z);
void matrix_rotate_z(senie_matrix* out, senie_matrix* a, f32 rad);

void matrix_transform_vec2(f32* outx, f32* outy, senie_matrix* m, f32 x, f32 y);
void matrix_transform_vec3(f32* outx, f32* outy, f32* outz, senie_matrix* m, f32 x, f32 y, f32 z);

#define MATRIX_STACK_SIZE 16

struct senie_matrix_stack {
  // stack
  senie_matrix* stack;
  i32           stack_size;

  i32 sp;

  senie_matrix* wip_transform; // a matrix for performing calculations
};

senie_matrix_stack* matrix_stack_allocate();
void                matrix_stack_free(senie_matrix_stack* matrix_stack);
void                matrix_stack_reset(senie_matrix_stack* matrix_stack);

senie_matrix* matrix_stack_push(senie_matrix_stack* matrix_stack);
senie_matrix* matrix_stack_pop(senie_matrix_stack* matrix_stack);
senie_matrix* matrix_stack_peek(senie_matrix_stack* matrix_stack);

// modify the top of the matrix stack
// (note: the matrix_stack cannot be empty)
//
void matrix_stack_scale(senie_matrix_stack* matrix_stack, f32 sx, f32 sy);
void matrix_stack_translate(senie_matrix_stack* matrix_stack, f32 tx, f32 ty);
void matrix_stack_rotate(senie_matrix_stack* matrix_stack, f32 a);

void matrix_stack_transform_vec2(f32*                outx,
                                 f32*                outy,
                                 senie_matrix_stack* matrix_stack,
                                 f32                 x,
                                 f32                 y);
void matrix_stack_transform_vec3(f32*                outx,
                                 f32*                outy,
                                 f32*                outz,
                                 senie_matrix_stack* matrix_stack,
                                 f32                 x,
                                 f32                 y,
                                 f32                 z);
