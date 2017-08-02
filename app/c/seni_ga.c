#include "seni_ga.h"
#include "seni_lang.h"
#include "seni_vm_compiler.h"
#include "seni_vm_interpreter.h"
#include "seni_prng.h"

#include <stdlib.h>
#include "utlist.h"

// global genetic algorithm word lookup table
seni_word_lut *g_ga_wl;
seni_trait_set *g_ga_trait_set;

seni_trait *trait_construct()
{
  seni_trait *trait = (seni_trait *)calloc(1, sizeof(seni_trait));

  return trait;
}

void trait_free(seni_trait *trait)
{
  if (trait->program) {
    program_free(trait->program);
  }
  free(trait);
}

seni_trait_set *trait_set_construct()
{
  seni_trait_set *trait_set = (seni_trait_set *)calloc(1, sizeof(seni_trait_set));

  return trait_set;
}

void trait_set_free(seni_trait_set *trait_set)
{
  // todo: test this
  seni_trait *t = trait_set->traits;
  seni_trait *next;
  while (t != NULL) {
    next = t->next;
    DL_DELETE(trait_set->traits, t);
    trait_free(t);
    t = next;
  }
  
  free(trait_set);
}

void trait_set_add_trait(seni_trait_set *trait_set, seni_trait *trait)
{
  DL_APPEND(trait_set->traits, trait);
}

seni_node *ga_traverse(seni_node *node, i32 program_max_size)
{
  seni_node *n = node;
  
  if (n->alterable) {
    node_pretty_print("ga ALTERABLE!!!", n, g_ga_wl);

    seni_trait *trait = trait_construct();
    
    // can compile the parameter_ast
    trait->program = compile_program(n->parameter_ast, program_max_size, g_ga_wl);
    trait_set_add_trait(g_ga_trait_set, trait);
  } else {
    node_pretty_print("ga             ", n, g_ga_wl);
  }

  if (n->type == NODE_LIST) {
    n = n->value.first_child;

    while (n != NULL) {
      ga_traverse(n, program_max_size);
      n = safe_next(n);
    }
  }
  
  return safe_next(node);
}

seni_trait_set *trait_set_compile(seni_node *ast, i32 trait_program_max_size, seni_word_lut *word_lut)
{
  // iterate through and build some traits

  g_ga_wl = word_lut;
  g_ga_trait_set = trait_set_construct();

  seni_node *n = ast;
  while (n != NULL) {
    n = ga_traverse(n, trait_program_max_size);
  }

  return g_ga_trait_set;
}

i32 trait_set_count(seni_trait_set *trait_set)
{
  seni_trait *t = trait_set->traits;
  i32 count = 0;

  while (t != NULL) {
    count++;
    t = t->next;
  }

  return count;
}

// gene

seni_gene *gene_build(seni_vm *vm, seni_env *env, seni_trait *trait)
{
  // todo: possibly implement a 'soft-reset' which is quicker than a vm_reset?
  vm_reset(vm);

  bool res = vm_interpret(vm, env, trait->program);
  if (res == false) {
    return NULL;
  }

  seni_gene *gene = (seni_gene *)calloc(1, sizeof(seni_gene));

  var_copy(&(gene->var), &(vm->stack[vm->sp - 1])); // is this right?

  return gene;
}

void gene_free(seni_gene *gene)
{
  free(gene);
}

seni_gene *gene_clone(seni_gene *source)
{
  seni_gene *gene = (seni_gene *)calloc(1, sizeof(seni_gene));

  var_copy(&(gene->var), &(source->var));

  return gene;
}

// genotype

void genotype_add_gene(seni_genotype *genotype, seni_gene *gene)
{
  DL_APPEND(genotype->genes, gene);
}

seni_genotype *genotype_build(seni_vm *vm, seni_env *env, seni_trait_set *trait_set, i32 seed)
{
  // the seed is set once per genotype (should it be once per-gene?)
  //
  seni_prng_set_state(&(vm->prng_state), (u64)seed);

  seni_genotype *genotype = (seni_genotype *)calloc(1, sizeof(seni_genotype));

  seni_trait *trait = trait_set->traits;
  while (trait != NULL) {
    seni_gene *gene = gene_build(vm, env, trait);

    if (gene != NULL) {
      genotype_add_gene(genotype, gene);
    } else {
      SENI_ERROR("gene_build returned NULL gene");
      genotype_free(genotype);
      return NULL;
    }

    trait = trait->next;
  }
  
  return genotype;
}

void genotype_free(seni_genotype *genotype)
{
  seni_gene *g = genotype->genes;
  seni_gene *next;
  while (g != NULL) {
    next = g->next;
    DL_DELETE(genotype->genes, g);
    gene_free(g);
    g = next;
  }
  
  free(genotype);  
}

// todo: add mutation_rate, traits, env and vm
void random_crossover(seni_genotype *a, seni_genotype *b, i32 genotype_length)
{
  // assuming that both genotypes are of the given length

  
}
