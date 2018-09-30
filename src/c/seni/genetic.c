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
sen_vm*      g_ga_vm;
sen_env*     g_ga_env;
sen_program* g_ga_program;

void gene_return_to_pool(sen_gene* gene);
void trait_return_to_pool(sen_trait* trait);

// setup pools

void gene_cleanup(sen_gene* gene) {
  if (gene->var) {
    var_return_to_pool(gene->var);
    gene->var = NULL;
  }
}

void trait_cleanup(sen_trait* trait) {
  if (trait->program) {
    program_free(trait->program);
    trait->program = NULL;
  }

  if (trait->initial_value) {
    var_return_to_pool(trait->initial_value);
    trait->initial_value = NULL;
  }
}

void genotype_cleanup(sen_genotype* genotype) {
  sen_gene* g = genotype->genes;
  sen_gene* next;
  while (g != NULL) {
    next = g->next;
    DL_DELETE(genotype->genes, g);
    gene_return_to_pool(g);
    g = next;
  }
  genotype->genes        = NULL;
  genotype->current_gene = NULL;
}

void trait_list_cleanup(sen_trait_list* trait_list) {
  sen_trait* t = trait_list->traits;
  sen_trait* next;
  while (t != NULL) {
    next = t->next;
    DL_DELETE(trait_list->traits, t);

    trait_return_to_pool(t);
    t = next;
  }

  trait_list->traits = NULL;
}

