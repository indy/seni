#include "seni_bind.h"

#include "seni_shapes.h"
#include "seni_buffer.h"
#include "seni_lang.h"
#include "seni_matrix.h"
#include "seni_mathutil.h"
#include "seni_prng.h"

#include <stdio.h>
#include <string.h>
#include <stdlib.h>

// struct used by binding functions for prng/take and prng/take-1
typedef struct {
  u64 state;
  u64 inc;
  f32 min;
  f32 max;

  // references to the heap allocated seni_vars on the vector need to be updated after seni_prng_f32 is called
  //
  seni_var *seni_var_state;
  seni_var *seni_var_inc;
} seni_prng_full_state;

// helper macros used by the bind code to parse arguments on the VM's stack
//
#define READ_STACK_ARGS_BEGIN i32 args_pointer_1 = vm->sp - (num_args * 2); \
  i32 i_1;                                                              \
  seni_var *label_1, *value_1, *tmp_1, *tmp_2;                          \
  tmp_1 = NULL;                                                         \
  tmp_2 = NULL;                                                         \
  for(i_1 = 0; i_1 < num_args; i_1++) {                                 \
  label_1 = &(vm->stack[args_pointer_1 + 0]);                           \
  value_1 = &(vm->stack[args_pointer_1 + 1]);                           \
  args_pointer_1 += 2;                                                  \
  i32 name_1 = label_1->value.i

#define READ_STACK_ARGS_END };
// #define READ_STACK_ARGS_END } vm->sp -= (num_args * 2);

#define READ_STACK_ARG_F32(n) if (name_1 == g_keyword_iname_##n) { n = value_1->value.f; }
#define READ_STACK_ARG_I32(n) if (name_1 == g_keyword_iname_##n) { n = value_1->value.i; }
#define READ_STACK_ARG_COL(n) if (name_1 == g_keyword_iname_##n) { n = value_1->value.c; }
#define READ_STACK_ARG_VAR(n) if (name_1 == g_keyword_iname_##n) { n = value_1; }

// traverse through the VAR_VEC_HEAD, VAR_VEC_RC nodes down into the values
// todo: make a seni_var type that can hold VEC2
#define READ_STACK_ARG_VEC2(n) if (name_1 == g_keyword_iname_##n) {    \
    tmp_1 = (value_1->value.v)->next;                                  \
    n[0] = tmp_1->value.f;                                             \
    n[1] = tmp_1->next->value.f;                                       \
  }

// this is probably obsolete now. It was used to parse colours
// when they were just represented as a vector of 4 floats
//
#define READ_STACK_ARG_VEC4(n) if (name_1 == g_keyword_iname_##n) {    \
    tmp_1 = (value_1->value.v)->next;                                  \
    n[0] = tmp_1->value.f;                                             \
    tmp_1 = tmp_1->next;                                               \
    n[1] = tmp_1->value.f;                                             \
    tmp_1 = tmp_1->next;                                               \
    n[2] = tmp_1->value.f;                                             \
    tmp_1 = tmp_1->next;                                               \
    n[3] = tmp_1->value.f;                                             \
  }

#define READ_STACK_ARG_PRNG(n) if (name_1 == g_keyword_iname_##n) {     \
    tmp_1 = (value_1->value.v)->next;                                   \
    n.state = tmp_1->value.l;                                           \
    n.seni_var_state = tmp_1;                                           \
    tmp_1 = tmp_1->next;                                                \
    n.inc = tmp_1->value.l;                                             \
    n.seni_var_inc = tmp_1;                                             \
    tmp_1 = tmp_1->next;                                                \
    n.min = tmp_1->value.f;                                             \
    tmp_1 = tmp_1->next;                                                \
    n.max = tmp_1->value.f;                                             \
  }

