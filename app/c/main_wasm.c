#include <webassembly.h>
#include <stdlib.h>

#include "seni_bind.h"
#include "seni_ga.h"
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
#include "seni_unparser.h"

#define SOURCE_BUFFER_SIZE 80000
char *g_source_buffer;

#define TRAITS_BUFFER_SIZE 8000
char *g_traits_buffer;
seni_text_buffer *g_traits_text_buffer;

#define GENOTYPE_BUFFER_SIZE 8000
bool g_use_genotype_when_compiling;
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

  lang_pools_startup();
  parser_pools_startup();
  ga_pools_startup();
  // build the global identity matrix used by the shape rendering
  seni_shapes_init_globals();
  uv_mapper_init();

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
  g_use_genotype_when_compiling = false;
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

  genotype_list_return_to_pool(g_genotype_list);
  
  env_free(g_e);

  vm_free(g_vm);
  uv_mapper_free();
  ga_pools_shutdown();
  parser_pools_shutdown();
  lang_pools_shutdown();
}

// ------------------------------

export
int compile_to_render_packets(void)
{
#ifdef SHOW_WASM_CALLS
  SENI_LOG("compile_to_render_packets");
#endif
  
  //TIMING_UNIT timing_a = get_timing();


  vm_reset(g_vm);

  seni_program *program = NULL;
  
  if (g_use_genotype_when_compiling) {
    seni_genotype *genotype = genotype_get_from_pool();
    text_buffer_reset(g_genotype_text_buffer);
    genotype_deserialize(genotype, g_genotype_text_buffer);
    program = program_compile_with_genotype(g_e, MAX_PROGRAM_SIZE, g_source_buffer, genotype);
    genotype_return_to_pool(genotype);
  } else {
    program = program_compile(g_e, MAX_PROGRAM_SIZE, g_source_buffer);
  }
  
  vm_debug_info_reset(g_vm);
  bool res = vm_interpret(g_vm, g_e, program);

  if (res) {
    // vm_debug_info_print(g_vm);
  }

  // cleanup
  env_post_interpret_cleanup(g_e);
  program_free(program);

  //f32 delta = timing_delta_from(timing_a);
  //SENI_PRINT("total c-side time taken %.2f ms", delta);

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
  trait_list_return_to_pool(trait_list);
  parser_return_nodes_to_pool(ast);
  
  f32 delta = timing_delta_from(timing_a);
  SENI_PRINT("build_traits: total c-side time taken %.2f ms", delta);
  
  return num_traits;
}

export
i32 create_initial_generation(i32 population_size)
{
  // read in traits and create an array of genotypes
  text_buffer_reset(g_traits_text_buffer);
  seni_trait_list *trait_list = trait_list_get_from_pool();
  bool res = trait_list_deserialize(trait_list, g_traits_text_buffer);
  if (res == false) {
    SENI_ERROR("create_initial_generation: trait_list_deserialize returned false");
    return 0;
  }
  
  if (g_genotype_list != NULL) {
    genotype_list_return_to_pool(g_genotype_list);
  }
  
  g_genotype_list = genotype_list_create_initial_generation(trait_list, population_size);
  if (g_genotype_list == NULL) {
    trait_list_return_to_pool(trait_list);
    SENI_ERROR("create_initial_generation: genotype_list_create_initial_generation returned null");
    return 0;
  }

  i32 count = genotype_list_count(g_genotype_list);
  if (count != population_size) {
    trait_list_return_to_pool(trait_list);
    SENI_ERROR("create_initial_generation: population size mismatch %d requested, %d created",
               population_size, count);
    return 0;
  }

  trait_list_return_to_pool(trait_list);

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

export
void use_genotype_when_compiling(bool use_genotype)
{
  SENI_PRINT("use_genotype_when_compiling is %d", use_genotype);
  g_use_genotype_when_compiling = use_genotype;
}


export
void next_generation_prepare()
{
  if (g_genotype_list != NULL) {
    genotype_list_return_to_pool(g_genotype_list);
  }
  g_genotype_list = genotype_list_get_from_pool();
}

export
void next_generation_add_genotype()
{
  seni_genotype *genotype = genotype_get_from_pool();
  
  text_buffer_reset(g_genotype_text_buffer);
  genotype_deserialize(genotype, g_genotype_text_buffer);

  genotype_list_add_genotype(g_genotype_list, genotype);
}

export
bool next_generation_build(i32 parent_size, i32 population_size, f32 mutation_rate, i32 rng)
{
  // confirm that we have parent_size genotypes in g_genotype_list
  i32 count = genotype_list_count(g_genotype_list);
  if (count != parent_size) {
    SENI_ERROR("next_generation_build: parent_size (%d) mismatch with genotypes given (%d)", parent_size, count);
    return false;
  }

  text_buffer_reset(g_traits_text_buffer);
  seni_trait_list *trait_list = trait_list_get_from_pool();
  bool res = trait_list_deserialize(trait_list, g_traits_text_buffer);
  if (res == false) {
    SENI_ERROR("next_generation_build: trait_list_deserialize returned false");
    return false;
  }

  seni_genotype_list *new_generation = genotype_list_next_generation(g_genotype_list, parent_size, population_size, mutation_rate, rng, trait_list);

  trait_list_return_to_pool(trait_list);
  
  // free the parent genotypes
  genotype_list_return_to_pool(g_genotype_list);

  // assign the new generation
  g_genotype_list = new_generation;

  return true;
}

export
void unparse_with_genotype()
{
  seni_genotype *genotype = genotype_get_from_pool();
  text_buffer_reset(g_genotype_text_buffer);
  genotype_deserialize(genotype, g_genotype_text_buffer);

  seni_vm *vm = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES);
  seni_env *env = env_allocate();
  seni_node *ast = parser_parse(env->wl, g_source_buffer);
  seni_text_buffer *text_buffer = text_buffer_allocate(g_source_buffer, SOURCE_BUFFER_SIZE);

  unparse(text_buffer, env->wl, ast, genotype);

  text_buffer_free(text_buffer);
  parser_return_nodes_to_pool(ast);
  env_free(env);
  vm_free(vm);
  genotype_return_to_pool(genotype);
}
