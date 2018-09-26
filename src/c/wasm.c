#include <stdlib.h>
#include <webassembly.h>

#include "sen/cursor.h"
#include "sen/genetic.h"
#include "sen/js_imports.h"
#include "sen/lang.h"
#include "sen/lib.h"
#include "sen/parser.h"
#include "sen/printf.h"
#include "sen/render_packet.h"
#include "sen/timing.h"
#include "sen/unparser.h"
#include "sen/vm_compiler.h"
#include "sen/vm_interpreter.h"

#define SOURCE_BUFFER_SIZE 20000
char* g_source_buffer;

char*       g_out_source_buffer;
sen_cursor* g_out_source_cursor;

#define TRAITS_BUFFER_SIZE 40000
char*       g_traits_buffer;
sen_cursor* g_traits_cursor;

#define GENOTYPE_BUFFER_SIZE 5000
bool               g_use_genotype_when_compiling;
char*              g_genotype_buffer;
sen_cursor*        g_genotype_cursor;
sen_genotype_list* g_genotype_list;

sen_vm*  g_vm = NULL;
sen_env* g_e  = NULL;

// #define SHOW_WASM_CALLS

void debug_size_source_buffer() {
#ifdef SHOW_WASM_CALLS
  size_t len = strlen(g_source_buffer);
  SEN_LOG("g_source_buffer size %d", len);
#endif
}

void debug_size_out_source_buffer() {
#ifdef SHOW_WASM_CALLS
  size_t len = strlen(g_out_source_buffer);
  SEN_LOG("g_out_source_buffer size %d", len);
#endif
}

void debug_size_traits_buffer() {
#ifdef SHOW_WASM_CALLS
  size_t len = strlen(g_traits_buffer);
  SEN_LOG("g_traits_buffer size %d", len);
#endif
}

void debug_size_genotype_buffer() {
#ifdef SHOW_WASM_CALLS
  size_t len = strlen(g_genotype_buffer);
  SEN_LOG("g_genotye_buffer size %d", len);
#endif
}

// called once at startup
export void sen_startup() {
#ifdef SHOW_WASM_CALLS
  SEN_LOG("sen_startup");
#endif

  sen_systems_startup();

  if (g_vm != NULL) {
    sen_free_vm(g_vm);
  }

  g_vm = sen_allocate_vm(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE,
                         VERTEX_PACKET_NUM_VERTICES);
  g_e  = sen_allocate_env();

  g_source_buffer = (char*)calloc(SOURCE_BUFFER_SIZE, sizeof(char));

  g_out_source_buffer = (char*)calloc(SOURCE_BUFFER_SIZE, sizeof(char));
  g_out_source_cursor =
      cursor_allocate(g_out_source_buffer, SOURCE_BUFFER_SIZE);

  g_traits_buffer = (char*)calloc(TRAITS_BUFFER_SIZE, sizeof(char));
  g_traits_cursor = cursor_allocate(g_traits_buffer, TRAITS_BUFFER_SIZE);

  g_genotype_buffer = (char*)calloc(GENOTYPE_BUFFER_SIZE, sizeof(char));
  g_genotype_cursor = cursor_allocate(g_genotype_buffer, GENOTYPE_BUFFER_SIZE);
  g_genotype_list   = NULL;
  g_use_genotype_when_compiling = false;
}

// called once at shutdown
export void sen_shutdown() {
#ifdef SHOW_WASM_CALLS
  SEN_LOG("sen_shutdown");
#endif

  cursor_free(g_out_source_cursor);
  cursor_free(g_traits_cursor);

  free(g_source_buffer);
  free(g_out_source_buffer);
  free(g_traits_buffer);
  free(g_genotype_buffer);

  genotype_list_return_to_pool(g_genotype_list);

  sen_free_env(g_e);
  sen_free_vm(g_vm);

  sen_systems_shutdown();
}

// ------------------------------

