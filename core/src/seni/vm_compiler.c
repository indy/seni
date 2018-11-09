#include "vm_compiler.h"
#include "colour.h"
#include "genetic.h"
#include "keyword_iname.h"
#include "lang.h"
#include "mathutil.h"

#include <string.h>

// N_CHK (node check):
// used by functions that return sen_error
// invokes a function that returns a result_node
// if it's an error, the error is logged and the current function returns
//
#define N_CHK(fn, msg)                     \
  result_node = fn;                        \
  if (is_result_node_error(result_node)) { \
    SEN_ERROR(msg);                        \
    return result_node.error;              \
  }

// B_CHK (bytecode check):
// used by functions that return sen_error
// invokes a function that returns a result_bytecode
// if it's an error, the error is logged and the current function returns
//
#define B_CHK(fn, msg)                             \
  result_bytecode = fn;                            \
  if (is_result_bytecode_error(result_bytecode)) { \
    SEN_ERROR(msg);                                \
    return result_bytecode.error;                  \
  }

// E_CHK (error check):
// used by functions that return sen_error
// invokes a function that returns a sen_error
// if it's an error, the error is logged and the current function returns
//
#define E_CHK(fn, msg) \
  err = fn;            \
  if (is_error(err)) { \
    SEN_ERROR(msg);    \
    return err;        \
  }

// I_CHK (i32 check):
// used by functions that return sen_error
// invokes a function that returns a result_i32
// if it's an error, the error is logged and the current function returns
//
#define I_CHK(fn, msg)                   \
  result_i32 = fn;                       \
  if (is_result_i32_error(result_i32)) { \
    SEN_ERROR(msg);                      \
    return result_i32.error;             \
  }

// F_CHK (f32 check):
// used by functions that return sen_error
// invokes a function that returns a result_f32
// if it's an error, the error is logged and the current function returns
//
#define F_CHK(fn, msg)                   \
  result_f32 = fn;                       \
  if (is_result_f32_error(result_f32)) { \
    SEN_ERROR(msg);                      \
    return result_f32.error;             \
  }

typedef struct sen_compilation {
  sen_program* program;

  i32 opcode_offset;
  i32 global_mappings[MEMORY_GLOBAL_SIZE]; // top-level defines
  i32 local_mappings[MEMORY_LOCAL_SIZE];   // store which word_lut values are
                                           // stored in which local memory
                                           // addresses
  sen_fn_info* current_fn_info;
} sen_compilation;

void sen_compilation_init(sen_compilation* compilation, sen_program* program) {
  compilation->program       = program;
  compilation->opcode_offset = 0;
}

i32 opcode_offset[] = {
#define OPCODE(_, offset) offset,
#include "opcodes.h"
#undef OPCODE
};

bool         g_use_genes;
sen_program* g_preamble_program;

sen_error           compile_vector(sen_compilation* compilation, sen_node* ast);
void                clear_global_mappings(sen_compilation* compilation);
void                clear_local_mappings(sen_compilation* compilation);
sen_error           register_top_level_preamble(sen_compilation* compilation);
sen_error           compile_preamble(sen_compilation* compilation);
sen_result_bytecode emit_opcode_i32(sen_compilation* compilation, sen_opcode op, i32 arg0,
                                    i32 arg1);

// compiler_subsystem_startup
//
sen_error compiler_subsystem_startup() {
  sen_error    err;
  sen_program* program = program_allocate(MAX_PREAMBLE_PROGRAM_SIZE);

  sen_compilation compilation;
  sen_compilation_init(&compilation, program);
  clear_global_mappings(&compilation);
  clear_local_mappings(&compilation);
  compilation.current_fn_info = NULL;

  err = register_top_level_preamble(&compilation);
  if (is_error(err)) {
    SEN_ERROR("compiler_subsystem_startup: register_top_level_preamble");
    return err;
  }

  err = compile_preamble(&compilation);
  if (is_error(err)) {
    SEN_ERROR("compiler_subsystem_startup: compile_preamble");
    return err;
  }

  if (program->code_size == MAX_PREAMBLE_PROGRAM_SIZE) {
    // try increasing the program_max_size
    SEN_ERROR("compiler_subsystem_startup: program code size == "
              "MAX_PREAMBLE_PROGRAM_SIZE ???");
    SEN_ERROR("try increasing the MAX_PREAMBLE_PROGRAM_SIZE value in "
              "compiler_subsystem_startup");
    return ERROR_COMPILER_PREAMBLE_CODE_SIZE;
  }

  // slap a stop onto the end of this program
  sen_result_bytecode result_bytecode;
  B_CHK(emit_opcode_i32(&compilation, STOP, 0, 0), "compiler_subsystem_startup: STOP");

  g_preamble_program = program;

  return NONE;
}

void compiler_subsystem_shutdown() { program_free(g_preamble_program); }

sen_result_program get_preamble_program() { return result_program_ok(g_preamble_program); }

void gene_assign_to_node(sen_word_lut* word_lut, sen_genotype* genotype, sen_node* node) {
  if (node->alterable) {
    if (node->type == NODE_VECTOR) {
      // grab a gene for every element in this vector
      for (sen_node* n = safe_first_child(node); n != NULL; n = safe_next(n)) {
        n->gene = genotype_pull_gene(genotype);
      }
    } else {
      node->gene = genotype_pull_gene(genotype);
    }

  } else {
    node->gene = NULL;

    if (get_node_value_in_use(node->type) == USE_FIRST_CHILD) {
      sen_node* first_child = safe_first(node->value.first_child);
      if (first_child) {
        gene_assign_to_node(word_lut, genotype, first_child);
      }
    }
  }

  // todo: is it safe to assume that node->next will always be valid? and that
  // leaf nodes will have next == null?
  if (node->next) {
    gene_assign_to_node(word_lut, genotype, node->next);
  }
}

sen_error genotype_assign_to_ast(sen_word_lut* word_lut, sen_genotype* genotype,
                                 sen_node* ast) {
  genotype->current_gene = genotype->genes;
  gene_assign_to_node(word_lut, genotype, ast);

  // current gene should be null since traversing the ast
  // and assigning genes to alterable nodes should have
  // resulted in all of the genes being assigned
  //
  sen_gene* gene = genotype->current_gene;
  if (gene != NULL) {
    SEN_ERROR("genotype_assign_to_ast: genes remaining after assigning "
              "genotype to ast");
    return ERROR_COMPILER_GENES_REMAINING_AFTER_ASSIGNING;
  }

  return NONE;
}

sen_result_i32 get_node_value_i32_from_gene(sen_node* node) {
  sen_gene* gene = node->gene;
  if (gene == NULL) {
    SEN_ERROR("null gene returned");
    return result_i32_error(ERROR_COMPILER_NULL_GENE);
  }

  sen_value_in_use value_in_use = get_var_value_in_use(gene->var->type);
  if (value_in_use != USE_I) {
    SEN_ERROR("Gene won't return an integer that the Node is expecting");
    gene_pretty_print("problematic gene:", gene);
    return result_i32_error(ERROR_COMPILER_INCOMPATIBLE_GENE);
  }

  return result_i32_ok(gene->var->value.i);
}

sen_result_f32 get_node_value_f32_from_gene(sen_node* node) {
  sen_gene* gene = node->gene;
  if (gene == NULL) {
    SEN_ERROR("null gene returned");
    return result_f32_error(ERROR_COMPILER_NULL_GENE);
  }

  sen_value_in_use value_in_use = get_var_value_in_use(gene->var->type);
  if (value_in_use != USE_F) {
    SEN_ERROR("Gene won't return a float that the Node is expecting");
    gene_pretty_print("problematic gene:", gene);
    return result_f32_error(ERROR_COMPILER_INCOMPATIBLE_GENE);
  }

  return result_f32_ok(gene->var->value.f);
}

bool alterable(sen_node* node) { return node->alterable && g_use_genes; }

sen_result_i32 get_node_value_i32(sen_node* node) {
  if (alterable(node)) {
    return get_node_value_i32_from_gene(node);
  } else {
    return result_i32_ok(node->value.i);
  }
}

sen_result_f32 get_node_value_f32(sen_node* node) {
  if (alterable(node)) {
    return get_node_value_f32_from_gene(node);
  } else {
    return result_f32_ok(node->value.f);
  }
}

// a temporary message for unimplemented alterable nodes
void warn_if_alterable(char* msg, sen_node* node) {
  if (node->alterable) {
    SEN_ERROR("warn_if_alterable: %s", msg);
  }
}

sen_result_bytecode emit_opcode(sen_compilation* compilation, sen_opcode op, sen_var* arg0,
                                sen_var* arg1) {
  sen_program* program = compilation->program;

  if (program->code_size >= program->code_max_size) {
    SEN_ERROR("%s %d program has reached max size", __FILE__, __LINE__);
    return result_bytecode_error(ERROR_COMPILER_PROGRAM_REACHED_MAX_SIZE);
  }

  sen_bytecode* b = &(program->code[program->code_size++]);
  b->op           = op;
  var_copy(&(b->arg0), arg0);
  var_copy(&(b->arg1), arg1);

  compilation->opcode_offset += opcode_offset[op];

  return result_bytecode_ok(b);
}

// emits an <opcode, i32, i32> triplet
sen_result_bytecode emit_opcode_i32(sen_compilation* compilation, sen_opcode op, i32 arg0,
                                    i32 arg1) {
  sen_program* program = compilation->program;

  if (program->code_size >= program->code_max_size) {
    SEN_ERROR("%s %d program has reached max size: program size=%d, max_size=%d", __FILE__,
              __LINE__, program->code_size, program->code_max_size);
    return result_bytecode_error(ERROR_COMPILER_PROGRAM_REACHED_MAX_SIZE);
  }

  sen_bytecode* b = &(program->code[program->code_size++]);
  b->op           = op;
  i32_as_var(&(b->arg0), arg0);
  i32_as_var(&(b->arg1), arg1);

  compilation->opcode_offset += opcode_offset[op];

  return result_bytecode_ok(b);
}

// emits an <opcode, i32, name> triplet
sen_result_bytecode emit_opcode_i32_name(sen_compilation* compilation, sen_opcode op,
                                         i32 arg0, i32 name) {
  sen_program* program = compilation->program;

  if (program->code_size >= program->code_max_size) {
    SEN_ERROR("%s %d program has reached max size", __FILE__, __LINE__);
    return result_bytecode_error(ERROR_COMPILER_PROGRAM_REACHED_MAX_SIZE);
  }

  sen_bytecode* b = &(program->code[program->code_size++]);
  b->op           = op;
  i32_as_var(&(b->arg0), arg0);
  name_as_var(&(b->arg1), name);

  compilation->opcode_offset += opcode_offset[op];

  return result_bytecode_ok(b);
}

// emits an <opcode, i32, f32> triplet
sen_result_bytecode emit_opcode_i32_f32(sen_compilation* compilation, sen_opcode op, i32 arg0,
                                        f32 arg1) {
  sen_program* program = compilation->program;

  if (program->code_size >= program->code_max_size) {
    SEN_ERROR("%s %d program has reached max size", __FILE__, __LINE__);
    return result_bytecode_error(ERROR_COMPILER_PROGRAM_REACHED_MAX_SIZE);
  }

  sen_bytecode* b = &(program->code[program->code_size++]);
  b->op           = op;
  i32_as_var(&(b->arg0), arg0);
  f32_as_var(&(b->arg1), arg1);

  compilation->opcode_offset += opcode_offset[op];

  return result_bytecode_ok(b);
}

