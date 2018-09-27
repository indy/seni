#include "bind.h"

#include "colour.h"
#include "colour_scheme.h"
#include "config.h"
#include "ease.h"
#include "focal.h"
#include "keyword_iname.h"
#include "lang.h"
#include "mathutil.h"
#include "matrix.h"
#include "parametric.h"
#include "path.h"
#include "prng.h"
#include "render_packet.h"
#include "repeat.h"
#include "shapes.h"
#include "vm_interpreter.h"

#include <math.h>
#include <stdlib.h>
#include <string.h>

i32 g_colour_constructor_start = 0;
i32 g_colour_constructor_end   = 0;

i32 get_colour_constructor_start() { return g_colour_constructor_start; }

i32 get_colour_constructor_end() { return g_colour_constructor_end; }

typedef enum {
  HEAP_STRUCTURE_UNDEFINED = 0,
  HEAP_STRUCTURE_PRNG,
  HEAP_STRUCTURE_PARAMETRIC,
  HEAP_STRUCTURE_FOCAL
} sen_heap_structure_type;

// struct used by binding functions for prng/take and prng/take-1
//
// [ VAR_VECTOR ->
//  (VAR_INT(structure_id) + min, max) ->
//  (VAR_LONG(state0)) ->
//  (VAR_LONE(state1))
// ]
//
typedef struct {
  sen_heap_structure_type structure_id;
  u64                     state0;
  u64                     state1;
  f32                     min;
  f32                     max;

  // references to the heap allocated sen_vars on the vector need to be updated
  // after sen_prng_f32 is called
  //
  sen_var* sen_var_state0;
  sen_var* sen_var_state1;
} sen_prng_full_state;

// [ VAR_VECTOR ->
//  (VAR_INT(structure_id) + from_m, to_m, from_c, to_c) ->
//  (VAR_INT(clamping) + to0, to1) ->
//  (VAR_INT(mapping))
// ]
//
typedef struct {
  sen_heap_structure_type structure_id;
  f32                     from_m;
  f32                     to_m;
  f32                     from_c;
  f32                     to_c;
  f32                     to0;
  f32                     to1;
  i32                     clamping;
  i32                     mapping;
} sen_parametric_state;

// [ VAR_VECTOR ->
//  (VAR_INT(structure_id) + x, y, distance) ->
//  (VAR_INT(type)) ->
//  (VAR_INT(mapping))
// ]
//
typedef struct {
  sen_heap_structure_type structure_id;
  sen_focal_type          type;
  f32                     x;
  f32                     y;
  f32                     distance;
  i32                     mapping;
} sen_focal_state;

// helper macros used by the bind code to parse arguments on the VM's stack
//
#define READ_STACK_ARGS_BEGIN                        \
  i32      args_pointer_1 = vm->sp - (num_args * 2); \
  i32      i_1;                                      \
  sen_var *label_1, *value_1, *tmp_1, *tmp_2;        \
  tmp_1 = NULL;                                      \
  tmp_2 = NULL;                                      \
  for (i_1 = 0; i_1 < num_args; i_1++) {             \
    label_1 = &(vm->stack[args_pointer_1 + 0]);      \
    value_1 = &(vm->stack[args_pointer_1 + 1]);      \
    args_pointer_1 += 2;                             \
    i32 name_1 = label_1->value.i

#define READ_STACK_ARGS_END \
  }                         \
  ;

