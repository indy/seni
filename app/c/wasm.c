#include <emscripten/emscripten.h>
#include <stdlib.h>
#include "seni_render_packet.h"
#include "seni_bind.h"
#include "seni_uv_mapper.h"
#include "seni_shapes.h"
#include "seni_lang.h"

#include <time.h>

seni_vm *g_vm = NULL;
seni_word_lut *g_wl = NULL;
seni_env *g_e = NULL;

// called once at startup
EMSCRIPTEN_KEEPALIVE
void seni_startup()
{
  // build the global identity matrix used by the shape rendering
  seni_shapes_init_globals();
  init_uv_mapper();

  if (g_vm != NULL) {
    vm_free(g_vm);
  }
  g_vm = vm_construct(STACK_SIZE, HEAP_SIZE);

  g_e = env_construct();
  g_wl = wlut_allocate();
  declare_bindings(g_wl, g_e);

}

// called once at shutdown
EMSCRIPTEN_KEEPALIVE
void seni_shutdown()
{
  wlut_free(g_wl);
  env_free(g_e);

  vm_free(g_vm);
  free_uv_mapper();
}

// ------------------------------

EMSCRIPTEN_KEEPALIVE
int compile_to_render_packets(char *script)
{
  seni_node *ast = NULL;
  seni_program *prog = NULL;

  int max_vertices = 10000;
 
  seni_render_data *render_data = render_data_construct(max_vertices);
  add_render_packet(render_data);

  ast = parser_parse(g_wl, script);
  prog = program_allocate(1024);
  prog->wl = g_wl;
  prog->env = g_e;

  vm_free_render_data(g_vm);
  vm_reset(g_vm);
  g_vm->render_data = render_data;

  clock_t start, diff;
  start = clock();

  // compile and evaluate
  compiler_compile(ast, prog);
  bool res = vm_interpret(g_vm, prog);

  if (res) {
    DEBUG_INFO_PRINT(g_vm);
    diff = clock() - start;
    int compile_and_evaluation_time = diff * 1000 / CLOCKS_PER_SEC;
    printf("compile_and_evaluation_time: %d msec\n", compile_and_evaluation_time);
  }

  // cleanup
  wlut_reset_words(g_wl);
  parser_free_nodes(ast);
  program_free(prog);

  return render_data->num_render_packets;
}

// ------------------------------

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

// ------------------------------

// called once by js once it has finished with the render packets and that memory can be free'd
EMSCRIPTEN_KEEPALIVE
void script_cleanup()
{
  vm_free_render_data(g_vm);
}