// **************************************************
// Compiler
// **************************************************

// local_mappings :   -1 == no mapping
//                    -2 == internal local mapping
//                  >= 0 == maps a word from word_lut

void clear_local_mappings(sen_compilation* compilation) {
  for (i32 i = 0; i < MEMORY_LOCAL_SIZE; i++) {
    compilation->local_mappings[i] = -1;
  }
}

sen_result_i32 add_local_mapping(sen_compilation* compilation, i32 word_lut_value) {
  for (i32 i = 0; i < MEMORY_LOCAL_SIZE; i++) {
    if (compilation->local_mappings[i] == -1) {
      compilation->local_mappings[i] = word_lut_value;
      return result_i32_ok(i);
    }
  }

  SEN_ERROR("add_local_mapping failed: increase MEMORY_LOCAL_SIZE from %d",
            MEMORY_LOCAL_SIZE);
  return result_i32_error(ERROR_COMPILER_ALLOCATION_FAILURE);
}

// we want a local mapping that's going to be used to store an internal variable
// (e.g. during a fence loop)
// note: it's up to the caller to manage this reference
sen_result_i32 add_internal_local_mapping(sen_compilation* compilation) {
  for (i32 i = 0; i < MEMORY_LOCAL_SIZE; i++) {
    if (compilation->local_mappings[i] == -1) {
      compilation->local_mappings[i] = -2;
      return result_i32_ok(i);
    }
  }

  SEN_ERROR("add_internal_local_mapping failed: increase MEMORY_LOCAL_SIZE from %d",
            MEMORY_LOCAL_SIZE);
  return result_i32_error(ERROR_COMPILER_ALLOCATION_FAILURE);
}

sen_option_i32 get_local_mapping(sen_compilation* compilation, i32 word_lut_value) {
  for (i32 i = 0; i < MEMORY_LOCAL_SIZE; i++) {
    if (compilation->local_mappings[i] == word_lut_value) {
      return option_i32_some(i);
    }
  }

  return option_i32_none();
}

void clear_global_mappings(sen_compilation* compilation) {
  for (i32 i = 0; i < MEMORY_GLOBAL_SIZE; i++) {
    compilation->global_mappings[i] = -1;
  }
}

sen_result_i32 add_global_mapping(sen_compilation* compilation, i32 word_lut_value) {
  for (i32 i = 0; i < MEMORY_GLOBAL_SIZE; i++) {
    if (compilation->global_mappings[i] == -1) {
      compilation->global_mappings[i] = word_lut_value;
      return result_i32_ok(i);
    }
  }

  SEN_ERROR("add_global_mapping failed: increase MEMORY_GLOBAL_SIZE from %d",
            MEMORY_GLOBAL_SIZE);
  return result_i32_error(ERROR_COMPILER_ALLOCATION_FAILURE);
}

sen_option_i32 get_global_mapping(sen_compilation* compilation, i32 word_lut_value) {
  for (i32 i = 0; i < MEMORY_GLOBAL_SIZE; i++) {
    if (compilation->global_mappings[i] == word_lut_value) {
      return option_i32_some(i);
    }
  }

  return option_i32_none();
}

sen_option_i32 get_argument_mapping(sen_fn_info* fn_info, i32 word_lut_value) {
  for (i32 i = 0; i < MAX_NUM_ARGUMENTS; i++) {
    if (fn_info->argument_offsets[i] == -1) {
      return option_i32_none();
    }
    if (fn_info->argument_offsets[i] == word_lut_value) {
      return option_i32_some((i * 2) + 1);
    }
  }
  return option_i32_none();
}

// returns the index into program->fn_info that represents this function
sen_option_i32 get_fn_info_index(sen_node* node, sen_program* program) {

  i32 name = node->value.i;

  for (i32 i = 0; i < MAX_TOP_LEVEL_FUNCTIONS; i++) {
    if (program->fn_info[i].active == false) {
      return option_i32_none();
    }
    if (program->fn_info[i].fn_name == name) {
      return option_i32_some(i);
    }
  }

  return option_i32_none();
}

sen_result_fn_info get_fn_info(sen_node* node, sen_program* program) {
  if (node->type != NODE_NAME) {
    SEN_ERROR("get_fn_info not given a name node");
    return result_fn_info_error(ERROR_COMPILER_EXPECTED_NAME_NODE);
  }

  i32 name = node->value.i;

  for (i32 i = 0; i < MAX_TOP_LEVEL_FUNCTIONS; i++) {
    if (program->fn_info[i].active == false) {
      return result_fn_info_error(ERROR_GENERAL);
    }
    if (program->fn_info[i].fn_name == name) {
      return result_fn_info_ok(&(program->fn_info[i]));
    }
  }

  SEN_ERROR("get_fn_info unable to find fn_info for a function");
  return result_fn_info_error(ERROR_COMPILER_UNABLE_TO_FIND_FN_INFO);
}

sen_result_node compile(sen_compilation* compilation, sen_node* ast);

i32 node_vector_length(sen_node* vector_node) {
  i32 length = 0;
  for (sen_node* node = safe_first(vector_node->value.first_child); node != NULL;
       node           = safe_next(node)) {
    length++;
  }
  return length;
}

sen_result_bool all_children_have_type(sen_node* parent, sen_node_type type) {
  if (parent->type != NODE_VECTOR && parent->type != NODE_LIST) {
    SEN_ERROR("all_children_have_type need a vector or list");
    return result_bool_error(ERROR_COMPILER_EXPECTED_VECTOR_OR_LIST);
  }

  sen_node* child = parent->value.first_child;
  while (child != NULL) {
    if (child->type != type) {
      return result_bool_ok(false);
    }
    child = safe_next(child);
  }

  return result_bool_ok(true);
}

sen_result_i32 count_children(sen_node* parent) {
  if (parent->type != NODE_VECTOR && parent->type != NODE_LIST) {
    SEN_ERROR("count_children need a vector or list");
    return result_i32_error(ERROR_COMPILER_EXPECTED_VECTOR_OR_LIST);
  }

  i32       count = 0;
  sen_node* child = safe_first(parent->value.first_child);
  while (child != NULL) {
    count++;
    child = safe_next(child);
  }

  return result_i32_ok(count);
}

sen_result_i32 store_locally(sen_compilation* compilation, i32 iname) {
  sen_option_i32 option_address = get_local_mapping(compilation, iname);
  i32            address;

  if (is_option_i32_none(option_address)) {
    sen_result_i32 result_address = add_local_mapping(compilation, iname);
    if (is_result_i32_error(result_address)) {
      SEN_ERROR("store_locally: allocation failure");
      return result_i32_error(result_address.error);
    }
    address = result_address.result;
  } else {
    address = option_address.some;
  }

  sen_result_bytecode result_bytecode =
      emit_opcode_i32(compilation, STORE, MEM_SEG_LOCAL, address);
  if (is_result_bytecode_error(result_bytecode)) {
    return result_i32_error(result_bytecode.error);
  }

  return result_i32_ok(address);
}

sen_result_i32 store_globally(sen_compilation* compilation, i32 iname) {
  sen_option_i32 option_address = get_global_mapping(compilation, iname);
  i32            address;

  if (is_option_i32_none(option_address)) {
    sen_result_i32 result_address = add_global_mapping(compilation, iname);
    if (is_result_i32_error(result_address)) {
      SEN_ERROR("store_globally: allocation failure");
      return result_i32_error(result_address.error);
    }
    address = result_address.result;
  } else {
    address = option_address.some;
  }

  sen_result_bytecode result_bytecode =
      emit_opcode_i32(compilation, STORE, MEM_SEG_GLOBAL, address);
  if (is_result_bytecode_error(result_bytecode)) {
    return result_i32_error(result_bytecode.error);
  }

  return result_i32_ok(address);
}

sen_result_i32 store_from_stack_to_memory(sen_compilation* compilation, sen_node* node,
                                          sen_memory_segment_type memory_segment_type) {
  if (memory_segment_type == MEM_SEG_LOCAL) {
    return store_locally(compilation, node->value.i);
  } else if (memory_segment_type == MEM_SEG_GLOBAL) {
    return store_globally(compilation, node->value.i);
  }

  SEN_ERROR("store_from_stack_to_memory: unknown memory_segment_type: %d",
            memory_segment_type);
  return result_i32_error(ERROR_COMPILER_UNKNOWN_MEMORY_SEGMENT_TYPE);
}

sen_error compile_define(sen_compilation* compilation, sen_node* ast,
                         sen_memory_segment_type memory_segment_type) {
  sen_node*           lhs_node = safe_next(ast);
  sen_node*           value_node;
  i32                 i;
  sen_result_bytecode result_bytecode;
  sen_result_i32      result_i32;
  sen_result_bool     result_bool;

  while (lhs_node != NULL) {

    value_node = safe_next(lhs_node);
    compile(compilation, value_node);

    if (lhs_node->type == NODE_NAME) {
      // define foo 10
      I_CHK(store_from_stack_to_memory(compilation, lhs_node, memory_segment_type),
            "compile_define: allocation failure in define");
    } else if (lhs_node->type == NODE_VECTOR) {
      // define [a b] (something-that-returns-a-vector ...)

      // check if we can use the PILE opcode
      result_bool = all_children_have_type(lhs_node, NODE_NAME);
      if (is_result_bool_error(result_bool)) {
        return result_bool.error;
      }
      bool all_children_have_type_name = result_bool.result;
      if (all_children_have_type_name) {
        I_CHK(count_children(lhs_node), "compile_define: count_children");
        i32 num_children = result_i32.result;

        // PILE will stack the elements in the rhs vector in order,
        // so the lhs values have to be popped in reverse order
        B_CHK(emit_opcode_i32(compilation, PILE, num_children, 0), "compile_define: PILE");

        compilation->opcode_offset += num_children - 1;

        sen_node* child = safe_first(lhs_node->value.first_child);

        for (i = 1; i < num_children; i++) {
          child = safe_next(child);
        }
        for (i = 0; i < num_children; i++) {
          I_CHK(store_from_stack_to_memory(compilation, child, memory_segment_type),
                "compile_define: allocation failure during destructure");
          child = safe_prev(child);
        }
      } else {
        // this may be recursive
        SEN_LOG("todo: push each item onto stack using nth");
      }

    } else {
      SEN_ERROR("compile_define lhs should be a name or a list");
      return ERROR_COMPILER_EXPECTED_NAME_OR_LIST;
    }

    lhs_node = safe_next(value_node);
  }

  return NONE;
}