#ifdef CHECK_STACK_ARGS
#define IS_F32(n)                            \
  if (value_1->type != VAR_FLOAT) {          \
    SEN_ERROR("expected f32 for: %s", #n);   \
    var_pretty_print("received: ", value_1); \
  }
#define IS_I32(n)                            \
  if (value_1->type != VAR_INT) {            \
    SEN_ERROR("expected i32 for: %s", #n);   \
    var_pretty_print("received: ", value_1); \
  }
#define IS_NAME(n)                           \
  if (value_1->type != VAR_NAME) {           \
    SEN_ERROR("expected name for: %s", #n);  \
    var_pretty_print("received: ", value_1); \
  }
#define IS_COL(n)                             \
  if (value_1->type != VAR_COLOUR) {          \
    SEN_ERROR("expected colour for: %s", #n); \
    var_pretty_print("received: ", value_1);  \
  }
#define IS_LONG(n)                           \
  if (value_1->type != VAR_LONG) {           \
    SEN_ERROR("expected long for: %s", #n);  \
    var_pretty_print("received: ", value_1); \
  }
#else
#define IS_F32
#define IS_I32
#define IS_COL
#define IS_LONG
#endif

#define READ_STACK_ARG_F32(k, n) \
  if (name_1 == k) {             \
    IS_F32(n);                   \
    n = value_1->value.f;        \
  }
#define READ_STACK_ARG_I32(k, n) \
  if (name_1 == k) {             \
    IS_I32(n);                   \
    n = value_1->value.i;        \
  }
#define READ_STACK_ARG_NAME(k, n) \
  if (name_1 == k) {              \
    IS_NAME(n);                   \
    n = value_1->value.i;         \
  }
#define READ_STACK_ARG_VAR(k, n) \
  if (name_1 == k) {             \
    n = value_1;                 \
  }

#define READ_STACK_ARG_COL(k, n)          \
  if (name_1 == k) {                      \
    IS_COL(#n);                           \
    n.format     = value_1->value.i;      \
    n.element[0] = value_1->f32_array[0]; \
    n.element[1] = value_1->f32_array[1]; \
    n.element[2] = value_1->f32_array[2]; \
    n.element[3] = value_1->f32_array[3]; \
  }

#define READ_STACK_ARG_VEC2(k, n) \
  if (name_1 == k) {              \
    n[0] = value_1->f32_array[0]; \
    n[1] = value_1->f32_array[1]; \
  }

// todo: eventually just pack a vec3 into a single sen_var's f32_array
//
#define READ_STACK_ARG_VEC3(k, n) \
  if (name_1 == k) {              \
    tmp_1   = value_1;            \
    value_1 = value_1->value.v;   \
    IS_F32(#n);                   \
    n[0]    = value_1->value.f;   \
    value_1 = value_1->next;      \
    IS_F32(#n);                   \
    n[1]    = value_1->value.f;   \
    value_1 = value_1->next;      \
    IS_F32(#n);                   \
    n[2]    = value_1->value.f;   \
    value_1 = tmp_1;              \
  }

// todo: eventually just pack a vec4 into a single sen_var's f32_array
//
#define READ_STACK_ARG_VEC4(k, n) \
  if (name_1 == k) {              \
    tmp_1   = value_1;            \
    value_1 = value_1->value.v;   \
    IS_F32(#n);                   \
    n[0]    = value_1->value.f;   \
    value_1 = value_1->next;      \
    IS_F32(#n);                   \
    n[1]    = value_1->value.f;   \
    value_1 = value_1->next;      \
    IS_F32(#n);                   \
    n[2]    = value_1->value.f;   \
    value_1 = value_1->next;      \
    IS_F32(#n);                   \
    n[3]    = value_1->value.f;   \
    value_1 = tmp_1;              \
  }

#define READ_STACK_ARG_PRNG(k, n)           \
  if (name_1 == k) {                        \
    tmp_1   = value_1;                      \
    value_1 = value_1->value.v;             \
    IS_I32(#n);                             \
    n.structure_id = value_1->value.i;      \
    n.min          = value_1->f32_array[0]; \
    n.max          = value_1->f32_array[1]; \
    value_1        = value_1->next;         \
    IS_LONG(#n);                            \
    n.state0         = value_1->value.l;    \
    n.sen_var_state0 = value_1;             \
    value_1          = value_1->next;       \
    IS_LONG(#n);                            \
    n.state1         = value_1->value.l;    \
    n.sen_var_state1 = value_1;             \
    value_1          = tmp_1;               \
  }

#define READ_STACK_ARG_PARAMETRIC(k, n)     \
  if (name_1 == k) {                        \
    tmp_1   = value_1;                      \
    value_1 = value_1->value.v;             \
    IS_I32(#n);                             \
    n.structure_id = value_1->value.i;      \
    n.from_m       = value_1->f32_array[0]; \
    n.to_m         = value_1->f32_array[1]; \
    n.from_c       = value_1->f32_array[2]; \
    n.to_c         = value_1->f32_array[3]; \
    value_1        = value_1->next;         \
    IS_I32(#n);                             \
    n.clamping = value_1->value.i;          \
    n.to0      = value_1->f32_array[0];     \
    n.to1      = value_1->f32_array[1];     \
    value_1    = value_1->next;             \
    IS_I32(#n);                             \
    n.mapping = value_1->value.i;           \
    value_1   = tmp_1;                      \
  }

#define READ_STACK_ARG_FOCAL(k, n)          \
  if (name_1 == k) {                        \
    tmp_1   = value_1;                      \
    value_1 = value_1->value.v;             \
    IS_I32(#n);                             \
    n.structure_id = value_1->value.i;      \
    n.x            = value_1->f32_array[0]; \
    n.y            = value_1->f32_array[1]; \
    n.distance     = value_1->f32_array[2]; \
    value_1        = value_1->next;         \
    IS_I32(#n);                             \
    n.type  = value_1->value.i;             \
    value_1 = value_1->next;                \
    IS_I32(#n);                             \
    n.mapping = value_1->value.i;           \
    value_1   = tmp_1;                      \
  }

#define READ_STACK_ARG_COORD3(k, n) \
  if (name_1 == k) {                \
    tmp_1 = value_1->value.v;       \
    n[0]  = tmp_1->f32_array[0];    \
    n[1]  = tmp_1->f32_array[1];    \
    tmp_1 = tmp_1->next;            \
    n[2]  = tmp_1->f32_array[0];    \
    n[3]  = tmp_1->f32_array[1];    \
    tmp_1 = tmp_1->next;            \
    n[4]  = tmp_1->f32_array[0];    \
    n[5]  = tmp_1->f32_array[1];    \
  }

#define READ_STACK_ARG_COORD4(k, n) \
  if (name_1 == k) {                \
    tmp_1 = value_1->value.v;       \
    n[0]  = tmp_1->f32_array[0];    \
    n[1]  = tmp_1->f32_array[1];    \
    tmp_1 = tmp_1->next;            \
    n[2]  = tmp_1->f32_array[0];    \
    n[3]  = tmp_1->f32_array[1];    \
    tmp_1 = tmp_1->next;            \
    n[4]  = tmp_1->f32_array[0];    \
    n[5]  = tmp_1->f32_array[1];    \
    tmp_1 = tmp_1->next;            \
    n[6]  = tmp_1->f32_array[0];    \
    n[7]  = tmp_1->f32_array[1];    \
  }

#define READ_STACK_ARG_COLFN(k, n)   \
  if (name_1 == k) {                 \
    tmp_1   = value_1;               \
    value_1 = value_1->value.v;      \
    IS_I32(#n);                      \
    n.type  = value_1->value.i;      \
    n.a[0]  = value_1->f32_array[0]; \
    n.a[1]  = value_1->f32_array[1]; \
    n.a[2]  = value_1->f32_array[2]; \
    n.a[3]  = value_1->f32_array[3]; \
    value_1 = value_1->next;         \
    n.alpha = value_1->value.f;      \
    n.b[0]  = value_1->f32_array[0]; \
    n.b[1]  = value_1->f32_array[1]; \
    n.b[2]  = value_1->f32_array[2]; \
    n.b[3]  = value_1->f32_array[3]; \
    value_1 = value_1->next;         \
    n.c[0]  = value_1->f32_array[0]; \
    n.c[1]  = value_1->f32_array[1]; \
    n.c[2]  = value_1->f32_array[2]; \
    n.c[3]  = value_1->f32_array[3]; \
    value_1 = value_1->next;         \
    n.d[0]  = value_1->f32_array[0]; \
    n.d[1]  = value_1->f32_array[1]; \
    n.d[2]  = value_1->f32_array[2]; \
    n.d[3]  = value_1->f32_array[3]; \
    value_1 = tmp_1;                 \
  }

// a global var that represents true, used as the default
// return type for bindings that only have side-effects
//
sen_var g_var_true;
// temporary sen_var, returned by native functions
sen_var g_var_scratch;

void declare_vm_keyword(sen_word_lut* word_lut, char* name) {
  bool res = wlut_add_keyword(word_lut, name);
  if (res == false) {
    SEN_ERROR("declare_vm_keyword: failed to add keword: %s", name);
  }
}

void declare_native(sen_word_lut* word_lut, sen_env* e, char* name,
                    native_function_ptr function_ptr) {
  e->function_ptr[word_lut->native_count] = function_ptr;

  bool res = wlut_add_native(word_lut, name);
  if (res == false) {
    SEN_ERROR("declare_native: failed to add keword: %s", name);
  }
}

sen_var* bind_debug_print(sen_vm* vm, i32 num_args) {
  sen_var* value = NULL;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VAR(INAME_VALUE, value);
  READ_STACK_ARGS_END;

  var_pretty_print("debug", value);

  return &g_var_true;
}

sen_var* bind_nth(sen_vm* vm, i32 num_args) {
  sen_var* from = NULL;
  f32      n    = 0;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VAR(INAME_FROM, from);
  READ_STACK_ARG_F32(INAME_N, n);
  READ_STACK_ARGS_END;

  i32 nth = (i32)n;

  if (from->type == VAR_2D && nth >= 0 && nth < 2) {

    f32_as_var(&g_var_scratch, from->f32_array[nth]);

  } else if (from->type == VAR_VECTOR) {

    sen_var* e = from->value.v;

    for (i32 i = 0; i < nth; i++) {
      e = e->next;
    }

    var_copy(&g_var_scratch, e);
  } else {
    SEN_ERROR("nth: neither a var_2d with n 0..2 or vector given\n");
    return &g_var_true;
  }

  return &g_var_scratch;
}

sen_var* bind_vector_length(sen_vm* vm, i32 num_args) {
  sen_var* vector = NULL;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VAR(INAME_VECTOR, vector);
  READ_STACK_ARGS_END;

  if (vector == NULL) {
    SEN_ERROR("vector/length expects a 'vector' parameter");
    return &g_var_true;
  }

  i32 len = vector_length(vector);

  f32_as_var(&g_var_scratch, (f32)len);

  return &g_var_scratch;
}

sen_var* bind_line(sen_vm* vm, i32 num_args) {
  // default values for line
  f32        width  = 4.0f;
  f32        from[] = {10.0f, 10.0f};
  f32        to[]   = {900.0f, 500.0f};
  sen_colour colour;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_WIDTH, width);
  READ_STACK_ARG_VEC2(INAME_FROM, from);
  READ_STACK_ARG_VEC2(INAME_TO, to);
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  sen_render_data* render_data = vm->render_data;
  sen_matrix*      matrix      = matrix_stack_peek(vm->matrix_stack);

  render_line(render_data, matrix, from[0], from[1], to[0], to[1], width,
              &colour);

  return &g_var_true;
}

sen_var* bind_rect(sen_vm* vm, i32 num_args) {
  // default values for rect
  f32        width      = 4.0f;
  f32        height     = 10.0f;
  f32        position[] = {10.0f, 23.0f};
  sen_colour colour;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_WIDTH, width);
  READ_STACK_ARG_F32(INAME_HEIGHT, height);
  READ_STACK_ARG_VEC2(INAME_POSITION, position);
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  sen_render_data* render_data = vm->render_data;
  sen_matrix*      matrix      = matrix_stack_peek(vm->matrix_stack);

  render_rect(render_data, matrix, position[0], position[1], width, height,
              &colour);

  return &g_var_true;
}

sen_var* bind_circle(sen_vm* vm, i32 num_args) {
  // default values for circle
  f32        width      = 4.0f;
  f32        height     = 10.0f;
  f32        position[] = {10.0f, 23.0f};
  sen_colour colour;
  f32        tessellation = 10.0f;
  f32        radius       = -1.0f;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_WIDTH, width);
  READ_STACK_ARG_F32(INAME_HEIGHT, height);
  READ_STACK_ARG_F32(INAME_RADIUS, radius);
  READ_STACK_ARG_VEC2(INAME_POSITION, position);
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARG_F32(INAME_TESSELLATION, tessellation);
  READ_STACK_ARGS_END;

  // if the radius has been defined then it overrides the width and height
  // parameters
  if (radius > 0.0f) {
    width  = radius;
    height = radius;
  }

  sen_render_data* render_data = vm->render_data;
  sen_matrix*      matrix      = matrix_stack_peek(vm->matrix_stack);

  render_circle(render_data, matrix, position[0], position[1], width, height,
                &colour, (i32)tessellation);

  return &g_var_true;
}

sen_var* bind_circle_slice(sen_vm* vm, i32 num_args) {
  // default values for circle-slice
  f32        width      = 4.0f;
  f32        height     = 10.0f;
  f32        radius     = -1.0f;
  f32        position[] = {10.0f, 23.0f};
  sen_colour colour;
  f32        tessellation = 10.0f;
  f32        angle_start  = 0.0f; // degrees
  f32        angle_end    = 0.0f;
  f32        inner_width  = 1.0f;
  f32        inner_height = 1.0f;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_WIDTH, width);
  READ_STACK_ARG_F32(INAME_HEIGHT, height);
  READ_STACK_ARG_F32(INAME_RADIUS, radius);
  READ_STACK_ARG_VEC2(INAME_POSITION, position);
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARG_F32(INAME_TESSELLATION, tessellation);
  READ_STACK_ARG_F32(INAME_ANGLE_START, angle_start);
  READ_STACK_ARG_F32(INAME_ANGLE_END, angle_end);
  READ_STACK_ARG_F32(INAME_INNER_WIDTH, inner_width);
  READ_STACK_ARG_F32(INAME_INNER_HEIGHT, inner_height);
  READ_STACK_ARGS_END;

  // if the radius has been defined then it overrides the width and height
  // parameters
  if (radius > 0.0f) {
    width  = radius;
    height = radius;
  }

  sen_render_data* render_data = vm->render_data;
  sen_matrix*      matrix      = matrix_stack_peek(vm->matrix_stack);

  render_circle_slice(render_data, matrix, position[0], position[1], width,
                      height, &colour, (i32)tessellation, angle_start,
                      angle_end, inner_width, inner_height);

  return &g_var_true;
}

sen_var* bind_poly(sen_vm* vm, i32 num_args) {
  // default values for poly
  sen_var* coords  = NULL;
  sen_var* colours = NULL;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VAR(INAME_COORDS, coords);
  READ_STACK_ARG_VAR(INAME_COLOURS, colours);
  READ_STACK_ARGS_END;

  sen_render_data* render_data = vm->render_data;
  sen_matrix*      matrix      = matrix_stack_peek(vm->matrix_stack);

  render_poly(render_data, matrix, coords, colours);

  return &g_var_true;
}

sen_var* bind_bezier(sen_vm* vm, i32 num_args) {
  // default values for bezier
  f32        line_width         = -1.0f;
  f32        line_width_start   = 4.0f;
  f32        line_width_end     = 4.0f;
  i32        line_width_mapping = INAME_LINEAR;
  f32        coords[]           = {100.0f, 500.0f, 300.0f, 300.0f,
                  600.0f, 700.0f, 900.0f, 500.0f};
  f32        t_start            = -1.0f;
  f32        t_end              = 2.0f;
  f32        tessellation       = 10.0f;
  sen_colour colour;
  i32        brush         = INAME_BRUSH_FLAT;
  f32        brush_subtype = 0.0f;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // line_width_mapping will be one of several constants

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_LINE_WIDTH, line_width);
  READ_STACK_ARG_F32(INAME_LINE_WIDTH_START, line_width_start);
  READ_STACK_ARG_F32(INAME_LINE_WIDTH_END, line_width_end);
  READ_STACK_ARG_I32(INAME_LINE_WIDTH_MAPPING, line_width_mapping);
  READ_STACK_ARG_COORD4(INAME_COORDS, coords);
  READ_STACK_ARG_F32(INAME_T_START, t_start);
  READ_STACK_ARG_F32(INAME_T_END, t_end);
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARG_F32(INAME_TESSELLATION, tessellation);
  READ_STACK_ARG_I32(INAME_BRUSH, brush);
  READ_STACK_ARG_F32(INAME_BRUSH_SUBTYPE, brush_subtype);
  READ_STACK_ARGS_END;

  // if the line_width has been defined then it overrides the separate start/end
  // variables
  if (line_width > 0.0f) {
    line_width_start = line_width;
    line_width_end   = line_width;
  }

  if (t_start < 0.0f) {
    t_start = 0.0f;
  }

  if (t_end > 1.0f) {
    t_end = 1.0f;
  }

  sen_render_data* render_data = vm->render_data;
  sen_matrix*      matrix      = matrix_stack_peek(vm->matrix_stack);

  render_bezier(render_data, matrix, coords, line_width_start, line_width_end,
                line_width_mapping, t_start, t_end, &colour, (i32)tessellation,
                brush, (i32)brush_subtype);

  return &g_var_true;
}

sen_var* bind_bezier_bulging(sen_vm* vm, i32 num_args) {
  // default values for bezier
  f32 line_width = 5.0f;
  // f32 line_width_start = 4.0f;
  // f32 line_width_end = 4.0f;
  // i32 line_width_mapping = INAME_LINEAR;
  f32        coords[]     = {100.0f, 500.0f, 300.0f, 300.0f,
                  600.0f, 700.0f, 900.0f, 500.0f};
  f32        t_start      = -1.0f;
  f32        t_end        = 2.0f;
  f32        tessellation = 10.0f;
  sen_colour colour;
  i32        brush         = INAME_BRUSH_FLAT;
  f32        brush_subtype = 0.0f;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_LINE_WIDTH, line_width);
  // READ_STACK_ARG_F32(INAME_LINE_WIDTH_START, line_width_start);
  // READ_STACK_ARG_F32(INAME_LINE_WIDTH_END, line_width_end);
  // READ_STACK_ARG_I32(INAME_LINE_WIDTH_MAPPING, line_width_mapping);
  READ_STACK_ARG_COORD4(INAME_COORDS, coords);
  READ_STACK_ARG_F32(INAME_T_START, t_start);
  READ_STACK_ARG_F32(INAME_T_END, t_end);
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARG_F32(INAME_TESSELLATION, tessellation);
  READ_STACK_ARG_I32(INAME_BRUSH, brush);
  READ_STACK_ARG_F32(INAME_BRUSH_SUBTYPE, brush_subtype);
  READ_STACK_ARGS_END;

  if (t_start < 0.0f) {
    t_start = 0.0f;
  }

  if (t_end > 1.0f) {
    t_end = 1.0f;
  }

  sen_render_data* render_data = vm->render_data;
  sen_matrix*      matrix      = matrix_stack_peek(vm->matrix_stack);

  render_bezier_bulging(render_data, matrix, coords, line_width, t_start, t_end,
                        &colour, (i32)tessellation, brush, (i32)brush_subtype);

  return &g_var_true;
}

sen_var* bind_stroked_bezier(sen_vm* vm, i32 num_args) {
  // default values for stroked-bezier
  f32        tessellation            = 15.0f;
  f32        coords[]                = {100.0f, 500.0f, 300.0f, 300.0f,
                  600.0f, 700.0f, 900.0f, 500.0f};
  f32        stroke_tessellation     = 10.0f;
  f32        stroke_noise            = 25;
  f32        stroke_line_width_start = 1.0f;
  f32        stroke_line_width_end   = 1.0f;
  sen_colour colour;
  f32        colour_volatility = 0.0f;
  f32        seed              = 0.0f;

  i32 line_width_mapping = INAME_LINEAR;
  i32 brush              = INAME_BRUSH_FLAT;
  f32 brush_subtype      = 0.0f;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // line_width_mapping will be one of several constants

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_TESSELLATION, tessellation);
  READ_STACK_ARG_COORD4(INAME_COORDS, coords);
  READ_STACK_ARG_F32(INAME_STROKE_TESSELLATION, stroke_tessellation);
  READ_STACK_ARG_F32(INAME_STROKE_NOISE, stroke_noise);
  READ_STACK_ARG_F32(INAME_STROKE_LINE_WIDTH_START, stroke_line_width_start);
  READ_STACK_ARG_F32(INAME_STROKE_LINE_WIDTH_END, stroke_line_width_end);
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARG_F32(INAME_COLOUR_VOLATILITY, colour_volatility);
  READ_STACK_ARG_F32(INAME_SEED, seed);
  READ_STACK_ARG_I32(INAME_LINE_WIDTH_MAPPING, line_width_mapping);
  READ_STACK_ARG_I32(INAME_BRUSH, brush);
  READ_STACK_ARG_F32(INAME_BRUSH_SUBTYPE, brush_subtype);
  READ_STACK_ARGS_END;

  sen_render_data* render_data = vm->render_data;
  sen_matrix*      matrix      = matrix_stack_peek(vm->matrix_stack);

  render_stroked_bezier(render_data, matrix, coords, &colour, (i32)tessellation,
                        stroke_line_width_start, stroke_line_width_end,
                        stroke_noise, (i32)stroke_tessellation,
                        colour_volatility, seed, line_width_mapping, brush,
                        (i32)brush_subtype);

  return &g_var_true;
}

sen_var* bind_stroked_bezier_rect(sen_vm* vm, i32 num_args) {
  // default values for stroked-bezier-rect
  f32        position[]          = {100.0f, 100.0f};
  f32        width               = 800.0f;
  f32        height              = 600.0f;
  f32        volatility          = 30.0f;
  f32        overlap             = 0.0f;
  f32        iterations          = 10.0f;
  f32        seed                = 0.0f;
  f32        tessellation        = 15;
  f32        stroke_tessellation = 10.0f;
  f32        stroke_noise        = 25;
  sen_colour colour;
  f32        colour_volatility = 40.0f;
  i32        brush             = INAME_BRUSH_FLAT;
  f32        brush_subtype     = 0.0f;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VEC2(INAME_POSITION, position);
  READ_STACK_ARG_F32(INAME_WIDTH, width);
  READ_STACK_ARG_F32(INAME_HEIGHT, height);
  READ_STACK_ARG_F32(INAME_VOLATILITY, volatility);
  READ_STACK_ARG_F32(INAME_OVERLAP, overlap);
  READ_STACK_ARG_F32(INAME_ITERATIONS, iterations);
  READ_STACK_ARG_F32(INAME_SEED, seed);
  READ_STACK_ARG_F32(INAME_TESSELLATION, tessellation);
  READ_STACK_ARG_F32(INAME_STROKE_TESSELLATION, stroke_tessellation);
  READ_STACK_ARG_F32(INAME_STROKE_NOISE, stroke_noise);
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARG_F32(INAME_COLOUR_VOLATILITY, colour_volatility);
  READ_STACK_ARG_I32(INAME_BRUSH, brush);
  READ_STACK_ARG_F32(INAME_BRUSH_SUBTYPE, brush_subtype);
  READ_STACK_ARGS_END;

  sen_render_data* render_data = vm->render_data;
  sen_matrix*      matrix      = matrix_stack_peek(vm->matrix_stack);

  render_stroked_bezier_rect(
      render_data, matrix, position, width, height, volatility, overlap,
      iterations, seed, (i32)tessellation, (i32)stroke_tessellation,
      stroke_noise, &colour, colour_volatility, brush, (i32)brush_subtype);

  return &g_var_true;
}

sen_var* bind_col_convert(sen_vm* vm, i32 num_args) {
  // (col/convert colour: col format: LAB)

  i32        format = INAME_RGB;
  sen_colour colour;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_NAME(INAME_FORMAT, format);
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  // the sen_var referencing the converted colour is going to be added to the
  // VM's stack so we need to get the referenced colour from the vm
  //
  sen_colour        out;
  sen_colour_format colour_format = RGB;

  if (format == INAME_RGB) {
    colour_format = RGB;
  } else if (format == INAME_HSL) {
    colour_format = HSL;
  } else if (format == INAME_LAB) {
    colour_format = LAB;
  } else if (format == INAME_HSV) {
    colour_format = HSV;
  } else if (format == INAME_HSLUV) {
    colour_format = HSLuv;
  } else {
    SEN_ERROR("bind_col_convert: unknown colour format: %d", format);
  }

  colour_clone_as(&out, &colour, colour_format);

  colour_as_var(&g_var_scratch, &out);
  return &g_var_scratch;
}

sen_var* bind_col_rgb(sen_vm* vm, i32 num_args) {
  // (col/rgb r: 0.627 g: 0.627 b: 0.627 alpha: 0.4)

  // default values for line
  f32 r     = 0.0f; // 0..1
  f32 g     = 0.0f; // 0..1
  f32 b     = 0.0f; // 0..1
  f32 alpha = 1.0f; // 0..1

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_R, r);
  READ_STACK_ARG_F32(INAME_G, g);
  READ_STACK_ARG_F32(INAME_B, b);
  READ_STACK_ARG_F32(INAME_ALPHA, alpha);
  READ_STACK_ARGS_END;

  sen_colour colour;
  colour.format     = RGB;
  colour.element[0] = r;
  colour.element[1] = g;
  colour.element[2] = b;
  colour.element[3] = alpha;

  colour_as_var(&g_var_scratch, &colour);
  return &g_var_scratch;
}

sen_var* bind_col_hsl(sen_vm* vm, i32 num_args) {
  // (col/hsl h: 180.0 s: 0.1 l: 0.2 alpha: 0.4)

  // default values for line
  f32 h     = 0.0f; // 0..360
  f32 s     = 0.0f; // 0..1
  f32 l     = 0.0f; // 0..1
  f32 alpha = 1.0f; // 0..1

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_H, h);
  READ_STACK_ARG_F32(INAME_S, s);
  READ_STACK_ARG_F32(INAME_L, l);
  READ_STACK_ARG_F32(INAME_ALPHA, alpha);
  READ_STACK_ARGS_END;

  sen_colour colour;
  colour.format     = HSL;
  colour.element[0] = h;
  colour.element[1] = s;
  colour.element[2] = l;
  colour.element[3] = alpha;

  colour_as_var(&g_var_scratch, &colour);
  return &g_var_scratch;
}

sen_var* bind_col_hsluv(sen_vm* vm, i32 num_args) {
  // (col/hsluv h: 180.0 s: 0.1 l: 0.2 alpha: 0.4)

  // default values for line
  f32 h     = 0.0f; // 0..360
  f32 s     = 0.0f; // 0..100
  f32 l     = 0.0f; // 0..100
  f32 alpha = 1.0f; // 0..1

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_H, h);
  READ_STACK_ARG_F32(INAME_S, s);
  READ_STACK_ARG_F32(INAME_L, l);
  READ_STACK_ARG_F32(INAME_ALPHA, alpha);
  READ_STACK_ARGS_END;

  sen_colour colour;
  colour.format     = HSLuv;
  colour.element[0] = h;
  colour.element[1] = s;
  colour.element[2] = l;
  colour.element[3] = alpha;

  colour_as_var(&g_var_scratch, &colour);
  return &g_var_scratch;
}

sen_var* bind_col_hsv(sen_vm* vm, i32 num_args) {
  // (col/hsv h: 180.0 s: 0.1 v: 0.2 alpha: 0.4)

  // default values for line
  f32 h     = 0.0f; // 0..360
  f32 s     = 0.0f; // 0..1
  f32 v     = 0.0f; // 0..1
  f32 alpha = 1.0f; // 0..1

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_H, h);
  READ_STACK_ARG_F32(INAME_S, s);
  READ_STACK_ARG_F32(INAME_V, v);
  READ_STACK_ARG_F32(INAME_ALPHA, alpha);
  READ_STACK_ARGS_END;

  sen_colour colour;
  colour.format     = HSV;
  colour.element[0] = h;
  colour.element[1] = s;
  colour.element[2] = v;
  colour.element[3] = alpha;

  colour_as_var(&g_var_scratch, &colour);
  return &g_var_scratch;
}

sen_var* bind_col_lab(sen_vm* vm, i32 num_args) {
  // (col/lab l: 0.2 a: -0.1 b: -0.3 alpha: 0.4)

  // default values for line
  f32 l     = 0.0f; // 0..
  f32 a     = 0.0f; // -1..1
  f32 b     = 0.0f; // -1..1
  f32 alpha = 1.0f; // 0..1

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_L, l);
  READ_STACK_ARG_F32(INAME_A, a);
  READ_STACK_ARG_F32(INAME_B, b);
  READ_STACK_ARG_F32(INAME_ALPHA, alpha);
  READ_STACK_ARGS_END;

  sen_colour colour;
  colour.format     = LAB;
  colour.element[0] = l;
  colour.element[1] = a;
  colour.element[2] = b;
  colour.element[3] = alpha;

  colour_as_var(&g_var_scratch, &colour);
  return &g_var_scratch;
}

sen_var* bind_col_complementary(sen_vm* vm, i32 num_args) {
  sen_colour colour;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  sen_colour ret_colour;
  complementary(&ret_colour, &colour);

  colour_as_var(&g_var_scratch, &ret_colour);
  return &g_var_scratch;
}

sen_var* bind_col_split_complementary(sen_vm* vm, i32 num_args) {
  sen_colour colour;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  sen_colour ret_colour0;
  sen_colour ret_colour1;
  split_complementary(&ret_colour0, &ret_colour1, &colour);

  // push the return values onto the stack as a vector

  vector_construct(&g_var_scratch);
  vector_append_col(vm, &g_var_scratch, &ret_colour0);
  vector_append_col(vm, &g_var_scratch, &ret_colour1);

  return &g_var_scratch;
}

sen_var* bind_col_analagous(sen_vm* vm, i32 num_args) {
  sen_colour colour;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  sen_colour ret_colour0;
  sen_colour ret_colour1;
  analagous(&ret_colour0, &ret_colour1, &colour);

  // push the return values onto the stack as a vector
  vector_construct(&g_var_scratch);
  vector_append_col(vm, &g_var_scratch, &ret_colour0);
  vector_append_col(vm, &g_var_scratch, &ret_colour1);

  return &g_var_scratch;
}

sen_var* bind_col_triad(sen_vm* vm, i32 num_args) {
  sen_colour colour;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  sen_colour ret_colour0;
  sen_colour ret_colour1;
  triad(&ret_colour0, &ret_colour1, &colour);

  // push the return values onto the stack as a vector
  vector_construct(&g_var_scratch);
  vector_append_col(vm, &g_var_scratch, &ret_colour0);
  vector_append_col(vm, &g_var_scratch, &ret_colour1);

  return &g_var_scratch;
}

sen_var* bind_col_darken(sen_vm* vm, i32 num_args) {
  sen_colour colour;
  f32        value = 0; // 0..100

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARG_F32(INAME_VALUE, value);
  READ_STACK_ARGS_END;

  sen_colour ret_colour;

  colour_clone_as(&ret_colour, &colour, LAB);
  ret_colour.element[0] = clamp(ret_colour.element[0] - value, 0.0f, 100.0f);

  colour_as_var(&g_var_scratch, &ret_colour);
  return &g_var_scratch;
}

sen_var* bind_col_lighten(sen_vm* vm, i32 num_args) {
  sen_colour colour;
  f32        value = 0; // 0..100

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARG_F32(INAME_VALUE, value);
  READ_STACK_ARGS_END;

  sen_colour ret_colour;

  colour_clone_as(&ret_colour, &colour, LAB);
  ret_colour.element[0] = clamp(ret_colour.element[0] + value, 0.0f, 100.0f);

  colour_as_var(&g_var_scratch, &ret_colour);
  return &g_var_scratch;
}

sen_var* bind_col_set_alpha(sen_vm* vm, i32 num_args) {
  sen_colour colour;
  f32        value = 0;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARG_F32(INAME_VALUE, value);
  READ_STACK_ARGS_END;

  sen_colour ret_colour;

  colour_clone_as(&ret_colour, &colour, colour.format);
  ret_colour.element[3] = value;

  colour_as_var(&g_var_scratch, &ret_colour);
  return &g_var_scratch;
}

sen_var* bind_col_get_alpha(sen_vm* vm, i32 num_args) {
  sen_colour colour;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  f32_as_var(&g_var_scratch, colour.element[3]);

  return &g_var_scratch;
}

sen_var* col_set_element(sen_vm* vm, i32 num_args, sen_colour_format format,
                         i32 index) {
  sen_colour colour;
  f32        value = 0;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARG_F32(INAME_VALUE, value);
  READ_STACK_ARGS_END;

  sen_colour ret_colour;
  colour_clone_as(&ret_colour, &colour, colour.format);

  if (colour.format == format) {
    ret_colour.element[index] = value;
  } else {
    SEN_ERROR("col_set_element: colour is in the incorrect format");
  }

  colour_as_var(&g_var_scratch, &ret_colour);

  return &g_var_scratch;
}

sen_var* col_get_element(sen_vm* vm, i32 num_args, sen_colour_format format,
                         i32 index) {
  sen_colour colour;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  f32 value = 0.0f;

  if (colour.format == format) {
    value = colour.element[index];
  } else {
    SEN_ERROR("col_get_element: colour is in the incorrect format");
  }

  f32_as_var(&g_var_scratch, value);

  return &g_var_scratch;
}

sen_var* bind_col_set_r(sen_vm* vm, i32 num_args) {
  return col_set_element(vm, num_args, RGB, 0);
}

sen_var* bind_col_get_r(sen_vm* vm, i32 num_args) {
  return col_get_element(vm, num_args, RGB, 0);
}

sen_var* bind_col_set_g(sen_vm* vm, i32 num_args) {
  return col_set_element(vm, num_args, RGB, 1);
}

sen_var* bind_col_get_g(sen_vm* vm, i32 num_args) {
  return col_get_element(vm, num_args, RGB, 1);
}

sen_var* bind_col_set_b(sen_vm* vm, i32 num_args) {
  sen_colour colour;
  f32        value = 0;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARG_F32(INAME_VALUE, value);
  READ_STACK_ARGS_END;

  sen_colour ret_colour;
  colour_clone_as(&ret_colour, &colour, colour.format);

  i32 index = 2;

  if (colour.format == RGB || colour.format == LAB) {
    ret_colour.element[index] = value;
  } else {
    SEN_ERROR("bind_col_set_b: colour is in the incorrect format");
  }

  colour_as_var(&g_var_scratch, &ret_colour);

  return &g_var_scratch;
}

sen_var* bind_col_get_b(sen_vm* vm, i32 num_args) {
  sen_colour colour;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  f32 value = 0.0f;
  i32 index = 2;

  if (colour.format == RGB || colour.format == LAB) {
    value = colour.element[index];
  } else {
    SEN_ERROR("bind_col_get_b: colour is in the incorrect format");
  }

  f32_as_var(&g_var_scratch, value);

  return &g_var_scratch;
}

sen_var* bind_col_set_h(sen_vm* vm, i32 num_args) {
  sen_colour colour;
  f32        value = 0;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARG_F32(INAME_VALUE, value);
  READ_STACK_ARGS_END;

  sen_colour ret_colour;
  colour_clone_as(&ret_colour, &colour, colour.format);

  i32 index = 0;

  if (colour.format == HSL || colour.format == HSLuv || colour.format == HSV) {
    ret_colour.element[index] = value;
  } else {
    SEN_ERROR("bind_col_set_h: colour is in the incorrect format");
  }

  colour_as_var(&g_var_scratch, &ret_colour);

  return &g_var_scratch;
}

sen_var* bind_col_get_h(sen_vm* vm, i32 num_args) {
  sen_colour colour;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  f32 value = 0.0f;
  i32 index = 0;

  if (colour.format == HSL || colour.format == HSLuv || colour.format == HSV) {
    value = colour.element[index];
  } else {
    SEN_ERROR("bind_col_get_h: colour is in the incorrect format: %d",
              colour.format);
  }

  f32_as_var(&g_var_scratch, value);

  return &g_var_scratch;
}

sen_var* bind_col_set_s(sen_vm* vm, i32 num_args) {
  sen_colour colour;
  f32        value = 0;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARG_F32(INAME_VALUE, value);
  READ_STACK_ARGS_END;

  sen_colour ret_colour;
  colour_clone_as(&ret_colour, &colour, colour.format);

  i32 index = 1;

  if (colour.format == HSL || colour.format == HSLuv || colour.format == HSV) {
    ret_colour.element[index] = value;
  } else {
    SEN_ERROR("bind_col_set_s: colour is in the incorrect format");
  }

  colour_as_var(&g_var_scratch, &ret_colour);

  return &g_var_scratch;
}

sen_var* bind_col_get_s(sen_vm* vm, i32 num_args) {
  sen_colour colour;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  f32 value = 0.0f;
  i32 index = 1;

  if (colour.format == HSL || colour.format == HSLuv || colour.format == HSV) {
    value = colour.element[index];
  } else {
    SEN_ERROR("bind_col_get_s: colour is in the incorrect format");
  }

  f32_as_var(&g_var_scratch, value);

  return &g_var_scratch;
}

sen_var* bind_col_set_l(sen_vm* vm, i32 num_args) {
  sen_colour colour;
  f32        value = 0;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARG_F32(INAME_VALUE, value);
  READ_STACK_ARGS_END;

  sen_colour ret_colour;
  colour_clone_as(&ret_colour, &colour, colour.format);

  if (colour.format == HSL || colour.format == HSLuv) {
    ret_colour.element[2] = value;
  } else if (colour.format == LAB) {
    ret_colour.element[0] = value;
  } else {
    SEN_ERROR("bind_col_set_l: colour is in the incorrect format: %d",
              colour.format);
  }

  colour_as_var(&g_var_scratch, &ret_colour);

  return &g_var_scratch;
}

sen_var* bind_col_get_l(sen_vm* vm, i32 num_args) {
  sen_colour colour;

  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 1.0f);

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  f32 value = 0.0f;

  if (colour.format == HSL || colour.format == HSLuv) {
    value = colour.element[2];
  } else if (colour.format == LAB) {
    value = colour.element[0];
  } else {
    SEN_ERROR("bind_col_get_l: colour is in the incorrect format");
  }

  f32_as_var(&g_var_scratch, value);

  return &g_var_scratch;
}

sen_var* bind_col_set_a(sen_vm* vm, i32 num_args) {
  return col_set_element(vm, num_args, LAB, 1);
}

sen_var* bind_col_get_a(sen_vm* vm, i32 num_args) {
  return col_get_element(vm, num_args, LAB, 1);
}

sen_var* bind_col_set_v(sen_vm* vm, i32 num_args) {
  return col_set_element(vm, num_args, HSV, 2);
}

sen_var* bind_col_get_v(sen_vm* vm, i32 num_args) {
  return col_get_element(vm, num_args, HSV, 2);
}

sen_var* bind_col_build_procedural(sen_vm* vm, i32 num_args) {
  // colour fn structure need to store 4 colours (for bezier-fn)
  // first element's value.i will represent procedural, bezier or quadratic

  i32 preset = 0;
  f32 alpha  = 1.0f;
  f32 a[3], b[3], c[3], d[3];

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_NAME(INAME_PRESET, preset);
  READ_STACK_ARG_F32(INAME_ALPHA, alpha);
  READ_STACK_ARG_VEC3(INAME_A, a);
  READ_STACK_ARG_VEC3(INAME_B, b);
  READ_STACK_ARG_VEC3(INAME_C, c);
  READ_STACK_ARG_VEC3(INAME_D, d);
  READ_STACK_ARGS_END;

  if (preset != 0) {
    get_colour_presets(a, b, c, d, preset);
  }

  sen_var* v;
  vector_construct(&g_var_scratch);

  v               = var_get_from_heap(vm);
  v->type         = VAR_INT;
  v->value.i      = COLOUR_FN_PROCEDURAL;
  v->f32_array[0] = a[0];
  v->f32_array[1] = a[1];
  v->f32_array[2] = a[2];
  vector_append_heap_var(&g_var_scratch, v);

  v               = var_get_from_heap(vm);
  v->type         = VAR_FLOAT;
  v->value.f      = alpha;
  v->f32_array[0] = b[0];
  v->f32_array[1] = b[1];
  v->f32_array[2] = b[2];
  vector_append_heap_var(&g_var_scratch, v);

  v               = var_get_from_heap(vm);
  v->type         = VAR_INT;
  v->value.i      = 0;
  v->f32_array[0] = c[0];
  v->f32_array[1] = c[1];
  v->f32_array[2] = c[2];
  vector_append_heap_var(&g_var_scratch, v);

  v               = var_get_from_heap(vm);
  v->type         = VAR_INT;
  v->value.i      = 0;
  v->f32_array[0] = d[0];
  v->f32_array[1] = d[1];
  v->f32_array[2] = d[2];
  vector_append_heap_var(&g_var_scratch, v);

  return &g_var_scratch;
}

sen_var* bind_col_build_bezier(sen_vm* vm, i32 num_args) {
  // colour fn structure need to store 4 colours (for bezier-fn)
  // first element's value.i will represent procedural, bezier or quadratic

  sen_colour a, b, c, d;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_A, a);
  READ_STACK_ARG_COL(INAME_B, b);
  READ_STACK_ARG_COL(INAME_C, c);
  READ_STACK_ARG_COL(INAME_D, d);
  READ_STACK_ARGS_END;

  sen_colour a_rgb, b_rgb, c_rgb, d_rgb;

  // todo: maybe default to a different colour space (LAB?)
  colour_clone_as(&a_rgb, &a, RGB);
  colour_clone_as(&b_rgb, &b, RGB);
  colour_clone_as(&c_rgb, &c, RGB);
  colour_clone_as(&d_rgb, &d, RGB);

  sen_var* v;
  vector_construct(&g_var_scratch);

  v               = var_get_from_heap(vm);
  v->type         = VAR_INT;
  v->value.i      = COLOUR_FN_BEZIER;
  v->f32_array[0] = a_rgb.element[0];
  v->f32_array[1] = a_rgb.element[1];
  v->f32_array[2] = a_rgb.element[2];
  v->f32_array[3] = a_rgb.element[3];
  vector_append_heap_var(&g_var_scratch, v);

  v               = var_get_from_heap(vm);
  v->type         = VAR_FLOAT;
  v->value.f      = 1.0f;
  v->f32_array[0] = b_rgb.element[0];
  v->f32_array[1] = b_rgb.element[1];
  v->f32_array[2] = b_rgb.element[2];
  v->f32_array[3] = b_rgb.element[3];
  vector_append_heap_var(&g_var_scratch, v);

  v               = var_get_from_heap(vm);
  v->type         = VAR_INT;
  v->value.i      = 0;
  v->f32_array[0] = c_rgb.element[0];
  v->f32_array[1] = c_rgb.element[1];
  v->f32_array[2] = c_rgb.element[2];
  v->f32_array[3] = c_rgb.element[3];
  vector_append_heap_var(&g_var_scratch, v);

  v               = var_get_from_heap(vm);
  v->type         = VAR_INT;
  v->value.i      = 0;
  v->f32_array[0] = d_rgb.element[0];
  v->f32_array[1] = d_rgb.element[1];
  v->f32_array[2] = d_rgb.element[2];
  v->f32_array[3] = d_rgb.element[3];
  vector_append_heap_var(&g_var_scratch, v);

  return &g_var_scratch;
}

sen_var* bind_col_value(sen_vm* vm, i32 num_args) {
  sen_colour_fn_state colour_fn_state;
  f32                 t = 0.0f;

  colour_fn_state.type = COLOUR_FN_UNKNOWN;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COLFN(INAME_FROM, colour_fn_state);
  READ_STACK_ARG_F32(INAME_T, t);
  READ_STACK_ARGS_END;

  sen_colour ret_colour;

  if (colour_fn_state.type == COLOUR_FN_PROCEDURAL) {
    colour_procedural(&ret_colour, &colour_fn_state, t);
  } else if (colour_fn_state.type == COLOUR_FN_BEZIER) {
    colour_bezier(&ret_colour, &colour_fn_state, t);
  } else {
    SEN_ERROR("unknown colour_fn_state.type %d", colour_fn_state.type);
  }

  colour_as_var(&g_var_scratch, &ret_colour);
  return &g_var_scratch;
}

sen_var* bind_translate(sen_vm* vm, i32 num_args) {
  f32 vector[] = {0.0f, 0.0f};

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VEC2(INAME_VECTOR, vector);
  READ_STACK_ARGS_END;

  matrix_stack_translate(vm->matrix_stack, vector[0], vector[1]);

  return &g_var_true;
}

sen_var* bind_rotate(sen_vm* vm, i32 num_args) {
  // angle in degrees
  f32 angle = 0.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_ANGLE, angle);
  READ_STACK_ARGS_END;

  matrix_stack_rotate(vm->matrix_stack, deg_to_rad(angle));

  return &g_var_true;
}

sen_var* bind_scale(sen_vm* vm, i32 num_args) {
  f32 vector[] = {1.0f, 1.0f};
  f32 scalar   = 1.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VEC2(INAME_VECTOR, vector);
  READ_STACK_ARG_F32(INAME_SCALAR, scalar);
  READ_STACK_ARGS_END;

  if (scalar != 1.0f) {
    matrix_stack_scale(vm->matrix_stack, scalar, scalar);
  } else {
    matrix_stack_scale(vm->matrix_stack, vector[0], vector[1]);
  }

  return &g_var_true;
}

sen_var* bind_math_distance(sen_vm* vm, i32 num_args) {
  f32 vec1[] = {0.0f, 0.0f};
  f32 vec2[] = {0.0f, 0.0f};

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VEC2(INAME_VEC1, vec1);
  READ_STACK_ARG_VEC2(INAME_VEC2, vec2);
  READ_STACK_ARGS_END;

  f32 distance = distance_v2(vec1[0], vec1[1], vec2[0], vec2[1]);

  f32_as_var(&g_var_scratch, distance);

  return &g_var_scratch;
}

sen_var* bind_math_clamp(sen_vm* vm, i32 num_args) {
  // todo: try and move functions like this into ones that initially
  // create and return a function that takes a single argument.
  // e.g.
  // (define my-clamp (math/clamp-fn min: 0.0 max: 42.0))
  // (my-clamp val: 22)
  //
  // then optimize for single argument functions as these will be much faster to
  // parse
  //
  f32 value = 0.0f;
  f32 min   = 0.0f;
  f32 max   = 1.0f;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_VALUE, value);
  READ_STACK_ARG_F32(INAME_MIN, min);
  READ_STACK_ARG_F32(INAME_MAX, max);
  READ_STACK_ARGS_END;

  f32_as_var(&g_var_scratch, clamp(value, min, max));

  return &g_var_scratch;
}

sen_var* bind_math_radians_to_degrees(sen_vm* vm, i32 num_args) {
  f32 angle = 0.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_ANGLE, angle);
  READ_STACK_ARGS_END;

  f32_as_var(&g_var_scratch, rad_to_deg(angle));

  return &g_var_scratch;
}

sen_var* bind_math_cos(sen_vm* vm, i32 num_args) {
  f32 angle = 0.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_ANGLE, angle);
  READ_STACK_ARGS_END;

  f32_as_var(&g_var_scratch, (f32)cos(angle));

  return &g_var_scratch;
}

sen_var* bind_math_sin(sen_vm* vm, i32 num_args) {
  f32 angle = 0.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_ANGLE, angle);
  READ_STACK_ARGS_END;

  f32_as_var(&g_var_scratch, (f32)sin(angle));

  return &g_var_scratch;
}

// (prng/build seed: 4324 min: 40 max: 100)
sen_var* bind_prng_build(sen_vm* vm, i32 num_args) {
  f32 seed = 12322.0f; // todo: in docs mention that seed should be in
                       // range 1..some-large-number
  f32 min = 0.0f;
  f32 max = 1.0f;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_SEED, seed);
  READ_STACK_ARG_F32(INAME_MIN, min);
  READ_STACK_ARG_F32(INAME_MAX, max);
  READ_STACK_ARGS_END;

  // build a sen_prng_state and call it once - this always returns 0 but
  // further calls will be valid
  sen_prng_state prng_state;
  sen_prng_set_state(&prng_state, (u64)seed);

  // push the return values onto the stack as a vector
  // the vector needs to represent a sen_prng_state struct as well as the min +
  // max values i.e. [u64 state, u64 inc, f32 min, f32 max]
  //
  sen_var* v;
  vector_construct(&g_var_scratch);
  v               = vector_append_i32(vm, &g_var_scratch, HEAP_STRUCTURE_PRNG);
  v->f32_array[0] = min;
  v->f32_array[1] = max;
  vector_append_u64(vm, &g_var_scratch, prng_state.state[0]);
  vector_append_u64(vm, &g_var_scratch, prng_state.state[1]);

  return &g_var_scratch;
}

// (prng/values from: rng num: 5)
sen_var* bind_prng_values(sen_vm* vm, i32 num_args) {
  f32                 num = 1.0f;
  sen_prng_full_state from;

  from.structure_id   = HEAP_STRUCTURE_UNDEFINED;
  from.state0         = 2222;
  from.state1         = 1;
  from.min            = 0.0f;
  from.max            = 1.0f;
  from.sen_var_state0 = NULL;
  from.sen_var_state1 = NULL;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_NUM, num);
  READ_STACK_ARG_PRNG(INAME_FROM, from);
  READ_STACK_ARGS_END;

  if (from.structure_id != HEAP_STRUCTURE_PRNG) {
    SEN_ERROR("prng/values requires a user specified 'from' parameter");
    return &g_var_true;
  }

  // build a sen_prng_state from the sen_prng_full_state
  sen_prng_state prng_state;
  prng_state.state[0] = from.state0;
  prng_state.state[1] = from.state1;

  // create the return vector
  f32 value;

  i32 inum = (i32)num;

  vector_construct(&g_var_scratch);
  for (i32 i = 0; i < inum; i++) {
    value = sen_prng_f32_range(&prng_state, from.min, from.max);
    vector_append_f32(vm, &g_var_scratch, value);
  }

  // update the state and inc values stored in the vector on the vm's stack
  if (from.sen_var_state0 != NULL && from.sen_var_state1 != NULL) {
    from.sen_var_state0->value.l = prng_state.state[0];
    from.sen_var_state1->value.l = prng_state.state[1];
  } else {
    SEN_ERROR("sen_prng_full_state has null pointers ???");
  }

  return &g_var_scratch;
}

// (prng/value from: rng)
sen_var* bind_prng_value(sen_vm* vm, i32 num_args) {
  sen_prng_full_state from;

  from.structure_id   = HEAP_STRUCTURE_UNDEFINED;
  from.state0         = 2222;
  from.state1         = 1;
  from.min            = 0.0f;
  from.max            = 1.0f;
  from.sen_var_state0 = NULL;
  from.sen_var_state1 = NULL;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_PRNG(INAME_FROM, from);
  READ_STACK_ARGS_END;

  if (from.structure_id != HEAP_STRUCTURE_PRNG) {
    SEN_ERROR("prng/value requires a user specified 'from' parameter");
    return &g_var_true;
  }

  // build a sen_prng_state from the sen_prng_full_state
  sen_prng_state prng_state;
  prng_state.state[0] = from.state0;
  prng_state.state[1] = from.state1;

  f32 value = sen_prng_f32_range(&prng_state, from.min, from.max);
  f32_as_var(&g_var_scratch, value);

  // update the state and inc values stored in the vector on the vm's stack
  if (from.sen_var_state0 != NULL && from.sen_var_state1 != NULL) {
    from.sen_var_state0->value.l = prng_state.state[0];
    from.sen_var_state1->value.l = prng_state.state[1];
  } else {
    SEN_ERROR("sen_prng_full_state has null pointers ???");
  }

  return &g_var_scratch;
}

sen_var* bind_prng_perlin(sen_vm* vm, i32 num_args) {
  f32 x = 1.0f;
  f32 y = 1.0f;
  f32 z = 1.0f;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_X, x);
  READ_STACK_ARG_F32(INAME_Y, y);
  READ_STACK_ARG_F32(INAME_Z, z);
  READ_STACK_ARGS_END;

  f32 value = sen_perlin(x, y, z);

  f32_as_var(&g_var_scratch, value);

  return &g_var_scratch;
}

sen_var* bind_parametric_build(sen_vm* vm, i32 num_args) {
  f32 from[]   = {0.0f, 1.0f};
  f32 to[]     = {0.0f, 100.0f};
  i32 clamping = INAME_FALSE;
  i32 mapping  = INAME_LINEAR;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VEC2(INAME_FROM, from);
  READ_STACK_ARG_VEC2(INAME_TO, to);
  READ_STACK_ARG_NAME(INAME_CLAMPING,
                      clamping); // true | FALSE, clamping); // true | false
  READ_STACK_ARG_NAME(INAME_MAPPING,
                      mapping); // linear or one of the easing functions
  READ_STACK_ARGS_END;

  f32 from_m = mc_m(from[0], 0.0f, from[1], 1.0f);
  f32 from_c = mc_c(from[0], 0.0f, from_m);

  f32 to_m = mc_m(0.0f, to[0], 1.0f, to[1]);
  f32 to_c = mc_c(0.0f, to[0], to_m);

  // id to signify that this structure stores data for interpolation
  // todo: fill this out properly and do the same for the other structures
  sen_var* v;

  vector_construct(&g_var_scratch);
  v = vector_append_i32(vm, &g_var_scratch, HEAP_STRUCTURE_PARAMETRIC);
  v->f32_array[0] = from_m;
  v->f32_array[1] = to_m;
  v->f32_array[2] = from_c;
  v->f32_array[3] = to_c;
  v               = vector_append_i32(vm, &g_var_scratch, clamping);
  v->f32_array[0] = to[0];
  v->f32_array[1] = to[1];
  vector_append_i32(vm, &g_var_scratch, mapping);

  return &g_var_scratch;
}

sen_var* bind_parametric_value(sen_vm* vm, i32 num_args) {
  sen_parametric_state from;
  f32                  t = 0.0f;

  from.structure_id = 0;
  from.from_m       = 0.0f;
  from.to_m         = 0.0f;
  from.from_c       = 0.0f;
  from.to_c         = 0.0f;
  from.to0          = 0.0f;
  from.to1          = 1.0f;
  from.clamping     = 0;
  from.mapping      = 0;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_PARAMETRIC(INAME_FROM, from);
  READ_STACK_ARG_F32(INAME_T, t);
  READ_STACK_ARGS_END;

  if (from.structure_id != HEAP_STRUCTURE_PARAMETRIC) {
    SEN_ERROR("interp/value requires a user specified 'from' parameter");
    return &g_var_true;
  }

  f32 from_interp = (from.from_m * t) + from.from_c;
  f32 to_interp   = easing(from_interp, from.mapping);
  f32 res         = (from.to_m * to_interp) + from.to_c;

  if (from.clamping == INAME_TRUE) {
    res = from_interp < 0.0f ? from.to0 : (from_interp > 1.0f) ? from.to1 : res;
  }

  f32_as_var(&g_var_scratch, res);

  return &g_var_scratch;
}

sen_var* bind_parametric_cos(sen_vm* vm, i32 num_args) {
  f32 amplitude = 1.0f;
  f32 frequency = 1.0f;
  f32 t         = 1.0f; // t goes from 0 to TAU

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_AMPLITUDE, amplitude);
  READ_STACK_ARG_F32(INAME_FREQUENCY, frequency);
  READ_STACK_ARG_F32(INAME_T, t);
  READ_STACK_ARGS_END;

  f32 value = sen_parametric_cos(amplitude, frequency, t);
  f32_as_var(&g_var_scratch, value);

  // SEN_PRINT("amp %.2f, freq %.2f, t %.2f value = %.2f", amplitude,
  // frequency, t, value);
  return &g_var_scratch;
}

