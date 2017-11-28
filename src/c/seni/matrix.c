#include "matrix.h"

#include "config.h"

#include "stdlib.h"
#include <math.h>

void matrix_copy(seni_matrix* out, seni_matrix* a) {
  out->m[0]  = a->m[0];
  out->m[1]  = a->m[1];
  out->m[2]  = a->m[2];
  out->m[3]  = a->m[3];
  out->m[4]  = a->m[4];
  out->m[5]  = a->m[5];
  out->m[6]  = a->m[6];
  out->m[7]  = a->m[7];
  out->m[8]  = a->m[8];
  out->m[9]  = a->m[9];
  out->m[10] = a->m[10];
  out->m[11] = a->m[11];
  out->m[12] = a->m[12];
  out->m[13] = a->m[13];
  out->m[14] = a->m[14];
  out->m[15] = a->m[15];
}

void matrix_identity(seni_matrix* out) {
  out->m[0]  = 1.0f;
  out->m[1]  = 0.0f;
  out->m[2]  = 0.0f;
  out->m[3]  = 0.0f;
  out->m[4]  = 0.0f;
  out->m[5]  = 1.0f;
  out->m[6]  = 0.0f;
  out->m[7]  = 0.0f;
  out->m[8]  = 0.0f;
  out->m[9]  = 0.0f;
  out->m[10] = 1.0f;
  out->m[11] = 0.0f;
  out->m[12] = 0.0f;
  out->m[13] = 0.0f;
  out->m[14] = 0.0f;
  out->m[15] = 1.0f;
}

void matrix_ortho(seni_matrix* out, f32 left, f32 right, f32 bottom, f32 top, f32 near, f32 far) {
  f32 lr = 1.0f / (left - right);
  f32 bt = 1.0f / (bottom - top);
  f32 nf = 1.0f / (near - far);

  out->m[0]  = -2.0f * lr;
  out->m[1]  = 0.0f;
  out->m[2]  = 0.0f;
  out->m[3]  = 0.0f;
  out->m[4]  = 0.0f;
  out->m[5]  = -2.0f * bt;
  out->m[6]  = 0.0f;
  out->m[7]  = 0.0f;
  out->m[8]  = 0.0f;
  out->m[9]  = 0.0f;
  out->m[10] = 2.0f * nf;
  out->m[11] = 0.0f;
  out->m[12] = (left + right) * lr;
  out->m[13] = (top + bottom) * bt;
  out->m[14] = (far + near) * nf;
  out->m[15] = 1.0f;
}

void matrix_multiply(seni_matrix* out, seni_matrix* a, seni_matrix* b) {
  f32 a00 = a->m[0], a01 = a->m[1], a02 = a->m[2], a03 = a->m[3];
  f32 a10 = a->m[4], a11 = a->m[5], a12 = a->m[6], a13 = a->m[7];
  f32 a20 = a->m[8], a21 = a->m[9], a22 = a->m[10], a23 = a->m[11];
  f32 a30 = a->m[12], a31 = a->m[13], a32 = a->m[14], a33 = a->m[15];

  // Cache only the current line of the second matrix
  f32 b0 = b->m[0], b1 = b->m[1], b2 = b->m[2], b3 = b->m[3];
  out->m[0] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
  out->m[1] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
  out->m[2] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
  out->m[3] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;

  b0        = b->m[4];
  b1        = b->m[5];
  b2        = b->m[6];
  b3        = b->m[7];
  out->m[4] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
  out->m[5] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
  out->m[6] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
  out->m[7] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;

  b0         = b->m[8];
  b1         = b->m[9];
  b2         = b->m[10];
  b3         = b->m[11];
  out->m[8]  = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
  out->m[9]  = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
  out->m[10] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
  out->m[11] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;

  b0         = b->m[12];
  b1         = b->m[13];
  b2         = b->m[14];
  b3         = b->m[15];
  out->m[12] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
  out->m[13] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
  out->m[14] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
  out->m[15] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;
}

