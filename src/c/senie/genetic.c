#include "genetic.h"
#include "config.h"

#include "bind.h"
#include "colour.h"
#include "cursor.h"
#include "keyword_iname.h"
#include "lang.h"
#include "prng.h"
#include "vm_compiler.h"
#include "vm_interpreter.h"

#include "../lib/utlist.h"
#include <stdlib.h>

#include "pool_macro.h"

// used to evaluate the initial values of traits
senie_vm*      g_ga_vm;
senie_env*     g_ga_env;
senie_program* g_ga_program;

void gene_return_to_pool(senie_gene* gene);
void trait_return_to_pool(senie_trait* trait);

// setup pools

void gene_cleanup(senie_gene* gene) {
  if (gene->var) {
    var_return_to_pool(gene->var);
    gene->var = NULL;
  }
}

void trait_cleanup(senie_trait* trait) {
  if (trait->program) {
    program_free(trait->program);
    trait->program = NULL;
  }

  if (trait->initial_value) {
    var_return_to_pool(trait->initial_value);
    trait->initial_value = NULL;
  }
}

void genotype_cleanup(senie_genotype* genotype) {
  senie_gene* g = genotype->genes;
  senie_gene* next;
  while (g != NULL) {
    next = g->next;
    DL_DELETE(genotype->genes, g);
    gene_return_to_pool(g);
    g = next;
  }
  genotype->genes        = NULL;
  genotype->current_gene = NULL;
}

void trait_list_cleanup(senie_trait_list* trait_list) {
  senie_trait* t = trait_list->traits;
  senie_trait* next;
  while (t != NULL) {
    next = t->next;
    DL_DELETE(trait_list->traits, t);

    trait_return_to_pool(t);
    t = next;
  }

  trait_list->traits = NULL;
}

void genotype_list_cleanup(senie_genotype_list* genotype_list) {
  // todo: test this
  senie_genotype* g = genotype_list->genotypes;
  senie_genotype* next;
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

SENIE_POOL(senie_gene, gene)
SENIE_POOL(senie_trait, trait)
SENIE_POOL(senie_genotype, genotype)
SENIE_POOL(senie_trait_list, trait_list)
SENIE_POOL(senie_genotype_list, genotype_list)

struct senie_gene_pool*          g_gene_pool;
struct senie_trait_pool*         g_trait_pool;
struct senie_genotype_pool*      g_genotype_pool;
struct senie_trait_list_pool*    g_trait_list_pool;
struct senie_genotype_list_pool* g_genotype_list_pool;

senie_gene* gene_get_from_pool() {
  senie_gene* gene = gene_pool_get(g_gene_pool);

  senie_var* var = var_get_from_pool();
  gene->var      = var;

  return gene;
}

void gene_return_to_pool(senie_gene* gene) {
  gene_cleanup(gene);
  gene_pool_return(g_gene_pool, gene);
}

senie_trait* trait_get_from_pool() {
  senie_trait* trait = trait_pool_get(g_trait_pool);

  // get a var from the pool in senie_lang
  senie_var* var       = var_get_from_pool();
  trait->initial_value = var;

  return trait;
}

void trait_return_to_pool(senie_trait* trait) {
  // free up any memory used to store the program and initial_value
  trait_cleanup(trait);
  trait_pool_return(g_trait_pool, trait);
}

senie_genotype* genotype_get_from_pool() {
  senie_genotype* genotype = genotype_pool_get(g_genotype_pool);

  return genotype;
}

void genotype_return_to_pool(senie_genotype* genotype) {
  genotype_cleanup(genotype);
  genotype_pool_return(g_genotype_pool, genotype);
}

senie_trait_list* trait_list_get_from_pool() {
  senie_trait_list* trait_list = trait_list_pool_get(g_trait_list_pool);

  return trait_list;
}

void trait_list_return_to_pool(senie_trait_list* trait_list) {
  trait_list_cleanup(trait_list);
  trait_list_pool_return(g_trait_list_pool, trait_list);
}

senie_genotype_list* genotype_list_get_from_pool() {
  senie_genotype_list* genotype_list = genotype_list_pool_get(g_genotype_list_pool);

  return genotype_list;
}

void genotype_list_return_to_pool(senie_genotype_list* genotype_list) {
  genotype_list_cleanup(genotype_list);
  genotype_list_pool_return(g_genotype_list_pool, genotype_list);
}

void ga_subsystem_startup() {
  // create 1 slab
  // each slab contains 200 genes
  // max of 10 slabs can be allocated
  g_gene_pool          = gene_pool_allocate(1, 200, 10);
  g_trait_pool         = trait_pool_allocate(1, 200, 10);
  g_genotype_pool      = genotype_pool_allocate(1, 200, 10);
  g_trait_list_pool    = trait_list_pool_allocate(1, 200, 10);
  g_genotype_list_pool = genotype_list_pool_allocate(1, 200, 10);

  g_ga_vm  = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES);
  g_ga_env = env_allocate();

  // global decl of program that's used for evaling initial values of traits
  senie_compiler_config compiler_config;
  compiler_config.program_max_size = MAX_PROGRAM_SIZE;
  compiler_config.word_lut         = NULL;
  g_ga_program                     = program_construct(&compiler_config);
}

