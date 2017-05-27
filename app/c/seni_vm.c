#include "seni_vm.h"
#include "seni_config.h"

#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#include "utlist.h"

// global keyword variables
#define KEYWORD(val,_,name) i32 g_keyword_iname_##name = KEYWORD_START + val;
#include "seni_keywords.h"
#undef KEYWORD


#ifdef SENI_DEBUG_MODE

void vm_debug_info_reset(seni_virtual_machine *vm)
{
  vm->debug.get_from_heap_count = 0;
  vm->debug.return_to_heap_count = 0;
}

void vm_debug_info_print(seni_virtual_machine *vm)
{
  printf("*** vm_debug_info_print ***\n");
  printf("var_get_from_heap: %d\n", vm->debug.get_from_heap_count);
  printf("var_return_to_heap: %d\n", vm->debug.return_to_heap_count);
}

// record information during execution of bytecode
#define DEBUG_INFO_RESET(vm) vm_debug_info_reset(vm)
#define DEBUG_INFO_PRINT(vm) vm_debug_info_print(vm)
#define DEBUG_INFO_GET_FROM_HEAP(vm) vm->debug.get_from_heap_count++
#define DEBUG_INFO_RETURN_TO_HEAP(vm) vm->debug.return_to_heap_count++
#else
// do nothing
#define DEBUG_INFO_RESET(vm)
#define DEBUG_INFO_PRINT(vm)
#define DEBUG_INFO_GET_FROM_HEAP(vm)
#define DEBUG_INFO_RETURN_TO_HEAP(vm)
#endif

// returns the next available seni_var that the calling code can write to
seni_var *stack_push(seni_virtual_machine *vm)
{
  seni_var *var = &(vm->stack[vm->sp]);
  vm->sp++;
  return var;
}

seni_var *stack_pop(seni_virtual_machine *vm)
{
  if (vm->sp == 0) {
    return NULL;
  }
  
  vm->sp--;
  return &(vm->stack[vm->sp]);
}

seni_var *stack_peek(seni_virtual_machine *vm)
{
  if (vm->sp == 0) {
    return NULL;
  }
  return &(vm->stack[vm->sp - 1]);
}

seni_var *stack_peek2(seni_virtual_machine *vm)
{
  if (vm->sp < 2) {
    return NULL;
  }
  return &(vm->stack[vm->sp - 2]);
}

void pretty_print_vm_stack(seni_virtual_machine *vm, char *msg)
{
  printf("%s stack sp: %d\n", msg, vm->sp);
}


// **************************************************
// Program
// **************************************************

#define STR(x) #x
#define XSTR(x) STR(x)

char *opcode_name(seni_opcode opcode)
{
  char *names[] = {
#define OPCODE(name,_) STR(name),
#include "seni_opcodes.h"
#undef OPCODE
  };

  return names[opcode];
}

i32 opcode_offset[] = {
#define OPCODE(_,offset) offset,
#include "seni_opcodes.h"
#undef OPCODE
};

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

void vm_safe_var_move(seni_var *dest, seni_var *src);

seni_bytecode *program_emit_opcode(seni_program *program, seni_opcode op, seni_var *arg0, seni_var *arg1)
{
  if (program->code_size >= program->code_max_size) {
    SENI_ERROR("%s %d program has reached max size", __FILE__, __LINE__);
    return NULL;
  }
  
  seni_bytecode *b = &(program->code[program->code_size++]);
  b->op = op;
  vm_safe_var_move(&(b->arg0), arg0);
  vm_safe_var_move(&(b->arg1), arg1);

  program->opcode_offset += opcode_offset[op];

  return b;
}

// emits an <opcode, i32, i32> triplet
seni_bytecode *program_emit_opcode_i32(seni_program *program, seni_opcode op, i32 arg0, i32 arg1)
{
  if (program->code_size >= program->code_max_size) {
    SENI_ERROR("%s %d program has reached max size", __FILE__, __LINE__);
    return NULL;
  }
  
  seni_bytecode *b = &(program->code[program->code_size++]);
  b->op = op;
  i32_as_var(&(b->arg0), arg0);
  i32_as_var(&(b->arg1), arg1);

  program->opcode_offset += opcode_offset[op];

  return b;
}

// emits an <opcode, i32, f32> triplet
seni_bytecode *program_emit_opcode_i32_f32(seni_program *program, seni_opcode op, i32 arg0, f32 arg1)
{
  if (program->code_size >= program->code_max_size) {
    SENI_ERROR("%s %d program has reached max size", __FILE__, __LINE__);
    return NULL;
  }
  
  seni_bytecode *b = &(program->code[program->code_size++]);
  b->op = op;
  i32_as_var(&(b->arg0), arg0);
  f32_as_var(&(b->arg1), arg1);

  program->opcode_offset += opcode_offset[op];

  return b;
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
  if (b->op == PUSH || b->op == POP || b->op == DEC_RC || b->op == INC_RC) {
    printf("%d\t%s\t%s\t",
           ip,
           opcode_name(b->op),
           memory_segment_name((seni_memory_segment_type)b->arg0.value.i));

    seni_value_in_use using = get_value_in_use(b->arg1.type);
    switch(using) {
    case USE_I:
      printf("%d\n", b->arg1.value.i);
      break;
    case USE_F:
      printf("%.2f\n", b->arg1.value.f);
      break;
    case USE_V:
      if (b->arg1.type == VAR_VEC_HEAD) {
        printf("[..]len %d\n", var_vector_length(&(b->arg1)));
      } else {
        printf("[..]\n");
      }
      break;
    default:
      printf("unknown type\n");
    }
    
  } else if (b->op == JUMP_IF || b->op == JUMP) {
    printf("%d\t%s\t",
           ip,
           opcode_name(b->op));
    if (b->arg0.value.i > 0) {
      printf("+%d\n", b->arg0.value.i);
    } else if (b->arg0.value.i < 0) {
      printf("%d\n", b->arg0.value.i);
    } else {
      printf("WTF!\n");
    }
  } else if (b->op == NATIVE) {
    printf("%d\t%s\t%d\t%d\n",
           ip,
           opcode_name(b->op),
           b->arg0.value.i,
           b->arg1.value.i);    
  } else if (b->op == CALL_0 || b->op == CALL) {
    printf("%d\t%s\t%d\t%d\n",
           ip,
           opcode_name(b->op),
           b->arg0.value.i,
           b->arg1.value.i);
  } else {
    printf("%d\t%s\n", ip, opcode_name(b->op));
  }  
}

