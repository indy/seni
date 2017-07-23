#include <webassembly.h>
#include <stdlib.h>
#include "seni_js_imports.h"
#include "seni_render_packet.h"
#include "seni_bind.h"
#include "seni_uv_mapper.h"
#include "seni_shapes.h"
#include "seni_lang.h"
#include "seni_vm_parser.h"
#include "seni_vm_compiler.h"
#include "seni_vm_interpreter.h"
#include "seni_printf.h"
#include "seni_timing.h"

#define STRING_BUFFER_SIZE 80000
char *g_string_buffer;

seni_vm *g_vm = NULL;
seni_env *g_e = NULL;

//#define SHOW_WASM_CALLS

// called once at startup
export
void seni_startup()
{
#ifdef SHOW_WASM_CALLS
  SENI_LOG("seni_startup");
#endif
  
  // build the global identity matrix used by the shape rendering
  seni_shapes_init_globals();
  init_uv_mapper();

  if (g_vm != NULL) {
    vm_free(g_vm);
  }
  g_vm = vm_construct(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES);

  g_e = env_construct();
}

// called once at shutdown
export
void seni_shutdown()
{
#ifdef SHOW_WASM_CALLS
  SENI_LOG("seni_shutdown");
#endif
  
  env_free(g_e);

  vm_free(g_vm);
  free_uv_mapper();
}

// ------------------------------

export
int compile_to_render_packets(void)
{
#ifdef SHOW_WASM_CALLS
  SENI_LOG("compile_to_render_packets");
#endif
  
  TIMING_UNIT timing_a = get_timing();

  vm_reset(g_vm);
  
  char *script = g_string_buffer;

  seni_program *prog = program_compile(g_e, MAX_PROGRAM_SIZE, script);

  vm_debug_info_reset(g_vm);
  bool res = vm_interpret(g_vm, prog);

  if (res) {
    vm_debug_info_print(g_vm);
  }

  // cleanup
  wlut_reset_words(g_e->wl);
  //parser_free_nodes(ast);
  program_free(prog);

  f32 delta = timing_delta_from(timing_a);
  SENI_PRINT("total c-side time taken %.2f ms", delta);

  return g_vm->render_data->num_render_packets;
}

// ------------------------------

export
int get_render_packet_num_vertices(int packet_number)
{
#ifdef SHOW_WASM_CALLS
  SENI_LOG("get_render_packet_num_vertices");
#endif
  
  seni_render_packet *render_packet = get_render_packet(g_vm->render_data, packet_number);
  if (render_packet == NULL) {
    return 0;
  }

  return render_packet->num_vertices;
}

export
f32 *get_render_packet_vbuf(int packet_number)
{
#ifdef SHOW_WASM_CALLS
  SENI_LOG("get_render_packet_vbuf");
#endif
  
  seni_render_packet *render_packet = get_render_packet(g_vm->render_data, packet_number);
  if (render_packet == NULL) {
    return NULL;
  }

  return render_packet->vbuf;
}

export
f32 *get_render_packet_cbuf(int packet_number)
{
#ifdef SHOW_WASM_CALLS
  SENI_LOG("get_render_packet_cbuf");
#endif

  seni_render_packet *render_packet = get_render_packet(g_vm->render_data, packet_number);
  if (render_packet == NULL) {
    return NULL;
  }

  return render_packet->cbuf;
}

export
f32 *get_render_packet_tbuf(int packet_number)
{
#ifdef SHOW_WASM_CALLS
  SENI_LOG("get_render_packet_tbuf");
#endif
  
  seni_render_packet *render_packet = get_render_packet(g_vm->render_data, packet_number);
  if (render_packet == NULL) {
    return NULL;
  }

  return render_packet->tbuf;
}

// called once by js once it has finished with the render packets and that memory can be free'd
export
void script_cleanup()
{
#ifdef SHOW_WASM_CALLS
  SENI_LOG("script_cleanup");
#endif

}


export
char *allocate_string_buffer()
{
#ifdef SHOW_WASM_CALLS
  SENI_LOG("allocate_string_buffer");
#endif
  
  g_string_buffer = (char *)malloc(STRING_BUFFER_SIZE * sizeof(char));

  return g_string_buffer;
}

export
void free_string_buffer()
{
#ifdef SHOW_WASM_CALLS
  SENI_LOG("free_string_buffer");
#endif
  
  free(g_string_buffer);
}
