#include "seni_ga.h"
#include "seni_config.h"

#include "seni_lang.h"
#include "seni_prng.h"
#include "seni_text_buffer.h"
#include "seni_vm_compiler.h"
#include "seni_vm_interpreter.h"

#include <stdlib.h>
#include "lib/utlist.h"

#include "seni_pool_macro.h"

void gene_return_to_pool(seni_gene *gene);
void trait_return_to_pool(seni_trait *trait);

// setup pools

void gene_cleanup(seni_gene *gene)
{
  if (gene->var) {
    var_return_to_pool(gene->var);
    gene->var = NULL;
  }
}

void trait_cleanup(seni_trait *trait)
{
  if (trait->program) {
    program_free(trait->program);
    trait->program = NULL;
  }

  if (trait->initial_value) {
    var_return_to_pool(trait->initial_value);
    trait->initial_value = NULL;
  }
}

void genotype_cleanup(seni_genotype *genotype)
{
  seni_gene *g = genotype->genes;
  seni_gene *next;
  while (g != NULL) {
    next = g->next;
    DL_DELETE(genotype->genes, g);
    gene_return_to_pool(g);
    g = next;
  }
  genotype->genes = NULL;
  genotype->current_gene = NULL;
}

void trait_list_cleanup(seni_trait_list *trait_list)
{
  seni_trait *t = trait_list->traits;
  seni_trait *next;
  while (t != NULL) {
    next = t->next;
    DL_DELETE(trait_list->traits, t);
  
    trait_return_to_pool(t);
    t = next;
  }

  trait_list->traits = NULL;
}

void genotype_list_cleanup(seni_genotype_list *genotype_list)
{
  // todo: test this
  seni_genotype *g = genotype_list->genotypes;
  seni_genotype *next;
  while (g != NULL) {
    next = g->next;
    DL_DELETE(genotype_list->genotypes, g);
    genotype_return_to_pool(g);
    g = next;
  }

  genotype_list->genotypes = NULL;
}

// define the pool structures
//

SENI_POOL(seni_gene, gene)
SENI_POOL(seni_trait, trait)
SENI_POOL(seni_genotype, genotype)
SENI_POOL(seni_trait_list, trait_list)
SENI_POOL(seni_genotype_list, genotype_list)

struct seni_gene_pool *g_gene_pool;
struct seni_trait_pool *g_trait_pool;
struct seni_genotype_pool *g_genotype_pool;
struct seni_trait_list_pool *g_trait_list_pool;
struct seni_genotype_list_pool *g_genotype_list_pool;

seni_gene *gene_get_from_pool()
{
  seni_gene *gene = gene_pool_get(g_gene_pool);

  seni_var *var = var_get_from_pool();
  gene->var = var;

  return gene;
}

void gene_return_to_pool(seni_gene *gene)
{
  gene_cleanup(gene);
  gene_pool_return(g_gene_pool, gene);
}

seni_trait *trait_get_from_pool()
{
  seni_trait *trait = trait_pool_get(g_trait_pool);

  // get a var from the pool in seni_lang
  seni_var *var = var_get_from_pool();
  trait->initial_value = var;

  return trait;
}

void trait_return_to_pool(seni_trait *trait)
{
  // free up any memory used to store the program and initial_value
  trait_cleanup(trait);
  trait_pool_return(g_trait_pool, trait);
}

seni_genotype *genotype_get_from_pool()
{
  seni_genotype *genotype = genotype_pool_get(g_genotype_pool);

  return genotype;
}

void genotype_return_to_pool(seni_genotype *genotype)
{
  genotype_cleanup(genotype);
  genotype_pool_return(g_genotype_pool, genotype);
}

seni_trait_list *trait_list_get_from_pool()
{
  seni_trait_list *trait_list = trait_list_pool_get(g_trait_list_pool);

  return trait_list;
}

void trait_list_return_to_pool(seni_trait_list *trait_list)
{
  trait_list_cleanup(trait_list);
  trait_list_pool_return(g_trait_list_pool, trait_list);
}

seni_genotype_list *genotype_list_get_from_pool()
{
  seni_genotype_list *genotype_list = genotype_list_pool_get(g_genotype_list_pool);

  return genotype_list;
}

void genotype_list_return_to_pool(seni_genotype_list *genotype_list)
{
  genotype_list_cleanup(genotype_list);
  genotype_list_pool_return(g_genotype_list_pool, genotype_list);
}