void ga_subsystem_shutdown() {
  program_free(g_ga_program);
  env_free(g_ga_env);
  vm_free(g_ga_vm);

  genotype_list_pool_free(g_genotype_list_pool);
  trait_list_pool_free(g_trait_list_pool);
  genotype_pool_free(g_genotype_pool);
  trait_pool_free(g_trait_pool);
  gene_pool_free(g_gene_pool);
}

bool trait_serialize(senie_cursor* cursor, senie_trait* trait) {
  cursor_sprintf(cursor, "%d", trait->id);
  cursor_sprintf(cursor, " ");

  var_serialize(cursor, trait->initial_value);
  cursor_sprintf(cursor, " ");

  program_serialize(cursor, trait->program);

  return true;
}

bool trait_deserialize(senie_trait* out, senie_cursor* cursor) {
  out->id = cursor_eat_i32(cursor);
  cursor_eat_space(cursor);

  var_deserialize(out->initial_value, cursor);
  cursor_eat_space(cursor);

  if (out->program != NULL) {
    free(out->program);
  }
  out->program = program_allocate(0);
  program_deserialize(out->program, cursor);

  return true;
}

void trait_list_add_trait(senie_trait_list* trait_list, senie_trait* trait) {
  DL_APPEND(trait_list->traits, trait);
}

senie_var* compile_and_execute_node(senie_node* node) {

  program_reset(g_ga_program);
  senie_program* program = g_ga_program;

  // only compile this node, not any of the neighbouring ASTs
  senie_node* next = node->next;
  node->next       = NULL;
  program          = compile_program(program, node);
  node->next       = next;

  env_reset(g_ga_env);
  vm_reset(g_ga_vm);

  vm_run(g_ga_vm, g_ga_env, program);

  senie_var* result = vm_stack_peek(g_ga_vm);

  return result;
}

senie_trait*
trait_build(senie_node* node, senie_node* parameter_ast, senie_compiler_config* compiler_config) {
  senie_trait* trait = trait_get_from_pool();

  senie_var* res = compile_and_execute_node(node);
  var_copy(trait->initial_value, res);

  // NOTE: this is allocating memory for program
  senie_program* program = program_construct(compiler_config);
  trait->program = compile_program_for_trait(program, parameter_ast, node, compiler_config->vary);

  return trait;
}

void add_single_trait(senie_trait_list*      trait_list,
                      senie_node*            node,
                      senie_compiler_config* compiler_config) {
  senie_trait* trait = trait_build(node, node->parameter_ast, compiler_config);
  trait_list_add_trait(trait_list, trait);
}

