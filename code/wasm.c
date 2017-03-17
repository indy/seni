#include <emscripten/emscripten.h>
#include "seni.h"

EMSCRIPTEN_KEEPALIVE
i32 add_wasm(i32 a, i32 b)
{
  return add(a, b);
}

EMSCRIPTEN_KEEPALIVE
int myFunction_wasm(int argc, char ** argv)
{
  printf("MyFunction Called in seni\n");
  return 1;
}
