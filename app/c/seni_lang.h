#ifndef SENI_LANG_H
#define SENI_LANG_H

#include "seni_config.h"
#include "seni_types.h"
#include "seni_render_packet.h"
#include "seni_matrix.h"
#include "seni_colour.h"
#include "seni_keyword_iname.h"

/* word lookup table */
typedef struct seni_word_lut {
  // set once at startup - doesn't change
  char *native[MAX_NATIVE_LOOKUPS];  
  i32 native_count;

  // set once at startup - doesn't change
  char *keyword[MAX_KEYWORD_LOOKUPS];  
  i32 keyword_count;

  // set once for each script
  char *word[MAX_WORD_LOOKUPS];
  i32 word_count;
} seni_word_lut;

typedef enum {
  NODE_LIST = 0,
  NODE_VECTOR,
  NODE_INT,
  NODE_FLOAT,
  NODE_NAME,
  NODE_LABEL,
  NODE_STRING,
  NODE_WHITESPACE,
  NODE_COMMENT
} seni_node_type;

typedef struct seni_node {
  seni_node_type type;

  union {
    i32 i;
    f32 f;
    char* s;                     /* needed for whitespace/comment nodes */
    struct seni_node *first_child;  /* list node */
  } value;

  bool alterable;

  // node mutate specific
  struct seni_node *parameter_ast;

  // need a place for nodes that occur within curly brackets that should
  // be ignored, e.g. the whitespace before the 2 in: (+ 1 { 2} (int))
  struct seni_node *parameter_prefix;

  /* for parameter_ast, parameter_prefix, first_child */
  struct seni_node *prev;
  struct seni_node *next;
} seni_node;

// start at 128 just to make it easier to spot mistakes when transforming seni_node_type -> seni_var_type
typedef enum {
  VAR_INT = 128, // value.i
  VAR_FLOAT,     // value.f
  VAR_BOOLEAN,   // value.i
  VAR_LONG,      // value.l
  VAR_NAME,      // seni_word_lut[value.i]
  VAR_VEC_HEAD,  // pointer to vec_rc is in value.v
  VAR_VEC_RC,    // pointer to first vector element is in value.v
  VAR_COLOUR,    // pointer to a colour: format is in value.i and elements in f32_array
  VAR_2D,
} seni_var_type;

// which value to use
typedef enum {
  USE_I,                        // integer
  USE_F,                        // float
  USE_L,                        // long
  USE_V                         // pointer to seni_var
} seni_value_in_use;

typedef struct seni_var {
  seni_var_type type;

  union {
    i32 i;
    f32 f;
    u64 l;                      // long - used by seni_prng_state
    i32 ref_count;              // reference count for VAR_VEC_RC
    struct seni_var *v;
  } value;

  bool allocated;

  // 4 floats used to represent colours, 2d/3d/4d vectors and quaternions
  // without resorting to expensive heap_slab allocated reference counted vectors
  f32 f32_array[4];

  /* for linked list used by the pool and for elements in a vector */
  struct seni_var *prev;
  struct seni_var *next;
} seni_var;

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

#define HEAP_SIZE 1024
#define STACK_SIZE 1024
#define MEMORY_GLOBAL_SIZE 20
#define MEMORY_LOCAL_SIZE 40

#define MAX_TOP_LEVEL_FUNCTIONS 32

#define MAX_NUM_ARGUMENTS 16

// Virtual Machine
//
typedef struct {
  i32 size;
  i32 get_count;
  i32 return_count;

  i32 delta;                    // get == +1, return == -1
  i32 high_water_mark;          // max(delta) == the highest number of elements that were in use at one time
} seni_slab_info;

void slab_reset(seni_slab_info *slab_info);
void slab_full_reset(seni_slab_info *slab_info);
void slab_get(seni_slab_info *slab_info);
void slab_return(seni_slab_info *slab_info, char *msg);
void slab_print(seni_slab_info *slab_info, char *message);