void matrix_scale(seni_matrix* out, seni_matrix* a, f32 x, f32 y, f32 z) {
  out->m[0]  = a->m[0] * x;
  out->m[1]  = a->m[1] * x;
  out->m[2]  = a->m[2] * x;
  out->m[3]  = a->m[3] * x;
  out->m[4]  = a->m[4] * y;
  out->m[5]  = a->m[5] * y;
  out->m[6]  = a->m[6] * y;
  out->m[7]  = a->m[7] * y;
  out->m[8]  = a->m[8] * z;
  out->m[9]  = a->m[9] * z;
  out->m[10] = a->m[10] * z;
  out->m[11] = a->m[11] * z;
  out->m[12] = a->m[12];
  out->m[13] = a->m[13];
  out->m[14] = a->m[14];
  out->m[15] = a->m[15];
}

void matrix_translate(seni_matrix* out, seni_matrix* a, f32 x, f32 y, f32 z) {
  if (a == out) {
    out->m[12] = a->m[0] * x + a->m[4] * y + a->m[8] * z + a->m[12];
    out->m[13] = a->m[1] * x + a->m[5] * y + a->m[9] * z + a->m[13];
    out->m[14] = a->m[2] * x + a->m[6] * y + a->m[10] * z + a->m[14];
    out->m[15] = a->m[3] * x + a->m[7] * y + a->m[11] * z + a->m[15];
  } else {
    f32 a00 = a->m[0], a01 = a->m[1], a02 = a->m[2], a03 = a->m[3];
    f32 a10 = a->m[4], a11 = a->m[5], a12 = a->m[6], a13 = a->m[7];
    f32 a20 = a->m[8], a21 = a->m[9], a22 = a->m[10], a23 = a->m[11];

    out->m[0]  = a00;
    out->m[1]  = a01;
    out->m[2]  = a02;
    out->m[3]  = a03;
    out->m[4]  = a10;
    out->m[5]  = a11;
    out->m[6]  = a12;
    out->m[7]  = a13;
    out->m[8]  = a20;
    out->m[9]  = a21;
    out->m[10] = a22;
    out->m[11] = a23;

    out->m[12] = a00 * x + a10 * y + a20 * z + a->m[12];
    out->m[13] = a01 * x + a11 * y + a21 * z + a->m[13];
    out->m[14] = a02 * x + a12 * y + a22 * z + a->m[14];
    out->m[15] = a03 * x + a13 * y + a23 * z + a->m[15];
  }
}

void matrix_rotate_z(seni_matrix* out, seni_matrix* a, f32 rad) {
  f32 s = sinf(rad), c = cosf(rad);
  f32 a00 = a->m[0], a01 = a->m[1], a02 = a->m[2], a03 = a->m[3];
  f32 a10 = a->m[4], a11 = a->m[5], a12 = a->m[6], a13 = a->m[7];

  if (a != out) {
    out->m[8]  = a->m[8];
    out->m[9]  = a->m[9];
    out->m[10] = a->m[10];
    out->m[11] = a->m[11];
    out->m[12] = a->m[12];
    out->m[13] = a->m[13];
    out->m[14] = a->m[14];
    out->m[15] = a->m[15];
  }

  // Perform axis-specific matrix multiplication
  out->m[0] = a00 * c + a10 * s;
  out->m[1] = a01 * c + a11 * s;
  out->m[2] = a02 * c + a12 * s;
  out->m[3] = a03 * c + a13 * s;
  out->m[4] = a10 * c - a00 * s;
  out->m[5] = a11 * c - a01 * s;
  out->m[6] = a12 * c - a02 * s;
  out->m[7] = a13 * c - a03 * s;
}

void matrix_transform_vec2(f32* outx, f32* outy, seni_matrix* m, f32 x, f32 y) {
  *outx = m->m[0] * x + m->m[4] * y + m->m[12];
  *outy = m->m[1] * x + m->m[5] * y + m->m[13];
}

void matrix_transform_vec3(f32* outx, f32* outy, f32* outz, seni_matrix* m, f32 x, f32 y, f32 z) {
  f32 w = m->m[3] * x + m->m[7] * y + m->m[11] * z + m->m[15];
  w     = w == 0.0f ? 1.0f : w;
  *outx = (m->m[0] * x + m->m[4] * y + m->m[8] * z + m->m[12]) / w;
  *outy = (m->m[1] * x + m->m[5] * y + m->m[9] * z + m->m[13]) / w;
  *outz = (m->m[2] * x + m->m[6] * y + m->m[10] * z + m->m[14]) / w;
}