sen_var* bind_parametric_sin(sen_vm* vm, i32 num_args) {
  f32 amplitude = 1.0f;
  f32 frequency = 1.0f;
  f32 t         = 1.0f; // t goes from 0 to TAU

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_AMPLITUDE, amplitude);
  READ_STACK_ARG_F32(INAME_FREQUENCY, frequency);
  READ_STACK_ARG_F32(INAME_T, t);
  READ_STACK_ARGS_END;

  f32 value = sen_parametric_sin(amplitude, frequency, t);
  f32_as_var(&g_var_scratch, value);

  return &g_var_scratch;
}

sen_var* bind_parametric_bezier(sen_vm* vm, i32 num_args) {
  f32 coords[] = {100.0f, 500.0f, 300.0f, 300.0f,
                  600.0f, 700.0f, 900.0f, 500.0f};
  f32 t        = 1.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COORD4(INAME_COORDS, coords);
  READ_STACK_ARG_F32(INAME_T, t);
  READ_STACK_ARGS_END;

  f32 x, y;
  sen_parametric_bezier(&x, &y, coords, t);

  g_var_scratch.type         = VAR_2D;
  g_var_scratch.f32_array[0] = x;
  g_var_scratch.f32_array[1] = y;

  return &g_var_scratch;
}

sen_var* bind_parametric_bezier_tangent(sen_vm* vm, i32 num_args) {
  f32 coords[] = {100.0f, 500.0f, 300.0f, 300.0f,
                  600.0f, 700.0f, 900.0f, 500.0f};
  f32 t        = 1.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COORD4(INAME_COORDS, coords);
  READ_STACK_ARG_F32(INAME_T, t);
  READ_STACK_ARGS_END;

  f32 x, y;
  sen_parametric_bezier_tangent(&x, &y, coords, t);

  g_var_scratch.type         = VAR_2D;
  g_var_scratch.f32_array[0] = x;
  g_var_scratch.f32_array[1] = y;

  return &g_var_scratch;
}

