#include "seni_lang.h"
#include "seni_config.h"
#include "seni_matrix.h"
#include "seni_mathutil.h"
#include "seni_printf.h"
#include "seni_bind.h"
#include "seni_vm_parser.h"
#include "seni_vm_compiler.h"

#include <string.h>
#include <stdlib.h>
#include <inttypes.h>

#include "utlist.h"

void wlut_free_keywords(seni_word_lut *wlut)
{
  for( int i = 0; i < MAX_KEYWORD_LOOKUPS; i++) {
    if (wlut->keyword[i]) {
      free(wlut->keyword[i]);
    }
    wlut->keyword[i] = 0;      
  }
  wlut->keyword_count = 0;
}

void wlut_free_natives(seni_word_lut *wlut)
{
  for( int i = 0; i < MAX_NATIVE_LOOKUPS; i++) {
    if (wlut->native[i]) {
      free(wlut->native[i]);
    }
    wlut->native[i] = 0;      
  }
  wlut->native_count = 0;
}

void wlut_free_words(seni_word_lut *wlut)
{
  for( int i = 0; i < MAX_WORD_LOOKUPS; i++) {
    if (wlut->word[i]) {
      free(wlut->word[i]);
    }
    wlut->word[i] = 0;      
  }
  wlut->word_count = 0;
}

seni_word_lut *wlut_allocate()
{
  seni_word_lut *wl = (seni_word_lut *)calloc(1, sizeof(seni_word_lut));
  return wl;
}

void wlut_free(seni_word_lut *wlut)
{
  wlut_free_words(wlut);
  wlut_free_keywords(wlut);
  wlut_free_natives(wlut);
  free(wlut);
}

// called after a script has been executed
void wlut_reset_words(seni_word_lut *wlut)
{
  wlut_free_words(wlut);
  // leave keywords and natives since they aren't mutable
}

char *wlut_get_word(seni_word_lut *word_lut, i32 iword)
{
  if (iword < word_lut->word_count) {
    return word_lut->word[iword];
  }
  return "UNKNOWN WORD";
}

char *wlut_reverse_lookup(seni_word_lut *word_lut, i32 iword)
{
  if (iword < word_lut->word_count) {
    return word_lut->word[iword];
  }
  if (iword >= KEYWORD_START && iword < KEYWORD_START + MAX_KEYWORD_LOOKUPS) {
    return word_lut->keyword[iword - KEYWORD_START];
  }
  if (iword >= NATIVE_START && iword < NATIVE_START + MAX_NATIVE_LOOKUPS) {
    return word_lut->native[iword - NATIVE_START];
  }
  return "UNKNOWN WORD";
}

void wlut_pretty_print(char *msg, seni_word_lut *word_lut)
{
  SENI_PRINT("%s native_count: %d", msg, word_lut->native_count);
  SENI_PRINT("%s keyword_count: %d", msg, word_lut->keyword_count);
  SENI_PRINT("%s word_count: %d", msg, word_lut->word_count);
}

seni_value_in_use get_node_value_in_use(seni_node_type type)
{
  switch(type) {
  case NODE_LIST:
    return USE_FIRST_CHILD; // ???
    break;
  case NODE_VECTOR:
    return USE_FIRST_CHILD; // ???
    break;
  case NODE_INT:
    return USE_I;
    break;
  case NODE_FLOAT:
    return USE_F;
    break;
  case NODE_NAME:
    return USE_I;
    break;
  case NODE_LABEL:
    return USE_I;
    break;
  case NODE_STRING:
    return USE_I;
    break;
  case NODE_WHITESPACE:
    return USE_S;
    break;
  case NODE_COMMENT:
    return USE_S;
    break;
  }

  return USE_UNKNOWN;
}

seni_node *safe_next(seni_node *expr)
{
  seni_node *sibling = expr->next;
  while(sibling && (sibling->type == NODE_WHITESPACE ||
                    sibling->type == NODE_COMMENT)) {
    sibling = sibling->next;
  }

  return sibling;
}

seni_node *safe_prev(seni_node *expr)
{
  seni_node *sibling = expr->prev;
  while(sibling && (sibling->type == NODE_WHITESPACE ||
                    sibling->type == NODE_COMMENT)) {
    sibling = sibling->prev;
  }

  return sibling;
}

char *node_type_name(seni_node *node)
{
  switch(node->type) {
  case NODE_LIST:       return "NODE_LIST";
  case NODE_VECTOR:     return "NODE_VECTOR";
  case NODE_INT:        return "NODE_INT";
  case NODE_FLOAT:      return "NODE_FLOAT";
  case NODE_NAME:       return "NODE_NAME";
  case NODE_LABEL:      return "NODE_LABEL";
  case NODE_STRING:     return "NODE_STRING";
  case NODE_WHITESPACE: return "NODE_WHITESPACE";
  case NODE_COMMENT:    return "NODE_COMMENT";
  default: return "unknown seni_node type";
  };
}