#define READ_STACK_ARG_COORD4(n) if (name_1 == g_keyword_iname_##n) { \
    tmp_1 = (value_1->value.v)->next;                                 \
    tmp_2 = (tmp_1->value.v)->next;                                   \
    n[0] = tmp_2->value.f;                                            \
    tmp_2 = tmp_2->next;                                              \
    n[1] = tmp_2->value.f;                                            \
    tmp_1 = tmp_1->next;                                              \
    tmp_2 = (tmp_1->value.v)->next;                                   \
    n[2] = tmp_2->value.f;                                            \
    tmp_2 = tmp_2->next;                                              \
    n[3] = tmp_2->value.f;                                            \
    tmp_1 = tmp_1->next;                                              \
    tmp_2 = (tmp_1->value.v)->next;                                   \
    n[4] = tmp_2->value.f;                                            \
    tmp_2 = tmp_2->next;                                              \
    n[5] = tmp_2->value.f;                                            \
    tmp_1 = tmp_1->next;                                              \
    tmp_2 = (tmp_1->value.v)->next;                                   \
    n[6] = tmp_2->value.f;                                            \
    tmp_2 = tmp_2->next;                                              \
    n[7] = tmp_2->value.f;                                            \
  }

// extern global keyword variables
#define KEYWORD(val,_,name) extern i32 g_keyword_iname_##name;
#include "seni_keywords.h"
#undef KEYWORD

// a global var that represents true, used as the default
// return type for bindings that only have side-effects
//
seni_var g_var_true;

void string_copy(char **dst, char *src)
{
  size_t len = strlen(src);
  
  char *c = (char *)malloc(sizeof(char) * (len + 1));
  strncpy(c, src, len);
  c[len] = '\0';

  *dst = c;
}

void declare_vm_keyword(seni_word_lut *wlut, char *name)
{
  string_copy(&(wlut->keyword[wlut->keyword_count]), name);
  wlut->keyword_count++;

  if (wlut->keyword_count > MAX_KEYWORD_LOOKUPS) {
    SENI_ERROR("cannot declare keyword - wlut is full");
  }
}

void declare_binding(seni_word_lut *wlut, seni_env *e, char *name, native_function_ptr function_ptr)
{
  string_copy(&(wlut->native[wlut->native_count]), name);

  e->function_ptr[wlut->native_count] = function_ptr;

  wlut->native_count++;

  if (wlut->native_count > MAX_NATIVE_LOOKUPS) {
    SENI_ERROR("cannot declare native - wlut is full");
  }
}

seni_var bind_line(seni_vm *vm, i32 num_args)
{
  // default values for line
  f32 width = 4.0f;
  f32 from[] = {10.0f, 10.0f};
  f32 to[] = {900.0f, 500.0f};
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(width);
  READ_STACK_ARG_VEC2(from);
  READ_STACK_ARG_VEC2(to);
  READ_STACK_ARG_COL(colour);
  READ_STACK_ARGS_END;

  seni_buffer *buffer = vm->buffer;
  seni_matrix *matrix = matrix_stack_peek(vm->matrix_stack);

  render_line(buffer, matrix, from[0], from[1], to[0], to[1], width, colour);


  return g_var_true;
}

seni_var bind_rect(seni_vm *vm, i32 num_args)
{
  // default values for rect
  f32 width = 4.0f;
  f32 height = 10.0f;
  f32 position[] = {10.0f, 23.0f};
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(width);
  READ_STACK_ARG_F32(height);
  READ_STACK_ARG_VEC2(position);
  READ_STACK_ARG_COL(colour);
  READ_STACK_ARGS_END;

  seni_buffer *buffer = vm->buffer;
  seni_matrix *matrix = matrix_stack_peek(vm->matrix_stack);

  render_rect(buffer, matrix, position[0], position[1], width, height, colour);


  return g_var_true;
}

seni_var bind_circle(seni_vm *vm, i32 num_args)
{
  // default values for circle
  f32 width = 4.0f;
  f32 height = 10.0f;
  f32 position[] = {10.0f, 23.0f};
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;
  f32 tessellation = 10.0f;
  f32 radius = -1.0f;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(width);
  READ_STACK_ARG_F32(height);
  READ_STACK_ARG_F32(radius);
  READ_STACK_ARG_VEC2(position);
  READ_STACK_ARG_COL(colour);
  READ_STACK_ARG_F32(tessellation);
  READ_STACK_ARGS_END;

  // if the radius has been defined then it overrides the width and height parameters
  if (radius > 0.0f) {
    width = radius;
    height = radius;
  }

  seni_buffer *buffer = vm->buffer;
  seni_matrix *matrix = matrix_stack_peek(vm->matrix_stack);

  render_circle(buffer, matrix, position[0], position[1], width, height, colour, (i32)tessellation);


  return g_var_true;
}

