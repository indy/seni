#include "seni_vm.h"
#include "seni_config.h"

#include <stdio.h>
#include <string.h>
#include <stdlib.h>

// returns the next available seni_var that the calling code can write to
seni_var *stack_push(seni_virtual_machine *vm)
{
  seni_var *var = &(vm->stack[vm->sp]);
  vm->sp++;
  return var;
}

seni_var *stack_pop(seni_virtual_machine *vm)
{
  if (vm->sp == 0) {
    return NULL;
  }
  
  vm->sp--;
  return &(vm->stack[vm->sp]);
}

seni_var *stack_peek(seni_virtual_machine *vm)
{
  if (vm->sp == 0) {
    return NULL;
  }
  return &(vm->stack[vm->sp - 1]);
}

seni_var *stack_peek2(seni_virtual_machine *vm)
{
  if (vm->sp < 2) {
    return NULL;
  }
  return &(vm->stack[vm->sp - 2]);
}

void pretty_print_vm_stack(seni_virtual_machine *vm, char *msg)
{
  printf("%s stack sp: %d\n", msg, vm->sp);
}


// **************************************************
// Program
// **************************************************

#define STR(x) #x
#define XSTR(x) STR(x)

char *opcode_name(seni_opcode opcode)
{
  char *names[] = {
#define OPCODE(name,_) STR(name),
#include "seni_opcodes.h"
#undef OPCODE
  };

  return names[opcode];
}

i32 opcode_offset[] = {
#define OPCODE(_,offset) offset,
#include "seni_opcodes.h"
#undef OPCODE
};

seni_program *program_allocate(i32 code_max_size)
{
  seni_program *program = (seni_program *)calloc(sizeof(seni_program), 1);

  program->code = (seni_bytecode *)calloc(sizeof(seni_bytecode), code_max_size);
  program->code_max_size = code_max_size;
  program->code_size = 0;
  program->opcode_offset = 0;

  return program;
}

void program_free(seni_program *program)
{
  free(program->code);
  free(program);
}

