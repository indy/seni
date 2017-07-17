#include "seni_bind.h"
#include "seni_config.h"
#include "seni_shapes.h"
#include "seni_render_packet.h"
#include "seni_lang.h"
#include "seni_vm_interpreter.h"
#include "seni_matrix.h"
#include "seni_mathutil.h"
#include "seni_prng.h"
#include "seni_interp.h"
#include "seni_repeat.h"
#include "seni_path.h"
#include "seni_focal.h"

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

typedef struct {
  i32 interp_fn_id;
  f32 from_m;
  f32 to_m;
  f32 from_c;
  f32 to_c;
  f32 to0;
  f32 to1;
  i32 clamping;
  i32 mapping;
} seni_interp_state;

typedef struct {
  seni_focal_type type;
  f32 x;
  f32 y;
  f32 distance;
  i32 mapping;
} seni_focal_state;

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

#ifdef CHECK_STACK_ARGS
#define IS_F32(n) if (value_1->type != VAR_FLOAT) { SENI_ERROR("expected f32 for: %s", #n); pretty_print_seni_var(value_1, "this is what was received"); }
#define IS_I32(n) if (value_1->type != VAR_INT) { SENI_ERROR("expected i32 for: %s", #n); }
#define IS_COL(n) if (value_1->type != VAR_COLOUR) { SENI_ERROR("expected colour for: %s", #n); }
#define IS_LONG(n) if (value_1->type != VAR_LONG) { SENI_ERROR("expected long for: %s", #n); }
#else
#define IS_F32
#define IS_I32
#define IS_COL
#define IS_LONG
#endif

#define READ_STACK_ARG_F32(k, n) if (name_1 == k) { IS_F32(n); n = value_1->value.f; }
#define READ_STACK_ARG_I32(k, n) if (name_1 == k) { IS_I32(n); n = value_1->value.i; }
#define READ_STACK_ARG_VAR(k, n) if (name_1 == k) { n = value_1; }

#define READ_STACK_ARG_COL(k, n) if (name_1 == k) {                 \
    IS_COL(n);                                                      \
    n->format = value_1->value.i;                                   \
    n->element[0] = value_1->f32_array[0];                          \
    n->element[1] = value_1->f32_array[1];                          \
    n->element[2] = value_1->f32_array[2];                          \
    n->element[3] = value_1->f32_array[3];                          \
}

#define READ_STACK_ARG_VEC2(k, n) if (name_1 == k) {                 \
    n[0] = value_1->f32_array[0];                                    \
    n[1] = value_1->f32_array[1];                                    \
}