void add_multiple_traits(senie_trait_list*      trait_list,
                         senie_node*            node,
                         senie_compiler_config* compiler_config) {
  senie_node* vector = node;
  senie_node* n      = safe_first(node->value.first_child);

  while (n) {
    senie_trait* trait = trait_build(n, vector->parameter_ast, compiler_config);
    trait_list_add_trait(trait_list, trait);

    n = safe_next(n);
  }
}

senie_node* ga_traverse(senie_node*            node,
                        senie_trait_list*      trait_list,
                        senie_compiler_config* compiler_config) {
  senie_node* n = node;

  if (n->alterable) {
    if (n->type == NODE_VECTOR) {
      add_multiple_traits(trait_list, n, compiler_config);
    } else {
      add_single_trait(trait_list, n, compiler_config);
    }
  }

  if (n->type == NODE_LIST || n->type == NODE_VECTOR) {
    n = n->value.first_child;

    while (n != NULL) {
      ga_traverse(n, trait_list, compiler_config);
      n = safe_next(n);
    }
  }

  return safe_next(node);
}

senie_trait_list* trait_list_compile(senie_node* ast, senie_compiler_config* compiler_config) {
  // iterate through and build some traits
  senie_trait_list* trait_list = trait_list_get_from_pool();

  g_ga_program->word_lut = compiler_config->word_lut;

  senie_node* n = ast;
  while (n != NULL) {
    n = ga_traverse(n, trait_list, compiler_config);
  }

  return trait_list;
}

i32 trait_list_count(senie_trait_list* trait_list) {
  senie_trait* t     = trait_list->traits;
  i32          count = 0;

  while (t != NULL) {
    count++;
    t = t->next;
  }

  return count;
}

bool trait_list_serialize(senie_cursor* cursor, senie_trait_list* trait_list) {
  // seed value
  cursor_sprintf(cursor, "%d", trait_list->seed_value);
  cursor_sprintf(cursor, " ");

  // number of traits
  i32 count = trait_list_count(trait_list);
  cursor_sprintf(cursor, "%d", count);
  cursor_sprintf(cursor, " ");

  // sequence of traits
  senie_trait* t = trait_list->traits;
  while (t != NULL) {
    trait_serialize(cursor, t);
    if (t->next != NULL) {
      cursor_sprintf(cursor, " ");
    }
    t = t->next;
  }

  return true;
}

bool trait_list_deserialize(senie_trait_list* out, senie_cursor* cursor) {
  senie_trait_list* trait_list = out;

  i32 seed_value = cursor_eat_i32(cursor);
  cursor_eat_space(cursor);
  trait_list->seed_value = seed_value;

  i32 count = cursor_eat_i32(cursor);
  cursor_eat_space(cursor);

  for (i32 i = 0; i < count; i++) {
    senie_trait* trait = trait_get_from_pool();
    trait_deserialize(trait, cursor);
    trait_list_add_trait(trait_list, trait);
    if (i < count - 1) {
      cursor_eat_space(cursor);
    }
  }

  return true;
}

// gene

senie_gene* gene_build_from_program(senie_vm* vm, senie_env* env, senie_program* program) {
  // todo: possibly implement a 'soft-reset' which is quicker than a vm_reset?
  vm_reset(vm);

  bool res = vm_run(vm, env, program);
  if (res == false) {
    return NULL;
  }

  senie_gene* gene = gene_get_from_pool();

  var_copy(gene->var, &(vm->stack[vm->sp - 1]));

  return gene;
}

senie_gene* gene_build_from_initial_value(senie_var* initial_value) {
  senie_gene* gene = gene_get_from_pool();

  var_copy(gene->var, initial_value);

  return gene;
}

senie_gene* gene_clone(senie_gene* source) {
  senie_gene* gene = gene_get_from_pool();

  var_copy(gene->var, source->var);

  return gene;
}

