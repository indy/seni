#pragma once

#include "config.h"
#include "types.h"

void seni_systems_startup();
void seni_systems_shutdown();

seni_vm  *seni_allocate_vm(i32 stack_size, i32 heap_size, i32 heap_min_size, i32 vertex_packet_num_vertices);
void seni_free_vm(seni_vm *vm);
void seni_reset_vm(seni_vm *vm);

seni_env *seni_allocate_env();
void seni_free_env(seni_env *env);

seni_program *seni_compile_program(char *source, seni_word_lut *word_lut, i32 program_max_size);
seni_program *seni_compile_program_with_genotype(char *source, seni_genotype *genotype, seni_word_lut *word_lut, i32 program_max_size);
void          seni_unparse_with_genotype(seni_cursor *out_cursor, char *source, seni_genotype *genotype, seni_word_lut *word_lut);

seni_genotype *seni_deserialize_genotype(seni_cursor *cursor);


seni_trait_list *seni_compile_trait_list(char *source, seni_word_lut *word_lut);
bool             seni_serialize_trait_list(seni_trait_list *trait_list, seni_cursor *cursor);
seni_trait_list *seni_deserialize_trait_list(seni_cursor *cursor);
