#include <emscripten/emscripten.h>
#include <stdlib.h>
#include "seni_buffer.h"
#include "seni_bind.h"
#include "seni_uv_mapper.h"

#include "seni_lang.h"

// --------------------------------------------------------------------------------

int max_vertices = 10000;

f32 *g_vbuf = NULL;
f32 *g_cbuf = NULL;
f32 *g_tbuf = NULL;

EMSCRIPTEN_KEEPALIVE
f32 *malloc_vbuf(int size)
{
  f32 *mem = (f32 *)malloc(size);

  return mem;
}

EMSCRIPTEN_KEEPALIVE
int get_max_vertices()
{
  // TODO: IMPLEMENT
  return 42;
}

EMSCRIPTEN_KEEPALIVE
int get_num_render_packets()
{
  // TODO: IMPLEMENT
  return 42;
}

EMSCRIPTEN_KEEPALIVE
int get_render_packet_num_vertices()
{
  // TODO: IMPLEMENT
  return 42;
}

EMSCRIPTEN_KEEPALIVE
f32 *get_render_packet_vbuf(int packet_number)
{
  return g_vbuf;
}

EMSCRIPTEN_KEEPALIVE
f32 *get_render_packet_cbuf(int packet_number)
{
  return g_cbuf;
}

EMSCRIPTEN_KEEPALIVE
f32 *get_render_packet_tbuf(int packet_number)
{
  return g_tbuf;
}

EMSCRIPTEN_KEEPALIVE
void free_all_render_packets()
{
  // TODO: IMPLEMENT
}

// --------------------------------------------------------------------------------





/*
  fill up the seni_buffer with data during the eval phase

  if more buffer is required, allocate 'overflow' buffers on the c side.
  The js will then repeatedly call a 'draining' function that copies data
  into the given vbuf,cbuf,tbuf

  don't forget to free the overflow buffers
*/
// returns the number of vertices to render

EMSCRIPTEN_KEEPALIVE
int compile_to_render_packets(char *script)
{
  seni_word_lut *wl = NULL;
  seni_env *e = NULL;
  seni_node *ast = NULL;
  seni_program *prog = NULL;
  seni_vm *vm = NULL;
  seni_buffer buffer;


  i32 vbuf_element_size = 2;
  g_vbuf = (f32 *)malloc(max_vertices * sizeof(f32) * vbuf_element_size);
  i32 cbuf_element_size = 4;
  g_cbuf = (f32 *)malloc(max_vertices * sizeof(f32) * cbuf_element_size);
  i32 tbuf_element_size = 2;
  g_tbuf = (f32 *)malloc(max_vertices * sizeof(f32) * tbuf_element_size);

  buffer.num_vertices = 0;
  buffer.max_vertices = max_vertices;
  buffer.vbuf = g_vbuf;
  buffer.cbuf = g_cbuf;
  buffer.tbuf = g_tbuf;

  init_uv_mapper();
  
  e = env_construct();
  
  wl = wlut_allocate();
  declare_bindings(wl, e);
  
  ast = parser_parse(wl, script);
  prog = program_allocate(1024);
  prog->wl = wl;
  prog->env = e;

  vm = vm_construct(STACK_SIZE, MEMORY_SIZE);
  vm->buffer = &buffer;


  // compile and evaluate
  compiler_compile(ast, prog);
  vm_interpret(vm, prog);

  // cleanup
  free_uv_mapper();
  wlut_free(wl);
  parser_free_nodes(ast);
  program_free(prog);
  env_free(e);
  vm_free(vm);

  return buffer.num_vertices;
}


