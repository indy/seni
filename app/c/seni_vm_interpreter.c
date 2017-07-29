#include "seni_vm_interpreter.h"
#include "seni_timing.h"
#include "seni_mathutil.h"

#include "utlist.h"

void gc_mark_vector(seni_var *vector)
{
  seni_var *v = vector->value.v; // the first heap-allocated seni_var
  
  while(v != NULL) {
    v->mark = true;
    if (v->type == VAR_VECTOR) {
      gc_mark_vector(v);
    }
    v = v->next;
  }
}

void gc_mark(seni_vm *vm)
{
  seni_var *v = vm->stack;
  
  for (i32 i = 0; i < vm->sp; i++) {
    // only VAR_VECTOR seni_vars allocated from the heap
    if (v->type == VAR_VECTOR) {
      gc_mark_vector(v);
    }
    v++;
  }
}

void gc_sweep(seni_vm *vm)
{
  vm->heap_avail = NULL;
  vm->heap_avail_size = 0;

  seni_var *v = vm->heap_slab;
  
  for (i32 i = 0; i < vm->heap_size; i++) {
    if (v->mark == true) {
      // in use, so clear mark for next gc
      v->mark = false;
    } else {
      // clear and add to heap_avail
      v->next = NULL;
      v->prev = NULL;
      v->type = VAR_INT;
      v->value.i = 0;

      DL_APPEND(vm->heap_avail, v);

      vm->heap_avail_size++;
    }

    v++;
  }
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
  for (i = 0; i < MEMORY_LOCAL_SIZE; i++) {
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
  vm_interpret(vm, vm->env, vm->program);


  // the above vm_interpret will eventually hit a RET, pop the frame,
  // push the function's result onto the stack and then jump to the stop_address
  // so we'll need to pop that function's return value off the stack

  vm->sp--;

  return true;
}

// executes a program on a vm
// returns true if we reached a STOP opcode
bool vm_interpret(seni_vm *vm, seni_env *env, seni_program *program)
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
  i32 stack_size = vm->stack_size;

#define STACK_PEEK v = stack_d - 1
#define STACK_POP stack_d--; sp--; v = stack_d
#define STACK_PUSH v = stack_d; stack_d++; sp++; if(sp == stack_size) { SENI_ERROR("Reached stack limit of %d", stack_size); return false;}

  TIMING_UNIT timing = get_timing();

  // store a reference to the program and env in the vm
  // required in case any of the native functions need to invoke vm_interpret
  vm->program = program;
  vm->env = env;

  for (;;) {

    if (vm->heap_avail_size < vm->heap_avail_size_before_gc) {
      SENI_LOG("GC Mark and Sweep");
      gc_mark(vm);
      gc_sweep(vm);
    }
    
    vm->opcodes_executed++;
    bc = &(program->code[ip++]);

#ifdef TRACE_PRINT_OPCODES
    bytecode_pretty_print(ip-1, bc); // ip has been incremented so back up to get the current ip
#endif
    
    switch(bc->op) {
    case LOAD:
      STACK_PUSH;

      memory_segment_type = (seni_memory_segment_type)bc->arg0.value.i;
      if (memory_segment_type == MEM_SEG_CONSTANT) {
        var_copy(v, &(bc->arg1));
      } else if (memory_segment_type == MEM_SEG_ARGUMENT) {

        // if we're referencing an ARG in-between CALL and CALL_0 make sure we use the right frame
        // i.e. we're using the caller function's ARG, not the callee
        fp = vm->fp;
        for (i = 0; i < hop_back; i++) {
          fp = vm->stack[fp].value.i;    // go back a frame
        }
        
        src = &(vm->stack[fp - bc->arg1.value.i - 1]);
#ifdef TRACE_PRINT_OPCODES
        var_pretty_print(src, "---");
        SENI_LOG("--- hop_back is %d fp is %d\n", hop_back, fp);
#endif

        var_copy(v, src);
        
      } else if (memory_segment_type == MEM_SEG_LOCAL) {

        // if we're referencing a LOCAL in-between CALL and CALL_0 make sure we use the right frame
        fp = vm->fp;
        for (i = 0; i < hop_back; i++) {
          fp = vm->stack[fp].value.i;    // go back a frame
        }
        local = fp + 3;         // get the correct frame's local
        
        src = &(vm->stack[local + bc->arg1.value.i]);

        var_copy(v, src);
        
      } else if (memory_segment_type == MEM_SEG_GLOBAL) {
        src = &(vm->stack[vm->global + bc->arg1.value.i]);

        var_copy(v, src);

      } else if (memory_segment_type == MEM_SEG_VOID) {
        // potential gc so sync vm->sp
        vm->sp = sp;

        // pushing from the void. i.e. create this object

        // temp: for the moment just assume that any LOAD VOID
        // means creating a new vector object.

        // also note that the VAR_VECTOR is a seni_var from the stack
        // so it should never be sent to the vm->heap_avail
        vector_construct(v);
        
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
        var_copy(dest, v);
#ifdef TRACE_PRINT_OPCODES
        var_pretty_print(dest, "---");
        SENI_LOG("--- fp is %d\n", vm->fp);
#endif        
      } else if (memory_segment_type == MEM_SEG_LOCAL) {
        dest = &(vm->stack[vm->local + bc->arg1.value.i]);
        // using a copy since we could have a define in a loop and so
        // the previously assigned value will need to be reference counted
        var_copy(dest, v);
        
      } else if (memory_segment_type == MEM_SEG_GLOBAL) {
        dest = &(vm->stack[vm->global + bc->arg1.value.i]);
        var_copy(dest, v);
      } else if (memory_segment_type == MEM_SEG_VOID) {
        // normally pop from the stack and lose the value
        // but if it's a vector then decrement its ref count
      } else {
        SENI_ERROR("STORE: unknown memory segment type %d", bc->arg0.value.i);
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

      num_args = vm->stack[vm->fp + 2].value.i;

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
      var_copy(v, src);

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

      native_function_ptr native_func = env->function_ptr[iname];
      seni_var *var = native_func(vm, num_args);
      
      // move vm->sp below the arguments, and decrement the rc of any vectors
      for (i = 0; i < num_args; i++) {
        vm->sp -= 2;
        tmp = &(vm->stack[vm->sp + 1]);
        if (tmp->type == VAR_VECTOR) {
          // this is now off the stack, so blow away the vector head
          tmp->type = VAR_INT;
          tmp->value.i = 0;
        }
      }
      
      // put the return value at the top of the stack
      var_copy(&(vm->stack[vm->sp++]), var);
      
      // sync registers with vm
      sp = vm->sp;
      stack_d = &(vm->stack[sp]);

      break;

    case APPEND:
      // pops top two values: a value and a vector appends the value onto the vector

      vm->sp = sp;
      
      seni_var *child_value = var_get_from_heap(vm);
      if (child_value == NULL) {
        SENI_ERROR("APPEND: cannot allocate child_value from pool");
        return false;
      }
      
      STACK_POP;
      src = v;                      // the seni_var to append onto the vector

      STACK_POP;
      // v is the vector

      if (v->type != VAR_VECTOR) {
        if (v->type == VAR_2D) {
          // convert the VAR_2D into a VAR_VECTOR
          f1 = v->f32_array[0];
          f2 = v->f32_array[1];

          vector_construct(v);
          vector_append_f32(vm, v, f1);
          vector_append_f32(vm, v, f2);
          
        } else {
          SENI_ERROR("APPEND expects the 2nd item on the stack to be a vector\n");
          var_pretty_print(v, "APPEND expects a vector");
          return false;
        }
      }

      var_copy(child_value, src);

      DL_APPEND(v->value.v, child_value);

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
        
      } else if (v->type == VAR_VECTOR) {
        // top of the stack contains a vector
        // take num_args elements from the vector and push them onto the stack
        seni_var _vec;
        var_copy(&_vec, v);
        src = _vec.value.v;
        for (i = 0; i < num_args; i++) {
          STACK_PUSH;
          var_copy(v, src);
          src = src->next;
        }
        
      } else {
        SENI_ERROR("PILE: expected to work with either VAR_2D or a Vector");
        var_pretty_print(v, "PILE input");
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

    case MOD:
      STACK_POP;
      f2 = v->value.f;
      
      STACK_POP;
      f1 = v->value.f;

      STACK_PUSH;

      i = (i32)f1 % (i32)f2;
      
      v->value.f = (f32)i;
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

    case SQRT:
      STACK_POP;
      f1 = v->value.f;
      
      STACK_PUSH;
      v->value.f = (f32)(sqrt(f1));
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
          var_copy(dest, v);
        }
      } else {
        SENI_ERROR("FLU_STORE: should only be used with MEM_SEG_ARGUMENT, not %d", memory_segment_type);
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
