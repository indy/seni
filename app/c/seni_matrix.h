#pragma once

#include "seni_types.h"

typedef struct {
  f32 m[16];
} seni_matrix;

seni_matrix *matrix_construct();
void matrix_free(seni_matrix* matrix);

void matrix_copy(seni_matrix *out, seni_matrix *a);
void matrix_identity(seni_matrix *out);
void matrix_ortho(seni_matrix *out, f32 left, f32 right, f32 bottom, f32 top, f32 near, f32 far);
void matrix_multiply(seni_matrix *out, seni_matrix *a, seni_matrix *b);
void matrix_scale(seni_matrix *out, seni_matrix *a, f32 x, f32 y, f32 z);
void matrix_translate(seni_matrix *out, seni_matrix *a, f32 x, f32 y, f32 z);
void matrix_rotate_z(seni_matrix *out, seni_matrix *a, f32 rad);

void matrix_transform_vec2(f32 *outx, f32 *outy, seni_matrix *m, f32 x, f32 y);
void matrix_transform_vec3(f32 *outx, f32 *outy, f32 *outz, seni_matrix *m, f32 x, f32 y, f32 z);

#define MATRIX_STACK_SIZE 16

typedef struct {
  // stack
  seni_matrix *stack;
  i32 stack_size;

  i32 sp;

  seni_matrix *wip_transform;        // a matrix for performing calculations
} seni_matrix_stack;

seni_matrix_stack *matrix_stack_construct();
void matrix_stack_free(seni_matrix_stack *matrix_stack);
void matrix_stack_reset(seni_matrix_stack *matrix_stack);
  
seni_matrix *matrix_stack_push(seni_matrix_stack *matrix_stack);
seni_matrix *matrix_stack_pop(seni_matrix_stack *matrix_stack);
seni_matrix *matrix_stack_peek(seni_matrix_stack *matrix_stack);

// modify the top of the matrix stack
// (note: the matrix_stack cannot be empty)
//
void matrix_stack_scale(seni_matrix_stack *matrix_stack, f32 sx, f32 sy);
void matrix_stack_translate(seni_matrix_stack *matrix_stack, f32 tx, f32 ty);
void matrix_stack_rotate(seni_matrix_stack *matrix_stack, f32 a);

void matrix_stack_transform_vec2(f32 *outx, f32 *outy, seni_matrix_stack *matrix_stack, f32 x, f32 y);
void matrix_stack_transform_vec3(f32 *outx, f32 *outy, f32 *outz, seni_matrix_stack *matrix_stack, f32 x, f32 y, f32 z);

