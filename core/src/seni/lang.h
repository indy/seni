#pragma once

#include "config.h"
#include "types.h"

/* word lookup table */
struct sen_word_lut {

  sen_multistring* native_buffer;
  sen_multistring* keyword_buffer;
  sen_multistring* word_buffer;

  // set once at startup - doesn't change
  sen_string_ref* native_ref; // array MAX_NATIVE_LOOKUPS in size
  i32             native_count;

  // set once at startup - doesn't change
  sen_string_ref* keyword_ref; // array MAX_KEYWORD_LOOKUPS in size
  i32             keyword_count;

  // set once for each script
  sen_string_ref* word_ref; // array MAX_WORD_LOOKUPS in size
  i32             word_count;
};

// word lookup
sen_word_lut* wlut_allocate();
void          wlut_free(sen_word_lut* word_lut);
void          wlut_reset_words(sen_word_lut* word_lut);
char*         wlut_get_word(sen_word_lut* word_lut, i32 iword);
char*         wlut_reverse_lookup(sen_word_lut* word_lut, i32 iword);
void          wlut_pretty_print(char* msg, sen_word_lut* word_lut);

bool wlut_add_native(sen_word_lut* word_lut, char* name);
bool wlut_add_keyword(sen_word_lut* word_lut, char* name);
bool wlut_add_word(sen_word_lut* word_lut, char* name, size_t len);

// which value to use from the unions that are specified in both sen_node and
// sen_var
typedef enum {
  USE_UNKNOWN,
  USE_I,           // integer
  USE_F,           // float
  USE_L,           // long
  USE_V,           // pointer to sen_var
  USE_SRC,         // (sen_node only) pointer to original source (for whitespace +
                   // comments)
  USE_I_AND_ARRAY, // used by VAR_COLOUR
  USE_FIRST_CHILD  // (sen_node only) first_child
} sen_value_in_use;

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
} sen_node_type;

sen_value_in_use get_node_value_in_use(sen_node_type type);

struct sen_node {
  sen_node_type type;

  union {
    i32              i;
    f32              f;
    struct sen_node* first_child; /* list node */
  } value;

  char* src;     // pointer back into the source
  i32   src_len; // length of source item

  int              alterable;
  struct sen_gene* gene; // only valid if alterable != 0

  // node mutate specific
  struct sen_node* parameter_ast;

  // need a place for nodes that occur within curly brackets that should
  // be ignored, e.g. the whitespace before the 2 in: (+ 1 { 2} (int))
  struct sen_node* parameter_prefix;

  /* for parameter_ast, parameter_prefix, first_child */
  struct sen_node* prev;
  struct sen_node* next;
};

// returns the first meaningful (non-whitespace, non-comment) node from expr
// onwards
sen_node* safe_first(sen_node* expr);
// returns the first meaningful (non-whitespace, non-comment) child node from
// expr onwards
sen_node* safe_first_child(sen_node* expr);
// returns the next meaningful (non-whitespace, non-comment) node from expr
sen_node* safe_next(sen_node* expr);
sen_node* safe_prev(sen_node* expr);
char*     node_type_name(sen_node* node);
void      node_pretty_print(char* msg, sen_node* node, sen_word_lut* word_lut);
void      ast_pretty_print(sen_node* ast, sen_word_lut* word_lut);

bool is_node_colour_constructor(sen_node* node);

// start at 128 just to make it easier to spot mistakes when transforming
// sen_node_type -> sen_var_type
typedef enum {
  VAR_INT = 128, // value.i
  VAR_FLOAT,     // value.f
  VAR_BOOLEAN,   // value.i
  VAR_LONG,      // value.l
  VAR_NAME,      // sen_word_lut[value.i]
  VAR_VECTOR,    // pointer to first heap allocated sen_var is in value.v
  VAR_COLOUR,    // pointer to a colour: format is in value.i and elements in
                 // f32_array
  VAR_2D,
} sen_var_type;

sen_value_in_use get_var_value_in_use(sen_var_type type);

struct sen_var {
  sen_var_type type;

  union {
    i32             i;
    f32             f;
    u64             l; // long - used by sen_prng_state
    struct sen_var* v;
  } value;

  bool mark;

  // 4 floats used to represent colours, 2d/3d/4d vectors and quaternions
  // without resorting to expensive heap_slab allocated reference counted
  // vectors
  f32 f32_array[4];

  /* for linked list used by the pool and for elements in a vector */
  struct sen_var* prev;
  struct sen_var* next;
};

