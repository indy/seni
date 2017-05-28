#include "seni_bind.h"

#include "seni_shapes.h"
#include "seni_buffer.h"
#include "seni_lang.h"

#include <stdio.h>
#include <string.h>
#include <stdlib.h>

// helper macros used by the bind code to parse arguments on the VM's stack
//
#define READ_STACK_ARGS_BEGIN i32 args_pointer_1 = vm->sp - (num_args * 2); \
  i32 i_1;                                                              \
  seni_var *label_1, *value_1, *tmp_1;                                  \
  tmp_1 = NULL;                                                         \
  for(i_1 = 0; i_1 < num_args; i_1++) {                                 \
  label_1 = &(vm->stack[args_pointer_1 + 0]);                           \
  value_1 = &(vm->stack[args_pointer_1 + 1]);                           \
  args_pointer_1 += 2;                                                  \
  i32 name_1 = label_1->value.i

#define READ_STACK_ARGS_END } vm->sp -= (num_args * 2);

#define READ_STACK_ARG_F32(n) if (name_1 == g_keyword_iname_##n) { n = value_1->value.f; }

// traverse through the VAR_VEC_HEAD, VAR_VEC_RC nodes down into the values
// todo: make a seni_var type that can hold VEC2
#define READ_STACK_ARG_VEC2(n) if (name_1 == g_keyword_iname_##n) {    \
    tmp_1 = (value_1->value.v)->value.v;                               \
    n[0] = tmp_1->value.f;                                             \
    n[1] = tmp_1->next->value.f;                                       \
  }

#define READ_STACK_ARG_VEC4(n) if (name_1 == g_keyword_iname_##n) {    \
    tmp_1 = (value_1->value.v)->value.v;                               \
    n[0] = tmp_1->value.f;                                             \
    tmp_1 = tmp_1->next;                                               \
    n[1] = tmp_1->value.f;                                             \
    tmp_1 = tmp_1->next;                                               \
    n[2] = tmp_1->value.f;                                             \
    tmp_1 = tmp_1->next;                                               \
    n[3] = tmp_1->value.f;                                             \
  }

#define WRITE_STACK(v) safe_var_move(&(vm->stack[vm->sp++]), &v)

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

// TEMPORARY
rgba array_to_colour(f32 *colour)
{
  rgba col;
  col.r = colour[0]; col.g = colour[1]; col.b = colour[2]; col.a = colour[3];
  return col;
}

void bind_line(seni_vm *vm, i32 num_args)
{
  // default values for line
  f32 width = 4.0f;
  f32 from[] = {10.0f, 10.0f};
  f32 to[] = {900.0f, 500.0f};
  f32 colour[] = { 0.0f, 1.0f, 0.0f, 1.0f };

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(width);
  READ_STACK_ARG_VEC2(from);
  READ_STACK_ARG_VEC2(to);
  READ_STACK_ARG_VEC4(colour);
  READ_STACK_ARGS_END;

  rgba col = array_to_colour(colour);

  render_line(vm->buffer, from[0], from[1], to[0], to[1], width, col);

  // push the return value onto the stack
  WRITE_STACK(g_var_true);
}

void bind_rect(seni_vm *vm, i32 num_args)
{
  // default values for line
  f32 width = 4.0f;
  f32 height = 10.0f;
  f32 position[] = {10.0f, 23.0f};
  f32 colour[] = { 0.0f, 1.0f, 0.0f, 1.0f };

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(width);
  READ_STACK_ARG_F32(height);
  READ_STACK_ARG_VEC2(position);
  READ_STACK_ARG_VEC4(colour);
  READ_STACK_ARGS_END;

  rgba col = array_to_colour(colour);

  render_rect(vm->buffer, position[0], position[1], width, height, col);

  // push the return value onto the stack
  WRITE_STACK(g_var_true);
}

void bind_circle(seni_vm *vm, i32 num_args)
{
  // default values for line
  f32 width = 4.0f;
  f32 height = 10.0f;
  f32 position[] = {10.0f, 23.0f};
  f32 colour[] = { 0.0f, 1.0f, 0.0f, 1.0f };
  f32 tessellation = 10.0f;
  f32 radius = -1.0f;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(width);
  READ_STACK_ARG_F32(height);
  READ_STACK_ARG_F32(radius);
  READ_STACK_ARG_VEC2(position);
  READ_STACK_ARG_VEC4(colour);
  READ_STACK_ARG_F32(tessellation);
  READ_STACK_ARGS_END;

  // if the radius has been defined then it overrides the width and height parameters
  if (radius > 0.0f) {
    width = radius;
    height = radius;
  }
  
  rgba col = array_to_colour(colour);

  render_circle(vm->buffer, position[0], position[1], width, height, col, (i32)tessellation);

  // push the return value onto the stack
  WRITE_STACK(g_var_true);
}

void bind_col_rgb(seni_vm *vm, i32 num_args)
{
  // (col/rgb r: 0.627 g: 0.627 b: 0.627 alpha: 0.4)
  
  // default values for line
  f32 r = 0.0f;
  f32 g = 0.0f;
  f32 b = 0.0f;
  f32 alpha = 0.0f;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(r);
  READ_STACK_ARG_F32(g);
  READ_STACK_ARG_F32(b);
  READ_STACK_ARG_F32(alpha);
  READ_STACK_ARGS_END;

  seni_var ret;

  construct_vector(vm, &ret);
  
  // append the rgba values to each other
  append_to_vector_f32(vm, &ret, r);
  append_to_vector_f32(vm, &ret, g);
  append_to_vector_f32(vm, &ret, b);
  append_to_vector_f32(vm, &ret, alpha);

  // push the return value onto the stack
  WRITE_STACK(ret);
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
  declare_binding(wlut, e, "bezier", &bind_line);

  declare_binding(wlut, e, "col/rgb", &bind_col_rgb);
}
