#include "seni_vm.h"
#include "seni_config.h"

#include <stdio.h>
#include <string.h>
#include <stdlib.h>

// **************************************************
// Stack
// **************************************************

void stack_construct(seni_stack *stack, seni_var *data, i32 base)
{
  stack->data = data;

  stack->sp = base;
  stack->base = base;
}

// returns the next available seni_var that the calling code can write to
seni_var *stack_push(seni_stack *stack)
{
  seni_var *var = &(stack->data[stack->sp]);
  stack->sp++;
  return var;
}

seni_var *stack_pop(seni_stack *stack)
{
  if (stack->sp == stack->base) {
    return NULL;
  }
  
  stack->sp--;
  return &(stack->data[stack->sp]);
}

seni_var *stack_peek(seni_stack *stack)
{
  if (stack->sp == stack->base) {
    return NULL;
  }
  return &(stack->data[stack->sp - 1]);
}

seni_var *stack_peek2(seni_stack *stack)
{
  if (stack->sp < stack->base + 2) {
    return NULL;
  }
  return &(stack->data[stack->sp - 2]);
}

void pretty_print_stack(seni_stack *stack, char *msg)
{
  printf("%s stack sp: %d\n", msg, stack->sp);
}


// **************************************************
// Program
// **************************************************

seni_program *program_allocate(i32 code_max_size)
{
  seni_program *program = (seni_program *)calloc(sizeof(seni_program), 1);

  program->code = (seni_bytecode *)calloc(sizeof(seni_bytecode), code_max_size);
  program->code_max_size = code_max_size;
  program->code_size = 0;

  return program;
}

void program_free(seni_program *program)
{
  free(program->code);
  free(program);
}

bool program_emit_opcode(seni_program *program, seni_opcode op, i32 arg0, i32 arg1)
{
  if (program->code_size >= program->code_max_size) {
    SENI_ERROR("%s %d program has reached max size", __FILE__, __LINE__);
    return false;
  }
  
  seni_bytecode *b = &(program->code[program->code_size++]);
  b->op = op;
  b->arg0 = arg0;
  b->arg1 = arg1;

  return true;
}

char *memory_segment_name(seni_memory_segment_type segment)
{
  switch(segment) {
  case MEM_SEG_ARGUMENT:
    return "ARG";
  case MEM_SEG_LOCAL:
    return "LOCAL";
  case MEM_SEG_STATIC:
    return "STATIC";
  case MEM_SEG_CONSTANT:
    return "CONST";
  case MEM_SEG_THIS:
    return "THIS";
  case MEM_SEG_THAT:
    return "THAT";
  case MEM_SEG_POINTER:
    return "PTR";
  case MEM_SEG_TEMP:
    return "TEMP";
  }
  return "UNKNOWN";
}

#define STR(x) #x
#define XSTR(x) STR(x)

char *opcode_name(seni_opcode opcode) {
  char *names[] = {
#define OPCODE(name,_) STR(name),
#include "seni_opcodes.h"
#undef OPCODE
  };

  return names[opcode];
}

void program_pretty_print(seni_program *program)
{

  for (i32 i = 0; i < program->code_size; i++) {
    seni_bytecode *b = &(program->code[i]);
    if (b->op == PUSH || b->op == POP) {
      printf("%d\t%s\t%s\t%d\n",
             i,
             opcode_name(b->op),
             memory_segment_name((seni_memory_segment_type)b->arg0),
             b->arg1);
    } else {
      printf("%d\t%s\n", i, opcode_name(b->op));
    }
  }
  printf("\n");
}

// **************************************************

// store which wlut values are stored in which local memory addresses
//
i32 local_mappings[MEMORY_LOCAL_SIZE];


// **************************************************
// Virtual Machine
// **************************************************

seni_virtual_machine *virtual_machine_construct(i32 stack_size, i32 heap_size)
{
  seni_virtual_machine *vm = (seni_virtual_machine *)calloc(sizeof(seni_virtual_machine), 1);

  vm->stack_memory = (seni_var *)calloc(sizeof(seni_var), stack_size);
  vm->stack_memory_size = stack_size;

  i32 base_offset = 0;
  stack_construct(&vm->local, vm->stack_memory, base_offset);
  base_offset += MEMORY_LOCAL_SIZE;
  stack_construct(&vm->args, vm->stack_memory, base_offset);
  stack_construct(&vm->stack, vm->stack_memory, base_offset);

  vm->heap = (seni_var *)calloc(sizeof(seni_var), heap_size);
  vm->heap_size = heap_size;

  return vm;
}

void virtual_machine_free(seni_virtual_machine *vm)
{
  free(vm->stack_memory);
  free(vm->heap);
  free(vm);
}

// **************************************************
// Compiler
// **************************************************


void clear_local_mappings()
{
  for (i32 i=0; i < MEMORY_LOCAL_SIZE; i++) {
    local_mappings[i] = -1;
  }
}

i32 get_local_mapping(i32 wlut_value)
{
  for (i32 i=0; i < MEMORY_LOCAL_SIZE; i++) {
    if(local_mappings[i] == wlut_value) {
      return i;
    }
  }

  return -1;
}

i32 add_local_mapping(i32 wlut_value)
{
  for (i32 i=0; i < MEMORY_LOCAL_SIZE; i++) {
    if(local_mappings[i] == -1) {
      local_mappings[i] = wlut_value;
      return i;
    }
  }
  return -1;
}

