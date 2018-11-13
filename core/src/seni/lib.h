#pragma once

#include "config.h"
#include "types.h"

sen_error sen_systems_startup();
void      sen_systems_shutdown();

sen_vm* sen_allocate_vm(i32 stack_size, i32 heap_size, i32 heap_min_size,
                        i32 vertex_packet_num_vertices);
void    sen_free_vm(sen_vm* vm);
void    sen_reset_vm(sen_vm* vm);

sen_env* sen_allocate_env();
void     sen_free_env(sen_env* env);

sen_result_program sen_compile_program(char* source, sen_word_lut* word_lut,
                                       i32 program_max_size);
sen_result_program sen_compile_program_with_genotype(char* source, sen_genotype* genotype,
                                                     sen_word_lut* word_lut,
                                                     i32           program_max_size);
sen_error          sen_unparse_with_genotype(sen_cursor* out_cursor, char* source,
                                             sen_genotype* genotype, sen_word_lut* word_lut);

sen_error sen_simplify_script(sen_cursor* out_cursor, char* source, sen_word_lut* word_lut);

sen_genotype* sen_deserialize_genotype(sen_cursor* cursor);

sen_result_trait_list sen_compile_trait_list(char* source, sen_word_lut* word_lut);
bool            sen_serialize_trait_list(sen_trait_list* trait_list, sen_cursor* cursor);
sen_trait_list* sen_deserialize_trait_list(sen_cursor* cursor);
