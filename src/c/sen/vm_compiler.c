#include "vm_compiler.h"
#include "colour.h"
#include "genetic.h"
#include "keyword_iname.h"
#include "lang.h"
#include "mathutil.h"

#include <string.h>

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

void          compile_vector(sen_compilation* compilation, sen_node* ast);
void          clear_global_mappings(sen_compilation* compilation);
void          clear_local_mappings(sen_compilation* compilation);
void          register_top_level_preamble(sen_compilation* compilation);
void          compile_preamble(sen_compilation* compilation);
sen_bytecode* emit_opcode_i32(sen_compilation* compilation, sen_opcode op, i32 arg0, i32 arg1);

void compiler_subsystem_startup() {
  i32          program_max_size = 100; // ???
  sen_program* program          = program_allocate(program_max_size);

  sen_compilation compilation;
  sen_compilation_init(&compilation, program);

  clear_global_mappings(&compilation);
  clear_local_mappings(&compilation);
  compilation.current_fn_info = NULL;

  register_top_level_preamble(&compilation);
  compile_preamble(&compilation);

  // slap a stop onto the end of this program
  emit_opcode_i32(&compilation, STOP, 0, 0);

  g_preamble_program = program;
}

void compiler_subsystem_shutdown() { program_free(g_preamble_program); }

sen_program* get_preamble_program() { return g_preamble_program; }

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

bool genotype_assign_to_ast(sen_word_lut* word_lut, sen_genotype* genotype, sen_node* ast) {
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
    return false;
  }

  return true;
}

i32 get_node_value_i32_from_gene(sen_node* node) {
  sen_gene* gene = node->gene;
  if (gene == NULL) {
    SEN_ERROR("null gene returned");
    return 0;
  }

  return gene->var->value.i;
}

f32 get_node_value_f32_from_gene(sen_node* node) {
  sen_gene* gene = node->gene;
  if (gene == NULL) {
    SEN_ERROR("null gene returned");
    return 0.0f;
  }

  return gene->var->value.f;
}

bool alterable(sen_node* node) { return node->alterable && g_use_genes; }

i32 get_node_value_i32(sen_node* node) {
  if (alterable(node)) {
    return get_node_value_i32_from_gene(node);
  } else {
    return node->value.i;
  }
}

f32 get_node_value_f32(sen_node* node) {
  if (alterable(node)) {
    return get_node_value_f32_from_gene(node);
  } else {
    return node->value.f;
  }
}

// a temporary message for unimplemented alterable nodes
void warn_if_alterable(char* msg, sen_node* node) {
  if (node->alterable) {
    SEN_ERROR("warn_if_alterable: %s", msg);
  }
}

sen_bytecode*
emit_opcode(sen_compilation* compilation, sen_opcode op, sen_var* arg0, sen_var* arg1) {
  sen_program* program = compilation->program;

  if (program->code_size >= program->code_max_size) {
    SEN_ERROR("%s %d program has reached max size", __FILE__, __LINE__);
    return NULL;
  }

  sen_bytecode* b = &(program->code[program->code_size++]);
  b->op           = op;
  var_copy(&(b->arg0), arg0);
  var_copy(&(b->arg1), arg1);

  compilation->opcode_offset += opcode_offset[op];

  return b;
}

// emits an <opcode, i32, i32> triplet
sen_bytecode* emit_opcode_i32(sen_compilation* compilation, sen_opcode op, i32 arg0, i32 arg1) {
  sen_program* program = compilation->program;

  if (program->code_size >= program->code_max_size) {
    SEN_ERROR("%s %d program has reached max size", __FILE__, __LINE__);
    return NULL;
  }

  sen_bytecode* b = &(program->code[program->code_size++]);
  b->op           = op;
  i32_as_var(&(b->arg0), arg0);
  i32_as_var(&(b->arg1), arg1);

  compilation->opcode_offset += opcode_offset[op];

  return b;
}

// emits an <opcode, i32, name> triplet
sen_bytecode*
emit_opcode_i32_name(sen_compilation* compilation, sen_opcode op, i32 arg0, i32 name) {
  sen_program* program = compilation->program;

  if (program->code_size >= program->code_max_size) {
    SEN_ERROR("%s %d program has reached max size", __FILE__, __LINE__);
    return NULL;
  }

  sen_bytecode* b = &(program->code[program->code_size++]);
  b->op           = op;
  i32_as_var(&(b->arg0), arg0);
  name_as_var(&(b->arg1), name);

  compilation->opcode_offset += opcode_offset[op];

  return b;
}

