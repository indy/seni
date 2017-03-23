#include <emscripten/emscripten.h>
#include "seni.h"

EMSCRIPTEN_KEEPALIVE
f32 mc_m_wasm(f32 xa, f32 ya, f32 xb, f32 yb)
{
  return mc_m(xa, ya, xb, yb);
}

EMSCRIPTEN_KEEPALIVE
int myFunction_wasm(int argc, char ** argv)
{
  printf("MyFunction Called in seni\n");
  return 1;
}