void genotype_list_cleanup(sen_genotype_list* genotype_list) {
  // todo: test this
  sen_genotype* g = genotype_list->genotypes;
  sen_genotype* next;
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

SEN_POOL(sen_gene, gene)
SEN_POOL(sen_trait, trait)
SEN_POOL(sen_genotype, genotype)
SEN_POOL(sen_trait_list, trait_list)
SEN_POOL(sen_genotype_list, genotype_list)

struct sen_gene_pool*          g_gene_pool;
struct sen_trait_pool*         g_trait_pool;
struct sen_genotype_pool*      g_genotype_pool;
struct sen_trait_list_pool*    g_trait_list_pool;
struct sen_genotype_list_pool* g_genotype_list_pool;

sen_gene* gene_get_from_pool() {
  sen_gene* gene = gene_pool_get(g_gene_pool);

  sen_var* var = var_get_from_pool();
  gene->var    = var;

  return gene;
}

void gene_return_to_pool(sen_gene* gene) {
  gene_cleanup(gene);
  gene_pool_return(g_gene_pool, gene);
}

sen_trait* trait_get_from_pool() {
  sen_trait* trait = trait_pool_get(g_trait_pool);

  // get a var from the pool in sen_lang
  sen_var* var         = var_get_from_pool();
  trait->initial_value = var;

  return trait;
}

void trait_return_to_pool(sen_trait* trait) {
  // free up any memory used to store the program and initial_value
  trait_cleanup(trait);
  trait_pool_return(g_trait_pool, trait);
}

sen_genotype* genotype_get_from_pool() {
  sen_genotype* genotype = genotype_pool_get(g_genotype_pool);

  return genotype;
}

void genotype_return_to_pool(sen_genotype* genotype) {
  genotype_cleanup(genotype);
  genotype_pool_return(g_genotype_pool, genotype);
}

sen_trait_list* trait_list_get_from_pool() {
  sen_trait_list* trait_list = trait_list_pool_get(g_trait_list_pool);

  return trait_list;
}

void trait_list_return_to_pool(sen_trait_list* trait_list) {
  trait_list_cleanup(trait_list);
  trait_list_pool_return(g_trait_list_pool, trait_list);
}

sen_genotype_list* genotype_list_get_from_pool() {
  sen_genotype_list* genotype_list =
      genotype_list_pool_get(g_genotype_list_pool);

  return genotype_list;
}

void genotype_list_return_to_pool(sen_genotype_list* genotype_list) {
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

  g_ga_vm  = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE,
                        VERTEX_PACKET_NUM_VERTICES);
  g_ga_env = env_allocate();

  // global decl of program that's used for evaling initial values of traits
  sen_compiler_config compiler_config;
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

bool trait_serialize(sen_cursor* cursor, sen_trait* trait) {
  cursor_sprintf(cursor, "%d", trait->id);
  cursor_sprintf(cursor, " ");

  cursor_sprintf(cursor, "%d", trait->within_vector);
  cursor_sprintf(cursor, " ");

  cursor_sprintf(cursor, "%d", trait->index);
  cursor_sprintf(cursor, " ");

  var_serialize(cursor, trait->initial_value);
  cursor_sprintf(cursor, " ");

  program_serialize(cursor, trait->program);

  return true;
}

bool trait_deserialize(sen_trait* out, sen_cursor* cursor) {
  out->id = cursor_eat_i32(cursor);
  cursor_eat_space(cursor);

  out->within_vector = cursor_eat_i32(cursor);
  cursor_eat_space(cursor);

  out->index = cursor_eat_i32(cursor);
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

void trait_list_add_trait(sen_trait_list* trait_list, sen_trait* trait) {
  DL_APPEND(trait_list->traits, trait);
}

sen_var* compile_and_execute_node(sen_node* node) {

  program_reset(g_ga_program);
  sen_program* program = g_ga_program;

  // only compile this node, not any of the neighbouring ASTs
  sen_node* next = node->next;
  node->next     = NULL;
  program        = compile_program(program, node);
  node->next     = next;

  env_reset(g_ga_env);
  vm_reset(g_ga_vm);

  vm_run(g_ga_vm, g_ga_env, program);

  sen_var* result = vm_stack_peek(g_ga_vm);

  return result;
}

sen_trait* trait_build(sen_node* node, sen_node* parameter_ast,
                       sen_compiler_config* compiler_config) {
  sen_trait* trait = trait_get_from_pool();

  sen_var* res = compile_and_execute_node(node);
  var_copy(trait->initial_value, res);

  // NOTE: this is allocating memory for program
  sen_program* program = program_construct(compiler_config);
  trait->program = compile_program_for_trait(program, parameter_ast, node);

  return trait;
}

void add_single_trait(sen_trait_list* trait_list, sen_node* node,
                      sen_compiler_config* compiler_config) {
  sen_trait* trait = trait_build(node, node->parameter_ast, compiler_config);
  trait->within_vector = 0;
  trait->index         = 0;
  trait_list_add_trait(trait_list, trait);
}

void add_multiple_traits(sen_trait_list* trait_list, sen_node* node,
                         sen_compiler_config* compiler_config) {
  sen_node* vector = node;
  sen_node* n      = safe_first(node->value.first_child);

  i32 i = 0;

  while (n) {
    sen_trait* trait = trait_build(n, vector->parameter_ast, compiler_config);
    trait->within_vector = 1;
    trait->index         = i++;
    trait_list_add_trait(trait_list, trait);

    n = safe_next(n);
  }
}

bool is_word_match(char* word, char* name) {
  char* wp = word;
  char* np = name;

  while (*wp && *np) {
    if (*wp++ != *np++) {
      return false;
    }
  }

  // both word and name should have reached their end
  if (*wp == 0 && *np == 0) {
    return true;
  }

  return false;
}

sen_node* ga_traverse(sen_node* node, sen_trait_list* trait_list,
                      sen_compiler_config* compiler_config) {
  sen_node* n = node;

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

sen_trait_list* trait_list_compile(sen_node*            ast,
                                   sen_compiler_config* compiler_config) {
  // iterate through and build some traits
  sen_trait_list* trait_list = trait_list_get_from_pool();

  g_ga_program->word_lut = compiler_config->word_lut;

  sen_node* n = ast;
  while (n != NULL) {
    n = ga_traverse(n, trait_list, compiler_config);
  }

  return trait_list;
}

i32 trait_list_count(sen_trait_list* trait_list) {
  sen_trait* t     = trait_list->traits;
  i32        count = 0;

  while (t != NULL) {
    count++;
    t = t->next;
  }

  return count;
}

bool trait_list_serialize(sen_cursor* cursor, sen_trait_list* trait_list) {
  // seed value
  cursor_sprintf(cursor, "%d", trait_list->seed_value);
  cursor_sprintf(cursor, " ");

  // number of traits
  i32 count = trait_list_count(trait_list);
  cursor_sprintf(cursor, "%d", count);
  cursor_sprintf(cursor, " ");

  // sequence of traits
  sen_trait* t = trait_list->traits;
  while (t != NULL) {
    trait_serialize(cursor, t);
    if (t->next != NULL) {
      cursor_sprintf(cursor, " ");
    }
    t = t->next;
  }

  return true;
}

bool trait_list_deserialize(sen_trait_list* out, sen_cursor* cursor) {
  sen_trait_list* trait_list = out;

  i32 seed_value = cursor_eat_i32(cursor);
  cursor_eat_space(cursor);
  trait_list->seed_value = seed_value;

  i32 count = cursor_eat_i32(cursor);
  cursor_eat_space(cursor);

  for (i32 i = 0; i < count; i++) {
    sen_trait* trait = trait_get_from_pool();
    trait_deserialize(trait, cursor);
    trait_list_add_trait(trait_list, trait);
    if (i < count - 1) {
      cursor_eat_space(cursor);
    }
  }

  return true;
}

// gene

sen_gene* gene_build_from_trait(sen_vm* vm, sen_env* env, sen_trait* trait) {
  sen_program* program = trait->program;

  // todo: possibly implement a 'soft-reset' which is quicker than a vm_reset?
  vm_reset(vm);

  vm->building_with_trait_within_vector = trait->within_vector;
  vm->trait_within_vector_index         = trait->index;

  bool res = vm_run(vm, env, program);
  if (res == false) {
    return NULL;
  }

  vm->building_with_trait_within_vector = 0;
  vm->trait_within_vector_index         = 0;

  sen_gene* gene = gene_get_from_pool();

  var_copy(gene->var, &(vm->stack[vm->sp - 1]));

  return gene;
}

sen_gene* gene_build_from_initial_value(sen_var* initial_value) {
  sen_gene* gene = gene_get_from_pool();

  var_copy(gene->var, initial_value);

  return gene;
}

sen_gene* gene_clone(sen_gene* source) {
  sen_gene* gene = gene_get_from_pool();

  var_copy(gene->var, source->var);

  return gene;
}

// genotype

void genotype_add_gene(sen_genotype* genotype, sen_gene* gene) {
  DL_APPEND(genotype->genes, gene);
}

sen_genotype* genotype_build_from_trait_list(sen_trait_list* trait_list,
                                             sen_vm* vm, sen_env* env,
                                             i32 seed) {
  // the seed is set once per genotype (should it be once per-gene?)
  //
  sen_prng_set_state(vm->prng_state, (u64)seed);

  sen_genotype* genotype = genotype_get_from_pool();

  sen_trait* trait = trait_list->traits;
  while (trait != NULL) {
    sen_gene* gene = gene_build_from_trait(vm, env, trait);

    if (gene != NULL) {
      genotype_add_gene(genotype, gene);
    } else {
      SEN_ERROR("gene_build_from_trait returned NULL gene");
      genotype_return_to_pool(genotype);
      return NULL;
    }

    trait = trait->next;
  }

  return genotype;
}

sen_genotype* genotype_build_from_initial_values(sen_trait_list* trait_list) {
  sen_genotype* genotype = genotype_get_from_pool();

  sen_trait* trait = trait_list->traits;
  while (trait != NULL) {
    sen_gene* gene = gene_build_from_initial_value(trait->initial_value);

    if (gene != NULL) {
      genotype_add_gene(genotype, gene);
    } else {
      SEN_ERROR("gene_build_from_initial_value returned NULL gene");
      genotype_return_to_pool(genotype);
      return NULL;
    }

    trait = trait->next;
  }

  return genotype;
}

i32 genotype_count(sen_genotype* genotype) {
  sen_gene* g     = genotype->genes;
  i32       count = 0;

  while (g != NULL) {
    count++;
    g = g->next;
  }

  return count;
}

sen_genotype* genotype_clone(sen_genotype* genotype) {
  sen_genotype* cloned_genotype = genotype_get_from_pool();

  sen_gene* src_gene = genotype->genes;

  while (src_gene) {
    sen_gene* cloned_gene = gene_clone(src_gene);
    genotype_add_gene(cloned_genotype, cloned_gene);

    src_gene = src_gene->next;
  }

  return cloned_genotype;
}

sen_genotype* genotype_crossover(sen_genotype* a, sen_genotype* b,
                                 i32 crossover_index, i32 genotype_length) {
  sen_genotype* genotype = genotype_get_from_pool();
  sen_gene*     gene;
  sen_gene*     gene_a = a->genes;
  sen_gene*     gene_b = b->genes;
  i32           i;

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

bool genotype_serialize(sen_cursor* cursor, sen_genotype* genotype) {
  // number of genes
  i32 count = genotype_count(genotype);
  cursor_sprintf(cursor, "%d", count);
  cursor_sprintf(cursor, " ");

  sen_gene* gene = genotype->genes;
  while (gene != NULL) {
    var_serialize(cursor, gene->var);
    if (gene->next != NULL) {
      cursor_sprintf(cursor, " ");
    }
    gene = gene->next;
  }

  return true;
}

bool genotype_deserialize(sen_genotype* out, sen_cursor* cursor) {
  i32 count = cursor_eat_i32(cursor);
  cursor_eat_space(cursor);

  sen_genotype* genotype = out;

  for (i32 i = 0; i < count; i++) {
    sen_gene* gene = gene_get_from_pool();
    var_deserialize(gene->var, cursor);
    genotype_add_gene(genotype, gene);
    if (i < count - 1) {
      cursor_eat_space(cursor);
    }
  }

  return true;
}

sen_gene* genotype_pull_gene(sen_genotype* genotype) {
  sen_gene* gene = genotype->current_gene;
  RETURN_IF_NULL(gene, "genotype_pull_gene: current gene is null");

  genotype->current_gene = genotype->current_gene->next;

  return gene;
}

void genotype_list_add_genotype(sen_genotype_list* genotype_list,
                                sen_genotype*      genotype) {
  DL_APPEND(genotype_list->genotypes, genotype);
}

sen_genotype* genotype_list_get_genotype(sen_genotype_list* genotype_list,
                                         i32                index) {
  sen_genotype* genotype = genotype_list->genotypes;
  i32           i        = 0;

  while (i < index) {
    if (genotype == NULL) {
      return NULL;
    }
    genotype = genotype->next;
    i++;
  }

  return genotype;
}

i32 genotype_list_count(sen_genotype_list* genotype_list) {
  sen_genotype* g     = genotype_list->genotypes;
  i32           count = 0;

  while (g != NULL) {
    count++;
    g = g->next;
  }

  return count;
}

bool genotype_list_serialize(sen_cursor*        cursor,
                             sen_genotype_list* genotype_list) {
  // number of genotypes
  i32 count = genotype_list_count(genotype_list);
  cursor_sprintf(cursor, "%d", count);
  cursor_sprintf(cursor, " ");

  // sequence of genotypes
  sen_genotype* g = genotype_list->genotypes;
  while (g != NULL) {
    genotype_serialize(cursor, g);
    if (g->next != NULL) {
      cursor_sprintf(cursor, " ");
    }
    g = g->next;
  }

  return true;
}

bool genotype_list_deserialize(sen_genotype_list* out, sen_cursor* cursor) {
  i32 count = cursor_eat_i32(cursor);
  cursor_eat_space(cursor);

  sen_genotype_list* genotype_list = out;

  for (i32 i = 0; i < count; i++) {
    sen_genotype* genotype = genotype_get_from_pool();
    genotype_deserialize(genotype, cursor);
    genotype_list_add_genotype(genotype_list, genotype);
    if (i < count - 1) {
      cursor_eat_space(cursor);
    }
  }

  return true;
}

sen_genotype_list*
genotype_list_create_single_genotype(sen_trait_list* trait_list, i32 seed) {
  sen_genotype_list* genotype_list = genotype_list_get_from_pool();

  // SEN_LOG("genotype_list_create_single_genotype seed: %d", seed);

  // fill out the remaining population with generated values
  sen_vm*  vm  = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE,
                           VERTEX_PACKET_NUM_VERTICES);
  sen_env* env = env_allocate();

  sen_genotype* genotype =
      genotype_build_from_trait_list(trait_list, vm, env, seed);
  genotype_list_add_genotype(genotype_list, genotype);

  env_free(env);
  vm_free(vm);

  return genotype_list;
}

sen_genotype_list*
genotype_list_create_initial_generation(sen_trait_list* trait_list,
                                        i32 population_size, i32 seed) {
  sen_genotype_list* genotype_list = genotype_list_get_from_pool();
  if (population_size == 0) {
    SEN_ERROR(
        "genotype_list_create_initial_generation: population_size of 0 ???");
    return genotype_list;
  }

  // create a genotype using the initial values from the traits
  sen_genotype* genotype = genotype_build_from_initial_values(trait_list);
  genotype_list_add_genotype(genotype_list, genotype);

  SEN_LOG("genotype_list_create_initial_generation seed: %d", seed);
  sen_prng_state prng_state;
  sen_prng_set_state(&prng_state, (u64)seed);
  i32 prng_min = 1 << 0;
  i32 prng_max = 1 << 16;
  i32 genotype_seed;

  // fill out the remaining population with generated values
  sen_vm*  vm  = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE,
                           VERTEX_PACKET_NUM_VERTICES);
  sen_env* env = env_allocate();

  for (i32 i = 1; i < population_size; i++) {
    genotype_seed = sen_prng_i32_range(&prng_state, prng_min, prng_max);
    SEN_LOG("%d genotype_seed %d", i, genotype_seed);
    genotype =
        genotype_build_from_trait_list(trait_list, vm, env, genotype_seed);
    genotype_list_add_genotype(genotype_list, genotype);
  }

  env_free(env);
  vm_free(vm);

  return genotype_list;
}

// mutates the gene and the prng_state
void gene_generate_new_var(sen_gene* gene, sen_trait* trait,
                           sen_prng_state* prng_state) {
  sen_vm*  vm  = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE,
                           VERTEX_PACKET_NUM_VERTICES);
  sen_env* env = env_allocate();

  sen_prng_copy(vm->prng_state, prng_state);

  bool res = vm_run(vm, env, trait->program);
  if (res == false) {
    SEN_ERROR("gene_generate_new_var: vm_interpret returned false");
    return;
  }

  var_copy(gene->var, &(vm->stack[vm->sp - 1]));

  // now update the prng_state
  sen_prng_copy(prng_state, vm->prng_state);
  env_free(env);
  vm_free(vm);
}

sen_genotype* genotype_possibly_mutate(sen_genotype* genotype,
                                       i32 genotype_length, f32 mutation_rate,
                                       sen_prng_state* prng_state,
                                       sen_trait_list* trait_list) {
  sen_gene*  gene     = genotype->genes;
  sen_trait* trait    = trait_list->traits;
  f32        prng_num = 0.0f;

  for (i32 i = 0; i < genotype_length; i++) {
    prng_num = sen_prng_f32(prng_state);
    if (prng_num < mutation_rate) {
      // SEN_LOG("we have a mutation! (index: %d mutation_rate: %.3f, random
      // number: %.3f)", i, mutation_rate, prng_num); replace the var in gene
      // with a newly created one
      gene_generate_new_var(gene, trait, prng_state);
    }

    gene  = gene->next;
    trait = trait->next;
  }

  return genotype;
}

sen_genotype_list* genotype_list_next_generation(sen_genotype_list* parents,
                                                 i32                num_parents,
                                                 i32 population_size,
                                                 f32 mutation_rate, i32 rng,
                                                 sen_trait_list* trait_list) {
  sen_genotype_list* genotype_list = genotype_list_get_from_pool();

  i32 population_remaining = population_size;

  // copy the parents onto the new generation
  sen_genotype* genotype;
  sen_genotype* parent_genotype = parents->genotypes;
  while (parent_genotype) {
    genotype = genotype_clone(parent_genotype);
    genotype_list_add_genotype(genotype_list, genotype);
    parent_genotype = parent_genotype->next;
    population_remaining--;
  }
  genotype = NULL; // going to re-use the genotype variable later

  sen_prng_state prng_state;
  sen_prng_set_state(&prng_state, (u64)rng);
  i32 retry_count = 10;

  while (population_remaining) {
    u32 a_index = sen_prng_i32_range(&prng_state, 0, num_parents - 1);
    u32 b_index = a_index;
    for (i32 retry = 0; retry < retry_count; retry++) {
      b_index = sen_prng_i32_range(&prng_state, 0, num_parents - 1);
      if (b_index != a_index) {
        break;
      }
    }
    if (b_index == a_index) {
      b_index = (a_index + 1) % num_parents;
    }

    sen_genotype* a = genotype_list_get_genotype(parents, a_index);
    sen_genotype* b = genotype_list_get_genotype(parents, b_index);

    i32 genotype_length = genotype_count(a);
    i32 crossover_index =
        sen_prng_i32_range(&prng_state, 0, genotype_length - 1);

    genotype = genotype_crossover(a, b, crossover_index, genotype_length);
    genotype = genotype_possibly_mutate(genotype, genotype_length,
                                        mutation_rate, &prng_state, trait_list);

    genotype_list_add_genotype(genotype_list, genotype);

    population_remaining--;
  }

  return genotype_list;
}
