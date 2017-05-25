#ifndef SENI_BIND_H
#define SENI_BIND_H

#include "seni_lang.h"
#include "seni_vm.h"

#define READ_STACK_ARGS_BEGIN   i32 args_pointer = vm->sp - (num_args * 2); \
  i32 i_1;                                                              \
  seni_var *label, *value;                                              \
  for(i_1 = 0; i_1 < num_args; i_1++) {                                 \
  label = &(vm->stack[args_pointer + 0]);                               \
  value = &(vm->stack[args_pointer + 1]);                               \
  args_pointer += 2;                                                    \
  i32 name = label->value.i

#define READ_STACK_ARGS_END } vm->sp -= (num_args * 2);

#define READ_STACK_ARG_F32(n) if (name == g_keyword_iname_##n) { n = value->value.f; }

#define WRITE_STACK(v) safe_var_move(&(vm->stack[vm->sp++]), &v)


// a register like seni_var for holding intermediate values
extern seni_var g_reg;

void interpreter_declare_keywords(seni_word_lut *wl);
void vm_declare_keywords(seni_word_lut *wl, seni_vm_environment *e);

#endif