sen_var* bind_parametric_circle(sen_vm* vm, i32 num_args) {
  f32 position[] = {0.0f, 0.0f};
  f32 radius     = 1.0f;
  f32 t          = 0.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VEC2(INAME_POSITION, position);
  READ_STACK_ARG_F32(INAME_RADIUS, radius);
  READ_STACK_ARG_F32(INAME_T, t);
  READ_STACK_ARGS_END;

  f32 x, y;
  sen_parametric_circle(&x, &y, position, radius, t);

  g_var_scratch.type         = VAR_2D;
  g_var_scratch.f32_array[0] = x;
  g_var_scratch.f32_array[1] = y;

  return &g_var_scratch;
}

sen_var* bind_path_linear(sen_vm* vm, i32 num_args) {
  // (path/linear fn: foo steps: 10 from: [0 0] to: [100 100])
  f32 from[]  = {0.0f, 0.0f};
  f32 to[]    = {100.0f, 100.0f};
  f32 steps   = 10.0f;
  f32 t_start = 0.0f;
  f32 t_end   = 1.0f;
  i32 fn      = -1; // todo: rename to fn

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VEC2(INAME_FROM, from);
  READ_STACK_ARG_VEC2(INAME_TO, to);
  READ_STACK_ARG_F32(INAME_STEPS, steps);
  READ_STACK_ARG_F32(INAME_T_START, t_start);
  READ_STACK_ARG_F32(INAME_T_END, t_end);
  READ_STACK_ARG_I32(INAME_FN, fn);
  READ_STACK_ARGS_END;

  if (fn == -1) {
    return &g_var_true;
  }

  path_linear(vm, fn, (i32)steps, t_start, t_end, from[0], from[1], to[0],
              to[1]);

  return &g_var_true;
}

