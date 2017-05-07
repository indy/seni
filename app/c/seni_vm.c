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

seni_bytecode *program_emit_opcode(seni_program *program, seni_opcode op, i32 arg0, i32 arg1)
{
  if (program->code_size >= program->code_max_size) {
    SENI_ERROR("%s %d program has reached max size", __FILE__, __LINE__);
    return NULL;
  }
  
  seni_bytecode *b = &(program->code[program->code_size++]);
  b->op = op;
  b->arg0 = arg0;
  b->arg1 = arg1;

  return b;
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
    } else if (b->op == JUMP_IF || b->op == JUMP) {
      printf("%d\t%s\t",
             i,
             opcode_name(b->op));
      if (b->arg0 > 0) {
        printf("+%d\n", b->arg0);
      } else if (b->arg0 < 0) {
        printf("%d\n", b->arg0);
      } else {
        printf("WTF!\n");
      }
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


seni_node *compile_if(seni_node *ast, seni_program *program)
{
  // if (> 200 100) 12 24
  // ^
  seni_node *if_node = safe_next(ast);
  seni_node *then_node = safe_next(if_node);
  seni_node *else_node = safe_next(then_node); // could be NULL

  compile(if_node, program);
  // insert jump to after the 'then' node if not true
  i32 addr_jump_then = program->code_size;
  seni_bytecode *bc_jump_then = program_emit_opcode(program, JUMP_IF, 0, 0);

  compile(then_node, program);

  if (else_node) {
    // insert a bc_jump_else opcode
    i32 addr_jump_else = program->code_size;
    seni_bytecode *bc_jump_else = program_emit_opcode(program, JUMP, 0, 0);

    bc_jump_then->arg0 = program->code_size - addr_jump_then;

    compile(else_node, program);

    bc_jump_else->arg0 = program->code_size - addr_jump_else;
  } else {
    bc_jump_then->arg0 = program->code_size - addr_jump_then;
  }

  return NULL;
}

seni_node *compile_loop(seni_node *ast, seni_program *program)
{
  // loop (x from: 0 to: 5) (+ 42 38)
  // ^

  // 0  PUSH    CONST 0
  // 1  POP     LOCAL 0   ;; set x to 0
  // 2  PUSH    LOCAL 0
  // 3  PUSH    CONST 5
  // 4  LT
  // 5  JUMP_IF +9        ;; jump out of loop if x >= 5
  // 6  PUSH    CONST 42
  // 7  PUSH    CONST 38
  // 8  ADD               ;; code in body form
  // 9  PUSH    LOCAL 0
  // 10	PUSH    CONST 1
  // 11	ADD               ;; increment x
  // 12	POP	    LOCAL 0
  // 13	JUMP    -11       ;; jump to loop exit check
  // 14	STOP
  
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
  compile(from_node, program);
  i32 looper_address = get_local_mapping(name_node->value.i);
  if (looper_address == -1) {
    looper_address = add_local_mapping(name_node->value.i);
  }
  program_emit_opcode(program, POP, MEM_SEG_LOCAL, looper_address);

  // compare looping variable against exit condition
  // and jump if looping variable >= exit value
  i32 addr_loop_start = program->code_size;
  program_emit_opcode(program, PUSH, MEM_SEG_LOCAL, looper_address);
  compile(to_node, program);
  program_emit_opcode(program, LT, 0, 0);
  i32 addr_exit_check = program->code_size;
  seni_bytecode *bc_exit_check = program_emit_opcode(program, JUMP_IF, 0, 0);

  // compile the body forms (woooaaaoohhh body form, body form for yoooouuuu)
  seni_node *body = safe_next(parameters_node);
  while (body != NULL) {
    compile(body, program);
    body = safe_next(body);
  }

  // increment the looping variable
  program_emit_opcode(program, PUSH, MEM_SEG_LOCAL, looper_address);
  program_emit_opcode(program, PUSH, MEM_SEG_CONSTANT, 1);
  program_emit_opcode(program, ADD, 0, 0);
  program_emit_opcode(program, POP, MEM_SEG_LOCAL, looper_address);

  // loop back to the comparison
  program_emit_opcode(program, JUMP, -(program->code_size - addr_loop_start), 0);
  bc_exit_check->arg0 = program->code_size - addr_exit_check;

  return NULL;
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
    } else if (string_matches(name, "define")) {
      return compile_define(ast, program);
    } else if (string_matches(name, "if")) {
      return compile_if(ast, program);
    } else if (string_matches(name, "loop")) {
      return compile_loop(ast, program);
    } else if (string_matches(name, "+")) {
      compile_rest(ast, program);
      program_emit_opcode(program, ADD, 0, 0);
      return safe_next(ast);
    } else if (string_matches(name, "-")) {
      // TODO: differentiate between neg and sub?
      compile_rest(ast, program);
      program_emit_opcode(program, SUB, 0, 0);
      return safe_next(ast);
    } else if (string_matches(name, "=")) {
      compile_rest(ast, program);
      program_emit_opcode(program, EQ, 0, 0);
      return safe_next(ast);
    } else if (string_matches(name, "<")) {
      compile_rest(ast, program);
      program_emit_opcode(program, LT, 0, 0);
      return safe_next(ast);
    } else if (string_matches(name, ">")) {
      compile_rest(ast, program);
      program_emit_opcode(program, GT, 0, 0);
      return safe_next(ast);
    } else if (string_matches(name, "and")) {
      compile_rest(ast, program);
      program_emit_opcode(program, AND, 0, 0);
      return safe_next(ast);
    } else if (string_matches(name, "or")) {
      compile_rest(ast, program);
      program_emit_opcode(program, OR, 0, 0);
      return safe_next(ast);
    } else if (string_matches(name, "not")) {
      compile_rest(ast, program);
      program_emit_opcode(program, NOT, 0, 0);
      return safe_next(ast);
    } else {
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
  i32 a, b;
  bool b1, b2;

  register seni_bytecode *bc = NULL;
  register seni_var *v = NULL;
  register i32 ip = 0;
  register i32 sp = vm->stack.sp;
  register seni_var *stack_d = &(vm->stack.data[sp]);

#define STACK_POP stack_d--; sp--; v = stack_d
#define STACK_PUSH v = stack_d; stack_d++; sp++

  for (;;) {
    bc = &(program->code[ip++]);
    
    switch(bc->op) {
    case PUSH:
      STACK_PUSH;
      if ((seni_memory_segment_type)bc->arg0 == MEM_SEG_CONSTANT) {
        v->type = VAR_INT;
        v->value.i = bc->arg1;
      } else if ((seni_memory_segment_type)bc->arg0 == MEM_SEG_LOCAL) {
        // get value from local memory - push onto stack

        seni_var *local_var = &(vm->local.data[vm->local.base + bc->arg1]);
        safe_var_move(v, local_var);
      } else {
        SENI_ERROR("PUSH: unknown memory segment type %d", bc->arg0);
      }
      break;

    case POP:
      STACK_POP;
      if ((seni_memory_segment_type)bc->arg0 == MEM_SEG_LOCAL) {
        seni_var *dest = &(vm->local.data[vm->local.base + bc->arg1]);
        safe_var_move(dest, v);
      } else {
        SENI_ERROR("POP: unknown memory segment type %d", bc->arg0);
      } 
      break;

    case JUMP:
      ip--;
      ip += bc->arg0;
      break;

    case JUMP_IF:
      STACK_POP;

      // jump if the top of the stack is false
      if (v->value.i == 0) {
        ip--;
        ip += bc->arg0;
      }
      break;

    case ADD:
      STACK_POP;
      b = v->value.i;
      STACK_POP;
      a = v->value.i;

      STACK_PUSH;
      v->value.i = a + b;
      break;

    case SUB:
      STACK_POP;
      b = v->value.i;
      STACK_POP;
      a = v->value.i;

      STACK_PUSH;
      v->value.i = a - b;

      // TEMP
      stack_d--;
      sp--;
      break;

    case EQ:
      STACK_POP;
      b = v->value.i;
      STACK_POP;
      a = v->value.i;

      STACK_PUSH;
      v->type = VAR_BOOLEAN;
      v->value.i = a == b;
      break;

    case GT:
      STACK_POP;
      b = v->value.i;
      STACK_POP;
      a = v->value.i;

      STACK_PUSH;
      v->type = VAR_BOOLEAN;
      v->value.i = a > b;
      break;

    case LT:
      STACK_POP;
      b = v->value.i;
      STACK_POP;
      a = v->value.i;

      STACK_PUSH;
      v->value.i = a < b;
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
      vm->stack.sp = sp;
      return;
    default:
      SENI_ERROR("Unhandled opcode: %s\n", opcode_name(bc->op));
    }
  }
}