// emits an <opcode, i32, f32> triplet
sen_bytecode* emit_opcode_i32_f32(sen_compilation* compilation, sen_opcode op, i32 arg0, f32 arg1) {
  sen_program* program = compilation->program;

  if (program->code_size >= program->code_max_size) {
    SEN_ERROR("%s %d program has reached max size", __FILE__, __LINE__);
    return NULL;
  }

  sen_bytecode* b = &(program->code[program->code_size++]);
  b->op           = op;
  i32_as_var(&(b->arg0), arg0);
  f32_as_var(&(b->arg1), arg1);

  compilation->opcode_offset += opcode_offset[op];

  return b;
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

i32 add_local_mapping(sen_compilation* compilation, i32 word_lut_value) {
  for (i32 i = 0; i < MEMORY_LOCAL_SIZE; i++) {
    if (compilation->local_mappings[i] == -1) {
      compilation->local_mappings[i] = word_lut_value;
      return i;
    }
  }

  SEN_ERROR("add_local_mapping failed: increase MEMORY_LOCAL_SIZE from %d", MEMORY_LOCAL_SIZE);
  return -1;
}

// we want a local mapping that's going to be used to store an internal variable
// (e.g. during a fence loop)
// note: it's up to the caller to manage this reference
i32 add_internal_local_mapping(sen_compilation* compilation) {
  for (i32 i = 0; i < MEMORY_LOCAL_SIZE; i++) {
    if (compilation->local_mappings[i] == -1) {
      compilation->local_mappings[i] = -2;
      return i;
    }
  }

  SEN_ERROR("add_internal_local_mapping failed: increase MEMORY_LOCAL_SIZE from %d",
            MEMORY_LOCAL_SIZE);
  return -1;
}

i32 get_local_mapping(sen_compilation* compilation, i32 word_lut_value) {
  for (i32 i = 0; i < MEMORY_LOCAL_SIZE; i++) {
    if (compilation->local_mappings[i] == word_lut_value) {
      return i;
    }
  }

  return -1;
}

void clear_global_mappings(sen_compilation* compilation) {
  for (i32 i = 0; i < MEMORY_GLOBAL_SIZE; i++) {
    compilation->global_mappings[i] = -1;
  }
}

i32 add_global_mapping(sen_compilation* compilation, i32 word_lut_value) {
  for (i32 i = 0; i < MEMORY_GLOBAL_SIZE; i++) {
    if (compilation->global_mappings[i] == -1) {
      compilation->global_mappings[i] = word_lut_value;
      return i;
    }
  }

  SEN_ERROR("add_global_mapping failed: increase MEMORY_GLOBAL_SIZE from %d", MEMORY_GLOBAL_SIZE);
  return -1;
}

i32 get_global_mapping(sen_compilation* compilation, i32 word_lut_value) {
  for (i32 i = 0; i < MEMORY_GLOBAL_SIZE; i++) {
    if (compilation->global_mappings[i] == word_lut_value) {
      return i;
    }
  }

  return -1;
}

i32 get_argument_mapping(sen_fn_info* fn_info, i32 word_lut_value) {
  for (i32 i = 0; i < MAX_NUM_ARGUMENTS; i++) {
    if (fn_info->argument_offsets[i] == -1) {
      return -1;
    }
    if (fn_info->argument_offsets[i] == word_lut_value) {
      return (i * 2) + 1;
    }
  }
  return -1;
}

// returns the index into program->fn_info that represents this function
i32 get_fn_info_index(sen_node* node, sen_program* program) {
  if (node->type != NODE_NAME) {
    SEN_ERROR("get_fn_info_index not given a name node");
    node_pretty_print("get_fn_info_index non-name node:", node, NULL);
    return -1;
  }

  i32 name = node->value.i;

  for (i32 i = 0; i < MAX_TOP_LEVEL_FUNCTIONS; i++) {
    if (program->fn_info[i].active == false) {
      return -1;
    }
    if (program->fn_info[i].fn_name == name) {
      return i;
    }
  }

  SEN_ERROR("get_fn_info_index unable to find fn_info for a function");
  return -1;
}

sen_fn_info* get_fn_info(sen_node* node, sen_program* program) {
  if (node->type != NODE_NAME) {
    return NULL;
  }

  i32 name = node->value.i;

  for (i32 i = 0; i < MAX_TOP_LEVEL_FUNCTIONS; i++) {
    if (program->fn_info[i].active == false) {
      return NULL;
    }
    if (program->fn_info[i].fn_name == name) {
      return &(program->fn_info[i]);
    }
  }
  return NULL;
}

sen_node* compile(sen_compilation* compilation, sen_node* ast);

i32 node_vector_length(sen_node* vector_node) {
  i32 length = 0;
  for (sen_node* node = safe_first(vector_node->value.first_child); node != NULL;
       node           = safe_next(node)) {
    length++;
  }
  return length;
}

bool all_children_have_type(sen_node* parent, sen_node_type type) {
  if (parent->type != NODE_VECTOR && parent->type != NODE_LIST) {
    SEN_ERROR("all_children_have_type need a vector or list");
    return false;
  }

  sen_node* child = parent->value.first_child;
  while (child != NULL) {
    if (child->type != type) {
      return false;
    }
    child = safe_next(child);
  }

  return true;
}

i32 count_children(sen_node* parent) {
  if (parent->type != NODE_VECTOR && parent->type != NODE_LIST) {
    SEN_ERROR("count_children need a vector or list");
    return 0;
  }

  i32       count = 0;
  sen_node* child = safe_first(parent->value.first_child);
  while (child != NULL) {
    count++;
    child = safe_next(child);
  }

  return count;
}

i32 store_locally(sen_compilation* compilation, i32 iname) {
  i32 address = get_local_mapping(compilation, iname);
  if (address == -1) {
    address = add_local_mapping(compilation, iname);
    if (address == -1) {
      // failed to allocate
      SEN_ERROR("store_locally: allocation failure");
    }
  }
  emit_opcode_i32(compilation, STORE, MEM_SEG_LOCAL, address);

  return address;
}

i32 store_globally(sen_compilation* compilation, i32 iname) {
  i32 address = get_global_mapping(compilation, iname);
  if (address == -1) {
    address = add_global_mapping(compilation, iname);
    if (address == -1) {
      // failed to allocate
      SEN_ERROR("store_globally: allocation failure");
    }
  }
  emit_opcode_i32(compilation, STORE, MEM_SEG_GLOBAL, address);

  return address;
}

i32 store_from_stack_to_memory(sen_compilation*        compilation,
                               sen_node*               node,
                               sen_memory_segment_type memory_segment_type) {
  if (memory_segment_type == MEM_SEG_LOCAL) {
    return store_locally(compilation, node->value.i);
  } else if (memory_segment_type == MEM_SEG_GLOBAL) {
    return store_globally(compilation, node->value.i);
  } else {
    SEN_ERROR("store_from_stack_to_memory: unknown memory_segment_type: %d", memory_segment_type);
  }

  return -1;
}

sen_node* compile_define(sen_compilation*        compilation,
                         sen_node*               ast,
                         sen_memory_segment_type memory_segment_type) {
  sen_node* lhs_node = safe_next(ast);
  sen_node* value_node;
  i32       i, m;

  while (lhs_node != NULL) {

    value_node = safe_next(lhs_node);
    compile(compilation, value_node);

    if (lhs_node->type == NODE_NAME) {
      // define foo 10
      m = store_from_stack_to_memory(compilation, lhs_node, memory_segment_type);
      if (m == -1) {
        SEN_ERROR("compile_define: allocation failure in define");
        return NULL;
      }
    } else if (lhs_node->type == NODE_VECTOR) {
      // define [a b] (something-that-returns-a-vector ...)

      // check if we can use the PILE opcode
      if (all_children_have_type(lhs_node, NODE_NAME)) {
        i32 num_children = count_children(lhs_node);

        // PILE will stack the elements in the rhs vector in order,
        // so the lhs values have to be popped in reverse order
        emit_opcode_i32(compilation, PILE, num_children, 0);
        compilation->opcode_offset += num_children - 1;

        sen_node* child = safe_first(lhs_node->value.first_child);

        for (i = 1; i < num_children; i++) {
          child = safe_next(child);
        }
        for (i = 0; i < num_children; i++) {
          m = store_from_stack_to_memory(compilation, child, memory_segment_type);
          if (m == -1) {
            SEN_ERROR("compile_define: allocation failure during destructure");
            return NULL;
          }
          child = safe_prev(child);
        }
        /*
                  while (child != NULL) {
                  store_from_stack_to_memory(program, child,
           memory_segment_type); child = safe_next(child);
                  }
        */

      } else {
        // this may be recursive
        SEN_LOG("todo: push each item onto stack using nth");
      }

    } else {
      SEN_ERROR("compile_define lhs should be a name or a list");
    }

    lhs_node = safe_next(value_node);
  }

  return NULL;
}

void compile_if(sen_compilation* compilation, sen_node* ast) {
  // if (> 200 100) 12 24
  // ^
  sen_node* if_node   = safe_next(ast);
  sen_node* then_node = safe_next(if_node);
  sen_node* else_node = safe_next(then_node); // could be NULL

  compile(compilation, if_node);

  // insert jump to after the 'then' node if not true
  i32           addr_jump_then = compilation->program->code_size;
  sen_bytecode* bc_jump_then   = emit_opcode_i32(compilation, JUMP_IF, 0, 0);

  // the offset after the if
  i32 offset_after_if = compilation->opcode_offset;

  compile(compilation, then_node);

  i32 offset_after_then = compilation->opcode_offset;

  if (else_node) {
    // logically we're now going to go down one of possibly two paths
    // so we can't just continue to add the compilation->opcode_offset since that
    // would result in the offset taking both of the conditional's paths

    compilation->opcode_offset = offset_after_if;

    // insert a bc_jump_else opcode
    i32           addr_jump_else = compilation->program->code_size;
    sen_bytecode* bc_jump_else   = emit_opcode_i32(compilation, JUMP, 0, 0);

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
}

// compiles everything after the current ast point
void compile_rest(sen_compilation* compilation, sen_node* ast) {
  ast = safe_next(ast);
  while (ast) {
    ast = compile(compilation, ast);
  }
}

// compiles the next node after the current ast point
void compile_next_one(sen_compilation* compilation, sen_node* ast) {
  ast = safe_next(ast);
  compile(compilation, ast);
}

void compile_math(sen_compilation* compilation, sen_node* ast, sen_opcode opcode) {
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

  ast = compile(compilation, ast); // compile the first argument
  while (ast) {
    ast = compile(compilation, ast); // compile the next argument
    emit_opcode_i32(compilation, opcode, 0, 0);
  }
}

void compile_address_of(sen_compilation* compilation, sen_node* ast) {
  sen_node* fn_name = safe_next(ast);

  // fn_name should be a defined function's name
  // it will be known at compile time

  if (fn_name->type != NODE_NAME) {
    SEN_ERROR("compile_address_of given non-function-name argument");
    return;
  }

  sen_fn_info* fn_info = get_fn_info(fn_name, compilation->program);
  if (fn_info == NULL) {
    SEN_ERROR("address-of could not find function");
    return;
  }

  // store the index into program->fn_info in the program
  emit_opcode_i32(compilation, LOAD, MEM_SEG_CONSTANT, fn_info->index);

  return;
}

//   (fn-call (aj z: 44))
void compile_fn_call(sen_compilation* compilation, sen_node* ast) {
  sen_node* invocation = safe_next(ast);

  // fn_name should be a defined function's name
  // it will be known at compile time

  if (invocation->type != NODE_LIST) {
    SEN_ERROR("compile_fn_call given non-list to invoke");
    return;
  }

  warn_if_alterable("compile_fn_call invocation", invocation);

  sen_node* fn_info_index = safe_first(invocation->value.first_child);

  // place the fn_info_index onto the stack so that CALL_F can find the function
  // offset and num args
  compile(compilation, fn_info_index);
  emit_opcode_i32(compilation, CALL_F, 0, 0);

  // compile the rest of the arguments

  // overwrite the default arguments with the actual arguments given by the fn
  // invocation
  sen_node* args = safe_next(fn_info_index); // pairs of label/value declarations
  while (args != NULL) {
    sen_node* label = args;
    sen_node* value = safe_next(label);

    // push value
    compile(compilation, value);
    compile(compilation, fn_info_index); // push the actual fn_info index so that
                                         // the _FLU opcode can find it

    i32 label_i = get_node_value_i32(label);
    emit_opcode_i32(compilation, STORE_F, MEM_SEG_ARGUMENT, label_i);

    args = safe_next(value);
  }

  // place the fn_info_index onto the stack so that CALL_F_0 can find the
  // function's body offset
  compile(compilation, fn_info_index);
  emit_opcode_i32(compilation, CALL_F_0, 0, 0);

  return;
}

void compile_vector_append(sen_compilation* compilation, sen_node* ast) {
  // (vector/append vector value)

  sen_node* vector = safe_next(ast);
  compile(compilation, vector);

  sen_node* value = safe_next(vector);
  compile(compilation, value);

  emit_opcode_i32(compilation, APPEND, 0, 0);

  if (vector->type == NODE_NAME) {

    i32 vector_i = get_node_value_i32(vector);

    i32 address = get_local_mapping(compilation, vector_i);
    if (address != -1) {
      emit_opcode_i32(compilation, STORE, MEM_SEG_LOCAL, address);
      return;
    }

    address = get_global_mapping(compilation, vector_i);
    if (address != -1) {
      emit_opcode_i32(compilation, STORE, MEM_SEG_GLOBAL, address);
      return;
    }

    SEN_ERROR("compile_vector_append: can't find local or global variable");
  }
}

void compile_vector_in_quote(sen_compilation* compilation, sen_node* ast) {
  // pushing from the VOID means creating a new, empty vector
  emit_opcode_i32(compilation, LOAD, MEM_SEG_VOID, 0);

  warn_if_alterable("compile_vector_in_quote", ast);
  for (sen_node* node = safe_first(ast->value.first_child); node != NULL; node = safe_next(node)) {
    // slightly hackish
    // if this is a form like: '(red green blue)
    // the compiler should output the names rather than the colours that are
    // actually referenced (compile_user_defined_name would genereate a
    // MEM_SEG_GLOBAL LOAD code)
    //
    if (node->type == NODE_NAME) {
      emit_opcode_i32_name(compilation, LOAD, MEM_SEG_CONSTANT, node->value.i);
    } else {
      compile(compilation, node);
    }
    emit_opcode_i32(compilation, APPEND, 0, 0);
  }
}

void compile_quote(sen_compilation* compilation, sen_node* ast) {
  sen_node* quoted_form = safe_next(ast);
  if (quoted_form->type == NODE_LIST) {
    // compile each entry individually, don't treat the list as a normal
    // function invocation
    compile_vector_in_quote(compilation, quoted_form);
  } else {
    if (quoted_form->type == NODE_NAME) {
      emit_opcode_i32_name(compilation, LOAD, MEM_SEG_CONSTANT, quoted_form->value.i);
    } else {
      compile(compilation, quoted_form);
    }
  }
}

void compile_loop(sen_compilation* compilation, sen_node* ast) {
  sen_node* parameters_node = safe_next(ast);
  if (parameters_node->type != NODE_LIST) {
    SEN_ERROR("expected a list that defines step parameters");
    return;
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
      return;
    }
  } else {
    use_to = true;
  }

  // set looping variable x to 'from' value
  if (have_from) {
    compile(compilation, from_node);
  } else {
    // else default to 0
    emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, 0.0f);
  }

  i32 looper_address = store_from_stack_to_memory(compilation, name_node, MEM_SEG_LOCAL);
  if (looper_address == -1) {
    SEN_ERROR("compile_loop: allocation failure");
    return;
  }

  // compare looping variable against exit condition
  // and jump if looping variable >= exit value
  i32 addr_loop_start = compilation->program->code_size;
  emit_opcode_i32(compilation, LOAD, MEM_SEG_LOCAL, looper_address);

  if (use_to) {
    // so jump if looping variable >= exit value
    compile(compilation, to_node);
    emit_opcode_i32(compilation, LT, 0, 0);
  } else {
    // so jump if looping variable > exit value
    compile(compilation, upto_node);
    emit_opcode_i32(compilation, GT, 0, 0);
    emit_opcode_i32(compilation, NOT, 0, 0);
  }

  i32           addr_exit_check = compilation->program->code_size;
  sen_bytecode* bc_exit_check   = emit_opcode_i32(compilation, JUMP_IF, 0, 0);

  i32 pre_body_opcode_offset = compilation->opcode_offset;

  // compile the body forms (woooaaaoohhh body form, body form for yoooouuuu)
  compile_rest(compilation, parameters_node);

  i32 post_body_opcode_offset = compilation->opcode_offset;
  i32 opcode_delta            = post_body_opcode_offset - pre_body_opcode_offset;

  // pop off any values that the body might leave on the stack
  for (i32 i = 0; i < opcode_delta; i++) {
    emit_opcode_i32(compilation, STORE, MEM_SEG_VOID, 0);
  }

  // increment the looping variable
  emit_opcode_i32(compilation, LOAD, MEM_SEG_LOCAL, looper_address);

  if (have_increment) {
    compile(compilation, increment_node);
  } else {
    emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, 1.0f);
  }
  emit_opcode_i32(compilation, ADD, 0, 0);
  emit_opcode_i32(compilation, STORE, MEM_SEG_LOCAL, looper_address);

  // loop back to the comparison
  emit_opcode_i32(compilation, JUMP, -(compilation->program->code_size - addr_loop_start), 0);
  bc_exit_check->arg0.value.i = compilation->program->code_size - addr_exit_check;
}