#define READ_STACK_ARG_PRNG(k, n) if (name_1 == k) {                    \
    tmp_1 = value_1;                                                    \
    value_1 = (value_1->value.v)->next;                                 \
    IS_LONG(#n);                                                        \
    n.state = value_1->value.l;                                         \
    n.seni_var_state = value_1;                                         \
    value_1 = value_1->next;                                            \
    IS_LONG(#n);                                                        \
    n.inc = value_1->value.l;                                           \
    n.seni_var_inc = value_1;                                           \
    value_1 = value_1->next;                                            \
    IS_F32(#n);                                                         \
    n.min = value_1->value.f;                                           \
    value_1 = value_1->next;                                            \
    IS_F32(#n);                                                         \
    n.max = value_1->value.f;                                           \
    value_1 = tmp_1;                                                    \
  }

#define READ_STACK_ARG_INTERP(k, n) if (name_1 == k) {                  \
    tmp_1 = value_1;                                                    \
    value_1 = (value_1->value.v)->next;                                 \
    IS_I32(#n);                                                         \
    n.interp_fn_id = value_1->value.i;                                  \
    value_1 = value_1->next;                                            \
    IS_F32(#n);                                                         \
    n.from_m = value_1->value.f;                                        \
    value_1 = value_1->next;                                            \
    IS_F32(#n);                                                         \
    n.to_m = value_1->value.f;                                          \
    value_1 = value_1->next;                                            \
    IS_F32(#n);                                                         \
    n.from_c = value_1->value.f;                                        \
    value_1 = value_1->next;                                            \
    IS_F32(#n);                                                         \
    n.to_c = value_1->value.f;                                          \
    value_1 = value_1->next;                                            \
    IS_F32(#n);                                                         \
    n.to0 = value_1->value.f;                                           \
    value_1 = value_1->next;                                            \
    IS_F32(#n);                                                         \
    n.to1 = value_1->value.f;                                           \
    value_1 = value_1->next;                                            \
    IS_I32(#n);                                                         \
    n.clamping = value_1->value.i;                                      \
    value_1 = value_1->next;                                            \
    IS_I32(#n);                                                         \
    n.mapping = value_1->value.i;                                       \
    value_1 = tmp_1;                                                    \
  }

#define READ_STACK_ARG_FOCAL(k, n) if (name_1 == k) { \
    tmp_1 = value_1;                                  \
    value_1 = (value_1->value.v)->next;               \
    IS_I32(#n);                                       \
    n.type = value_1->value.i;                        \
    n.x = value_1->f32_array[0];                      \
    n.y = value_1->f32_array[1];                      \
    n.distance = value_1->f32_array[2];               \
    value_1 = value_1->next;                          \
    IS_I32(#n);                                       \
    n.mapping = value_1->value.i;                     \
    value_1 = tmp_1;                                  \
  }

#define READ_STACK_ARG_COORD3(k, n) if (name_1 == k) {                \
    tmp_1 = (value_1->value.v)->next;                                 \
    n[0] = tmp_1->f32_array[0];                                       \
    n[1] = tmp_1->f32_array[1];                                       \
    tmp_1 = tmp_1->next;                                              \
    n[2] = tmp_1->f32_array[0];                                       \
    n[3] = tmp_1->f32_array[1];                                       \
    tmp_1 = tmp_1->next;                                              \
    n[4] = tmp_1->f32_array[0];                                       \
    n[5] = tmp_1->f32_array[1];                                       \
  }

#define READ_STACK_ARG_COORD4(k, n) if (name_1 == k) {                \
    tmp_1 = (value_1->value.v)->next;                                 \
    n[0] = tmp_1->f32_array[0];                                       \
    n[1] = tmp_1->f32_array[1];                                       \
    tmp_1 = tmp_1->next;                                              \
    n[2] = tmp_1->f32_array[0];                                       \
    n[3] = tmp_1->f32_array[1];                                       \
    tmp_1 = tmp_1->next;                                              \
    n[4] = tmp_1->f32_array[0];                                       \
    n[5] = tmp_1->f32_array[1];                                       \
    tmp_1 = tmp_1->next;                                              \
    n[6] = tmp_1->f32_array[0];                                       \
    n[7] = tmp_1->f32_array[1];                                       \
  }

// a global var that represents true, used as the default
// return type for bindings that only have side-effects
//
seni_var g_var_true;
// temporary seni_var, returned by native functions
seni_var g_var_scratch;

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

void declare_native(seni_word_lut *wlut, seni_env *e, char *name, native_function_ptr function_ptr)
{
  string_copy(&(wlut->native[wlut->native_count]), name);

  e->function_ptr[wlut->native_count] = function_ptr;

  wlut->native_count++;

  if (wlut->native_count > MAX_NATIVE_LOOKUPS) {
    SENI_ERROR("cannot declare native - wlut is full");
  }
}

seni_var *bind_debug_print(seni_vm *vm, i32 num_args)
{
  seni_var *val = NULL;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VAR(INAME_VAL, val);
  READ_STACK_ARGS_END;

  pretty_print_seni_var(val, "debug");

  return &g_var_true;
}

seni_var *bind_nth(seni_vm *vm, i32 num_args)
{
  seni_var *from = NULL;
  f32 n = 0;
  
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VAR(INAME_FROM, from);
  READ_STACK_ARG_F32(INAME_N, n);
  READ_STACK_ARGS_END;

  i32 nth = (i32)n;

  if (from->type == VAR_2D && nth >= 0 && nth < 2) {
    
    f32_as_var(&g_var_scratch, from->f32_array[nth]);
    
  } else if (from->type == VAR_VEC_HEAD){
    
    seni_var *e = from->value.v;

    // e is pointing to the rc, so even a nth of 0 requires one call to e->next
    for (i32 i = 0; i <= nth; i++) {
      e = e->next;
    }

    bool copied = var_copy_onto_junk(vm, &g_var_scratch, e);
    if (copied == false) {
      SENI_ERROR("var_copy_onto_junk failed in bind_nth");
    }
    
  } else {
    SENI_ERROR("nth: neither a var_2d with n 0..2 or vector given\n");
    return &g_var_true;
  }

  return &g_var_scratch;
}

seni_var *bind_line(seni_vm *vm, i32 num_args)
{
  // default values for line
  f32 width = 4.0f;
  f32 from[] = {10.0f, 10.0f};
  f32 to[] = {900.0f, 500.0f};
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_WIDTH, width);
  READ_STACK_ARG_VEC2(INAME_FROM, from);
  READ_STACK_ARG_VEC2(INAME_TO, to);
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  seni_render_data *render_data = vm->render_data;
  seni_matrix *matrix = matrix_stack_peek(vm->matrix_stack);

  render_line(render_data, matrix, from[0], from[1], to[0], to[1], width, colour);


  return &g_var_true;
}

seni_var *bind_rect(seni_vm *vm, i32 num_args)
{
  // default values for rect
  f32 width = 4.0f;
  f32 height = 10.0f;
  f32 position[] = {10.0f, 23.0f};
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_WIDTH, width);
  READ_STACK_ARG_F32(INAME_HEIGHT, height);
  READ_STACK_ARG_VEC2(INAME_POSITION, position);
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  seni_render_data *render_data = vm->render_data;
  seni_matrix *matrix = matrix_stack_peek(vm->matrix_stack);

  render_rect(render_data, matrix, position[0], position[1], width, height, colour);


  return &g_var_true;
}

seni_var *bind_circle(seni_vm *vm, i32 num_args)
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
  READ_STACK_ARG_F32(INAME_WIDTH, width);
  READ_STACK_ARG_F32(INAME_HEIGHT, height);
  READ_STACK_ARG_F32(INAME_RADIUS, radius);
  READ_STACK_ARG_VEC2(INAME_POSITION, position);
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARG_F32(INAME_TESSELLATION, tessellation);
  READ_STACK_ARGS_END;

  // if the radius has been defined then it overrides the width and height parameters
  if (radius > 0.0f) {
    width = radius;
    height = radius;
  }

  seni_render_data *render_data = vm->render_data;
  seni_matrix *matrix = matrix_stack_peek(vm->matrix_stack);

  render_circle(render_data, matrix, position[0], position[1], width, height, colour, (i32)tessellation);


  return &g_var_true;
}

seni_var *bind_bezier(seni_vm *vm, i32 num_args)
{
  // default values for bezier
  f32 line_width = -1.0f;
  f32 line_width_start = 4.0f;
  f32 line_width_end = 4.0f;
  i32 line_width_mapping = INAME_LINEAR;
  f32 coords[] = { 100.0f, 500.0f, 300.0f, 300.0f, 600.0f, 700.0f, 900.0f, 500.0f };
  f32 t_start = -1.0f;
  f32 t_end = 2.0f;
  f32 tessellation = 10.0f;
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;
  i32 brush = INAME_BRUSH_FLAT;
  f32 brush_subtype = 0.0f;
    

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

  seni_render_data *render_data = vm->render_data;
  seni_matrix *matrix = matrix_stack_peek(vm->matrix_stack);

  render_bezier(render_data, matrix,
                coords, line_width_start, line_width_end, line_width_mapping,
                t_start, t_end, colour, (i32)tessellation, brush, (i32)brush_subtype);


  return &g_var_true;
}

seni_var *bind_bezier_bulging(seni_vm *vm, i32 num_args)
{
  // default values for bezier
  f32 line_width = 5.0f;
  // f32 line_width_start = 4.0f;
  // f32 line_width_end = 4.0f;
  // i32 line_width_mapping = INAME_LINEAR;
  f32 coords[] = { 100.0f, 500.0f, 300.0f, 300.0f, 600.0f, 700.0f, 900.0f, 500.0f };
  f32 t_start = -1.0f;
  f32 t_end = 2.0f;
  f32 tessellation = 10.0f;
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;
  i32 brush = INAME_BRUSH_FLAT;
  f32 brush_subtype = 0.0f;

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

  seni_render_data *render_data = vm->render_data;
  seni_matrix *matrix = matrix_stack_peek(vm->matrix_stack);

  render_bezier_bulging(render_data, matrix,
                        coords, line_width, t_start, t_end, colour, (i32)tessellation,
                        brush, (i32)brush_subtype);


  return &g_var_true;
}

seni_var *bind_stroked_bezier(seni_vm *vm, i32 num_args)
{
  // default values for stroked-bezier
  f32 tessellation = 15.0f;
  f32 coords[] = { 100.0f, 500.0f, 300.0f, 300.0f, 600.0f, 700.0f, 900.0f, 500.0f };
  f32 stroke_tessellation = 10.0f;
  f32 stroke_noise = 25;
  f32 stroke_line_width_start = 1.0f;
  f32 stroke_line_width_end = 1.0f;
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;
  f32 colour_volatility = 0.0f;
  f32 seed = 0.0f;

  i32 line_width_mapping = INAME_LINEAR;
  i32 brush = INAME_BRUSH_FLAT;
  f32 brush_subtype = 0.0f;
    

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

  seni_render_data *render_data = vm->render_data;
  seni_matrix *matrix = matrix_stack_peek(vm->matrix_stack);

  render_stroked_bezier(render_data, matrix,
                        coords, colour, (i32)tessellation,
                        stroke_line_width_start, stroke_line_width_end, stroke_noise, (i32)stroke_tessellation,
                        colour_volatility, seed,
                        line_width_mapping, brush, (i32)brush_subtype);


  return &g_var_true;
}

seni_var *bind_stroked_bezier_rect(seni_vm *vm, i32 num_args)
{
  // default values for stroked-bezier-rect
  f32 position[] = {100.0f, 100.0f};
  f32 width = 800.0f;
  f32 height = 600.0f;
  f32 volatility = 30.0f;
  f32 overlap = 0.0f;
  f32 iterations = 10.0f;
  f32 seed = 0.0f;
  f32 tessellation = 15;
  f32 stroke_tessellation = 10.0f;
  f32 stroke_noise = 25;
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;
  f32 colour_volatility = 40.0f;
  i32 brush = INAME_BRUSH_FLAT;
  f32 brush_subtype = 0.0f;

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

  seni_render_data *render_data = vm->render_data;
  seni_matrix *matrix = matrix_stack_peek(vm->matrix_stack);

  render_stroked_bezier_rect(render_data, matrix,
                             position, width, height, volatility, overlap, iterations, seed,
                             (i32)tessellation, (i32)stroke_tessellation, stroke_noise, colour, colour_volatility,
                             brush, (i32)brush_subtype);

  return &g_var_true;
}

seni_var *bind_col_convert(seni_vm *vm, i32 num_args)
{
  // (col/convert colour: col format: LAB)
  
  i32 format = INAME_RGB;
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_I32(INAME_FORMAT, format);
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  // the seni_var referencing the converted colour is going to be added to the VM's stack
  // so we need to get the referenced colour from the vm
  //
  seni_colour out;
  seni_colour_format colour_format = RGB;

  if (format == INAME_RGB) {
    colour_format = RGB;
  } else if (format == INAME_HSL) {
    colour_format = HSL;
  } else if (format == INAME_LAB) {
    colour_format = LAB;
  } else if (format == INAME_HSV) {
    colour_format = HSV;
  }
  
  colour_clone_as(&out, colour, colour_format);

  colour_as_var(&g_var_scratch, &out);
  return &g_var_scratch;
}

seni_var *bind_col_rgb(seni_vm *vm, i32 num_args)
{
  // (col/rgb r: 0.627 g: 0.627 b: 0.627 alpha: 0.4)
  
  // default values for line
  f32 r = 0.0f;                 // 0..1
  f32 g = 0.0f;                 // 0..1
  f32 b = 0.0f;                 // 0..1
  f32 alpha = 1.0f;             // 0..1

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_R, r);
  READ_STACK_ARG_F32(INAME_G, g);
  READ_STACK_ARG_F32(INAME_B, b);
  READ_STACK_ARG_F32(INAME_ALPHA, alpha);
  READ_STACK_ARGS_END;

  seni_colour colour;
  colour.format = RGB;
  colour.element[0] = r;
  colour.element[1] = g;
  colour.element[2] = b;
  colour.element[3] = alpha;


  colour_as_var(&g_var_scratch, &colour);
  return &g_var_scratch;
}

seni_var *bind_col_hsl(seni_vm *vm, i32 num_args)
{
  // (col/hsl h: 180.0 s: 0.1 l: 0.2 alpha: 0.4)
  
  // default values for line
  f32 h = 0.0f;                 // 0..360
  f32 s = 0.0f;                 // 0..1
  f32 l = 0.0f;                 // 0..1
  f32 alpha = 1.0f;             // 0..1

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_H, h);
  READ_STACK_ARG_F32(INAME_S, s);
  READ_STACK_ARG_F32(INAME_L, l);
  READ_STACK_ARG_F32(INAME_ALPHA, alpha);
  READ_STACK_ARGS_END;

  seni_colour colour;
  colour.format = HSL;
  colour.element[0] = h;
  colour.element[1] = s;
  colour.element[2] = l;
  colour.element[3] = alpha;

  colour_as_var(&g_var_scratch, &colour);
  return &g_var_scratch;
}

seni_var *bind_col_hsv(seni_vm *vm, i32 num_args)
{
  // (col/hsv h: 180.0 s: 0.1 v: 0.2 alpha: 0.4)
  
  // default values for line
  f32 h = 0.0f;                 // 0..360
  f32 s = 0.0f;                 // 0..1
  f32 v = 0.0f;                 // 0..1
  f32 alpha = 1.0f;             // 0..1

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_H, h);
  READ_STACK_ARG_F32(INAME_S, s);
  READ_STACK_ARG_F32(INAME_V, v);
  READ_STACK_ARG_F32(INAME_ALPHA, alpha);
  READ_STACK_ARGS_END;

  seni_colour colour;
  colour.format = HSV;
  colour.element[0] = h;
  colour.element[1] = s;
  colour.element[2] = v;
  colour.element[3] = alpha;

  colour_as_var(&g_var_scratch, &colour);
  return &g_var_scratch;
}

seni_var *bind_col_lab(seni_vm *vm, i32 num_args)
{
  // (col/lab l: 0.2 a: -0.1 b: -0.3 alpha: 0.4)
  
  // default values for line
  f32 l = 0.0f;                 // 0..
  f32 a = 0.0f;                 // -1..1
  f32 b = 0.0f;                 // -1..1
  f32 alpha = 1.0f;             // 0..1

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_L, l);
  READ_STACK_ARG_F32(INAME_A, a);
  READ_STACK_ARG_F32(INAME_B, b);
  READ_STACK_ARG_F32(INAME_ALPHA, alpha);
  READ_STACK_ARGS_END;

  seni_colour colour;
  colour.format = LAB;
  colour.element[0] = l;
  colour.element[1] = a;
  colour.element[2] = b;
  colour.element[3] = alpha;

  colour_as_var(&g_var_scratch, &colour);
  return &g_var_scratch;
}

seni_var *bind_col_complementary(seni_vm *vm, i32 num_args)
{
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  seni_colour ret_colour;
  complementary(&ret_colour, colour);

  colour_as_var(&g_var_scratch, &ret_colour);
  return &g_var_scratch;
}

seni_var *bind_col_split_complementary(seni_vm *vm, i32 num_args)
{
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  seni_colour ret_colour0;
  seni_colour ret_colour1;
  split_complementary(&ret_colour0, &ret_colour1, colour);

  // push the return values onto the stack as a vector

  vector_construct(vm, &g_var_scratch);
  append_to_vector_col(vm, &g_var_scratch, &ret_colour0);
  append_to_vector_col(vm, &g_var_scratch, &ret_colour1);

  return &g_var_scratch;
}

seni_var *bind_col_analagous(seni_vm *vm, i32 num_args)
{
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  seni_colour ret_colour0;
  seni_colour ret_colour1;
  analagous(&ret_colour0, &ret_colour1, colour);

  // push the return values onto the stack as a vector
  vector_construct(vm, &g_var_scratch);
  append_to_vector_col(vm, &g_var_scratch, &ret_colour0);
  append_to_vector_col(vm, &g_var_scratch, &ret_colour1);

  return &g_var_scratch;
}

seni_var *bind_col_triad(seni_vm *vm, i32 num_args)
{
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  seni_colour ret_colour0;
  seni_colour ret_colour1;
  triad(&ret_colour0, &ret_colour1, colour);

  // push the return values onto the stack as a vector
  vector_construct(vm, &g_var_scratch);
  append_to_vector_col(vm, &g_var_scratch, &ret_colour0);
  append_to_vector_col(vm, &g_var_scratch, &ret_colour1);

  return &g_var_scratch;
}

seni_var *bind_col_darken(seni_vm *vm, i32 num_args)
{
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;
  f32 value = 0;                // 0..100

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARG_F32(INAME_VALUE, value);
  READ_STACK_ARGS_END;

  seni_colour ret_colour;

  colour_clone_as(&ret_colour, colour, LAB);
  ret_colour.element[0] = clamp(ret_colour.element[0] - value, 0.0f, 100.0f);

  colour_as_var(&g_var_scratch, &ret_colour);
  return &g_var_scratch;
}

seni_var *bind_col_lighten(seni_vm *vm, i32 num_args)
{
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;
  f32 value = 0;                // 0..100

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARG_F32(INAME_VALUE, value);
  READ_STACK_ARGS_END;

  seni_colour ret_colour;

  colour_clone_as(&ret_colour, colour, LAB);
  ret_colour.element[0] = clamp(ret_colour.element[0] + value, 0.0f, 100.0f);

  colour_as_var(&g_var_scratch, &ret_colour);
  return &g_var_scratch;
}

seni_var *bind_col_set_alpha(seni_vm *vm, i32 num_args)
{
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;
  f32 value = 0;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARG_F32(INAME_VALUE, value);
  READ_STACK_ARGS_END;

  seni_colour ret_colour;

  colour_clone_as(&ret_colour, colour, colour->format);
  ret_colour.element[3] = value;

  colour_as_var(&g_var_scratch, &ret_colour);
  return &g_var_scratch;
}

seni_var *bind_col_get_alpha(seni_vm *vm, i32 num_args)
{
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  f32_as_var(&g_var_scratch, colour->element[3]);

  return &g_var_scratch;
}

seni_var *bind_col_set_lab_l(seni_vm *vm, i32 num_args)
{
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;
  f32 value = 0;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARG_F32(INAME_VALUE, value);
  READ_STACK_ARGS_END;

  seni_colour ret_colour;
  colour_clone_as(&ret_colour, colour, LAB);

  i32 l_index = 0; // L is the first element
  ret_colour.element[l_index] = value;

  colour_as_var(&g_var_scratch, &ret_colour);

  return &g_var_scratch;
}

seni_var *bind_col_get_lab_l(seni_vm *vm, i32 num_args)
{
  seni_colour col; colour_set(&col, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
  seni_colour *colour = &col;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COL(INAME_COLOUR, colour);
  READ_STACK_ARGS_END;

  seni_colour lab_colour;
  colour_clone_as(&lab_colour, colour, LAB);

  i32 l_index = 0;

  f32_as_var(&g_var_scratch, colour->element[l_index]);

  return &g_var_scratch;
}

seni_var *bind_translate(seni_vm *vm, i32 num_args)
{
  f32 vector[] = {0.0f, 0.0f};

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VEC2(INAME_VECTOR, vector);
  READ_STACK_ARGS_END;

  matrix_stack_translate(vm->matrix_stack, vector[0], vector[1]);

  return &g_var_true;
}

seni_var *bind_rotate(seni_vm *vm, i32 num_args)
{
  // angle in degrees
  f32 angle = 0.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_ANGLE, angle);
  READ_STACK_ARGS_END;

  matrix_stack_rotate(vm->matrix_stack, deg_to_rad(angle));

  return &g_var_true;
}

seni_var *bind_scale(seni_vm *vm, i32 num_args)
{
  f32 vector[] = {1.0f, 1.0f};
  f32 scalar = 1.0f;

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

seni_var *bind_math_distance(seni_vm *vm, i32 num_args)
{
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

seni_var *bind_math_clamp(seni_vm *vm, i32 num_args)
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
  READ_STACK_ARG_F32(INAME_VAL, val);
  READ_STACK_ARG_F32(INAME_MIN, min);
  READ_STACK_ARG_F32(INAME_MAX, max);
  READ_STACK_ARGS_END;

  f32_as_var(&g_var_scratch, clamp(val, min, max));
 
  return &g_var_scratch;
}

seni_var *bind_math_radians_to_degrees(seni_vm *vm, i32 num_args)
{
  f32 angle = 0.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_ANGLE, angle);
  READ_STACK_ARGS_END;

  f32_as_var(&g_var_scratch, rad_to_deg(angle));

  return &g_var_scratch;
}

// (prng/build seed: 4324 min: 40 max: 100)
seni_var *bind_prng_build(seni_vm *vm, i32 num_args)
{
  f32 seed = 12322.0f;            // todo: in docs mention that seed should be in range 1..some-large-number
  f32 min = 0.0f;
  f32 max = 1.0f;
  
  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_SEED, seed);
  READ_STACK_ARG_F32(INAME_MIN, min);
  READ_STACK_ARG_F32(INAME_MAX, max);
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
  vector_construct(vm, &g_var_scratch);
  append_to_vector_u64(vm, &g_var_scratch, prng_state.state);
  append_to_vector_u64(vm, &g_var_scratch, prng_state.inc);
  append_to_vector_f32(vm, &g_var_scratch, min);
  append_to_vector_f32(vm, &g_var_scratch, max);

  return &g_var_scratch;
}

// (prng/take num: 5 from: rng)
seni_var *bind_prng_take(seni_vm *vm, i32 num_args)
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
  READ_STACK_ARG_F32(INAME_NUM, num);
  READ_STACK_ARG_PRNG(INAME_FROM, from);
  READ_STACK_ARGS_END;

  // build a seni_prng_state from the seni_prng_full_state
  seni_prng_state prng_state;
  prng_state.state = from.state;
  prng_state.inc = from.inc;

  // create the return vector
  f32 value;

  i32 inum = (i32)num;

  vector_construct(vm, &g_var_scratch);
  for (i32 i = 0; i < inum; i++) {
    value = seni_prng_f32_range(&prng_state, from.min, from.max);
    append_to_vector_f32(vm, &g_var_scratch, value);
  }

  // update the state and inc values stored in the vector on the vm's stack
  if (from.seni_var_state != NULL && from.seni_var_inc != NULL) {
    from.seni_var_state->value.l = prng_state.state;
    from.seni_var_inc->value.l = prng_state.inc;
  } else {
    SENI_ERROR("seni_prng_full_state has null pointers ???");
  }

  return &g_var_scratch;
}

// (prng/take-1 from: rng)
seni_var *bind_prng_take_1(seni_vm *vm, i32 num_args)
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
  READ_STACK_ARG_PRNG(INAME_FROM, from);
  READ_STACK_ARGS_END;

  // build a seni_prng_state from the seni_prng_full_state
  seni_prng_state prng_state;
  prng_state.state = from.state;
  prng_state.inc = from.inc;

  f32 value = seni_prng_f32_range(&prng_state, from.min, from.max);
  f32_as_var(&g_var_scratch, value);

  // update the state and inc values stored in the vector on the vm's stack
  if (from.seni_var_state != NULL && from.seni_var_inc != NULL) {
    from.seni_var_state->value.l = prng_state.state;
    from.seni_var_inc->value.l = prng_state.inc;
  } else {
    SENI_ERROR("seni_prng_full_state has null pointers ???");
  }

  return &g_var_scratch;
}