seni_var bind_bezier(seni_vm *vm, i32 num_args)
{
  // default values for bezier
  f32 line_width = -1.0f;
  f32 line_width_start = 4.0f;
  f32 line_width_end = 4.0f;
  i32 line_width_mapping = g_keyword_iname_linear;
  f32 coords[] = { 100.0f, 500.0f, 300.0f, 300.0f, 600.0f, 700.0f, 900.0f, 500.0f };
  f32 t_start = -1.0f;
  f32 t_end = 2.0f;
  f32 tessellation = 10.0f;
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;
  i32 brush = g_keyword_iname_brush_flat;
  f32 brush_subtype = 0.0f;
    

  // line_width_mapping will be one of several constants
  
  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(line_width);
  READ_STACK_ARG_F32(line_width_start);
  READ_STACK_ARG_F32(line_width_end);
  READ_STACK_ARG_I32(line_width_mapping);
  READ_STACK_ARG_COORD4(coords);
  READ_STACK_ARG_F32(t_start);
  READ_STACK_ARG_F32(t_end);
  READ_STACK_ARG_COL(colour);
  READ_STACK_ARG_F32(tessellation);
  READ_STACK_ARG_I32(brush);
  READ_STACK_ARG_F32(brush_subtype);
  READ_STACK_ARGS_END;

  // if the line_width has been defined then it overrides the separate start/end variables
  if (line_width > 0.0f) {
    line_width_start = line_width;
    line_width_end = line_width;
  }

  if (t_start < 0.0f) {
    t_start = 0.0f;
  }
  
  if (t_end > 1.0f) {
    t_end = 1.0f;
  }

  seni_buffer *buffer = vm->buffer;
  seni_matrix *matrix = matrix_stack_peek(vm->matrix_stack);

  render_bezier(buffer, matrix,
                coords, line_width_start, line_width_end, line_width_mapping,
                t_start, t_end, colour, (i32)tessellation, brush, (i32)brush_subtype);


  return g_var_true;
}

seni_var bind_col_convert(seni_vm *vm, i32 num_args)
{
  // (col/convert colour: col format: LAB)
  
  i32 format = RGB;
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_I32(format);
  READ_STACK_ARG_COL(colour);
  READ_STACK_ARGS_END;

  // the seni_var referencing the converted colour is going to be added to the VM's stack
  // so we need to get the referenced colour from the vm
  //
  seni_colour *out = colour_get_from_vm(vm);
  colour_clone_as(out, colour, format);

  seni_var ret;
  colour_as_var(&ret, out);


  return ret;
}

seni_var bind_col_rgb(seni_vm *vm, i32 num_args)
{
  // (col/rgb r: 0.627 g: 0.627 b: 0.627 alpha: 0.4)
  
  // default values for line
  f32 r = 0.0f;                 // 0..1
  f32 g = 0.0f;                 // 0..1
  f32 b = 0.0f;                 // 0..1
  f32 alpha = 0.0f;             // 0..1

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(r);
  READ_STACK_ARG_F32(g);
  READ_STACK_ARG_F32(b);
  READ_STACK_ARG_F32(alpha);
  READ_STACK_ARGS_END;

  seni_var ret;

  seni_colour *colour = colour_get_from_vm(vm);
  colour->format = RGB;
  colour->element[0] = r;
  colour->element[1] = g;
  colour->element[2] = b;
  colour->element[3] = alpha;

  colour_as_var(&ret, colour);


  return ret;
}

seni_var bind_col_hsl(seni_vm *vm, i32 num_args)
{
  // (col/hsl h: 180.0 s: 0.1 l: 0.2 alpha: 0.4)
  
  // default values for line
  f32 h = 0.0f;                 // 0..360
  f32 s = 0.0f;                 // 0..1
  f32 l = 0.0f;                 // 0..1
  f32 alpha = 0.0f;             // 0..1

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(h);
  READ_STACK_ARG_F32(s);
  READ_STACK_ARG_F32(l);
  READ_STACK_ARG_F32(alpha);
  READ_STACK_ARGS_END;

  seni_var ret;

  seni_colour *colour = colour_get_from_vm(vm);
  colour->format = HSL;
  colour->element[0] = h;
  colour->element[1] = s;
  colour->element[2] = l;
  colour->element[3] = alpha;

  colour_as_var(&ret, colour);


  return ret;
}