void compile_fence(sen_compilation* compilation, sen_node* ast) {
  // (fence (x from: 0 to: 5 quantity: 5) (+ 42 38))

  sen_node* parameters_node = safe_next(ast);
  if (parameters_node->type != NODE_LIST) {
    SEN_ERROR("expected a list that defines fence parameters");
    return;
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
  i32 quantity_address = add_internal_local_mapping(compilation);
  if (have_num) {
    compile(compilation, num_node);
  } else {
    // else default to 2
    emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, 2.0f);
  }
  emit_opcode_i32(compilation, STORE, MEM_SEG_LOCAL, quantity_address);

  // reserve a memory location in local memory for a counter from 0 to quantity
  i32 counter_address = add_internal_local_mapping(compilation);
  emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, 0.0f);
  emit_opcode_i32(compilation, STORE, MEM_SEG_LOCAL, counter_address);

  // delta that needs to be added at every iteration
  //
  // (to - from) / (quantity - 1)
  if (have_to) {
    compile(compilation, to_node);
  } else {
    // else default to 1
    emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, 1.0f);
  }
  if (have_from) {
    compile(compilation, from_node);
  } else {
    // else default to 0
    emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, 0.0f);
  }
  emit_opcode_i32(compilation, SUB, 0, 0);

  compile(compilation, num_node);
  emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, 1.0f);
  emit_opcode_i32(compilation, SUB, 0, 0);
  emit_opcode_i32(compilation, DIV, 0, 0);

  i32 delta_address = add_internal_local_mapping(compilation);
  emit_opcode_i32(compilation, STORE, MEM_SEG_LOCAL, delta_address);

  // set looping variable x to 'from' value
  if (have_from) {
    compile(compilation, from_node);
  } else {
    // else default to 0
    emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, 0.0f);
  }

  i32 from_address = add_internal_local_mapping(compilation);
  emit_opcode_i32(compilation, STORE, MEM_SEG_LOCAL, from_address);

  // store the starting 'from' value in the locally scoped variable
  emit_opcode_i32(compilation, LOAD, MEM_SEG_LOCAL, from_address);
  i32 looper_address = store_from_stack_to_memory(compilation, name_node, MEM_SEG_LOCAL);
  if (looper_address == -1) {
    SEN_ERROR("compile_fence: allocation failure");
    return;
  }

  // compare looping variable against exit condition
  // and jump if looping variable >= exit value
  i32 addr_loop_start = compilation->program->code_size;

  // load from counter address
  emit_opcode_i32(compilation, LOAD, MEM_SEG_LOCAL, counter_address);

  // load from quantity address
  emit_opcode_i32(compilation, LOAD, MEM_SEG_LOCAL, quantity_address);

  // exit check
  emit_opcode_i32(compilation, LT, 0, 0);

  i32           addr_exit_check = compilation->program->code_size;
  sen_bytecode* bc_exit_check   = emit_opcode_i32(compilation, JUMP_IF, 0, 0);

  // looper = from + (counter * delta)
  emit_opcode_i32(compilation, LOAD, MEM_SEG_LOCAL, from_address);
  emit_opcode_i32(compilation, LOAD, MEM_SEG_LOCAL, counter_address);
  emit_opcode_i32(compilation, LOAD, MEM_SEG_LOCAL, delta_address);
  emit_opcode_i32(compilation, MUL, 0, 0);
  emit_opcode_i32(compilation, ADD, 0, 0);
  emit_opcode_i32(compilation, STORE, MEM_SEG_LOCAL, looper_address);

  i32 pre_body_opcode_offset = compilation->opcode_offset;

  // compile the body forms (woooaaaoohhh body form, body form for yoooouuuu)
  compile_rest(compilation, parameters_node);

  i32 post_body_opcode_offset = compilation->opcode_offset;
  i32 opcode_delta            = post_body_opcode_offset - pre_body_opcode_offset;

  // pop off any values that the body might leave on the stack
  for (i32 i = 0; i < opcode_delta; i++) {
    emit_opcode_i32(compilation, STORE, MEM_SEG_VOID, 0);
  }

  // increment counter
  emit_opcode_i32(compilation, LOAD, MEM_SEG_LOCAL, counter_address);
  emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, 1.0f);
  emit_opcode_i32(compilation, ADD, 0, 0);
  emit_opcode_i32(compilation, STORE, MEM_SEG_LOCAL, counter_address);

  // loop back to the comparison
  emit_opcode_i32(compilation, JUMP, -(compilation->program->code_size - addr_loop_start), 0);
  bc_exit_check->arg0.value.i = compilation->program->code_size - addr_exit_check;
}

