#include <emscripten/emscripten.h>
#include "seni.h"
#include "seni_buffer.h"
#include "seni_bind.h"
#include "seni_uv_mapper.h"

f32 mult = 3.2f;
int lensub = 0;

EMSCRIPTEN_KEEPALIVE
int buffer_fill(f32* array, int length, char *script)
{
  printf("the script is %s\n", script);
  
  printf("array length is %d\n", length);

  for (int i=0; i<length; i++) {
    printf("array[%d] = %.2f\n", i, array[i]);
    array[i] = (f32)i * mult;
  }

  int retlength = length - lensub;
  lensub++;
  mult += 1.2f;
  
  return retlength;
}

/*
  fill up the seni_buffer with data during the eval phase

  if more buffer is required, allocate 'overflow' buffers on the c side.
  The js will then repeatedly call a 'draining' function that copies data
  into the given vbuf,cbuf,tbuf

  don't forget to free the overflow buffers
*/
// returns the number of vertices to render
EMSCRIPTEN_KEEPALIVE
int render(f32* vbuf, f32* cbuf, f32* tbuf, int max_vertices, char *script)
{
  word_lut *wl = NULL;
  seni_env *env = NULL;
  seni_node *ast = NULL;
  seni_var *var = NULL;
    
  seni_buffer buffer;

  buffer.num_vertices = 0;
  buffer.max_vertices = max_vertices;
  buffer.vbuf = vbuf;
  buffer.cbuf = cbuf;
  buffer.tbuf = tbuf;
  
  debug_reset();

  wl = wlut_allocate();
  interpreter_declare_keywords(wl);

  init_uv_mapper();

  env_allocate_pools();
  env = get_initial_env(&buffer);
  
  ast = parser_parse(wl, script);
  var = evaluate(env, ast, true);
  debug_var_info(env);

  env_free_pools();
  free_uv_mapper();
  wlut_free(wl);
  parser_free_nodes(ast);

  return buffer.num_vertices;
}