seni_var *bind_prng_perlin(seni_vm *vm, i32 num_args)
{
  f32 x = 1.0f;
  f32 y = 1.0f;
  f32 z = 1.0f;
  
  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_X, x);
  READ_STACK_ARG_F32(INAME_Y, y);
  READ_STACK_ARG_F32(INAME_Z, z);
  READ_STACK_ARGS_END;

  f32 value = seni_perlin(x, y, z);

  f32_as_var(&g_var_scratch, value);

  return &g_var_scratch;
}

seni_var *bind_interp_fn(seni_vm *vm, i32 num_args)
{
  f32 from[] = {0.0f, 1.0f};
  f32 to[] = {0.0f, 100.0f};
  i32 clamping = INAME_FALSE;
  i32 mapping = INAME_LINEAR;
  
  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VEC2(INAME_FROM, from);
  READ_STACK_ARG_VEC2(INAME_TO, to);
  READ_STACK_ARG_I32(INAME_CLAMPING, clamping); // true | FALSE, clamping); // true | false
  READ_STACK_ARG_I32(INAME_MAPPING, mapping);  // linear, quick, slow-in, slow-in-out
  READ_STACK_ARGS_END;

  f32 from_m = mc_m(from[0], 0.0f, from[1], 1.0f);
  f32 from_c = mc_c(from[0], 0.0f, from_m);

  f32 to_m = mc_m(0.0f, to[0], 1.0f, to[1]);
  f32 to_c = mc_c(0.0f, to[0], to_m);

  // id to signify that this structure stores data for interpolation
  // todo: fill this out properly and do the same for the other structures
  i32 interp_fn_id = 42;

  vector_construct(vm, &g_var_scratch);
  append_to_vector_i32(vm, &g_var_scratch, interp_fn_id);
  append_to_vector_f32(vm, &g_var_scratch, from_m);
  append_to_vector_f32(vm, &g_var_scratch, to_m);
  append_to_vector_f32(vm, &g_var_scratch, from_c);
  append_to_vector_f32(vm, &g_var_scratch, to_c);
  append_to_vector_f32(vm, &g_var_scratch, to[0]);
  append_to_vector_f32(vm, &g_var_scratch, to[1]);
  append_to_vector_i32(vm, &g_var_scratch, clamping);
  append_to_vector_i32(vm, &g_var_scratch, mapping);
  
  return &g_var_scratch;
}