void node_pretty_print(char* msg, seni_node *node, seni_word_lut *word_lut)
{
  if (node == NULL) {
    SENI_ERROR("node_pretty_print: given NULL");
    return;
  }
  
  char *type = node_type_name(node);
  seni_value_in_use using = get_node_value_in_use(node->type);

  switch(using) {
  case USE_UNKNOWN:
    SENI_PRINT("%s: UNKNOWN %s", msg,  type);
    break;
  case USE_I:
    if (word_lut != NULL &&
        (node->type == NODE_NAME || node->type == NODE_LABEL || node->type == NODE_STRING)) {
      SENI_PRINT("%s: %s : %s (%d)", msg, type, wlut_reverse_lookup(word_lut, node->value.i), node->value.i);
    } else {
      SENI_PRINT("%s: %s : %d", msg, type, node->value.i);
    }
    break;
  case USE_F:
    SENI_PRINT("%s: %s : %.2f", msg,  type, node->value.f);
    break;
  case USE_L:
    SENI_PRINT("%s: L %s", msg,  type);
    break;
  case USE_V:
    SENI_PRINT("%s: V %s", msg,  type);
    break;
  case USE_S:
    SENI_PRINT("%s: %s", msg,  type);
    break;
  case USE_FIRST_CHILD:
    SENI_PRINT("%s: %s", msg,  type);
    break;
  default:
    SENI_ERROR("unknown using value for a seni_node: %d", using);
  }
}

seni_value_in_use get_var_value_in_use(seni_var_type type)
{
  switch(type) {
  case VAR_FLOAT:
    return USE_F;
  case VAR_LONG:
    return USE_L;
  case VAR_VECTOR:
    return USE_V;
  case VAR_COLOUR:
    return USE_I;
  default:
    // default to something even though VAR_2D etc aren't going to use anything in the value union
    return USE_I;
  };
}

char *var_type_name(seni_var *var)
{
  switch(var->type) {
  case VAR_INT:      return "VAR_INT";
  case VAR_FLOAT:    return "VAR_FLOAT";
  case VAR_BOOLEAN:  return "VAR_BOOLEAN";
  case VAR_LONG:     return "VAR_LONG";
  case VAR_NAME:     return "VAR_NAME";
  case VAR_VECTOR: return "VAR_VECTOR";
  case VAR_COLOUR:   return "VAR_COLOUR";
  case VAR_2D:       return "VAR_2D";
  default: return "unknown seni_var type";
  }
}

void var_pretty_print(char* msg, seni_var *var)
{
  if (var == NULL) {
    SENI_ERROR("var_pretty_print: given NULL");
    return;
  }
  
  char *type = var_type_name(var);
  seni_value_in_use using = get_var_value_in_use(var->type);

  switch(using) {
  case USE_I:
    if (var->type == VAR_COLOUR) {
      SENI_PRINT("%s: %s : %d (%.2f, %.2f, %.2f, %.2f)", msg, type, var->value.i,
                 var->f32_array[0], var->f32_array[1], var->f32_array[2], var->f32_array[3]);
    } else {
      SENI_PRINT("%s: %s : %d", msg, type, var->value.i);
    }
    break;
  case USE_F:
    SENI_PRINT("%s: %s : %.2f", msg,  type, var->value.f);
    break;
  case USE_L:
    SENI_PRINT("%s: %s : %llu", msg, type, (long long unsigned int)(var->value.l));
    break;
  case USE_V:
    if (var->type == VAR_VECTOR) {
      SENI_PRINT("%s: %s : length %d", msg, type, vector_length(var));
    } else {
      SENI_PRINT("%s: %s", msg,  type);
    }
    break;
  default:
    SENI_ERROR("unknown using value for a seni_var: %d", using);
  }
}

void v2_as_var(seni_var *out, f32 x, f32 y)
{
  out->type = VAR_2D;
  out->f32_array[0] = x;
  out->f32_array[1] = y;
}

void f32_as_var(seni_var *out, f32 f)
{
  out->type = VAR_FLOAT;
  out->value.f = f;
}

void i32_as_var(seni_var *out, i32 i)
{
  out->type = VAR_INT;
  out->value.i = i;
}

