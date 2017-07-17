#include "seni_vm_interpreter.h"
#include "seni_timing.h"

#include "utlist.h"

// like a seni_var_copy without any modifications to the ref count
void var_move(seni_var *dest, seni_var *src)
{
  if (dest == src) {
    return;
  }

  dest->type = src->type;
  dest->f32_array[0] = src->f32_array[0];
  dest->f32_array[1] = src->f32_array[1];
  dest->f32_array[2] = src->f32_array[2];
  dest->f32_array[3] = src->f32_array[3];

  seni_value_in_use using = get_value_in_use(src->type);
  
  if (using == USE_I) {
    dest->value.i = src->value.i;
  } else if (using == USE_F) {
    dest->value.f = src->value.f;
  } else if (using == USE_L) {
    dest->value.l = src->value.l;
  } else if (using == USE_V) {
    if (src->type == VAR_VEC_HEAD) {
      dest->value.v = src->value.v;
    } else {
      SENI_ERROR("what the fuck?\n");
    }
  } else {
    SENI_ERROR("unknown seni_value_in_use for var_move");
  }
}

bool var_copy(seni_vm *vm, seni_var *dest, seni_var *src)
{
  if (dest == src) {
    return true;
  }

  bool res = vector_ref_count_decrement(vm, dest);
  if (res == false) {
    SENI_ERROR("var_copy - vector_ref_count_decrement failed");
    return false;
  }

  dest->type = src->type;
  dest->f32_array[0] = src->f32_array[0];
  dest->f32_array[1] = src->f32_array[1];
  dest->f32_array[2] = src->f32_array[2];
  dest->f32_array[3] = src->f32_array[3];

  seni_value_in_use using = get_value_in_use(src->type);
  
  if (using == USE_I) {
    dest->value.i = src->value.i;
  } else if (using == USE_F) {
    dest->value.f = src->value.f;
  } else if (using == USE_L) {
    dest->value.l = src->value.l;
  } else if (using == USE_V) {
    if (src->type == VAR_VEC_HEAD) {
      dest->value.v = src->value.v;
      vector_ref_count_increment(vm, src);
    } else {
      SENI_ERROR("what the fuck?\n");
    }
  } else {
    SENI_ERROR("unknown seni_value_in_use for var_copy");
  }

  return true;
}

// copying the src onto a var that we're not using (e.g. the top of a stack)
// only the reference counts of the src should be updated
bool var_copy_onto_junk(seni_vm *vm, seni_var *dest, seni_var *src)
{
  if (dest == src) {
    return true;
  }

  dest->type = src->type;
  dest->f32_array[0] = src->f32_array[0];
  dest->f32_array[1] = src->f32_array[1];
  dest->f32_array[2] = src->f32_array[2];
  dest->f32_array[3] = src->f32_array[3];

  seni_value_in_use using = get_value_in_use(src->type);
  
  if (using == USE_I) {
    dest->value.i = src->value.i;
  } else if (using == USE_F) {
    dest->value.f = src->value.f;
  } else if (using == USE_L) {
    dest->value.l = src->value.l;
  } else if (using == USE_V) {
    if (src->type == VAR_VEC_HEAD) {
      dest->value.v = src->value.v;
      vector_ref_count_increment(vm, src);
    } else {
      SENI_ERROR("what the fuck?\n");
    }
  } else {
    SENI_ERROR("unknown seni_value_in_use for var_copy_onto_junk");
  }

  return true;
}

seni_var *var_get_from_heap(seni_vm *vm)
{
  seni_var *head = vm->heap_avail;

  if (head != NULL) {
    DEBUG_INFO_GET_FROM_HEAP(vm);
    DL_DELETE(vm->heap_avail, head);
  } else {
    SENI_ERROR("no more vars in pool");
    return NULL;
  }

  if (head->allocated) {
    SENI_ERROR("how did an already allocated seni_var get on the heap?");
    pretty_print_seni_var(head, "ERROR: var_get_from_heap");
    return NULL;
  }

  head->allocated = true;

  head->next = NULL;
  head->prev = NULL;

  head->value.i = 0;
  head->type = VAR_INT;         // just make sure that it isn't VAR_VEC_HEAD from a previous allocation

  //pretty_print_seni_var(head, "getting");

  return head;
}

