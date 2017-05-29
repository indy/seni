#include "seni_matrix.h"
#include "seni_config.h"
#include "stdlib.h"
#include <math.h>

seni_matrix matrix_construct()
{
  seni_matrix out = (seni_matrix )calloc(16, sizeof(f32));
  return out;
}

void matrix_free(seni_matrix matrix)
{
  free(matrix);
}

void matrix_copy(seni_matrix out, seni_matrix a)
{
  out[0] = a[0];
  out[1] = a[1];
  out[2] = a[2];
  out[3] = a[3];
  out[4] = a[4];
  out[5] = a[5];
  out[6] = a[6];
  out[7] = a[7];
  out[8] = a[8];
  out[9] = a[9];
  out[10] = a[10];
  out[11] = a[11];
  out[12] = a[12];
  out[13] = a[13];
  out[14] = a[14];
  out[15] = a[15];
}

void matrix_identity(seni_matrix out)
{
  out[0] = 1.0f;
  out[1] = 0.0f;
  out[2] = 0.0f;
  out[3] = 0.0f;
  out[4] = 0.0f;
  out[5] = 1.0f;
  out[6] = 0.0f;
  out[7] = 0.0f;
  out[8] = 0.0f;
  out[9] = 0.0f;
  out[10] = 1.0f;
  out[11] = 0.0f;
  out[12] = 0.0f;
  out[13] = 0.0f;
  out[14] = 0.0f;
  out[15] = 1.0f;
}

void matrix_ortho(seni_matrix out, f32 left, f32 right, f32 bottom, f32 top, f32 near, f32 far)
{
  f32 lr = 1.0f / (left - right);
  f32 bt = 1.0f / (bottom - top);
  f32 nf = 1.0f / (near - far);

  out[0] = -2.0f * lr;
  out[1] = 0.0f;
  out[2] = 0.0f;
  out[3] = 0.0f;
  out[4] = 0.0f;
  out[5] = -2.0 * bt;
  out[6] = 0.0f;
  out[7] = 0.0f;
  out[8] = 0.0f;
  out[9] = 0.0f;
  out[10] = 2.0f * nf;
  out[11] = 0.0f;
  out[12] = (left + right) * lr;
  out[13] = (top + bottom) * bt;
  out[14] = (far + near) * nf;
  out[15] = 1.0f;
}

void matrix_multiply(seni_matrix out, seni_matrix a, seni_matrix b)
{
  f32 a00 = a[0], a01 = a[1], a02 = a[2], a03 = a[3];
  f32 a10 = a[4], a11 = a[5], a12 = a[6], a13 = a[7];
  f32 a20 = a[8], a21 = a[9], a22 = a[10], a23 = a[11];
  f32 a30 = a[12], a31 = a[13], a32 = a[14], a33 = a[15];

  // Cache only the current line of the second matrix
  f32 b0  = b[0], b1 = b[1], b2 = b[2], b3 = b[3];
  out[0] = b0*a00 + b1*a10 + b2*a20 + b3*a30;
  out[1] = b0*a01 + b1*a11 + b2*a21 + b3*a31;
  out[2] = b0*a02 + b1*a12 + b2*a22 + b3*a32;
  out[3] = b0*a03 + b1*a13 + b2*a23 + b3*a33;

  b0 = b[4]; b1 = b[5]; b2 = b[6]; b3 = b[7];
  out[4] = b0*a00 + b1*a10 + b2*a20 + b3*a30;
  out[5] = b0*a01 + b1*a11 + b2*a21 + b3*a31;
  out[6] = b0*a02 + b1*a12 + b2*a22 + b3*a32;
  out[7] = b0*a03 + b1*a13 + b2*a23 + b3*a33;

  b0 = b[8]; b1 = b[9]; b2 = b[10]; b3 = b[11];
  out[8] = b0*a00 + b1*a10 + b2*a20 + b3*a30;
  out[9] = b0*a01 + b1*a11 + b2*a21 + b3*a31;
  out[10] = b0*a02 + b1*a12 + b2*a22 + b3*a32;
  out[11] = b0*a03 + b1*a13 + b2*a23 + b3*a33;

  b0 = b[12]; b1 = b[13]; b2 = b[14]; b3 = b[15];
  out[12] = b0*a00 + b1*a10 + b2*a20 + b3*a30;
  out[13] = b0*a01 + b1*a11 + b2*a21 + b3*a31;
  out[14] = b0*a02 + b1*a12 + b2*a22 + b3*a32;
  out[15] = b0*a03 + b1*a13 + b2*a23 + b3*a33;
}