sen_var* bind_path_circle(sen_vm* vm, i32 num_args) {
  f32 pos[]   = {0.0f, 0.0f};
  f32 radius  = 100.0f;
  f32 steps   = 10.0f;
  f32 t_start = 0.0f;
  f32 t_end   = 1.0f;
  i32 fn      = -1; // todo: rename to fn

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VEC2(INAME_POSITION, pos);
  READ_STACK_ARG_F32(INAME_RADIUS, radius);
  READ_STACK_ARG_F32(INAME_STEPS, steps);
  READ_STACK_ARG_F32(INAME_T_START, t_start);
  READ_STACK_ARG_F32(INAME_T_END, t_end);
  READ_STACK_ARG_I32(INAME_FN, fn);
  READ_STACK_ARGS_END;

  if (fn == -1) {
    return &g_var_true;
  }

  path_circle(vm, fn, (i32)steps, t_start, t_end, pos[0], pos[1], radius);

  return &g_var_true;
}

sen_var* bind_path_spline(sen_vm* vm, i32 num_args) {
  f32 coords[] = {100.0f, 500.0f, 300.0f, 300.0f, 600.0f, 700.0f};
  f32 steps    = 10.0f;
  f32 t_start  = 0.0f;
  f32 t_end    = 1.0f;
  i32 fn       = -1; // todo: rename to fn

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COORD3(INAME_COORDS, coords);
  READ_STACK_ARG_F32(INAME_STEPS, steps);
  READ_STACK_ARG_F32(INAME_T_START, t_start);
  READ_STACK_ARG_F32(INAME_T_END, t_end);
  READ_STACK_ARG_I32(INAME_FN, fn);
  READ_STACK_ARGS_END;

  if (fn == -1) {
    return &g_var_true;
  }

  path_spline(vm, fn, (i32)steps, t_start, t_end, coords);

  return &g_var_true;
}

