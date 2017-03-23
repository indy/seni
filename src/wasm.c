#include <emscripten/emscripten.h>
#include "seni.h"

my_struct *users2 = NULL;    /* important! initialize to NULL */

EMSCRIPTEN_KEEPALIVE
f32 mc_m_wasm(f32 xa, f32 ya, f32 xb, f32 yb)
{
  return mc_m(xa, ya, xb, yb);
}

EMSCRIPTEN_KEEPALIVE
int myFunction_wasm(int argc, char ** argv)
{
  int user_id = 42;
  f32 fvalue = 99.88f;
  char* name = "hello";

  my_struct *s;

  s = malloc(sizeof(my_struct));
  s->id = user_id;
  s->ff = fvalue;
  strcpy(s->name, name);
  HASH_ADD_INT( users2, id, s );  /* id: name of key field */

  my_struct *t;
  HASH_FIND_INT( users2, &user_id, t );  /* t: output pointer */
  
  printf("MyFunction Called in seni %d\n", t->id);
  return 1;
}