seni_var bind_col_hsv(seni_vm *vm, i32 num_args)
{
  // (col/hsv h: 180.0 s: 0.1 v: 0.2 alpha: 0.4)
  
  // default values for line
  f32 h = 0.0f;                 // 0..360
  f32 s = 0.0f;                 // 0..1
  f32 v = 0.0f;                 // 0..1
  f32 alpha = 0.0f;             // 0..1

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(h);
  READ_STACK_ARG_F32(s);
  READ_STACK_ARG_F32(v);
  READ_STACK_ARG_F32(alpha);
  READ_STACK_ARGS_END;

  seni_var ret;

  seni_colour *colour = colour_get_from_vm(vm);
  colour->format = HSV;
  colour->element[0] = h;
  colour->element[1] = s;
  colour->element[2] = v;
  colour->element[3] = alpha;

  colour_as_var(&ret, colour);


  return ret;
}

seni_var bind_col_lab(seni_vm *vm, i32 num_args)
{
  // (col/lab l: 0.2 a: -0.1 b: -0.3 alpha: 0.4)
  
  // default values for line
  f32 l = 0.0f;                 // 0..
  f32 a = 0.0f;                 // -1..1
  f32 b = 0.0f;                 // -1..1
  f32 alpha = 0.0f;             // 0..1

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(l);
  READ_STACK_ARG_F32(a);
  READ_STACK_ARG_F32(b);
  READ_STACK_ARG_F32(alpha);
  READ_STACK_ARGS_END;

  seni_var ret;

  seni_colour *colour = colour_get_from_vm(vm);
  colour->format = LAB;
  colour->element[0] = l;
  colour->element[1] = a;
  colour->element[2] = b;
  colour->element[3] = alpha;

  colour_as_var(&ret, colour);


  return ret;
}

seni_var bind_col_complementary(seni_vm *vm, i32 num_args)
{
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(colour);
  READ_STACK_ARGS_END;

  seni_colour *ret_colour = colour_get_from_vm(vm);
  complementary(ret_colour, colour);


  seni_var ret;
  colour_as_var(&ret, ret_colour);
  return ret;
}

seni_var bind_col_split_complementary(seni_vm *vm, i32 num_args)
{
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(colour);
  READ_STACK_ARGS_END;

  seni_colour *ret_colour0 = colour_get_from_vm(vm);
  seni_colour *ret_colour1 = colour_get_from_vm(vm);
  split_complementary(ret_colour0, ret_colour1, colour);

  // push the return values onto the stack as a vector
  seni_var ret;
  vector_construct(vm, &ret);
  append_to_vector_col(vm, &ret, ret_colour0);
  append_to_vector_col(vm, &ret, ret_colour1);
  return ret;
}

seni_var bind_col_analagous(seni_vm *vm, i32 num_args)
{
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(colour);
  READ_STACK_ARGS_END;

  seni_colour *ret_colour0 = colour_get_from_vm(vm);
  seni_colour *ret_colour1 = colour_get_from_vm(vm);
  analagous(ret_colour0, ret_colour1, colour);

  // push the return values onto the stack as a vector
  seni_var ret;
  vector_construct(vm, &ret);
  append_to_vector_col(vm, &ret, ret_colour0);
  append_to_vector_col(vm, &ret, ret_colour1);
  return ret;
}

seni_var bind_col_triad(seni_vm *vm, i32 num_args)
{
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(colour);
  READ_STACK_ARGS_END;

  seni_colour *ret_colour0 = colour_get_from_vm(vm);
  seni_colour *ret_colour1 = colour_get_from_vm(vm);
  triad(ret_colour0, ret_colour1, colour);

  // push the return values onto the stack as a vector
  seni_var ret;
  vector_construct(vm, &ret);
  append_to_vector_col(vm, &ret, ret_colour0);
  append_to_vector_col(vm, &ret, ret_colour1);
  return ret;
}

