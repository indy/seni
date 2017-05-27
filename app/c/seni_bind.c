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

void declare_vm_native(seni_word_lut *wlut, char *name, seni_env *e, native_function_ptr function_ptr)
{
  string_copy(&(wlut->native[wlut->native_count]), name);

  e->function_ptr[wlut->native_count] = function_ptr;

  wlut->native_count++;

  if (wlut->native_count > MAX_NATIVE_LOOKUPS) {
    SENI_ERROR("cannot declare native - wlut is full");
  }
}

void native_fn_line(seni_vm *vm, i32 num_args)
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

  seni_var res;
  res.type = VAR_BOOLEAN;
  res.value.i = 1;

  rgba col;
  col.r = colour[0]; col.g = colour[1]; col.b = colour[2]; col.a = colour[3];

  render_line(vm->buffer, from[0], from[1], to[0], to[1], width, col);

  // push the return value onto the stack
  WRITE_STACK(res);
}

void native_fn_rect(seni_vm *vm, i32 num_args)
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

  seni_var res;
  res.type = VAR_BOOLEAN;
  res.value.i = 1;

  rgba col;
  col.r = colour[0]; col.g = colour[1]; col.b = colour[2]; col.a = colour[3];

  render_rect(vm->buffer, position[0], position[1], width, height, col);

  // push the return value onto the stack
  WRITE_STACK(res);
}

void native_fn_circle(seni_vm *vm, i32 num_args)
{
  // default values for line
  f32 width = 4.0f;
  f32 height = 10.0f;
  f32 position[] = {10.0f, 23.0f};
  f32 colour[] = { 0.0f, 1.0f, 0.0f, 1.0f };
  f32 tessellation = 10.0f;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(width);
  READ_STACK_ARG_F32(height);
  READ_STACK_ARG_VEC2(position);
  READ_STACK_ARG_VEC4(colour);
  READ_STACK_ARG_F32(tessellation);
  READ_STACK_ARGS_END;

  seni_var res;
  res.type = VAR_BOOLEAN;
  res.value.i = 1;

  rgba col;
  col.r = colour[0]; col.g = colour[1]; col.b = colour[2]; col.a = colour[3];

  render_circle(vm->buffer, position[0], position[1], width, height, col, (i32)tessellation);

  // push the return value onto the stack
  WRITE_STACK(res);
}

void declare_bindings(seni_word_lut *wlut, seni_env *e)
{
  wlut->keyword_count = 0;

  // this fills out wlut->keyword and that's used in the wlut_lookup_ functions
  //
#define KEYWORD(_,string,__) declare_vm_keyword(wlut, string);
#include "seni_keywords.h"
#undef KEYWORD

  declare_vm_native(wlut, "line", e, &native_fn_line);
  declare_vm_native(wlut, "rect", e, &native_fn_rect);
  declare_vm_native(wlut, "circle", e, &native_fn_circle);
  declare_vm_native(wlut, "bezier", e, &native_fn_line);
}
