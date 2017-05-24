#ifndef SENI_BIND_H
#define SENI_BIND_H

#include "seni_lang.h"
#include "seni_vm.h"

// a register like seni_var for holding intermediate values
extern seni_var g_reg;

#define COMMON_ARG(VAR,_) extern i32 VAR;
#include "seni_common_args.h"
#undef COMMON_ARG

void interpreter_declare_keywords(seni_word_lut *wl);
void vm_declare_keywords(seni_word_lut *wl, seni_vm_environment *e);

#endif