seni_var *bind_interp_call(seni_vm *vm, i32 num_args)
{
  seni_interp_state using;
  f32 val = 0.0f;

  using.interp_fn_id = 0;
  using.from_m = 0.0f;
  using.to_m = 0.0f;
  using.from_c = 0.0f;
  using.to_c = 0.0f;
  using.to0 = 0.0f;
  using.to1 = 1.0f;
  using.clamping = 0;
  using.mapping = 0;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_INTERP(INAME_USING, using);
  READ_STACK_ARG_F32(INAME_VAL, val);
  READ_STACK_ARGS_END;

  f32 from_interp = (using.from_m * val) + using.from_c;
  f32 to_interp = from_interp;

  if (using.mapping == INAME_LINEAR) {
    to_interp = from_interp;
  } else if (using.mapping == INAME_QUICK) {
    to_interp = map_quick_ease(from_interp);
  } else if (using.mapping == INAME_SLOW_IN) {
    to_interp = map_slow_ease_in(from_interp);
  } else { // INAME_slow_in_out
    to_interp = map_slow_ease_in_ease_out(from_interp);
  }

  f32 res = (using.to_m * to_interp) + using.to_c;

  if (using.clamping == INAME_TRUE) {
    res = from_interp < 0.0f ? using.to0 : (from_interp > 1.0f) ? using.to1 : res;
  }
  
  f32_as_var(&g_var_scratch, res);

  return &g_var_scratch;
}