seni_var bind_col_darken(seni_vm *vm, i32 num_args)
{
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;
  f32 value = 0;                // 0..100

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(colour);
  READ_STACK_ARG_F32(value);
  READ_STACK_ARGS_END;

  seni_colour *ret_colour = colour_get_from_vm(vm);

  colour_clone_as(ret_colour, colour, LAB);
  ret_colour->element[0] = clamp(ret_colour->element[0] - value, 0.0f, 100.0f);


  seni_var ret;
  colour_as_var(&ret, ret_colour);
  return ret;
}

seni_var bind_col_lighten(seni_vm *vm, i32 num_args)
{
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;
  f32 value = 0;                // 0..100

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(colour);
  READ_STACK_ARG_F32(value);
  READ_STACK_ARGS_END;

  seni_colour *ret_colour = colour_get_from_vm(vm);

  colour_clone_as(ret_colour, colour, LAB);
  ret_colour->element[0] = clamp(ret_colour->element[0] + value, 0.0f, 100.0f);


  seni_var ret;
  colour_as_var(&ret, ret_colour);
  return ret;
}

seni_var bind_col_set_alpha(seni_vm *vm, i32 num_args)
{
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;
  f32 value = 0;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(colour);
  READ_STACK_ARG_F32(value);
  READ_STACK_ARGS_END;

  seni_colour *ret_colour = colour_get_from_vm(vm);

  colour_clone_as(ret_colour, colour, colour->format);
  ret_colour->element[3] = value;


  seni_var ret;
  colour_as_var(&ret, ret_colour);
  return ret;
}

seni_var bind_col_get_alpha(seni_vm *vm, i32 num_args)
{
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(colour);
  READ_STACK_ARGS_END;

  seni_var ret;
  f32_as_var(&ret, colour->element[3]);


  return ret;
}

seni_var bind_translate(seni_vm *vm, i32 num_args)
{
  f32 vector[] = {0.0f, 0.0f};

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VEC2(vector);
  READ_STACK_ARGS_END;

  matrix_stack_translate(vm->matrix_stack, vector[0], vector[1]);

  return g_var_true;
}

seni_var bind_rotate(seni_vm *vm, i32 num_args)
{
  f32 angle = 0.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(angle);
  READ_STACK_ARGS_END;

  matrix_stack_rotate(vm->matrix_stack, angle);

  return g_var_true;
}

seni_var bind_scale(seni_vm *vm, i32 num_args)
{
  f32 vector[] = {1.0f, 1.0f};
  f32 scalar = 1.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VEC2(vector);
  READ_STACK_ARG_F32(scalar);
  READ_STACK_ARGS_END;

  if (scalar != 1.0f) {
    matrix_stack_scale(vm->matrix_stack, scalar, scalar);
  } else {
    matrix_stack_scale(vm->matrix_stack, vector[0], vector[1]);
  }

  return g_var_true;
}


seni_var bind_math_distance(seni_vm *vm, i32 num_args)
{
  f32 vec1[] = {0.0f, 0.0f};
  f32 vec2[] = {0.0f, 0.0f};
  
  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VEC2(vec1);
  READ_STACK_ARG_VEC2(vec2);
  READ_STACK_ARGS_END;

  v2 a,b;
  a.x = vec1[0]; a.y = vec1[1];
  b.x = vec2[0]; b.y = vec2[1];

  f32 distance = distance_v2(a, b);
  
  seni_var ret;
  f32_as_var(&ret, distance);


  return ret;
}

seni_var bind_math_clamp(seni_vm *vm, i32 num_args)
{
  // todo: try and move functions like this into ones that initially
  // create and return a function that takes a single argument.
  // e.g.
  // (define my-clamp (math/clamp-fn min: 0.0 max: 42.0))
  // (my-clamp val: 22)
  //
  // then optimize for single argument functions as these will be much faster to parse
  //
  f32 val = 0.0f;
  f32 min = 0.0f;
  f32 max = 1.0f;
  
  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(val);
  READ_STACK_ARG_F32(min);
  READ_STACK_ARG_F32(max);
  READ_STACK_ARGS_END;

  seni_var ret;
  f32_as_var(&ret, clamp(val, min, max));
  

  return ret;
}

