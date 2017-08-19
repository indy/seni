#include "seni_ga.h"

#include "seni_lang.h"
#include "seni_prng.h"
#include "seni_text_buffer.h"
#include "seni_vm_compiler.h"
#include "seni_vm_interpreter.h"

#include <stdlib.h>
#include "utlist.h"

// global genetic algorithm word lookup table
seni_word_lut *g_ga_wl;
seni_trait_list *g_ga_trait_list;

seni_trait *trait_allocate()
{
  seni_trait *trait = (seni_trait *)calloc(1, sizeof(seni_trait));

  trait->initial_value = (seni_var *)calloc(1, sizeof(seni_var));

  return trait;
}

void trait_free(seni_trait *trait)
{
  if (trait->program) {
    program_free(trait->program);
  }

  if (trait->initial_value) {
    free(trait->initial_value);
  }
  
  free(trait);
}

bool trait_serialize(seni_text_buffer *text_buffer, seni_trait *trait)
{
  text_buffer_sprintf(text_buffer, "%d", trait->id);
  text_buffer_sprintf(text_buffer, " ");

  var_serialize(text_buffer, trait->initial_value);
  text_buffer_sprintf(text_buffer, " ");

  program_serialize(text_buffer, trait->program);

  return true;
}

bool trait_deserialize(seni_trait *out, seni_text_buffer *text_buffer)
{
  out->id = text_buffer_eat_i32(text_buffer);
  text_buffer_eat_space(text_buffer);

  var_deserialize(out->initial_value, text_buffer);
  text_buffer_eat_space(text_buffer);

  if (out->program != NULL) {
    free(out->program);
  }
  out->program = program_allocate(0);
  program_deserialize(out->program, text_buffer);

  return true;
}

seni_trait_list *trait_list_allocate()
{
  seni_trait_list *trait_list = (seni_trait_list *)calloc(1, sizeof(seni_trait_list));

  return trait_list;
}

void trait_list_free(seni_trait_list *trait_list)
{
  // todo: test this
  seni_trait *t = trait_list->traits;
  seni_trait *next;
  while (t != NULL) {
    next = t->next;
    DL_DELETE(trait_list->traits, t);
    trait_free(t);
    t = next;
  }
  
  free(trait_list);
}

void trait_list_add_trait(seni_trait_list *trait_list, seni_trait *trait)
{
  DL_APPEND(trait_list->traits, trait);
}

bool hack_node_to_var(seni_var *out, seni_node *node)
{
  switch(node->type) {
  case NODE_INT:
    out->type = VAR_INT;
    out->value.i = node->value.i;
    break;
  case NODE_FLOAT:
    out->type = VAR_FLOAT;
    out->value.f = node->value.f;
    break;
  case NODE_NAME:
    out->type = VAR_NAME;
    out->value.i = node->value.i;
    break;
  default:
    // todo: check NODE_LIST for colour, 2D etc
    return false;
  }

  return true;
}