sen_error compile_if(sen_compilation* compilation, sen_node* ast) {
  sen_result_bytecode result_bytecode;
  // if (> 200 100) 12 24
  // ^
  sen_node* if_node   = safe_next(ast);
  sen_node* then_node = safe_next(if_node);
  sen_node* else_node = safe_next(then_node); // could be NULL

  compile(compilation, if_node);

  // insert jump to after the 'then' node if not true
  i32 addr_jump_then = compilation->program->code_size;
  B_CHK(emit_opcode_i32(compilation, JUMP_IF, 0, 0), "compile_if: JUMP_IF");
  sen_bytecode* bc_jump_then = result_bytecode.result;

  // the offset after the if
  i32 offset_after_if = compilation->opcode_offset;

  compile(compilation, then_node);

  i32 offset_after_then = compilation->opcode_offset;

  if (else_node) {
    // logically we're now going to go down one of possibly two paths
    // so we can't just continue to add the compilation->opcode_offset since
    // that would result in the offset taking both of the conditional's paths

    compilation->opcode_offset = offset_after_if;

    // insert a bc_jump_else opcode
    i32 addr_jump_else = compilation->program->code_size;

    B_CHK(emit_opcode_i32(compilation, JUMP, 0, 0), "compile_if: JUMP");
    sen_bytecode* bc_jump_else = result_bytecode.result;

    bc_jump_then->arg0.value.i = compilation->program->code_size - addr_jump_then;

    compile(compilation, else_node);

    i32 offset_after_else = compilation->opcode_offset;

    if (offset_after_then != offset_after_else) {
      // is this case actually going to happen?
      // if so we can check which of the two paths has the lower opcode offset
      // and pad out that path by inserting some LOAD CONST 9999 into the
      // program
      SEN_ERROR("different opcode_offsets for the two paths in a conditional");
    }

    bc_jump_else->arg0.value.i = compilation->program->code_size - addr_jump_else;
  } else {
    bc_jump_then->arg0.value.i = compilation->program->code_size - addr_jump_then;
  }

  return NONE;
}

// compiles everything after the current ast point
sen_error compile_rest(sen_compilation* compilation, sen_node* ast) {
  sen_result_node result_node;

  ast = safe_next(ast);
  while (ast) {
    N_CHK(compile(compilation, ast), "compile_rest: compile");
    ast = result_node.result;
  }

  return NONE;
}

// compiles the next node after the current ast point
sen_error compile_next_one(sen_compilation* compilation, sen_node* ast) {
  sen_result_node result_node;

  ast = safe_next(ast);
  N_CHK(compile(compilation, ast), "compile_next_one: compile");

  return NONE;
}

sen_error compile_math(sen_compilation* compilation, sen_node* ast, sen_opcode opcode) {
  sen_result_node     result_node;
  sen_result_bytecode result_bytecode;
  // + 3 4 5 6
  //
  // 1	LOAD	CONST	3.00
  // 2	LOAD	CONST	4.00
  // 3	ADD
  // 4	LOAD	CONST	5.00
  // 5	ADD
  // 6	LOAD	CONST	6.00
  // 7	ADD

  ast = safe_next(ast); // skip the opcode

  // compile the first argument
  N_CHK(compile(compilation, ast);, "compile_math: compile");
  ast = result_node.result;
  while (ast) {
    // compile the next argument
    N_CHK(compile(compilation, ast);, "compile_math: compile");
    ast = result_node.result;

    // store the index into program->fn_info in the program
    B_CHK(emit_opcode_i32(compilation, opcode, 0, 0), "compile_math: opcode");
  }
  return NONE;
}

sen_error compile_address_of(sen_compilation* compilation, sen_node* ast) {
  sen_result_bytecode result_bytecode;
  sen_node*           fn_name = safe_next(ast);

  // fn_name should be a defined function's name
  // it will be known at compile time
  if (fn_name->type != NODE_NAME) {
    SEN_ERROR("compile_address_of given non-function-name argument");
    return ERROR_COMPILER_ADDRESS_OF_NAME;
  }

  sen_result_fn_info result_fn_info = get_fn_info(fn_name, compilation->program);
  if (is_result_fn_info_error(result_fn_info)) {
    SEN_ERROR("address-of could not find function");
    return result_fn_info.error;
  }
  sen_fn_info* fn_info = result_fn_info.result;

  // store the index into program->fn_info in the program
  B_CHK(emit_opcode_i32(compilation, LOAD, MEM_SEG_CONSTANT, fn_info->index),
        "compile_address_of: CONSTANT");

  return NONE;
}

//   (fn-call (aj z: 44))
sen_error compile_fn_call(sen_compilation* compilation, sen_node* ast) {
  sen_result_node     result_node;
  sen_result_bytecode result_bytecode;
  sen_result_i32      result_i32;
  sen_node*           invocation = safe_next(ast);

  // fn_name should be a defined function's name
  // it will be known at compile time

  if (invocation->type != NODE_LIST) {
    SEN_ERROR("compile_fn_call given non-list to invoke");
    return ERROR_COMPILER_FN_CALL_GIVEN_NON_LIST;
  }

  warn_if_alterable("compile_fn_call invocation", invocation);

  sen_node* fn_info_index = safe_first(invocation->value.first_child);

  // place the fn_info_index onto the stack so that CALL_F can find the function
  // offset and num args
  N_CHK(compile(compilation, fn_info_index), "compile_fn_call: compile");
  B_CHK(emit_opcode_i32(compilation, CALL_F, 0, 0), "compile_fn_call: emit_opcode_i32");

  // compile the rest of the arguments

  // overwrite the default arguments with the actual arguments given by the fn
  // invocation
  sen_node* args = safe_next(fn_info_index); // pairs of label/value declarations
  while (args != NULL) {
    sen_node* label = args;
    sen_node* value = safe_next(label);

    // push value
    N_CHK(compile(compilation, value), "compile_fn_call: compile");

    // push the actual fn_info index so that the _FLU opcode can find it
    N_CHK(compile(compilation, fn_info_index), "compile_fn_call: compile");

    I_CHK(get_node_value_i32(label), "compile_fn_call: get_node_value_i32");
    i32 label_i = result_i32.result;

    B_CHK(emit_opcode_i32(compilation, STORE_F, MEM_SEG_ARGUMENT, label_i),
          "compile_fn_call: STORE_F");

    args = safe_next(value);
  }

  // place the fn_info_index onto the stack so that CALL_F_0 can find the
  // function's body offset
  N_CHK(compile(compilation, fn_info_index), "compile_fn_call: compile");
  B_CHK(emit_opcode_i32(compilation, CALL_F_0, 0, 0), "compile_fn_call: CALL_F");

  return NONE;
}

sen_error compile_vector_append(sen_compilation* compilation, sen_node* ast) {
  sen_result_node     result_node;
  sen_result_bytecode result_bytecode;
  sen_result_i32      result_i32;

  // (vector/append vector value)
  sen_node* vector = safe_next(ast);
  N_CHK(compile(compilation, vector), "compile_vector_append: compile");

  sen_node* value = safe_next(vector);
  N_CHK(compile(compilation, value), "compile_vector_append: compile");
  B_CHK(emit_opcode_i32(compilation, APPEND, 0, 0), "compile_vector_append: APPEND");

  if (vector->type == NODE_NAME) {
    I_CHK(get_node_value_i32(vector), "compile_vector_append: get_node_value_i32");
    i32 vector_i = result_i32.result;

    sen_option_i32 option_i32;

    option_i32 = get_local_mapping(compilation, vector_i);
    if (is_option_i32_some(option_i32)) {
      i32 address = option_i32.some;
      B_CHK(emit_opcode_i32(compilation, STORE, MEM_SEG_LOCAL, address),
            "compile_vector_append: STORE");
      return NONE;
    }

    option_i32 = get_global_mapping(compilation, vector_i);
    if (is_option_i32_some(option_i32)) {
      i32 address = option_i32.some;
      B_CHK(emit_opcode_i32(compilation, STORE, MEM_SEG_GLOBAL, address),
            "compile_vector_append: STORE global");
      return NONE;
    }

    SEN_ERROR("compile_vector_append: can't find local or global variable");
    return ERROR_COMPILER_UNABLE_TO_FIND_VARIABLE;
  }

  return NONE;
}

sen_error compile_vector_in_quote(sen_compilation* compilation, sen_node* ast) {
  sen_result_node     result_node;
  sen_result_bytecode result_bytecode;

  // pushing from the VOID means creating a new, empty vector
  B_CHK(emit_opcode_i32(compilation, LOAD, MEM_SEG_VOID, 0), "compile_vector_in_quote: LOAD");

  warn_if_alterable("compile_vector_in_quote", ast);
  for (sen_node* node = safe_first(ast->value.first_child); node != NULL;
       node           = safe_next(node)) {
    // slightly hackish
    // if this is a form like: '(red green blue)
    // the compiler should output the names rather than the colours that are
    // actually referenced (compile_user_defined_name would genereate a
    // MEM_SEG_GLOBAL LOAD code)
    //
    if (node->type == NODE_NAME) {
      B_CHK(emit_opcode_i32_name(compilation, LOAD, MEM_SEG_CONSTANT, node->value.i),
            "compile_vector_in_quote: LOAD");
    } else {
      N_CHK(compile(compilation, node), "compile_vector_in_quote: compile");
    }

    B_CHK(emit_opcode_i32(compilation, APPEND, 0, 0), "compile_vector_in_quote: APPEND");
  }

  return NONE;
}

sen_error compile_quote(sen_compilation* compilation, sen_node* ast) {
  sen_result_bytecode result_bytecode;
  sen_result_node     result_node;
  sen_node*           quoted_form = safe_next(ast);

  if (quoted_form->type == NODE_LIST) {
    // compile each entry individually, don't treat the list as a normal
    // function invocation
    sen_error err = compile_vector_in_quote(compilation, quoted_form);
    if (is_error(err)) {
      SEN_ERROR("compile_quote: compile_vector_in_quote");
      return err;
    }
  } else {
    if (quoted_form->type == NODE_NAME) {
      B_CHK(emit_opcode_i32_name(compilation, LOAD, MEM_SEG_CONSTANT, quoted_form->value.i),
            "compile_quote: emit_opcode_i32_name");
    } else {
      N_CHK(compile(compilation, quoted_form), "compile_quote: compile");
    }
  }

  return NONE;
}