void var_return_to_heap(seni_vm *vm,  seni_var *var)
{
  if(var->allocated == false) {
    // in case of 2 bindings to the same variable
    // e.g. (define a [1 2]) (define b [3 4]) (setq a b)
    // a and b both point to [3 4]
    return;
  }

  DEBUG_INFO_RETURN_TO_HEAP(vm);

  bool res = vector_ref_count_decrement(vm, var);
  if(res == false) {
    SENI_ERROR("var_return_to_heap");
  }

  // the var is part of an allocated list
  if (var->next != NULL) {
    var_return_to_heap(vm, var->next);
  }

  var->allocated = false;
  DL_APPEND(vm->heap_avail, var);
}
  
bool vector_ref_count_decrement(seni_vm *vm, seni_var *vec_head)
{
  if (vec_head->type != VAR_VEC_HEAD) {
    return true;
  }
  
  seni_var *var_rc = vec_head->value.v;
  if (var_rc->type != VAR_VEC_RC) {
    SENI_ERROR("a VAR_VEC_HEAD that isn't pointing to a VAR_VEC_RC???");
    pretty_print_seni_var(vec_head, "vector_ref_count_decrement called on this???");
    return false;
  }

  var_rc->value.ref_count--;

  // decrement the ref counts of any nested vectors
  seni_var *element = var_rc->next;
  while (element != NULL) {
    vector_ref_count_decrement(vm, element);
    element = element->next;
  }

  // pretty_print_seni_var(vec_head, "dec");
      
  if (var_rc->value.ref_count == 0) {
    var_return_to_heap(vm, var_rc);
  } else if (var_rc->value.ref_count < 0) {
    SENI_ERROR("vector_ref_count_decrement: ref_count is %d", var_rc->value.ref_count);
    return false;
  }

  return true;
}

void vector_ref_count_increment(seni_vm *vm, seni_var *vec_head)
{
  if (vec_head->type != VAR_VEC_HEAD) {
    return;
  }
  seni_var *var_rc = vec_head->value.v;
  if (var_rc->type != VAR_VEC_RC) {
    SENI_ERROR("a VAR_VEC_HEAD that isn't pointing to a VAR_VEC_RC %d???", vm->sp);
  }
  
  var_rc->value.ref_count++;

  // pretty_print_seni_var(vec_head, "inc");
}

// [ ] <<- this is the VAR_VEC_HEAD (value.v points to VAR_VEC_RC)
//  |
// [ ] <<- this is the VAR_VEC_RC (value.ref_count is used)
//  |
//  v  <<= the VAR_VEC_RC's next pointer points to the contents of the vector
// [4] -> [7] -> [3] -> [5] -> NULL  <<- these are seni_vars
//
void vector_construct(seni_vm *vm, seni_var *head)
{
  seni_var *rc = var_get_from_heap(vm);    // get a vec_rc
  if (rc == NULL) {
    SENI_ERROR("vector_construct");
    return;
  }
  
  rc->type = VAR_VEC_RC;
  rc->value.ref_count = 1;

  // assuming that it's ok to wipe out head->value.v
  head->type = VAR_VEC_HEAD;
  head->value.v = NULL;           // attach vec_rc to vec_head
  DL_APPEND(head->value.v, rc);
}

bool append_to_vector(seni_vm *vm, seni_var *head, seni_var *val)
{
  // assuming that head is VAR_VEC_HEAD
  
  seni_var *child_value = var_get_from_heap(vm);
  if (child_value == NULL) {
    SENI_ERROR("cannot allocate child_value from pool");
    return false;
  }

  bool res = var_copy(vm, child_value, val);
  if (res == false) {
    SENI_ERROR("var_copy failed in append_to_vector");
    return false;
  }
  
  DL_APPEND(head->value.v, child_value);
  return true;
}

seni_var *append_to_vector_i32(seni_vm *vm, seni_var *head, i32 val)
{
  seni_var *v = var_get_from_heap(vm);
  if (v == NULL) {
    SENI_ERROR("append_to_vector_i32");
    return NULL;
  }
  
  v->type = VAR_INT;
  v->value.i = val;

  DL_APPEND(head->value.v, v);

  return v;
}

seni_var *append_to_vector_f32(seni_vm *vm, seni_var *head, f32 val)
{
  seni_var *v = var_get_from_heap(vm);
  if (v == NULL) {
    SENI_ERROR("append_to_vector_f32");
    return NULL;
  }
  
  v->type = VAR_FLOAT;
  v->value.f = val;

  DL_APPEND(head->value.v, v);

  return v;
}