void program_pretty_print(seni_program *program)
{
  for (i32 i = 0; i < program->code_size; i++) {
    seni_bytecode *b = &(program->code[i]);
    bytecode_pretty_print(i, b);
  }
  printf("\n");
}

seni_vm_environment *vm_environment_construct()
{
  seni_vm_environment *e = (seni_vm_environment *)calloc(1, sizeof(seni_vm_environment));
  return e;
}

void vm_environment_free(seni_vm_environment *e)
{
  free(e);
}

// **************************************************
// Virtual Machine
// **************************************************

void vm_stack_set_value_i32(seni_virtual_machine *vm, i32 index, i32 value)
{
  seni_var *v = &(vm->stack[index]);
  v->type = VAR_INT;
  v->value.i = value;
}

seni_virtual_machine *virtual_machine_construct(i32 stack_size, i32 heap_size)
{
  i32 base_offset = 0;
  seni_virtual_machine *vm = (seni_virtual_machine *)calloc(1, sizeof(seni_virtual_machine));

  vm->stack = (seni_var *)calloc(stack_size, sizeof(seni_var));
  vm->stack_size = stack_size;

  vm->global = base_offset;
  base_offset += MEMORY_GLOBAL_SIZE;

  vm->ip = 0;
  
  vm->fp = base_offset;
  vm_stack_set_value_i32(vm, vm->fp, vm->fp);

  // add some offsets so that the memory after fp matches a standard format
  base_offset++;                // the caller's frame pointer
  base_offset++;                // the caller's ip
  base_offset++;                // the num_args of the called function

  vm->local = base_offset;
  base_offset += MEMORY_LOCAL_SIZE;
  vm->sp = base_offset;
  
  seni_var *var = (seni_var *)calloc(heap_size, sizeof(seni_var));
  vm->heap = var;
  vm->heap_list = NULL;
  vm->heap_size = heap_size;
  for (i32 i = 0; i < heap_size; i++) {
#ifdef SENI_DEBUG_MODE
    var[i].debug_id = i;
    var[i].debug_allocatable = true;
#endif
    var[i].allocated = false;
    DL_APPEND(vm->heap_list, &(var[i]));
  }

  return vm;
}

void virtual_machine_free(seni_virtual_machine *vm)
{
  free(vm->stack);
  free(vm->heap);
  free(vm);
}

void pretty_print_virtual_machine(seni_virtual_machine *vm, char* msg)
{
  printf("%s\tvm: fp:%d sp:%d ip:%d local:%d\n",
         msg,
         vm->fp,
         vm->sp,
         vm->ip,
         vm->local);

  seni_var *fp = &(vm->stack[vm->fp]);
  i32 onStackFP = (fp + 0)->value.i;
  i32 onStackIP = (fp + 1)->value.i;
  i32 onStackNumArgs = (fp + 2)->value.i;
  printf("\ton stack: fp:%d ip:%d numArgs:%d\n", onStackFP, onStackIP, onStackNumArgs);
}

void vm_vector_ref_count_decrement(seni_virtual_machine *vm, seni_var *vec_head);

seni_var *var_get_from_heap(seni_virtual_machine *vm)
{
  DEBUG_INFO_GET_FROM_HEAP(vm);

  seni_var *head = vm->heap_list;

  if (head != NULL) {
    DL_DELETE(vm->heap_list, head);
  } else {
    SENI_ERROR("no more vars in pool");
  }

  if (head->allocated == true) {
    SENI_ERROR("how did an already allocated seni_var get on the heap?");
    pretty_print_seni_var(head, "ERROR: var_get_from_heap");
  }

  head->allocated = true;

  head->next = NULL;
  head->prev = NULL;

  head->ref_count = 0;

  //pretty_print_seni_var(head, "getting");

  return head;
}

void var_return_to_heap(seni_virtual_machine *vm,  seni_var *var)
{
  if(var->allocated == false) {
    // in case of 2 bindings to the same variable
    // e.g. (define a [1 2]) (define b [3 4]) (setq a b)
    // a and b both point to [3 4]
    return;
  }

  DEBUG_INFO_RETURN_TO_HEAP(vm);

#ifdef SENI_DEBUG_MODE
  if (var->debug_allocatable == false) {
    SENI_ERROR("trying to return a seni_var to the pool that wasnt originally from the pool");
  }
#endif

  if (var->type == VAR_VEC_HEAD) {
    vm_vector_ref_count_decrement(vm, var);
  }
  
  if (var->type == VAR_VEC_RC) {
    if (var->value.v != NULL) {
      var_return_to_heap(vm, var->value.v);
    }
  }

  // the var is part of an allocated list
  if (var->next != NULL) {
    var_return_to_heap(vm, var->next);
  }

  var->allocated = false;
  DL_APPEND(vm->heap_list, var);
}
  
void vm_vector_ref_count_decrement(seni_virtual_machine *vm, seni_var *vec_head)
{
  seni_var *var_rc = vec_head->value.v;
  if (var_rc->type != VAR_VEC_RC) {
    SENI_ERROR("a VAR_VEC_HEAD that isn't pointing to a VAR_VEC_RC???");
  }

  var_rc->ref_count--;
      
  if (var_rc->ref_count == 0) {
    var_return_to_heap(vm, var_rc);
  }
}