export int compile_to_render_packets(void) {
#ifdef SHOW_WASM_CALLS
  SEN_LOG("compile_to_render_packets");
#endif

  // TIMING_UNIT timing_a = get_timing();

  debug_size_source_buffer();
  debug_size_genotype_buffer();

  sen_reset_vm(g_vm);

  sen_program* program = NULL;

  if (g_use_genotype_when_compiling) {
    sen_genotype* genotype = sen_deserialize_genotype(g_genotype_cursor);
    program                = sen_compile_program_with_genotype(
        g_source_buffer, genotype, g_e->word_lut, MAX_PROGRAM_SIZE);
    genotype_return_to_pool(genotype);
  } else {
    program =
        sen_compile_program(g_source_buffer, g_e->word_lut, MAX_PROGRAM_SIZE);
  }

  vm_debug_info_reset(g_vm);
  bool res = vm_run(g_vm, g_e, program);

  if (res) {
    // vm_debug_info_print(g_vm);
  }

  // cleanup
  wlut_reset_words(g_e->word_lut);
  program_free(program);

  // f32 delta = timing_delta_from(timing_a);
  // SEN_PRINT("total c-side time taken %.2f ms", delta);

  return g_vm->render_data->num_render_packets;
}

// ------------------------------

export int get_render_packet_num_vertices(int packet_number) {
#ifdef SHOW_WASM_CALLS
  SEN_LOG("get_render_packet_num_vertices");
#endif

  sen_render_packet* render_packet =
      get_render_packet(g_vm->render_data, packet_number);
  if (render_packet == NULL) {
    return 0;
  }

  return render_packet->num_vertices;
}

export f32* get_render_packet_vbuf(int packet_number) {
#ifdef SHOW_WASM_CALLS
  SEN_LOG("get_render_packet_vbuf");
#endif

  sen_render_packet* render_packet =
      get_render_packet(g_vm->render_data, packet_number);
  if (render_packet == NULL) {
    return NULL;
  }

  return render_packet->vbuf;
}

export f32* get_render_packet_cbuf(int packet_number) {
#ifdef SHOW_WASM_CALLS
  SEN_LOG("get_render_packet_cbuf");
#endif

  sen_render_packet* render_packet =
      get_render_packet(g_vm->render_data, packet_number);
  if (render_packet == NULL) {
    return NULL;
  }

  return render_packet->cbuf;
}

export f32* get_render_packet_tbuf(int packet_number) {
#ifdef SHOW_WASM_CALLS
  SEN_LOG("get_render_packet_tbuf");
#endif

  sen_render_packet* render_packet =
      get_render_packet(g_vm->render_data, packet_number);
  if (render_packet == NULL) {
    return NULL;
  }

  return render_packet->tbuf;
}

// parses the g_source_buffer and serializes the traits to g_traits_buffer
export i32 build_traits() {
#ifdef SHOW_WASM_CALLS
  SEN_LOG("build_traits");
#endif

  debug_size_source_buffer();

  // TIMING_UNIT timing_a = get_timing();

  sen_trait_list* trait_list =
      sen_compile_trait_list(g_source_buffer, g_e->word_lut);
  bool res = sen_serialize_trait_list(trait_list, g_traits_cursor);
  if (res == false) {
    SEN_ERROR("sen_serialize_trait_list returned false");
    return 0;
  }

  i32 num_traits = trait_list_count(trait_list);

  trait_list_return_to_pool(trait_list);

  // f32 delta = timing_delta_from(timing_a);
  // SEN_PRINT("build_traits: total c-side time taken %.2f ms", delta);

  debug_size_traits_buffer();

  return num_traits;
}

export i32 single_genotype_from_seed(i32 seed) {
  debug_size_traits_buffer();

  sen_trait_list* trait_list = sen_deserialize_trait_list(g_traits_cursor);

  if (g_genotype_list != NULL) {
    genotype_list_return_to_pool(g_genotype_list);
  }

  g_genotype_list = genotype_list_create_single_genotype(trait_list, seed);
  if (g_genotype_list == NULL) {
    trait_list_return_to_pool(trait_list);
    SEN_ERROR("create_initial_generation: "
              "genotype_list_create_initial_generation returned null");
    return 0;
  }

  trait_list_return_to_pool(trait_list);

  return 1;
}