sen_error compile_loop(sen_compilation* compilation, sen_node* ast) {
  sen_node* parameters_node = safe_next(ast);
  if (parameters_node->type != NODE_LIST) {
    SEN_ERROR("expected a list that defines step parameters");
    return ERROR_COMPILER_NO_STEP_PARAMETERS_FOR_LOOP;
  }

  warn_if_alterable("compile_loop parameters_node", parameters_node);

  // the looping variable x
  sen_node* name_node = safe_first(parameters_node->value.first_child);

  sen_node* from_node      = NULL;
  sen_node* to_node        = NULL;
  sen_node* upto_node      = NULL;
  sen_node* increment_node = NULL;
  bool      have_from      = false;
  bool      have_to        = false;
  bool      have_upto      = false;
  bool      have_increment = false;

  sen_node*           node = name_node;
  sen_result_bytecode result_bytecode;
  sen_result_node     result_node;
  sen_result_i32      result_i32;
  sen_error           err;

  while (node) {
    node = safe_next(node); // the label part
    if (node == NULL) {
      break;
    }
    if (node->value.i == INAME_FROM) {
      have_from = true;
      from_node = safe_next(node);
    }
    if (node->value.i == INAME_TO) {
      have_to = true;
      to_node = safe_next(node);
    }
    if (node->value.i == INAME_UPTO) {
      have_upto = true;
      upto_node = safe_next(node);
    }
    if (node->value.i == INAME_INC) {
      have_increment = true;
      increment_node = safe_next(node);
    }
    node = safe_next(node); // the value part
  }

  bool use_to = false;

  if (have_to == false) {
    if (have_upto == false) {
      SEN_ERROR("step form requires either a 'to' or 'upto' parameter");
      return ERROR_COMPILER_NO_TO_OR_UPTO_PARAMETERS_FOR_LOOP;
    }
  } else {
    use_to = true;
  }

  // set looping variable x to 'from' value
  if (have_from) {
    N_CHK(compile(compilation, from_node), "compile_loop: compile")
  } else {
    // else default to 0
    B_CHK(emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, 0.0f),
          "compile_loop: emit_opcode_i32_f32");
  }

  I_CHK(store_from_stack_to_memory(compilation, name_node, MEM_SEG_LOCAL),
        "compile_loop: store_from_stack_to_memory");
  i32 looper_address = result_i32.result;

  // compare looping variable against exit condition
  // and jump if looping variable >= exit value
  i32 addr_loop_start = compilation->program->code_size;

  B_CHK(emit_opcode_i32(compilation, LOAD, MEM_SEG_LOCAL, looper_address),
        "compile_loop: emit_opcode_i32");

  if (use_to) {
    // so jump if looping variable >= exit value
    N_CHK(compile(compilation, to_node), "compile_loop: compile");

    B_CHK(emit_opcode_i32(compilation, LT, 0, 0), "compile_loop: emit_opcode_i32");
  } else {
    // so jump if looping variable > exit value
    N_CHK(compile(compilation, upto_node), "compile_loop: compile");

    B_CHK(emit_opcode_i32(compilation, GT, 0, 0), "compile_loop: emit_opcode_i32");
    B_CHK(emit_opcode_i32(compilation, NOT, 0, 0), "compile_loop: emit_opcode_i32");
  }

  i32 addr_exit_check = compilation->program->code_size;

  B_CHK(emit_opcode_i32(compilation, JUMP_IF, 0, 0), "compile_loop: JUMP_IF");
  sen_bytecode* bc_exit_check = result_bytecode.result;

  i32 pre_body_opcode_offset = compilation->opcode_offset;

  // compile the body forms (woooaaaoohhh body form, body form for yoooouuuu)
  E_CHK(compile_rest(compilation, parameters_node), "compile_loop: compile_rest");

  i32 post_body_opcode_offset = compilation->opcode_offset;
  i32 opcode_delta            = post_body_opcode_offset - pre_body_opcode_offset;

  // pop off any values that the body might leave on the stack
  for (i32 i = 0; i < opcode_delta; i++) {
    B_CHK(emit_opcode_i32(compilation, STORE, MEM_SEG_VOID, 0),
          "compile_loop: emit_opcode_i32");
  }

  // increment the looping variable
  B_CHK(emit_opcode_i32(compilation, LOAD, MEM_SEG_LOCAL, looper_address),
        "compile_loop: emit_opcode_i32");

  if (have_increment) {
    N_CHK(compile(compilation, increment_node), "compile_loop: compile");
  } else {
    B_CHK(emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, 1.0f),
          "compile_loop: emit_opcode_i32_f32");
  }

  B_CHK(emit_opcode_i32(compilation, ADD, 0, 0), "compile_loop: emit_opcode_i32");

  B_CHK(emit_opcode_i32(compilation, STORE, MEM_SEG_LOCAL, looper_address),
        "compile_loop: emit_opcode_i32");

  // loop back to the comparison
  B_CHK(emit_opcode_i32(compilation, JUMP,
                        -(compilation->program->code_size - addr_loop_start), 0),
        "compile_loop: emit_opcode_i32");

  bc_exit_check->arg0.value.i = compilation->program->code_size - addr_exit_check;

  return NONE;
}

sen_error compile_fence(sen_compilation* compilation, sen_node* ast) {
  sen_result_i32      result_i32;
  sen_result_bytecode result_bytecode;
  sen_result_node     result_node;
  sen_error           err;
  // (fence (x from: 0 to: 5 quantity: 5) (+ 42 38))

  sen_node* parameters_node = safe_next(ast);
  if (parameters_node->type != NODE_LIST) {
    SEN_ERROR("expected a list that defines fence parameters");
    return ERROR_COMPILER_EXPECTED_LIST;
  }

  warn_if_alterable("compile_fence parameters_node", parameters_node);

  // the looping variable x
  sen_node* name_node = safe_first(parameters_node->value.first_child);

  sen_node* from_node = NULL;
  sen_node* to_node   = NULL;
  sen_node* num_node  = NULL;
  bool      have_from = false;
  bool      have_to   = false;
  bool      have_num  = false;

  sen_node* node = name_node;

  while (node) {
    node = safe_next(node); // the label part
    if (node == NULL) {
      break;
    }
    if (node->value.i == INAME_FROM) {
      have_from = true;
      from_node = safe_next(node);
    }
    if (node->value.i == INAME_TO) {
      have_to = true;
      to_node = safe_next(node);
    }
    if (node->value.i == INAME_NUM) {
      have_num = true;
      num_node = safe_next(node);
    }
    node = safe_next(node); // the value part
  }

  // store the quantity
  I_CHK(add_internal_local_mapping(compilation), "compile_fence: add_internal_local_mapping");
  i32 quantity_address = result_i32.result;
  if (have_num) {
    N_CHK(compile(compilation, num_node), "compile_fence: compile");
  } else {
    // else default to 2
    B_CHK(emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, 2.0f),
          "compile_fence: LOAD");
  }

  B_CHK(emit_opcode_i32(compilation, STORE, MEM_SEG_LOCAL, quantity_address),
        "compile_fence: STORE");

  // reserve a memory location in local memory for a counter from 0 to quantity
  I_CHK(add_internal_local_mapping(compilation), "compile_loop: add_internal_local_mapping");
  i32 counter_address = result_i32.result;

  B_CHK(emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, 0.0f), "compile_fence");
  B_CHK(emit_opcode_i32(compilation, STORE, MEM_SEG_LOCAL, counter_address), "compile_fence");

  // delta that needs to be added at every iteration
  //
  // (to - from) / (quantity - 1)
  if (have_to) {
    N_CHK(compile(compilation, to_node), "compile_fence: compile");
  } else {
    // else default to 1
    B_CHK(emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, 1.0f), "compile_fence");
  }
  if (have_from) {
    N_CHK(compile(compilation, from_node), "compile_fence: compile");
  } else {
    // else default to 0
    B_CHK(emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, 0.0f), "compile_fence");
  }
  B_CHK(emit_opcode_i32(compilation, SUB, 0, 0), "compile_fence");

  N_CHK(compile(compilation, num_node), "compile_fence: compile");
  B_CHK(emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, 1.0f), "compile_fence");
  B_CHK(emit_opcode_i32(compilation, SUB, 0, 0), "compile_fence");
  B_CHK(emit_opcode_i32(compilation, DIV, 0, 0), "compile_fence");
  I_CHK(add_internal_local_mapping(compilation), "compile_loop: add_internal_local_mapping");
  i32 delta_address = result_i32.result;
  B_CHK(emit_opcode_i32(compilation, STORE, MEM_SEG_LOCAL, delta_address), "compile_fence");

  // set looping variable x to 'from' value
  if (have_from) {
    N_CHK(compile(compilation, from_node), "compile_fence: compile");
  } else {
    // else default to 0
    B_CHK(emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, 0.0f), "compile_fence");
  }

  I_CHK(add_internal_local_mapping(compilation), "compile_loop: add_internal_local_mapping");
  i32 from_address = result_i32.result;

  B_CHK(emit_opcode_i32(compilation, STORE, MEM_SEG_LOCAL, from_address), "compile_fence");

  // store the starting 'from' value in the locally scoped variable
  B_CHK(emit_opcode_i32(compilation, LOAD, MEM_SEG_LOCAL, from_address), "compile_fence");

  I_CHK(store_from_stack_to_memory(compilation, name_node, MEM_SEG_LOCAL),
        "compile_loop: store_from_stack_to_memory");
  i32 looper_address = result_i32.result;

  // compare looping variable against exit condition
  // and jump if looping variable >= exit value
  i32 addr_loop_start = compilation->program->code_size;

  // load from counter address
  B_CHK(emit_opcode_i32(compilation, LOAD, MEM_SEG_LOCAL, counter_address), "compile_fence");

  // load from quantity address
  B_CHK(emit_opcode_i32(compilation, LOAD, MEM_SEG_LOCAL, quantity_address), "compile_fence");

  // exit check
  B_CHK(emit_opcode_i32(compilation, LT, 0, 0), "compile_fence");

  i32 addr_exit_check = compilation->program->code_size;

  B_CHK(emit_opcode_i32(compilation, JUMP_IF, 0, 0), "compile_fence");
  sen_bytecode* bc_exit_check = result_bytecode.result;

  // looper = from + (counter * delta)
  B_CHK(emit_opcode_i32(compilation, LOAD, MEM_SEG_LOCAL, from_address), "compile_fence");
  B_CHK(emit_opcode_i32(compilation, LOAD, MEM_SEG_LOCAL, counter_address), "compile_fence");
  B_CHK(emit_opcode_i32(compilation, LOAD, MEM_SEG_LOCAL, delta_address), "compile_fence");
  B_CHK(emit_opcode_i32(compilation, MUL, 0, 0), "compile_fence");
  B_CHK(emit_opcode_i32(compilation, ADD, 0, 0), "compile_fence");
  B_CHK(emit_opcode_i32(compilation, STORE, MEM_SEG_LOCAL, looper_address), "compile_fence");

  i32 pre_body_opcode_offset = compilation->opcode_offset;

  // compile the body forms (woooaaaoohhh body form, body form for yoooouuuu)
  E_CHK(compile_rest(compilation, parameters_node), "compile_loop: compile_rest");

  i32 post_body_opcode_offset = compilation->opcode_offset;
  i32 opcode_delta            = post_body_opcode_offset - pre_body_opcode_offset;

  // pop off any values that the body might leave on the stack
  for (i32 i = 0; i < opcode_delta; i++) {
    B_CHK(emit_opcode_i32(compilation, STORE, MEM_SEG_VOID, 0), "compile_fence");
  }

  // increment counter
  B_CHK(emit_opcode_i32(compilation, LOAD, MEM_SEG_LOCAL, counter_address), "compile_fence");
  B_CHK(emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, 1.0f), "compile_fence");
  B_CHK(emit_opcode_i32(compilation, ADD, 0, 0), "compile_fence");
  B_CHK(emit_opcode_i32(compilation, STORE, MEM_SEG_LOCAL, counter_address), "compile_fence");

  // loop back to the comparison
  emit_opcode_i32(compilation, JUMP, -(compilation->program->code_size - addr_loop_start), 0);
  bc_exit_check->arg0.value.i = compilation->program->code_size - addr_exit_check;

  return NONE;
}

sen_error compile_on_matrix_stack(sen_compilation* compilation, sen_node* ast) {
  sen_result_bytecode result_bytecode;
  sen_error           err;

  B_CHK(emit_opcode_i32(compilation, MTX_LOAD, 0, 0), "compile_on_matrix_stack");
  E_CHK(compile_rest(compilation, ast), "compile_on_matrix_stack: compile_rest");
  B_CHK(emit_opcode_i32(compilation, MTX_STORE, 0, 0), "compile_on_matrix_stack");

  return NONE;
}

