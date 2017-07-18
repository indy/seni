#include "seni_vm_compiler.h"

#include <string.h>

i32 opcode_offset[] = {
#define OPCODE(_,offset) offset,
#include "seni_opcodes.h"
#undef OPCODE
};

seni_node *safe_next(seni_node *expr)
{
  seni_node *sibling = expr->next;
  while(sibling && (sibling->type == NODE_WHITESPACE ||
                    sibling->type == NODE_COMMENT)) {
    sibling = sibling->next;
  }

  return sibling;
}

seni_node *safe_prev(seni_node *expr)
{
  seni_node *sibling = expr->prev;
  while(sibling && (sibling->type == NODE_WHITESPACE ||
                    sibling->type == NODE_COMMENT)) {
    sibling = sibling->prev;
  }

  return sibling;
}

// todo: replace with var_move
void var_dup(seni_var *dest, seni_var *src)
{
  if (dest == src) {
    return;
  }

  dest->type = src->type;
  dest->f32_array[0] = src->f32_array[0];
  dest->f32_array[1] = src->f32_array[1];
  dest->f32_array[2] = src->f32_array[2];
  dest->f32_array[3] = src->f32_array[3];

  seni_value_in_use using = get_value_in_use(src->type);
  
  if (using == USE_I) {
    dest->value.i = src->value.i;
  } else if (using == USE_F) {
    dest->value.f = src->value.f;
  } else if (using == USE_L) {
    dest->value.l = src->value.l;
  } else if (using == USE_V) {
    if (src->type == VAR_VECTOR) {
      dest->value.v = src->value.v;
    } else {
      SENI_ERROR("var_dup USE_V but non-VAR_VECTOR?\n");
    }
  } else {
    SENI_ERROR("unknown seni_value_in_use for var_dup");
  }
}

seni_bytecode *program_emit_opcode(seni_program *program, seni_opcode op, seni_var *arg0, seni_var *arg1)
{
  if (program->code_size >= program->code_max_size) {
    SENI_ERROR("%s %d program has reached max size", __FILE__, __LINE__);
    return NULL;
  }
  
  seni_bytecode *b = &(program->code[program->code_size++]);
  b->op = op;
  var_dup(&(b->arg0), arg0);
  var_dup(&(b->arg1), arg1);

  program->opcode_offset += opcode_offset[op];

  return b;
}

// emits an <opcode, i32, i32> triplet
seni_bytecode *program_emit_opcode_i32(seni_program *program, seni_opcode op, i32 arg0, i32 arg1)
{
  if (program->code_size >= program->code_max_size) {
    SENI_ERROR("%s %d program has reached max size", __FILE__, __LINE__);
    return NULL;
  }
  
  seni_bytecode *b = &(program->code[program->code_size++]);
  b->op = op;
  i32_as_var(&(b->arg0), arg0);
  i32_as_var(&(b->arg1), arg1);

  program->opcode_offset += opcode_offset[op];

  return b;
}

// emits an <opcode, i32, f32> triplet
seni_bytecode *program_emit_opcode_i32_f32(seni_program *program, seni_opcode op, i32 arg0, f32 arg1)
{
  if (program->code_size >= program->code_max_size) {
    SENI_ERROR("%s %d program has reached max size", __FILE__, __LINE__);
    return NULL;
  }
  
  seni_bytecode *b = &(program->code[program->code_size++]);
  b->op = op;
  i32_as_var(&(b->arg0), arg0);
  f32_as_var(&(b->arg1), arg1);

  program->opcode_offset += opcode_offset[op];

  return b;
}

// **************************************************
// Compiler
// **************************************************


void clear_local_mappings(seni_program *program)
{
  for (i32 i=0; i < MEMORY_LOCAL_SIZE; i++) {
    program->local_mappings[i] = -1;
  }
}

i32 add_local_mapping(seni_program *program, i32 wlut_value)
{
  for (i32 i=0; i < MEMORY_LOCAL_SIZE; i++) {
    if(program->local_mappings[i] == -1) {
      program->local_mappings[i] = wlut_value;
      return i;
    }
  }

  SENI_ERROR("add_local_mapping failed: increase MEMORY_LOCAL_SIZE from %d", MEMORY_LOCAL_SIZE);  
  return -1;
}

i32 get_local_mapping(seni_program *program, i32 wlut_value)
{
  for (i32 i=0; i < MEMORY_LOCAL_SIZE; i++) {
    if(program->local_mappings[i] == wlut_value) {
      return i;
    }
  }

  return -1;
}

void clear_global_mappings(seni_program *program)
{
  for (i32 i=0; i < MEMORY_GLOBAL_SIZE; i++) {
    program->global_mappings[i] = -1;
  }
}

i32 add_global_mapping(seni_program *program, i32 wlut_value)
{
  for (i32 i=0; i < MEMORY_GLOBAL_SIZE; i++) {
    if(program->global_mappings[i] == -1) {
      program->global_mappings[i] = wlut_value;
      return i;
    }
  }

  SENI_ERROR("add_global_mapping failed: increase MEMORY_GLOBAL_SIZE from %d", MEMORY_GLOBAL_SIZE);
  return -1;
}

