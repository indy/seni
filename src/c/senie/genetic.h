#pragma once

#include "types.h"

void ga_subsystem_startup();
void ga_subsystem_shutdown();

struct senie_trait {
  i32 id;

  senie_var*     initial_value;
  senie_program* program;

  struct senie_trait* next;
  struct senie_trait* prev;
};

bool trait_serialize(senie_cursor* cursor, senie_trait* trait);
bool trait_deserialize(senie_trait* out, senie_cursor* cursor);

// store a list of traits
struct senie_trait_list {
  senie_trait* traits;
  i32          seed_value;

  senie_trait_list* next;
  senie_trait_list* prev;
};

senie_trait_list*
                  trait_list_compile(senie_node* ast, i32 trait_program_max_size, senie_word_lut* word_lut, i32 vary);
senie_trait_list* trait_list_get_from_pool();
void              trait_list_return_to_pool(senie_trait_list* trait_list);
i32               trait_list_count(senie_trait_list* trait_list);
bool              trait_list_serialize(senie_cursor* cursor, senie_trait_list* trait_list);
bool              trait_list_deserialize(senie_trait_list* out, senie_cursor* cursor);

struct senie_gene {
  struct senie_var* var;

  struct senie_gene* next;
  struct senie_gene* prev;
};

struct senie_genotype {
  senie_gene* genes;

  // set/get by compiler when compiling program with a genotype
  senie_gene* current_gene;

  // next/prev needed to store in genotype_list
  struct senie_genotype* next;
  struct senie_genotype* prev;
};

senie_genotype* genotype_get_from_pool();
void            genotype_return_to_pool(senie_genotype* genotype);
senie_genotype*
                genotype_build_from_program(senie_trait_list* trait_list, senie_vm* vm, senie_env* env, i32 seed);
senie_genotype* genotype_build_from_initial_values(senie_trait_list* trait_list);
senie_genotype* genotype_clone(senie_genotype* genotype);
senie_genotype*
            genotype_crossover(senie_genotype* a, senie_genotype* b, i32 crossover_index, i32 genotype_length);
bool        genotype_serialize(senie_cursor* cursor, senie_genotype* genotype);
bool        genotype_deserialize(senie_genotype* out, senie_cursor* cursor);
senie_gene* genotype_pull_gene(senie_genotype* genotype);

struct senie_genotype_list {
  senie_genotype* genotypes;

  struct senie_genotype_list* next;
  struct senie_genotype_list* prev;
};

senie_genotype_list* genotype_list_get_from_pool();
void                 genotype_list_return_to_pool(senie_genotype_list* genotype_list);
void genotype_list_add_genotype(senie_genotype_list* genotype_list, senie_genotype* genotype);
senie_genotype* genotype_list_get_genotype(senie_genotype_list* genotype_list, i32 index);
i32             genotype_list_count(senie_genotype_list* genotype_list);
bool            genotype_list_serialize(senie_cursor* cursor, senie_genotype_list* genotype_list);
bool            genotype_list_deserialize(senie_genotype_list* out, senie_cursor* cursor);

senie_genotype_list* genotype_list_create_initial_generation(senie_trait_list* trait_list,
                                                             i32               population_size,
                                                             i32               seed);

senie_genotype_list* genotype_list_next_generation(senie_genotype_list* parents,
                                                   i32                  num_parents,
                                                   i32                  population_size,
                                                   f32                  mutation_rate,
                                                   i32                  rng,
                                                   senie_trait_list*    trait_list);