void colour_as_var(seni_var *out, seni_colour *c)
{
  out->type = VAR_COLOUR;

  out->value.i = (i32)(c->format);

  out->f32_array[0] = c->element[0];
  out->f32_array[1] = c->element[1];
  out->f32_array[2] = c->element[2];
  out->f32_array[3] = c->element[3];  
}

seni_env *env_construct()
{
  seni_env *e = (seni_env *)calloc(1, sizeof(seni_env));
  e->wl = wlut_allocate();

  declare_bindings(e->wl, e);
  
  return e;
}

void env_free(seni_env *e)
{
  wlut_free(e->wl);
  free(e);
}

void env_post_interpret_cleanup(seni_env *e)
{
  wlut_reset_words(e->wl);
}

// **************************************************
// Program
// **************************************************

char *opcode_name(seni_opcode opcode)
{
#define STR(x) #x
  
  switch(opcode) {
#define OPCODE(id,_) case id: return STR(id);
#include "seni_opcodes.h"
#undef OPCODE
  default:
    return "unknown opcode";
  }
#undef STR
}

seni_program *program_allocate(i32 code_max_size)
{
  seni_program *program = (seni_program *)calloc(1, sizeof(seni_program));

  program->code = (seni_bytecode *)calloc(code_max_size, sizeof(seni_bytecode));
  program->code_max_size = code_max_size;
  program->code_size = 0;
  program->opcode_offset = 0;

  return program;
}

void program_free(seni_program *program)
{
  free(program->code);
  free(program);
}

seni_program *program_compile(seni_env *env, i32 program_max_size, char *source)
{
  seni_node *ast = parser_parse(env->wl, source);

  seni_program *program = compile_program(ast, program_max_size, env->wl);
  
  parser_free_nodes(ast);

  return program;
}

i32 program_stop_location(seni_program *program)
{
  // the final opcode in the program will always be a STOP
  return program->code_size - 1;
}

char *memory_segment_name(seni_memory_segment_type segment)
{
  switch(segment) {
  case MEM_SEG_ARGUMENT:
    return "ARG";
  case MEM_SEG_LOCAL:
    return "LOCAL";
  case MEM_SEG_GLOBAL:
    return "GLOBAL";
  case MEM_SEG_CONSTANT:
    return "CONST";
  case MEM_SEG_VOID:
    return "VOID";
  }
  return "UNKNOWN";
}

void bytecode_pretty_print(i32 ip, seni_bytecode *b)
{
#define PPC_BUF_SIZE 200

  char buf[PPC_BUF_SIZE];
  int buf_len = 0;
  char *buf_start = &buf[0];

#define PRINT_BC buf_len += seni_sprintf
#define BUF_ARGS buf_start + buf_len, PPC_BUF_SIZE - buf_len

  buf[0] = 0;  
  
  if (b->op == LOAD || b->op == STORE || b->op == FLU_STORE) {

    char *seg_name = memory_segment_name((seni_memory_segment_type)b->arg0.value.i);

    if (b->op == LOAD || b->op == STORE) {
      PRINT_BC(BUF_ARGS, "%d\t%s\t\t%s\t\t", ip, opcode_name(b->op), seg_name);
    } else if (b->op == FLU_STORE) {
      PRINT_BC(BUF_ARGS, "%d\t%s\t%s\t\t", ip, opcode_name(b->op), seg_name);
    } 

    seni_value_in_use using = get_var_value_in_use(b->arg1.type);
    switch(using) {
    case USE_I:
      if (b->arg1.type == VAR_COLOUR) {
        i32 type = b->arg1.value.i;
        f32 *a = b->arg1.f32_array;
        PRINT_BC(BUF_ARGS, "colour: %d (%.2f, %.2f, %.2f, %.2f)", type, a[0], a[1], a[2], a[3]);
      } else {
        PRINT_BC(BUF_ARGS, "%d", b->arg1.value.i);
      }
      break;
    case USE_F:
      PRINT_BC(BUF_ARGS, "%.2f", b->arg1.value.f);
      break;
    case USE_L:
      PRINT_BC(BUF_ARGS, "%llu", (long long unsigned int)(b->arg1.value.l));
      break;
    case USE_V:
      if (b->arg1.type == VAR_VECTOR) {
        PRINT_BC(BUF_ARGS, "[..]len %d", vector_length(&(b->arg1)));
      } else {
        PRINT_BC(BUF_ARGS, "[..]");
      }
      break;
    default:
      PRINT_BC(BUF_ARGS, "unknown type");
    }
    
  } else if (b->op == JUMP_IF || b->op == JUMP) {
    PRINT_BC(BUF_ARGS, "%d\t%s\t\t", ip, opcode_name(b->op));
    if (b->arg0.value.i > 0) {
      PRINT_BC(BUF_ARGS, "+%d", b->arg0.value.i);
    } else if (b->arg0.value.i < 0) {
      PRINT_BC(BUF_ARGS, "%d", b->arg0.value.i);
    } else {
      PRINT_BC(BUF_ARGS, "WTF!");
    }
  } else if (b->op == NATIVE) {
    PRINT_BC(BUF_ARGS, "%d\t%s\t\t%d\t\t%d",
             ip, opcode_name(b->op), b->arg0.value.i, b->arg1.value.i);    
  } else if (b->op == PILE) {
    PRINT_BC(BUF_ARGS, "%d\t%s\t\t%d",
             ip, opcode_name(b->op), b->arg0.value.i);
  } else {
    PRINT_BC(BUF_ARGS, "%d\t%s", ip, opcode_name(b->op));
  }

  SENI_PRINT("%s", buf);
}

