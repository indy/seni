#ifndef SENI_VM_H
#define SENI_VM_H

#include "seni_lang.h"

// Memory Segments
//
typedef enum {
  MEM_SEG_ARGUMENT,             // store the function's arguments
  MEM_SEG_LOCAL,                // store the function's local arguments
  MEM_SEG_GLOBAL,               // global variables shared by all functions
  MEM_SEG_CONSTANT,             // pseudo-segment holds constants in range 0..32767
  MEM_SEG_VOID,                 // nothing
} seni_memory_segment_type;

// known memory addresses

#define SP     0
#define LCL    1
#define ARG    2

#define MEMORY_SIZE 1024
#define STACK_SIZE 1024
// TODO: put global on the heap rather than at the bottom of the stack
#define MEMORY_GLOBAL_SIZE 10
#define MEMORY_LOCAL_SIZE 10

#define MAX_TOP_LEVEL_FUNCTIONS 32

#define MAX_NUM_ARGUMENTS 16



// Virtual Machine
//

typedef struct seni_vm_debug_info {
  i32 get_from_heap_count;
  i32 return_to_heap_count;
} seni_vm_debug_info;

typedef struct seni_virtual_machine {
  seni_buffer *buffer;          // used for rendering vertices

  seni_var *heap;               // the contiguous block of allocated memory
  i32 heap_size;
  seni_var *heap_list;          // doubly linked list of unallocated seni_vars from the heap
  
  seni_var *stack;
  i32 stack_size;

  i32 fp;                       // frame pointer
  i32 sp;                       // stack pointer
  i32 ip;                       // instruction pointer

  i32 global;                   // single segment of memory at top of stack
  i32 local;                    // per-frame segment of memory for local variables

#ifdef SENI_DEBUG_MODE
  seni_vm_debug_info debug;     // debug info regarding vm
#endif

} seni_virtual_machine;

seni_var *stack_push(seni_virtual_machine *vm);
seni_var *stack_pop(seni_virtual_machine *vm);
seni_var *stack_peek(seni_virtual_machine *vm);
seni_var *stack_peek2(seni_virtual_machine *vm);

seni_virtual_machine *virtual_machine_construct(i32 stack_size, i32 heap_size);
void virtual_machine_free(seni_virtual_machine *vm);
void pretty_print_virtual_machine(seni_virtual_machine *vm, char* msg);

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
  i32 argument_offsets[MAX_NUM_ARGUMENTS];
} seni_fn_info;


typedef void (*native_function_ptr)(seni_virtual_machine *vm, i32 num_args);
typedef struct {
  native_function_ptr function_ptr[MAX_NATIVE_LOOKUPS];
} seni_vm_environment;


typedef struct {
  seni_bytecode *code;
  i32 code_max_size;
  i32 code_size;

  // variables used during compilation phase
  //
  i32 opcode_offset;
  i32 global_mappings[MEMORY_GLOBAL_SIZE]; // top-level defines
  i32 local_mappings[MEMORY_LOCAL_SIZE]; // store which wlut values are stored in which local memory addresses
  seni_fn_info *current_fn_info;

  seni_fn_info fn_info[MAX_TOP_LEVEL_FUNCTIONS];

  // todo: splut up seni_word_lut, keep the word array in seni_program but move the
  // native and keyword stuff into their own structure that can be shared amongst
  // multiple seni_programs
  seni_word_lut *wl;

  seni_vm_environment *vm_environment;

} seni_program;

seni_program *program_allocate(i32 code_max_size);
void program_free(seni_program *program);
void program_pretty_print(seni_program *program);

seni_vm_environment *vm_environment_construct();
void vm_environment_free(seni_vm_environment *e);

void declare_vm_keyword(seni_word_lut *wlut, char *name);
void declare_vm_native(seni_word_lut *wlut, char *name, seni_vm_environment *e, native_function_ptr function_ptr);

void compiler_compile(seni_node *ast, seni_program *program);
void vm_interpret(seni_virtual_machine *vm, seni_program *program);

#endif