void matrix_scale(seni_matrix out, seni_matrix a, f32 x, f32 y, f32 z)
{
  out[0] = a[0] * x;
  out[1] = a[1] * x;
  out[2] = a[2] * x;
  out[3] = a[3] * x;
  out[4] = a[4] * y;
  out[5] = a[5] * y;
  out[6] = a[6] * y;
  out[7] = a[7] * y;
  out[8] = a[8] * z;
  out[9] = a[9] * z;
  out[10] = a[10] * z;
  out[11] = a[11] * z;
  out[12] = a[12];
  out[13] = a[13];
  out[14] = a[14];
  out[15] = a[15];  
}

void matrix_translate(seni_matrix out, seni_matrix a, f32 x, f32 y, f32 z)
{
  if (a == out) {
    out[12] = a[0] * x + a[4] * y + a[8] * z + a[12];
    out[13] = a[1] * x + a[5] * y + a[9] * z + a[13];
    out[14] = a[2] * x + a[6] * y + a[10] * z + a[14];
    out[15] = a[3] * x + a[7] * y + a[11] * z + a[15];
  } else {
    f32 a00 = a[0], a01 = a[1], a02 = a[2], a03 = a[3];
    f32 a10 = a[4], a11 = a[5], a12 = a[6], a13 = a[7];
    f32 a20 = a[8], a21 = a[9], a22 = a[10], a23 = a[11];

    out[0] = a00; out[1] = a01; out[2] = a02; out[3] = a03;
    out[4] = a10; out[5] = a11; out[6] = a12; out[7] = a13;
    out[8] = a20; out[9] = a21; out[10] = a22; out[11] = a23;

    out[12] = a00 * x + a10 * y + a20 * z + a[12];
    out[13] = a01 * x + a11 * y + a21 * z + a[13];
    out[14] = a02 * x + a12 * y + a22 * z + a[14];
    out[15] = a03 * x + a13 * y + a23 * z + a[15];
  }
}

void matrix_rotate_z(seni_matrix out, seni_matrix a, f32 rad)
{
  f32 s = sin(rad), c = cos(rad);
  f32 a00 = a[0], a01 = a[1], a02 = a[2], a03 = a[3];
  f32 a10 = a[4], a11 = a[5], a12 = a[6], a13 = a[7];

  if (a != out) {
    out[8] = a[8];
    out[9] = a[9];
    out[10] = a[10];
    out[11] = a[11];
    out[12] = a[12];
    out[13] = a[13];
    out[14] = a[14];
    out[15] = a[15];
  }

  // Perform axis-specific matrix multiplication
  out[0] = a00 * c + a10 * s;
  out[1] = a01 * c + a11 * s;
  out[2] = a02 * c + a12 * s;
  out[3] = a03 * c + a13 * s;
  out[4] = a10 * c - a00 * s;
  out[5] = a11 * c - a01 * s;
  out[6] = a12 * c - a02 * s;
  out[7] = a13 * c - a03 * s;
}

void matrix_transform_vec2(f32 *out, seni_matrix m, f32 x, f32 y)
{
  out[0] = m[0] * x + m[4] * y + m[12];
  out[1] = m[1] * x + m[5] * y + m[13];
}