// genotype

void genotype_add_gene(senie_genotype* genotype, senie_gene* gene) {
  DL_APPEND(genotype->genes, gene);
}

senie_genotype*
genotype_build_from_program(senie_trait_list* trait_list, senie_vm* vm, senie_env* env, i32 seed) {
  // the seed is set once per genotype (should it be once per-gene?)
  //
  senie_prng_set_state(vm->prng_state, (u64)seed);

  senie_genotype* genotype = genotype_get_from_pool();

  senie_trait* trait = trait_list->traits;
  while (trait != NULL) {
    senie_gene* gene = gene_build_from_program(vm, env, trait->program);

    if (gene != NULL) {
      genotype_add_gene(genotype, gene);
    } else {
      SENIE_ERROR("gene_build_from_program returned NULL gene");
      genotype_return_to_pool(genotype);
      return NULL;
    }

    trait = trait->next;
  }

  return genotype;
}

senie_genotype* genotype_build_from_initial_values(senie_trait_list* trait_list) {
  senie_genotype* genotype = genotype_get_from_pool();

  senie_trait* trait = trait_list->traits;
  while (trait != NULL) {
    senie_gene* gene = gene_build_from_initial_value(trait->initial_value);

    if (gene != NULL) {
      genotype_add_gene(genotype, gene);
    } else {
      SENIE_ERROR("gene_build_from_initial_value returned NULL gene");
      genotype_return_to_pool(genotype);
      return NULL;
    }

    trait = trait->next;
  }

  return genotype;
}

i32 genotype_count(senie_genotype* genotype) {
  senie_gene* g     = genotype->genes;
  i32         count = 0;

  while (g != NULL) {
    count++;
    g = g->next;
  }

  return count;
}

senie_genotype* genotype_clone(senie_genotype* genotype) {
  senie_genotype* cloned_genotype = genotype_get_from_pool();

  senie_gene* src_gene = genotype->genes;

  while (src_gene) {
    senie_gene* cloned_gene = gene_clone(src_gene);
    genotype_add_gene(cloned_genotype, cloned_gene);

    src_gene = src_gene->next;
  }

  return cloned_genotype;
}

senie_genotype*
genotype_crossover(senie_genotype* a, senie_genotype* b, i32 crossover_index, i32 genotype_length) {
  senie_genotype* genotype = genotype_get_from_pool();
  senie_gene*     gene;
  senie_gene*     gene_a = a->genes;
  senie_gene*     gene_b = b->genes;
  i32             i;

  for (i = 0; i < crossover_index; i++) {
    gene = gene_clone(gene_a);
    genotype_add_gene(genotype, gene);
    gene_a = gene_a->next;
    gene_b = gene_b->next; // keep gene_b in sync
  }

  for (i = crossover_index; i < genotype_length; i++) {
    gene = gene_clone(gene_b);
    genotype_add_gene(genotype, gene);
    gene_b = gene_b->next;
  }

  return genotype;
}

bool genotype_serialize(senie_cursor* cursor, senie_genotype* genotype) {
  // number of genes
  i32 count = genotype_count(genotype);
  cursor_sprintf(cursor, "%d", count);
  cursor_sprintf(cursor, " ");

  senie_gene* gene = genotype->genes;
  while (gene != NULL) {
    var_serialize(cursor, gene->var);
    if (gene->next != NULL) {
      cursor_sprintf(cursor, " ");
    }
    gene = gene->next;
  }

  return true;
}

bool genotype_deserialize(senie_genotype* out, senie_cursor* cursor) {
  i32 count = cursor_eat_i32(cursor);
  cursor_eat_space(cursor);

  senie_genotype* genotype = out;

  for (i32 i = 0; i < count; i++) {
    senie_gene* gene = gene_get_from_pool();
    var_deserialize(gene->var, cursor);
    genotype_add_gene(genotype, gene);
    if (i < count - 1) {
      cursor_eat_space(cursor);
    }
  }

  return true;
}