sen_var* bind_path_bezier(sen_vm* vm, i32 num_args) {
  f32 coords[] = {100.0f, 500.0f, 300.0f, 300.0f,
                  600.0f, 700.0f, 900.0f, 500.0f};
  f32 steps    = 10.0f;
  f32 t_start  = 0.0f;
  f32 t_end    = 1.0f;
  i32 fn       = -1; // todo: rename to fn

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COORD4(INAME_COORDS, coords);
  READ_STACK_ARG_F32(INAME_STEPS, steps);
  READ_STACK_ARG_F32(INAME_T_START, t_start);
  READ_STACK_ARG_F32(INAME_T_END, t_end);
  READ_STACK_ARG_I32(INAME_FN, fn);
  READ_STACK_ARGS_END;

  if (fn == -1) {
    return &g_var_true;
  }

  path_bezier(vm, fn, (i32)steps, t_start, t_end, coords);

  return &g_var_true;
}

sen_var* bind_repeat_symmetry_vertical(sen_vm* vm, i32 num_args) {
  // draw is the index into program->fn_info (obtained with address-of)
  i32 fn = -1;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_I32(INAME_FN, fn);
  READ_STACK_ARGS_END;

  if (fn == -1) {
    return &g_var_true;
  }

  i32 copy = 0;
  repeat_symmetry_vertical(vm, fn, &copy);

  return &g_var_true;
}