void matrix_transform_vec3(f32 *out, seni_matrix m, f32 x, f32 y, f32 z)
{
  f32 w = m[3] * x + m[7] * y + m[11] * z + m[15];
  w = w == 0.0 ? 1.0 : w;
  out[0] = (m[0] * x + m[4] * y + m[8] * z + m[12]) / w;
  out[1] = (m[1] * x + m[5] * y + m[9] * z + m[13]) / w;
  out[2] = (m[2] * x + m[6] * y + m[10] * z + m[14]) / w;
}


seni_matrix_stack *matrix_stack_construct()
{
  seni_matrix_stack *matrix_stack = (seni_matrix_stack *)calloc(1, sizeof(seni_matrix_stack));

  matrix_stack->stack_size = MATRIX_STACK_SIZE;
  matrix_stack->stack = (seni_matrix *)calloc(MATRIX_STACK_SIZE, sizeof(seni_matrix));

  // note: seni_matrix is just a typedef to a pointer to f32
  // so each matrix on the stack needs to be explicitly allocated
  for (i32 i = 0; i < MATRIX_STACK_SIZE; i++) {
    matrix_stack->stack[i] = matrix_construct();
  }

  matrix_stack->wip_transform = matrix_construct();

  matrix_stack->sp = 0;
  
  return matrix_stack;
}

void matrix_stack_free(seni_matrix_stack *matrix_stack)
{
  free(matrix_stack->wip_transform);

  for (i32 i = 0; i < MATRIX_STACK_SIZE; i++) {
    matrix_free(matrix_stack->stack[i]);
  }

  free(matrix_stack->stack);
  free(matrix_stack);
}

seni_matrix matrix_stack_push(seni_matrix_stack *matrix_stack)
{
  if (matrix_stack->sp == MATRIX_STACK_SIZE) {
    SENI_ERROR("matrix_stack_push: matrix stack is full");
    return NULL;
  }
  
  seni_matrix m = matrix_stack->stack[matrix_stack->sp++];
  return m;
}

seni_matrix matrix_stack_pop(seni_matrix_stack *matrix_stack)
{
  if (matrix_stack->sp == 0) {
    SENI_ERROR("matrix_stack_pop: matrix stack is empty");
    return NULL;
  }

  matrix_stack->sp--;
  seni_matrix m = matrix_stack->stack[matrix_stack->sp];

  return m;
}

seni_matrix matrix_stack_peek(seni_matrix_stack *matrix_stack)
{
  seni_matrix head = matrix_stack->stack[matrix_stack->sp - 1];

  return head;
}

void matrix_stack_scale(seni_matrix_stack *matrix_stack, f32 sx, f32 sy)
{
  seni_matrix m = matrix_stack->wip_transform;
  matrix_identity(m);
  matrix_scale(m, m, sx, sy, 1.0f);

  seni_matrix head = matrix_stack_peek(matrix_stack);
  matrix_multiply(head, head, m);
}

void matrix_stack_translate(seni_matrix_stack *matrix_stack, f32 tx, f32 ty)
{
  seni_matrix m = matrix_stack->wip_transform;
  matrix_identity(m);
  matrix_translate(m, m, tx, ty, 0.0f);

  seni_matrix head = matrix_stack_peek(matrix_stack);
  matrix_multiply(head, head, m);
}

void matrix_stack_rotate(seni_matrix_stack *matrix_stack, f32 a)
{
  seni_matrix m = matrix_stack->wip_transform;
  matrix_identity(m);
  matrix_rotate_z(m, m, a);

  seni_matrix head = matrix_stack_peek(matrix_stack);
  matrix_multiply(head, head, m);
}

void matrix_stack_transform_vec2(f32 *out, seni_matrix_stack *matrix_stack, f32 x, f32 y)
{
  seni_matrix head = matrix_stack_peek(matrix_stack);
  matrix_transform_vec2(out, head, x, y);
}

void matrix_stack_transform_vec3(f32 *out, seni_matrix_stack *matrix_stack, f32 x, f32 y, f32 z)
{
  seni_matrix head = matrix_stack_peek(matrix_stack);
  matrix_transform_vec3(out, head, x, y, z);
}