void ga_pools_startup()
{
  // create 1 slab
  // each slab contains 20 genes
  // max of 10 slabs can be allocated
  g_gene_pool = gene_pool_allocate(1, 20, 10);
  g_trait_pool = trait_pool_allocate(1, 20, 10);
  g_genotype_pool = genotype_pool_allocate(1, 20, 10);
  g_trait_list_pool = trait_list_pool_allocate(1, 20, 10);
  g_genotype_list_pool = genotype_list_pool_allocate(1, 20, 10);
}

void ga_pools_shutdown()
{
  genotype_list_pool_free(g_genotype_list_pool);
  trait_list_pool_free(g_trait_list_pool);
  genotype_pool_free(g_genotype_pool);
  trait_pool_free(g_trait_pool);
  gene_pool_free(g_gene_pool);
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

seni_node *ga_traverse(seni_node *node, i32 program_max_size, seni_trait_list *trait_list, seni_word_lut *word_lut)
{
  seni_node *n = node;

  if (n->alterable) {
    seni_trait *trait = trait_get_from_pool();

    if (hack_node_to_var(trait->initial_value, n) == false) {
      SENI_PRINT("hack_node_to_var failed");
      node_pretty_print("failed node", n, word_lut);
    }
    
    // can compile the parameter_ast
    trait->program = compile_program(n->parameter_ast, program_max_size, word_lut);
    
    trait_list_add_trait(trait_list, trait);
  }

  if (n->type == NODE_LIST || n->type == NODE_VECTOR) {
    n = n->value.first_child;

    while (n != NULL) {
      ga_traverse(n, program_max_size, trait_list, word_lut);
      n = safe_next(n);
    }
  }
  
  return safe_next(node);
}

seni_trait_list *trait_list_compile(seni_node *ast, i32 trait_program_max_size, seni_word_lut *word_lut)
{
  // iterate through and build some traits
  seni_trait_list *trait_list = trait_list_get_from_pool();

  seni_node *n = ast;
  while (n != NULL) {
    n = ga_traverse(n, trait_program_max_size, trait_list, word_lut);
  }

  return trait_list;
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
    seni_trait *trait = trait_get_from_pool();
    trait_deserialize(trait, text_buffer);
    trait_list_add_trait(trait_list, trait);
    if (i < count - 1) {
      text_buffer_eat_space(text_buffer);
    }
  }

  return true;
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

  seni_gene *gene = gene_get_from_pool();

  var_copy(gene->var, &(vm->stack[vm->sp - 1])); // is this right?

  return gene;
}

seni_gene *gene_build_from_initial_value(seni_trait *trait)
{
  seni_gene *gene = gene_get_from_pool(); //  gene_allocate();

  var_copy(gene->var, trait->initial_value);
  
  return gene;  
}

seni_gene *gene_clone(seni_gene *source)
{
  seni_gene *gene = gene_get_from_pool();

  var_copy(gene->var, source->var);

  return gene;
}

// genotype

void genotype_add_gene(seni_genotype *genotype, seni_gene *gene)
{
  DL_APPEND(genotype->genes, gene);
}

seni_genotype *genotype_build(seni_vm *vm, seni_env *env, seni_trait_list *trait_list, i32 seed)
{
  // the seed is set once per genotype (should it be once per-gene?)
  //
  seni_prng_set_state(vm->prng_state, (u64)seed);

  seni_genotype *genotype = genotype_get_from_pool();

  seni_trait *trait = trait_list->traits;
  while (trait != NULL) {
    seni_gene *gene = gene_build(vm, env, trait);

    if (gene != NULL) {
      genotype_add_gene(genotype, gene);
    } else {
      SENI_ERROR("gene_build returned NULL gene");
      genotype_return_to_pool(genotype);
      return NULL;
    }

    trait = trait->next;
  }
  
  return genotype;
}