void vm_vector_ref_count_increment(seni_virtual_machine *vm, seni_var *vec_head)
{
  seni_var *var_rc = vec_head->value.v;
  if (var_rc->type != VAR_VEC_RC) {
    SENI_ERROR("a VAR_VEC_HEAD that isn't pointing to a VAR_VEC_RC???");
  }
  var_rc->ref_count++;
}

void vm_safe_var_copy(seni_virtual_machine *vm, seni_var *dest, seni_var *src)
{
  if (dest == src) {
    return;
  }

  if (dest->type == VAR_VEC_HEAD) {
    vm_vector_ref_count_decrement(vm, dest);
  }
  
  dest->type = src->type;

  seni_value_in_use using = get_value_in_use(src->type);
  
  if (using == USE_I) {
    dest->value.i = src->value.i;
  } else if (using == USE_F) {
    dest->value.f = src->value.f;
  } else if (using == USE_N) {
    dest->value.n = src->value.n;
  } else if (using == USE_V) {
    if (src->type == VAR_VEC_HEAD) {
      dest->value.v = src->value.v;
      vm_vector_ref_count_increment(vm, dest);
    } else {
      printf("what the fuck?\n");
    }
  }
}

// like a seni_var_copy without any modifications to the ref count
void vm_safe_var_move(seni_var *dest, seni_var *src)
{
  if (dest == src) {
    return;
  }

  dest->type = src->type;

  seni_value_in_use using = get_value_in_use(src->type);
  
  if (using == USE_I) {
    dest->value.i = src->value.i;
  } else if (using == USE_F) {
    dest->value.f = src->value.f;
  } else if (using == USE_N) {
    dest->value.n = src->value.n;
  } else if (using == USE_V) {
    if (src->type == VAR_VEC_HEAD) {
      dest->value.v = src->value.v;
    } else {
      printf("what the fuck?\n");
    }
  }
}

// [ ] <<- this is the VAR_VEC_HEAD
//  |
// [4] -> [7] -> [3] -> [5] -> NULL  <<- these are seni_vars
//
seni_var *vm_append_to_vector(seni_virtual_machine *vm, seni_var *head, seni_var *val)
{
  // assuming that head is VAR_VEC_HEAD
  
  seni_var *child_value = var_get_from_heap(vm);
  if (child_value == NULL) {
    SENI_ERROR("cannot allocate child_value from pool");
    return NULL;
  }
  vm_safe_var_copy(vm, child_value, val);
  //pretty_print_seni_var(child_value, "child val");

  seni_var *vec_rc = head->value.v;
  
  DL_APPEND(vec_rc->value.v, child_value);

  return head;
}

void declare_vm_keyword(seni_word_lut *wlut, char *name)
{
  string_copy(&(wlut->keyword[wlut->keyword_count]), name);
  wlut->keyword_count++;

  if (wlut->keyword_count > MAX_KEYWORD_LOOKUPS) {
    SENI_ERROR("cannot declare keyword - wlut is full");
  }
}

void declare_vm_native(seni_word_lut *wlut, char *name, seni_vm_environment *e, native_function_ptr function_ptr)
{
  string_copy(&(wlut->native[wlut->native_count]), name);

  e->function_ptr[wlut->native_count] = function_ptr;

  wlut->native_count++;

  if (wlut->native_count > MAX_NATIVE_LOOKUPS) {
    SENI_ERROR("cannot declare native - wlut is full");
  }
}


// **************************************************
// Compiler
// **************************************************


void clear_local_mappings(seni_program *program)
{
  for (i32 i=0; i < MEMORY_LOCAL_SIZE; i++) {
    program->local_mappings[i] = -1;
  }
}

i32 add_local_mapping(seni_program *program, i32 wlut_value)
{
  for (i32 i=0; i < MEMORY_LOCAL_SIZE; i++) {
    if(program->local_mappings[i] == -1) {
      program->local_mappings[i] = wlut_value;
      return i;
    }
  }
  return -1;
}

i32 get_local_mapping(seni_program *program, i32 wlut_value)
{
  for (i32 i=0; i < MEMORY_LOCAL_SIZE; i++) {
    if(program->local_mappings[i] == wlut_value) {
      return i;
    }
  }

  return -1;
}

void clear_global_mappings(seni_program *program)
{
  for (i32 i=0; i < MEMORY_GLOBAL_SIZE; i++) {
    program->global_mappings[i] = -1;
  }
}

i32 add_global_mapping(seni_program *program, i32 wlut_value)
{
  for (i32 i=0; i < MEMORY_GLOBAL_SIZE; i++) {
    if(program->global_mappings[i] == -1) {
      program->global_mappings[i] = wlut_value;
      return i;
    }
  }
  return -1;
}

i32 get_global_mapping(seni_program *program, i32 wlut_value)
{
  for (i32 i=0; i < MEMORY_GLOBAL_SIZE; i++) {
    if(program->global_mappings[i] == wlut_value) {
      return i;
    }
  }

  return -1;
}

i32 get_argument_mapping(seni_fn_info *fn_info, i32 wlut_value)
{
  for (i32 i = 0; i < MAX_NUM_ARGUMENTS; i++) {
    if (fn_info->argument_offsets[i] == -1) {
      return -1;
    }
    if (fn_info->argument_offsets[i] == wlut_value) {
      return (i * 2) + 1;
    }
  }
  return -1;
}


seni_node *compile(seni_node *ast, seni_program *program, bool global_scope);

