#include <emscripten/emscripten.h>
#include <stdlib.h>
#include "seni_render_packet.h"
#include "seni_bind.h"
#include "seni_uv_mapper.h"
#include "seni_shapes.h"
#include "seni_lang.h"

seni_vm *g_vm = NULL;

EMSCRIPTEN_KEEPALIVE
int get_render_packet_num_vertices(int packet_number)
{
  seni_render_packet *render_packet = get_render_packet(g_vm->render_data, packet_number);
  if (render_packet == NULL) {
    return 0;
  }

  return render_packet->num_vertices;
}

EMSCRIPTEN_KEEPALIVE
f32 *get_render_packet_vbuf(int packet_number)
{
  seni_render_packet *render_packet = get_render_packet(g_vm->render_data, packet_number);
  if (render_packet == NULL) {
    return NULL;
  }

  return render_packet->vbuf;
}

EMSCRIPTEN_KEEPALIVE
f32 *get_render_packet_cbuf(int packet_number)
{
  seni_render_packet *render_packet = get_render_packet(g_vm->render_data, packet_number);
  if (render_packet == NULL) {
    return NULL;
  }

  return render_packet->cbuf;
}

EMSCRIPTEN_KEEPALIVE
f32 *get_render_packet_tbuf(int packet_number)
{
  seni_render_packet *render_packet = get_render_packet(g_vm->render_data, packet_number);
  if (render_packet == NULL) {
    return NULL;
  }

  return render_packet->tbuf;
}

EMSCRIPTEN_KEEPALIVE
int compile_to_render_packets(char *script)
{
  seni_word_lut *wl = NULL;
  seni_env *e = NULL;
  seni_node *ast = NULL;
  seni_program *prog = NULL;

  int max_vertices = 10000;

  // build the global identity matrix used by the shape rendering
  seni_shapes_init_globals();
 
  seni_render_data *render_data = render_data_construct(max_vertices);
  add_render_packet(render_data);

  init_uv_mapper();
  
  e = env_construct();
  
  wl = wlut_allocate();
  declare_bindings(wl, e);
  
  ast = parser_parse(wl, script);
  prog = program_allocate(1024);
  prog->wl = wl;
  prog->env = e;

  if (g_vm != NULL) {
    vm_free(g_vm);
  }
  g_vm = vm_construct(STACK_SIZE, MEMORY_SIZE);
  g_vm->render_data = render_data;


  // compile and evaluate
  compiler_compile(ast, prog);
  vm_interpret(g_vm, prog);

  // cleanup
  free_uv_mapper();
  wlut_free(wl);
  parser_free_nodes(ast);
  program_free(prog);
  env_free(e);
  //  vm_free(vm);

  return render_data->num_render_packets;
}


