#include "seni_vm.h"
#include "seni_config.h"

#include <stdio.h>
#include <stdlib.h>

// **************************************************
// Stack
// **************************************************

seni_stack *stack_construct(i32 size)
{
  seni_stack *stack = (seni_stack *)calloc(sizeof(seni_stack), 1);
  stack->data = (seni_var *)(calloc(sizeof(seni_var), size));

  stack->stack_size = size;
  stack->sp = 0;

  return stack;
}

void stack_free(seni_stack *stack)
{
  free(stack->data);
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
  if (stack->sp == 0) {
    return NULL;
  }
  
  stack->sp--;
  return &(stack->data[stack->sp]);
}

seni_var *stack_peek(seni_stack *stack)
{
  if (stack->sp == 0) {
    return NULL;
  }
  return &(stack->data[stack->sp - 1]);
}

seni_var *stack_peek2(seni_stack *stack)
{
  if (stack->sp < 2) {
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

#define STR(x) #x
#define XSTR(x) STR(x)

void program_pretty_print(seni_program *program)
{
  char *names[] = {
#define OPCODE(name,_) STR(name),
#include "seni_opcodes.h"
#undef OPCODE
  };

  for (i32 i = 0; i < program->code_size; i++) {
    seni_bytecode *b = &(program->code[i]);
    if (b->op == PUSH) {
      printf("%d\t%s\t%d\n", i, names[b->op], b->arg0);
    } else {
      printf("%d\t%s\n", i, names[b->op]);
    }
  }
  printf("\n");
}

// **************************************************
// Virtual Machine
// **************************************************

seni_virtual_machine *virtual_machine_construct(i32 stack_size)
{
  seni_virtual_machine *vm = (seni_virtual_machine *)calloc(sizeof(seni_virtual_machine), 1);

  vm->stack = stack_construct(stack_size);
  vm->ram = (int *)calloc(sizeof(int), RAM_SIZE);
  
  return vm;
}

// **************************************************
// Compiler
// **************************************************

void compile(seni_node *ast, seni_program *program);
word_lut *g_wl = NULL;

void compile_list(seni_node *ast, seni_program *program)
{
  // first element is the op, the rest are arguments
  //

  seni_node *op = ast->value.first_child;

  seni_node *arg = safe_next(op);
  while (arg) {
    compile(arg, program);
    arg = safe_next(arg);
  }

  compile(op, program);
}

void compile(seni_node *ast, seni_program *program)
{
  if (ast->type == NODE_LIST) {
    compile_list(ast, program);
  }
  if (ast->type == NODE_INT) {
    program_emit_opcode(program, PUSH, ast->value.i, 0);
  }
  if (ast->type == NODE_NAME) {
    char *name = wlut_lookup(g_wl, ast->value.i);
    if (name[0] == '+') {
      program_emit_opcode(program, ADD, 0, 0);
    }
    if (name[0] == '-') {
      // TODO: differentiate between neg and sub?
      program_emit_opcode(program, SUB, 0, 0);
    }
    if (name[0] == '=') {
      program_emit_opcode(program, EQ, 0, 0);
    }
    if (name[0] == '<') {
      program_emit_opcode(program, LT, 0, 0);
    }
    if (name[0] == '>') {
      program_emit_opcode(program, GT, 0, 0);
    }
    
  }
}

// compiles the ast into bytecode for a stack based VM
//
void compiler_compile(seni_node *ast, seni_program *program, word_lut *wl)
{
  g_wl = wl;
  for (seni_node *n = ast; n != NULL; n = safe_next(n)) {
    compile(n, program);
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
      v  = stack_push(vm->stack);
      v->type = VAR_INT;
      v->value.i = bc->arg0;
      break;

    case ADD:
      v = stack_pop(vm->stack);
      b = v->value.i;
      v = stack_pop(vm->stack);
      a = v->value.i;

      v = stack_push(vm->stack);
      v->type = VAR_INT;
      v->value.i = a + b;
      break;

    case SUB:
      v = stack_pop(vm->stack);
      b = v->value.i;
      v = stack_pop(vm->stack);
      a = v->value.i;

      v = stack_push(vm->stack);
      v->type = VAR_INT;
      v->value.i = a - b;
      break;
      
    case STOP:
      return;
    }
  }
}
