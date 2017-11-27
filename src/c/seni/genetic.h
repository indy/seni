#pragma once

#include "types.h"

void ga_subsystem_startup();
void ga_subsystem_shutdown();

struct seni_trait {
  i32 id;

  seni_var *    initial_value;
  seni_program *program;

  struct seni_trait *next;
  struct seni_trait *prev;
};

bool trait_serialize(seni_cursor *cursor, seni_trait *trait);
bool trait_deserialize(seni_trait *out, seni_cursor *cursor);

// store a list of traits
struct seni_trait_list {
  seni_trait *traits;
  i32         seed_value;

  seni_trait_list *next;
  seni_trait_list *prev;
};

seni_trait_list *
                 trait_list_compile(seni_node *ast, i32 trait_program_max_size, seni_word_lut *word_lut);
seni_trait_list *trait_list_get_from_pool();
void             trait_list_return_to_pool(seni_trait_list *trait_list);
i32              trait_list_count(seni_trait_list *trait_list);
bool             trait_list_serialize(seni_cursor *cursor, seni_trait_list *trait_list);
bool             trait_list_deserialize(seni_trait_list *out, seni_cursor *cursor);

struct seni_gene {
  struct seni_var *var;

  struct seni_gene *next;
  struct seni_gene *prev;
};

struct seni_genotype {
  seni_gene *genes;

  // set/get by compiler when compiling program with a genotype
  seni_gene *current_gene;

  // next/prev needed to store in genotype_list
  struct seni_genotype *next;
  struct seni_genotype *prev;
};

seni_genotype *genotype_get_from_pool();
void           genotype_return_to_pool(seni_genotype *genotype);
seni_genotype *
               genotype_build_from_program(seni_trait_list *trait_list, seni_vm *vm, seni_env *env, i32 seed);
seni_genotype *genotype_build_from_initial_values(seni_trait_list *trait_list);
seni_genotype *genotype_clone(seni_genotype *genotype);
seni_genotype *
           genotype_crossover(seni_genotype *a, seni_genotype *b, i32 crossover_index, i32 genotype_length);
bool       genotype_serialize(seni_cursor *cursor, seni_genotype *genotype);
bool       genotype_deserialize(seni_genotype *out, seni_cursor *cursor);
seni_gene *genotype_pull_gene(seni_genotype *genotype);

struct seni_genotype_list {
  seni_genotype *genotypes;

  struct seni_genotype_list *next;
  struct seni_genotype_list *prev;
};

seni_genotype_list *genotype_list_get_from_pool();
void                genotype_list_return_to_pool(seni_genotype_list *genotype_list);
void genotype_list_add_genotype(seni_genotype_list *genotype_list, seni_genotype *genotype);
seni_genotype *genotype_list_get_genotype(seni_genotype_list *genotype_list, i32 index);
i32            genotype_list_count(seni_genotype_list *genotype_list);
bool           genotype_list_serialize(seni_cursor *cursor, seni_genotype_list *genotype_list);
bool           genotype_list_deserialize(seni_genotype_list *out, seni_cursor *cursor);

seni_genotype_list *
genotype_list_create_initial_generation(seni_trait_list *trait_list, i32 population_size, i32 seed);

seni_genotype_list *genotype_list_next_generation(seni_genotype_list *parents,
                                                  i32                 num_parents,
                                                  i32                 population_size,
                                                  f32                 mutation_rate,
                                                  i32                 rng,
                                                  seni_trait_list *   trait_list);