void compile_on_matrix_stack(sen_compilation* compilation, sen_node* ast) {
  emit_opcode_i32(compilation, MTX_LOAD, 0, 0);
  compile_rest(compilation, ast);
  emit_opcode_i32(compilation, MTX_STORE, 0, 0);
}

void register_top_level_fns(sen_compilation* compilation, sen_node* ast) {
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
      return;
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
}

void register_names_in_define(sen_compilation* compilation, sen_node* lhs) {
  warn_if_alterable("register_names_in_define lhs", lhs);
  if (lhs->type == NODE_NAME) {
    // (define foo 42)
    i32 global_address = get_global_mapping(compilation, lhs->value.i);
    if (global_address == -1) {
      global_address = add_global_mapping(compilation, lhs->value.i);
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
}

void register_top_level_defines(sen_compilation* compilation, sen_node* ast) {
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
    while (lhs != NULL) {
      register_names_in_define(compilation, lhs);
      lhs = safe_next(lhs); // points to the value
      lhs = safe_next(lhs); // points to the next define statement if there multiple
    }

    ast = safe_next(ast);
  }
}

/*
  invoking code will first CALL into the arg_address to setup the default values
  for all args the fn code will then return back to the invoking code invoking
  code will then overwrite specific data in arg memory invoking code will then
  CALL into the body_address
*/
void compile_fn(sen_compilation* compilation, sen_node* ast) {
  // fn (adder a: 0 b: 0) (+ a b)

  clear_local_mappings(compilation);

  // (adder a: 0 b: 0)
  sen_node* signature = safe_next(ast);

  warn_if_alterable("compile_fn signature", signature);
  sen_node*    fn_name = safe_first(signature->value.first_child);
  sen_fn_info* fn_info = get_fn_info(fn_name, compilation->program);
  if (fn_info == NULL) {
    SEN_ERROR("Unable to find fn_info for function %d", fn_name->value.i);
    return;
  }

  compilation->current_fn_info = fn_info;

  // -------------
  // the arguments
  // -------------

  fn_info->arg_address               = compilation->program->code_size;
  sen_node* args                     = safe_next(fn_name); // pairs of label/value declarations
  i32       num_args                 = 0;
  i32       counter                  = 0;
  i32       argument_offsets_counter = 0;
  while (args != NULL) {
    sen_node* label   = args;
    i32       label_i = get_node_value_i32(label);
    sen_node* value   = safe_next(label);

    // get_argument_mapping
    fn_info->argument_offsets[argument_offsets_counter++] = label_i;

    // push pairs of label+value values onto the args stack
    emit_opcode_i32(compilation, LOAD, MEM_SEG_CONSTANT, label_i);
    emit_opcode_i32(compilation, STORE, MEM_SEG_ARGUMENT, counter++);

    compile(compilation, value);
    emit_opcode_i32(compilation, STORE, MEM_SEG_ARGUMENT, counter++);

    num_args++;
    args = safe_next(value);
  }

  fn_info->num_args = num_args;

  emit_opcode_i32(compilation, RET_0, 0, 0);

  // --------
  // the body
  // --------

  fn_info->body_address = compilation->program->code_size;

  // (+ a b)
  compile_rest(compilation, signature);

  // Don't need any STORE, MEM_SEG_VOID instructions as the RET will
  // pop the frame and blow the stack

  emit_opcode_i32(compilation, RET, 0, 0);

  compilation->current_fn_info = NULL;
}

void correct_function_addresses(sen_compilation* compilation) {
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
        return;
      }
      offset->arg1.value.i = fn_info->arg_address;

      offset = bc - 1;
      if (offset->op != LOAD && offset->arg0.value.i != MEM_SEG_CONSTANT) {
        SEN_ERROR("correct_function_addresses expected a 'LOAD CONST' 1 "
                  "opcode before a CALL");
        return;
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
        return;
      }
      offset->arg1.value.i = fn_info->body_address;
    }

    if (bc->op == PLACEHOLDER_STORE) {
      bc->op = STORE;

      // opcode's arg0 is the fn_info_index and arg1 is the label_value
      fn_info_index = bc->arg0.value.i;
      fn_info       = &(compilation->program->fn_info[fn_info_index]);
      label_value   = bc->arg1.value.i;

      i32 data_index   = get_argument_mapping(fn_info, label_value);
      bc->arg1.value.i = data_index;

      if (data_index != -1) {
        bc->arg0.value.i = MEM_SEG_ARGUMENT;
      } else {
        // otherwise this function was invoked with a parameter that is doesn't
        // use so just essentially turn these ops into no-ops
        bc->arg0.value.i = MEM_SEG_VOID;
      }
    }

    bc++;
  }
}