sen_error register_top_level_fns(sen_compilation* compilation, sen_node* ast) {
  i32 i;
  i32 num_fns = 0;

  // clear all fn data
  for (i = 0; i < MAX_TOP_LEVEL_FUNCTIONS; i++) {
    compilation->program->fn_info[i].active = false;
  }

  // register top level fns
  while (ast != NULL) {

    if (ast->type != NODE_LIST) {
      ast = safe_next(ast);
      continue;
    }

    // if any of these 'register' functions encounter an alterable node we can't
    // just take a gene from the genotype since we'll go out of sync because the
    // bodies aren't being parsed yet
    // warn_if_alterable("register_top_level_fns", ast);

    sen_node* fn_keyword = safe_first(ast->value.first_child);
    if (!(fn_keyword->type == NODE_NAME && fn_keyword->value.i == INAME_FN)) {
      ast = safe_next(ast);
      continue;
    }

    // (fn (add-up a: 0 b: 0) (+ a b))
    // get the name of the fn
    sen_node* name_and_params = safe_next(fn_keyword);
    if (name_and_params->type != NODE_LIST) {
      ast = safe_next(ast);
      continue;
    }

    sen_node* name       = safe_first(name_and_params->value.first_child);
    i32       name_value = name->value.i;

    // we have a named top-level fn declaration
    sen_fn_info* fn_info = &(compilation->program->fn_info[num_fns]);
    num_fns++;
    if (num_fns > MAX_TOP_LEVEL_FUNCTIONS) {
      SEN_ERROR("Script has more than %d top-level functions\n", MAX_TOP_LEVEL_FUNCTIONS);
      return ERROR_COMPILER_MAX_TOP_LEVEL_FUNCTIONS;
    }

    fn_info->active  = true;
    fn_info->index   = num_fns - 1;
    fn_info->fn_name = name_value;

    // these will be filled in by compile_fn:
    fn_info->num_args = 0;
    for (i = 0; i < MAX_NUM_ARGUMENTS; i++) {
      fn_info->argument_offsets[i] = -1;
    }

    ast = safe_next(ast);
  }

  return NONE;
}

sen_error register_names_in_define(sen_compilation* compilation, sen_node* lhs) {
  warn_if_alterable("register_names_in_define lhs", lhs);
  if (lhs->type == NODE_NAME) {
    // (define foo 42)
    sen_option_i32 option_i32 = get_global_mapping(compilation, lhs->value.i);
    if (is_option_i32_none(option_i32)) {
      sen_result_i32 result_i32 = add_global_mapping(compilation, lhs->value.i);
      if (is_result_i32_error(result_i32)) {
        SEN_ERROR("register_names_in_define: add_global_mapping");
        return result_i32.error;
      }
    }
  } else if (lhs->type == NODE_LIST || lhs->type == NODE_VECTOR) {
    // (define [a b] (something))
    // (define [a [x y]] (something))

    sen_node* child = safe_first(lhs->value.first_child);

    while (child != NULL) {
      register_names_in_define(compilation, child);
      child = safe_next(child);
    }
  }

  return NONE;
}

sen_error register_top_level_defines(sen_compilation* compilation, sen_node* ast) {
  // register top level fns
  while (ast != NULL) {

    if (ast->type != NODE_LIST) {
      ast = safe_next(ast);
      continue;
    }

    // warn_if_alterable("register_top_level_defines define_keyword", ast);

    sen_node* define_keyword = safe_first(ast->value.first_child);
    if (!(define_keyword->type == NODE_NAME && define_keyword->value.i == INAME_DEFINE)) {
      ast = safe_next(ast);
      continue;
    }

    sen_node* lhs = safe_next(define_keyword);
    sen_error err;
    while (lhs != NULL) {
      E_CHK(register_names_in_define(compilation, lhs),
            "register_top_level_defines: register_names_in_define");
      lhs = safe_next(lhs); // points to the value
      lhs = safe_next(lhs); // points to the next define statement if there multiple
    }

    ast = safe_next(ast);
  }

  return NONE;
}

/*
  invoking code will first CALL into the arg_address to setup the default values
  for all args the fn code will then return back to the invoking code invoking
  code will then overwrite specific data in arg memory invoking code will then
  CALL into the body_address
*/
sen_error compile_fn(sen_compilation* compilation, sen_node* ast) {
  sen_result_node     result_node;
  sen_result_bytecode result_bytecode;
  sen_error           err;
  // fn (adder a: 0 b: 0) (+ a b)

  clear_local_mappings(compilation);

  // (adder a: 0 b: 0)
  sen_node* signature = safe_next(ast);

  warn_if_alterable("compile_fn signature", signature);
  sen_node* fn_name = safe_first(signature->value.first_child);

  sen_result_fn_info result_fn_info = get_fn_info(fn_name, compilation->program);
  if (is_result_fn_info_error(result_fn_info)) {
    SEN_ERROR("Unable to find fn_info for function %d", fn_name->value.i);
    return result_fn_info.error;
  }
  sen_fn_info* fn_info         = result_fn_info.result;
  compilation->current_fn_info = fn_info;

  // -------------
  // the arguments
  // -------------

  fn_info->arg_address    = compilation->program->code_size;
  sen_node*      args     = safe_next(fn_name); // pairs of label/value declarations
  i32            num_args = 0;
  i32            counter  = 0;
  i32            argument_offsets_counter = 0;
  sen_result_i32 result_i32;
  while (args != NULL) {
    sen_node* label = args;

    I_CHK(get_node_value_i32(label), "compile_fn: get_node_value_i32");
    i32 label_i = result_i32.result;

    sen_node* value = safe_next(label);

    // get_argument_mapping
    fn_info->argument_offsets[argument_offsets_counter++] = label_i;

    // push pairs of label+value values onto the args stack
    B_CHK(emit_opcode_i32(compilation, LOAD, MEM_SEG_CONSTANT, label_i), "compile_fn");
    B_CHK(emit_opcode_i32(compilation, STORE, MEM_SEG_ARGUMENT, counter++), "compile_fn");

    N_CHK(compile(compilation, value), "compile_fn: compile");
    B_CHK(emit_opcode_i32(compilation, STORE, MEM_SEG_ARGUMENT, counter++), "compile_fn");

    num_args++;
    args = safe_next(value);
  }

  fn_info->num_args = num_args;

  B_CHK(emit_opcode_i32(compilation, RET_0, 0, 0), "compile_fn");

  // --------
  // the body
  // --------

  fn_info->body_address = compilation->program->code_size;

  // (+ a b)
  E_CHK(compile_rest(compilation, signature), "compile_fn: compile_rest");

  // Don't need any STORE, MEM_SEG_VOID instructions as the RET will
  // pop the frame and blow the stack
  B_CHK(emit_opcode_i32(compilation, RET, 0, 0), "compile_fn");

  compilation->current_fn_info = NULL;

  return NONE;
}

sen_error correct_function_addresses(sen_compilation* compilation) {
  // go through the bytecode fixing up function call addresses

  sen_bytecode* bc     = compilation->program->code;
  sen_bytecode* offset = NULL;
  i32           fn_info_index, label_value;
  sen_fn_info*  fn_info;

  for (i32 i = 0; i < compilation->program->code_size; i++) {
    // replace the temporarily stored index in the args of CALL and CALL_0 with
    // the actual values
    if (bc->op == CALL) {
      fn_info_index = bc->arg0.value.i;
      fn_info       = &(compilation->program->fn_info[fn_info_index]);

      // the previous two bytecodes will be LOADs of CONST.
      // i - 2 == the address to call
      // i - 1 == the number of arguments used by the function
      offset = bc - 2;
      if (offset->op != LOAD && offset->arg0.value.i != MEM_SEG_CONSTANT) {
        SEN_ERROR("correct_function_addresses expected a 'LOAD CONST' 2 "
                  "opcodes before a CALL");
        return ERROR_COMPILER_UNABLE_TO_CORRECT_FN_ADDR;
      }
      offset->arg1.value.i = fn_info->arg_address;

      offset = bc - 1;
      if (offset->op != LOAD && offset->arg0.value.i != MEM_SEG_CONSTANT) {
        SEN_ERROR("correct_function_addresses expected a 'LOAD CONST' 1 "
                  "opcode before a CALL");
        return ERROR_COMPILER_UNABLE_TO_CORRECT_FN_ADDR;
      }
      offset->arg1.value.i = fn_info->num_args;
    }

    if (bc->op == CALL_0) {
      fn_info_index = bc->arg0.value.i;
      fn_info       = &(compilation->program->fn_info[fn_info_index]);

      offset = bc - 1;
      if (offset->op != LOAD && offset->arg0.value.i != MEM_SEG_CONSTANT) {
        SEN_ERROR("correct_function_addresses expected a 'LOAD CONST' 1 "
                  "opcode before a CALL_0");
        return ERROR_COMPILER_UNABLE_TO_CORRECT_FN_ADDR;
      }
      offset->arg1.value.i = fn_info->body_address;
    }

    if (bc->op == PLACEHOLDER_STORE) {
      bc->op = STORE;

      // opcode's arg0 is the fn_info_index and arg1 is the label_value
      fn_info_index = bc->arg0.value.i;
      fn_info       = &(compilation->program->fn_info[fn_info_index]);
      label_value   = bc->arg1.value.i;

      sen_option_i32 option_i32 = get_argument_mapping(fn_info, label_value);
      if (is_option_i32_some(option_i32)) {
        i32 data_index   = option_i32.some;
        bc->arg1.value.i = data_index;
        bc->arg0.value.i = MEM_SEG_ARGUMENT;
      } else {
        // otherwise this function was invoked with a parameter that is doesn't
        // use so just essentially turn these ops into no-ops
        bc->arg0.value.i = MEM_SEG_VOID;
      }
    }

    bc++;
  }

  return NONE;
}

sen_error compile_fn_invocation(sen_compilation* compilation, sen_node* ast,
                                i32 fn_info_index) {
  sen_result_node     result_node;
  sen_result_bytecode result_bytecode;
  // ast == adder a: 10 b: 20

  // NOTE: CALL and CALL_0 get their function offsets and num args from the
  // stack so add some placeholder LOAD CONST opcodes and fill the CALL, CALL_0
  // with fn_info indexes that can later be used to fill in the LOAD CONST
  // opcodes with their correct offsets doing it this way enables functions to
  // call other functions that are declared later in the script

  // prepare the MEM_SEG_ARGUMENT with default values

  // for the function address
  B_CHK(emit_opcode_i32(compilation, LOAD, MEM_SEG_CONSTANT, 666);
        , "address of called function");
  // for the num args
  B_CHK(emit_opcode_i32(compilation, LOAD, MEM_SEG_CONSTANT, 667);
        , "num args of called function");
  B_CHK(emit_opcode_i32(compilation, CALL, fn_info_index, fn_info_index),
        "compile_fn_invocation");

  // overwrite the default arguments with the actual arguments given by the fn
  // invocation
  sen_node*      args = safe_next(ast); // pairs of label/value declarations
  sen_result_i32 result_i32;
  while (args != NULL) {
    sen_node* label = args;
    I_CHK(get_node_value_i32(label), "compile_fn_invocation: get_node_value_i32");

    i32 label_i = result_i32.result;

    sen_node* value = safe_next(label);

    // push value
    N_CHK(compile(compilation, value), "compile_fn_invocation: compile");
    B_CHK(emit_opcode_i32(compilation, PLACEHOLDER_STORE, fn_info_index, label_i),
          "compile_fn_invocation");

    args = safe_next(value);
  }

  // call the body of the function
  // for the function body address
  B_CHK(emit_opcode_i32(compilation, LOAD, MEM_SEG_CONSTANT, 668);, "compile_fn_invocation");
  B_CHK(emit_opcode_i32(compilation, CALL_0, fn_info_index, fn_info_index),
        "compile_fn_invocation");

  return NONE;
}