seni_genotype *genotype_build_from_initial_values(seni_trait_list *trait_list)
{
  seni_genotype *genotype = genotype_get_from_pool();

  seni_trait *trait = trait_list->traits;
  while (trait != NULL) {
    seni_gene *gene = gene_build_from_initial_value(trait);

    if (gene != NULL) {
      genotype_add_gene(genotype, gene);
    } else {
      SENI_ERROR("gene_build returned NULL gene");
      genotype_return_to_pool(genotype);
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

seni_genotype *genotype_clone(seni_genotype *genotype)
{
  seni_genotype *cloned_genotype = genotype_get_from_pool();

  seni_gene *src_gene = genotype->genes;

  while (src_gene) {
    seni_gene *cloned_gene = gene_clone(src_gene);
    genotype_add_gene(cloned_genotype, cloned_gene);
    
    src_gene = src_gene->next;
  }
  
  return cloned_genotype;
}

seni_genotype *genotype_crossover(seni_genotype *a, seni_genotype *b, i32 crossover_index, i32 genotype_length)
{
  seni_genotype *genotype = genotype_get_from_pool();
  seni_gene *gene;
  seni_gene *gene_a = a->genes;
  seni_gene *gene_b = b->genes;
  i32 i;
  
  for (i = 0; i < crossover_index; i++) {
    gene = gene_clone(gene_a);
    genotype_add_gene(genotype, gene);
    gene_a = gene_a->next;
    gene_b = gene_b->next;      // keep gene_b in sync
  }

  for (i = crossover_index; i < genotype_length; i++) {
    gene = gene_clone(gene_b);
    genotype_add_gene(genotype, gene);
    gene_b = gene_b->next;
  }

  return genotype;
}

bool genotype_serialize(seni_text_buffer *text_buffer, seni_genotype *genotype)
{
  // number of genes
  i32 count = genotype_count(genotype);
  text_buffer_sprintf(text_buffer, "%d", count);
  text_buffer_sprintf(text_buffer, " ");
  
  seni_gene *gene = genotype->genes;
  while (gene != NULL) {
    var_serialize(text_buffer, gene->var);
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
    seni_gene *gene = gene_get_from_pool();
    var_deserialize(gene->var, text_buffer);
    genotype_add_gene(genotype, gene);
    if (i < count - 1) {
      text_buffer_eat_space(text_buffer);
    }
  }

  return true;
}

// todo: add mutation_rate, traits, env and vm
// void random_crossover(seni_genotype *a, seni_genotype *b, i32 genotype_length)
// {
//   // assuming that both genotypes are of the given length
// }

void genotype_list_add_genotype(seni_genotype_list *genotype_list, seni_genotype *genotype)
{
  DL_APPEND(genotype_list->genotypes, genotype);
}

seni_genotype *genotype_list_get_genotype(seni_genotype_list *genotype_list, i32 index)
{
  seni_genotype *genotype = genotype_list->genotypes;
  i32 i = 0;

  while (i < index) {
    if (genotype == NULL) {
      return NULL;
    }
    genotype = genotype->next;
    i++;
  }

  return genotype;
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
    seni_genotype *genotype = genotype_get_from_pool();
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
  seni_genotype_list *genotype_list = genotype_list_get_from_pool();
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

seni_genotype_list *genotype_list_next_generation(seni_genotype_list *parents,
                                                  i32 num_parents,
                                                  i32 population_size,
                                                  f32 mutation_rate,
                                                  i32 rng,
                                                  seni_trait_list *trait_list)
{
  // pleasing the compiler
  trait_list = NULL;
  mutation_rate = 0;
  
  seni_genotype_list *genotype_list = genotype_list_get_from_pool();

  i32 population_remaining = population_size;

  // copy the parents onto the new generation
  seni_genotype *genotype;
  seni_genotype *parent_genotype = parents->genotypes;
  while (parent_genotype) {
    genotype = genotype_clone(parent_genotype);
    genotype_list_add_genotype(genotype_list, genotype);
    parent_genotype = parent_genotype->next;
    population_remaining--;
  }

  SENI_PRINT("genotype_list_next_generation: rng = %d", rng);
  seni_prng_state prng_state;
  seni_prng_set_state(&prng_state, (u64)rng);
  i32 retry_count = 10;
  
  SENI_PRINT("num_parents = %d", num_parents);
  
  while (population_remaining) {
    u32 a_index = seni_prng_i32_range(&prng_state, 0, num_parents - 1);
    u32 b_index = a_index;
    for (i32 retry = 0; retry < retry_count; retry++) {
      b_index = seni_prng_i32_range(&prng_state, 0, num_parents - 1);
      if (b_index != a_index) {
        break;
      }
    }
    if (b_index == a_index) {
      b_index = (a_index + 1) % num_parents;
    }
    SENI_PRINT("a_index = %d, b_index = %d", a_index, b_index);

    seni_genotype *a = genotype_list_get_genotype(parents, a_index);
    seni_genotype *b = genotype_list_get_genotype(parents, b_index);

    i32 genotype_length = genotype_count(a);
    i32 crossover_index = seni_prng_i32_range(&prng_state, 0, genotype_length - 1);

    SENI_PRINT("genotype_length %d, crossover_index %d", genotype_length, crossover_index);
    
    seni_genotype *g = genotype_crossover(a, b, crossover_index, genotype_length);
    genotype_list_add_genotype(genotype_list, g);

    population_remaining--;
  }

  i32 final_count = genotype_list_count(genotype_list);
  SENI_PRINT("population_size = %d, final_count = %d", population_size, final_count);
  
  return genotype_list;
}