void compile_fn_invocation(sen_compilation* compilation, sen_node* ast, i32 fn_info_index) {
  // ast == adder a: 10 b: 20

  // NOTE: CALL and CALL_0 get their function offsets and num args from the
  // stack so add some placeholder LOAD CONST opcodes and fill the CALL, CALL_0
  // with fn_info indexes that can later be used to fill in the LOAD CONST
  // opcodes with their correct offsets doing it this way enables functions to
  // call other functions that are declared later in the script

  // prepare the MEM_SEG_ARGUMENT with default values

  emit_opcode_i32(compilation, LOAD, MEM_SEG_CONSTANT,
                  666); // for the function address
  emit_opcode_i32(compilation, LOAD, MEM_SEG_CONSTANT,
                  667); // for the num args
  emit_opcode_i32(compilation, CALL, fn_info_index, fn_info_index);

  // overwrite the default arguments with the actual arguments given by the fn
  // invocation
  sen_node* args = safe_next(ast); // pairs of label/value declarations
  while (args != NULL) {
    sen_node* label   = args;
    i32       label_i = get_node_value_i32(label);

    sen_node* value = safe_next(label);

    // push value
    compile(compilation, value);
    emit_opcode_i32(compilation, PLACEHOLDER_STORE, fn_info_index, label_i);

    args = safe_next(value);
  }

  // call the body of the function
  emit_opcode_i32(compilation, LOAD, MEM_SEG_CONSTANT,
                  668); // for the function body address
  emit_opcode_i32(compilation, CALL_0, fn_info_index, fn_info_index);
}