export i32 create_initial_generation(i32 population_size, i32 seed) {
  debug_size_traits_buffer();

  // read in traits and create an array of genotypes
  sen_trait_list* trait_list = sen_deserialize_trait_list(g_traits_cursor);

  if (g_genotype_list != NULL) {
    genotype_list_return_to_pool(g_genotype_list);
  }

  g_genotype_list = genotype_list_create_initial_generation(
      trait_list, population_size, seed);
  if (g_genotype_list == NULL) {
    trait_list_return_to_pool(trait_list);
    SEN_ERROR("create_initial_generation: "
              "genotype_list_create_initial_generation returned null");
    return 0;
  }

  i32 count = genotype_list_count(g_genotype_list);
  if (count != population_size) {
    trait_list_return_to_pool(trait_list);
    SEN_ERROR("create_initial_generation: population size mismatch %d "
              "requested, %d created",
              population_size, count);
    return 0;
  }

  trait_list_return_to_pool(trait_list);

  return population_size;
}

export void genotype_move_to_buffer(i32 index) {
  cursor_reset(g_genotype_cursor);

  sen_genotype* genotype = g_genotype_list->genotypes;
  i32           i        = 0;
  while (i != index) {
    genotype = genotype->next;
    i++;
  }

  bool res = genotype_serialize(g_genotype_cursor, genotype);
  if (res == false) {
    SEN_ERROR("genotype_move_to_buffer: genotype_serialize returned false "
              "(for index %d)",
              index);
  }

  cursor_write_null(g_genotype_cursor);

  debug_size_genotype_buffer();
}

// called once by js once it has finished with the render packets and that
// memory can be free'd
export void script_cleanup() {
#ifdef SHOW_WASM_CALLS
  SEN_LOG("script_cleanup");
#endif
}

export char* get_source_buffer() {
#ifdef SHOW_WASM_CALLS
  SEN_LOG("get_source_buffer");
#endif

  return g_source_buffer;
}

export char* get_out_source_buffer() {
#ifdef SHOW_WASM_CALLS
  SEN_LOG("get_out_source_buffer");
#endif

  return g_out_source_buffer;
}

export char* get_traits_buffer() {
#ifdef SHOW_WASM_CALLS
  SEN_LOG("get_traits_buffer");
#endif

  return g_traits_buffer;
}

export char* get_genotype_buffer() {
#ifdef SHOW_WASM_CALLS
  SEN_LOG("get_genotype_buffer");
#endif

  return g_genotype_buffer;
}

export void use_genotype_when_compiling(bool use_genotype) {
#ifdef SHOW_WASM_CALLS
  SEN_LOG("use_genotype_when_compiling is %d", use_genotype);
#endif

  g_use_genotype_when_compiling = use_genotype;
}

export void next_generation_prepare() {
  if (g_genotype_list != NULL) {
    genotype_list_return_to_pool(g_genotype_list);
  }
  g_genotype_list = genotype_list_get_from_pool();
}

export void next_generation_add_genotype() {
  debug_size_genotype_buffer();

  sen_genotype* genotype = sen_deserialize_genotype(g_genotype_cursor);

  genotype_list_add_genotype(g_genotype_list, genotype);
}

export bool next_generation_build(i32 parent_size, i32 population_size,
                                  f32 mutation_rate, i32 rng) {

  debug_size_traits_buffer();

  // confirm that we have parent_size genotypes in g_genotype_list
  i32 count = genotype_list_count(g_genotype_list);
  if (count != parent_size) {
    SEN_ERROR("next_generation_build: parent_size (%d) mismatch with "
              "genotypes given (%d)",
              parent_size, count);
    return false;
  }

  sen_trait_list* trait_list = sen_deserialize_trait_list(g_traits_cursor);

  sen_genotype_list* new_generation = genotype_list_next_generation(
      g_genotype_list, parent_size, population_size, mutation_rate, rng,
      trait_list);

  trait_list_return_to_pool(trait_list);

  // free the parent genotypes
  genotype_list_return_to_pool(g_genotype_list);

  // assign the new generation
  g_genotype_list = new_generation;

  return true;
}

export void unparse_with_genotype() {
  debug_size_genotype_buffer();
  debug_size_source_buffer();

  sen_genotype* genotype = sen_deserialize_genotype(g_genotype_cursor);

  sen_unparse_with_genotype(g_out_source_cursor, g_source_buffer, genotype,
                            g_e->word_lut);

  genotype_return_to_pool(genotype);

  debug_size_out_source_buffer();
}

export void simplify_script() {
  debug_size_source_buffer();

  sen_simplify_script(g_out_source_cursor, g_source_buffer, g_e->word_lut);

  debug_size_out_source_buffer();
}