// ast is a NODE_VECTOR of length 2
//
sen_error compile_2d_from_gene(sen_compilation* compilation, sen_node* ast) {
  sen_result_bytecode result_bytecode;
  sen_gene*           gene = ast->gene;

  f32 a = gene->var->f32_array[0];
  f32 b = gene->var->f32_array[1];

  B_CHK(emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, a),
        "compile_2d_from_gene: emit_opcode_i32_f32");
  B_CHK(emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, b),
        "compile_2d_from_gene: emit_opcode_i32_f32");
  B_CHK(emit_opcode_i32(compilation, SQUISH2, 0, 0), "compile_2d_from_gene: emit_opcode_i32");

  return NONE;
}

sen_error compile_alterable_element(sen_compilation* compilation, sen_node* node) {
  sen_result_i32      result_i32;
  sen_result_f32      result_f32;
  sen_result_bytecode result_bytecode;
  sen_error           err;

  if (node->type == NODE_FLOAT) {
    F_CHK(get_node_value_f32_from_gene(node), "compile_alterable_element");
    f32 f = result_f32.result;

    B_CHK(emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, f),
          "compile_alterable_element: emit_opcode_i32_f32");
  } else if (node->type == NODE_INT) {

    I_CHK(get_node_value_i32_from_gene(node), "compile_alterable_element");
    i32 i = result_i32.result;
    B_CHK(emit_opcode_i32(compilation, LOAD, MEM_SEG_CONSTANT, i),
          "compile_alterable_element");
  } else if (node->type == NODE_VECTOR) {

    if (node_vector_length(node) == 2) {
      E_CHK(compile_2d_from_gene(compilation, node),
            "compile_alterable_element: compile_2d_from_gene");
    } else {
      E_CHK(compile_vector(compilation, node), "compile_alterable_element: compile_vector");
    }
  }

  return NONE;
}

// ast is a NODE_VECTOR of length 2
//
sen_error compile_2d(sen_compilation* compilation, sen_node* ast) {
  sen_result_node result_node;
  sen_error       err;
  bool            use_gene = alterable(ast);

  for (sen_node* node = safe_first_child(ast); node != NULL; node = safe_next(node)) {
    if (use_gene) {
      E_CHK(compile_alterable_element(compilation, node),
            "compile_2d: compile_alterable_element");
    } else {
      N_CHK(compile(compilation, node), "compile_2d: compile");
    }
  }

  sen_result_bytecode result_bytecode;
  B_CHK(emit_opcode_i32(compilation, SQUISH2, 0, 0), "compile_2d");

  return NONE;
}

sen_error compile_vector(sen_compilation* compilation, sen_node* ast) {
  sen_result_node     result_node;
  sen_error           err;
  sen_result_bytecode result_bytecode;

  // pushing from the VOID means creating a new, empty vector
  result_bytecode = emit_opcode_i32(compilation, LOAD, MEM_SEG_VOID, 0);
  if (is_result_bytecode_error(result_bytecode)) {
    return result_bytecode.error;
  }

  // if this is an alterable vector, we'll have to pull values for each element
  // from the genes
  bool use_gene = alterable(ast);

  for (sen_node* node = safe_first_child(ast); node != NULL; node = safe_next(node)) {
    if (use_gene) {
      E_CHK(compile_alterable_element(compilation, node),
            "compile_2d: compile_alterable_element");
    } else {
      N_CHK(compile(compilation, node), "compile_2d: compile");
    }

    B_CHK(emit_opcode_i32(compilation, APPEND, 0, 0), "compile_vector");
  }

  return NONE;
}

sen_result_node compile_user_defined_name(sen_compilation* compilation, sen_node* ast,
                                          i32 iname) {
  sen_result_bytecode result_bytecode;

  sen_option_i32 option_i32 = get_local_mapping(compilation, iname);
  if (is_option_i32_some(option_i32)) {
    i32 local_mapping = option_i32.some;
    emit_opcode_i32_name(compilation, LOAD, MEM_SEG_LOCAL, local_mapping);
    return result_node_ok(safe_next(ast));
  }

  // check arguments if we're in a function
  if (compilation->current_fn_info) {
    option_i32 = get_argument_mapping(compilation->current_fn_info, iname);
    if (is_option_i32_some(option_i32)) {
      i32 argument_mapping = option_i32.some;
      result_bytecode =
          emit_opcode_i32(compilation, LOAD, MEM_SEG_ARGUMENT, argument_mapping);
      if (is_result_bytecode_error(result_bytecode)) {
        SEN_ERROR("compile_user_defined_name");
        return result_node_error(result_bytecode.error);
      }
      return result_node_ok(safe_next(ast));
    }
  }

  option_i32 = get_global_mapping(compilation, iname);
  if (is_option_i32_some(option_i32)) {
    i32 global_mapping = option_i32.some;
    result_bytecode    = emit_opcode_i32(compilation, LOAD, MEM_SEG_GLOBAL, global_mapping);
    if (is_result_bytecode_error(result_bytecode)) {
      return result_node_error(result_bytecode.error);
    }
    return result_node_ok(safe_next(ast));
  }

  // could be a keyword such as linear, ease-in etc
  if (iname >= KEYWORD_START && iname < KEYWORD_START + MAX_KEYWORD_LOOKUPS) {
    emit_opcode_i32_name(compilation, LOAD, MEM_SEG_CONSTANT, iname);
    return result_node_ok(safe_next(ast));
  }

  SEN_ERROR("unknown mapping for: %s", wlut_get_word(compilation->program->word_lut, iname));
  return result_node_error(ERROR_COMPILER_UNKNOWN_MAPPING_FOR_NAME);

  // todo: this used to have the following line:
  // return safe_next(ast);
  // is it allowed to return safe_next if there's an unknown mapping???
}

sen_result_node compile(sen_compilation* compilation, sen_node* ast) {
  sen_error           err;
  sen_result_node     result_node;
  sen_result_bytecode result_bytecode;
  sen_result_i32      result_i32;
  sen_node*           n;
  i32                 i;
  f32                 f;

  if (ast->type == NODE_LIST) {
    if (alterable(ast) && is_node_colour_constructor(ast)) {
      sen_var* var = ast->gene->var;
      sen_var  arg0;

      // we have an alterable colour constructor so just
      // load in the colour value stored in the gene
      //
      i32_as_var(&arg0, MEM_SEG_CONSTANT);

      result_bytecode = emit_opcode(compilation, LOAD, &arg0, var);
      if (is_result_bytecode_error(result_bytecode)) {
        SEN_ERROR("compile: emit_opcode");
        return result_node_error(result_bytecode.error);
      }
    } else {
      if (alterable(ast)) {
        warn_if_alterable("NODE_LIST", ast);
        SEN_ERROR("given an alterable list that wasn't a colour constructor???");
      }
      n = safe_first(ast->value.first_child);

      sen_node* name = n;
      if (name->type != NODE_NAME) {
        SEN_ERROR("compile: get_fn_info_index requires a name node");
        node_pretty_print("get_fn_info_index non-name node:", name, NULL);
        return result_node_error(ERROR_COMPILER_EXPECTED_NAME_NODE);
      }

      sen_option_i32 option_i32 = get_fn_info_index(name, compilation->program);
      if (is_option_i32_some(option_i32)) {
        i32 fn_info_index = option_i32.some;
        err               = compile_fn_invocation(compilation, name, fn_info_index);
        if (is_error(err)) {
          SEN_ERROR("compile: compile_fn_invocation");
          return result_node_error(err);
        }
      } else {
        result_node = compile(compilation, name);
        if (is_result_node_error(result_node)) {
          SEN_ERROR("compile: compile");
          return result_node;
        }
      }

      return result_node_ok(safe_next(ast));
    }
  }
  if (ast->type == NODE_FLOAT) {
    sen_result_f32 result_f32 = get_node_value_f32(ast);
    if (is_result_f32_error(result_f32)) {
      SEN_ERROR("compile: get_node_value_f32");
      return result_node_error(result_f32.error);
    }
    f = result_f32.result;

    result_bytecode = emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, f);
    if (is_result_bytecode_error(result_bytecode)) {
      return result_node_error(result_bytecode.error);
    }
    return result_node_ok(safe_next(ast));
  }
  if (ast->type == NODE_INT) {
    result_i32 = get_node_value_i32(ast);
    if (is_result_i32_error(result_i32)) {
      SEN_ERROR("compile: get_node_value_i32");
      return result_node_error(result_i32.error);
    }
    i = result_i32.result;

    result_bytecode = emit_opcode_i32(compilation, LOAD, MEM_SEG_CONSTANT, i);
    if (is_result_bytecode_error(result_bytecode)) {
      return result_node_error(result_bytecode.error);
    }

    return result_node_ok(safe_next(ast));
  }
  if (ast->type == NODE_VECTOR) {
    if (node_vector_length(ast) == 2) {
      err = compile_2d(compilation, ast);
      if (is_error(err)) {
        SEN_ERROR("compile: compile_2d");
        return result_node_error(err);
      }

    } else {
      err = compile_vector(compilation, ast);
      if (is_error(err)) {
        SEN_ERROR("compile: compile_vector");
        return result_node_error(err);
      }
    }
    return result_node_ok(safe_next(ast));
  }
  if (ast->type == NODE_NAME) {

    result_i32 = get_node_value_i32(ast);
    if (is_result_i32_error(result_i32)) {
      SEN_ERROR("compile: get_node_value_i32");
      return result_node_error(result_i32.error);
    }
    i32 iname = result_i32.result;

    if (iname >= WORD_START && iname < WORD_START + MAX_WORD_LOOKUPS) { // a user defined name
      return compile_user_defined_name(compilation, ast, iname);
    } else if (iname >= KEYWORD_START && iname < KEYWORD_START + MAX_KEYWORD_LOOKUPS) {

      switch (iname) {
      case INAME_DEFINE:
        err = compile_define(compilation, ast, MEM_SEG_LOCAL);
        if (is_error(err)) {
          return result_node_error(err);
        }
        return result_node_ok(NULL);
      case INAME_IF:
        err = compile_if(compilation, ast);
        if (is_error(err)) {
          return result_node_error(err);
        }
        return result_node_ok(safe_next(ast));
      case INAME_LOOP:
        err = compile_loop(compilation, ast);
        if (is_error(err)) {
          return result_node_error(err);
        }
        return result_node_ok(safe_next(ast));
      case INAME_FENCE:
        err = compile_fence(compilation, ast);
        if (is_error(err)) {
          return result_node_error(err);
        }
        return result_node_ok(safe_next(ast));
      case INAME_ON_MATRIX_STACK:
        err = compile_on_matrix_stack(compilation, ast);
        if (is_error(err)) {
          return result_node_error(err);
        }
        return result_node_ok(safe_next(ast));
      case INAME_FN:
        err = compile_fn(compilation, ast);
        if (is_error(err)) {
          return result_node_error(err);
        }
        return result_node_ok(safe_next(ast));
      case INAME_PLUS:
        err = compile_math(compilation, ast, ADD);
        if (is_error(err)) {
          return result_node_error(err);
        }
        return result_node_ok(safe_next(ast));
      case INAME_MINUS:
        // TODO: differentiate between neg and sub?
        err = compile_math(compilation, ast, SUB);
        if (is_error(err)) {
          return result_node_error(err);
        }
        return result_node_ok(safe_next(ast));
      case INAME_MULT:
        err = compile_math(compilation, ast, MUL);
        if (is_error(err)) {
          return result_node_error(err);
        }
        return result_node_ok(safe_next(ast));
      case INAME_DIVIDE:
        err = compile_math(compilation, ast, DIV);
        if (is_error(err)) {
          return result_node_error(err);
        }
        return result_node_ok(safe_next(ast));
      case INAME_MOD:
        err = compile_math(compilation, ast, MOD);
        if (is_error(err)) {
          return result_node_error(err);
        }
        return result_node_ok(safe_next(ast));
      case INAME_EQUAL:
        err = compile_math(compilation, ast, EQ);
        if (is_error(err)) {
          return result_node_error(err);
        }
        return result_node_ok(safe_next(ast));
      case INAME_LT:
        err = compile_math(compilation, ast, LT);
        if (is_error(err)) {
          return result_node_error(err);
        }
        return result_node_ok(safe_next(ast));
      case INAME_GT:
        err = compile_math(compilation, ast, GT);
        if (is_error(err)) {
          return result_node_error(err);
        }
        return result_node_ok(safe_next(ast));
      case INAME_AND:
        err = compile_math(compilation, ast, AND);
        if (is_error(err)) {
          return result_node_error(err);
        }
        return result_node_ok(safe_next(ast));
      case INAME_OR:
        err = compile_math(compilation, ast, OR);
        if (is_error(err)) {
          return result_node_error(err);
        }
        return result_node_ok(safe_next(ast));
      case INAME_NOT:
        err = compile_next_one(compilation, ast);
        if (is_error(err)) {
          return result_node_error(err);
        }
        result_bytecode = emit_opcode_i32(compilation, NOT, 0, 0);
        if (is_result_bytecode_error(result_bytecode)) {
          return result_node_error(result_bytecode.error);
        }
        return result_node_ok(safe_next(ast));
      case INAME_SQRT:
        err = compile_next_one(compilation, ast);
        if (is_error(err)) {
          return result_node_error(err);
        }
        result_bytecode = emit_opcode_i32(compilation, SQRT, 0, 0);
        if (is_result_bytecode_error(result_bytecode)) {
          return result_node_error(result_bytecode.error);
        }
        return result_node_ok(safe_next(ast));
      case INAME_ADDRESS_OF:
        err = compile_address_of(compilation, ast);
        if (is_error(err)) {
          return result_node_error(err);
        }
        return result_node_ok(safe_next(ast));
      case INAME_FN_CALL:
        err = compile_fn_call(compilation, ast);
        if (is_error(err)) {
          return result_node_error(err);
        }
        return result_node_ok(safe_next(ast));
      case INAME_VECTOR_APPEND:
        err = compile_vector_append(compilation, ast);
        if (is_error(err)) {
          return result_node_error(err);
        }
        return result_node_ok(safe_next(ast));
      case INAME_QUOTE:
        err = compile_quote(compilation, ast);
        if (is_error(err)) {
          return result_node_error(err);
        }
        return result_node_ok(safe_next(ast));
      default:
        // look up the name as a user defined variable
        // normally get here when a script contains variables
        // that have the same name as common parameters.
        // e.g. r, g, b, alpha
        // or if we're passing a pre-defined argument value
        // e.g. linear in (bezier line-width-mapping: linear)
        return compile_user_defined_name(compilation, ast, iname);
      };
    } else if (iname >= NATIVE_START && iname < NATIVE_START + MAX_NATIVE_LOOKUPS) {
      // NATIVE

      // note: how to count the stack delta? how many pop voids are required?
      i32       num_args = 0;
      sen_node* args     = safe_next(ast); // pairs of label/value declarations
      while (args != NULL) {
        sen_node* label = args;
        sen_node* value = safe_next(label);

        result_i32 = get_node_value_i32(label);
        if (is_result_i32_error(result_i32)) {
          SEN_ERROR("compile: get_node_value_i32");
          return result_node_error(result_i32.error);
        }
        i               = result_i32.result;
        result_bytecode = emit_opcode_i32(compilation, LOAD, MEM_SEG_CONSTANT, i);
        if (is_result_bytecode_error(result_bytecode)) {
          return result_node_error(result_bytecode.error);
        }

        result_node = compile(compilation, value);
        if (is_result_node_error(result_node)) {
          return result_node;
        }

        num_args++;
        args = safe_next(value);
      }

      result_bytecode = emit_opcode_i32(compilation, NATIVE, iname, num_args);
      if (is_result_bytecode_error(result_bytecode)) {
        return result_node_error(result_bytecode.error);
      }

      // modify opcode_offset according to how many args were given
      compilation->opcode_offset -= (num_args * 2) - 1;

      return result_node_ok(safe_next(ast));
    }
  }

  return result_node_ok(safe_next(ast));
}