// a define statement in the global scope
seni_node *compile_global_define(seni_node *ast, seni_program *program)
{
  // define a 42
  // ^

  seni_node *name_node = safe_next(ast);
  // TODO: assert that name_node is NODE_NAME
  
  seni_node *value_node = safe_next(name_node);
  
  compile(value_node, program, false);

  i32 global_address = get_global_mapping(program, name_node->value.i);
  if (global_address == -1) {
    global_address = add_global_mapping(program, name_node->value.i);
  }

  program_emit_opcode_i32(program, POP, MEM_SEG_GLOBAL, global_address);

  return safe_next(value_node);
}

// single pair of name/value for the moment
seni_node *compile_define(seni_node *ast, seni_program *program)
{
  // define a 42
  // ^

  seni_node *name_node = safe_next(ast);
  // TODO: assert that name_node is NODE_NAME
  
  seni_node *value_node = safe_next(name_node);
  
  compile(value_node, program, false);

  i32 local_address = get_local_mapping(program, name_node->value.i);
  if (local_address == -1) {
    local_address = add_local_mapping(program, name_node->value.i);
  }

  program_emit_opcode_i32(program, POP, MEM_SEG_LOCAL, local_address);

  return safe_next(value_node);
}


void compile_if(seni_node *ast, seni_program *program)
{
  // if (> 200 100) 12 24
  // ^
  seni_node *if_node = safe_next(ast);
  seni_node *then_node = safe_next(if_node);
  seni_node *else_node = safe_next(then_node); // could be NULL

  compile(if_node, program, false);
  // insert jump to after the 'then' node if not true
  i32 addr_jump_then = program->code_size;
  seni_bytecode *bc_jump_then = program_emit_opcode_i32(program, JUMP_IF, 0, 0);

  compile(then_node, program, false);

  if (else_node) {
    // insert a bc_jump_else opcode
    i32 addr_jump_else = program->code_size;
    seni_bytecode *bc_jump_else = program_emit_opcode_i32(program, JUMP, 0, 0);

    bc_jump_then->arg0.value.i = program->code_size - addr_jump_then;

    compile(else_node, program, false);

    bc_jump_else->arg0.value.i = program->code_size - addr_jump_else;
  } else {
    bc_jump_then->arg0.value.i = program->code_size - addr_jump_then;
  }
}

