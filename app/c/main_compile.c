#include "stdio.h"

#include "seni_lang.h"
#include "seni_vm_parser.h"
#include "seni_vm_compiler.h"
#include "seni_bind.h"

#define PRINT_COMPILE(EXPR) seni_word_lut *wl = wlut_allocate();     \
  seni_env *e = env_construct();                                     \
  declare_bindings(wl, e);                                           \
  seni_node *ast = parser_parse(wl, EXPR);                           \
  seni_program *prog = program_allocate(256);                        \
  prog->wl = wl;                                                     \
  prog->env = e;                                                     \
  compiler_compile(ast, prog);                                       \
  seni_vm *vm = vm_construct(STACK_SIZE,HEAP_SIZE);                  \
  printf("%s\n", EXPR);                                              \
  pretty_print_program(prog);

#define CLEANUP wlut_free(wl); \
  parser_free_nodes(ast);      \
  program_free(prog);          \
  env_free(e);                 \
  vm_free(vm)

#define COMPILE(EXPR) {PRINT_COMPILE(EXPR);CLEANUP;}

void test_vm_temp(void)
{
  //  COMPILE("(fn (k) (+ 9 8)) (k)");


  COMPILE("(fn (monkey)    (define foo [1 2 3 4])    (rect position: [200 500]          colour: (col/rgb g: 1)          width: 100          height: 100)    foo)(monkey)");
  
}

int main(void)
{
  if (INAME_NUMBER_OF_KNOWN_WORDS >= NATIVE_START) {
    SENI_LOG("WARNING: keywords are overwriting into NATIVE_START area");
  }

  test_vm_temp();

  return 0;
}
