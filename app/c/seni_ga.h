#ifndef SENI_GA_H
#define SENI_GA_H

#include "seni_lang.h"

typedef struct seni_trait {
  i32 id;

  seni_program *program;

  struct seni_trait *next;
  struct seni_trait *prev;
  
} seni_trait;

// store a list of traits
typedef struct seni_trait_set {
  seni_trait *traits;
} seni_trait_set;

typedef struct seni_gene {
  seni_var var;
  
  struct seni_gene *next;
  struct seni_gene *prev;
} seni_gene;

typedef struct seni_genotype {
  seni_gene *genes;
} seni_genotype;

seni_trait_set *ga_compile_traits(seni_node *ast, i32 trait_program_max_size, seni_word_lut *word_lut);
i32 ga_num_traits(seni_trait_set *trait_set);



#endif
