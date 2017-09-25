#include "lib.h"

#include "cursor.h"
#include "genetic.h"
#include "lang.h"
#include "parser.h"
#include "shapes.h"
#include "unparser.h"
#include "uv_mapper.h"
#include "vm_compiler.h"

void seni_systems_startup()
{
  shapes_subsystem_startup();

  lang_subsystem_startup();
  parser_subsystem_startup();
  ga_subsystem_startup();
  uv_mapper_subsystem_startup();

  compiler_subsystem_startup();
}

void seni_systems_shutdown()
{
  compiler_subsystem_shutdown();

  uv_mapper_subsystem_shutdown();
  ga_subsystem_shutdown();
  parser_subsystem_shutdown();
  lang_subsystem_shutdown();
}

seni_vm  *seni_allocate_vm(i32 stack_size, i32 heap_size, i32 heap_min_size, i32 vertex_packet_num_vertices)
{
  seni_vm *vm = vm_allocate(stack_size, heap_size, heap_min_size, vertex_packet_num_vertices);

  return vm;
}

void seni_free_vm(seni_vm *vm)
{
  vm_free(vm);
}

void seni_reset_vm(seni_vm *vm)
{
  vm_reset(vm);
}

seni_env *seni_allocate_env()
{
  seni_env *env = env_allocate();

  return env;
}

void seni_free_env(seni_env *env)
{
  env_free(env);
}

seni_program *seni_compile_program(char *source, seni_word_lut *word_lut, i32 program_max_size)
{
  seni_node *ast = parser_parse(word_lut, source);

  seni_program *program = compile_program(ast, program_max_size, word_lut);
  
  parser_return_nodes_to_pool(ast);

  return program;
}

seni_program *seni_compile_program_with_genotype(char *source, seni_genotype *genotype, seni_word_lut *word_lut, i32 program_max_size)
{
  seni_node *ast = parser_parse(word_lut, source);

  seni_program *program = compile_program_with_genotype(ast, program_max_size, word_lut, genotype);
  
  parser_return_nodes_to_pool(ast);

  return program;
}

void seni_unparse_with_genotype(seni_cursor *out_cursor, char *source, seni_genotype *genotype, seni_word_lut *word_lut)
{
  seni_node *ast = parser_parse(word_lut, source);

  cursor_reset(out_cursor);

  unparse(out_cursor, word_lut, ast, genotype);

  parser_return_nodes_to_pool(ast);
}


seni_genotype *seni_deserialize_genotype(seni_cursor *cursor)
{
  seni_genotype *genotype = genotype_get_from_pool();
  cursor_reset(cursor);

  bool res = genotype_deserialize(genotype, cursor);
  if (res == false) {
    SENI_ERROR("seni_deserialize_genotype: genotype_deserialize returned false");
    return NULL;
  }

  return genotype;
}

seni_trait_list *seni_compile_trait_list(char *source, seni_word_lut *word_lut)
{
  seni_node *ast = parser_parse(word_lut, source);
  seni_trait_list *trait_list = trait_list_compile(ast, MAX_TRAIT_PROGRAM_SIZE, word_lut);

  parser_return_nodes_to_pool(ast);

  return trait_list;
}

bool seni_serialize_trait_list(seni_trait_list *trait_list, seni_cursor *cursor)
{
  cursor_reset(cursor);

  bool res = trait_list_serialize(cursor, trait_list);

  if (res == false) {
    SENI_ERROR("seni_serialize_trait_list returned false");
    return false;
  }

  cursor_write_null(cursor);

  return true;
}

seni_trait_list *seni_deserialize_trait_list(seni_cursor *cursor)
{
  seni_trait_list *trait_list = trait_list_get_from_pool();
  cursor_reset(cursor);

  bool res = trait_list_deserialize(trait_list, cursor);
  if (res == false) {
    SENI_ERROR("seni_deserialize_trait_list: trait_list_deserialize returned false");
    return NULL;
  }

  return trait_list;
}
