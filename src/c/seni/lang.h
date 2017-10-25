#pragma once

#include "config.h"
#include "types.h"

/* word lookup table */
struct seni_word_lut {

  seni_multistring *native_buffer;
  seni_multistring *keyword_buffer;
  seni_multistring *word_buffer;
  
  // set once at startup - doesn't change
  seni_string_ref *native_ref; // array MAX_NATIVE_LOOKUPS in size
  i32 native_count;

  // set once at startup - doesn't change
  seni_string_ref *keyword_ref; // array MAX_KEYWORD_LOOKUPS in size
  i32 keyword_count;

  // set once for each script
  seni_string_ref *word_ref; // array MAX_WORD_LOOKUPS in size
  i32 word_count;
};

// word lookup
seni_word_lut *wlut_allocate();
void           wlut_free(seni_word_lut *word_lut);
void           wlut_reset_words(seni_word_lut *word_lut);
char          *wlut_get_word(seni_word_lut *word_lut, i32 iword);
char          *wlut_reverse_lookup(seni_word_lut *word_lut, i32 iword);
void           wlut_pretty_print(char *msg, seni_word_lut *word_lut);

bool           wlut_add_native(seni_word_lut *word_lut, char *name);
bool           wlut_add_keyword(seni_word_lut *word_lut, char *name);
bool           wlut_add_word(seni_word_lut *word_lut, char *name, size_t len);

// which value to use from the unions that are specified in both seni_node and seni_var
typedef enum {
  USE_UNKNOWN,
  USE_I,                        // integer
  USE_F,                        // float
  USE_L,                        // long
  USE_V,                        // pointer to seni_var
  USE_SRC,                      // (seni_node only) pointer to original source (for whitespace + comments)
  USE_FIRST_CHILD               // (seni_node only) first_child
} seni_value_in_use;

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

seni_value_in_use get_node_value_in_use(seni_node_type type);

struct seni_node {
  seni_node_type type;

  union {
    i32 i;
    f32 f;
    struct seni_node *first_child;  /* list node */
  } value;

  char *src;                    // pointer back into the source
  i32 src_len;                  // length of source item

  int alterable;
  struct seni_gene *gene;       // only valid if alterable != 0

  // node mutate specific
  struct seni_node *parameter_ast;

  // need a place for nodes that occur within curly brackets that should
  // be ignored, e.g. the whitespace before the 2 in: (+ 1 { 2} (int))
  struct seni_node *parameter_prefix;

  /* for parameter_ast, parameter_prefix, first_child */
  struct seni_node *prev;
  struct seni_node *next;
};

// returns the first meaningful (non-whitespace, non-comment) node from expr onwards
seni_node *safe_first(seni_node *expr);
// returns the first meaningful (non-whitespace, non-comment) child node from expr onwards
seni_node *safe_first_child(seni_node *expr);
// returns the next meaningful (non-whitespace, non-comment) node from expr
seni_node *safe_next(seni_node *expr);
seni_node *safe_prev(seni_node *expr);
char      *node_type_name(seni_node *node);
void       node_pretty_print(char* msg, seni_node *node, seni_word_lut *word_lut);

bool       is_node_colour_constructor(seni_node *node);

// start at 128 just to make it easier to spot mistakes when transforming seni_node_type -> seni_var_type
typedef enum {
  VAR_INT = 128, // value.i
  VAR_FLOAT,     // value.f
  VAR_BOOLEAN,   // value.i
  VAR_LONG,      // value.l
  VAR_NAME,      // seni_word_lut[value.i]
  VAR_VECTOR,    // pointer to first heap allocated seni_var is in value.v
  VAR_COLOUR,    // pointer to a colour: format is in value.i and elements in f32_array
  VAR_2D,
} seni_var_type;

seni_value_in_use get_var_value_in_use(seni_var_type type);

struct seni_var {
  seni_var_type type;

  union {
    i32 i;
    f32 f;
    u64 l;                      // long - used by seni_prng_state
    struct seni_var *v;
  } value;

  bool mark;

  // 4 floats used to represent colours, 2d/3d/4d vectors and quaternions
  // without resorting to expensive heap_slab allocated reference counted vectors
  f32 f32_array[4];

  /* for linked list used by the pool and for elements in a vector */
  struct seni_var *prev;
  struct seni_var *next;
};

char     *var_type_name(seni_var *var);
void      var_pretty_print(char* msg, seni_var *var);
bool      var_serialize(seni_cursor *cursor, seni_var *var);
bool      var_deserialize(seni_var *out, seni_cursor *cursor);
void      v2_as_var(seni_var *out, f32 x, f32 y);
void      f32_as_var(seni_var *out, f32 f);
void      i32_as_var(seni_var *out, i32 i);
void      name_as_var(seni_var *out, i32 name);
void      colour_as_var(seni_var *out, seni_colour *c);

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