seni_var *append_to_vector_u64(seni_vm *vm, seni_var *head, u64 val)
{
  seni_var *v = var_get_from_heap(vm);
  if (v == NULL) {
    SENI_ERROR("append_to_vector_u64");
    return NULL;
  }
  v->type = VAR_LONG;
  v->value.l = val;

  DL_APPEND(head->value.v, v);

  return v;
}

seni_var *append_to_vector_col(seni_vm *vm, seni_var *head, seni_colour *col)
{
  seni_var *v = var_get_from_heap(vm);
  if (v == NULL) {
    SENI_ERROR("append_to_vector_col");
    return NULL;
  }

  colour_as_var(v, col);

  DL_APPEND(head->value.v, v);

  return v;
}

// **************************************************
// VM bytecode interpreter
// **************************************************


seni_var *arg_memory_from_iname(seni_fn_info *fn_info, i32 iname, seni_var *args)
{
  // args is the point on the stack that contains the args for the function about to be called

  i32 num_args = fn_info->num_args;

  // search the ARG memory for iname
  for (i32 i = 0; i < num_args; i++) {
    if (args->value.i == iname) {
      args--; // move from the label onto the arg's default value
      return args;
    }
    args--; // the value of the arg
    args--; // the next label's iname
  }

  return NULL;
}

// invokes a no-arg function during execution of a program (some native functions use this)
// push a frame onto the vm's stack, and invoke vm_interpret so that
// it executes the given function and then stops
//
bool vm_invoke_no_arg_function(seni_vm *vm, seni_fn_info *fn_info)
{
  if (fn_info->num_args != 0) {
    SENI_ERROR("repeat/test draw function cannot have any arguments");
    return false;
  }

  vm_setup_function_invoke(vm, fn_info);
  return vm_function_invoke(vm);
}

void vm_setup_function_invoke(seni_vm *vm, seni_fn_info *fn_info)
{
  // push a frame onto the stack whose return address is the program's STOP instruction
  i32 stop_address = program_stop_location(vm->program);
  i32 i;

  seni_var *stack_d = &(vm->stack[vm->sp]);
  seni_var *v = NULL;

  // make room for the labelled arguments
  for (i = 0; i < fn_info->num_args * 2; i++) {
    v = stack_d; stack_d++; vm->sp++;
  }
  
  i32 fp = vm->sp;

  // push the caller's fp
  v = stack_d; stack_d++; vm->sp++;
  v->type = VAR_INT;
  v->value.i = vm->fp;

  // push stop address ip
  v = stack_d; stack_d++; vm->sp++;
  v->type = VAR_INT;
  v->value.i = stop_address;

  // push num_args
  v = stack_d; stack_d++; vm->sp++;
  v->type = VAR_INT;
  v->value.i = fn_info->num_args;

  vm->ip = fn_info->body_address;
  vm->fp = fp;
  vm->local = vm->sp;

  // clear the memory that's going to be used for locals
  for (i32 i = 0; i < MEMORY_LOCAL_SIZE; i++) {
    // setting all memory as VAR_INT will prevent any weird ref count
    // stuff when we deal with the RET opcodes later on
    vm->stack[vm->sp].type = VAR_INT; 
    vm->sp++;
  }
}

bool vm_function_invoke(seni_vm *vm)
{
  // vm is now in a state that will execute the given function and then return to the STOP opcode
  //
  vm_interpret(vm, vm->program);


  // the above vm_interpret will eventually hit a RET, pop the frame,
  // push the function's result onto the stack and then jump to the stop_address
  // so we'll need to pop that function's return value off the stack

  vm->sp--;
  // correct ref-count if the function returned a vector
  seni_var *v = &(vm->stack[vm->sp]);

  bool b1 = vector_ref_count_decrement(vm, v);
  if (b1 == false) {
    SENI_ERROR("vm_invoke_no_arg_function: vector_ref_count_decrement failed");
    return false;
  }

  return true;
}