void program_pretty_print(seni_program *program)
{
  for (i32 i = 0; i < program->code_size; i++) {
    seni_bytecode *b = &(program->code[i]);
    bytecode_pretty_print(i, b);
  }
  SENI_PRINT("\n");
}


// **************************************************
// Virtual Machine
// **************************************************

seni_vm *vm_construct(i32 stack_size, i32 heap_size, i32 heap_min_size, i32 vertex_packet_num_vertices)
{
  seni_vm *vm = (seni_vm *)calloc(1, sizeof(seni_vm));

  vm->render_data = NULL;

  vm->stack_size = stack_size;
  vm->stack = (seni_var *)calloc(stack_size, sizeof(seni_var));

  vm->heap_size = heap_size;
  vm->heap_slab = (seni_var *)calloc(heap_size, sizeof(seni_var));

  vm->heap_avail_size_before_gc = heap_min_size;

  vm->matrix_stack = matrix_stack_construct();

  // prepare storage for vertices
  seni_render_data *render_data = render_data_construct(vertex_packet_num_vertices);
  vm->render_data = render_data;
  
  vm_reset(vm);

  return vm;
}

void vm_reset(seni_vm *vm)
{
  seni_var *var;
  i32 base_offset = 0;
  i32 i;

  vm->global = base_offset;
  base_offset += MEMORY_GLOBAL_SIZE;

  vm->ip = 0;
  
  vm->fp = base_offset;
  var = &(vm->stack[vm->fp]);
  var->type = VAR_INT;
  var->value.i = vm->fp;

  // add some offsets so that the memory after fp matches a standard format
  base_offset++;                // the caller's frame pointer
  base_offset++;                // the caller's ip
  base_offset++;                // the num_args of the called function

  vm->local = base_offset;
  base_offset += MEMORY_LOCAL_SIZE;
  vm->sp = base_offset;

  vm->heap_avail = NULL;
  for (i = 0; i < vm->heap_size; i++) {
    vm->heap_slab[i].next = NULL;
    vm->heap_slab[i].prev = NULL;
  }

  var = vm->heap_slab;
  for (i = 0; i < vm->heap_size; i++) {
    var[i].mark = false;
    DL_APPEND(vm->heap_avail, &(var[i]));
  }

  vm->heap_avail_size = vm->heap_size;

  matrix_stack_reset(vm->matrix_stack);

  render_data_free_render_packets(vm->render_data);
  add_render_packet(vm->render_data);
}


void vm_free_render_data(seni_vm *vm)
{
  render_data_free(vm->render_data);
  vm->render_data = NULL;
}

void vm_free(seni_vm *vm)
{
  vm_free_render_data(vm);
  matrix_stack_free(vm->matrix_stack);
  free(vm->stack);
  free(vm->heap_slab);
  free(vm);
}

void vm_pretty_print(seni_vm *vm, char* msg)
{
  SENI_LOG("%s\tvm: fp:%d sp:%d ip:%d local:%d",
             msg,
             vm->fp,
             vm->sp,
             vm->ip,
             vm->local);

  seni_var *fp = &(vm->stack[vm->fp]);
  i32 onStackFP = (fp + 0)->value.i;
  i32 onStackIP = (fp + 1)->value.i;
  i32 onStackNumArgs = (fp + 2)->value.i;
  SENI_LOG("\ton stack: fp:%d ip:%d numArgs:%d", onStackFP, onStackIP, onStackNumArgs);
}


// returns the next available seni_var that the calling code can write to
seni_var *stack_push(seni_vm *vm)
{
  seni_var *var = &(vm->stack[vm->sp]);
  vm->sp++;
  return var;
}

