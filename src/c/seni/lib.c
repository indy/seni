#include "lib.h"

#include "cursor.h"
#include "genetic.h"
#include "lang.h"
#include "parser.h"
#include "shapes.h"
#include "unparser.h"
#include "uv_mapper.h"
#include "vm_compiler.h"

void sen_systems_startup() {
  shapes_subsystem_startup();

  lang_subsystem_startup();
  parser_subsystem_startup();
  ga_subsystem_startup();
  uv_mapper_subsystem_startup();

  compiler_subsystem_startup();
}

void sen_systems_shutdown() {
  compiler_subsystem_shutdown();

  uv_mapper_subsystem_shutdown();
  ga_subsystem_shutdown();
  parser_subsystem_shutdown();
  lang_subsystem_shutdown();
}

sen_vm* sen_allocate_vm(i32 stack_size, i32 heap_size, i32 heap_min_size,
                        i32 vertex_packet_num_vertices) {
  sen_vm* vm = vm_allocate(stack_size, heap_size, heap_min_size,
                           vertex_packet_num_vertices);

  return vm;
}

void sen_free_vm(sen_vm* vm) { vm_free(vm); }

void sen_reset_vm(sen_vm* vm) { vm_reset(vm); }

sen_env* sen_allocate_env() {
  sen_env* env = env_allocate();

  return env;
}

void sen_free_env(sen_env* env) { env_free(env); }

sen_program* sen_compile_program(char* source, sen_word_lut* word_lut,
                                 i32 program_max_size) {
  sen_node* ast = parser_parse(word_lut, source);

  // ast_pretty_print(ast, word_lut);

  sen_compiler_config compiler_config;
  compiler_config.program_max_size = program_max_size;
  compiler_config.word_lut         = word_lut;

  sen_program* program = program_construct(&compiler_config);
  program              = compile_program(program, ast);

  parser_return_nodes_to_pool(ast);

  return program;
}

sen_program* sen_compile_program_with_genotype(char*         source,
                                               sen_genotype* genotype,
                                               sen_word_lut* word_lut,
                                               i32           program_max_size) {
  sen_node* ast = parser_parse(word_lut, source);

  sen_compiler_config compiler_config;
  compiler_config.program_max_size = program_max_size;
  compiler_config.word_lut         = word_lut;

  sen_program* program = program_construct(&compiler_config);

  program = compile_program_with_genotype(program, word_lut, ast, genotype);

  parser_return_nodes_to_pool(ast);

  return program;
}

void sen_unparse_with_genotype(sen_cursor* out_cursor, char* source,
                               sen_genotype* genotype, sen_word_lut* word_lut) {
  sen_node* ast = parser_parse(word_lut, source);

  cursor_reset(out_cursor);

  unparse(out_cursor, word_lut, ast, genotype);

  cursor_write_null(out_cursor);

  parser_return_nodes_to_pool(ast);
}

void sen_simplify_script(sen_cursor* out_cursor, char* source,
                         sen_word_lut* word_lut) {
  sen_node* ast = parser_parse(word_lut, source);

  cursor_reset(out_cursor);

  simplified_unparse(out_cursor, word_lut, ast);

  cursor_write_null(out_cursor);

  parser_return_nodes_to_pool(ast);
}

sen_genotype* sen_deserialize_genotype(sen_cursor* cursor) {
  sen_genotype* genotype = genotype_get_from_pool();
  cursor_reset(cursor);

  bool res = genotype_deserialize(genotype, cursor);
  if (res == false) {
    SEN_ERROR("sen_deserialize_genotype: genotype_deserialize returned false");
    return NULL;
  }

  return genotype;
}

sen_trait_list* sen_compile_trait_list(char* source, sen_word_lut* word_lut) {
  sen_node* ast = parser_parse(word_lut, source);

  sen_compiler_config compiler_config;
  compiler_config.program_max_size = MAX_TRAIT_PROGRAM_SIZE;
  compiler_config.word_lut         = word_lut;

  sen_trait_list* trait_list = trait_list_compile(ast, &compiler_config);

  parser_return_nodes_to_pool(ast);

  return trait_list;
}

bool sen_serialize_trait_list(sen_trait_list* trait_list, sen_cursor* cursor) {
  cursor_reset(cursor);

  bool res = trait_list_serialize(cursor, trait_list);

  if (res == false) {
    SEN_ERROR("sen_serialize_trait_list returned false");
    return false;
  }

  cursor_write_null(cursor);

  return true;
}

sen_trait_list* sen_deserialize_trait_list(sen_cursor* cursor) {
  sen_trait_list* trait_list = trait_list_get_from_pool();
  cursor_reset(cursor);

  bool res = trait_list_deserialize(trait_list, cursor);
  if (res == false) {
    SEN_ERROR(
        "sen_deserialize_trait_list: trait_list_deserialize returned false");
    return NULL;
  }

  return trait_list;
}
