#include "seni_ga.h"
#include "seni_lang.h"
#include "seni_vm_compiler.h"

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

seni_trait_set *ga_compile_traits(seni_node *ast, i32 trait_program_max_size, seni_word_lut *word_lut)
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

i32 ga_num_traits(seni_trait_set *trait_set)
{
  seni_trait *t = trait_set->traits;
  i32 count = 0;

  while (t != NULL) {
    count++;
    t = t->next;
  }

  return count;
}
