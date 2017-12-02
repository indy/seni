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

void gene_return_to_pool(seni_gene* gene);
void trait_return_to_pool(seni_trait* trait);

// setup pools

void gene_cleanup(seni_gene* gene) {
  if (gene->var) {
    var_return_to_pool(gene->var);
    gene->var = NULL;
  }
}

void trait_cleanup(seni_trait* trait) {
  if (trait->program) {
    program_free(trait->program);
    trait->program = NULL;
  }

  if (trait->initial_value) {
    var_return_to_pool(trait->initial_value);
    trait->initial_value = NULL;
  }
}

void genotype_cleanup(seni_genotype* genotype) {
  seni_gene* g = genotype->genes;
  seni_gene* next;
  while (g != NULL) {
    next = g->next;
    DL_DELETE(genotype->genes, g);
    gene_return_to_pool(g);
    g = next;
  }
  genotype->genes        = NULL;
  genotype->current_gene = NULL;
}

void trait_list_cleanup(seni_trait_list* trait_list) {
  seni_trait* t = trait_list->traits;
  seni_trait* next;
  while (t != NULL) {
    next = t->next;
    DL_DELETE(trait_list->traits, t);

    trait_return_to_pool(t);
    t = next;
  }

  trait_list->traits = NULL;
}

void genotype_list_cleanup(seni_genotype_list* genotype_list) {
  // todo: test this
  seni_genotype* g = genotype_list->genotypes;
  seni_genotype* next;
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

struct seni_gene_pool*          g_gene_pool;
struct seni_trait_pool*         g_trait_pool;
struct seni_genotype_pool*      g_genotype_pool;
struct seni_trait_list_pool*    g_trait_list_pool;
struct seni_genotype_list_pool* g_genotype_list_pool;

seni_gene* gene_get_from_pool() {
  seni_gene* gene = gene_pool_get(g_gene_pool);

  seni_var* var = var_get_from_pool();
  gene->var     = var;

  return gene;
}

void gene_return_to_pool(seni_gene* gene) {
  gene_cleanup(gene);
  gene_pool_return(g_gene_pool, gene);
}

seni_trait* trait_get_from_pool() {
  seni_trait* trait = trait_pool_get(g_trait_pool);

  // get a var from the pool in seni_lang
  seni_var* var        = var_get_from_pool();
  trait->initial_value = var;

  return trait;
}

void trait_return_to_pool(seni_trait* trait) {
  // free up any memory used to store the program and initial_value
  trait_cleanup(trait);
  trait_pool_return(g_trait_pool, trait);
}

seni_genotype* genotype_get_from_pool() {
  seni_genotype* genotype = genotype_pool_get(g_genotype_pool);

  return genotype;
}

void genotype_return_to_pool(seni_genotype* genotype) {
  genotype_cleanup(genotype);
  genotype_pool_return(g_genotype_pool, genotype);
}

seni_trait_list* trait_list_get_from_pool() {
  seni_trait_list* trait_list = trait_list_pool_get(g_trait_list_pool);

  return trait_list;
}

void trait_list_return_to_pool(seni_trait_list* trait_list) {
  trait_list_cleanup(trait_list);
  trait_list_pool_return(g_trait_list_pool, trait_list);
}

seni_genotype_list* genotype_list_get_from_pool() {
  seni_genotype_list* genotype_list = genotype_list_pool_get(g_genotype_list_pool);

  return genotype_list;
}

void genotype_list_return_to_pool(seni_genotype_list* genotype_list) {
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
}

void ga_subsystem_shutdown() {
  genotype_list_pool_free(g_genotype_list_pool);
  trait_list_pool_free(g_trait_list_pool);
  genotype_pool_free(g_genotype_pool);
  trait_pool_free(g_trait_pool);
  gene_pool_free(g_gene_pool);
}

bool trait_serialize(seni_cursor* cursor, seni_trait* trait) {
  cursor_sprintf(cursor, "%d", trait->id);
  cursor_sprintf(cursor, " ");

  var_serialize(cursor, trait->initial_value);
  cursor_sprintf(cursor, " ");

  program_serialize(cursor, trait->program);

  return true;
}

bool trait_deserialize(seni_trait* out, seni_cursor* cursor) {
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

void trait_list_add_trait(seni_trait_list* trait_list, seni_trait* trait) {
  DL_APPEND(trait_list->traits, trait);
}

// this is terrible and should be replaced with an interpreter asap
bool super_hacky_colour_parser(seni_var* out, seni_node* node) {
  // assume that node is pointing to a list like: (col/rgb r: 1 g: 0 b: 0.3
  // alpha: 0.3)

  // also assuming that the first colour constructor in seni_bind is col/rgb
  //
  i32 rgb_constructor_fn_index = get_colour_constructor_start();

  seni_node* constructor_fn_name_node = safe_first(node->value.first_child);
  i32        native_index             = constructor_fn_name_node->value.i - NATIVE_START;
  if (native_index != rgb_constructor_fn_index) {
    SENI_ERROR("super_hacky_colour_parser only works with col/rgb");
    return false;
  }

  f32 r, g, b;
  f32 alpha = 1.0f;
  r = g = b = 0.0f;

  seni_node* n = safe_next(constructor_fn_name_node);
  seni_node* val;

  while (n != NULL && n->type == NODE_LABEL) {
    val = safe_next(n);
    if (val == NULL) {
      break;
    }

    if (n->value.i == INAME_R) {
      r = val->value.f;
    }
    if (n->value.i == INAME_G) {
      g = val->value.f;
    }
    if (n->value.i == INAME_B) {
      b = val->value.f;
    }
    if (n->value.i == INAME_ALPHA) {
      alpha = val->value.f;
    }

    n = safe_next(val); // skip past the value
  }

  out->type         = VAR_COLOUR;
  out->value.i      = RGB;
  out->f32_array[0] = r;
  out->f32_array[1] = g;
  out->f32_array[2] = b;
  out->f32_array[3] = alpha;

  return true;
}

bool super_hacky_2d_vector_parser(seni_var* out, seni_node* node) {
  if (node->type != NODE_VECTOR) {
    return false;
  }

  f32 a, b;

  seni_node* n = safe_first(node->value.first_child);
  if (n == NULL || n->type != NODE_FLOAT) {
    return false;
  }
  a = n->value.f;

  n = safe_next(n);
  // second element can't be null
  if (n == NULL || n->type != NODE_FLOAT) {
    return false;
  }
  b = n->value.f;

  out->type         = VAR_2D;
  out->f32_array[0] = a;
  out->f32_array[1] = b;

  return true;
}

bool is_2d_vector(seni_node* node) {
  if (node->type != NODE_VECTOR) {
    return false;
  }

  seni_node* n = safe_first(node->value.first_child);
  // first element can't be null
  if (n == NULL) {
    return false;
  }
  n = safe_next(n);
  // second element can't be null
  if (n == NULL) {
    return false;
  }

  n = safe_next(n);
  // should be no third element, it has to be null
  if (n == NULL) {
    return true;
  }

  return false;
}

// TODO: could really do with a small interpreter here that parses the seni_node
// ast
//
bool hack_node_to_var(seni_var* out, seni_node* node) {
  switch (node->type) {
  case NODE_INT:
    out->type    = VAR_INT;
    out->value.i = node->value.i;
    break;
  case NODE_FLOAT:
    out->type    = VAR_FLOAT;
    out->value.f = node->value.f;
    break;
  case NODE_NAME:
    out->type    = VAR_NAME;
    out->value.i = node->value.i;
    break;
  case NODE_LIST:
    if (is_node_colour_constructor(node)) {
      if (super_hacky_colour_parser(out, node) == false) {
        return false;
      }
      break;
    } else {
      SENI_LOG("list that isn't a colour");
      return false;
    }
  case NODE_VECTOR:
    if (is_2d_vector(node)) {
      if (super_hacky_2d_vector_parser(out, node) == false) {
        return false;
      }
    } else {
      SENI_LOG("vector that isn't 2d");
      return false;
    }
    break;
  default:
    return false;
  }

  return true;
}

void add_trait(seni_node*       node,
               seni_node*       parameter_ast,
               i32              program_max_size,
               seni_trait_list* trait_list,
               seni_word_lut*   word_lut) {
  seni_trait* trait = trait_get_from_pool();

  if (hack_node_to_var(trait->initial_value, node) == false) {
    SENI_PRINT("hack_node_to_var failed");
    node_pretty_print("failed node", node, word_lut);
  }

  // can compile the parameter_ast
  trait->program = compile_program(parameter_ast, program_max_size, word_lut);

  trait_list_add_trait(trait_list, trait);
}

void add_single_trait(seni_node*       node,
                      i32              program_max_size,
                      seni_trait_list* trait_list,
                      seni_word_lut*   word_lut) {
  add_trait(node, node->parameter_ast, program_max_size, trait_list, word_lut);
}

void add_multiple_traits(seni_node*       node,
                         i32              program_max_size,
                         seni_trait_list* trait_list,
                         seni_word_lut*   word_lut) {
  seni_node* vector = node;
  seni_node* n      = safe_first(node->value.first_child);

  while (n) {
    add_trait(n, vector->parameter_ast, program_max_size, trait_list, word_lut);
    n = safe_next(n);
  }
}

seni_node* ga_traverse(seni_node*       node,
                       i32              program_max_size,
                       seni_trait_list* trait_list,
                       seni_word_lut*   word_lut) {
  seni_node* n = node;

  if (n->alterable) {
    if (n->type == NODE_VECTOR) {
      add_multiple_traits(n, program_max_size, trait_list, word_lut);
    } else {
      add_single_trait(n, program_max_size, trait_list, word_lut);
    }
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

seni_trait_list*
trait_list_compile(seni_node* ast, i32 trait_program_max_size, seni_word_lut* word_lut) {
  // iterate through and build some traits
  seni_trait_list* trait_list = trait_list_get_from_pool();

  seni_node* n = ast;
  while (n != NULL) {
    n = ga_traverse(n, trait_program_max_size, trait_list, word_lut);
  }

  return trait_list;
}

i32 trait_list_count(seni_trait_list* trait_list) {
  seni_trait* t     = trait_list->traits;
  i32         count = 0;

  while (t != NULL) {
    count++;
    t = t->next;
  }

  return count;
}

bool trait_list_serialize(seni_cursor* cursor, seni_trait_list* trait_list) {
  // seed value
  cursor_sprintf(cursor, "%d", trait_list->seed_value);
  cursor_sprintf(cursor, " ");

  // number of traits
  i32 count = trait_list_count(trait_list);
  cursor_sprintf(cursor, "%d", count);
  cursor_sprintf(cursor, " ");

  // sequence of traits
  seni_trait* t = trait_list->traits;
  while (t != NULL) {
    trait_serialize(cursor, t);
    if (t->next != NULL) {
      cursor_sprintf(cursor, " ");
    }
    t = t->next;
  }

  return true;
}

bool trait_list_deserialize(seni_trait_list* out, seni_cursor* cursor) {
  seni_trait_list* trait_list = out;

  i32 seed_value = cursor_eat_i32(cursor);
  cursor_eat_space(cursor);
  trait_list->seed_value = seed_value;

  i32 count = cursor_eat_i32(cursor);
  cursor_eat_space(cursor);

  for (i32 i = 0; i < count; i++) {
    seni_trait* trait = trait_get_from_pool();
    trait_deserialize(trait, cursor);
    trait_list_add_trait(trait_list, trait);
    if (i < count - 1) {
      cursor_eat_space(cursor);
    }
  }

  return true;
}

// gene

seni_gene* gene_build_from_program(seni_vm* vm, seni_env* env, seni_program* program) {
  // todo: possibly implement a 'soft-reset' which is quicker than a vm_reset?
  vm_reset(vm);

  bool res = vm_run(vm, env, program);
  if (res == false) {
    return NULL;
  }

  seni_gene* gene = gene_get_from_pool();

  var_copy(gene->var, &(vm->stack[vm->sp - 1]));

  return gene;
}

seni_gene* gene_build_from_initial_value(seni_var* initial_value) {
  seni_gene* gene = gene_get_from_pool();

  var_copy(gene->var, initial_value);

  return gene;
}

seni_gene* gene_clone(seni_gene* source) {
  seni_gene* gene = gene_get_from_pool();

  var_copy(gene->var, source->var);

  return gene;
}

// genotype

void genotype_add_gene(seni_genotype* genotype, seni_gene* gene) {
  DL_APPEND(genotype->genes, gene);
}

seni_genotype*
genotype_build_from_program(seni_trait_list* trait_list, seni_vm* vm, seni_env* env, i32 seed) {
  // the seed is set once per genotype (should it be once per-gene?)
  //
  seni_prng_set_state(vm->prng_state, (u64)seed);

  seni_genotype* genotype = genotype_get_from_pool();

  seni_trait* trait = trait_list->traits;
  while (trait != NULL) {
    seni_gene* gene = gene_build_from_program(vm, env, trait->program);

    if (gene != NULL) {
      genotype_add_gene(genotype, gene);
    } else {
      SENI_ERROR("gene_build_from_program returned NULL gene");
      genotype_return_to_pool(genotype);
      return NULL;
    }

    trait = trait->next;
  }

  return genotype;
}

seni_genotype* genotype_build_from_initial_values(seni_trait_list* trait_list) {
  seni_genotype* genotype = genotype_get_from_pool();

  seni_trait* trait = trait_list->traits;
  while (trait != NULL) {
    seni_gene* gene = gene_build_from_initial_value(trait->initial_value);

    if (gene != NULL) {
      genotype_add_gene(genotype, gene);
    } else {
      SENI_ERROR("gene_build_from_initial_value returned NULL gene");
      genotype_return_to_pool(genotype);
      return NULL;
    }

    trait = trait->next;
  }

  return genotype;
}

i32 genotype_count(seni_genotype* genotype) {
  seni_gene* g     = genotype->genes;
  i32        count = 0;

  while (g != NULL) {
    count++;
    g = g->next;
  }

  return count;
}

seni_genotype* genotype_clone(seni_genotype* genotype) {
  seni_genotype* cloned_genotype = genotype_get_from_pool();

  seni_gene* src_gene = genotype->genes;

  while (src_gene) {
    seni_gene* cloned_gene = gene_clone(src_gene);
    genotype_add_gene(cloned_genotype, cloned_gene);

    src_gene = src_gene->next;
  }

  return cloned_genotype;
}

seni_genotype*
genotype_crossover(seni_genotype* a, seni_genotype* b, i32 crossover_index, i32 genotype_length) {
  seni_genotype* genotype = genotype_get_from_pool();
  seni_gene*     gene;
  seni_gene*     gene_a = a->genes;
  seni_gene*     gene_b = b->genes;
  i32            i;

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

bool genotype_serialize(seni_cursor* cursor, seni_genotype* genotype) {
  // number of genes
  i32 count = genotype_count(genotype);
  cursor_sprintf(cursor, "%d", count);
  cursor_sprintf(cursor, " ");

  seni_gene* gene = genotype->genes;
  while (gene != NULL) {
    var_serialize(cursor, gene->var);
    if (gene->next != NULL) {
      cursor_sprintf(cursor, " ");
    }
    gene = gene->next;
  }

  return true;
}

bool genotype_deserialize(seni_genotype* out, seni_cursor* cursor) {
  i32 count = cursor_eat_i32(cursor);
  cursor_eat_space(cursor);

  seni_genotype* genotype = out;

  for (i32 i = 0; i < count; i++) {
    seni_gene* gene = gene_get_from_pool();
    var_deserialize(gene->var, cursor);
    genotype_add_gene(genotype, gene);
    if (i < count - 1) {
      cursor_eat_space(cursor);
    }
  }

  return true;
}

seni_gene* genotype_pull_gene(seni_genotype* genotype) {
  seni_gene* gene = genotype->current_gene;
  RETURN_IF_NULL(gene, "genotype_pull_gene: current gene is null");

  genotype->current_gene = genotype->current_gene->next;

  return gene;
}

void genotype_list_add_genotype(seni_genotype_list* genotype_list, seni_genotype* genotype) {
  DL_APPEND(genotype_list->genotypes, genotype);
}

seni_genotype* genotype_list_get_genotype(seni_genotype_list* genotype_list, i32 index) {
  seni_genotype* genotype = genotype_list->genotypes;
  i32            i        = 0;

  while (i < index) {
    if (genotype == NULL) {
      return NULL;
    }
    genotype = genotype->next;
    i++;
  }

  return genotype;
}

i32 genotype_list_count(seni_genotype_list* genotype_list) {
  seni_genotype* g     = genotype_list->genotypes;
  i32            count = 0;

  while (g != NULL) {
    count++;
    g = g->next;
  }

  return count;
}

bool genotype_list_serialize(seni_cursor* cursor, seni_genotype_list* genotype_list) {
  // number of genotypes
  i32 count = genotype_list_count(genotype_list);
  cursor_sprintf(cursor, "%d", count);
  cursor_sprintf(cursor, " ");

  // sequence of genotypes
  seni_genotype* g = genotype_list->genotypes;
  while (g != NULL) {
    genotype_serialize(cursor, g);
    if (g->next != NULL) {
      cursor_sprintf(cursor, " ");
    }
    g = g->next;
  }

  return true;
}

bool genotype_list_deserialize(seni_genotype_list* out, seni_cursor* cursor) {
  i32 count = cursor_eat_i32(cursor);
  cursor_eat_space(cursor);

  seni_genotype_list* genotype_list = out;

  for (i32 i = 0; i < count; i++) {
    seni_genotype* genotype = genotype_get_from_pool();
    genotype_deserialize(genotype, cursor);
    genotype_list_add_genotype(genotype_list, genotype);
    if (i < count - 1) {
      cursor_eat_space(cursor);
    }
  }

  return true;
}

seni_genotype_list* genotype_list_create_initial_generation(seni_trait_list* trait_list,
                                                            i32              population_size,
                                                            i32              seed) {
  seni_genotype_list* genotype_list = genotype_list_get_from_pool();
  if (population_size == 0) {
    SENI_ERROR("genotype_list_create_initial_generation: population_size of 0 ???");
    return genotype_list;
  }

  // create a genotype using the initial valued from the traits
  seni_genotype* genotype = genotype_build_from_initial_values(trait_list);
  genotype_list_add_genotype(genotype_list, genotype);

  SENI_LOG("genotype_list_create_initial_generation seed: %d", seed);
  seni_prng_state prng_state;
  seni_prng_set_state(&prng_state, (u64)seed);
  i32 prng_min = 1 << 0;
  i32 prng_max = 1 << 16;
  i32 genotype_seed;

  // fill out the remaining population with generated values
  seni_vm*  vm  = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES);
  seni_env* env = env_allocate();

  for (i32 i = 1; i < population_size; i++) {
    genotype_seed = seni_prng_i32_range(&prng_state, prng_min, prng_max);
    SENI_LOG("%d genotype_seed %d", i, genotype_seed);
    genotype = genotype_build_from_program(trait_list, vm, env, genotype_seed);
    genotype_list_add_genotype(genotype_list, genotype);
  }

  env_free(env);
  vm_free(vm);

  return genotype_list;
}

// mutates the gene and the prng_state
void gene_generate_new_var(seni_gene* gene, seni_trait* trait, seni_prng_state* prng_state) {
  seni_vm*  vm  = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES);
  seni_env* env = env_allocate();

  seni_prng_copy(vm->prng_state, prng_state);

  bool res = vm_run(vm, env, trait->program);
  if (res == false) {
    SENI_ERROR("gene_generate_new_var: vm_interpret returned false");
    return;
  }

  var_copy(gene->var, &(vm->stack[vm->sp - 1]));

  // now update the prng_state
  seni_prng_copy(prng_state, vm->prng_state);
  env_free(env);
  vm_free(vm);
}

seni_genotype* genotype_possibly_mutate(seni_genotype*   genotype,
                                        i32              genotype_length,
                                        f32              mutation_rate,
                                        seni_prng_state* prng_state,
                                        seni_trait_list* trait_list) {
  seni_gene*  gene     = genotype->genes;
  seni_trait* trait    = trait_list->traits;
  f32         prng_num = 0.0f;

  for (i32 i = 0; i < genotype_length; i++) {
    prng_num = seni_prng_f32(prng_state);
    if (prng_num < mutation_rate) {
      // SENI_LOG("we have a mutation! (index: %d mutation_rate: %.3f, random
      // number: %.3f)", i, mutation_rate, prng_num); replace the var in gene
      // with a newly created one
      gene_generate_new_var(gene, trait, prng_state);
    }

    gene  = gene->next;
    trait = trait->next;
  }

  return genotype;
}

seni_genotype_list* genotype_list_next_generation(seni_genotype_list* parents,
                                                  i32                 num_parents,
                                                  i32                 population_size,
                                                  f32                 mutation_rate,
                                                  i32                 rng,
                                                  seni_trait_list*    trait_list) {
  seni_genotype_list* genotype_list = genotype_list_get_from_pool();

  i32 population_remaining = population_size;

  // copy the parents onto the new generation
  seni_genotype* genotype;
  seni_genotype* parent_genotype = parents->genotypes;
  while (parent_genotype) {
    genotype = genotype_clone(parent_genotype);
    genotype_list_add_genotype(genotype_list, genotype);
    parent_genotype = parent_genotype->next;
    population_remaining--;
  }
  genotype = NULL; // going to re-use the genotype variable later

  seni_prng_state prng_state;
  seni_prng_set_state(&prng_state, (u64)rng);
  i32 retry_count = 10;

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

    seni_genotype* a = genotype_list_get_genotype(parents, a_index);
    seni_genotype* b = genotype_list_get_genotype(parents, b_index);

    i32 genotype_length = genotype_count(a);
    i32 crossover_index = seni_prng_i32_range(&prng_state, 0, genotype_length - 1);

    genotype = genotype_crossover(a, b, crossover_index, genotype_length);
    genotype =
        genotype_possibly_mutate(genotype, genotype_length, mutation_rate, &prng_state, trait_list);

    genotype_list_add_genotype(genotype_list, genotype);

    population_remaining--;
  }

  return genotype_list;
}
