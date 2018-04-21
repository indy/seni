#include "lib.h"

#include "cursor.h"
#include "genetic.h"
#include "lang.h"
#include "parser.h"
#include "shapes.h"
#include "unparser.h"
#include "uv_mapper.h"
#include "vm_compiler.h"

void senie_systems_startup() {
  shapes_subsystem_startup();

  lang_subsystem_startup();
  parser_subsystem_startup();
  ga_subsystem_startup();
  uv_mapper_subsystem_startup();

  compiler_subsystem_startup();
}

void senie_systems_shutdown() {
  compiler_subsystem_shutdown();

  uv_mapper_subsystem_shutdown();
  ga_subsystem_shutdown();
  parser_subsystem_shutdown();
  lang_subsystem_shutdown();
}

senie_vm* senie_allocate_vm(i32 stack_size,
                            i32 heap_size,
                            i32 heap_min_size,
                            i32 vertex_packet_num_vertices) {
  senie_vm* vm = vm_allocate(stack_size, heap_size, heap_min_size, vertex_packet_num_vertices);

  return vm;
}

void senie_free_vm(senie_vm* vm) { vm_free(vm); }

void senie_reset_vm(senie_vm* vm) { vm_reset(vm); }

senie_env* senie_allocate_env() {
  senie_env* env = env_allocate();

  return env;
}

void senie_free_env(senie_env* env) { env_free(env); }

senie_program* senie_compile_program(char* source, senie_word_lut* word_lut, i32 program_max_size) {
  senie_node* ast = parser_parse(word_lut, source);

  // ast_pretty_print(ast, word_lut);

  senie_program* program = compile_program(ast, program_max_size, word_lut);

  parser_return_nodes_to_pool(ast);

  return program;
}

senie_program* senie_compile_program_with_genotype(char*           source,
                                                   senie_genotype* genotype,
                                                   senie_word_lut* word_lut,
                                                   i32             program_max_size) {
  senie_node* ast = parser_parse(word_lut, source);

  senie_program* program = compile_program_with_genotype(ast, program_max_size, word_lut, genotype);

  parser_return_nodes_to_pool(ast);

  return program;
}

void senie_unparse_with_genotype(senie_cursor*   out_cursor,
                                 char*           source,
                                 senie_genotype* genotype,
                                 senie_word_lut* word_lut) {
  senie_node* ast = parser_parse(word_lut, source);

  cursor_reset(out_cursor);

  unparse(out_cursor, word_lut, ast, genotype);

  cursor_write_null(out_cursor);

  parser_return_nodes_to_pool(ast);
}

senie_genotype* senie_deserialize_genotype(senie_cursor* cursor) {
  senie_genotype* genotype = genotype_get_from_pool();
  cursor_reset(cursor);

  bool res = genotype_deserialize(genotype, cursor);
  if (res == false) {
    SENIE_ERROR("senie_deserialize_genotype: genotype_deserialize returned false");
    return NULL;
  }

  return genotype;
}

senie_trait_list* senie_compile_trait_list(char* source, senie_word_lut* word_lut, i32 vary) {
  senie_node*       ast        = parser_parse(word_lut, source);
  senie_trait_list* trait_list = trait_list_compile(ast, MAX_TRAIT_PROGRAM_SIZE, word_lut, vary);

  parser_return_nodes_to_pool(ast);

  return trait_list;
}

bool senie_serialize_trait_list(senie_trait_list* trait_list, senie_cursor* cursor) {
  cursor_reset(cursor);

  bool res = trait_list_serialize(cursor, trait_list);

  if (res == false) {
    SENIE_ERROR("senie_serialize_trait_list returned false");
    return false;
  }

  cursor_write_null(cursor);

  return true;
}

senie_trait_list* senie_deserialize_trait_list(senie_cursor* cursor) {
  senie_trait_list* trait_list = trait_list_get_from_pool();
  cursor_reset(cursor);

  bool res = trait_list_deserialize(trait_list, cursor);
  if (res == false) {
    SENIE_ERROR("senie_deserialize_trait_list: trait_list_deserialize returned false");
    return NULL;
  }

  return trait_list;
}