// ast is a NODE_VECTOR of length 2
//
void compile_2d_from_gene(sen_compilation* compilation, sen_node* ast) {
  sen_gene* gene = ast->gene;

  f32 a = gene->var->f32_array[0];
  f32 b = gene->var->f32_array[1];

  emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, a);
  emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, b);

  emit_opcode_i32(compilation, SQUISH2, 0, 0);
}

void compile_alterable_element(sen_compilation* compilation, sen_node* node) {
  if (node->type == NODE_FLOAT) {
    f32 f = get_node_value_f32_from_gene(node);
    emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, f);
  } else if (node->type == NODE_INT) {
    i32 i = get_node_value_i32_from_gene(node);
    emit_opcode_i32(compilation, LOAD, MEM_SEG_CONSTANT, i);
  } else if (node->type == NODE_VECTOR) {
    if (node_vector_length(node) == 2) {
      compile_2d_from_gene(compilation, node);
    } else {
      compile_vector(compilation, node);
    }
  }
}

// ast is a NODE_VECTOR of length 2
//
void compile_2d(sen_compilation* compilation, sen_node* ast) {

  bool use_gene = alterable(ast);

  for (sen_node* node = safe_first_child(ast); node != NULL; node = safe_next(node)) {
    if (use_gene) {
      compile_alterable_element(compilation, node);
    } else {
      compile(compilation, node);
    }
  }
  emit_opcode_i32(compilation, SQUISH2, 0, 0);
}

void compile_vector(sen_compilation* compilation, sen_node* ast) {
  // pushing from the VOID means creating a new, empty vector
  emit_opcode_i32(compilation, LOAD, MEM_SEG_VOID, 0);

  // if this is an alterable vector, we'll have to pull values for each element
  // from the genes
  bool use_gene = alterable(ast);

  for (sen_node* node = safe_first_child(ast); node != NULL; node = safe_next(node)) {
    if (use_gene) {
      compile_alterable_element(compilation, node);
    } else {
      compile(compilation, node);
    }
    emit_opcode_i32(compilation, APPEND, 0, 0);
  }
}

sen_node* compile_user_defined_name(sen_compilation* compilation, sen_node* ast, i32 iname) {
  i32 local_mapping = get_local_mapping(compilation, iname);
  if (local_mapping != -1) {
    emit_opcode_i32_name(compilation, LOAD, MEM_SEG_LOCAL, local_mapping);
    return safe_next(ast);
  }

  // check arguments if we're in a function
  if (compilation->current_fn_info) {
    i32 argument_mapping = get_argument_mapping(compilation->current_fn_info, iname);
    if (argument_mapping != -1) {
      emit_opcode_i32(compilation, LOAD, MEM_SEG_ARGUMENT, argument_mapping);
      return safe_next(ast);
    }
  }

  i32 global_mapping = get_global_mapping(compilation, iname);
  if (global_mapping != -1) {
    emit_opcode_i32(compilation, LOAD, MEM_SEG_GLOBAL, global_mapping);
    return safe_next(ast);
  }

  // could be a keyword such as linear, ease-in etc
  if (iname >= KEYWORD_START && iname < KEYWORD_START + MAX_KEYWORD_LOOKUPS) {
    emit_opcode_i32_name(compilation, LOAD, MEM_SEG_CONSTANT, iname);
    return safe_next(ast);
  }

  SEN_ERROR("unknown mapping for: %s", wlut_get_word(compilation->program->word_lut, iname));
  return safe_next(ast);
}