seni_var *bind_interp_cos(seni_vm *vm, i32 num_args)
{
  f32 amplitude = 1.0f;
  f32 frequency = 1.0f;
  f32 t = 1.0f;                 // t goes from 0 to TAU

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_AMPLITUDE, amplitude);
  READ_STACK_ARG_F32(INAME_FREQUENCY, frequency);
  READ_STACK_ARG_F32(INAME_T, t);
  READ_STACK_ARGS_END;

  f32 value = seni_interp_cos(amplitude, frequency, t);
  f32_as_var(&g_var_scratch, value);

  return &g_var_scratch;
}

seni_var *bind_interp_sin(seni_vm *vm, i32 num_args)
{
  f32 amplitude = 1.0f;
  f32 frequency = 1.0f;
  f32 t = 1.0f;                 // t goes from 0 to TAU

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(INAME_AMPLITUDE, amplitude);
  READ_STACK_ARG_F32(INAME_FREQUENCY, frequency);
  READ_STACK_ARG_F32(INAME_T, t);
  READ_STACK_ARGS_END;

  f32 value = seni_interp_sin(amplitude, frequency, t);
  f32_as_var(&g_var_scratch, value);

  return &g_var_scratch;
}

seni_var *bind_interp_bezier(seni_vm *vm, i32 num_args)
{
  f32 coords[] = { 100.0f, 500.0f, 300.0f, 300.0f, 600.0f, 700.0f, 900.0f, 500.0f };
  f32 t = 1.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COORD4(INAME_COORDS, coords);
  READ_STACK_ARG_F32(INAME_T, t);
  READ_STACK_ARGS_END;

  f32 x, y;
  seni_interp_bezier(&x, &y, coords, t);

  // push the return values onto the stack as a vector
  vector_construct(vm, &g_var_scratch);
  append_to_vector_f32(vm, &g_var_scratch, x);
  append_to_vector_f32(vm, &g_var_scratch, y);

  return &g_var_scratch;
}

