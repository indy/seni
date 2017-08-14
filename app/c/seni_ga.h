#pragma once

#include "seni_types.h"
#include "seni_lang.h"

typedef struct seni_trait {
  i32 id;

  seni_program *program;

  struct seni_trait *next;
  struct seni_trait *prev;
  
} seni_trait;

bool trait_serialize(seni_text_buffer *text_buffer, seni_trait *trait);
bool trait_deserialize(seni_trait *out, seni_text_buffer *text_buffer);

// store a list of traits
typedef struct seni_trait_set {
  seni_trait *traits;
} seni_trait_set;

void trait_set_free(seni_trait_set *trait_set);
seni_trait_set *trait_set_compile(seni_node *ast, i32 trait_program_max_size, seni_word_lut *word_lut);
i32 trait_set_count(seni_trait_set *trait_set);

bool trait_set_serialize(seni_text_buffer *text_buffer, seni_trait_set *trait_set);
bool trait_set_deserialize(seni_trait_set *out, seni_text_buffer *text_buffer);


typedef struct seni_gene {
  seni_var var;
  
  struct seni_gene *next;
  struct seni_gene *prev;
} seni_gene;

typedef struct seni_genotype {
  seni_gene *genes;

  // set/get by compiler when compiling program with a genotype
  seni_gene *current_gene;
} seni_genotype;

seni_genotype *genotype_build(seni_vm *vm, seni_env *env, seni_trait_set *trait_set, i32 seed);
void genotype_free(seni_genotype *genotype);