senie_gene* genotype_pull_gene(senie_genotype* genotype) {
  senie_gene* gene = genotype->current_gene;
  RETURN_IF_NULL(gene, "genotype_pull_gene: current gene is null");

  genotype->current_gene = genotype->current_gene->next;

  return gene;
}

void genotype_list_add_genotype(senie_genotype_list* genotype_list, senie_genotype* genotype) {
  DL_APPEND(genotype_list->genotypes, genotype);
}

senie_genotype* genotype_list_get_genotype(senie_genotype_list* genotype_list, i32 index) {
  senie_genotype* genotype = genotype_list->genotypes;
  i32             i        = 0;

  while (i < index) {
    if (genotype == NULL) {
      return NULL;
    }
    genotype = genotype->next;
    i++;
  }

  return genotype;
}

i32 genotype_list_count(senie_genotype_list* genotype_list) {
  senie_genotype* g     = genotype_list->genotypes;
  i32             count = 0;

  while (g != NULL) {
    count++;
    g = g->next;
  }

  return count;
}

bool genotype_list_serialize(senie_cursor* cursor, senie_genotype_list* genotype_list) {
  // number of genotypes
  i32 count = genotype_list_count(genotype_list);
  cursor_sprintf(cursor, "%d", count);
  cursor_sprintf(cursor, " ");

  // sequence of genotypes
  senie_genotype* g = genotype_list->genotypes;
  while (g != NULL) {
    genotype_serialize(cursor, g);
    if (g->next != NULL) {
      cursor_sprintf(cursor, " ");
    }
    g = g->next;
  }

  return true;
}

bool genotype_list_deserialize(senie_genotype_list* out, senie_cursor* cursor) {
  i32 count = cursor_eat_i32(cursor);
  cursor_eat_space(cursor);

  senie_genotype_list* genotype_list = out;

  for (i32 i = 0; i < count; i++) {
    senie_genotype* genotype = genotype_get_from_pool();
    genotype_deserialize(genotype, cursor);
    genotype_list_add_genotype(genotype_list, genotype);
    if (i < count - 1) {
      cursor_eat_space(cursor);
    }
  }

  return true;
}

senie_genotype_list* genotype_list_create_single_genotype(senie_trait_list* trait_list, i32 seed) {
  senie_genotype_list* genotype_list = genotype_list_get_from_pool();

  SENIE_LOG("genotype_list_create_single_genotype seed: %d", seed);

  // fill out the remaining population with generated values
  senie_vm*  vm  = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES);
  senie_env* env = env_allocate();

  senie_genotype* genotype = genotype_build_from_program(trait_list, vm, env, seed);
  genotype_list_add_genotype(genotype_list, genotype);

  env_free(env);
  vm_free(vm);

  return genotype_list;
}

senie_genotype_list* genotype_list_create_initial_generation(senie_trait_list* trait_list,
                                                             i32               population_size,
                                                             i32               seed) {
  senie_genotype_list* genotype_list = genotype_list_get_from_pool();
  if (population_size == 0) {
    SENIE_ERROR("genotype_list_create_initial_generation: population_size of 0 ???");
    return genotype_list;
  }

  // create a genotype using the initial values from the traits
  senie_genotype* genotype = genotype_build_from_initial_values(trait_list);
  genotype_list_add_genotype(genotype_list, genotype);

  SENIE_LOG("genotype_list_create_initial_generation seed: %d", seed);
  senie_prng_state prng_state;
  senie_prng_set_state(&prng_state, (u64)seed);
  i32 prng_min = 1 << 0;
  i32 prng_max = 1 << 16;
  i32 genotype_seed;

  // fill out the remaining population with generated values
  senie_vm*  vm  = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES);
  senie_env* env = env_allocate();

  for (i32 i = 1; i < population_size; i++) {
    genotype_seed = senie_prng_i32_range(&prng_state, prng_min, prng_max);
    SENIE_LOG("%d genotype_seed %d", i, genotype_seed);
    genotype = genotype_build_from_program(trait_list, vm, env, genotype_seed);
    genotype_list_add_genotype(genotype_list, genotype);
  }

  env_free(env);
  vm_free(vm);

  return genotype_list;
}