sen_var* bind_repeat_symmetry_horizontal(sen_vm* vm, i32 num_args) {
  // draw is the index into program->fn_info (obtained with address-of)
  i32 fn = -1;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_I32(INAME_FN, fn);
  READ_STACK_ARGS_END;

  if (fn == -1) {
    return &g_var_true;
  }

  i32 copy = 0;
  repeat_symmetry_horizontal(vm, fn, &copy);

  return &g_var_true;
}

sen_var* bind_repeat_symmetry_4(sen_vm* vm, i32 num_args) {
  // fn is the index into program->fn_info (obtained with address-of)
  i32 fn = -1;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_I32(INAME_FN, fn);
  READ_STACK_ARGS_END;

  if (fn == -1) {
    return &g_var_true;
  }

  i32 copy = 0;
  repeat_symmetry_4(vm, fn, &copy);

  return &g_var_true;
}

sen_var* bind_repeat_symmetry_8(sen_vm* vm, i32 num_args) {
  // fn is the index into program->fn_info (obtained with address-of)
  i32 fn = -1;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_I32(INAME_FN, fn);
  READ_STACK_ARGS_END;

  if (fn == -1) {
    return &g_var_true;
  }

  i32 copy = 0;
  repeat_symmetry_8(vm, fn, &copy);

  return &g_var_true;
}

sen_var* bind_repeat_rotate(sen_vm* vm, i32 num_args) {
  // fn is the index into program->fn_info (obtained with address-of)
  i32 fn     = -1;
  f32 copies = 3.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_I32(INAME_FN, fn);
  READ_STACK_ARG_F32(INAME_COPIES, copies);
  READ_STACK_ARGS_END;

  if (fn == -1) {
    return &g_var_true;
  }

  repeat_rotate(vm, fn, (i32)copies);

  return &g_var_true;
}

sen_var* bind_repeat_rotate_mirrored(sen_vm* vm, i32 num_args) {
  // fn is the index into program->fn_info (obtained with address-of)
  i32 fn     = -1;
  f32 copies = 3.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_I32(INAME_FN, fn);
  READ_STACK_ARG_F32(INAME_COPIES, copies);
  READ_STACK_ARGS_END;

  if (fn == -1) {
    return &g_var_true;
  }

  repeat_rotate_mirrored(vm, fn, (i32)copies);

  return &g_var_true;
}

sen_var* bind_focal_generic(sen_vm* vm, i32 num_args, sen_focal_type type) {
  f32 position[] = {0.0f, 0.0f};
  f32 distance   = 1.0f;
  i32 mapping    = INAME_LINEAR;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VEC2(INAME_POSITION, position);
  READ_STACK_ARG_F32(INAME_DISTANCE, distance);
  READ_STACK_ARG_NAME(INAME_MAPPING,
                      mapping); // linear, quick, slow-in, slow-in-out
  READ_STACK_ARGS_END;

  // store position in canvas space coordinates
  f32 x, y;
  matrix_stack_transform_vec2(&x, &y, vm->matrix_stack, position[0],
                              position[1]);

  // returns vector where:
  // first item contains format in value.i, postion in f32_array[0,1] and
  // distance in f32_array[2] second item contains mapping in value.i

  sen_var* v;

  vector_construct(&g_var_scratch);
  v               = vector_append_i32(vm, &g_var_scratch, HEAP_STRUCTURE_FOCAL);
  v->f32_array[0] = x;
  v->f32_array[1] = y;
  v->f32_array[2] = distance;
  vector_append_i32(vm, &g_var_scratch, type);
  vector_append_i32(vm, &g_var_scratch, mapping);

  return &g_var_scratch;
}

sen_var* bind_focal_build_point(sen_vm* vm, i32 num_args) {
  return bind_focal_generic(vm, num_args, FOCAL_POINT);
}

sen_var* bind_focal_build_vline(sen_vm* vm, i32 num_args) {
  return bind_focal_generic(vm, num_args, FOCAL_VLINE);
}

sen_var* bind_focal_build_hline(sen_vm* vm, i32 num_args) {
  return bind_focal_generic(vm, num_args, FOCAL_HLINE);
}

sen_var* bind_focal_value(sen_vm* vm, i32 num_args) {
  sen_focal_state from;
  f32             position[] = {0.0f, 0.0f};

  from.structure_id = HEAP_STRUCTURE_UNDEFINED;
  from.type         = FOCAL_UNKNOWN;
  from.distance     = 0.0f;
  from.x            = 0.0f;
  from.y            = 0.0f;
  from.mapping      = INAME_LINEAR;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_FOCAL(INAME_FROM, from);
  READ_STACK_ARG_VEC2(INAME_POSITION, position);
  READ_STACK_ARGS_END;

  if (from.structure_id != HEAP_STRUCTURE_FOCAL) {
    SEN_ERROR("focal/value requires a user specified 'from' parameter");
    return &g_var_true;
  }

  f32 res = 0.0f;

  // transform position to canvas space coordinates
  f32 x, y;
  matrix_stack_transform_vec2(&x, &y, vm->matrix_stack, position[0],
                              position[1]);

  switch (from.type) {
  case FOCAL_POINT:
    res = focal_point(x, y, from.distance, from.mapping, from.x, from.y);
    break;
  case FOCAL_HLINE:
    res = focal_hline(y, from.distance, from.mapping, from.y);
    break;
  case FOCAL_VLINE:
    res = focal_vline(x, from.distance, from.mapping, from.x);
    break;
  default:
    // FOCAL_UNKNOWN
    SEN_ERROR("invalid focal structure given to focal/call");
    break;
  }

  f32_as_var(&g_var_scratch, res);

  return &g_var_scratch;
}

sen_var* get_gen_initial(sen_vm* vm) {
  // vm_compiler::register_top_level_preamble : the 2nd global mapping
  i32      gen_initial_global_offset = 1;
  sen_var* initial_value =
      vm_get_from_global_offset(vm, gen_initial_global_offset);

  return initial_value;
}

// NOTE: gen/stray-int should still parse values as
// float as sen scripts won't produce any real ints
//
sen_var* bind_gen_stray_int(sen_vm* vm, i32 num_args) {
  f32 from = 0.0f;
  f32 by   = 1.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_FROM, from);
  READ_STACK_ARG_F32(INAME_BY, by);
  READ_STACK_ARGS_END;

  by = absf(by);

  f32 value = sen_prng_f32_range(vm->prng_state, from - by, from + by);

  f32_as_var(&g_var_scratch, (f32)floor_f32(value));

  return &g_var_scratch;
}

sen_var* bind_gen_stray(sen_vm* vm, i32 num_args) {
  f32 from = 1.0f;
  f32 by   = 0.2f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_FROM, from);
  READ_STACK_ARG_F32(INAME_BY, by);
  READ_STACK_ARGS_END;

  by = absf(by);

  f32 value = sen_prng_f32_range(vm->prng_state, from - by, from + by);

  f32_as_var(&g_var_scratch, value);

  return &g_var_scratch;
}

sen_var* bind_gen_stray_2d(sen_vm* vm, i32 num_args) {
  if (vm->building_with_trait_within_vector != 1) {
    SEN_LOG("bind_gen_stray_2d should always be called with "
            "building_with_trait_within_vector = 1");
  }

  f32 from[] = {10.0f, 10.0f};
  f32 by[]   = {1.0f, 1.0f};

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VEC2(INAME_FROM, from);
  READ_STACK_ARG_VEC2(INAME_BY, by);
  READ_STACK_ARGS_END;

  i32 index = vm->trait_within_vector_index;
  by[index] = absf(by[index]);

  f32 res = sen_prng_f32_range(vm->prng_state, from[index] - by[index],
                               from[index] + by[index]);
  f32_as_var(&g_var_scratch, res);

  return &g_var_scratch;
}

sen_var* bind_gen_stray_3d(sen_vm* vm, i32 num_args) {
  if (vm->building_with_trait_within_vector != 1) {
    SEN_LOG("bind_gen_stray_3d should always be called with "
            "building_with_trait_within_vector = 1");
  }

  f32 from[] = {10.0f, 10.0f, 10.0f};
  f32 by[]   = {1.0f, 1.0f, 1.0f};

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VEC3(INAME_FROM, from);
  READ_STACK_ARG_VEC3(INAME_BY, by);
  READ_STACK_ARGS_END;

  i32 index = vm->trait_within_vector_index;
  by[index] = absf(by[index]);

  f32 res = sen_prng_f32_range(vm->prng_state, from[index] - by[index],
                               from[index] + by[index]);
  f32_as_var(&g_var_scratch, res);

  return &g_var_scratch;
}

sen_var* bind_gen_stray_4d(sen_vm* vm, i32 num_args) {
  if (vm->building_with_trait_within_vector != 1) {
    SEN_LOG("bind_gen_stray_4d should always be called with "
            "building_with_trait_within_vector = 1");
  }

  f32 from[] = {10.0f, 10.0f, 10.0f, 10.0f};
  f32 by[]   = {1.0f, 1.0f, 1.0f, 1.0f};

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VEC4(INAME_FROM, from);
  READ_STACK_ARG_VEC4(INAME_BY, by);
  READ_STACK_ARGS_END;

  i32 index = vm->trait_within_vector_index;
  by[index] = absf(by[index]);

  f32 res = sen_prng_f32_range(vm->prng_state, from[index] - by[index],
                               from[index] + by[index]);
  f32_as_var(&g_var_scratch, res);

  return &g_var_scratch;
}