seni_bytecode *program_emit_opcode(seni_program *program, seni_opcode op, seni_var *arg0, seni_var *arg1)
{
  if (program->code_size >= program->code_max_size) {
    SENI_ERROR("%s %d program has reached max size", __FILE__, __LINE__);
    return NULL;
  }
  
  seni_bytecode *b = &(program->code[program->code_size++]);
  b->op = op;
  safe_var_move(&(b->arg0), arg0);
  safe_var_move(&(b->arg1), arg1);

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

char *memory_segment_name(seni_memory_segment_type segment)
{
  switch(segment) {
  case MEM_SEG_ARGUMENT:
    return "ARG";
  case MEM_SEG_LOCAL:
    return "LOCAL";
  case MEM_SEG_GLOBAL:
    return "GLOBAL";
  case MEM_SEG_CONSTANT:
    return "CONST";
  case MEM_SEG_VOID:
    return "VOID";
  }
  return "UNKNOWN";
}

void bytecode_pretty_print(i32 ip, seni_bytecode *b)
{
  if (b->op == PUSH || b->op == POP) {
    printf("%d\t%s\t%s\t",
           ip,
           opcode_name(b->op),
           memory_segment_name((seni_memory_segment_type)b->arg0.value.i));

    seni_value_in_use using = get_value_in_use(b->arg1.type);
    switch(using) {
    case USE_I:
      printf("%d\n", b->arg1.value.i);
      break;
    case USE_F:
      printf("%.2f\n", b->arg1.value.f);
      break;
    case USE_V:
      if (b->arg1.type == VAR_VEC_HEAD) {
        printf("[..]len %d\n", var_vector_length(&(b->arg1)));
      } else {
        printf("[..]\n");
      }
      break;
    default:
      printf("unknown type\n");
    }
    
  } else if (b->op == JUMP_IF || b->op == JUMP) {
    printf("%d\t%s\t",
           ip,
           opcode_name(b->op));
    if (b->arg0.value.i > 0) {
      printf("+%d\n", b->arg0.value.i);
    } else if (b->arg0.value.i < 0) {
      printf("%d\n", b->arg0.value.i);
    } else {
      printf("WTF!\n");
    }
  } else if (b->op == CALL_0 || b->op == CALL) {
    printf("%d\t%s\t%d\t%d\n",
           ip,
           opcode_name(b->op),
           b->arg0.value.i,
           b->arg1.value.i);
  } else {
    printf("%d\t%s\n", ip, opcode_name(b->op));
  }  
}

void program_pretty_print(seni_program *program)
{
  for (i32 i = 0; i < program->code_size; i++) {
    seni_bytecode *b = &(program->code[i]);
    bytecode_pretty_print(i, b);
  }
  printf("\n");
}

// **************************************************
// Virtual Machine
// **************************************************

void vm_stack_set_value_i32(seni_virtual_machine *vm, i32 index, i32 value)
{
  seni_var *v = &(vm->stack[index]);
  v->type = VAR_INT;
  v->value.i = value;
}

seni_virtual_machine *virtual_machine_construct(i32 stack_size, i32 heap_size)
{
  i32 base_offset = 0;
  seni_virtual_machine *vm = (seni_virtual_machine *)calloc(sizeof(seni_virtual_machine), 1);

  vm->stack = (seni_var *)calloc(sizeof(seni_var), stack_size);
  vm->stack_size = stack_size;

  vm->global = base_offset;
  base_offset += MEMORY_GLOBAL_SIZE;

  vm->ip = 0;
  
  vm->fp = base_offset;
  vm_stack_set_value_i32(vm, vm->fp, vm->fp);

  // add some offsets so that the memory after fp matches a standard format
  base_offset++;                // the caller's frame pointer
  base_offset++;                // the caller's ip
  base_offset++;                // the num_args of the called function

  vm->local = base_offset;
  //  printf("construct local: %d\n", base_offset);
  base_offset += MEMORY_LOCAL_SIZE;
  vm->sp = base_offset;
  //  printf("construct stack: %d\n", base_offset);

  vm->heap = (seni_var *)calloc(sizeof(seni_var), heap_size);
  vm->heap_size = heap_size;

  return vm;
}

void virtual_machine_free(seni_virtual_machine *vm)
{
  free(vm->stack);
  free(vm->heap);
  free(vm);
}

void pretty_print_virtual_machine(seni_virtual_machine *vm, char* msg)
{
  printf("%s\tvm: fp:%d sp:%d ip:%d local:%d\n",
         msg,
         vm->fp,
         vm->sp,
         vm->ip,
         vm->local);

  seni_var *fp = &(vm->stack[vm->fp]);
  i32 onStackFP = (fp + 0)->value.i;
  i32 onStackIP = (fp + 1)->value.i;
  i32 onStackNumArgs = (fp + 2)->value.i;
  printf("\ton stack: fp:%d ip:%d numArgs:%d\n", onStackFP, onStackIP, onStackNumArgs);
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


seni_node *compile(seni_node *ast, seni_program *program, bool global_scope);

// a define statement in the global scope
seni_node *compile_global_define(seni_node *ast, seni_program *program)
{
  // define a 42
  // ^

  seni_node *name_node = safe_next(ast);
  // TODO: assert that name_node is NODE_NAME
  
  seni_node *value_node = safe_next(name_node);
  
  compile(value_node, program, false);

  i32 global_address = get_global_mapping(program, name_node->value.i);
  if (global_address == -1) {
    global_address = add_global_mapping(program, name_node->value.i);
  }

  program_emit_opcode_i32(program, POP, MEM_SEG_GLOBAL, global_address);

  return safe_next(value_node);
}

// single pair of name/value for the moment
seni_node *compile_define(seni_node *ast, seni_program *program)
{
  // define a 42
  // ^

  seni_node *name_node = safe_next(ast);
  // TODO: assert that name_node is NODE_NAME
  
  seni_node *value_node = safe_next(name_node);
  
  compile(value_node, program, false);

  i32 local_address = get_local_mapping(program, name_node->value.i);
  if (local_address == -1) {
    local_address = add_local_mapping(program, name_node->value.i);
  }

  program_emit_opcode_i32(program, POP, MEM_SEG_LOCAL, local_address);

  return safe_next(value_node);
}


seni_node *compile_if(seni_node *ast, seni_program *program)
{
  // if (> 200 100) 12 24
  // ^
  seni_node *if_node = safe_next(ast);
  seni_node *then_node = safe_next(if_node);
  seni_node *else_node = safe_next(then_node); // could be NULL

  compile(if_node, program, false);
  // insert jump to after the 'then' node if not true
  i32 addr_jump_then = program->code_size;
  seni_bytecode *bc_jump_then = program_emit_opcode_i32(program, JUMP_IF, 0, 0);

  compile(then_node, program, false);

  if (else_node) {
    // insert a bc_jump_else opcode
    i32 addr_jump_else = program->code_size;
    seni_bytecode *bc_jump_else = program_emit_opcode_i32(program, JUMP, 0, 0);

    bc_jump_then->arg0.value.i = program->code_size - addr_jump_then;

    compile(else_node, program, false);

    bc_jump_else->arg0.value.i = program->code_size - addr_jump_else;
  } else {
    bc_jump_then->arg0.value.i = program->code_size - addr_jump_then;
  }

  return NULL;
}

seni_node *compile_loop(seni_node *ast, seni_program *program)
{
  // (loop (x from: 0 to: 5) (+ 42 38))
  //
  // 0       PUSH    CONST   0
  // 1       POP     LOCAL   0
  // 2       PUSH    LOCAL   0
  // 3       PUSH    CONST   5
  // 4       LT
  // 5       JUMP_IF +10
  // 6       PUSH    CONST   42
  // 7       PUSH    CONST   38
  // 8       ADD
  // 9       POP     VOID    0
  // 10      PUSH    LOCAL   0
  // 11      PUSH    CONST   1
  // 12      ADD
  // 13      POP     LOCAL   0
  // 14      JUMP    -12
  // 15      STOP
  
  seni_node *parameters_node = safe_next(ast);
  if (parameters_node->type != NODE_LIST) {
    SENI_ERROR("expected a list that defines loop parameters");
    return NULL;
  }

  // the looping variable x
  seni_node *name_node = parameters_node->value.first_child;
  // from: 0
  seni_node *from_node = safe_next(name_node); // the label 'from'
  from_node = safe_next(from_node);            // the value of 'from'
  // to: 5
  seni_node *to_node = safe_next(from_node); // the label 'to'
  to_node = safe_next(to_node);              // the value of 'to'

  // set looping variable x to 'from' value
  compile(from_node, program, false);
  i32 looper_address = get_local_mapping(program, name_node->value.i);
  if (looper_address == -1) {
    looper_address = add_local_mapping(program, name_node->value.i);
  }
  program_emit_opcode_i32(program, POP, MEM_SEG_LOCAL, looper_address);

  // compare looping variable against exit condition
  // and jump if looping variable >= exit value
  i32 addr_loop_start = program->code_size;
  program_emit_opcode_i32(program, PUSH, MEM_SEG_LOCAL, looper_address);
  compile(to_node, program, false);
  program_emit_opcode_i32(program, LT, 0, 0);
  i32 addr_exit_check = program->code_size;
  seni_bytecode *bc_exit_check = program_emit_opcode_i32(program, JUMP_IF, 0, 0);


  i32 pre_body_opcode_offset = program->opcode_offset;

  // compile the body forms (woooaaaoohhh body form, body form for yoooouuuu)
  seni_node *body = safe_next(parameters_node);
  while (body != NULL) {
    compile(body, program, false);
    body = safe_next(body);
  }

  i32 post_body_opcode_offset = program->opcode_offset;
  i32 opcode_delta = post_body_opcode_offset - pre_body_opcode_offset;

  // pop off any values that the body might leave on the stack
  for(i32 i = 0;i < opcode_delta; i++) {
    program_emit_opcode_i32(program, POP, MEM_SEG_VOID, 0);
  }

  // increment the looping variable
  program_emit_opcode_i32(program, PUSH, MEM_SEG_LOCAL, looper_address);
  program_emit_opcode_i32_f32(program, PUSH, MEM_SEG_CONSTANT, 1.0f);
  program_emit_opcode_i32(program, ADD, 0, 0);
  program_emit_opcode_i32(program, POP, MEM_SEG_LOCAL, looper_address);

  // loop back to the comparison
  program_emit_opcode_i32(program, JUMP, -(program->code_size - addr_loop_start), 0);
  bc_exit_check->arg0.value.i = program->code_size - addr_exit_check;

  return NULL;
}

seni_fn_info *get_local_fn_info(seni_node *node, seni_program *program)
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


i32 index_of_keyword(const char *keyword, word_lut *wl)
{
  for (i32 i = 0; i < wl->keyword_count; i++) {
    if (strcmp(keyword, wl->keyword[i]) == 0) {
      return KEYWORD_START + i; // the keywords have KEYWORD_START added onto their index
    }
  }

  return -1;
}

bool is_seni_node_a_function(seni_node *ast, i32 fn_index)
{
  if (ast->type != NODE_LIST) {
    return false;
  }      

  seni_node *fn_keyword = ast->value.first_child;
  if (fn_keyword->type == NODE_NAME && fn_keyword->value.i == fn_index) {
    return true;
  }

  return false;
}

void register_top_level_fns(seni_node *ast, seni_program *program)
{
  i32 i;
  i32 num_fns = 0;
  
  // clear all fn data
  for (i = 0; i < MAX_TOP_LEVEL_FUNCTIONS; i++) {
    program->fn_info[i].active = false;
  }
  
  // search the wlut for the index of the 'fn' keyword
  i32 fn_index = index_of_keyword("fn", program->wl);

  // register top level fns
  while (ast != NULL) {

    if (ast->type != NODE_LIST) {
      ast = safe_next(ast);
      continue;
    }      

    seni_node *fn_keyword = ast->value.first_child;
    if (!(fn_keyword->type == NODE_NAME && fn_keyword->value.i == fn_index)) {
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

/*
  invoking code will first CALL into the arg_address to setup the default values for all args
  the fn code will then return back to the invoking code
  invoking code will then overwrite specific data in arg memory
  invoking code will then CALL into the body_address
*/
seni_node *compile_fn(seni_node *ast, seni_program *program)
{
  // fn (adder a: 0 b: 0) (+ a b)

  clear_local_mappings(program);

  // (adder a: 0 b: 0)
  seni_node *signature = safe_next(ast);

  seni_node *fn_name = signature->value.first_child;
  seni_fn_info *fn_info = get_local_fn_info(fn_name, program);
  if (fn_info == NULL) {
    SENI_ERROR("Unable to find fn_info for function %d", fn_name->value.i);
    return NULL;
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
    program_emit_opcode_i32(program, PUSH, MEM_SEG_CONSTANT, label->value.i);
    program_emit_opcode_i32(program, POP, MEM_SEG_ARGUMENT, counter++);

    // HACK:
    // think of a more robust way of pushing a seni_node value into a bytecode instruction
    //
    program_emit_opcode_i32_f32(program, PUSH, MEM_SEG_CONSTANT, value->value.f);
    program_emit_opcode_i32(program, POP, MEM_SEG_ARGUMENT, counter++);

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
  seni_node *body = safe_next(signature);

  while (body != NULL) {
    compile(body, program, false);
    body = safe_next(body);
  }

  // Don't need any POP, MEM_SEG_VOID instructions as the RET will
  // pop the frame and blow the stack

  program_emit_opcode_i32(program, RET, 0, 0);

  program->current_fn_info = NULL;

  return NULL;
}

// compiles everything after the current ast point
void compile_rest(seni_node *ast, seni_program *program)
{
  ast = safe_next(ast);
  while (ast) {
    ast = compile(ast, program, false);
  }
}

void compile_fn_invocation(seni_node *ast, seni_program *program, seni_fn_info *fn_info, bool global_scope)
{
  // ast == adder a: 10 b: 20

  // prepare the MEM_SEG_ARGUMENT with default values
  program_emit_opcode_i32(program, CALL, fn_info->arg_address, fn_info->num_args);

  // overwrite the default arguments with the actual arguments given by the fn invocation
  seni_node *args = safe_next(ast); // pairs of label/value declarations
  while (args != NULL) {
    seni_node *label = args;
    seni_node *value = safe_next(label);

    // find the index within MEM_SEG_ARGUMENT that holds the default value for label
    i32 data_index = get_argument_mapping(fn_info, label->value.i);
    if (data_index != -1) {
      // push value
      compile(value, program, global_scope);
      program_emit_opcode_i32(program, POP, MEM_SEG_ARGUMENT, data_index);
    }

    args = safe_next(value);
  }
  
  // call the body of the function
  program_emit_opcode_i32(program, CALL_0, fn_info->body_address, fn_info->num_args);

}

seni_node *compile(seni_node *ast, seni_program *program, bool global_scope)
{
  seni_node *n;

  if (ast->type == NODE_LIST) {
    n = ast->value.first_child;

    seni_fn_info *fn_info = get_local_fn_info(n, program);
    if (fn_info) {
      compile_fn_invocation(n, program, fn_info, global_scope);
    } else {
      compile(n, program, global_scope);
    }
    return safe_next(ast);
  }
  if (ast->type == NODE_FLOAT) {
    program_emit_opcode_i32_f32(program, PUSH, MEM_SEG_CONSTANT, ast->value.f);
    return safe_next(ast);
  }
  if (ast->type == NODE_INT) {
    program_emit_opcode_i32(program, PUSH, MEM_SEG_CONSTANT, ast->value.i);
    return safe_next(ast);
  }
  if (ast->type == NODE_NAME) {

    i32 iname = ast->value.i;
    
    if (iname >= WORD_START && iname < WORD_START + MAX_WORD_LOOKUPS) { // a user defined name
      
      i32 local_mapping = get_local_mapping(program, iname);
      if (local_mapping != -1) {
        program_emit_opcode_i32(program, PUSH, MEM_SEG_LOCAL, local_mapping);
        return safe_next(ast);
      }

      // check arguments if we're in a function
      if (program->current_fn_info) {
        i32 argument_mapping = get_argument_mapping(program->current_fn_info, iname);
        if (argument_mapping != -1) {
          program_emit_opcode_i32(program, PUSH, MEM_SEG_ARGUMENT, argument_mapping);
          return safe_next(ast);
        }
      }

      i32 global_mapping = get_global_mapping(program, iname);
      if (global_mapping != -1) {
        program_emit_opcode_i32(program, PUSH, MEM_SEG_GLOBAL, global_mapping);
        return safe_next(ast);
      }
    } else if (iname >= KEYWORD_START && iname < KEYWORD_START + MAX_KEYWORD_LOOKUPS) {

      if (iname == program->wl->iname_define) {
        if (global_scope) {
          return compile_global_define(ast, program);
        } else {
          return compile_define(ast, program);
        }        
      } else if (iname == program->wl->iname_if) {
        return compile_if(ast, program);          
      } else if (iname == program->wl->iname_loop) {
        return compile_loop(ast, program);
      } else if (iname == program->wl->iname_fn) {
        return compile_fn(ast, program);          
      } else if (iname == program->wl->iname_plus) {
        compile_rest(ast, program);
        program_emit_opcode_i32(program, ADD, 0, 0);
        return safe_next(ast);
      } else if (iname == program->wl->iname_minus) {
        // TODO: differentiate between neg and sub?
        compile_rest(ast, program);
        program_emit_opcode_i32(program, SUB, 0, 0);
        return safe_next(ast);
      } else if (iname == program->wl->iname_equal) {
        compile_rest(ast, program);
        program_emit_opcode_i32(program, EQ, 0, 0);
        return safe_next(ast);
      } else if (iname == program->wl->iname_lt) {
        compile_rest(ast, program);
        program_emit_opcode_i32(program, LT, 0, 0);
        return safe_next(ast);
      } else if (iname == program->wl->iname_gt) {
        compile_rest(ast, program);
        program_emit_opcode_i32(program, GT, 0, 0);
        return safe_next(ast);
      } else if (iname == program->wl->iname_and) {
        compile_rest(ast, program);
        program_emit_opcode_i32(program, AND, 0, 0);
        return safe_next(ast);
      } else if (iname == program->wl->iname_or) {
        compile_rest(ast, program);
        program_emit_opcode_i32(program, OR, 0, 0);
        return safe_next(ast);
      } else if (iname == program->wl->iname_not) {
        compile_rest(ast, program);
        program_emit_opcode_i32(program, NOT, 0, 0);
        return safe_next(ast);
      } else {
        // look up the name as a local variable?
        return safe_next(ast);        
      }

    } else if ( iname >= NATIVE_START && iname < NATIVE_START + MAX_NATIVE_LOOKUPS){
      // NATIVE
    }
  }

  return safe_next(ast);
}

// compiles the ast into bytecode for a stack based VM
//
void compiler_compile(seni_node *ast, seni_program *program)
{
  clear_global_mappings(program);
  clear_local_mappings(program);
  program->current_fn_info = NULL;
  
  register_top_level_fns(ast, program);

  i32 fn_index = index_of_keyword("fn", program->wl);
  seni_bytecode *start = program_emit_opcode_i32(program, JUMP, 0, 0);
  bool found_start = false;
  
  seni_node *n = ast;
  while (n != NULL) {
    // ghetto jump to start
    if (found_start == false && is_seni_node_a_function(n, fn_index) == false) {
      start->arg0.type = VAR_INT;
      start->arg0.value.i = program->code_size;
      found_start = true;
    }
    n = compile(n, program, true);
  }

  program_emit_opcode_i32(program, STOP, 0, 0);
}

// **************************************************
// VM bytecode interpreter
// **************************************************

// executes a program on a vm 
void vm_interpret(seni_virtual_machine *vm, seni_program *program)
{
  bool b1, b2;
  f32 f1, f2;
  seni_memory_segment_type memory_segment_type;
  seni_var *src, *dest;

  register seni_bytecode *bc = NULL;
  register seni_var *v = NULL;
  register i32 ip = vm->ip;
  register i32 sp = vm->sp;
  register seni_var *stack_d = &(vm->stack[sp]);

  i32 new_fp;
  i32 num_args;

#define STACK_POP stack_d--; sp--; v = stack_d
#define STACK_PUSH v = stack_d; stack_d++; sp++

  for (;;) {
    // printf("%d\n", ip);
    bc = &(program->code[ip++]);
    
    switch(bc->op) {
    case PUSH:
      STACK_PUSH;

      memory_segment_type = (seni_memory_segment_type)bc->arg0.value.i;
      if (memory_segment_type == MEM_SEG_CONSTANT) {
        safe_var_move(v, &(bc->arg1));
      } else if (memory_segment_type == MEM_SEG_ARGUMENT) {
        src = &(vm->stack[vm->fp - bc->arg1.value.i - 1]);
        safe_var_move(v, src);
        // printf("pushing ARG value %d onto the stack from %d\n", v->value.i, a);
      } else if (memory_segment_type == MEM_SEG_LOCAL) {
        src = &(vm->stack[vm->local + bc->arg1.value.i]);
        safe_var_move(v, src);
      }else if (memory_segment_type == MEM_SEG_GLOBAL) {
        src = &(vm->stack[vm->global + bc->arg1.value.i]);
        safe_var_move(v, src);
      } else {
        SENI_ERROR("PUSH: unknown memory segment type %d", bc->arg0.value.i);
      }
      break;

    case POP:
      STACK_POP;

      memory_segment_type = (seni_memory_segment_type)bc->arg0.value.i;
      if (memory_segment_type == MEM_SEG_ARGUMENT) {
        dest = &(vm->stack[vm->fp - bc->arg1.value.i - 1]);
        safe_var_move(dest, v);
        // printf("popping ARG value %d from the stack onto %d\n", v->value.i, a);
      } else if (memory_segment_type == MEM_SEG_LOCAL) {
        dest = &(vm->stack[vm->local + bc->arg1.value.i]);
        safe_var_move(dest, v);
      } else if (memory_segment_type == MEM_SEG_GLOBAL) {
        dest = &(vm->stack[vm->global + bc->arg1.value.i]);
        safe_var_move(dest, v);
      } else if (memory_segment_type == MEM_SEG_VOID) {
        // do nothing - just pop from the stack and lose the value
      } else {
        SENI_ERROR("POP: unknown memory segment type %d", bc->arg0.value.i);
      } 
      break;

    case JUMP:
      ip--;
      ip += bc->arg0.value.i;
      break;

    case JUMP_IF:
      STACK_POP;

      // jump if the top of the stack is false
      if (v->value.i == 0) {
        ip--;
        ip += bc->arg0.value.i;
      }
      break;

    case CALL:
      num_args = bc->arg1.value.i;

      // make room for the labelled arguments
      for (i32 i = 0; i < num_args * 2; i++) {
        STACK_PUSH;
      }
      
      new_fp = sp;

      // push the caller's fp
      STACK_PUSH;
      v->type = VAR_INT;
      v->value.i = vm->fp;

      // push ip
      STACK_PUSH;
      v->type = VAR_INT;
      v->value.i = ip;

      // push num_args
      STACK_PUSH;
      v->type = VAR_INT;
      v->value.i = num_args;

      vm->ip = bc->arg0.value.i;
      vm->fp = new_fp;
      vm->local = sp;
      sp += MEMORY_LOCAL_SIZE;

      stack_d = &(vm->stack[sp]);
      ip = vm->ip;
      
      vm->sp = sp;

      break;

    case CALL_0:
      // like CALL but keep the existing frame and just update the ip and return ip
      
      // set the correct return ip
      vm->stack[vm->fp + 1].value.i = ip;

      // leap to a location
      ip = bc->arg0.value.i;
      vm->ip = ip;
      break;

    case RET_0:
      // leap to the return ip
      vm->ip = vm->stack[vm->fp + 1].value.i;
      ip = vm->ip;
      break;
      
    case RET:
      // pop the frame
      //

      // grab whatever was the last value on the soon to be popped frame
      src = &(vm->stack[sp - 1]);

      num_args = vm->stack[vm->fp + 2].value.i;

      vm->sp = vm->fp - (num_args * 2);
      vm->ip = vm->stack[vm->fp + 1].value.i;
      vm->fp = vm->stack[vm->fp].value.i;
      vm->local = vm->fp + 3;

      // sync up the fast registers
      ip = vm->ip;
      sp = vm->sp;
      stack_d = &(vm->stack[sp]);

      // copy the previous frame's top stack value onto the current frame's stack
      STACK_PUSH;
      safe_var_move(v, src);

      break;

    case ADD:
      STACK_POP;
      f2 = v->value.f;
      
      STACK_POP;
      f1 = v->value.f;

      STACK_PUSH;
      v->value.f = f1 + f2;
      break;

    case SUB:
      STACK_POP;
      f2 = v->value.f;
      STACK_POP;
      f1 = v->value.f;

      STACK_PUSH;
      v->value.f = f1 - f2;

      break;

    case EQ:
      STACK_POP;
      f2 = v->value.f;
      STACK_POP;
      f1 = v->value.f;

      STACK_PUSH;
      v->type = VAR_BOOLEAN;
      v->value.i = f1 == f2;
      break;

    case GT:
      STACK_POP;
      f2 = v->value.f;
      STACK_POP;
      f1 = v->value.f;

      STACK_PUSH;
      v->type = VAR_BOOLEAN;
      v->value.i = f1 > f2;
      break;

    case LT:
      STACK_POP;
      f2 = v->value.f;
      STACK_POP;
      f1 = v->value.f;

      STACK_PUSH;
      v->value.i = f1 < f2;
      v->type = VAR_BOOLEAN;
      break;

    case AND:
      STACK_POP;
      b2 = (bool)v->value.i;
      STACK_POP;
      b1 = (bool)v->value.i;

      STACK_PUSH;
      v->value.i = b1 && b2;
      v->type = VAR_BOOLEAN;
      break;
      
    case OR:
      STACK_POP;
      b2 = (bool)v->value.i;
      STACK_POP;
      b1 = (bool)v->value.i;

      STACK_PUSH;
      v->value.i = b1 || b2;
      v->type = VAR_BOOLEAN;
      break;
      
    case NOT:
      STACK_POP;
      b1 = (bool)v->value.i;

      STACK_PUSH;
      v->value.i = !b1;
      v->type = VAR_BOOLEAN;
      break;
      
    case STOP:
      vm->sp = sp;
      return;
    default:
      SENI_ERROR("Unhandled opcode: %s\n", opcode_name(bc->op));
    }
  }
}
