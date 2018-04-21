#pragma once

#include "config.h"
#include "types.h"

/* word lookup table */
struct senie_word_lut {

  senie_multistring* native_buffer;
  senie_multistring* keyword_buffer;
  senie_multistring* word_buffer;

  // set once at startup - doesn't change
  senie_string_ref* native_ref; // array MAX_NATIVE_LOOKUPS in size
  i32               native_count;

  // set once at startup - doesn't change
  senie_string_ref* keyword_ref; // array MAX_KEYWORD_LOOKUPS in size
  i32               keyword_count;

  // set once for each script
  senie_string_ref* word_ref; // array MAX_WORD_LOOKUPS in size
  i32               word_count;
};

// word lookup
senie_word_lut* wlut_allocate();
void            wlut_free(senie_word_lut* word_lut);
void            wlut_reset_words(senie_word_lut* word_lut);
char*           wlut_get_word(senie_word_lut* word_lut, i32 iword);
char*           wlut_reverse_lookup(senie_word_lut* word_lut, i32 iword);
void            wlut_pretty_print(char* msg, senie_word_lut* word_lut);

bool wlut_add_native(senie_word_lut* word_lut, char* name);
bool wlut_add_keyword(senie_word_lut* word_lut, char* name);
bool wlut_add_word(senie_word_lut* word_lut, char* name, size_t len);

// which value to use from the unions that are specified in both senie_node and
// senie_var
typedef enum {
  USE_UNKNOWN,
  USE_I,          // integer
  USE_F,          // float
  USE_L,          // long
  USE_V,          // pointer to senie_var
  USE_SRC,        // (senie_node only) pointer to original source (for whitespace +
                  // comments)
  USE_FIRST_CHILD // (senie_node only) first_child
} senie_value_in_use;

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
} senie_node_type;

senie_value_in_use get_node_value_in_use(senie_node_type type);

struct senie_node {
  senie_node_type type;

  union {
    i32                i;
    f32                f;
    struct senie_node* first_child; /* list node */
  } value;

  char* src;     // pointer back into the source
  i32   src_len; // length of source item

  int                alterable;
  struct senie_gene* gene; // only valid if alterable != 0

  // node mutate specific
  struct senie_node* parameter_ast;

  // need a place for nodes that occur within curly brackets that should
  // be ignored, e.g. the whitespace before the 2 in: (+ 1 { 2} (int))
  struct senie_node* parameter_prefix;

  /* for parameter_ast, parameter_prefix, first_child */
  struct senie_node* prev;
  struct senie_node* next;
};

// returns the first meaningful (non-whitespace, non-comment) node from expr
// onwards
senie_node* safe_first(senie_node* expr);
// returns the first meaningful (non-whitespace, non-comment) child node from
// expr onwards
senie_node* safe_first_child(senie_node* expr);
// returns the next meaningful (non-whitespace, non-comment) node from expr
senie_node* safe_next(senie_node* expr);
senie_node* safe_prev(senie_node* expr);
char*       node_type_name(senie_node* node);
void        node_pretty_print(char* msg, senie_node* node, senie_word_lut* word_lut);
void        ast_pretty_print(senie_node* ast, senie_word_lut* word_lut);

bool is_node_colour_constructor(senie_node* node);

// start at 128 just to make it easier to spot mistakes when transforming
// senie_node_type -> senie_var_type
typedef enum {
  VAR_INT = 128, // value.i
  VAR_FLOAT,     // value.f
  VAR_BOOLEAN,   // value.i
  VAR_LONG,      // value.l
  VAR_NAME,      // senie_word_lut[value.i]
  VAR_VECTOR,    // pointer to first heap allocated senie_var is in value.v
  VAR_COLOUR,    // pointer to a colour: format is in value.i and elements in
                 // f32_array
  VAR_2D,
} senie_var_type;

senie_value_in_use get_var_value_in_use(senie_var_type type);

struct senie_var {
  senie_var_type type;

  union {
    i32               i;
    f32               f;
    u64               l; // long - used by senie_prng_state
    struct senie_var* v;
  } value;

  bool mark;

  // 4 floats used to represent colours, 2d/3d/4d vectors and quaternions
  // without resorting to expensive heap_slab allocated reference counted
  // vectors
  f32 f32_array[4];

  /* for linked list used by the pool and for elements in a vector */
  struct senie_var* prev;
  struct senie_var* next;
};

char* var_type_name(senie_var* var);
void  var_pretty_print(char* msg, senie_var* var);
bool  var_serialize(senie_cursor* cursor, senie_var* var);
bool  var_deserialize(senie_var* out, senie_cursor* cursor);
void  v2_as_var(senie_var* out, f32 x, f32 y);
void  f32_as_var(senie_var* out, f32 f);
void  i32_as_var(senie_var* out, i32 i);
void  name_as_var(senie_var* out, i32 name);
void  colour_as_var(senie_var* out, senie_colour* c);