// executes a program on a vm
// returns true if we reached a STOP opcode
bool vm_interpret(seni_vm *vm, seni_program *program)
{
  bool b1, b2;
  f32 f1, f2;
  seni_memory_segment_type memory_segment_type;
  seni_fn_info *fn_info;
  seni_var *src, *dest, *tmp;

  register seni_bytecode *bc = NULL;
  register seni_var *v = NULL;
  register i32 ip = vm->ip;
  register i32 sp = vm->sp;
  register seni_var *stack_d = &(vm->stack[sp]);

  i32 num_args, addr;
  i32 iname;
  i32 i;

  // the function calling convention means that references to LOCAL variables after a
  // CALL need to hop-back down the frame pointers to the real local frame that they
  // should be referencing. (see notes.org: bytecode sequence when calling functions)
  //
  i32 hop_back = 0;
  i32 local, fp;

#define STACK_PEEK v = stack_d - 1
#define STACK_POP stack_d--; sp--; v = stack_d
#define STACK_PUSH v = stack_d; stack_d++; sp++

  TIMING_UNIT timing = get_timing();

  // store a reference to the program in the vm
  // required in case any of the native functions need to invoke vm_interpret
  vm->program = program;

  for (;;) {
    vm->opcodes_executed++;
    bc = &(program->code[ip++]);

#ifdef TRACE_PRINT_OPCODES
    pretty_print_bytecode(ip-1, bc); // 0-index the ip so that it matches the pretty print program output
#endif
    
    switch(bc->op) {
    case LOAD:
      STACK_PUSH;

      memory_segment_type = (seni_memory_segment_type)bc->arg0.value.i;
      if (memory_segment_type == MEM_SEG_CONSTANT) {
        var_move(v, &(bc->arg1));
      } else if (memory_segment_type == MEM_SEG_ARGUMENT) {

        // if we're referencing an ARG in-between CALL and CALL_0 make sure we use the right frame
        // i.e. we're using the caller function's ARG, not the callee
        fp = vm->fp;
        for (i = 0; i < hop_back; i++) {
          fp = vm->stack[fp].value.i;    // go back a frame
        }
        
        src = &(vm->stack[fp - bc->arg1.value.i - 1]);
#ifdef TRACE_PRINT_OPCODES
        pretty_print_seni_var(src, "---");
        SENI_LOG("--- hop_back is %d fp is %d\n", hop_back, fp);
#endif

        var_copy_onto_junk(vm, v, src);
        
      } else if (memory_segment_type == MEM_SEG_LOCAL) {

        // if we're referencing a LOCAL in-between CALL and CALL_0 make sure we use the right frame
        fp = vm->fp;
        for (i = 0; i < hop_back; i++) {
          fp = vm->stack[fp].value.i;    // go back a frame
        }
        local = fp + 3;         // get the correct frame's local
        
        src = &(vm->stack[local + bc->arg1.value.i]);

        var_copy_onto_junk(vm, v, src);
        
      } else if (memory_segment_type == MEM_SEG_GLOBAL) {
        src = &(vm->stack[vm->global + bc->arg1.value.i]);

        var_copy_onto_junk(vm, v, src);

      } else if (memory_segment_type == MEM_SEG_VOID) {
        // pushing from the void. i.e. create this object

        // temp: for the moment just assume that any LOAD VOID
        // means creating a new vector object.

        // also note that the VAR_VEC_HEAD is a seni_var from the stack
        // so it should never be sent to the vm->heap_avail
        vector_construct(vm, v);
        
      } else {
        SENI_ERROR("LOAD: unknown memory segment type %d", bc->arg0.value.i);
      }
      break;

    case STORE:
      STACK_POP;

      memory_segment_type = (seni_memory_segment_type)bc->arg0.value.i;
      if (memory_segment_type == MEM_SEG_ARGUMENT) {
        dest = &(vm->stack[vm->fp - bc->arg1.value.i - 1]);
        
        // check the current value of dest,
        var_move(dest, v);
#ifdef TRACE_PRINT_OPCODES
        pretty_print_seni_var(dest, "---");
        SENI_LOG("--- fp is %d\n", vm->fp);
#endif        
      } else if (memory_segment_type == MEM_SEG_LOCAL) {
        dest = &(vm->stack[vm->local + bc->arg1.value.i]);
        // using a copy since we could have a define in a loop and so
        // the previously assigned value will need to be reference counted
        var_copy(vm, dest, v);

        // the stack no longer references the vector, so decrement the rc
        b1 = vector_ref_count_decrement(vm, v);
        if (b1 == false) {
          SENI_ERROR("POP MEM_SEG_LOCAL: vector_ref_count_decrement failed");
          return false;
        }
        
      } else if (memory_segment_type == MEM_SEG_GLOBAL) {
        dest = &(vm->stack[vm->global + bc->arg1.value.i]);
        var_move(dest, v);
      } else if (memory_segment_type == MEM_SEG_VOID) {
        // normally pop from the stack and lose the value
        // but if it's a vector then decrement its ref count
        b1 = vector_ref_count_decrement(vm, v);
        if (b1 == false) {
          SENI_ERROR("STORE MEM_SEG_VOID: vector_ref_count_decrement failed");
          return false;
        }
      } else {
        SENI_ERROR("STORE: unknown memory segment type %d", bc->arg0.value.i);
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
        b1 = vector_ref_count_decrement(vm, dest);
        if (b1 == false) {
          SENI_ERROR("DEC_RC: vector_ref_count_decrement failed");
          return false;
        }
      } else if (memory_segment_type == MEM_SEG_VOID) {
        // no nothing
      } else {
        SENI_ERROR("DEC_RC: unknown memory segment type %d", bc->arg0.value.i);
      }
      break;

    case INC_RC:
      //
      memory_segment_type = (seni_memory_segment_type)bc->arg0.value.i;
      if (memory_segment_type == MEM_SEG_ARGUMENT) {
        dest = &(vm->stack[vm->fp - bc->arg1.value.i - 1]);
        vector_ref_count_increment(vm, dest);
      } else if (memory_segment_type == MEM_SEG_VOID) {
        // no nothing
      } else {
        SENI_ERROR("INC_RC: unknown memory segment type %d", bc->arg0.value.i);
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
      STACK_POP;
      num_args = v->value.i;

      STACK_POP;
      addr = v->value.i;

      // make room for the labelled arguments
      for (i = 0; i < num_args * 2; i++) {
        STACK_PUSH;
      }
      
      fp = sp;

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

      vm->ip = addr;
      vm->fp = fp;
      vm->local = sp;

      // clear the memory that's going to be used for locals
      for (i = 0; i < MEMORY_LOCAL_SIZE; i++) {
        // setting all memory as VAR_INT will prevent any weird ref count
        // stuff when we deal with the RET opcodes later on
        vm->stack[sp].type = VAR_INT; 
        sp++;
      }

      stack_d = &(vm->stack[sp]);
      ip = vm->ip;
      
      vm->sp = sp;

#ifdef TRACE_PRINT_OPCODES
      SENI_LOG("--- fp is %d\n", vm->fp);
#endif        
      break;

    case CALL_0:
      STACK_POP;
      addr = v->value.i;
      
      // like CALL but keep the existing frame and just update the ip and return ip
      
      // set the correct return ip
      vm->stack[vm->fp + 1].value.i = ip;

      // leap to a location
      ip = addr;
      vm->ip = ip;

      // we're now executing the body of the function so don't
      // hop back when we push any arguments or locals onto the stack
      hop_back = 0;
      break;

    case RET_0:
      // leap to the return ip
      vm->ip = vm->stack[vm->fp + 1].value.i;
      ip = vm->ip;

      hop_back++;

      break;
      
    case RET:
      // pop the frame
      //

      // grab whatever was the last value on the soon to be popped frame
      src = &(vm->stack[sp - 1]);
      vector_ref_count_increment(vm, src);

      num_args = vm->stack[vm->fp + 2].value.i;

      // decrement ref count on any locally defined vectors
      for (i = 0; i < MEMORY_LOCAL_SIZE; i++) {
        tmp = &(vm->stack[vm->local + i]);
        b1 = vector_ref_count_decrement(vm, tmp);
        if (b1 == false) {
          SENI_ERROR("RET local vector: vector_ref_count_decrement failed");
          return false;
        }
      }

      for (i = 0; i < num_args; i++) {
        tmp = &(vm->stack[vm->fp - ((i+1) * 2)]);
        b1 = vector_ref_count_decrement(vm, tmp);
        if (b1 == false) {
          SENI_ERROR("RET args: vector_ref_count_decrement failed");
          return false;
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
      var_move(v, src);

#ifdef TRACE_PRINT_OPCODES
      SENI_LOG("--- fp is %d\n", vm->fp);
#endif        
      break;

    case CALL_F:
      // like CALL but gets it's function information from program->fn_info
      
      // read the index into program->fn_name
      STACK_POP;
      i = v->value.i;
      fn_info = &(program->fn_info[i]);

      num_args = fn_info->num_args;
      addr = fn_info->arg_address;
      
      //num_args = bc->arg1.value.i;

      // make room for the labelled arguments
      for (i = 0; i < num_args * 2; i++) {
        STACK_PUSH;
      }
      
      fp = sp;

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

      vm->ip = addr;
      vm->fp = fp;
      vm->local = sp;

      // clear the memory that's going to be used for locals
      for (i = 0; i < MEMORY_LOCAL_SIZE; i++) {
        // setting all memory as VAR_INT will prevent any weird ref count
        // stuff when we deal with the RET opcodes later on
        vm->stack[sp].type = VAR_INT; 
        sp++;
      }

      stack_d = &(vm->stack[sp]);
      ip = vm->ip;
      
      vm->sp = sp;

#ifdef TRACE_PRINT_OPCODES
      SENI_LOG("--- fp is %d", vm->fp);
#endif        
      break;

    case CALL_F_0:
      // like CALL_0 but gets it's function information from program->fn_info      
      // read the index into program->fn_name
      STACK_POP;
      i = v->value.i;
      fn_info = &(program->fn_info[i]);

      addr = fn_info->body_address;
      
      // like CALL but keep the existing frame and just update the ip and return ip
      
      // set the correct return ip
      vm->stack[vm->fp + 1].value.i = ip;

      // leap to a location
      ip = addr;
      vm->ip = ip;

      // we're now executing the body of the function so don't
      // hop back when we push any arguments or locals onto the stack
      hop_back = 0;
      break;

    case NATIVE:
      iname = bc->arg0.value.i - NATIVE_START;
      num_args = bc->arg1.value.i;

      // sync vm with registers
      vm->sp = sp;

      native_function_ptr native_func = program->env->function_ptr[iname];
      seni_var *var = native_func(vm, num_args);
      
      // move vm->sp below the arguments, and decrement the rc of any vectors
      for (i = 0; i < num_args; i++) {
        vm->sp -= 2;
        tmp = &(vm->stack[vm->sp + 1]);
        if (tmp->type == VAR_VEC_HEAD) {
          b1 = vector_ref_count_decrement(vm, tmp);
          if (b1 == false) {
            SENI_ERROR("NATIVE: vector_ref_count_decrement failed");
            return false;
          }

          // this is now off the stack, so blow away the vector head
          tmp->type = VAR_INT;
          tmp->value.i = 0;
        }
      }
      
      // put the return value at the top of the stack
      var_move(&(vm->stack[vm->sp++]), var);
      
      // sync registers with vm
      sp = vm->sp;
      stack_d = &(vm->stack[sp]);

      break;

    case APPEND:
      // pops top two values: a value and a vector appends the value onto the vector
      
      STACK_POP;
      src = v;                      // the seni_var to append onto the vector

      STACK_POP;
      // v is the vector
      if (v->type != VAR_VEC_HEAD) {
        SENI_ERROR("APPEND expects the 2nd item on the stack to be a vector\n");
        return false;
      }

      b1 = append_to_vector(vm, v, src); // note: this uses a copy, should it be a move instead?
      if (b1 == false) {
        SENI_ERROR("append_to_vector failed in APPEND");
        DEBUG_INFO_PRINT(vm);
        pretty_print_seni_var(v, "the vector");
        pretty_print_seni_var(src, "the item to append");
        return false;
      }

      STACK_PUSH;
      break;

    case PILE:
      // takes a VAR_2D or a VECTOR and pushes the given number of elements onto the stack
      
      num_args = bc->arg0.value.i;

      STACK_POP;

      if (v->type == VAR_2D) {
        // top of the stack is a var_2d

        if (num_args == 2) {
          f1 = v->f32_array[0];
          f2 = v->f32_array[1];
        
          STACK_PUSH;
          f32_as_var(v, f1);
          STACK_PUSH;
          f32_as_var(v, f2);
        } else {
          SENI_ERROR("PILE: VAR_2D num_args = %d, requires 2", num_args);
        }
        
      } else if (v->type == VAR_VEC_HEAD) {
        // top of the stack contains a vector
        // take num_args elements from the vector and push them onto the stack
        seni_var _vec;
        var_move(&_vec, v);
        src = _vec.value.v->next;
        for (i = 0; i < num_args; i++) {
          STACK_PUSH;
          var_copy_onto_junk(vm, v, src);
          src = src->next;
        }
        b1 = vector_ref_count_decrement(vm, &_vec);
        if (b1 == false) {
          SENI_ERROR("PILE: vector_ref_count_decrement failed");
          return false;
        }
      } else {
        SENI_ERROR("PILE: expected to work with either VAR_2D or a Vector");
        pretty_print_seni_var(v, "PILE input");
      }
      
      break;

    case SQUISH2:
      // combines two floats from the stack into a single VAR_2D
      
      STACK_POP;
      if (v->type != VAR_FLOAT) {
        SENI_ERROR("SQUISH2 expects a float - non float in 2nd element of vector");
        // was the seni code declaring a vector of length 2 that didn't contain floats?
        // e.g. (define z [LAB RGB])
        // when would we ever want this kind of code?
        return false;
      }
      f2 = v->value.f;

      STACK_POP;
      if (v->type != VAR_FLOAT) {
        SENI_ERROR("SQUISH2 expects a float - non float in 1st element of vector");
        return false;
      }
      f1 = v->value.f;

      STACK_PUSH;
      v->type = VAR_2D;
      v->value.i = 0;
      v->f32_array[0] = f1;
      v->f32_array[1] = f2;
      break;

    case MTX_LOAD:
      matrix_stack_push(vm->matrix_stack);
      break;

    case MTX_STORE:
      matrix_stack_pop(vm->matrix_stack);
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

    case FLU_STORE:
      // function look-up version of STORE
      // pops the fn_info_index from the stack in order to determine
      // the correct location to store an argument parameter
      
      STACK_POP;
      i = v->value.i;
      
      STACK_POP;

      memory_segment_type = (seni_memory_segment_type)bc->arg0.value.i;
      if (memory_segment_type == MEM_SEG_ARGUMENT) {

        fn_info = &(program->fn_info[i]);
        iname = bc->arg1.value.i;
        
        dest = arg_memory_from_iname(fn_info, iname, &(vm->stack[vm->fp - 1]));
        if (dest != NULL) {
          var_move(dest, v);
        }
      } else {
        SENI_ERROR("FLU_STORE: should only be used with MEM_SEG_ARGUMENT, not %d", memory_segment_type);
      }
      break;

    case FLU_DEC_RC:
      // function look-up version of DEC_RC
      // pops the fn_info_index from the stack in order to determine
      // the correct location to decrement a vector's ref-count
      
      STACK_POP;

      // the var referenced by the bytecode is a default value for a function argument
      // it's going to be overwritten by a parameter that was specified by the calling
      // code.
      // We'll need to decrement it's ref count
      //
      memory_segment_type = (seni_memory_segment_type)bc->arg0.value.i;
      if (memory_segment_type == MEM_SEG_ARGUMENT) {
        i = v->value.i;
        fn_info = &(program->fn_info[i]);
        iname = bc->arg1.value.i; // get the iname of the argument

        dest = arg_memory_from_iname(fn_info, iname, &(vm->stack[vm->fp - 1]));
        if (dest != NULL) {
          b1 = vector_ref_count_decrement(vm, dest);
          if (b1 == false) {
            SENI_ERROR("FLU_DEC_RC: vector_ref_count_decrement failed");
            return false;
          }
        }

      } else if (memory_segment_type == MEM_SEG_VOID) {
        // no nothing
      } else {
        SENI_ERROR("FLU_DEC_RC: unknown memory segment type %d", memory_segment_type);
      }
      break;

    case FLU_INC_RC:
      // function look-up version of INC_RC
      // pops the fn_info_index from the stack in order to determine
      // the correct location to increment a vector's ref-count

      STACK_POP;
      
      memory_segment_type = (seni_memory_segment_type)bc->arg0.value.i;
      if (memory_segment_type == MEM_SEG_ARGUMENT) {
        i = v->value.i;
        fn_info = &(program->fn_info[i]);
        iname = bc->arg1.value.i; // get the iname of the argument

        dest = arg_memory_from_iname(fn_info, iname, &(vm->stack[vm->fp - 1]));
        if (dest != NULL) {
          vector_ref_count_increment(vm, dest);
        }
      } else if (memory_segment_type == MEM_SEG_VOID) {
        // no nothing
      } else {
        SENI_ERROR("FLU_INC_RC: unknown memory segment type %d", memory_segment_type);
      }
      break;
    case STOP:
      // stop execution of the program on this vm and return
      
      vm->sp = sp;
      vm->execution_time = timing_delta_from(timing);
      return true;
    default:
      SENI_ERROR("Unhandled opcode: %s\n", opcode_name(bc->op));
      return false;
    }
  }
}
