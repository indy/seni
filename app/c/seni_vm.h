#ifndef SENI_VM_H
#define SENI_VM_H

#include "seni_lang.h"

// Stack
// 
typedef struct {
  seni_var *data;
  i32 sp;
  i32 base; // the index within 'data' that this stack starts
} seni_stack;

void stack_construct(seni_stack *stack, seni_var *data, i32 base);
seni_var *stack_push(seni_stack *stack);
seni_var *stack_pop(seni_stack *stack);
seni_var *stack_peek(seni_stack *stack);
seni_var *stack_peek2(seni_stack *stack);


// Memory Segments
//
typedef enum {
  MEM_SEG_ARGUMENT,             // store the function's arguments
  MEM_SEG_LOCAL,                // store the function's local arguments
  MEM_SEG_GLOBAL,               // global variables shared by all functions
  MEM_SEG_CONSTANT,             // pseudo-segment holds constants in range 0..32767
  MEM_SEG_THIS,                 // general purpose - point to different areas of the heap
  MEM_SEG_THAT,                 // general purpose - point to different areas of the heap
  MEM_SEG_POINTER,              // two entry segment that holds base address of this and that
  MEM_SEG_TEMP,                 // fixed 8 entry segment that holds temporary variables
  MEM_SEG_VOID,                 // nothing
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

#define MEMORY_SIZE 1024
#define STACK_SIZE 1024
// TODO: put global on the heap rather than at the bottom of the stack
#define MEMORY_GLOBAL_SIZE 10
#define MEMORY_LOCAL_SIZE 10
// ARGUMENT stores pairs of label/value, so twice as large
#define MEMORY_ARGUMENT_SIZE 32

#define MAX_TOP_LEVEL_FUNCTIONS 32

// codes
//
typedef enum {
#define OPCODE(name,_) name,
#include "seni_opcodes.h"
#undef OPCODE  
} seni_opcode;

typedef struct {
  seni_opcode op;
  seni_var arg0;
  seni_var arg1;
} seni_bytecode;

typedef struct {
  bool active;                  // is this struct being used

  i32 index;                    // the index into program->fn_info
  i32 fn_name;
  i32 arg_address;
  i32 body_address;
  i32 num_args;
  i32 argument_offsets[MEMORY_ARGUMENT_SIZE >> 1];
} seni_fn_info;



typedef struct {
  seni_bytecode *code;
  i32 code_max_size;
  i32 code_size;

  // variables used during compilation phase
  //
  i32 opcode_offset;
  i32 global_mappings[MEMORY_GLOBAL_SIZE]; // top-level defines
  i32 local_mappings[MEMORY_LOCAL_SIZE]; // store which wlut values are stored in which local memory addresses

  seni_fn_info fn_info[MAX_TOP_LEVEL_FUNCTIONS];

} seni_program;

seni_program *program_allocate(i32 code_max_size);
void program_free(seni_program *program);
void program_pretty_print(seni_program *program);

// Virtual Machine
//
typedef struct {
  seni_var *heap;
  i32 heap_size;
  
  seni_var *stack_memory;
  i32 stack_memory_size;

  // these reference the memory allocated in stack_memory
  seni_stack stack;
  seni_stack global;
  seni_stack local;
  seni_stack args;
  
} seni_virtual_machine;

seni_virtual_machine *virtual_machine_construct(i32 stack_size, i32 heap_size);
void virtual_machine_free(seni_virtual_machine *vm);

void compiler_compile(seni_node *ast, seni_program *program, word_lut *wl);

void vm_interpret(seni_virtual_machine *vm, seni_program *program);

#endif
