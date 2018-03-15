#pragma once

#include "config.h"
#include "types.h"

void senie_systems_startup();
void senie_systems_shutdown();

senie_vm*
     senie_allocate_vm(i32 stack_size, i32 heap_size, i32 heap_min_size, i32 vertex_packet_num_vertices);
void senie_free_vm(senie_vm* vm);
void senie_reset_vm(senie_vm* vm);

senie_env* senie_allocate_env();
void       senie_free_env(senie_env* env);

senie_program* senie_compile_program(char* source, senie_word_lut* word_lut, i32 program_max_size);
senie_program* senie_compile_program_with_genotype(char*           source,
                                                   senie_genotype* genotype,
                                                   senie_word_lut* word_lut,
                                                   i32             program_max_size);
void           senie_unparse_with_genotype(senie_cursor*   out_cursor,
                                           char*           source,
                                           senie_genotype* genotype,
                                           senie_word_lut* word_lut);

senie_genotype* senie_deserialize_genotype(senie_cursor* cursor);

senie_trait_list* senie_compile_trait_list(char* source, senie_word_lut* word_lut);
bool              senie_serialize_trait_list(senie_trait_list* trait_list, senie_cursor* cursor);
senie_trait_list* senie_deserialize_trait_list(senie_cursor* cursor);