typedef struct seni_vm {
  seni_render_data *render_data;   // stores the generated vertex data
  
  seni_matrix_stack *matrix_stack;

  i32 heap_size;
  seni_var *heap_slab;             // the contiguous block of allocated memory
  seni_var *heap_avail;            // doubly linked list of unallocated seni_vars from the heap_slab
  seni_slab_info heap_slab_info;
  u64 opcodes_executed;
  i32 execution_time;              // in msec
  
  seni_var *stack;
  i32 stack_size;

  i32 fp;                          // frame pointer
  i32 sp;                          // stack pointer
  i32 ip;                          // instruction pointer

  i32 global;                      // single segment of memory at top of stack
  i32 local;                       // per-frame segment of memory for local variables

} seni_vm;

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

// todo: replace returned seni_var with a seni_var *
typedef seni_var (*native_function_ptr)(seni_vm *vm, i32 num_args);
typedef struct {
  native_function_ptr function_ptr[MAX_NATIVE_LOOKUPS];
} seni_env;


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

  // todo: split up seni_word_lut, keep the word array in seni_program but move the
  // native and keyword stuff into their own structure that can be shared amongst
  // multiple seni_programs
  seni_word_lut *wl;

  seni_env *env;

} seni_program;

// word lookup
seni_word_lut *wlut_allocate();
void           wlut_free(seni_word_lut *wlut);
void           wlut_reset_words(seni_word_lut *wlut);
  
// parser
seni_node     *parser_parse(seni_word_lut *wlut, char *s);
void           parser_free_nodes(seni_node *nodes);

char          *node_type_name(seni_node *node);
char          *var_type_name(seni_var *var);

seni_var      *stack_peek(seni_vm *vm);
seni_var      *var_get_from_heap(seni_vm *vm);
void           var_return_to_heap(seni_vm *vm,  seni_var *var);

seni_vm       *vm_construct(i32 stack_size, i32 heap_size);
void           vm_reset(seni_vm *vm);
void           vm_free(seni_vm *vm);
void           vm_free_render_data(seni_vm *vm);

seni_program  *program_allocate(i32 code_max_size);
void           program_free(seni_program *program);
void           pretty_print_program(seni_program *program);

seni_env      *env_construct();
void           env_free(seni_env *e);

void           compiler_compile(seni_node *ast, seni_program *program);
bool           vm_interpret(seni_vm *vm, seni_program *program);
bool           var_copy(seni_vm *vm, seni_var *dest, seni_var *src);

void           pretty_print_seni_var(seni_var *var, char* msg);

void           vector_construct(seni_vm *vm, seni_var *head);
void           append_to_vector_i32(seni_vm *vm, seni_var *head, i32 val);
void           append_to_vector_f32(seni_vm *vm, seni_var *head, f32 val);
void           append_to_vector_u64(seni_vm *vm, seni_var *head, u64 val);
void           append_to_vector_col(seni_vm *vm, seni_var *head, seni_colour *col);
bool           append_to_vector(seni_vm *vm, seni_var *head, seni_var *val);

void           f32_as_var(seni_var *out, f32 f);
void           i32_as_var(seni_var *out, i32 i);
void           colour_as_var(seni_var *out, seni_colour *c);


#ifdef SENI_DEBUG_MODE

void vm_debug_info_reset(seni_vm *vm);
void vm_debug_info_print(seni_vm *vm);

// record information during execution of bytecode
#define DEBUG_INFO_RESET(vm) vm_debug_info_reset(vm)
#define DEBUG_INFO_PRINT(vm) vm_debug_info_print(vm)
#define DEBUG_INFO_GET_FROM_HEAP(vm) slab_get(&(vm->heap_slab_info))
#define DEBUG_INFO_RETURN_TO_HEAP(vm) slab_return(&(vm->heap_slab_info), "RETURN_TO_HEAP")

#else

// do nothing
#define DEBUG_INFO_RESET(vm)
#define DEBUG_INFO_PRINT(vm)
#define DEBUG_INFO_GET_FROM_HEAP(vm)
#define DEBUG_INFO_RETURN_TO_HEAP(vm)

#endif



#endif