seni_var *stack_pop(seni_vm *vm)
{
  if (vm->sp == 0) {
    return NULL;
  }
  
  vm->sp--;
  return &(vm->stack[vm->sp]);
}

seni_var *stack_peek(seni_vm *vm)
{
  if (vm->sp == 0) {
    return NULL;
  }
  return &(vm->stack[vm->sp - 1]);
}

// [ ] <<- this is the VAR_VECTOR (value.v points to the first heap allocated seni_var)
//  |
//  v 
// [4] -> [7] -> [3] -> [5] -> NULL  <<- these are heap allocated seni_vars
//
void vector_construct(seni_var *head)
{
  // assuming that it's ok to wipe out head->value.v
  head->type = VAR_VECTOR;
  head->value.v = NULL;           // attach vec_rc to vec_head
}

i32 vector_length(seni_var *var)
{
  if (var->type != VAR_VECTOR) {
    return 0;
  }

  i32 len = 0;
  seni_var *v = var->value.v;

  while (v != NULL) {
    len++;
    v = v->next;
  }

  return len;
}

void vector_append_heap_var(seni_var *head, seni_var *val)
{
  // assuming that head is VAR_VECTOR and val is a seni_var from the heap
  DL_APPEND(head->value.v, val);
}

seni_var *vector_append_i32(seni_vm *vm, seni_var *head, i32 val)
{
  seni_var *v = var_get_from_heap(vm);
  if (v == NULL) {
    SENI_ERROR("vector_append_i32");
    return NULL;
  }
  
  v->type = VAR_INT;
  v->value.i = val;

  DL_APPEND(head->value.v, v);

  return v;
}

seni_var *vector_append_f32(seni_vm *vm, seni_var *head, f32 val)
{
  seni_var *v = var_get_from_heap(vm);
  if (v == NULL) {
    SENI_ERROR("vector_append_f32");
    return NULL;
  }
  
  v->type = VAR_FLOAT;
  v->value.f = val;

  DL_APPEND(head->value.v, v);

  return v;
}

seni_var *vector_append_u64(seni_vm *vm, seni_var *head, u64 val)
{
  seni_var *v = var_get_from_heap(vm);
  if (v == NULL) {
    SENI_ERROR("vector_append_u64");
    return NULL;
  }
  v->type = VAR_LONG;
  v->value.l = val;

  DL_APPEND(head->value.v, v);

  return v;
}

seni_var *vector_append_col(seni_vm *vm, seni_var *head, seni_colour *col)
{
  seni_var *v = var_get_from_heap(vm);
  if (v == NULL) {
    SENI_ERROR("vector_append_col");
    return NULL;
  }

  colour_as_var(v, col);

  DL_APPEND(head->value.v, v);

  return v;
}

seni_var *var_get_from_heap(seni_vm *vm)
{
  seni_var *head = vm->heap_avail;

  if (head != NULL) {
    DL_DELETE(vm->heap_avail, head);
  } else {
    SENI_ERROR("out of heap memory error");
    return NULL;
  }

  vm->heap_avail_size--;

  head->next = NULL;
  head->prev = NULL;

  head->value.i = 0;
  head->type = VAR_INT;         // just make sure that it isn't VAR_VECTOR from a previous allocation

  return head;
}

void var_copy(seni_var *dest, seni_var *src)
{
  if (dest == src) {
    return;
  }

  dest->type = src->type;
  dest->f32_array[0] = src->f32_array[0];
  dest->f32_array[1] = src->f32_array[1];
  dest->f32_array[2] = src->f32_array[2];
  dest->f32_array[3] = src->f32_array[3];

  seni_value_in_use using = get_var_value_in_use(src->type);
  
  if (using == USE_I) {
    dest->value.i = src->value.i;
  } else if (using == USE_F) {
    dest->value.f = src->value.f;
  } else if (using == USE_L) {
    dest->value.l = src->value.l;
  } else if (using == USE_V) {
    if (src->type == VAR_VECTOR) {
      dest->value.v = src->value.v;
    } else {
      SENI_ERROR("what the fuck?\n");
    }
  } else {
    SENI_ERROR("unknown seni_value_in_use for var_copy");
  }
}

#ifdef SENI_DEBUG_MODE

void vm_debug_info_reset(seni_vm *vm)
{
  vm->opcodes_executed = 0;
}

void vm_debug_info_print(seni_vm *vm)
{
  SENI_PRINT("*** vm_debug_info_print ***");
  SENI_PRINT("bytecodes executed:\t%llu", (long long unsigned int)(vm->opcodes_executed));
  SENI_PRINT("bytecode execution time:\t%.2f msec", vm->execution_time);
}

#endif