seni_var bind_math_radians_to_degrees(seni_vm *vm, i32 num_args)
{
  f32 angle = 0.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(angle);
  READ_STACK_ARGS_END;

  seni_var ret;
  f32_as_var(&ret, rad_to_deg(angle));

  return ret;
}

// (prng/build seed: 4324 min: 40 max: 100)
seni_var bind_prng_build(seni_vm *vm, i32 num_args)
{
  f32 seed = 12322.0f;            // todo: in docs mention that seed should be in range 1..some-large-number
  f32 min = 0.0f;
  f32 max = 1.0f;
  
  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(seed);
  READ_STACK_ARG_F32(min);
  READ_STACK_ARG_F32(max);
  READ_STACK_ARGS_END;

  u64 seed_u64 = (u64)seed;

  // build a seni_prng_state and call it once - this always returns 0 but further calls will be valid
  seni_prng_state prng_state;
  prng_state.state = seed_u64;
  prng_state.inc = 1L;
  seni_prng_f32(&prng_state);
  
  // push the return values onto the stack as a vector
  // the vector needs to represent a seni_prng_state struct as well as the min + max values
  // i.e. [u64 state, u64 inc, f32 min, f32 max]
  //
  seni_var ret;
  vector_construct(vm, &ret);
  append_to_vector_u64(vm, &ret, prng_state.state);
  append_to_vector_u64(vm, &ret, prng_state.inc);
  append_to_vector_f32(vm, &ret, min);
  append_to_vector_f32(vm, &ret, max);

  return ret;
}

// (prng/take num: 5 from: rng)
seni_var bind_prng_take(seni_vm *vm, i32 num_args)
{
  f32 num = 1.0f;
  seni_prng_full_state from;
  // just have anything as the default values, this function should always be given a 'from' parameter
  from.state = 2222;
  from.inc = 1;
  from.min = 0.0f;
  from.max = 1.0f;
  from.seni_var_state = NULL;
  from.seni_var_inc = NULL;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(num);
  READ_STACK_ARG_PRNG(from);
  READ_STACK_ARGS_END;

  // build a seni_prng_state from the seni_prng_full_state
  seni_prng_state prng_state;
  prng_state.state = from.state;
  prng_state.inc = from.inc;

  // create the return vector
  seni_var ret;
  f32 value;

  i32 inum = (i32)num;

  vector_construct(vm, &ret);
  for (i32 i = 0; i < inum; i++) {
    value = seni_prng_f32_range(&prng_state, from.min, from.max);
    append_to_vector_f32(vm, &ret, value);
  }

  // update the state and inc values stored in the vector on the vm's stack
  if (from.seni_var_state != NULL && from.seni_var_inc != NULL) {
    from.seni_var_state->value.l = prng_state.state;
    from.seni_var_inc->value.l = prng_state.inc;
  } else {
    SENI_ERROR("seni_prng_full_state has null pointers ???");
  }

  return ret;
}

// (prng/take-1 from: rng)
seni_var bind_prng_take_1(seni_vm *vm, i32 num_args)
{
  seni_prng_full_state from;
  // just have anything as the default values, this function should always be given a 'from' parameter
  from.state = 2222;
  from.inc = 1;
  from.min = 0.0f;
  from.max = 1.0f;
  from.seni_var_state = NULL;
  from.seni_var_inc = NULL;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_PRNG(from);
  READ_STACK_ARGS_END;

  // build a seni_prng_state from the seni_prng_full_state
  seni_prng_state prng_state;
  prng_state.state = from.state;
  prng_state.inc = from.inc;


  seni_var ret;
  f32 value = seni_prng_f32_range(&prng_state, from.min, from.max);
  f32_as_var(&ret, value);

  // update the state and inc values stored in the vector on the vm's stack
  if (from.seni_var_state != NULL && from.seni_var_inc != NULL) {
    from.seni_var_state->value.l = prng_state.state;
    from.seni_var_inc->value.l = prng_state.inc;
  } else {
    SENI_ERROR("seni_prng_full_state has null pointers ???");
  }

  return ret;
}

