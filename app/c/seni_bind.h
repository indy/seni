#ifndef SENI_BIND_H
#define SENI_BIND_H

#include "seni_lang.h"
#include "seni_vm.h"

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


// a register like seni_var for holding intermediate values
extern seni_var g_reg;

void interpreter_declare_keywords(seni_word_lut *wl);
void vm_declare_keywords(seni_word_lut *wl, seni_vm_environment *e);

#endif