seni_node *compile(seni_node *ast, seni_program *program);
word_lut *g_wl = NULL;

bool string_matches(char *a, char *b)
{
  return strcmp(a, b) == 0;
}

// single pair of name/value for the moment
seni_node *compile_define(seni_node *ast, seni_program *program)
{
  // define a 42
  // ^

  seni_node *name_node = safe_next(ast);
  // TODO: assert that name_node is NODE_NAME
  
  seni_node *value_node = safe_next(name_node);
  
  compile(value_node, program);

  i32 local_address = get_local_mapping(name_node->value.i);
  if (local_address == -1) {
    local_address = add_local_mapping(name_node->value.i);
  }

  program_emit_opcode(program, POP, MEM_SEG_LOCAL, local_address);

  return safe_next(value_node);
}

// compiles everything after the current ast point
void compile_rest(seni_node *ast, seni_program *program)
{
  ast = safe_next(ast);
  while (ast) {
    ast = compile(ast, program);
  }
}

seni_node *compile(seni_node *ast, seni_program *program)
{
  seni_node *n;
  
  if (ast->type == NODE_LIST) {
    n = ast->value.first_child;
    compile(n, program);
    return safe_next(ast);
  }
  if (ast->type == NODE_INT) {
    program_emit_opcode(program, PUSH, MEM_SEG_CONSTANT, ast->value.i);
    return safe_next(ast);
  }
  if (ast->type == NODE_NAME) {
    // TODO: compare the wlut values against known ones for keywords
    char *name = wlut_lookup(g_wl, ast->value.i);

    i32 local_mapping = get_local_mapping(ast->value.i);
    
    if (local_mapping != -1) {
      program_emit_opcode(program, PUSH, MEM_SEG_LOCAL, local_mapping);
      return safe_next(ast);
    }
    else if (string_matches(name, "define")) {
      return compile_define(ast, program);
    }
    else if (string_matches(name, "+")) {

      // push the arguments onto the stack
      compile_rest(ast, program);
      
      program_emit_opcode(program, ADD, 0, 0);
      return safe_next(ast);
    }
    else if (string_matches(name, "-")) {
      // TODO: differentiate between neg and sub?
      // push the arguments onto the stack
      compile_rest(ast, program);

      program_emit_opcode(program, SUB, 0, 0);
      return safe_next(ast);
    }
    else if (string_matches(name, "=")) {
      program_emit_opcode(program, EQ, 0, 0);
      return safe_next(ast);
    }
    else if (string_matches(name, "<")) {
      program_emit_opcode(program, LT, 0, 0);
      return safe_next(ast);
    }
    else if (string_matches(name, ">")) {
      program_emit_opcode(program, GT, 0, 0);
      return safe_next(ast);
    }
    else {
      // look up the name as a local variable?
      
      return safe_next(ast);
    }
  }

  return safe_next(ast);
}

// compiles the ast into bytecode for a stack based VM
//
void compiler_compile(seni_node *ast, seni_program *program, word_lut *wl)
{
  g_wl = wl;

  // temporary invocation here
  clear_local_mappings();
  
  seni_node *n = ast;
  while (n != NULL) {
    n = compile(n, program);
  }

  program_emit_opcode(program, STOP, 0, 0);
}

// **************************************************
// VM bytecode interpreter
// **************************************************

// executes a program on a vm 
void vm_interpret(seni_virtual_machine *vm, seni_program *program)
{
  seni_bytecode *bc = NULL;
  seni_var *v = NULL;
  i32 a, b;
  i32 pc = 0;

  for (;;) {
    bc = &(program->code[pc++]);
    
    switch(bc->op) {
    case PUSH:
      v = stack_push(&(vm->stack));
      if ((seni_memory_segment_type)bc->arg0 == MEM_SEG_CONSTANT) {
        v->type = VAR_INT;
        v->value.i = bc->arg1;
      } else if ((seni_memory_segment_type)bc->arg0 == MEM_SEG_LOCAL) {
        // get value from local memory - push onto stack

        seni_var *local_var = &(vm->local.data[vm->local.base + bc->arg1]);
        // TODO: safe_var_copy also does ref-counting on vectors which is wrong for this use case
        safe_var_copy(v, local_var); 
        
      } else {
        SENI_ERROR("PUSH: unknown memory segment type %d", bc->arg0);
      }
      break;

    case POP:
      v = stack_pop(&(vm->stack));
      if ((seni_memory_segment_type)bc->arg0 == MEM_SEG_LOCAL) {
        seni_var *dest = &(vm->local.data[vm->local.base + bc->arg1]);
        safe_var_copy(dest, v);
      } else {
        SENI_ERROR("POP: unknown memory segment type %d", bc->arg0);
      } 
      break;

    case ADD:
      v = stack_pop(&(vm->stack));
      b = v->value.i;
      v = stack_pop(&(vm->stack));
      a = v->value.i;

      v = stack_push(&(vm->stack));
      v->type = VAR_INT;
      v->value.i = a + b;
      break;

    case SUB:
      v = stack_pop(&(vm->stack));
      b = v->value.i;
      v = stack_pop(&(vm->stack));
      a = v->value.i;

      v = stack_push(&(vm->stack));
      v->type = VAR_INT;
      v->value.i = a - b;
      break;
      
    case STOP:
      return;
    default:
      SENI_ERROR("Unhandled opcode: %s\n", opcode_name(bc->op));
    }
  }
}