sen_node* compile(sen_compilation* compilation, sen_node* ast) {
  sen_node* n;
  i32       i;
  f32       f;

  if (ast->type == NODE_LIST) {
    if (alterable(ast) && is_node_colour_constructor(ast)) {
      sen_var* var = ast->gene->var;
      sen_var  arg0;

      // we have an alterable colour constructor so just
      // load in the colour value stored in the gene
      //
      i32_as_var(&arg0, MEM_SEG_CONSTANT);
      emit_opcode(compilation, LOAD, &arg0, var);

    } else {
      if (alterable(ast)) {
        warn_if_alterable("NODE_LIST", ast);
        SEN_ERROR("given an alterable list that wasn't a colour constructor???");
      }
      n = safe_first(ast->value.first_child);

      i32 fn_info_index = get_fn_info_index(n, compilation->program);
      if (fn_info_index != -1) {
        compile_fn_invocation(compilation, n, fn_info_index);
      } else {
        compile(compilation, n);
      }

      return safe_next(ast);
    }
  }
  if (ast->type == NODE_FLOAT) {
    f = get_node_value_f32(ast);
    emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, f);
    return safe_next(ast);
  }
  if (ast->type == NODE_INT) {
    i = get_node_value_i32(ast);
    emit_opcode_i32(compilation, LOAD, MEM_SEG_CONSTANT, i);
    return safe_next(ast);
  }
  if (ast->type == NODE_VECTOR) {
    if (node_vector_length(ast) == 2) {
      compile_2d(compilation, ast);
    } else {
      compile_vector(compilation, ast);
    }
    return safe_next(ast);
  }
  if (ast->type == NODE_NAME) {

    i32 iname = get_node_value_i32(ast);

    if (iname >= WORD_START && iname < WORD_START + MAX_WORD_LOOKUPS) { // a user defined name
      return compile_user_defined_name(compilation, ast, iname);
    } else if (iname >= KEYWORD_START && iname < KEYWORD_START + MAX_KEYWORD_LOOKUPS) {

      switch (iname) {
      case INAME_DEFINE:
        return compile_define(compilation, ast, MEM_SEG_LOCAL);
      case INAME_IF:
        compile_if(compilation, ast);
        return safe_next(ast);
      case INAME_LOOP:
        compile_loop(compilation, ast);
        return safe_next(ast);
      case INAME_FENCE:
        compile_fence(compilation, ast);
        return safe_next(ast);
      case INAME_ON_MATRIX_STACK:
        compile_on_matrix_stack(compilation, ast);
        return safe_next(ast);
      case INAME_FN:
        compile_fn(compilation, ast);
        return safe_next(ast);
      case INAME_PLUS:
        compile_math(compilation, ast, ADD);
        return safe_next(ast);
      case INAME_MINUS:
        // TODO: differentiate between neg and sub?
        compile_math(compilation, ast, SUB);
        return safe_next(ast);
      case INAME_MULT:
        compile_math(compilation, ast, MUL);
        return safe_next(ast);
      case INAME_DIVIDE:
        compile_math(compilation, ast, DIV);
        return safe_next(ast);
      case INAME_MOD:
        compile_math(compilation, ast, MOD);
        return safe_next(ast);
      case INAME_EQUAL:
        compile_math(compilation, ast, EQ);
        return safe_next(ast);
      case INAME_LT:
        compile_math(compilation, ast, LT);
        return safe_next(ast);
      case INAME_GT:
        compile_math(compilation, ast, GT);
        return safe_next(ast);
      case INAME_AND:
        compile_math(compilation, ast, AND);
        return safe_next(ast);
      case INAME_OR:
        compile_math(compilation, ast, OR);
        return safe_next(ast);
      case INAME_NOT:
        compile_next_one(compilation, ast);
        emit_opcode_i32(compilation, NOT, 0, 0);
        return safe_next(ast);
      case INAME_SQRT:
        compile_next_one(compilation, ast);
        emit_opcode_i32(compilation, SQRT, 0, 0);
        return safe_next(ast);
      case INAME_ADDRESS_OF:
        compile_address_of(compilation, ast);
        return safe_next(ast);
      case INAME_FN_CALL:
        compile_fn_call(compilation, ast);
        return safe_next(ast);
      case INAME_VECTOR_APPEND:
        compile_vector_append(compilation, ast);
        return safe_next(ast);
      case INAME_QUOTE:
        compile_quote(compilation, ast);
        return safe_next(ast);
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

        i = get_node_value_i32(label);
        emit_opcode_i32(compilation, LOAD, MEM_SEG_CONSTANT, i);
        compile(compilation, value);

        num_args++;
        args = safe_next(value);
      }

      emit_opcode_i32(compilation, NATIVE, iname, num_args);

      // modify opcode_offset according to how many args were given
      compilation->opcode_offset -= (num_args * 2) - 1;

      return safe_next(ast);
    }
  }

  return safe_next(ast);
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

void compile_global_bind_node(sen_compilation* compilation, i32 iname, sen_node* node) {
  compile(compilation, node);
  store_globally(compilation, iname);
}

void compile_global_bind_i32(sen_compilation* compilation, i32 iname, i32 value) {
  emit_opcode_i32(compilation, LOAD, MEM_SEG_CONSTANT, value);
  store_globally(compilation, iname);
}

void compile_global_bind_f32(sen_compilation* compilation, i32 iname, f32 value) {
  emit_opcode_i32_f32(compilation, LOAD, MEM_SEG_CONSTANT, value);
  store_globally(compilation, iname);
}

void compile_global_bind_col(sen_compilation* compilation, i32 iname, f32 r, f32 g, f32 b, f32 a) {
  sen_var mem_location, colour_arg;

  i32_as_var(&mem_location, MEM_SEG_CONSTANT);

  colour_arg.type    = VAR_COLOUR;
  colour_arg.value.i = RGB;

  colour_arg.f32_array[0] = r;
  colour_arg.f32_array[1] = g;
  colour_arg.f32_array[2] = b;
  colour_arg.f32_array[3] = a;

  emit_opcode(compilation, LOAD, &mem_location, &colour_arg);

  store_globally(compilation, iname);
}

void append_name(sen_compilation* compilation, i32 iname) {
  emit_opcode_i32_name(compilation, LOAD, MEM_SEG_CONSTANT, iname);
  emit_opcode_i32(compilation, APPEND, 0, 0);
}

void compile_global_bind_procedural_presets(sen_compilation* compilation) {
  // create a vector
  emit_opcode_i32(compilation, LOAD, MEM_SEG_VOID, 0);

  // append the names
  append_name(compilation, INAME_CHROME);
  append_name(compilation, INAME_HOTLINE_MIAMI);
  append_name(compilation, INAME_KNIGHT_RIDER);
  append_name(compilation, INAME_MARS);
  append_name(compilation, INAME_RAINBOW);
  append_name(compilation, INAME_ROBOCOP);
  append_name(compilation, INAME_TRANSFORMERS);

  store_globally(compilation, INAME_COL_PROCEDURAL_FN_PRESETS);
}