seni_node *ga_traverse(seni_node *node, i32 program_max_size)
{
  seni_node *n = node;

  if (n->alterable) {
    seni_trait *trait = trait_allocate();

    if (hack_node_to_var(trait->initial_value, n) == false) {
      SENI_PRINT("hack_node_to_var failed");
    }
    
    // can compile the parameter_ast
    trait->program = compile_program(n->parameter_ast, program_max_size, g_ga_wl);
    
    trait_list_add_trait(g_ga_trait_list, trait);
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

seni_trait_list *trait_list_compile(seni_node *ast, i32 trait_program_max_size, seni_word_lut *word_lut)
{
  // iterate through and build some traits

  g_ga_wl = word_lut;
  g_ga_trait_list = trait_list_allocate();

  seni_node *n = ast;
  while (n != NULL) {
    n = ga_traverse(n, trait_program_max_size);
  }

  return g_ga_trait_list;
}

i32 trait_list_count(seni_trait_list *trait_list)
{
  seni_trait *t = trait_list->traits;
  i32 count = 0;

  while (t != NULL) {
    count++;
    t = t->next;
  }

  return count;
}

bool trait_list_serialize(seni_text_buffer *text_buffer, seni_trait_list *trait_list)
{
  // number of traits
  i32 count = trait_list_count(trait_list);
  text_buffer_sprintf(text_buffer, "%d", count);
  text_buffer_sprintf(text_buffer, " ");

  // sequence of traits
  seni_trait *t = trait_list->traits;
  while (t != NULL) {
    trait_serialize(text_buffer, t);
    if (t->next != NULL) {
      text_buffer_sprintf(text_buffer, " ");
    }
    t = t->next;
  }

  return true;
}

bool trait_list_deserialize(seni_trait_list *out, seni_text_buffer *text_buffer)
{
  i32 count = text_buffer_eat_i32(text_buffer);
  text_buffer_eat_space(text_buffer);

  seni_trait_list *trait_list = out;

  for (i32 i = 0; i < count; i++) {
    seni_trait *trait = trait_allocate();
    trait_deserialize(trait, text_buffer);
    trait_list_add_trait(trait_list, trait);
    if (i < count - 1) {
      text_buffer_eat_space(text_buffer);
    }
  }

  return true;
}


// gene

seni_gene *gene_allocate()
{
  seni_gene *gene = (seni_gene *)calloc(1, sizeof(seni_gene));

  return gene;
}

void gene_free(seni_gene *gene)
{
  free(gene);
}

seni_gene *gene_build(seni_vm *vm, seni_env *env, seni_trait *trait)
{
  // todo: possibly implement a 'soft-reset' which is quicker than a vm_reset?
  vm_reset(vm);

  bool res = vm_interpret(vm, env, trait->program);
  if (res == false) {
    return NULL;
  }

  seni_gene *gene = gene_allocate();

  var_copy(&(gene->var), &(vm->stack[vm->sp - 1])); // is this right?

  return gene;
}

seni_gene *gene_build_from_initial_value(seni_trait *trait)
{
  seni_gene *gene = gene_allocate();

  var_copy(&(gene->var), trait->initial_value);
  
  return gene;  
}

seni_gene *gene_clone(seni_gene *source)
{
  seni_gene *gene = (seni_gene *)calloc(1, sizeof(seni_gene));

  var_copy(&(gene->var), &(source->var));

  return gene;
}

// genotype

seni_genotype *genotype_allocate()
{
  seni_genotype *genotype = (seni_genotype *)calloc(1, sizeof(seni_genotype));

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

void genotype_add_gene(seni_genotype *genotype, seni_gene *gene)
{
  DL_APPEND(genotype->genes, gene);
}

seni_genotype *genotype_build(seni_vm *vm, seni_env *env, seni_trait_list *trait_list, i32 seed)
{
  // the seed is set once per genotype (should it be once per-gene?)
  //
  seni_prng_set_state(vm->prng_state, (u64)seed);

  seni_genotype *genotype = genotype_allocate();

  seni_trait *trait = trait_list->traits;
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

seni_genotype *genotype_build_from_initial_values(seni_trait_list *trait_list)
{
  seni_genotype *genotype = genotype_allocate();

  seni_trait *trait = trait_list->traits;
  while (trait != NULL) {
    seni_gene *gene = gene_build_from_initial_value(trait);

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


i32 genotype_count(seni_genotype *genotype)
{
  seni_gene *g = genotype->genes;
  i32 count = 0;

  while (g != NULL) {
    count++;
    g = g->next;
  }

  return count;
}

bool genotype_serialize(seni_text_buffer *text_buffer, seni_genotype *genotype)
{
  // number of genes
  i32 count = genotype_count(genotype);
  text_buffer_sprintf(text_buffer, "%d", count);
  text_buffer_sprintf(text_buffer, " ");
  
  seni_gene *gene = genotype->genes;
  while (gene != NULL) {
    var_serialize(text_buffer, &(gene->var));
    if (gene->next != NULL) {
      text_buffer_sprintf(text_buffer, " ");
    }
    gene = gene->next;
  }

  return true;
}

bool genotype_deserialize(seni_genotype *out, seni_text_buffer *text_buffer)
{
  i32 count = text_buffer_eat_i32(text_buffer);
  text_buffer_eat_space(text_buffer);

  seni_genotype *genotype = out;

  for (i32 i = 0; i < count; i++) {
    seni_gene *gene = gene_allocate();
    var_deserialize(&(gene->var), text_buffer);
    genotype_add_gene(genotype, gene);
    if (i < count - 1) {
      text_buffer_eat_space(text_buffer);
    }
  }

  return true;
}

// todo: add mutation_rate, traits, env and vm
void random_crossover(seni_genotype *a, seni_genotype *b, i32 genotype_length)
{
  // assuming that both genotypes are of the given length
}


seni_genotype_list *genotype_list_allocate()
{
  seni_genotype_list *genotype_list = (seni_genotype_list *)calloc(1, sizeof(seni_genotype_list));

  return genotype_list;
}

void genotype_list_free(seni_genotype_list *genotype_list)
{
  // todo: test this
  seni_genotype *g = genotype_list->genotypes;
  seni_genotype *next;
  while (g != NULL) {
    next = g->next;
    DL_DELETE(genotype_list->genotypes, g);
    genotype_free(g);
    g = next;
  }
  
  free(genotype_list);
}

void genotype_list_add_genotype(seni_genotype_list *genotype_list, seni_genotype *genotype)
{
  DL_APPEND(genotype_list->genotypes, genotype);
}

i32 genotype_list_count(seni_genotype_list *genotype_list)
{
  seni_genotype *g = genotype_list->genotypes;
  i32 count = 0;

  while (g != NULL) {
    count++;
    g = g->next;
  }

  return count;
}

bool genotype_list_serialize(seni_text_buffer *text_buffer, seni_genotype_list *genotype_list)
{
  // number of genotypes
  i32 count = genotype_list_count(genotype_list);
  text_buffer_sprintf(text_buffer, "%d", count);
  text_buffer_sprintf(text_buffer, " ");

  // sequence of genotypes
  seni_genotype *g = genotype_list->genotypes;
  while (g != NULL) {
    genotype_serialize(text_buffer, g);
    if (g->next != NULL) {
      text_buffer_sprintf(text_buffer, " ");
    }
    g = g->next;
  }

  return true;
}

bool genotype_list_deserialize(seni_genotype_list *out, seni_text_buffer *text_buffer)
{
  i32 count = text_buffer_eat_i32(text_buffer);
  text_buffer_eat_space(text_buffer);

  seni_genotype_list *genotype_list = out;

  for (i32 i = 0; i < count; i++) {
    seni_genotype *genotype = genotype_allocate();
    genotype_deserialize(genotype, text_buffer);
    genotype_list_add_genotype(genotype_list, genotype);
    if (i < count - 1) {
      text_buffer_eat_space(text_buffer);
    }
  }

  return true;
}

seni_genotype_list *genotype_list_create_initial_generation(seni_trait_list *trait_list, i32 population_size)
{
  seni_genotype_list *genotype_list = genotype_list_allocate();
  if (population_size == 0) {
    SENI_ERROR("genotype_list_create_initial_generation: population_size of 0 ???");
    return genotype_list;
  }

  // create a genotype using the initial valued from the traits
  seni_genotype *genotype = genotype_build_from_initial_values(trait_list);
  genotype_list_add_genotype(genotype_list, genotype);

  /*
    the genotype_build function (or vm setup) seems to be crashing wasm
   */
  //#define GENO_HACK
#ifdef GENO_HACK

  for (i32 i = 1; i < population_size; i++) {
    genotype = genotype_build_from_initial_values(trait_list);
    genotype_list_add_genotype(genotype_list, genotype);
  }

#else

  // fill out the remaining population with generated values
  seni_vm *vm = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES);
  seni_env *env = env_allocate();

  i32 seed_value = 42;
  for (i32 i = 1; i < population_size; i++) {
    seed_value += 1425;
    genotype = genotype_build(vm, env, trait_list, seed_value);
    genotype_list_add_genotype(genotype_list, genotype);
  }

  env_free(env);
  vm_free(vm);

#endif
  

  return genotype_list;
}
