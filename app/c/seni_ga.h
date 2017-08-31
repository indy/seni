#pragma once

#include "seni_types.h"

struct seni_trait {
  i32 id;

  seni_var *initial_value;
  seni_program *program;

  struct seni_trait *next;
  struct seni_trait *prev;
};

bool trait_serialize(seni_text_buffer *text_buffer, seni_trait *trait);
bool trait_deserialize(seni_trait *out, seni_text_buffer *text_buffer);

// store a list of traits
struct seni_trait_list {
  seni_trait *traits;
};

seni_trait_list *trait_list_compile(seni_node *ast, i32 trait_program_max_size, seni_word_lut *word_lut);
seni_trait_list *trait_list_allocate();
void             trait_list_free(seni_trait_list *trait_list);
i32              trait_list_count(seni_trait_list *trait_list);
bool             trait_list_serialize(seni_text_buffer *text_buffer, seni_trait_list *trait_list);
bool             trait_list_deserialize(seni_trait_list *out, seni_text_buffer *text_buffer);

struct seni_gene {
  struct seni_var *var;
    
  struct seni_gene *next;
  struct seni_gene *prev;
};

struct seni_gene_slab {
  seni_gene *genes;

  i32 slab_size;

  struct seni_gene_slab *next;
  struct seni_gene_slab *prev;
};

struct seni_gene_pool {
  struct seni_gene_slab *gene_slabs;
  i32 slab_size;
  i32 num_slabs;
  i32 max_slabs_allowed;

  seni_gene *available;

  i32 get_count;
  i32 return_count;
  i32 high_water_mark;
  i32 current_water_mark;
};

// void gene_constructor(seni_gene *gene);
// void gene_destructor(seni_gene *gene);
// struct seni_gene_slab *gene_slab_allocate(i32 num_items);
// void gene_slab_free(struct seni_gene_slab *gene_slab);
// bool gene_pool_add_slab(struct seni_gene_pool *gene_pool);
void gene_pool_free(struct seni_gene_pool *gene_pool);
struct seni_gene_pool *gene_pool_allocate(i32 num_slabs, i32 slab_size, i32 max_slabs_allowed);
seni_gene *gene_pool_get(struct seni_gene_pool *gene_pool);
void gene_pool_return(struct seni_gene_pool *gene_pool, seni_gene *gene);

struct seni_genotype {
  seni_gene *genes;

  // set/get by compiler when compiling program with a genotype
  seni_gene *current_gene;

  // next/prev needed to store in genotype_list
  struct seni_genotype *next;
  struct seni_genotype *prev;
};

seni_genotype *genotype_allocate();
void           genotype_free(seni_genotype *genotype);
seni_genotype *genotype_build(seni_vm *vm, seni_env *env, seni_trait_list *trait_list, i32 seed);
seni_genotype *genotype_build_from_initial_values(seni_trait_list *trait_list);
seni_genotype *genotype_clone(seni_genotype *genotype);
seni_genotype *genotype_crossover(seni_genotype *a, seni_genotype *b, i32 crossover_index, i32 genotype_length);
bool           genotype_serialize(seni_text_buffer *text_buffer, seni_genotype *genotype);
bool           genotype_deserialize(seni_genotype *out, seni_text_buffer *text_buffer);


struct seni_genotype_list {
  seni_genotype *genotypes;
};

seni_genotype_list *genotype_list_allocate();
void                genotype_list_free(seni_genotype_list *genotype_list);
void                genotype_list_add_genotype(seni_genotype_list *genotype_list, seni_genotype *genotype);
seni_genotype      *genotype_list_get_genotype(seni_genotype_list *genotype_list, i32 index);
i32                 genotype_list_count(seni_genotype_list *genotype_list);
bool                genotype_list_serialize(seni_text_buffer *text_buffer, seni_genotype_list *genotype_list);
bool                genotype_list_deserialize(seni_genotype_list *out, seni_text_buffer *text_buffer);

seni_genotype_list *genotype_list_create_initial_generation(seni_trait_list *trait_list, i32 population_size);

seni_genotype_list *genotype_list_next_generation(seni_genotype_list *parents,
                                                  i32 num_parents,
                                                  i32 population_size,
                                                  f32 mutation_rate,
                                                  i32 rng,
                                                  seni_trait_list *trait_list);
