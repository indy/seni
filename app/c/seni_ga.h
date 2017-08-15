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
struct seni_trait_set {
  seni_trait *traits;
};

seni_trait_set *trait_set_compile(seni_node *ast, i32 trait_program_max_size, seni_word_lut *word_lut);
seni_trait_set *trait_set_allocate();
void            trait_set_free(seni_trait_set *trait_set);
i32             trait_set_count(seni_trait_set *trait_set);
bool            trait_set_serialize(seni_text_buffer *text_buffer, seni_trait_set *trait_set);
bool            trait_set_deserialize(seni_trait_set *out, seni_text_buffer *text_buffer);

struct seni_genotype {
  seni_gene *genes;

  // set/get by compiler when compiling program with a genotype
  seni_gene *current_gene;
};

seni_genotype *genotype_build(seni_vm *vm, seni_env *env, seni_trait_set *trait_set, i32 seed);
void           genotype_free(seni_genotype *genotype);