seni_matrix_stack* matrix_stack_allocate() {
  seni_matrix_stack* matrix_stack = (seni_matrix_stack*)calloc(1, sizeof(seni_matrix_stack));

  matrix_stack->stack_size = MATRIX_STACK_SIZE;
  matrix_stack->stack      = (seni_matrix*)calloc(MATRIX_STACK_SIZE, sizeof(seni_matrix));

  matrix_stack->wip_transform = (seni_matrix*)calloc(1, sizeof(seni_matrix));

  matrix_stack->sp = 0;

  return matrix_stack;
}

void matrix_stack_free(seni_matrix_stack* matrix_stack) {
  free(matrix_stack->wip_transform);
  free(matrix_stack->stack);
  free(matrix_stack);
}

void matrix_stack_reset(seni_matrix_stack* matrix_stack) {
  // add an identity matrix onto the stack so that further
  // scale/rotate/translate ops can work
  seni_matrix* m = &(matrix_stack->stack[0]);
  matrix_identity(m);

  matrix_stack->sp = 1;
}

seni_matrix* matrix_stack_push(seni_matrix_stack* matrix_stack) {
  if (matrix_stack->sp == MATRIX_STACK_SIZE) {
    SENI_ERROR("matrix_stack_push: matrix stack is full");
    return NULL;
  }

  seni_matrix* old_top = &(matrix_stack->stack[matrix_stack->sp - 1]);
  seni_matrix* new_top = &(matrix_stack->stack[matrix_stack->sp]);

  matrix_stack->sp++;

  matrix_copy(new_top, old_top);

  return new_top;
}

seni_matrix* matrix_stack_pop(seni_matrix_stack* matrix_stack) {
  if (matrix_stack->sp == 0) {
    SENI_ERROR("matrix_stack_pop: matrix stack is empty");
    return NULL;
  }

  matrix_stack->sp--;
  seni_matrix* m = &(matrix_stack->stack[matrix_stack->sp]);

  return m;
}

seni_matrix* matrix_stack_peek(seni_matrix_stack* matrix_stack) {
  seni_matrix* head = &(matrix_stack->stack[matrix_stack->sp - 1]);

  return head;
}

void matrix_stack_scale(seni_matrix_stack* matrix_stack, f32 sx, f32 sy) {
  seni_matrix* m = matrix_stack->wip_transform;
  matrix_identity(m);
  matrix_scale(m, m, sx, sy, 1.0f);

  seni_matrix* head = matrix_stack_peek(matrix_stack);
  matrix_multiply(head, head, m);
}

void matrix_stack_translate(seni_matrix_stack* matrix_stack, f32 tx, f32 ty) {
  seni_matrix* m = matrix_stack->wip_transform;
  matrix_identity(m);
  matrix_translate(m, m, tx, ty, 0.0f);

  seni_matrix* head = matrix_stack_peek(matrix_stack);
  matrix_multiply(head, head, m);
}

void matrix_stack_rotate(seni_matrix_stack* matrix_stack, f32 a) {
  seni_matrix* m = matrix_stack->wip_transform;
  matrix_identity(m);
  matrix_rotate_z(m, m, a);

  seni_matrix* head = matrix_stack_peek(matrix_stack);
  matrix_multiply(head, head, m);
}

void matrix_stack_transform_vec2(f32*               outx,
                                 f32*               outy,
                                 seni_matrix_stack* matrix_stack,
                                 f32                x,
                                 f32                y) {
  seni_matrix* head = matrix_stack_peek(matrix_stack);
  matrix_transform_vec2(outx, outy, head, x, y);
}

void matrix_stack_transform_vec3(f32*               outx,
                                 f32*               outy,
                                 f32*               outz,
                                 seni_matrix_stack* matrix_stack,
                                 f32                x,
                                 f32                y,
                                 f32                z) {
  seni_matrix* head = matrix_stack_peek(matrix_stack);
  matrix_transform_vec3(outx, outy, outz, head, x, y, z);
}
