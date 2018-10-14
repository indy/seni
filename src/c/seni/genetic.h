#pragma once

#include "types.h"

void ga_subsystem_startup();
void ga_subsystem_shutdown();

struct sen_trait {
  i32 id;

  // 1 == instantiated as one of multiple traits within a vector
  i32 within_vector;
  // if within_vector then this is the index within the parent vector
  i32 index;

  sen_var*     initial_value;
  sen_program* program;

  struct sen_trait* next;
  struct sen_trait* prev;
};
void trait_pretty_print(sen_trait* trait);
bool trait_serialize(sen_cursor* cursor, sen_trait* trait);
bool trait_deserialize(sen_trait* out, sen_cursor* cursor);

// store a list of traits
struct sen_trait_list {
  sen_trait* traits;
  i32        seed_value;

  sen_trait_list* next;
  sen_trait_list* prev;
};

sen_trait_list* trait_list_compile(sen_node*            ast,
                                   sen_compiler_config* compiler_config);
sen_trait_list* trait_list_get_from_pool();
void            trait_list_return_to_pool(sen_trait_list* trait_list);
i32             trait_list_count(sen_trait_list* trait_list);
void            trait_list_pretty_print(char* msg, sen_trait_list* trait_list);
bool trait_list_serialize(sen_cursor* cursor, sen_trait_list* trait_list);
bool trait_list_deserialize(sen_trait_list* out, sen_cursor* cursor);

struct sen_gene {
  struct sen_var* var;

  struct sen_gene* next;
  struct sen_gene* prev;
};

void gene_pretty_print(char* msg, sen_gene* gene);

struct sen_genotype {
  sen_gene* genes;

  // set/get by compiler when compiling program with a genotype
  sen_gene* current_gene;

  // next/prev needed to store in genotype_list
  struct sen_genotype* next;
  struct sen_genotype* prev;
};

void          genotype_pretty_print(sen_genotype* genotype);
sen_genotype* genotype_get_from_pool();
void          genotype_return_to_pool(sen_genotype* genotype);
sen_genotype* genotype_build_from_trait_list(sen_trait_list* trait_list,
                                             sen_vm* vm, sen_env* env,
                                             i32 seed);
sen_genotype* genotype_build_from_initial_values(sen_trait_list* trait_list);
sen_genotype* genotype_clone(sen_genotype* genotype);
sen_genotype* genotype_crossover(sen_genotype* a, sen_genotype* b,
                                 i32 crossover_index, i32 genotype_length);
bool          genotype_serialize(sen_cursor* cursor, sen_genotype* genotype);
bool          genotype_deserialize(sen_genotype* out, sen_cursor* cursor);
sen_gene*     genotype_pull_gene(sen_genotype* genotype);

struct sen_genotype_list {
  sen_genotype* genotypes;

  struct sen_genotype_list* next;
  struct sen_genotype_list* prev;
};

sen_genotype_list* genotype_list_get_from_pool();
void          genotype_list_return_to_pool(sen_genotype_list* genotype_list);
void          genotype_list_add_genotype(sen_genotype_list* genotype_list,
                                         sen_genotype*      genotype);
sen_genotype* genotype_list_get_genotype(sen_genotype_list* genotype_list,
                                         i32                index);
i32           genotype_list_count(sen_genotype_list* genotype_list);
bool          genotype_list_serialize(sen_cursor*        cursor,
                                      sen_genotype_list* genotype_list);
bool genotype_list_deserialize(sen_genotype_list* out, sen_cursor* cursor);

sen_genotype_list*
genotype_list_create_initial_generation(sen_trait_list* trait_list,
                                        i32 population_size, i32 seed);

sen_genotype_list* genotype_list_next_generation(sen_genotype_list* parents,
                                                 i32                num_parents,
                                                 i32 population_size,
                                                 f32 mutation_rate, i32 rng,
                                                 sen_trait_list* trait_list);