// NOTE: each entry in compile_preamble should have a corresponding entry here
void register_top_level_preamble(sen_compilation* compilation) {
  add_global_mapping(compilation, INAME_GEN_INITIAL);

  add_global_mapping(compilation, INAME_CANVAS_WIDTH);
  add_global_mapping(compilation, INAME_CANVAS_HEIGHT);

  add_global_mapping(compilation, INAME_MATH_TAU);

  add_global_mapping(compilation, INAME_WHITE);
  add_global_mapping(compilation, INAME_BLACK);

  add_global_mapping(compilation, INAME_RED);
  add_global_mapping(compilation, INAME_GREEN);
  add_global_mapping(compilation, INAME_BLUE);

  add_global_mapping(compilation, INAME_YELLOW);
  add_global_mapping(compilation, INAME_MAGENTA);
  add_global_mapping(compilation, INAME_CYAN);

  add_global_mapping(compilation, INAME_COL_PROCEDURAL_FN_PRESETS);
}

void compile_preamble(sen_compilation* compilation) {
  // ********************************************************************************
  // NOTE: each entry should have a corresponding entry in
  // register_top_level_preamble
  // ********************************************************************************
  compile_global_bind_i32(compilation, INAME_GEN_INITIAL, 0);

  compile_global_bind_f32(compilation, INAME_CANVAS_WIDTH, 1000.0f);
  compile_global_bind_f32(compilation, INAME_CANVAS_HEIGHT, 1000.0f);

  compile_global_bind_f32(compilation, INAME_MATH_TAU, TAU);

  compile_global_bind_col(compilation, INAME_WHITE, 1.0f, 1.0f, 1.0f, 1.0f);
  compile_global_bind_col(compilation, INAME_BLACK, 0.0f, 0.0f, 0.0f, 1.0f);

  compile_global_bind_col(compilation, INAME_RED, 1.0f, 0.0f, 0.0f, 1.0f);
  compile_global_bind_col(compilation, INAME_GREEN, 0.0f, 1.0f, 0.0f, 1.0f);
  compile_global_bind_col(compilation, INAME_BLUE, 0.0f, 0.0f, 1.0f, 1.0f);

  compile_global_bind_col(compilation, INAME_YELLOW, 1.0f, 1.0f, 0.0f, 1.0f);
  compile_global_bind_col(compilation, INAME_MAGENTA, 1.0f, 0.0f, 1.0f, 1.0f);
  compile_global_bind_col(compilation, INAME_CYAN, 0.0f, 1.0f, 1.0f, 1.0f);

  compile_global_bind_procedural_presets(compilation);
  // ********************************************************************************
  // NOTE: each entry should have a corresponding entry in
  // register_top_level_preamble
  // ********************************************************************************
}

void compile_common_prologue(sen_compilation* compilation, sen_node* ast) {
  clear_global_mappings(compilation);
  clear_local_mappings(compilation);
  compilation->current_fn_info = NULL;

  register_top_level_preamble(compilation);

  // register top-level functions
  register_top_level_fns(compilation, ast);

  // register top-level defines
  register_top_level_defines(compilation, ast);
}

void compile_common_top_level_fns(sen_compilation* compilation, sen_node* ast) {
  sen_bytecode* start = emit_opcode_i32(compilation, JUMP, 0, 0);

  // compile the top-level functions
  sen_node* n = ast;
  while (n != NULL) {
    if (is_list_beginning_with(n, INAME_FN)) {
      n = compile(compilation, n);
    } else {
      n = safe_next(n);
    }
  }

  // compile the global defines common to all sen programs
  // (e.g. canvas/width)
  // this is where the program will start from
  start->arg0.type    = VAR_INT;
  start->arg0.value.i = compilation->program->code_size;
}

void compile_common_top_level_defines(sen_compilation* compilation, sen_node* ast) {
  sen_node* n = ast;
  while (n != NULL) {
    if (is_list_beginning_with(n, INAME_DEFINE)) {
      compile_define(compilation, safe_first(n->value.first_child), MEM_SEG_GLOBAL);
      n = safe_next(n);
    } else {
      n = safe_next(n);
    }
  }
}

void compile_common_top_level_forms(sen_compilation* compilation, sen_node* ast) {
  sen_node* n = ast;
  while (n != NULL) {
    if (is_list_beginning_with(n, INAME_FN) == false &&
        is_list_beginning_with(n, INAME_DEFINE) == false) {
      n = compile(compilation, n);
    } else {
      n = safe_next(n);
    }
  }
}

void compile_common_epilogue(sen_compilation* compilation) {
  emit_opcode_i32(compilation, STOP, 0, 0);

  // we can now update the addreses used by CALL and CALL_0
  correct_function_addresses(compilation);
}

// compiles the ast into bytecode for a stack based VM
//
void compile_common(sen_compilation* compilation, sen_node* ast) {
  compile_common_prologue(compilation, ast);
  compile_common_top_level_fns(compilation, ast);
  compile_common_top_level_defines(compilation, ast);
  compile_common_top_level_forms(compilation, ast);
  compile_common_epilogue(compilation);

  // SEN_LOG("program compiled: %d lines\n", program->code_size);
}

// compiles the ast into bytecode for a stack based VM
//
sen_program* compile_program(sen_program* program, sen_node* ast) {
  g_use_genes = false;

  sen_compilation compilation;
  sen_compilation_init(&compilation, program);

  compile_common(&compilation, ast);

  return compilation.program;
}

sen_program*
compile_program_with_genotype(sen_program* program, sen_word_lut* word_lut, sen_node* ast, sen_genotype* genotype) {
  g_use_genes = true;

  bool all_genes_assigned = genotype_assign_to_ast(word_lut, genotype, ast);
  if (all_genes_assigned == false) {
    SEN_ERROR("not all genes were assigned");
    return NULL;
  }

  // ast_pretty_print(ast, word_lut);

  sen_compilation compilation;
  sen_compilation_init(&compilation, program);

  compile_common(&compilation, ast);

  return compilation.program;
}

sen_program* compile_program_for_trait(sen_program* program,
                                       sen_node*    ast,
                                       sen_node*    gen_initial_value) {

  g_use_genes = false;

  sen_compilation compilation;
  sen_compilation_init(&compilation, program);

  compile_common_prologue(&compilation, ast);
  compile_common_top_level_fns(&compilation, ast);

  // this is a sub-program for a trait, bind the initial value to gen/initial-value
  compile_global_bind_node(&compilation, INAME_GEN_INITIAL, gen_initial_value);

  compile_common_top_level_defines(&compilation, ast);
  compile_common_top_level_forms(&compilation, ast);
  compile_common_epilogue(&compilation);

  return compilation.program;
}