void compile_loop(seni_node *ast, seni_program *program)
{
  // (loop (x from: 0 to: 5) (+ 42 38))
  //
  // 0       PUSH    CONST   0
  // 1       POP     LOCAL   0
  // 2       PUSH    LOCAL   0
  // 3       PUSH    CONST   5
  // 4       LT
  // 5       JUMP_IF +10
  // 6       PUSH    CONST   42
  // 7       PUSH    CONST   38
  // 8       ADD
  // 9       POP     VOID    0
  // 10      PUSH    LOCAL   0
  // 11      PUSH    CONST   1
  // 12      ADD
  // 13      POP     LOCAL   0
  // 14      JUMP    -12
  // 15      STOP
  
  seni_node *parameters_node = safe_next(ast);
  if (parameters_node->type != NODE_LIST) {
    SENI_ERROR("expected a list that defines loop parameters");
    return;
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
  compile(from_node, program, false);
  i32 looper_address = get_local_mapping(program, name_node->value.i);
  if (looper_address == -1) {
    looper_address = add_local_mapping(program, name_node->value.i);
  }
  program_emit_opcode_i32(program, POP, MEM_SEG_LOCAL, looper_address);

  // compare looping variable against exit condition
  // and jump if looping variable >= exit value
  i32 addr_loop_start = program->code_size;
  program_emit_opcode_i32(program, PUSH, MEM_SEG_LOCAL, looper_address);
  compile(to_node, program, false);
  program_emit_opcode_i32(program, LT, 0, 0);
  i32 addr_exit_check = program->code_size;
  seni_bytecode *bc_exit_check = program_emit_opcode_i32(program, JUMP_IF, 0, 0);


  i32 pre_body_opcode_offset = program->opcode_offset;

  // compile the body forms (woooaaaoohhh body form, body form for yoooouuuu)
  seni_node *body = safe_next(parameters_node);
  while (body != NULL) {
    compile(body, program, false);
    body = safe_next(body);
  }

  i32 post_body_opcode_offset = program->opcode_offset;
  i32 opcode_delta = post_body_opcode_offset - pre_body_opcode_offset;

  // pop off any values that the body might leave on the stack
  for(i32 i = 0;i < opcode_delta; i++) {
    program_emit_opcode_i32(program, POP, MEM_SEG_VOID, 0);
  }

  // increment the looping variable
  program_emit_opcode_i32(program, PUSH, MEM_SEG_LOCAL, looper_address);
  program_emit_opcode_i32_f32(program, PUSH, MEM_SEG_CONSTANT, 1.0f);
  program_emit_opcode_i32(program, ADD, 0, 0);
  program_emit_opcode_i32(program, POP, MEM_SEG_LOCAL, looper_address);

  // loop back to the comparison
  program_emit_opcode_i32(program, JUMP, -(program->code_size - addr_loop_start), 0);
  bc_exit_check->arg0.value.i = program->code_size - addr_exit_check;
}

seni_fn_info *get_local_fn_info(seni_node *node, seni_program *program)
{
  if (node->type != NODE_NAME) {
    return NULL;
  }

  i32 name = node->value.i;
  
  for (i32 i = 0; i < MAX_TOP_LEVEL_FUNCTIONS; i++) {
    if (program->fn_info[i].active == false) {
      return NULL;
    }
    if (program->fn_info[i].fn_name == name) {
      return &(program->fn_info[i]);
    }
  }
  return NULL;
}


i32 index_of_keyword(const char *keyword, seni_word_lut *wl)
{
  for (i32 i = 0; i < wl->keyword_count; i++) {
    if (strcmp(keyword, wl->keyword[i]) == 0) {
      return KEYWORD_START + i; // the keywords have KEYWORD_START added onto their index
    }
  }

  return -1;
}

bool is_seni_node_a_function(seni_node *ast, i32 fn_index)
{
  if (ast->type != NODE_LIST) {
    return false;
  }      

  seni_node *fn_keyword = ast->value.first_child;
  if (fn_keyword->type == NODE_NAME && fn_keyword->value.i == fn_index) {
    return true;
  }

  return false;
}

void register_top_level_fns(seni_node *ast, seni_program *program)
{
  i32 i;
  i32 num_fns = 0;
  
  // clear all fn data
  for (i = 0; i < MAX_TOP_LEVEL_FUNCTIONS; i++) {
    program->fn_info[i].active = false;
  }
  
  // search the wlut for the index of the 'fn' keyword
  i32 fn_index = index_of_keyword("fn", program->wl);

  // register top level fns
  while (ast != NULL) {

    if (ast->type != NODE_LIST) {
      ast = safe_next(ast);
      continue;
    }      

    seni_node *fn_keyword = ast->value.first_child;
    if (!(fn_keyword->type == NODE_NAME && fn_keyword->value.i == fn_index)) {
      ast = safe_next(ast);
      continue;
    }

    // (fn (add-up a: 0 b: 0) (+ a b))
    // get the name of the fn
    seni_node *name_and_params = safe_next(fn_keyword);
    if (name_and_params->type != NODE_LIST) {
      ast = safe_next(ast);
      continue;
    }

    seni_node *name = name_and_params->value.first_child;
    i32 name_value = name->value.i;

    // we have a named top-level fn declaration
    seni_fn_info *fn_info = &(program->fn_info[num_fns]);
    num_fns++;
    if (num_fns > MAX_TOP_LEVEL_FUNCTIONS) {
      SENI_ERROR("Script has more than %d top-level functions\n", MAX_TOP_LEVEL_FUNCTIONS);
      return;
    }

    fn_info->active = true;
    fn_info->index = num_fns - 1;
    fn_info->fn_name = name_value;

    // these will be filled in by compile_fn:
    fn_info->num_args = 0;
    for (i = 0; i < MAX_NUM_ARGUMENTS; i++) {
      fn_info->argument_offsets[i] = -1;
    }

    ast = safe_next(ast);
  }
}

/*
  invoking code will first CALL into the arg_address to setup the default values for all args
  the fn code will then return back to the invoking code
  invoking code will then overwrite specific data in arg memory
  invoking code will then CALL into the body_address
*/
void compile_fn(seni_node *ast, seni_program *program)
{
  // fn (adder a: 0 b: 0) (+ a b)

  clear_local_mappings(program);

  // (adder a: 0 b: 0)
  seni_node *signature = safe_next(ast);

  seni_node *fn_name = signature->value.first_child;
  seni_fn_info *fn_info = get_local_fn_info(fn_name, program);
  if (fn_info == NULL) {
    SENI_ERROR("Unable to find fn_info for function %d", fn_name->value.i);
    return;
  }

  program->current_fn_info = fn_info;

  // -------------
  // the arguments
  // -------------
  
  fn_info->arg_address = program->code_size;
  seni_node *args = safe_next(fn_name); // pairs of label/value declarations
  i32 num_args = 0;
  i32 counter = 0;
  i32 argument_offsets_counter = 0;
  while (args != NULL) {
    seni_node *label = args;
    seni_node *value = safe_next(label);

    // get_argument_mapping
    fn_info->argument_offsets[argument_offsets_counter++] = label->value.i;

    // push pairs of label+value values onto the args stack
    program_emit_opcode_i32(program, PUSH, MEM_SEG_CONSTANT, label->value.i);
    program_emit_opcode_i32(program, POP, MEM_SEG_ARGUMENT, counter++);

    compile(value, program, false);
    program_emit_opcode_i32(program, POP, MEM_SEG_ARGUMENT, counter++);

    num_args++;
    args = safe_next(value);
  }

  fn_info->num_args = num_args;

  program_emit_opcode_i32(program, RET_0, 0, 0);

  // --------
  // the body
  // --------

  fn_info->body_address = program->code_size;

  // (+ a b)
  seni_node *body = safe_next(signature);

  while (body != NULL) {
    compile(body, program, false);
    body = safe_next(body);
  }

  // Don't need any POP, MEM_SEG_VOID instructions as the RET will
  // pop the frame and blow the stack

  program_emit_opcode_i32(program, RET, 0, 0);

  program->current_fn_info = NULL;
}

// compiles everything after the current ast point
void compile_rest(seni_node *ast, seni_program *program)
{
  ast = safe_next(ast);
  while (ast) {
    ast = compile(ast, program, false);
  }
}

void compile_fn_invocation(seni_node *ast, seni_program *program, seni_fn_info *fn_info, bool global_scope)
{
  // ast == adder a: 10 b: 20

  // prepare the MEM_SEG_ARGUMENT with default values
  program_emit_opcode_i32(program, CALL, fn_info->arg_address, fn_info->num_args);

  // overwrite the default arguments with the actual arguments given by the fn invocation
  seni_node *args = safe_next(ast); // pairs of label/value declarations
  while (args != NULL) {
    seni_node *label = args;
    seni_node *value = safe_next(label);

    // find the index within MEM_SEG_ARGUMENT that holds the default value for label
    i32 data_index = get_argument_mapping(fn_info, label->value.i);
    if (data_index != -1) {
      // push value
      compile(value, program, global_scope);
      program_emit_opcode_i32(program, DEC_RC, MEM_SEG_ARGUMENT, data_index);
      program_emit_opcode_i32(program, POP, MEM_SEG_ARGUMENT, data_index);

      if (value->type != NODE_VECTOR) {
        // not an explicitly declared vector so increment it's rc
        program_emit_opcode_i32(program, INC_RC, MEM_SEG_ARGUMENT, data_index);

        // explicitly declared vectors will have an rc of 1, when the function
        // returns this will be decremented and they will be returned to the heap
      }
    }

    args = safe_next(value);
  }
  
  // call the body of the function
  program_emit_opcode_i32(program, CALL_0, fn_info->body_address, fn_info->num_args);

}

void compile_vector(seni_node *ast, seni_program *program)
{
  // pushing from the VOID means creating a new, empty vector
  program_emit_opcode_i32(program, PUSH, MEM_SEG_VOID, 0);

  for (seni_node *node = ast->value.first_child; node != NULL; node = safe_next(node)) {
    compile(node, program, false);
    program_emit_opcode_i32(program, APPEND, 0, 0);
  }
}

seni_node *compile(seni_node *ast, seni_program *program, bool global_scope)
{
  seni_node *n;

  if (ast->type == NODE_LIST) {
    n = ast->value.first_child;

    seni_fn_info *fn_info = get_local_fn_info(n, program);
    if (fn_info) {
      compile_fn_invocation(n, program, fn_info, global_scope);
    } else {
      compile(n, program, global_scope);
    }
    return safe_next(ast);
  }
  if (ast->type == NODE_FLOAT) {
    program_emit_opcode_i32_f32(program, PUSH, MEM_SEG_CONSTANT, ast->value.f);
    return safe_next(ast);
  }
  if (ast->type == NODE_INT) {
    program_emit_opcode_i32(program, PUSH, MEM_SEG_CONSTANT, ast->value.i);
    return safe_next(ast);
  }
  if (ast->type == NODE_VECTOR) {
    compile_vector(ast, program);
    return safe_next(ast);
  }
  if (ast->type == NODE_NAME) {

    i32 iname = ast->value.i;
    
    if (iname >= WORD_START && iname < WORD_START + MAX_WORD_LOOKUPS) { // a user defined name
      
      i32 local_mapping = get_local_mapping(program, iname);
      if (local_mapping != -1) {
        program_emit_opcode_i32(program, PUSH, MEM_SEG_LOCAL, local_mapping);
        return safe_next(ast);
      }

      // check arguments if we're in a function
      if (program->current_fn_info) {
        i32 argument_mapping = get_argument_mapping(program->current_fn_info, iname);
        if (argument_mapping != -1) {
          program_emit_opcode_i32(program, PUSH, MEM_SEG_ARGUMENT, argument_mapping);
          return safe_next(ast);
        }
      }

      i32 global_mapping = get_global_mapping(program, iname);
      if (global_mapping != -1) {
        program_emit_opcode_i32(program, PUSH, MEM_SEG_GLOBAL, global_mapping);
        return safe_next(ast);
      }
    } else if (iname >= KEYWORD_START && iname < KEYWORD_START + MAX_KEYWORD_LOOKUPS) {

      if (iname == g_keyword_iname_define) {
        if (global_scope) {
          return compile_global_define(ast, program);
        } else {
          return compile_define(ast, program);
        }        
      } else if (iname == g_keyword_iname_if) {
        compile_if(ast, program);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_loop) {
        compile_loop(ast, program);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_fn) {
        compile_fn(ast, program);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_plus) {
        compile_rest(ast, program);
        program_emit_opcode_i32(program, ADD, 0, 0);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_minus) {
        // TODO: differentiate between neg and sub?
        compile_rest(ast, program);
        program_emit_opcode_i32(program, SUB, 0, 0);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_mult) {
        compile_rest(ast, program);
        program_emit_opcode_i32(program, MUL, 0, 0);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_divide) {
        compile_rest(ast, program);
        program_emit_opcode_i32(program, DIV, 0, 0);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_equal) {
        compile_rest(ast, program);
        program_emit_opcode_i32(program, EQ, 0, 0);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_lt) {
        compile_rest(ast, program);
        program_emit_opcode_i32(program, LT, 0, 0);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_gt) {
        compile_rest(ast, program);
        program_emit_opcode_i32(program, GT, 0, 0);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_and) {
        compile_rest(ast, program);
        program_emit_opcode_i32(program, AND, 0, 0);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_or) {
        compile_rest(ast, program);
        program_emit_opcode_i32(program, OR, 0, 0);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_not) {
        compile_rest(ast, program);
        program_emit_opcode_i32(program, NOT, 0, 0);
        return safe_next(ast);
      } else {
        // look up the name as a local variable?
        printf("cannot find %d\n", iname);
        return safe_next(ast);        
      }

    } else if ( iname >= NATIVE_START && iname < NATIVE_START + MAX_NATIVE_LOOKUPS){
      // NATIVE

      // note: how to count the stack delta? how many pop voids are required?
      i32 num_args = 0;
      seni_node *args = safe_next(ast); // pairs of label/value declarations
      while (args != NULL) {
        seni_node *label = args;
        seni_node *value = safe_next(label);

        program_emit_opcode_i32(program, PUSH, MEM_SEG_CONSTANT, label->value.i);
        compile(value, program, global_scope);

        num_args++;
        args = safe_next(value);
      }
      
      program_emit_opcode_i32(program, NATIVE, iname, num_args);

      // modify opcode_offset according to how many args were given
      program->opcode_offset -= (num_args * 2) - 1;
      
      
      return safe_next(ast);
    }
  }

  return safe_next(ast);
}

// compiles the ast into bytecode for a stack based VM
//
void compiler_compile(seni_node *ast, seni_program *program)
{
  clear_global_mappings(program);
  clear_local_mappings(program);
  program->current_fn_info = NULL;
  
  register_top_level_fns(ast, program);

  i32 fn_index = index_of_keyword("fn", program->wl);
  seni_bytecode *start = program_emit_opcode_i32(program, JUMP, 0, 0);
  bool found_start = false;
  
  seni_node *n = ast;
  while (n != NULL) {
    // ghetto jump to start
    if (found_start == false && is_seni_node_a_function(n, fn_index) == false) {
      start->arg0.type = VAR_INT;
      start->arg0.value.i = program->code_size;
      found_start = true;
    }
    n = compile(n, program, true);
  }

  program_emit_opcode_i32(program, STOP, 0, 0);
}

// **************************************************
// VM bytecode interpreter
// **************************************************

// executes a program on a vm 
void vm_interpret(seni_virtual_machine *vm, seni_program *program)
{
  bool b1, b2;
  f32 f1, f2;
  seni_memory_segment_type memory_segment_type;
  seni_var *src, *dest, *tmp;

  register seni_bytecode *bc = NULL;
  register seni_var *v = NULL;
  register i32 ip = vm->ip;
  register i32 sp = vm->sp;
  register seni_var *stack_d = &(vm->stack[sp]);

  i32 new_fp;
  i32 num_args;
  i32 iname;
  i32 i;

#define STACK_POP stack_d--; sp--; v = stack_d
#define STACK_PUSH v = stack_d; stack_d++; sp++

  DEBUG_INFO_RESET(vm);

  for (;;) {
    bc = &(program->code[ip++]);
    
    switch(bc->op) {
    case PUSH:
      STACK_PUSH;

      memory_segment_type = (seni_memory_segment_type)bc->arg0.value.i;
      if (memory_segment_type == MEM_SEG_CONSTANT) {
        vm_safe_var_move(v, &(bc->arg1));
      } else if (memory_segment_type == MEM_SEG_ARGUMENT) {
        src = &(vm->stack[vm->fp - bc->arg1.value.i - 1]);
        vm_safe_var_move(v, src);
      } else if (memory_segment_type == MEM_SEG_LOCAL) {
        src = &(vm->stack[vm->local + bc->arg1.value.i]);
        vm_safe_var_move(v, src);
      } else if (memory_segment_type == MEM_SEG_GLOBAL) {
        src = &(vm->stack[vm->global + bc->arg1.value.i]);
        vm_safe_var_move(v, src);
      } else if (memory_segment_type == MEM_SEG_VOID) {
        // pushing from the void. i.e. create this object

        // temp: for the moment just assume that any PUSH VOID
        // means creating a new vector object.

        // also note that the VAR_VEC_HEAD is a seni_var from the stack
        // so it should never be sent to the vm->heap_list
        src = v;
        src->type = VAR_VEC_HEAD;
        
        tmp = var_get_from_heap(vm);
        if (tmp == NULL) {
          SENI_ERROR("unable to get a var from the pool");
          return;
        }
        tmp->type = VAR_VEC_RC;
        tmp->ref_count = 1;
        tmp->value.v = NULL;

        src->value.v = tmp;
        
        vm_safe_var_copy(vm, v, src);
      } else {
        SENI_ERROR("PUSH: unknown memory segment type %d", bc->arg0.value.i);
      }
      break;

    case POP:
      STACK_POP;

      memory_segment_type = (seni_memory_segment_type)bc->arg0.value.i;
      if (memory_segment_type == MEM_SEG_ARGUMENT) {
        dest = &(vm->stack[vm->fp - bc->arg1.value.i - 1]);
        //printf("POP ARG stack value %d is %d\n", bc->arg1.value.i, vm->fp - bc->arg1.value.i - 1);
        // check the current value of dest, 
        vm_safe_var_move(dest, v);
      } else if (memory_segment_type == MEM_SEG_LOCAL) {
        dest = &(vm->stack[vm->local + bc->arg1.value.i]);
        vm_safe_var_move(dest, v);
      } else if (memory_segment_type == MEM_SEG_GLOBAL) {
        dest = &(vm->stack[vm->global + bc->arg1.value.i]);
        vm_safe_var_move(dest, v);
      } else if (memory_segment_type == MEM_SEG_VOID) {
        // normally pop from the stack and lose the value
        // but if it's a vector then decrement its ref count
        if (v->type == VAR_VEC_HEAD) {
          vm_vector_ref_count_decrement(vm, v);
        }
      } else {
        SENI_ERROR("POP: unknown memory segment type %d", bc->arg0.value.i);
      }
      break;

    case DEC_RC:
      // the var referenced by the bytecode is a default value for a function argument
      // it's going to be overwritten by a parameter that was specified by the calling
      // code.
      // We'll need to decrement it's ref count
      //
      memory_segment_type = (seni_memory_segment_type)bc->arg0.value.i;
      if (memory_segment_type == MEM_SEG_ARGUMENT) {
        dest = &(vm->stack[vm->fp - bc->arg1.value.i - 1]);
        if (dest->type == VAR_VEC_HEAD) {
          vm_vector_ref_count_decrement(vm, dest);
        }
      } else {
        SENI_ERROR("DEC_RC: unknown memory segment type %d", bc->arg0.value.i);
      }
      break;

    case INC_RC:
      //
      memory_segment_type = (seni_memory_segment_type)bc->arg0.value.i;
      if (memory_segment_type == MEM_SEG_ARGUMENT) {
        dest = &(vm->stack[vm->fp - bc->arg1.value.i - 1]);
        if (dest->type == VAR_VEC_HEAD) {
          vm_vector_ref_count_increment(vm, dest);
        }
      } else {
        SENI_ERROR("DEC_RC: unknown memory segment type %d", bc->arg0.value.i);
      }
      break;

    case JUMP:
      ip--;
      ip += bc->arg0.value.i;
      break;

    case JUMP_IF:
      STACK_POP;

      // jump if the top of the stack is false
      if (v->value.i == 0) {
        ip--;
        ip += bc->arg0.value.i;
      }
      break;

    case CALL:
      num_args = bc->arg1.value.i;

      // make room for the labelled arguments
      for (i32 i = 0; i < num_args * 2; i++) {
        STACK_PUSH;
      }
      
      new_fp = sp;

      // push the caller's fp
      STACK_PUSH;
      v->type = VAR_INT;
      v->value.i = vm->fp;

      // push ip
      STACK_PUSH;
      v->type = VAR_INT;
      v->value.i = ip;

      // push num_args
      STACK_PUSH;
      v->type = VAR_INT;
      v->value.i = num_args;

      vm->ip = bc->arg0.value.i;
      vm->fp = new_fp;
      vm->local = sp;

      // clear ref count on the new local memory
      // so that during pop we can correctly return memory to the heap
      for (i = 0; i < MEMORY_LOCAL_SIZE; i++) {
        vm->stack[sp].ref_count = 0;
        vm->stack[sp].type = VAR_INT; // anything but VAR_VEC_HEAD
        sp++;
      }

      stack_d = &(vm->stack[sp]);
      ip = vm->ip;
      
      vm->sp = sp;
      break;

    case CALL_0:
      // like CALL but keep the existing frame and just update the ip and return ip
      
      // set the correct return ip
      vm->stack[vm->fp + 1].value.i = ip;

      // leap to a location
      ip = bc->arg0.value.i;
      vm->ip = ip;
      break;

    case RET_0:
      // leap to the return ip
      vm->ip = vm->stack[vm->fp + 1].value.i;
      ip = vm->ip;
      break;
      
    case RET:
      // pop the frame
      //

      // grab whatever was the last value on the soon to be popped frame
      src = &(vm->stack[sp - 1]);
      if (src->type == VAR_VEC_HEAD) {
        vm_vector_ref_count_increment(vm, src);
      }

      num_args = vm->stack[vm->fp + 2].value.i;

      // decrement ref count on any locally defined vectors
      for (i = 0; i < MEMORY_LOCAL_SIZE; i++) {
        tmp = &(vm->stack[vm->local + i]);
        if (tmp->type == VAR_VEC_HEAD) {
          vm_vector_ref_count_decrement(vm, tmp);
        }
      }

      for (i = 0; i < num_args; i++) {
        tmp = &(vm->stack[vm->fp - ((i+1) * 2)]);
        // printf("fp %d RET arg decrement %d on stack: %d\n", vm->fp, num_args, vm->fp - ((i+1) * 2));
        // pretty_print_seni_var(tmp, "tmp");
        if (tmp->type == VAR_VEC_HEAD) {
          // printf("RET arg decrement\n");
          vm_vector_ref_count_decrement(vm, tmp);
        }
      }

      // update vm
      vm->sp = vm->fp - (num_args * 2);
      vm->ip = vm->stack[vm->fp + 1].value.i;
      vm->fp = vm->stack[vm->fp].value.i;
      vm->local = vm->fp + 3;

      // sync registers with vm
      ip = vm->ip;
      sp = vm->sp;
      stack_d = &(vm->stack[sp]);

      // copy the previous frame's top stack value onto the current frame's stack
      STACK_PUSH;
      vm_safe_var_move(v, src);

      break;

    case NATIVE:
      iname = bc->arg0.value.i - NATIVE_START;
      num_args = bc->arg1.value.i;

      // sync vm with registers
      vm->sp = sp;

      native_function_ptr native_func = program->vm_environment->function_ptr[iname];
      native_func(vm, num_args);

      // sync registers with vm
      sp = vm->sp;
      stack_d = &(vm->stack[sp]);
      
      break;

    case APPEND:
      // pops top two values: a value and a vector
      // appends the value onto the vector
      STACK_POP;
      src = v;                      // the seni_var to append onto the vector

      STACK_POP;
      // v is the vector
      if (v->type != VAR_VEC_HEAD) {
        printf("APPEND expects the 2nd item on the stack to be a vector\n");
        return;
      }

      vm_append_to_vector(vm, v, src); // note: this uses a copy, should it be a move instead?

      STACK_PUSH;
      break;

    case ADD:
      STACK_POP;
      f2 = v->value.f;
      
      STACK_POP;
      f1 = v->value.f;

      STACK_PUSH;
      v->value.f = f1 + f2;
      break;

    case SUB:
      STACK_POP;
      f2 = v->value.f;
      STACK_POP;
      f1 = v->value.f;

      STACK_PUSH;
      v->value.f = f1 - f2;

      break;

    case MUL:
      STACK_POP;
      f2 = v->value.f;
      
      STACK_POP;
      f1 = v->value.f;

      STACK_PUSH;
      v->value.f = f1 * f2;
      break;

    case DIV:
      STACK_POP;
      f2 = v->value.f;
      
      STACK_POP;
      f1 = v->value.f;

      STACK_PUSH;
      v->value.f = f1 / f2;
      break;

    case EQ:
      STACK_POP;
      f2 = v->value.f;
      STACK_POP;
      f1 = v->value.f;

      STACK_PUSH;
      v->type = VAR_BOOLEAN;
      v->value.i = f1 == f2;
      break;

    case GT:
      STACK_POP;
      f2 = v->value.f;
      STACK_POP;
      f1 = v->value.f;

      STACK_PUSH;
      v->type = VAR_BOOLEAN;
      v->value.i = f1 > f2;
      break;

    case LT:
      STACK_POP;
      f2 = v->value.f;
      STACK_POP;
      f1 = v->value.f;

      STACK_PUSH;
      v->value.i = f1 < f2;
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
      vm->sp = sp;
      //DEBUG_INFO_PRINT(vm);
      return;
    default:
      SENI_ERROR("Unhandled opcode: %s\n", opcode_name(bc->op));
    }
  }
}
