#include <webassembly.h>
#include <stdlib.h>

#include "seni_bind.h"
#include "seni_js_imports.h"
#include "seni_lang.h"
#include "seni_parser.h"
#include "seni_printf.h"
#include "seni_render_packet.h"
#include "seni_shapes.h"
#include "seni_timing.h"
#include "seni_uv_mapper.h"
#include "seni_vm_compiler.h"
#include "seni_vm_interpreter.h"

#define SOURCE_BUFFER_SIZE 80000
char *g_source_buffer;

#define TRAITS_BUFFER_SIZE 8000
char *g_traits_buffer;
seni_text_buffer *g_traits_text_buffer;

#define GENOTYPE_BUFFER_SIZE 8000
char *g_genotype_buffer;
seni_text_buffer *g_genotype_text_buffer;
seni_genotype_list *g_genotype_list;

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

  g_vm = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES);
  g_e = env_allocate();

  g_source_buffer = (char *)calloc(SOURCE_BUFFER_SIZE, sizeof(char));

  g_traits_buffer = (char *)calloc(TRAITS_BUFFER_SIZE, sizeof(char));
  g_traits_text_buffer = text_buffer_allocate(g_traits_buffer, TRAITS_BUFFER_SIZE);

  g_genotype_buffer = (char *)calloc(GENOTYPE_BUFFER_SIZE, sizeof(char));
  g_genotype_text_buffer = text_buffer_allocate(g_genotype_buffer, GENOTYPE_BUFFER_SIZE);
  g_genotype_list = NULL;
}

// called once at shutdown
export
void seni_shutdown()
{
#ifdef SHOW_WASM_CALLS
  SENI_LOG("seni_shutdown");
#endif

  text_buffer_free(g_traits_text_buffer);
  
  free(g_source_buffer);
  free(g_traits_buffer);
  free(g_genotype_buffer);

  genotype_list_free(g_genotype_list);
  
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
  
  char *script = g_source_buffer;

  seni_program *prog = program_compile(g_e, MAX_PROGRAM_SIZE, script);

  vm_debug_info_reset(g_vm);
  bool res = vm_interpret(g_vm, g_e, prog);

  if (res) {
    vm_debug_info_print(g_vm);
  }

  // cleanup
  env_post_interpret_cleanup(g_e);
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

// parses the g_source_buffer and serializes the traits to g_traits_buffer
export
i32 build_traits()
{
#ifdef SHOW_WASM_CALLS
  SENI_LOG("build_traits");
#endif

  TIMING_UNIT timing_a = get_timing();

  seni_node *ast = parser_parse(g_e->wl, g_source_buffer);
  seni_trait_list *trait_list = trait_list_compile(ast, MAX_TRAIT_PROGRAM_SIZE, g_e->wl);
  i32 num_traits = trait_list_count(trait_list);

  // g_traits_text_buffer is wrapping g_traits_buffer
  text_buffer_reset(g_traits_text_buffer);

  bool res = trait_list_serialize(g_traits_text_buffer, trait_list);
  if (res == false) {
    SENI_ERROR("trait_list_serialize returned false");
  }
  text_buffer_write_null(g_traits_text_buffer);

  // text_buffer_free(text_buffer);
  trait_list_free(trait_list);
  parser_free_nodes(ast);
  
  f32 delta = timing_delta_from(timing_a);
  SENI_PRINT("build_traits: total c-side time taken %.2f ms", delta);
  
  return num_traits;
}

export
i32 create_foo(i32 population_size)
{
  SENI_PRINT("hello from foo!!");
  return population_size;
}

export
i32 create_initial_generation(i32 population_size)
{
  // read in traits and create an array of genotypes
  text_buffer_reset(g_traits_text_buffer);
  seni_trait_list *trait_list = trait_list_allocate();
  bool res = trait_list_deserialize(trait_list, g_traits_text_buffer);
  if (res == false) {
    SENI_ERROR("create_initial_generation: trait_list_deserialize returned false");
    return 0;
  }
  
  if (g_genotype_list != NULL) {
    genotype_list_free(g_genotype_list);
  }
  
  g_genotype_list = genotype_list_create_initial_generation(trait_list, population_size);
  if (g_genotype_list == NULL) {
    trait_list_free(trait_list);
    SENI_ERROR("create_initial_generation: genotype_list_create_initial_generation returned null");
    return 0;
  }

  i32 count = genotype_list_count(g_genotype_list);
  if (count != population_size) {
    trait_list_free(trait_list);
    SENI_ERROR("create_initial_generation: population size mismatch %d requested, %d created",
               population_size, count);
    return 0;
  }

  trait_list_free(trait_list);

  return population_size;
}

export
void genotype_move_to_buffer(i32 index)
{
  text_buffer_reset(g_genotype_text_buffer);

  seni_genotype *genotype = g_genotype_list->genotypes;
  i32 i = 0;
  while(i != index) {
    genotype = genotype->next;
    i++;
  }
  
  bool res = genotype_serialize(g_genotype_text_buffer, genotype);
  if (res == false) {
    SENI_ERROR("genotype_move_to_buffer: genotype_serialize returned false (for index %d)", index);
  }
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
char *get_source_buffer()
{
#ifdef SHOW_WASM_CALLS
  SENI_LOG("get_source_buffer");
#endif

  return g_source_buffer;
}

export
char *get_traits_buffer()
{
#ifdef SHOW_WASM_CALLS
  SENI_LOG("get_traits_buffer");
#endif
  
  return g_traits_buffer;
}

export
char *get_genotype_buffer()
{
#ifdef SHOW_WASM_CALLS
  SENI_LOG("get_genotype_buffer");
#endif
  
  return g_genotype_buffer;
}