seni_var bind_prng_perlin(seni_vm *vm, i32 num_args)
{
  f32 x = 1.0f;
  f32 y = 1.0f;
  f32 z = 1.0f;
  
  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(x);
  READ_STACK_ARG_F32(y);
  READ_STACK_ARG_F32(z);
  READ_STACK_ARGS_END;

  seni_var ret;
  f32 value = seni_perlin(x, y, z);
  f32_as_var(&ret, value);

  return ret;
}

seni_var  bind_debug_print(seni_vm *vm, i32 num_args)
{
  seni_var *val = NULL;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VAR(val);
  READ_STACK_ARGS_END;

  
  pretty_print_seni_var(val, "debug");

  return g_var_true;
}

seni_var bind_nth(seni_vm *vm, i32 num_args)
{
  seni_var *from = NULL;
  f32 n = 0;
  
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VAR(from);
  READ_STACK_ARG_F32(n);
  READ_STACK_ARGS_END;

  if (from->type != VAR_VEC_HEAD) {
    printf("wtf\n");
    return g_var_true;
  }

  i32 nth = (i32)n;
  seni_var *e = from->value.v;

  // e is pointing to the rc, so even a nth of 0 requires one call to e->next
  for (i32 i = 0; i <= nth; i++) {
    e = e->next;
  }

  seni_var ret;
  safe_var_copy(vm, &ret, e);

  return ret;
}


void declare_bindings(seni_word_lut *wlut, seni_env *e)
{
  g_var_true.type = VAR_BOOLEAN;
  g_var_true.value.i = 1;
  
  wlut->keyword_count = 0;

  // this fills out wlut->keyword and that's used in the wlut_lookup_ functions
  //
#define KEYWORD(_,string,__) declare_vm_keyword(wlut, string);
#include "seni_keywords.h"
#undef KEYWORD

  declare_binding(wlut, e, "line", &bind_line);
  declare_binding(wlut, e, "rect", &bind_rect);
  declare_binding(wlut, e, "circle", &bind_circle);
  declare_binding(wlut, e, "bezier", &bind_bezier);

  declare_binding(wlut, e, "translate", &bind_translate);
  declare_binding(wlut, e, "rotate", &bind_rotate);
  declare_binding(wlut, e, "scale", &bind_scale);

  declare_binding(wlut, e, "col/convert", &bind_col_convert);
  declare_binding(wlut, e, "col/rgb", &bind_col_rgb);
  declare_binding(wlut, e, "col/hsl", &bind_col_hsl);
  declare_binding(wlut, e, "col/hsv", &bind_col_hsv);
  declare_binding(wlut, e, "col/lab", &bind_col_lab);
  declare_binding(wlut, e, "col/complementary", &bind_col_complementary);
  declare_binding(wlut, e, "col/split-complementary", &bind_col_split_complementary);
  declare_binding(wlut, e, "col/analagous", &bind_col_analagous);
  declare_binding(wlut, e, "col/triad", &bind_col_triad);
  declare_binding(wlut, e, "col/darken", &bind_col_darken);
  declare_binding(wlut, e, "col/lighten", &bind_col_lighten);
  declare_binding(wlut, e, "col/set-alpha", &bind_col_set_alpha);
  declare_binding(wlut, e, "col/get-alpha", &bind_col_get_alpha);

  // col/procedural-fn-presets
  // col/procedural-fn
  // col/bezier-fn
  // col/quadratic-fn

  declare_binding(wlut, e, "math/distance", &bind_math_distance);
  declare_binding(wlut, e, "math/clamp", &bind_math_clamp);
  declare_binding(wlut, e, "math/radians->degrees", &bind_math_radians_to_degrees);

  declare_binding(wlut, e, "prng/build", &bind_prng_build);
  declare_binding(wlut, e, "prng/take", &bind_prng_take);
  declare_binding(wlut, e, "prng/take-1", &bind_prng_take_1);
  declare_binding(wlut, e, "prng/perlin", &bind_prng_perlin); // was prng/perlin-signed

  declare_binding(wlut, e, "debug/print", &bind_debug_print);

  declare_binding(wlut, e, "nth", &bind_nth);
}