// mutates the gene and the prng_state
void gene_generate_new_var(senie_gene* gene, senie_trait* trait, senie_prng_state* prng_state) {
  senie_vm*  vm  = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES);
  senie_env* env = env_allocate();

  senie_prng_copy(vm->prng_state, prng_state);

  bool res = vm_run(vm, env, trait->program);
  if (res == false) {
    SENIE_ERROR("gene_generate_new_var: vm_interpret returned false");
    return;
  }

  var_copy(gene->var, &(vm->stack[vm->sp - 1]));

  // now update the prng_state
  senie_prng_copy(prng_state, vm->prng_state);
  env_free(env);
  vm_free(vm);
}

senie_genotype* genotype_possibly_mutate(senie_genotype*   genotype,
                                         i32               genotype_length,
                                         f32               mutation_rate,
                                         senie_prng_state* prng_state,
                                         senie_trait_list* trait_list) {
  senie_gene*  gene     = genotype->genes;
  senie_trait* trait    = trait_list->traits;
  f32          prng_num = 0.0f;

  for (i32 i = 0; i < genotype_length; i++) {
    prng_num = senie_prng_f32(prng_state);
    if (prng_num < mutation_rate) {
      // SENIE_LOG("we have a mutation! (index: %d mutation_rate: %.3f, random
      // number: %.3f)", i, mutation_rate, prng_num); replace the var in gene
      // with a newly created one
      gene_generate_new_var(gene, trait, prng_state);
    }

    gene  = gene->next;
    trait = trait->next;
  }

  return genotype;
}

senie_genotype_list* genotype_list_next_generation(senie_genotype_list* parents,
                                                   i32                  num_parents,
                                                   i32                  population_size,
                                                   f32                  mutation_rate,
                                                   i32                  rng,
                                                   senie_trait_list*    trait_list) {
  senie_genotype_list* genotype_list = genotype_list_get_from_pool();

  i32 population_remaining = population_size;

  // copy the parents onto the new generation
  senie_genotype* genotype;
  senie_genotype* parent_genotype = parents->genotypes;
  while (parent_genotype) {
    genotype = genotype_clone(parent_genotype);
    genotype_list_add_genotype(genotype_list, genotype);
    parent_genotype = parent_genotype->next;
    population_remaining--;
  }
  genotype = NULL; // going to re-use the genotype variable later

  senie_prng_state prng_state;
  senie_prng_set_state(&prng_state, (u64)rng);
  i32 retry_count = 10;

  while (population_remaining) {
    u32 a_index = senie_prng_i32_range(&prng_state, 0, num_parents - 1);
    u32 b_index = a_index;
    for (i32 retry = 0; retry < retry_count; retry++) {
      b_index = senie_prng_i32_range(&prng_state, 0, num_parents - 1);
      if (b_index != a_index) {
        break;
      }
    }
    if (b_index == a_index) {
      b_index = (a_index + 1) % num_parents;
    }

    senie_genotype* a = genotype_list_get_genotype(parents, a_index);
    senie_genotype* b = genotype_list_get_genotype(parents, b_index);

    i32 genotype_length = genotype_count(a);
    i32 crossover_index = senie_prng_i32_range(&prng_state, 0, genotype_length - 1);

    genotype = genotype_crossover(a, b, crossover_index, genotype_length);
    genotype =
        genotype_possibly_mutate(genotype, genotype_length, mutation_rate, &prng_state, trait_list);

    genotype_list_add_genotype(genotype_list, genotype);

    population_remaining--;
  }

  return genotype_list;
}