// how low can the heap go before a GC is invoked
//
#define HEAP_MIN_SIZE 10
#define MEMORY_GLOBAL_SIZE 40
#define MEMORY_LOCAL_SIZE 40

#define MAX_TOP_LEVEL_FUNCTIONS 32

#define MAX_NUM_ARGUMENTS 16

// Virtual Machine
//

// codes
//
typedef enum {
#define OPCODE(name,_) name,
#include "opcodes.h"
#undef OPCODE
} seni_opcode;

static const char *opcode_string[] = {
#define OPCODE(name,_) #name,
#include "opcodes.h"
#undef OPCODE
};

struct seni_bytecode {
  seni_opcode op;
  seni_var arg0;
  seni_var arg1;
};

void bytecode_pretty_print(i32 ip, seni_bytecode *b, seni_word_lut *word_lut);
bool bytecode_serialize(seni_cursor *cursor, seni_bytecode *bytecode);
bool bytecode_deserialize(seni_bytecode *out, seni_cursor *cursor);

struct seni_fn_info {
  bool active;                  // is this struct being used

  i32 index;                    // the index into program->fn_info
  i32 fn_name;
  i32 arg_address;
  i32 body_address;
  i32 num_args;

  // need to know the argument's inames and default values
  // add a new constraint that the default arguments need to be easily evaluatable at compile time
  
  i32 argument_offsets[MAX_NUM_ARGUMENTS];
};

struct seni_vm;

typedef seni_var *(*native_function_ptr)(struct seni_vm *vm, i32 num_args);

struct seni_env {
  native_function_ptr function_ptr[MAX_NATIVE_LOOKUPS];

  seni_word_lut *word_lut;
};

seni_env      *env_allocate();
void           env_free(seni_env *e);

struct seni_program {
  seni_bytecode *code;
  i32 code_max_size;
  i32 code_size;

  // variables used during compilation phase
  //
  i32 opcode_offset;
  i32 global_mappings[MEMORY_GLOBAL_SIZE]; // top-level defines
  i32 local_mappings[MEMORY_LOCAL_SIZE]; // store which word_lut values are stored in which local memory addresses
  seni_fn_info *current_fn_info;

  seni_fn_info fn_info[MAX_TOP_LEVEL_FUNCTIONS];

  seni_word_lut *word_lut;      //  needed in seni_program for error messages

};

char           *opcode_name(seni_opcode opcode);

void           program_free(seni_program *program);
i32            program_stop_location(seni_program *program);
void           program_pretty_print(seni_program *program);

bool           program_serialize(seni_cursor *cursor, seni_program *program);
bool           program_deserialize(seni_program *out, seni_cursor *cursor);

seni_program  *program_allocate(i32 code_max_size);

struct seni_vm {
  seni_program *program;
  seni_env *env;
  
  seni_render_data *render_data;   // stores the generated vertex data
  
  seni_matrix_stack *matrix_stack;

  seni_prng_state *prng_state;     // only used when evaluating bracket bindings

  i32 heap_size;
  seni_var *heap_slab;             // the contiguous block of allocated memory
  seni_var *heap_avail;            // doubly linked list of unallocated seni_vars from the heap_slab
  i32 heap_avail_size_before_gc;   // how small can the heap get before a gc is invoked

  i32 heap_avail_size;
  
  u64 opcodes_executed;
  f32 execution_time;              // in msec
  
  seni_var *stack;
  i32 stack_size;

  i32 fp;                          // frame pointer
  i32 sp;                          // stack pointer
  i32 ip;                          // instruction pointer

  i32 global;                      // single segment of memory at top of stack
  i32 local;                       // per-frame segment of memory for local variables

};

seni_vm  *vm_allocate(i32 stack_size, i32 heap_size, i32 heap_min_size, i32 vertex_packet_num_vertices);
void      vm_reset(seni_vm *vm);
void      vm_free(seni_vm *vm);
void      vm_free_render_data(seni_vm *vm);
void      vm_pretty_print(seni_vm *vm, char* msg);

seni_var *stack_peek(seni_vm *vm);

void      vector_construct(seni_var *head);
i32       vector_length(seni_var *var);
seni_var *vector_get(seni_var *var, i32 index);
void      vector_append_heap_var(seni_var *head, seni_var *val);
seni_var *vector_append_i32(seni_vm *vm, seni_var *head, i32 val);
seni_var *vector_append_f32(seni_vm *vm, seni_var *head, f32 val);
seni_var *vector_append_u64(seni_vm *vm, seni_var *head, u64 val);
seni_var *vector_append_col(seni_vm *vm, seni_var *head, seni_colour *col);

seni_var *var_get_from_heap(seni_vm *vm);
void      var_copy(seni_var *dest, seni_var *src);

void      vm_debug_info_reset(seni_vm *vm);
void      vm_debug_info_print(seni_vm *vm);


void lang_subsystem_startup();
void lang_subsystem_shutdown();
// get/return seni_var from pool
seni_var *var_get_from_pool();
void var_return_to_pool(seni_var *var);