// NOTE: gen/int should still parse values as float
// as sen scripts won't produce any real ints
//
sen_var* bind_gen_int(sen_vm* vm, i32 num_args) {
  f32 min = 0.0f;
  f32 max = 1000.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_MIN, min);
  READ_STACK_ARG_F32(INAME_MAX, max);
  READ_STACK_ARGS_END;

  // value should be inclusive of both min and max (hence the + 1.0f)
  f32 value = sen_prng_f32_range(vm->prng_state, min, max + 1.0f);

  f32_as_var(&g_var_scratch, (f32)floor_f32(value));

  return &g_var_scratch;
}

sen_var* bind_gen_scalar(sen_vm* vm, i32 num_args) {
  f32 min = 0.0f;
  f32 max = 1.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_MIN, min);
  READ_STACK_ARG_F32(INAME_MAX, max);
  READ_STACK_ARGS_END;

  // pick a scalar between min and max
  f32 value = sen_prng_f32_range(vm->prng_state, min, max);

  f32_as_var(&g_var_scratch, value);

  return &g_var_scratch;
}

sen_var* bind_gen_2d(sen_vm* vm, i32 num_args) {
  f32 min = 0.0f;
  f32 max = 1.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_MIN, min);
  READ_STACK_ARG_F32(INAME_MAX, max);
  READ_STACK_ARGS_END;

  f32 x = sen_prng_f32_range(vm->prng_state, min, max);
  f32 y = sen_prng_f32_range(vm->prng_state, min, max);

  v2_as_var(&g_var_scratch, x, y);

  return &g_var_scratch;
}

sen_var* bind_gen_select(sen_vm* vm, i32 num_args) {
  // 'from' parameter should be a list
  // i.e. (gen/select from: '(1 2 3 4 5))
  //
  // this prevents any confusion between a vector and vec_2d
  // e.g. (gen/select from: [1 2 3 4 5]) vs. (gen/select from: [1 2])

  sen_var* from = NULL;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VAR(INAME_FROM, from);
  READ_STACK_ARGS_END;

  i32 from_length = vector_length(from);
  i32 index       = sen_prng_i32_range(vm->prng_state, 0, from_length - 1);

  sen_var* res = vector_get(from, index);

  var_copy(&g_var_scratch, res);

  return &g_var_scratch;
}

sen_var* bind_gen_col(sen_vm* vm, i32 num_args) {
  f32 alpha = -1.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_ALPHA, alpha);
  READ_STACK_ARGS_END;

  sen_colour colour;

  colour.format     = RGB;
  colour.element[0] = sen_prng_f32_range(vm->prng_state, 0.0f, 1.0f);
  colour.element[1] = sen_prng_f32_range(vm->prng_state, 0.0f, 1.0f);
  colour.element[2] = sen_prng_f32_range(vm->prng_state, 0.0f, 1.0f);

  if (alpha < 0.0f) {
    // no alpha was given so generate a random value
    colour.element[3] = sen_prng_f32_range(vm->prng_state, 0.0f, 1.0f);
  } else {
    colour.element[3] = alpha;
  }

  colour_as_var(&g_var_scratch, &colour);

  return &g_var_scratch;
}

void declare_bindings(sen_word_lut* word_lut, sen_env* e) {
  g_var_true.type    = VAR_BOOLEAN;
  g_var_true.value.i = 1;

  word_lut->keyword_count = 0;

  // this fills out word_lut->keyword and that's used in the wlut_lookup_
  // functions
  //
#define REGISTER_KEYWORD(string, _) declare_vm_keyword(word_lut, string);
#include "keywords.h"
#undef REGISTER_KEYWORD

  declare_native(word_lut, e, "debug/print", &bind_debug_print);
  declare_native(word_lut, e, "nth", &bind_nth);
  declare_native(word_lut, e, "vector/length", &bind_vector_length);

  // map

  declare_native(word_lut, e, "line", &bind_line);
  declare_native(word_lut, e, "rect", &bind_rect);
  declare_native(word_lut, e, "circle", &bind_circle);
  declare_native(word_lut, e, "circle-slice", &bind_circle_slice);
  declare_native(word_lut, e, "poly", &bind_poly);
  declare_native(word_lut, e, "bezier", &bind_bezier);
  declare_native(word_lut, e, "bezier-bulging", &bind_bezier_bulging);
  declare_native(word_lut, e, "stroked-bezier", &bind_stroked_bezier);
  declare_native(word_lut, e, "stroked-bezier-rect", &bind_stroked_bezier_rect);

  declare_native(word_lut, e, "translate", &bind_translate);
  declare_native(word_lut, e, "rotate", &bind_rotate);
  declare_native(word_lut, e, "scale", &bind_scale);

  declare_native(word_lut, e, "col/convert", &bind_col_convert);
  g_colour_constructor_start = word_lut->native_count;
  declare_native(word_lut, e, "col/rgb", &bind_col_rgb);
  declare_native(word_lut, e, "col/hsl", &bind_col_hsl);
  declare_native(word_lut, e, "col/hsluv", &bind_col_hsluv);
  declare_native(word_lut, e, "col/hsv", &bind_col_hsv);
  declare_native(word_lut, e, "col/lab", &bind_col_lab);
  g_colour_constructor_end = word_lut->native_count;
  declare_native(word_lut, e, "col/complementary", &bind_col_complementary);
  declare_native(word_lut, e, "col/split-complementary",
                 &bind_col_split_complementary);
  declare_native(word_lut, e, "col/analagous", &bind_col_analagous);
  declare_native(word_lut, e, "col/triad", &bind_col_triad);
  declare_native(word_lut, e, "col/darken", &bind_col_darken);
  declare_native(word_lut, e, "col/lighten", &bind_col_lighten);

  declare_native(word_lut, e, "col/set-alpha", &bind_col_set_alpha);
  declare_native(word_lut, e, "col/get-alpha", &bind_col_get_alpha);
  declare_native(word_lut, e, "col/set-r", &bind_col_set_r);
  declare_native(word_lut, e, "col/get-r", &bind_col_get_r);
  declare_native(word_lut, e, "col/set-g", &bind_col_set_g);
  declare_native(word_lut, e, "col/get-g", &bind_col_get_g);
  declare_native(word_lut, e, "col/set-b", &bind_col_set_b);
  declare_native(word_lut, e, "col/get-b", &bind_col_get_b);
  declare_native(word_lut, e, "col/set-h", &bind_col_set_h);
  declare_native(word_lut, e, "col/get-h", &bind_col_get_h);
  declare_native(word_lut, e, "col/set-s", &bind_col_set_s);
  declare_native(word_lut, e, "col/get-s", &bind_col_get_s);
  declare_native(word_lut, e, "col/set-l", &bind_col_set_l);
  declare_native(word_lut, e, "col/get-l", &bind_col_get_l);
  declare_native(word_lut, e, "col/set-a", &bind_col_set_a);
  declare_native(word_lut, e, "col/get-a", &bind_col_get_a);
  declare_native(word_lut, e, "col/set-v", &bind_col_set_v);
  declare_native(word_lut, e, "col/get-v", &bind_col_get_v);

  declare_native(word_lut, e, "col/build-procedural",
                 &bind_col_build_procedural);
  declare_native(word_lut, e, "col/build-bezier", &bind_col_build_bezier);
  declare_native(word_lut, e, "col/value", &bind_col_value);

  declare_native(word_lut, e, "math/distance", &bind_math_distance);
  declare_native(word_lut, e, "math/clamp", &bind_math_clamp);
  declare_native(word_lut, e, "math/radians->degrees",
                 &bind_math_radians_to_degrees);
  declare_native(word_lut, e, "math/cos", &bind_math_cos);
  declare_native(word_lut, e, "math/sin", &bind_math_sin);

  declare_native(word_lut, e, "prng/build", &bind_prng_build);
  declare_native(word_lut, e, "prng/values", &bind_prng_values);
  declare_native(word_lut, e, "prng/value", &bind_prng_value);
  declare_native(word_lut, e, "prng/perlin", &bind_prng_perlin);

  declare_native(word_lut, e, "interp/build", &bind_parametric_build);
  declare_native(word_lut, e, "interp/value", &bind_parametric_value);
  declare_native(word_lut, e, "interp/cos", &bind_parametric_cos);
  declare_native(word_lut, e, "interp/sin", &bind_parametric_sin);
  declare_native(word_lut, e, "interp/bezier", &bind_parametric_bezier);
  declare_native(word_lut, e, "interp/bezier-tangent",
                 &bind_parametric_bezier_tangent);
  declare_native(word_lut, e, "interp/circle", &bind_parametric_circle);

  declare_native(word_lut, e, "path/linear", &bind_path_linear);
  declare_native(word_lut, e, "path/circle", &bind_path_circle);
  declare_native(word_lut, e, "path/spline", &bind_path_spline);
  declare_native(word_lut, e, "path/bezier", &bind_path_bezier);

  declare_native(word_lut, e, "repeat/symmetry-vertical",
                 &bind_repeat_symmetry_vertical);
  declare_native(word_lut, e, "repeat/symmetry-horizontal",
                 &bind_repeat_symmetry_horizontal);
  declare_native(word_lut, e, "repeat/symmetry-4", &bind_repeat_symmetry_4);
  declare_native(word_lut, e, "repeat/symmetry-8", &bind_repeat_symmetry_8);
  declare_native(word_lut, e, "repeat/rotate", &bind_repeat_rotate);
  declare_native(word_lut, e, "repeat/rotate-mirrored",
                 &bind_repeat_rotate_mirrored);

  declare_native(word_lut, e, "focal/build-point", &bind_focal_build_point);
  declare_native(word_lut, e, "focal/build-vline", &bind_focal_build_vline);
  declare_native(word_lut, e, "focal/build-hline", &bind_focal_build_hline);
  declare_native(word_lut, e, "focal/value", &bind_focal_value);

  declare_native(word_lut, e, "gen/stray-int", &bind_gen_stray_int);
  declare_native(word_lut, e, "gen/stray", &bind_gen_stray);
  declare_native(word_lut, e, "gen/stray-2d", &bind_gen_stray_2d);
  declare_native(word_lut, e, "gen/stray-3d", &bind_gen_stray_3d);
  declare_native(word_lut, e, "gen/stray-4d", &bind_gen_stray_4d);
  declare_native(word_lut, e, "gen/int", &bind_gen_int);
  declare_native(word_lut, e, "gen/scalar", &bind_gen_scalar);
  declare_native(word_lut, e, "gen/2d", &bind_gen_2d);
  declare_native(word_lut, e, "gen/select", &bind_gen_select); // broken?
  declare_native(word_lut, e, "gen/col", &bind_gen_col);
}