seni_var *bind_interp_bezier_tangent(seni_vm *vm, i32 num_args)
{
  f32 coords[] = { 100.0f, 500.0f, 300.0f, 300.0f, 600.0f, 700.0f, 900.0f, 500.0f };
  f32 t = 1.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_COORD4(INAME_COORDS, coords);
  READ_STACK_ARG_F32(INAME_T, t);
  READ_STACK_ARGS_END;

  f32 x, y;
  seni_interp_bezier_tangent(&x, &y, coords, t);

  // push the return values onto the stack as a vector
  vector_construct(vm, &g_var_scratch);
  append_to_vector_f32(vm, &g_var_scratch, x);
  append_to_vector_f32(vm, &g_var_scratch, y);

  return &g_var_scratch;
}

seni_var *bind_interp_circle(seni_vm *vm, i32 num_args)
{
  f32 position[] = {0.0f, 0.0f};
  f32 radius = 1.0f;
  f32 t = 0.0f;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VEC2(INAME_POSITION, position);
  READ_STACK_ARG_F32(INAME_RADIUS, radius);
  READ_STACK_ARG_F32(INAME_T, t);
  READ_STACK_ARGS_END;

  f32 x, y;
  seni_interp_circle(&x, &y, position, radius, t);

  // push the return values onto the stack as a vector
  vector_construct(vm, &g_var_scratch);
  append_to_vector_f32(vm, &g_var_scratch, x);
  append_to_vector_f32(vm, &g_var_scratch, y);

  return &g_var_scratch;
}

