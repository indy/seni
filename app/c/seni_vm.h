#ifndef SENI_VM_H
#define SENI_VM_H

#include "seni_lang.h"

// Stack
// 
typedef struct {
  seni_var *data;
  i32 stack_size;
  i32 sp;
} seni_stack;

seni_stack *stack_construct(i32 size);
void stack_free(seni_stack *stack);
  
seni_var * stack_push(seni_stack *stack);
seni_var *stack_pop(seni_stack *stack);
seni_var *stack_peek(seni_stack *stack);
seni_var *stack_peek2(seni_stack *stack);


// Memory Segments
//
typedef enum {
  MEM_SEG_ARGUMENT,             // store the function's arguments
  MEM_SEG_LOCAL,                // store the function's local arguments
  MEM_SEG_STATIC,               // static variables shared by all functions
  MEM_SEG_CONSTANT,             // pseudo-segment holds constants in range 0..32767
  MEM_SEG_THIS,                 // general purpose - point to different areas of the heap
  MEM_SEG_THAT,                 // general purpose - point to different areas of the heap
  MEM_SEG_POINTER,              // two entry segment that holds base address of this and that
  MEM_SEG_TEMP                  // fixed 8 entry segment that holds temporary variables
} seni_memory_segment_type;

// known memory addresses

#define SP     0
#define LCL    1
#define ARG    2
#define THIS   3
#define THAT   4
#define TEMP0  5
#define TEMP1  6
#define TEMP2  7
#define TEMP3  8
#define TEMP4  9
#define TEMP5 10
#define TEMP6 11
#define TEMP7 12
#define R13   13
#define R14   14
#define R15   15

#define RAM_SIZE 1024


// codes
//
typedef enum {
#define OPCODE(name,_) name,
#include "seni_opcodes.h"
#undef OPCODE  
} seni_opcode;

typedef struct {
  seni_opcode op;
  i32 arg0;
  i32 arg1;
} seni_bytecode;

typedef struct {
  seni_bytecode *code;
  i32 code_max_size;
  i32 code_size;
} seni_program;

seni_program *program_allocate(i32 code_max_size);
void program_free(seni_program *program);
bool program_emit_opcode(seni_program *program, seni_opcode op, i32 arg0, i32 arg1);
void program_pretty_print(seni_program *program);

// Virtual Machine
//
typedef struct {
  seni_stack *stack;
  int *ram;
} seni_virtual_machine;

seni_virtual_machine *virtual_machine_construct(i32 stack_size);

void compiler_compile(seni_node *ast, seni_program *program, word_lut *wl);

void vm_interpret(seni_virtual_machine *vm, seni_program *program);

#endif