i32 get_global_mapping(seni_program *program, i32 wlut_value)
{
  for (i32 i=0; i < MEMORY_GLOBAL_SIZE; i++) {
    if(program->global_mappings[i] == wlut_value) {
      return i;
    }
  }

  return -1;
}

i32 get_argument_mapping(seni_fn_info *fn_info, i32 wlut_value)
{
  for (i32 i = 0; i < MAX_NUM_ARGUMENTS; i++) {
    if (fn_info->argument_offsets[i] == -1) {
      return -1;
    }
    if (fn_info->argument_offsets[i] == wlut_value) {
      return (i * 2) + 1;
    }
  }
  return -1;
}


// returns the index into program->fn_info that represents this function
i32 get_fn_info_index(seni_node *node, seni_program *program)
{
  if (node->type != NODE_NAME) {
    SENI_ERROR("get_fn_info_index not given a name node");
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

  SENI_ERROR("get_fn_info_index unable to find fn_info for a function");
  return -1;
}

seni_fn_info *get_fn_info(seni_node *node, seni_program *program)
{
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

seni_node *compile(seni_node *ast, seni_program *program);

i32 node_vector_length(seni_node *vector_node)
{
  i32 length = 0;
  for (seni_node *node = vector_node->value.first_child; node != NULL; node = safe_next(node)) {
    length++;
  }
  return length;
}

bool all_children_have_type(seni_node *parent, seni_node_type type)
{
  if (parent->type != NODE_VECTOR && parent->type != NODE_LIST) {
    SENI_ERROR("all_children_have_type need a vector or list");
    return false;
  }

  seni_node *child = parent->value.first_child;
  while (child != NULL) {
    if (child->type != type) {
      return false;
    }
    child = safe_next(child);
  }

  return true;
}

i32 count_children(seni_node *parent)
{
  if (parent->type != NODE_VECTOR && parent->type != NODE_LIST) {
    SENI_ERROR("count_children need a vector or list");
    return 0;
  }

  i32 count = 0;
  seni_node *child = parent->value.first_child;
  while (child != NULL) {
    count++;
    child = safe_next(child);
  }

  return count;
}

i32 pop_from_stack_to_memory(seni_program *program, seni_node *node, seni_memory_segment_type memory_segment_type)
{
  i32 address = -1;
  
  if (memory_segment_type == MEM_SEG_LOCAL) {
    address = get_local_mapping(program, node->value.i);
    if (address == -1) {
      address = add_local_mapping(program, node->value.i);
      if (address == -1) {
        // failed to allocate
        SENI_ERROR("pop_from_stack_to_memory: allocation failure");
      }
    }
    program_emit_opcode_i32(program, STORE, MEM_SEG_LOCAL, address);
  } else if (memory_segment_type == MEM_SEG_GLOBAL) {
    address = get_global_mapping(program, node->value.i);
    if (address == -1) {
      address = add_global_mapping(program, node->value.i);
    }
    program_emit_opcode_i32(program, STORE, MEM_SEG_GLOBAL, address);
  } else {
    SENI_ERROR("pop_from_stack_to_memory: unknown memory_segment_type: %d", memory_segment_type);
  }

  return address;
}

seni_node *compile_define(seni_node *ast, seni_program *program, seni_memory_segment_type memory_segment_type)
{
  seni_node *lhs_node = safe_next(ast);
  seni_node *value_node;
  i32 i, m;

  while (lhs_node != NULL) {

    value_node = safe_next(lhs_node);
    compile(value_node, program);

    if (lhs_node->type == NODE_NAME) {
      // define foo 10
      m = pop_from_stack_to_memory(program, lhs_node, memory_segment_type);
      if (m == -1) {
        SENI_ERROR("compile_define: allocation failure in define");
        return NULL;
      }
    } else if (lhs_node->type == NODE_VECTOR) {
      // define [a b] (something-that-returns-a-vector ...)

      // check if we can use the PILE opcode
      if (all_children_have_type(lhs_node, NODE_NAME)) {
        i32 num_children = count_children(lhs_node);

        // PILE will stack the elements in the rhs vector in order,
        // so the lhs values have to be popped in reverse order
        program_emit_opcode_i32(program, PILE, num_children, 0);
        program->opcode_offset += num_children - 1;

        seni_node *child = lhs_node->value.first_child;


        for (i = 1; i < num_children; i++) {
          child = safe_next(child);
        }
        for (i = 0; i < num_children; i++) {
          m = pop_from_stack_to_memory(program, child, memory_segment_type);
          if (m == -1) {
            SENI_ERROR("compile_define: allocation failure during destructure");
            return NULL;
          }
          child = safe_prev(child);
        }
        /*        
                  while (child != NULL) {
                  pop_from_stack_to_memory(program, child, memory_segment_type);
                  child = safe_next(child);
                  }
        */
        
        
      } else {
        // this may be recursive
        SENI_LOG("todo: push each item onto stack using nth");
      }

    } else {
      SENI_ERROR("compile_define lhs should be a name or a list");
    }

    lhs_node = safe_next(value_node);
  }

  return NULL;
}


void compile_if(seni_node *ast, seni_program *program)
{
  // if (> 200 100) 12 24
  // ^
  seni_node *if_node = safe_next(ast);
  seni_node *then_node = safe_next(if_node);
  seni_node *else_node = safe_next(then_node); // could be NULL

  compile(if_node, program);
  // insert jump to after the 'then' node if not true
  i32 addr_jump_then = program->code_size;
  seni_bytecode *bc_jump_then = program_emit_opcode_i32(program, JUMP_IF, 0, 0);

  compile(then_node, program);

  if (else_node) {
    // insert a bc_jump_else opcode
    i32 addr_jump_else = program->code_size;
    seni_bytecode *bc_jump_else = program_emit_opcode_i32(program, JUMP, 0, 0);

    bc_jump_then->arg0.value.i = program->code_size - addr_jump_then;

    compile(else_node, program);

    bc_jump_else->arg0.value.i = program->code_size - addr_jump_else;
  } else {
    bc_jump_then->arg0.value.i = program->code_size - addr_jump_then;
  }
}

// compiles everything after the current ast point
void compile_rest(seni_node *ast, seni_program *program)
{
  ast = safe_next(ast);
  while (ast) {
    ast = compile(ast, program);
  }
}

// compiles the next node after the current ast point
void compile_next_one(seni_node *ast, seni_program *program)
{
  ast = safe_next(ast);
  compile(ast, program);
}

void compile_math(seni_node *ast, seni_program *program, seni_opcode opcode)
{
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

  ast = compile(ast, program); // compile the first argument
  while (ast) {
    ast = compile(ast, program); // compile the next argument
    program_emit_opcode_i32(program, opcode, 0, 0);
  }
}

void compile_address_of(seni_node *ast, seni_program *program)
{
  seni_node *fn_name = safe_next(ast);

  // fn_name should be a defined function's name
  // it will be known at compile time
  
  if (fn_name->type != NODE_NAME) {
    SENI_ERROR("compile_address_of given non-function-name argument");
    return;
  }

  seni_fn_info *fn_info = get_fn_info(fn_name, program);
  if (fn_info == NULL) {
    SENI_ERROR("address-of could not find function");
    return;
  }

  // store the index into program->fn_info in the program
  program_emit_opcode_i32(program, LOAD, MEM_SEG_CONSTANT, fn_info->index);

  return;
}

//   (fn-call (aj z: 44))
void compile_fn_call(seni_node *ast, seni_program *program)
{
  seni_node *invocation = safe_next(ast);

  // fn_name should be a defined function's name
  // it will be known at compile time
  
  if (invocation->type != NODE_LIST) {
    SENI_ERROR("compile_fn_call given non-list to invoke");
    return;
  }

  seni_node *fn_info_index = invocation->value.first_child;

  // place the fn_info_index onto the stack so that CALL_F can find the function offset and num args
  compile(fn_info_index, program);
  program_emit_opcode_i32(program, CALL_F, 0, 0);

  // compile the rest of the arguments

  // how is this going to work if we don't know the indices of the arguments?
  

  // overwrite the default arguments with the actual arguments given by the fn invocation
  seni_node *args = safe_next(fn_info_index); // pairs of label/value declarations
  while (args != NULL) {
    seni_node *label = args;
    seni_node *value = safe_next(label);

    // push value
    compile(value, program);
    compile(fn_info_index, program); // push the actual fn_info index so that the _FLU opcode can find it
    program_emit_opcode_i32(program, FLU_STORE, MEM_SEG_ARGUMENT, label->value.i);

    args = safe_next(value);
  }


  // place the fn_info_index onto the stack so that CALL_F_0 can find the function's body offset
  compile(fn_info_index, program);
  program_emit_opcode_i32(program, CALL_F_0, 0, 0);
  
  return;
}

void compile_loop(seni_node *ast, seni_program *program)
{
  // (loop (x from: 0 to: 5) (+ 42 38))
  //
  // 0       LOAD    CONST   0
  // 1       STORE     LOCAL   0
  // 2       LOAD    LOCAL   0
  // 3       LOAD    CONST   5
  // 4       LT
  // 5       JUMP_IF +10
  // 6       LOAD    CONST   42
  // 7       LOAD    CONST   38
  // 8       ADD
  // 9       STORE     VOID    0
  // 10      LOAD    LOCAL   0
  // 11      LOAD    CONST   1
  // 12      ADD
  // 13      STORE     LOCAL   0
  // 14      JUMP    -12
  // 15      STOP
  
  seni_node *parameters_node = safe_next(ast);
  if (parameters_node->type != NODE_LIST) {
    SENI_ERROR("expected a list that defines loop parameters");
    return;
  }

  // the looping variable x
  seni_node *name_node = parameters_node->value.first_child;

  seni_node *from_node = NULL;
  seni_node *to_node = NULL;
  seni_node *upto_node = NULL;
  seni_node *increment_node = NULL;
  bool have_from = false;
  bool have_to = false;
  bool have_upto = false;
  bool have_increment = false;
  
  seni_node *node = name_node;

  while (node) {
    node = safe_next(node);  // the label part
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
    if (node->value.i == INAME_INCREMENT) {
      have_increment = true;
      increment_node = safe_next(node);
    }
    node = safe_next(node); // the value part
  }

  bool use_to = false;
  
  if (have_to == false) {
    if (have_upto == false) {
      SENI_ERROR("loop form requires either a 'to' or 'upto' parameter");
      return;
    }
  } else {
    use_to = true;
  }

  // set looping variable x to 'from' value
  if (have_from) {
    compile(from_node, program);
  } else {
    // else default to 0
    program_emit_opcode_i32_f32(program, LOAD, MEM_SEG_CONSTANT, 0.0f);
  }

  i32 looper_address = pop_from_stack_to_memory(program, name_node, MEM_SEG_LOCAL);
  if (looper_address == -1) {
    SENI_ERROR("compile_loop: allocation failure");
    return;
  }

  // compare looping variable against exit condition
  // and jump if looping variable >= exit value
  i32 addr_loop_start = program->code_size;
  program_emit_opcode_i32(program, LOAD, MEM_SEG_LOCAL, looper_address);

  if (use_to) {
    // so jump if looping variable >= exit value
    compile(to_node, program);
    program_emit_opcode_i32(program, LT, 0, 0);
  } else {
    // so jump if looping variable > exit value    
    compile(upto_node, program);
    program_emit_opcode_i32(program, GT, 0, 0);
    program_emit_opcode_i32(program, NOT, 0, 0);
  }

  i32 addr_exit_check = program->code_size;
  seni_bytecode *bc_exit_check = program_emit_opcode_i32(program, JUMP_IF, 0, 0);

  i32 pre_body_opcode_offset = program->opcode_offset;

  // compile the body forms (woooaaaoohhh body form, body form for yoooouuuu)
  compile_rest(parameters_node, program);

  i32 post_body_opcode_offset = program->opcode_offset;
  i32 opcode_delta = post_body_opcode_offset - pre_body_opcode_offset;

  // pop off any values that the body might leave on the stack
  for(i32 i = 0;i < opcode_delta; i++) {
    program_emit_opcode_i32(program, STORE, MEM_SEG_VOID, 0);
  }

  // increment the looping variable
  program_emit_opcode_i32(program, LOAD, MEM_SEG_LOCAL, looper_address);

  if (have_increment) {
    compile(increment_node, program);
  } else {
    program_emit_opcode_i32_f32(program, LOAD, MEM_SEG_CONSTANT, 1.0f);
  }
  program_emit_opcode_i32(program, ADD, 0, 0);
  program_emit_opcode_i32(program, STORE, MEM_SEG_LOCAL, looper_address);

  // loop back to the comparison
  program_emit_opcode_i32(program, JUMP, -(program->code_size - addr_loop_start), 0);
  bc_exit_check->arg0.value.i = program->code_size - addr_exit_check;
}

void compile_on_matrix_stack(seni_node *ast, seni_program *program)
{
  program_emit_opcode_i32(program, MTX_LOAD, 0, 0);
  compile_rest(ast, program);
  program_emit_opcode_i32(program, MTX_STORE, 0, 0);
}

i32 index_of_keyword(const char *keyword, seni_word_lut *wl)
{
  for (i32 i = 0; i < wl->keyword_count; i++) {
    if (strcmp(keyword, wl->keyword[i]) == 0) {
      return KEYWORD_START + i; // the keywords have KEYWORD_START added onto their index
    }
  }

  return -1;
}

void register_top_level_fns(seni_node *ast, seni_program *program)
{
  i32 i;
  i32 num_fns = 0;
  
  // clear all fn data
  for (i = 0; i < MAX_TOP_LEVEL_FUNCTIONS; i++) {
    program->fn_info[i].active = false;
  }

  // register top level fns
  while (ast != NULL) {

    if (ast->type != NODE_LIST) {
      ast = safe_next(ast);
      continue;
    }      

    seni_node *fn_keyword = ast->value.first_child;
    if (!(fn_keyword->type == NODE_NAME && fn_keyword->value.i == INAME_FN)) {
      ast = safe_next(ast);
      continue;
    }

    // (fn (add-up a: 0 b: 0) (+ a b))
    // get the name of the fn
    seni_node *name_and_params = safe_next(fn_keyword);
    if (name_and_params->type != NODE_LIST) {
      ast = safe_next(ast);
      continue;
    }

    seni_node *name = name_and_params->value.first_child;
    i32 name_value = name->value.i;

    // we have a named top-level fn declaration
    seni_fn_info *fn_info = &(program->fn_info[num_fns]);
    num_fns++;
    if (num_fns > MAX_TOP_LEVEL_FUNCTIONS) {
      SENI_ERROR("Script has more than %d top-level functions\n", MAX_TOP_LEVEL_FUNCTIONS);
      return;
    }

    fn_info->active = true;
    fn_info->index = num_fns - 1;
    fn_info->fn_name = name_value;

    // these will be filled in by compile_fn:
    fn_info->num_args = 0;
    for (i = 0; i < MAX_NUM_ARGUMENTS; i++) {
      fn_info->argument_offsets[i] = -1;
    }

    ast = safe_next(ast);
  }
}


void register_names_in_define(seni_node *lhs, seni_program *program)
{
  if (lhs->type == NODE_NAME) {
    // (define foo 42)
    i32 global_address = get_global_mapping(program, lhs->value.i);
    if (global_address == -1) {
      global_address = add_global_mapping(program, lhs->value.i);
    }
  } else if (lhs->type == NODE_LIST || lhs->type == NODE_VECTOR) {
    // (define [a b] (something))
    // (define [a [x y]] (something))
    seni_node *child = lhs->value.first_child;

    while (child != NULL) {
      register_names_in_define(child, program);
      child = safe_next(child);
    }
  }  
}

void register_top_level_defines(seni_node *ast, seni_program *program)
{
  // register top level fns
  while (ast != NULL) {

    if (ast->type != NODE_LIST) {
      ast = safe_next(ast);
      continue;
    }

    seni_node *define_keyword = ast->value.first_child;
    if (!(define_keyword->type == NODE_NAME && define_keyword->value.i == INAME_DEFINE)) {
      ast = safe_next(ast);
      continue;
    }

    seni_node *lhs = safe_next(define_keyword);
    while (lhs != NULL) {
      register_names_in_define(lhs, program);
      lhs = safe_next(lhs); // points to the value
      lhs = safe_next(lhs); // points to the next define statement if there multiple
    }

    ast = safe_next(ast);
  }
}

/*
  invoking code will first CALL into the arg_address to setup the default values for all args
  the fn code will then return back to the invoking code
  invoking code will then overwrite specific data in arg memory
  invoking code will then CALL into the body_address
*/
void compile_fn(seni_node *ast, seni_program *program)
{
  // fn (adder a: 0 b: 0) (+ a b)

  clear_local_mappings(program);

  // (adder a: 0 b: 0)
  seni_node *signature = safe_next(ast);

  seni_node *fn_name = signature->value.first_child;
  seni_fn_info *fn_info = get_fn_info(fn_name, program);
  if (fn_info == NULL) {
    SENI_ERROR("Unable to find fn_info for function %d", fn_name->value.i);
    return;
  }

  program->current_fn_info = fn_info;

  // -------------
  // the arguments
  // -------------
  
  fn_info->arg_address = program->code_size;
  seni_node *args = safe_next(fn_name); // pairs of label/value declarations
  i32 num_args = 0;
  i32 counter = 0;
  i32 argument_offsets_counter = 0;
  while (args != NULL) {
    seni_node *label = args;
    seni_node *value = safe_next(label);

    // get_argument_mapping
    fn_info->argument_offsets[argument_offsets_counter++] = label->value.i;

    // push pairs of label+value values onto the args stack
    program_emit_opcode_i32(program, LOAD, MEM_SEG_CONSTANT, label->value.i);
    program_emit_opcode_i32(program, STORE, MEM_SEG_ARGUMENT, counter++);

    compile(value, program);
    program_emit_opcode_i32(program, STORE, MEM_SEG_ARGUMENT, counter++);

    num_args++;
    args = safe_next(value);
  }

  fn_info->num_args = num_args;

  program_emit_opcode_i32(program, RET_0, 0, 0);

  // --------
  // the body
  // --------

  fn_info->body_address = program->code_size;

  // (+ a b)
  compile_rest(signature, program);

  // Don't need any STORE, MEM_SEG_VOID instructions as the RET will
  // pop the frame and blow the stack

  program_emit_opcode_i32(program, RET, 0, 0);

  program->current_fn_info = NULL;
}

void correct_function_addresses(seni_program *program)
{
  // go through the bytecode fixing up function call addresses

  seni_bytecode *bc = program->code;
  seni_bytecode *offset = NULL;
  i32 fn_info_index, label_value;
  seni_fn_info *fn_info;

  for (i32 i = 0; i < program->code_size; i++) {
    // replace the temporarily stored index in the args of CALL and CALL_0 with the actual values
    if (bc->op == CALL) {
      fn_info_index = bc->arg0.value.i; 
      fn_info = &(program->fn_info[fn_info_index]);

      // the previous two bytecodes will be LOADs of CONST.
      // i - 2 == the address to call
      // i - 1 == the number of arguments used by the function
      offset = bc - 2;
      if (offset->op != LOAD && offset->arg0.value.i != MEM_SEG_CONSTANT) {
        SENI_ERROR("correct_function_addresses expected a 'LOAD CONST' 2 opcodes before a CALL");
        return;
      }
      offset->arg1.value.i = fn_info->arg_address;
      
      offset = bc - 1;
      if (offset->op != LOAD && offset->arg0.value.i != MEM_SEG_CONSTANT) {
        SENI_ERROR("correct_function_addresses expected a 'LOAD CONST' 1 opcode before a CALL");
        return;
      }
      offset->arg1.value.i = fn_info->num_args;
    }
    
    if (bc->op == CALL_0) {
      fn_info_index = bc->arg0.value.i; 
      fn_info = &(program->fn_info[fn_info_index]);

      offset = bc - 1;
      if (offset->op != LOAD && offset->arg0.value.i != MEM_SEG_CONSTANT) {
        SENI_ERROR("correct_function_addresses expected a 'LOAD CONST' 1 opcode before a CALL_0");
        return;
      }
      offset->arg1.value.i = fn_info->body_address;
    }

    if (bc->op == PLACEHOLDER_STORE) {
      bc->op = STORE;

      // opcode's arg0 is the fn_info_index and arg1 is the label_value
      fn_info_index = bc->arg0.value.i; 
      fn_info = &(program->fn_info[fn_info_index]);
      label_value = bc->arg1.value.i;
      
      i32 data_index = get_argument_mapping(fn_info, label_value);
      bc->arg1.value.i = data_index;

      if (data_index != -1) {
        bc->arg0.value.i = MEM_SEG_ARGUMENT;
      } else {
        // otherwise this function was invoked with a parameter that is doesn't use
        // so just essentially turn these ops into no-ops
        bc->arg0.value.i = MEM_SEG_VOID;
      }
    }

    bc++;
  }
}

void compile_fn_invocation(seni_node *ast, seni_program *program, i32 fn_info_index)
{
  // ast == adder a: 10 b: 20

  // NOTE: CALL and CALL_0 get their function offsets and num args from the stack
  // so add some placeholder LOAD CONST opcodes and fill the CALL, CALL_0 with
  // fn_info indexes that can later be used to fill in the LOAD CONST opcodes with their
  // correct offsets
  // doing it this way enables functions to call other functions that are declared later in the script

  // prepare the MEM_SEG_ARGUMENT with default values

  program_emit_opcode_i32(program, LOAD, MEM_SEG_CONSTANT, 666); // for the function address
  program_emit_opcode_i32(program, LOAD, MEM_SEG_CONSTANT, 667); // for the num args
  program_emit_opcode_i32(program, CALL, fn_info_index, fn_info_index);

  // overwrite the default arguments with the actual arguments given by the fn invocation
  seni_node *args = safe_next(ast); // pairs of label/value declarations
  while (args != NULL) {
    seni_node *label = args;
    seni_node *value = safe_next(label);

    // push value
    compile(value, program);
    program_emit_opcode_i32(program, PLACEHOLDER_STORE, fn_info_index, label->value.i);

    args = safe_next(value);
  }
  
  // call the body of the function
  program_emit_opcode_i32(program, LOAD, MEM_SEG_CONSTANT, 668); // for the function body address
  program_emit_opcode_i32(program, CALL_0, fn_info_index, fn_info_index);
}

// ast is a NODE_VECTOR of length 2
//
void compile_2d(seni_node *ast, seni_program *program)
{
  for (seni_node *node = ast->value.first_child; node != NULL; node = safe_next(node)) {
    compile(node, program);
  }
  program_emit_opcode_i32(program, SQUISH2, 0, 0);
}

void compile_vector(seni_node *ast, seni_program *program)
{
  // pushing from the VOID means creating a new, empty vector
  program_emit_opcode_i32(program, LOAD, MEM_SEG_VOID, 0);

  for (seni_node *node = ast->value.first_child; node != NULL; node = safe_next(node)) {
    compile(node, program);
    program_emit_opcode_i32(program, APPEND, 0, 0);
  }
}

seni_node *compile_user_defined_name(seni_node *ast, seni_program *program, i32 iname)
{
  i32 local_mapping = get_local_mapping(program, iname);
  if (local_mapping != -1) {
    program_emit_opcode_i32(program, LOAD, MEM_SEG_LOCAL, local_mapping);
    return safe_next(ast);
  }

  // check arguments if we're in a function
  if (program->current_fn_info) {
    i32 argument_mapping = get_argument_mapping(program->current_fn_info, iname);
    if (argument_mapping != -1) {
      program_emit_opcode_i32(program, LOAD, MEM_SEG_ARGUMENT, argument_mapping);
      return safe_next(ast);
    }
  }

  i32 global_mapping = get_global_mapping(program, iname);
  if (global_mapping != -1) {
    program_emit_opcode_i32(program, LOAD, MEM_SEG_GLOBAL, global_mapping);
    return safe_next(ast);
  }

  // could be a keyword such as linear, ease-in etc
  if (iname >= KEYWORD_START && iname < KEYWORD_START + MAX_KEYWORD_LOOKUPS) {
    program_emit_opcode_i32(program, LOAD, MEM_SEG_CONSTANT, iname);
    return safe_next(ast);
  }


  SENI_ERROR("compile_user_defined_name: unknown mapping for %d", iname);
  return safe_next(ast);
}

seni_node *compile(seni_node *ast, seni_program *program)
{
  seni_node *n;

  if (ast->type == NODE_LIST) {
    n = ast->value.first_child;

    i32 fn_info_index = get_fn_info_index(n, program);
    if (fn_info_index != -1) {
      compile_fn_invocation(n, program, fn_info_index);
    } else {
      compile(n, program);
    }
    
    return safe_next(ast);
  }
  if (ast->type == NODE_FLOAT) {
    program_emit_opcode_i32_f32(program, LOAD, MEM_SEG_CONSTANT, ast->value.f);
    return safe_next(ast);
  }
  if (ast->type == NODE_INT) {
    program_emit_opcode_i32(program, LOAD, MEM_SEG_CONSTANT, ast->value.i);
    return safe_next(ast);
  }
  if (ast->type == NODE_VECTOR) {
    if (node_vector_length(ast) == 2) {
      compile_2d(ast, program);
    } else {
      compile_vector(ast, program);
    }
    return safe_next(ast);
  }
  if (ast->type == NODE_NAME) {

    i32 iname = ast->value.i;
    
    if (iname >= WORD_START && iname < WORD_START + MAX_WORD_LOOKUPS) { // a user defined name
      return compile_user_defined_name(ast, program, iname);
    } else if (iname >= KEYWORD_START && iname < KEYWORD_START + MAX_KEYWORD_LOOKUPS) {

      switch(iname) {
      case INAME_DEFINE:
        return compile_define(ast, program, MEM_SEG_LOCAL);
      case INAME_IF:
        compile_if(ast, program);
        return safe_next(ast);
      case INAME_LOOP:
        compile_loop(ast, program);
        return safe_next(ast);
      case INAME_ON_MATRIX_STACK:
        compile_on_matrix_stack(ast, program);
        return safe_next(ast);
      case INAME_FN:
        compile_fn(ast, program);
        return safe_next(ast);
      case INAME_PLUS:
        compile_math(ast, program, ADD);
        return safe_next(ast);
      case INAME_MINUS:
        // TODO: differentiate between neg and sub?
        compile_math(ast, program, SUB);
        return safe_next(ast);
      case INAME_MULT:
        compile_math(ast, program, MUL);
        return safe_next(ast);
      case INAME_DIVIDE:
        compile_math(ast, program, DIV);        
        return safe_next(ast);
      case INAME_EQUAL:
        compile_math(ast, program, EQ);
        return safe_next(ast);
      case INAME_LT:
        compile_math(ast, program, LT);        
        return safe_next(ast);
      case INAME_GT:
        compile_math(ast, program, GT);
        return safe_next(ast);
      case INAME_AND:
        compile_math(ast, program, AND);
        return safe_next(ast);
      case INAME_OR:
        compile_math(ast, program, OR);
        return safe_next(ast);
      case INAME_NOT:
        compile_next_one(ast, program);
        program_emit_opcode_i32(program, NOT, 0, 0);
        return safe_next(ast);
      case INAME_SQRT:
        compile_next_one(ast, program);
        program_emit_opcode_i32(program, SQRT, 0, 0);
        return safe_next(ast);
      case INAME_ADDRESS_OF:
        compile_address_of(ast, program);
        return safe_next(ast);
      case INAME_FN_CALL:
        compile_fn_call(ast, program);
        return safe_next(ast);
      default:
        // look up the name as a user defined variable
        // normally get here when a script contains variables
        // that have the same name as common parameters.
        // e.g. r, g, b, alpha
        // or if we're passing a pre-defined argument value
        // e.g. linear in (bezier line-width-mapping: linear)
        return compile_user_defined_name(ast, program, iname);
      };
    } else if ( iname >= NATIVE_START && iname < NATIVE_START + MAX_NATIVE_LOOKUPS){
      // NATIVE

      // note: how to count the stack delta? how many pop voids are required?
      i32 num_args = 0;
      seni_node *args = safe_next(ast); // pairs of label/value declarations
      while (args != NULL) {
        seni_node *label = args;
        seni_node *value = safe_next(label);

        program_emit_opcode_i32(program, LOAD, MEM_SEG_CONSTANT, label->value.i);
        compile(value, program);

        num_args++;
        args = safe_next(value);
      }
      
      program_emit_opcode_i32(program, NATIVE, iname, num_args);

      // modify opcode_offset according to how many args were given
      program->opcode_offset -= (num_args * 2) - 1;
      
      
      return safe_next(ast);
    }
  }

  return safe_next(ast);
}

bool is_list_beginning_with(seni_node *ast, i32 index)
{
  if (ast->type != NODE_LIST) {
    return false;
  }      

  seni_node *keyword = ast->value.first_child;
  if (keyword->type == NODE_NAME && keyword->value.i == index) {
    return true;
  }

  return false;  
}


void compile_preamble_f32(seni_program *program, i32 iname, f32 value)
{
  program_emit_opcode_i32_f32(program, LOAD, MEM_SEG_CONSTANT, value);
  i32 address = get_global_mapping(program, iname);
  if (address == -1) {
    address = add_global_mapping(program, iname);
  }
  program_emit_opcode_i32(program, STORE, MEM_SEG_GLOBAL, address);
}

void compile_preamble_col(seni_program *program, i32 iname, f32 r, f32 g, f32 b, f32 a)
{
  seni_var mem_location, colour_arg;

  i32_as_var(&mem_location, MEM_SEG_CONSTANT);

  colour_arg.type = VAR_COLOUR;
  colour_arg.value.i = RGB;
  
  colour_arg.f32_array[0] = r;
  colour_arg.f32_array[1] = g;
  colour_arg.f32_array[2] = b;
  colour_arg.f32_array[3] = a;

  program_emit_opcode(program, LOAD, &mem_location, &colour_arg);

  i32 address = get_global_mapping(program, iname);
  if (address == -1) {
    address = add_global_mapping(program, iname);
  }
  program_emit_opcode_i32(program, STORE, MEM_SEG_GLOBAL, address);
}

// NOTE: each entry in compile_preamble should have a corresponding entry here
void register_top_level_preamble(seni_program *program)
{
  add_global_mapping(program, INAME_CANVAS_WIDTH);
  add_global_mapping(program, INAME_CANVAS_HEIGHT);

  add_global_mapping(program, INAME_WHITE);
  add_global_mapping(program, INAME_BLACK);

  add_global_mapping(program, INAME_RED);
  add_global_mapping(program, INAME_GREEN);
  add_global_mapping(program, INAME_BLUE);

  add_global_mapping(program, INAME_YELLOW);
  add_global_mapping(program, INAME_MAGENTA);
  add_global_mapping(program, INAME_CYAN);
}

void compile_preamble(seni_program *program)
{
// ********************************************************************************
// NOTE: each entry should have a corresponding entry in register_top_level_preamble
// ********************************************************************************
  compile_preamble_f32(program, INAME_CANVAS_WIDTH, 1000.0f);
  compile_preamble_f32(program, INAME_CANVAS_HEIGHT, 1000.0f);

  compile_preamble_col(program, INAME_WHITE, 1.0f, 1.0f, 1.0f, 1.0f);
  compile_preamble_col(program, INAME_BLACK, 0.0f, 0.0f, 0.0f, 1.0f);
  
  compile_preamble_col(program, INAME_RED, 1.0f, 0.0f, 0.0f, 1.0f);
  compile_preamble_col(program, INAME_GREEN, 0.0f, 1.0f, 0.0f, 1.0f);
  compile_preamble_col(program, INAME_BLUE, 0.0f, 0.0f, 1.0f, 1.0f);

  compile_preamble_col(program, INAME_YELLOW, 1.0f, 1.0f, 0.0f, 1.0f);
  compile_preamble_col(program, INAME_MAGENTA, 1.0f, 0.0f, 1.0f, 1.0f);
  compile_preamble_col(program, INAME_CYAN, 0.0f, 1.0f, 1.0f, 1.0f);
// ********************************************************************************
// NOTE: each entry should have a corresponding entry in register_top_level_preamble
// ********************************************************************************
}

// compiles the ast into bytecode for a stack based VM
//
void compiler_compile(seni_node *ast, seni_program *program)
{
  clear_global_mappings(program);
  clear_local_mappings(program);
  program->current_fn_info = NULL;

  // register top-level functions
  register_top_level_fns(ast, program);

  // register top-level defines
  register_top_level_preamble(program);
  register_top_level_defines(ast, program);

  seni_bytecode *start = program_emit_opcode_i32(program, JUMP, 0, 0);

  // compile the top-level functions
  seni_node *n = ast;
  while (n != NULL) {
    if (is_list_beginning_with(n, INAME_FN)) {
      n = compile(n, program);
    } else {
      n = safe_next(n);
    }
  }

  // compile the global defines common to all seni programs
  // (e.g. canvas/width)
  // this is where the program will start from
  start->arg0.type = VAR_INT;
  start->arg0.value.i = program->code_size;
  compile_preamble(program);

  // compile the top-level defines
  n = ast;
  while (n != NULL) {
    if (is_list_beginning_with(n, INAME_DEFINE)) {
      compile_define(n->value.first_child, program, MEM_SEG_GLOBAL);
      n = safe_next(n);
    } else {
      n = safe_next(n);
    }
  }

  // compile all other top-level forms
  n = ast;
  while (n != NULL) {
    if (is_list_beginning_with(n, INAME_FN) == false &&
        is_list_beginning_with(n, INAME_DEFINE) == false) {
      n = compile(n, program);
    } else {
      n = safe_next(n);
    }
  }
  
  program_emit_opcode_i32(program, STOP, 0, 0);

  // we can now update the addreses used by CALL and CALL_0
  correct_function_addresses(program);

  // SENI_LOG("program compiled: %d lines\n", program->code_size);
}