seni_var *bind_path_linear(seni_vm *vm, i32 num_args)
{
  // (path/linear fn: foo steps: 10 from: [0 0] to: [100 100])
  f32 from[] = {0.0f, 0.0f};
  f32 to[] = {100.0f, 100.0f};
  f32 steps = 10.0f;
  f32 t_start = 0.0f;
  f32 t_end = 1.0f;
  i32 fn = -1;                // todo: rename to fn

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

  path_linear(vm, fn, (i32)steps, t_start, t_end, from[0], from[1], to[0], to[1]);

  return &g_var_true;
}

seni_var *bind_path_circle(seni_vm *vm, i32 num_args)
{
  f32 pos[] = {0.0f, 0.0f};
  f32 radius = 100.0f;
  f32 steps = 10.0f;
  f32 t_start = 0.0f;
  f32 t_end = 1.0f;
  i32 fn = -1;                // todo: rename to fn

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

seni_var *bind_path_spline(seni_vm *vm, i32 num_args)
{
  f32 coords[] = { 100.0f, 500.0f, 300.0f, 300.0f, 600.0f, 700.0f };
  f32 steps = 10.0f;
  f32 t_start = 0.0f;
  f32 t_end = 1.0f;
  i32 fn = -1;                // todo: rename to fn

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

seni_var *bind_path_bezier(seni_vm *vm, i32 num_args)
{
  f32 coords[] = { 100.0f, 500.0f, 300.0f, 300.0f, 600.0f, 700.0f, 900.0f, 500.0f };
  f32 steps = 10.0f;
  f32 t_start = 0.0f;
  f32 t_end = 1.0f;
  i32 fn = -1;                // todo: rename to fn

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

seni_var *bind_repeat_symmetry_vertical(seni_vm *vm, i32 num_args)
{
  // draw is the index into program->fn_info (obtained with address-of)
  i32 fn = -1;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_I32(INAME_FN, fn);
  READ_STACK_ARGS_END;

  if (fn == -1) {
    return &g_var_true;
  }

  repeat_symmetry_vertical(vm, fn);

  return &g_var_true;
}

seni_var *bind_repeat_symmetry_horizontal(seni_vm *vm, i32 num_args)
{
  // draw is the index into program->fn_info (obtained with address-of)
  i32 fn = -1;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_I32(INAME_FN, fn);
  READ_STACK_ARGS_END;

  if (fn == -1) {
    return &g_var_true;
  }

  repeat_symmetry_horizontal(vm, fn);

  return &g_var_true;
}

seni_var *bind_repeat_symmetry_4(seni_vm *vm, i32 num_args)
{
  // fn is the index into program->fn_info (obtained with address-of)
  i32 fn = -1;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_I32(INAME_FN, fn);
  READ_STACK_ARGS_END;

  if (fn == -1) {
    return &g_var_true;
  }

  repeat_symmetry_4(vm, fn);

  return &g_var_true;
}

seni_var *bind_repeat_symmetry_8(seni_vm *vm, i32 num_args)
{
  // fn is the index into program->fn_info (obtained with address-of)
  i32 fn = -1;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_I32(INAME_FN, fn);
  READ_STACK_ARGS_END;

  if (fn == -1) {
    return &g_var_true;
  }

  repeat_symmetry_8(vm, fn);

  return &g_var_true;
}

seni_var *bind_repeat_rotate(seni_vm *vm, i32 num_args)
{
  // fn is the index into program->fn_info (obtained with address-of)
  i32 fn = -1;
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

seni_var *bind_repeat_rotate_mirrored(seni_vm *vm, i32 num_args)
{
  // fn is the index into program->fn_info (obtained with address-of)
  i32 fn = -1;
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

seni_var *bind_focal_generic(seni_vm *vm, i32 num_args, seni_focal_type type)
{
  f32 position[] = { 0.0f, 0.0f };
  f32 distance = 1.0f;
  i32 mapping = INAME_LINEAR;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_VEC2(INAME_POSITION, position);
  READ_STACK_ARG_F32(INAME_DISTANCE, distance);
  READ_STACK_ARG_I32(INAME_MAPPING, mapping);  // linear, quick, slow-in, slow-in-out
  READ_STACK_ARGS_END;

  // store position in canvas space coordinates
  f32 x, y;
  matrix_stack_transform_vec2(&x, &y, vm->matrix_stack, position[0], position[1]);

  // returns vector where:
  // first item contains format in value.i, postion in f32_array[0,1] and distance in f32_array[2]
  // second item contains mapping in value.i

  vector_construct(vm, &g_var_scratch);
  seni_var *v = append_to_vector_i32(vm, &g_var_scratch, type);
  v->f32_array[0] = x;
  v->f32_array[1] = y;
  v->f32_array[2] = distance;
  
  append_to_vector_i32(vm, &g_var_scratch, mapping);

  return &g_var_scratch;
}

seni_var *bind_focal_point(seni_vm *vm, i32 num_args)
{
  return bind_focal_generic(vm, num_args, FOCAL_POINT);
}

seni_var *bind_focal_vline(seni_vm *vm, i32 num_args)
{
  return bind_focal_generic(vm, num_args, FOCAL_VLINE);
}

seni_var *bind_focal_hline(seni_vm *vm, i32 num_args)
{
  return bind_focal_generic(vm, num_args, FOCAL_HLINE);
}

seni_var *bind_focal_call(seni_vm *vm, i32 num_args)
{
  seni_focal_state using;
  f32 position[] = { 0.0f, 0.0f };

  using.type = FOCAL_UNKNOWN;
  using.distance = 0.0f;
  using.x = 0.0f;
  using.y = 0.0f;
  using.mapping = INAME_LINEAR;

  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_FOCAL(INAME_USING, using);
  READ_STACK_ARG_VEC2(INAME_POSITION, position);
  READ_STACK_ARGS_END;

  f32 res = 0.0f;

  // transform position to canvas space coordinates
  f32 x, y;
  matrix_stack_transform_vec2(&x, &y, vm->matrix_stack, position[0], position[1]);

  switch(using.type) {
  case FOCAL_POINT:
    res = focal_point(x, y, using.distance, using.mapping, using.x, using.y);
    break;
  case FOCAL_HLINE:
    res = focal_hline(y, using.distance, using.mapping, using.y);
    break;
  case FOCAL_VLINE:
    res = focal_vline(x, using.distance, using.mapping, using.x);
    break;
  default:
    // FOCAL_UNKNOWN
    SENI_ERROR("invalid focal structure given to focal/call");
    break;
  }

  f32_as_var(&g_var_scratch, res);
  
  return &g_var_scratch;
}

void declare_bindings(seni_word_lut *wlut, seni_env *e)
{
  g_var_true.type = VAR_BOOLEAN;
  g_var_true.value.i = 1;
  
  wlut->keyword_count = 0;

  // this fills out wlut->keyword and that's used in the wlut_lookup_ functions
  //
#define REGISTER_KEYWORD(string,_) declare_vm_keyword(wlut, string);
#include "seni_keywords.h"
#undef REGISTER_KEYWORD

  declare_native(wlut, e, "debug/print", &bind_debug_print);
  declare_native(wlut, e, "nth", &bind_nth);
  // map

  declare_native(wlut, e, "line", &bind_line);
  declare_native(wlut, e, "rect", &bind_rect);
  declare_native(wlut, e, "circle", &bind_circle);
  declare_native(wlut, e, "bezier", &bind_bezier);
  declare_native(wlut, e, "bezier-bulging", &bind_bezier_bulging);
  declare_native(wlut, e, "stroked-bezier", &bind_stroked_bezier);
  declare_native(wlut, e, "stroked-bezier-rect", &bind_stroked_bezier_rect);

  declare_native(wlut, e, "translate", &bind_translate);
  declare_native(wlut, e, "rotate", &bind_rotate);
  declare_native(wlut, e, "scale", &bind_scale);

  declare_native(wlut, e, "col/convert", &bind_col_convert);
  declare_native(wlut, e, "col/rgb", &bind_col_rgb);
  declare_native(wlut, e, "col/hsl", &bind_col_hsl);
  declare_native(wlut, e, "col/hsv", &bind_col_hsv);
  declare_native(wlut, e, "col/lab", &bind_col_lab);
  declare_native(wlut, e, "col/complementary", &bind_col_complementary);
  declare_native(wlut, e, "col/split-complementary", &bind_col_split_complementary);
  declare_native(wlut, e, "col/analagous", &bind_col_analagous);
  declare_native(wlut, e, "col/triad", &bind_col_triad);
  declare_native(wlut, e, "col/darken", &bind_col_darken);
  declare_native(wlut, e, "col/lighten", &bind_col_lighten);
  declare_native(wlut, e, "col/set-alpha", &bind_col_set_alpha);
  declare_native(wlut, e, "col/get-alpha", &bind_col_get_alpha);
  declare_native(wlut, e, "col/set-lab-l", &bind_col_set_lab_l);
  declare_native(wlut, e, "col/get-lab-l", &bind_col_get_lab_l);

  // col/procedural-fn-presets
  // col/procedural-fn
  // col/bezier-fn
  // col/quadratic-fn

  declare_native(wlut, e, "math/distance", &bind_math_distance);
  declare_native(wlut, e, "math/clamp", &bind_math_clamp);
  declare_native(wlut, e, "math/radians->degrees", &bind_math_radians_to_degrees);

  declare_native(wlut, e, "prng/build", &bind_prng_build);
  declare_native(wlut, e, "prng/take", &bind_prng_take);
  declare_native(wlut, e, "prng/take-1", &bind_prng_take_1);
  declare_native(wlut, e, "prng/perlin", &bind_prng_perlin);

  declare_native(wlut, e, "interp/fn", &bind_interp_fn);
  declare_native(wlut, e, "interp/call", &bind_interp_call);
  declare_native(wlut, e, "interp/cos", &bind_interp_cos);
  declare_native(wlut, e, "interp/sin", &bind_interp_sin);
  declare_native(wlut, e, "interp/bezier", &bind_interp_bezier);
  declare_native(wlut, e, "interp/bezier-tangent", &bind_interp_bezier_tangent);
  declare_native(wlut, e, "interp/circle", &bind_interp_circle);

  declare_native(wlut, e, "path/linear", &bind_path_linear);
  declare_native(wlut, e, "path/circle", &bind_path_circle);
  declare_native(wlut, e, "path/spline", &bind_path_spline);
  declare_native(wlut, e, "path/bezier", &bind_path_bezier);

  declare_native(wlut, e, "repeat/symmetry-vertical", &bind_repeat_symmetry_vertical);
  declare_native(wlut, e, "repeat/symmetry-horizontal", &bind_repeat_symmetry_horizontal);
  declare_native(wlut, e, "repeat/symmetry-4", &bind_repeat_symmetry_4);
  declare_native(wlut, e, "repeat/symmetry-8", &bind_repeat_symmetry_8);
  declare_native(wlut, e, "repeat/rotate", &bind_repeat_rotate);
  declare_native(wlut, e, "repeat/rotate_mirrored", &bind_repeat_rotate_mirrored);

  declare_native(wlut, e, "focal/call", &bind_focal_call);
  declare_native(wlut, e, "focal/point", &bind_focal_point);
  declare_native(wlut, e, "focal/vline", &bind_focal_vline);
  declare_native(wlut, e, "focal/hline", &bind_focal_hline);
}