char* var_type_name(sen_var* var);
void  var_pretty_print(char* msg, sen_var* var);
bool  var_serialize(sen_cursor* cursor, sen_var* var);
bool  var_deserialize(sen_var* out, sen_cursor* cursor);
void  v2_as_var(sen_var* out, f32 x, f32 y);
void  f32_as_var(sen_var* out, f32 f);
void  i32_as_var(sen_var* out, i32 i);
void  name_as_var(sen_var* out, i32 name);
void  colour_as_var(sen_var* out, sen_colour* c);

// Memory Segments
//
typedef enum {
  MEM_SEG_ARGUMENT, // store the function's arguments
  MEM_SEG_LOCAL,    // store the function's local arguments
  MEM_SEG_GLOBAL,   // global variables shared by all functions
  MEM_SEG_CONSTANT, // pseudo-segment holds constants in range 0..32767
  MEM_SEG_VOID,     // nothing
} sen_memory_segment_type;

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
} sen_opcode;

struct sen_bytecode {
  sen_opcode op;
  sen_var    arg0;
  sen_var    arg1;
};

void bytecode_pretty_print(i32 ip, sen_bytecode* b, sen_word_lut* word_lut);
bool bytecode_serialize(sen_cursor* cursor, sen_bytecode* bytecode);
bool bytecode_deserialize(sen_bytecode* out, sen_cursor* cursor);

struct sen_fn_info {
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

struct sen_vm;

typedef sen_var* (*native_function_ptr)(struct sen_vm* vm, i32 num_args);

struct sen_env {
  native_function_ptr function_ptr[MAX_NATIVE_LOOKUPS];

  sen_word_lut* word_lut;
};

sen_env* env_allocate();
void     env_free(sen_env* e);
void     env_reset(sen_env* e);

struct sen_program {
  sen_bytecode* code;
  i32           code_max_size;
  i32           code_size;

  sen_fn_info   fn_info[MAX_TOP_LEVEL_FUNCTIONS];
  sen_word_lut* word_lut; //  needed in sen_program for error messages
};

char* opcode_name(sen_opcode opcode);

sen_program* program_construct(sen_compiler_config* compiler_config);
sen_program* program_allocate(i32 code_max_size);
void         program_reset(sen_program* program);
void         program_free(sen_program* program);
i32          program_stop_location(sen_program* program);
void         program_pretty_print(sen_program* program);

bool program_serialize(sen_cursor* cursor, sen_program* program);
bool program_deserialize(sen_program* out, sen_cursor* cursor);

struct sen_vm {
  sen_program* program;
  sen_env*     env;

  sen_render_data* render_data; // stores the generated vertex data

  sen_matrix_stack* matrix_stack;

  sen_prng_state* prng_state; // only used when evaluating bracket bindings

  i32      heap_size;
  sen_var* heap_slab;            // the contiguous block of allocated memory
  sen_var* heap_avail;           // doubly linked list of unallocated sen_vars from the
                                 // heap_slab
  i32 heap_avail_size_before_gc; // how small can the heap get before a gc is
                                 // invoked

  i32 heap_avail_size;

  u64 opcodes_executed;
  f32 execution_time; // in msec

  sen_var* stack;
  i32      stack_size;

  i32 fp; // frame pointer
  i32 sp; // stack pointer
  i32 ip; // instruction pointer

  i32 global; // single segment of memory at top of stack
  i32 local;  // per-frame segment of memory for local variables

  i32 building_with_trait_within_vector;
  i32 trait_within_vector_index;
};

sen_vm* vm_allocate(i32 stack_size, i32 heap_size, i32 heap_min_size,
                    i32 vertex_packet_num_vertices);
void    vm_reset(sen_vm* vm);
void    vm_free(sen_vm* vm);
void    vm_free_render_data(sen_vm* vm);
void    vm_pretty_print(sen_vm* vm, char* msg);

// access global variables when they're in a known location
sen_var* vm_get_from_global_offset(sen_vm* vm, i32 offset);
sen_var* vm_stack_peek(sen_vm* vm);

void     vector_construct(sen_var* head);
i32      vector_length(sen_var* var);
sen_var* vector_get(sen_var* var, i32 index);
void     vector_append_heap_var(sen_var* head, sen_var* val);
sen_var* vector_append_i32(sen_vm* vm, sen_var* head, i32 val);
sen_var* vector_append_f32(sen_vm* vm, sen_var* head, f32 val);
sen_var* vector_append_u64(sen_vm* vm, sen_var* head, u64 val);
sen_var* vector_append_col(sen_vm* vm, sen_var* head, sen_colour* col);

sen_var* var_get_from_heap(sen_vm* vm);
void     var_copy(sen_var* dest, sen_var* src);

void vm_debug_info_reset(sen_vm* vm);
void vm_debug_info_print(sen_vm* vm);

void lang_subsystem_startup();
void lang_subsystem_shutdown();
// get/return sen_var from pool
sen_var* var_get_from_pool();
void     var_return_to_pool(sen_var* var);
