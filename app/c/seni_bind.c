#include "seni_bind.h"
#include "seni_bind_core.h"
#include "seni_bind_shapes.h"

// a register like seni_var for holding intermediate values
seni_var g_reg;

#define COMMON_ARG(VAR,_) i32 VAR = 0;
#include "seni_common_args.h"
#undef COMMON_ARG

void interpreter_declare_keywords(word_lut *wlut)
{
#ifdef SENI_DEBUG_MODE
  g_reg.debug_allocatable = false;
#endif

  wlut->keyword_count = 0;

  // common arguments used by keywords and the standard api
  #define COMMON_ARG(VAR,WORD) declare_common_arg(wlut, WORD, &VAR);
  #include "seni_common_args.h"
  #undef COMMON_ARG

  bind_core_declarations(wlut);
  bind_shape_declarations(wlut);
}

void vm_declare_keywords(word_lut *wlut)
{
#ifdef SENI_DEBUG_MODE
  g_reg.debug_allocatable = false;
#endif

  wlut->keyword_count = 0;
  
  // common arguments used by keywords and the standard api
  #define COMMON_ARG(VAR,WORD) declare_common_arg(wlut, WORD, &VAR);
  #include "seni_common_args.h"
  #undef COMMON_ARG

  bind_vm_core_declarations(wlut);
  bind_vm_shape_declarations(wlut);
}