bool is_list_beginning_with(sen_node* ast, i32 index) {
  if (ast->type != NODE_LIST) {
    return false;
  }

  sen_node* keyword = safe_first(ast->value.first_child);
  if (keyword->type == NODE_NAME && keyword->value.i == index) {
    return true;
  }

  return false;
}

sen_error compile_global_bind_node(sen_compilation* compilation, i32 iname, sen_node* node) {
  sen_result_node result_node;
  sen_result_i32  result_i32;

  N_CHK(compile(compilation, node), "compile_global_bind_node");
  I_CHK(store_globally(compilation, iname), "compile_global_bind_node");

  return NONE;
}

sen_error compile_global_bind_i32(sen_compilation* compilation, i32 iname, i32 value) {
  sen_result_bytecode result_bytecode;
  sen_result_i32      result_i32;

  B_CHK(emit_opcode_i32(compilation, LOAD, MEM_SEG_CONSTANT, value),
        "compile_global_bind_i32");
  I_CHK(store_globally(compilation, iname), "compile_global_bind_i32");

  return NONE;
}

sen_error compile_global_bind_f32(sen_compilation* compilation, i32 iname, f32 value) {
  sen_result_bytecode result_bytecode;
  sen_result_i32      result_i32;

  B_CHK(emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, value),
        "compile_global_bind_f32");
  I_CHK(store_globally(compilation, iname), "compile_global_bind_f32");

  return NONE;
}

sen_error compile_global_bind_col(sen_compilation* compilation, i32 iname, f32 r, f32 g,
                                  f32 b, f32 a) {
  sen_result_bytecode result_bytecode;
  sen_result_i32      result_i32;

  sen_var mem_location, colour_arg;

  i32_as_var(&mem_location, MEM_SEG_CONSTANT);

  colour_arg.type    = VAR_COLOUR;
  colour_arg.value.i = RGB;

  colour_arg.f32_array[0] = r;
  colour_arg.f32_array[1] = g;
  colour_arg.f32_array[2] = b;
  colour_arg.f32_array[3] = a;

  B_CHK(emit_opcode(compilation, LOAD, &mem_location, &colour_arg),
        "compile_global_bind_col: emit_opcode");
  I_CHK(store_globally(compilation, iname), "compile_global_bind_col");

  return NONE;
}

sen_error append_name(sen_compilation* compilation, i32 iname) {
  sen_result_bytecode result_bytecode;

  B_CHK(emit_opcode_i32_name(compilation, LOAD, MEM_SEG_CONSTANT, iname), "append_name");
  B_CHK(emit_opcode_i32(compilation, APPEND, 0, 0), "append_name");

  return NONE;
}

sen_error compile_global_bind_procedural_presets(sen_compilation* compilation) {
  sen_result_bytecode result_bytecode;
  sen_result_i32      result_i32;
  sen_error           err;

  // create a vector
  B_CHK(emit_opcode_i32(compilation, LOAD, MEM_SEG_VOID, 0),
        "compile_global_bind_procedural_presets");

  // append the names
  E_CHK(append_name(compilation, INAME_CHROME), "CHROME");
  E_CHK(append_name(compilation, INAME_HOTLINE_MIAMI), "HOTLINE_MIAMI");
  E_CHK(append_name(compilation, INAME_KNIGHT_RIDER), "KNIGHT_RIDER");
  E_CHK(append_name(compilation, INAME_MARS), "MARS");
  E_CHK(append_name(compilation, INAME_RAINBOW), "RAINBOW");
  E_CHK(append_name(compilation, INAME_ROBOCOP), "ROBOCOP");
  E_CHK(append_name(compilation, INAME_TRANSFORMERS), "TRANSFORMERS");

  I_CHK(store_globally(compilation, INAME_COL_PROCEDURAL_FN_PRESETS),
        "compile_global_bind_procedural_presets");

  return NONE;
}

sen_error compile_global_bind_ease_presets(sen_compilation* compilation) {
  sen_result_bytecode result_bytecode;
  sen_result_i32      result_i32;
  sen_error           err;

  // create a vector
  B_CHK(emit_opcode_i32(compilation, LOAD, MEM_SEG_VOID, 0),
        "compile_global_bind_ease_presets");

  // append the names
  E_CHK(append_name(compilation, INAME_LINEAR), "LINEAR");
  E_CHK(append_name(compilation, INAME_EASE_QUICK), "EASE_QUICK");
  E_CHK(append_name(compilation, INAME_EASE_SLOW_IN), "EASE_SLOW_IN");
  E_CHK(append_name(compilation, INAME_EASE_SLOW_IN_OUT), "EASE_SLOW_IN_OUT");
  E_CHK(append_name(compilation, INAME_EASE_QUADRATIC_IN), "EASE_QUADRATIC_IN");
  E_CHK(append_name(compilation, INAME_EASE_QUADRATIC_OUT), "EASE_QUADRATIC_OUT");
  E_CHK(append_name(compilation, INAME_EASE_QUADRATIC_IN_OUT), "EASE_QUADRATIC_IN_OUT");
  E_CHK(append_name(compilation, INAME_EASE_CUBIC_IN), "EASE_CUBIC_IN");
  E_CHK(append_name(compilation, INAME_EASE_CUBIC_OUT), "EASE_CUBIC_OUT");
  E_CHK(append_name(compilation, INAME_EASE_CUBIC_IN_OUT), "EASE_CUBIC_IN_OUT");
  E_CHK(append_name(compilation, INAME_EASE_QUARTIC_IN), "EASE_QUARTIC_IN");
  E_CHK(append_name(compilation, INAME_EASE_QUARTIC_OUT), "EASE_QUARTIC_OUT");
  E_CHK(append_name(compilation, INAME_EASE_QUARTIC_IN_OUT), "EASE_QUARTIC_IN_OUT");
  E_CHK(append_name(compilation, INAME_EASE_QUINTIC_IN), "EASE_QUINTIC_IN");
  E_CHK(append_name(compilation, INAME_EASE_QUINTIC_OUT), "EASE_QUINTIC_OUT");
  E_CHK(append_name(compilation, INAME_EASE_QUINTIC_IN_OUT), "EASE_QUINTIC_IN_OUT");
  E_CHK(append_name(compilation, INAME_EASE_SIN_IN), "EASE_SIN_IN");
  E_CHK(append_name(compilation, INAME_EASE_SIN_OUT), "EASE_SIN_OUT");
  E_CHK(append_name(compilation, INAME_EASE_SIN_IN_OUT), "EASE_SIN_IN_OUT");
  E_CHK(append_name(compilation, INAME_EASE_CIRCULAR_IN), "EASE_CIRCULAR_IN");
  E_CHK(append_name(compilation, INAME_EASE_CIRCULAR_OUT), "EASE_CIRCULAR_OUT");
  E_CHK(append_name(compilation, INAME_EASE_CIRCULAR_IN_OUT), "EASE_CIRCULAR_IN_OUT");
  E_CHK(append_name(compilation, INAME_EASE_EXPONENTIAL_IN), "EASE_EXPONENTIAL_IN");
  E_CHK(append_name(compilation, INAME_EASE_EXPONENTIAL_OUT), "EASE_EXPONENTIAL_OUT");
  E_CHK(append_name(compilation, INAME_EASE_EXPONENTIAL_IN_OUT), "EASE_EXPONENTIAL_IN_OUT");
  E_CHK(append_name(compilation, INAME_EASE_ELASTIC_IN), "EASE_ELASTIC_IN");
  E_CHK(append_name(compilation, INAME_EASE_ELASTIC_OUT), "EASE_ELASTIC_OUT");
  E_CHK(append_name(compilation, INAME_EASE_ELASTIC_IN_OUT), "EASE_ELASTIC_IN_OUT");
  E_CHK(append_name(compilation, INAME_EASE_BACK_IN), "EASE_BACK_IN");
  E_CHK(append_name(compilation, INAME_EASE_BACK_OUT), "EASE_BACK_OUT");
  E_CHK(append_name(compilation, INAME_EASE_BACK_IN_OUT), "EASE_BACK_IN_OUT");
  E_CHK(append_name(compilation, INAME_EASE_BOUNCE_IN), "EASE_BOUNCE_IN");
  E_CHK(append_name(compilation, INAME_EASE_BOUNCE_OUT), "EASE_BOUNCE_OUT");
  E_CHK(append_name(compilation, INAME_EASE_BOUNCE_IN_OUT), "EASE_BOUNCE_IN_OUT");

  I_CHK(store_globally(compilation, INAME_EASE_PRESETS), "compile_global_bind_ease_presets");

  return NONE;
}