// Memory Segments
//
typedef enum {
  MEM_SEG_ARGUMENT, // store the function's arguments
  MEM_SEG_LOCAL,    // store the function's local arguments
  MEM_SEG_GLOBAL,   // global variables shared by all functions
  MEM_SEG_CONSTANT, // pseudo-segment holds constants in range 0..32767
  MEM_SEG_VOID,     // nothing
} senie_memory_segment_type;

// known memory addresses

#define SP 0
#define LCL 1
#define ARG 2

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
#define OPCODE(name, _) name,
#include "opcodes.h"
#undef OPCODE
} senie_opcode;

static const char* opcode_string[] = {
#define OPCODE(name, _) #name,
#include "opcodes.h"
#undef OPCODE
};

struct senie_bytecode {
  senie_opcode op;
  senie_var    arg0;
  senie_var    arg1;
};

void bytecode_pretty_print(i32 ip, senie_bytecode* b, senie_word_lut* word_lut);
bool bytecode_serialize(senie_cursor* cursor, senie_bytecode* bytecode);
bool bytecode_deserialize(senie_bytecode* out, senie_cursor* cursor);

struct senie_fn_info {
  bool active; // is this struct being used

  i32 index; // the index into program->fn_info
  i32 fn_name;
  i32 arg_address;
  i32 body_address;
  i32 num_args;

  // need to know the argument's inames and default values
  // add a new constraint that the default arguments need to be easily
  // evaluatable at compile time

  i32 argument_offsets[MAX_NUM_ARGUMENTS];
};

struct senie_vm;

typedef senie_var* (*native_function_ptr)(struct senie_vm* vm, i32 num_args);

struct senie_env {
  native_function_ptr function_ptr[MAX_NATIVE_LOOKUPS];

  senie_word_lut* word_lut;
};

senie_env* env_allocate();
void       env_free(senie_env* e);

struct senie_program {
  senie_bytecode* code;
  i32             code_max_size;
  i32             code_size;

  // variables used during compilation phase, won't be available during runtime
  //
  i32 opcode_offset;
  i32 global_mappings[MEMORY_GLOBAL_SIZE]; // top-level defines
  i32 local_mappings[MEMORY_LOCAL_SIZE];   // store which word_lut values are
                                           // stored in which local memory
                                           // addresses
  senie_fn_info* current_fn_info;

  senie_fn_info fn_info[MAX_TOP_LEVEL_FUNCTIONS];

  senie_word_lut* word_lut; //  needed in senie_program for error messages
};

char* opcode_name(senie_opcode opcode);

void program_free(senie_program* program);
i32  program_stop_location(senie_program* program);
void program_pretty_print(senie_program* program);

bool program_serialize(senie_cursor* cursor, senie_program* program);
bool program_deserialize(senie_program* out, senie_cursor* cursor);

senie_program* program_allocate(i32 code_max_size);

struct senie_vm {
  senie_program* program;
  senie_env*     env;

  senie_render_data* render_data; // stores the generated vertex data

  senie_matrix_stack* matrix_stack;

  senie_prng_state* prng_state; // only used when evaluating bracket bindings

  i32        heap_size;
  senie_var* heap_slab;          // the contiguous block of allocated memory
  senie_var* heap_avail;         // doubly linked list of unallocated senie_vars from the
                                 // heap_slab
  i32 heap_avail_size_before_gc; // how small can the heap get before a gc is
                                 // invoked

  i32 heap_avail_size;

  u64 opcodes_executed;
  f32 execution_time; // in msec

  senie_var* stack;
  i32        stack_size;

  i32 fp; // frame pointer
  i32 sp; // stack pointer
  i32 ip; // instruction pointer

  i32 global; // single segment of memory at top of stack
  i32 local;  // per-frame segment of memory for local variables
};

senie_vm*
     vm_allocate(i32 stack_size, i32 heap_size, i32 heap_min_size, i32 vertex_packet_num_vertices);
void vm_reset(senie_vm* vm);
void vm_free(senie_vm* vm);
void vm_free_render_data(senie_vm* vm);
void vm_pretty_print(senie_vm* vm, char* msg);

// access global variables when they're in a known location
senie_var* vm_get_from_global_offset(senie_vm* vm, i32 offset);
senie_var* vm_stack_peek(senie_vm* vm);

void       vector_construct(senie_var* head);
i32        vector_length(senie_var* var);
senie_var* vector_get(senie_var* var, i32 index);
void       vector_append_heap_var(senie_var* head, senie_var* val);
senie_var* vector_append_i32(senie_vm* vm, senie_var* head, i32 val);
senie_var* vector_append_f32(senie_vm* vm, senie_var* head, f32 val);
senie_var* vector_append_u64(senie_vm* vm, senie_var* head, u64 val);
senie_var* vector_append_col(senie_vm* vm, senie_var* head, senie_colour* col);

senie_var* var_get_from_heap(senie_vm* vm);
void       var_copy(senie_var* dest, senie_var* src);

void vm_debug_info_reset(senie_vm* vm);
void vm_debug_info_print(senie_vm* vm);

void lang_subsystem_startup();
void lang_subsystem_shutdown();
// get/return senie_var from pool
senie_var* var_get_from_pool();
void       var_return_to_pool(senie_var* var);