// NOTE: each entry in compile_preamble should have a corresponding entry here
sen_error register_top_level_preamble(sen_compilation* compilation) {
  sen_result_i32 result_i32;

  I_CHK(add_global_mapping(compilation, INAME_GEN_INITIAL), "register_top_level_preamble");

  I_CHK(add_global_mapping(compilation, INAME_CANVAS_WIDTH), "register_top_level_preamble");
  I_CHK(add_global_mapping(compilation, INAME_CANVAS_HEIGHT), "register_top_level_preamble");

  I_CHK(add_global_mapping(compilation, INAME_MATH_TAU), "register_top_level_preamble");

  I_CHK(add_global_mapping(compilation, INAME_WHITE), "register_top_level_preamble");
  I_CHK(add_global_mapping(compilation, INAME_BLACK), "register_top_level_preamble");
  I_CHK(add_global_mapping(compilation, INAME_RED), "register_top_level_preamble");
  I_CHK(add_global_mapping(compilation, INAME_GREEN), "register_top_level_preamble");
  I_CHK(add_global_mapping(compilation, INAME_BLUE), "register_top_level_preamble");
  I_CHK(add_global_mapping(compilation, INAME_YELLOW), "register_top_level_preamble");
  I_CHK(add_global_mapping(compilation, INAME_MAGENTA), "register_top_level_preamble");
  I_CHK(add_global_mapping(compilation, INAME_CYAN), "register_top_level_preamble");

  I_CHK(add_global_mapping(compilation, INAME_COL_PROCEDURAL_FN_PRESETS),
        "register_top_level_preamble");
  I_CHK(add_global_mapping(compilation, INAME_EASE_PRESETS), "register_top_level_preamble");

  return NONE;
}

sen_error compile_preamble(sen_compilation* compilation) {
  sen_error err;
  // ********************************************************************************
  // NOTE: each entry should have a corresponding entry in
  // register_top_level_preamble
  // ********************************************************************************
  E_CHK(compile_global_bind_i32(compilation, INAME_GEN_INITIAL, 0), "compile_preamble");
  E_CHK(compile_global_bind_f32(compilation, INAME_CANVAS_WIDTH, 1000.0f),
        "compile_preamble");
  E_CHK(compile_global_bind_f32(compilation, INAME_CANVAS_HEIGHT, 1000.0f),
        "compile_preamble");

  E_CHK(compile_global_bind_f32(compilation, INAME_MATH_TAU, TAU), "compile_preamble");

  E_CHK(compile_global_bind_col(compilation, INAME_WHITE, 1.0f, 1.0f, 1.0f, 1.0f),
        "compile_preamble");
  E_CHK(compile_global_bind_col(compilation, INAME_BLACK, 0.0f, 0.0f, 0.0f, 1.0f),
        "compile_preamble");

  E_CHK(compile_global_bind_col(compilation, INAME_RED, 1.0f, 0.0f, 0.0f, 1.0f),
        "compile_preamble");
  E_CHK(compile_global_bind_col(compilation, INAME_GREEN, 0.0f, 1.0f, 0.0f, 1.0f),
        "compile_preamble");
  E_CHK(compile_global_bind_col(compilation, INAME_BLUE, 0.0f, 0.0f, 1.0f, 1.0f),
        "compile_preamble");

  E_CHK(compile_global_bind_col(compilation, INAME_YELLOW, 1.0f, 1.0f, 0.0f, 1.0f),
        "compile_preamble");
  E_CHK(compile_global_bind_col(compilation, INAME_MAGENTA, 1.0f, 0.0f, 1.0f, 1.0f),
        "compile_preamble");
  E_CHK(compile_global_bind_col(compilation, INAME_CYAN, 0.0f, 1.0f, 1.0f, 1.0f),
        "compile_preamble");

  E_CHK(compile_global_bind_procedural_presets(compilation), "compile_preamble");
  E_CHK(compile_global_bind_ease_presets(compilation), "compile_preamble");
  // ********************************************************************************
  // NOTE: each entry should have a corresponding entry in
  // register_top_level_preamble
  // ********************************************************************************

  return NONE;
}

sen_error compile_common_prologue(sen_compilation* compilation, sen_node* ast) {
  sen_error err;

  clear_global_mappings(compilation);
  clear_local_mappings(compilation);
  compilation->current_fn_info = NULL;

  E_CHK(register_top_level_preamble(compilation), "compile_common_prologue");
  E_CHK(register_top_level_fns(compilation, ast), "compile_common_prologue");
  E_CHK(register_top_level_defines(compilation, ast), "compile_common_prologue");

  return NONE;
}

sen_error compile_common_top_level_fns(sen_compilation* compilation, sen_node* ast) {
  sen_result_bytecode result_bytecode = emit_opcode_i32(compilation, JUMP, 0, 0);
  if (is_result_bytecode_error(result_bytecode)) {
    return result_bytecode.error;
  }
  sen_bytecode* start = result_bytecode.result;

  // compile the top-level functions
  sen_node*       n = ast;
  sen_result_node result_node;
  while (n != NULL) {
    if (is_list_beginning_with(n, INAME_FN)) {
      N_CHK(compile(compilation, n), "compile_common_top_level_fns: compile");
      n = result_node.result;
    } else {
      n = safe_next(n);
    }
  }

  // compile the global defines common to all sen programs
  // (e.g. canvas/width)
  // this is where the program will start from
  start->arg0.type    = VAR_INT;
  start->arg0.value.i = compilation->program->code_size;

  return NONE;
}

sen_error compile_common_top_level_defines(sen_compilation* compilation, sen_node* ast) {
  sen_error err;

  sen_node* n = ast;
  while (n != NULL) {
    if (is_list_beginning_with(n, INAME_DEFINE)) {
      E_CHK(compile_define(compilation, safe_first(n->value.first_child), MEM_SEG_GLOBAL),
            "compile_common_top_level_defines: compile_define");
      n = safe_next(n);
    } else {
      n = safe_next(n);
    }
  }

  return NONE;
}

sen_error compile_common_top_level_forms(sen_compilation* compilation, sen_node* ast) {
  sen_node*       n = ast;
  sen_result_node result_node;
  while (n != NULL) {
    if (is_list_beginning_with(n, INAME_FN) == false &&
        is_list_beginning_with(n, INAME_DEFINE) == false) {
      N_CHK(compile(compilation, n), "compile_common_top_level_forms: compile");
      n = result_node.result;
    } else {
      n = safe_next(n);
    }
  }

  return NONE;
}

sen_error compile_common_epilogue(sen_compilation* compilation) {
  sen_result_bytecode result_bytecode = emit_opcode_i32(compilation, STOP, 0, 0);
  if (is_result_bytecode_error(result_bytecode)) {
    return result_bytecode.error;
  }

  // we can now update the addreses used by CALL and CALL_0
  sen_error err;

  E_CHK(correct_function_addresses(compilation),
        "compile_common_epilogue: correct_function_addresses");

  return NONE;
}

// compiles the ast into bytecode for a stack based VM
//
sen_error compile_common(sen_compilation* compilation, sen_node* ast) {
  sen_error err;

  E_CHK(compile_common_prologue(compilation, ast), "compile_common");
  E_CHK(compile_common_top_level_fns(compilation, ast), "compile_common");
  E_CHK(compile_common_top_level_defines(compilation, ast), "compile_common");
  E_CHK(compile_common_top_level_forms(compilation, ast), "compile_common");
  E_CHK(compile_common_epilogue(compilation), "compile_common");

  // SEN_LOG("program compiled: %d lines\n", program->code_size);

  return NONE;
}

// compiles the ast into bytecode for a stack based VM
//
sen_result_program compile_program(sen_program* program, sen_node* ast) {
  g_use_genes = false;

  sen_compilation compilation;
  sen_compilation_init(&compilation, program);

  sen_error err = compile_common(&compilation, ast);
  if (is_error(err)) {
    SEN_ERROR("compile_program: compile_common");
    return result_program_error(err);
  }

  return result_program_ok(compilation.program);
}

sen_result_program compile_program_with_genotype(sen_program* program, sen_word_lut* word_lut,
                                                 sen_node* ast, sen_genotype* genotype) {
  g_use_genes = true;

  sen_error err = genotype_assign_to_ast(word_lut, genotype, ast);
  if (is_error(err)) {
    SEN_ERROR("not all genes were assigned");
    // todo: this should return a sen_result_program;
    // return err;
    return result_program_error(err);
  }

  // ast_pretty_print(ast, word_lut);

  sen_compilation compilation;
  sen_compilation_init(&compilation, program);

  err = compile_common(&compilation, ast);
  if (is_error(err)) {
    SEN_ERROR("compile_program_with_genotype: compile_common");
    return result_program_error(err);
  }

  sen_program* sen_program = compilation.program;

  return result_program_ok(sen_program);
}

sen_result_program compile_program_for_trait(sen_program* program, sen_node* ast,
                                             sen_node* gen_initial_value) {
  sen_error err;
  g_use_genes = false;

  sen_compilation compilation;
  sen_compilation_init(&compilation, program);

  err = compile_common_prologue(&compilation, ast);
  if (is_error(err)) {
    return result_program_error(err);
  }
  err = compile_common_top_level_fns(&compilation, ast);
  if (is_error(err)) {
    return result_program_error(err);
  }

  // this is a sub-program for a trait, bind the initial value to
  // gen/initial-value
  err = compile_global_bind_node(&compilation, INAME_GEN_INITIAL, gen_initial_value);
  if (is_error(err)) {
    return result_program_error(err);
  }

  err = compile_common_top_level_defines(&compilation, ast);
  if (is_error(err)) {
    return result_program_error(err);
  }
  err = compile_common_top_level_forms(&compilation, ast);
  if (is_error(err)) {
    return result_program_error(err);
  }
  err = compile_common_epilogue(&compilation);
  if (is_error(err)) {
    return result_program_error(err);
  }

  return result_program_ok(compilation.program);
}
