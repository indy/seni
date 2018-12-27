// Copyright (C) 2018 Inderjit Gill

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::compiler::{Bytecode, BytecodeArg, ColourFormat, Mem, Program};
use crate::error::{Error, Result};
use crate::opcodes::Opcode;
//use crate::parser::WordLut;
use crate::placeholder::*;

const FP_OFFSET_TO_LOCALS: usize = 4;
const FP_OFFSET_TO_HOP_BACK: usize = 3;
const FP_OFFSET_TO_NUM_ARGS: usize = 2;
const FP_OFFSET_TO_IP: usize = 1;

// known memory addresses

const SP: usize = 0;
const LCL: usize = 1;
const ARG: usize = 2;

// todo: update this comment + size constants
// these sizes are in terms of sen_var structures
// currently (23/11/2018), each sen_var is 56 bytes
// so 1MB can contain
// (1048576 / 56) == 18,724 sen_var structures
const HEAP_SIZE: usize = 18724;
const STACK_SIZE: usize = 1024;

// how low can the heap go before a GC is invoked
//
const HEAP_MIN_SIZE: usize = 10;
const MEMORY_GLOBAL_SIZE: usize = 40;
const MEMORY_LOCAL_SIZE: usize = 40;

// void gc_mark_vector(sen_var* vector)
// void gc_mark(sen_vm* vm)
// void gc_sweep(sen_vm* vm)

// **************************************************
// VM bytecode interpreter
// **************************************************

// sen_var* arg_memory_from_iname(sen_fn_info* fn_info, i32 iname, sen_var* args)
// void vm_function_set_argument_to_var(sen_vm* vm, sen_fn_info* fn_info, i32 iname, sen_var* src)
// void vm_function_set_argument_to_f32(sen_vm* vm, sen_fn_info* fn_info, i32 iname, f32 f)
// void vm_function_set_argument_to_2d(sen_vm* vm, sen_fn_info* fn_info, i32 iname, f32 x, f32 y)
// void vm_function_call_default_arguments(sen_vm* vm, sen_fn_info* fn_info)
// void vm_function_call_body(sen_vm* vm, sen_fn_info* fn_info)
// bool vm_run(sen_vm* vm, sen_env* env, sen_program* program)
// bool vm_interpret(sen_vm* vm, sen_env* env, sen_program* program)

#[derive(Clone, Debug)]
pub enum Var {
    Int(i32, bool),
    Float(f32, bool),
    Bool(bool, bool),
    Long(u64, bool),
    Name(i32, bool),
    Vector(Box<Var>, bool),
    Colour(ColourFormat, f32, f32, f32, f32, bool),
    V2D(f32, f32, bool),
}

pub struct Env {
    function_ptr: Placeholder,
    // word_lut: WordLut,
}

impl Env {
    pub fn new() -> Self {
        Env { function_ptr: 42 }
    }
}

// the c-impl of vm (sen_vm) had pointers to env and program. these were required
// in case any of the native functions had to invoke vm_interpret.
// the rust version should just pass in these 2 extra args into the native functions
pub struct Vm {
    // store a reference to the program and env in the vm
    // required in case any of the native functions need to invoke vm_interpret
    // program: &'a Program,
    // env: &'a Env,
    render_data: RenderData, // stores the generated vertex data

    matrix_stack: MatrixStack,

    prng_state: PrngState, // only used when evaluating bracket bindings

    // heap_size: i32,
    // heap_slab: Var,            // the contiguous block of allocated memory
    // heap_avail: Var,           // doubly linked list of unallocated sen_vars from the
    // // heap_slab
    // heap_avail_size_before_gc: i32, // how small can the heap get before a gc is
    // // invoked

    // heap_avail_size: i32,
    opcodes_executed: u64,
    execution_time: f32, // in msec

    stack: Vec<Var>,
    stack_size: usize,

    fp: usize, // frame pointer
    sp: usize, // stack pointer (points to the next free stack index)
    ip: usize, // instruction pointer

    global: usize, // single segment of memory at top of stack
    local: usize,  // per-frame segment of memory for local variables

    building_with_trait_within_vector: bool,
    trait_within_vector_index: bool,
}

impl Default for Vm {
    fn default() -> Vm {
        let stack_size = 1024;
        let stack = vec![Var::Int(0, true); stack_size];

        let mut base_offset: usize = 0;
        let global = base_offset;
        base_offset = base_offset + MEMORY_GLOBAL_SIZE;
        let fp = base_offset;

        // add some offsets so that the memory after fp matches a standard format
        base_offset += 1; // the caller's frame pointer
        base_offset += 1; // the caller's ip
        base_offset += 1; // the num_args of the called function
        base_offset += 1; // the caller's hop back

        let local = base_offset;
        base_offset += MEMORY_LOCAL_SIZE;
        let sp = base_offset;

        Vm {
            render_data: 0,

            matrix_stack: 0,

            prng_state: 0,

            // heap_size: usize,
            // heap_slab: Var::new(),
            // heap_avail: Var,           // doubly linked list of unallocated sen_vars from the
            // // heap_slab
            // heap_avail_size_before_gc: usize, // how small can the heap get before a gc is
            // // invoked

            // heap_avail_size: usize,
            opcodes_executed: 0,
            execution_time: 0.0, // in msec

            stack,
            stack_size,

            fp, // frame pointer
            sp, // stack pointer
            ip: 0,

            global,
            local,

            building_with_trait_within_vector: false,
            trait_within_vector_index: false,
        }
    }
}

fn bytecode_arg_to_var(bytecode_arg: &BytecodeArg) -> Result<Var> {
    match bytecode_arg {
        BytecodeArg::Int(i) => Ok(Var::Int(*i, true)),
        BytecodeArg::Float(f) => Ok(Var::Float(*f, true)),
        BytecodeArg::Name(iname) => Ok(Var::Name(*iname, true)),
        BytecodeArg::Mem(_mem) => Err(Error::VM(
            "bytecode_arg_to_var not implemented for BytecodeArg::Mem".to_string(),
        )),
        BytecodeArg::Colour(format, e0, e1, e2, e3) => {
            Ok(Var::Colour(*format, *e0, *e1, *e2, *e3, true))
        }
    }
}

impl Vm {
    pub fn new() -> Vm {
        Default::default()
    }

    fn sp_inc_by(&self, delta: usize) -> Result<usize> {
        if self.sp + delta >= self.stack_size {
            return Err(Error::VMStackOverflow);
        }
        Ok(self.sp + delta)
    }

    fn sp_inc(&self) -> Result<usize> {
        if self.sp + 1 >= self.stack_size {
            return Err(Error::VMStackOverflow);
        }
        Ok(self.sp + 1)
    }

    fn sp_dec(&self) -> Result<usize> {
        if self.sp == 0 {
            return Err(Error::VMStackUnderflow);
        }
        Ok(self.sp - 1)
    }

    fn opcode_load(&mut self, bc: &Bytecode) -> Result<()> {
        self.sp = self.sp_inc()?; // stack push

        if let BytecodeArg::Mem(mem) = bc.arg0 {
            match mem {
                Mem::Argument => {
                    // if we're referencing an ARG in-between CALL and CALL_0 make sure we
                    // use the right frame i.e. we're using the caller function's ARG, not
                    // the callee
                    let mut fp = self.fp;
                    if let Var::Int(hop_back, _) = self.stack[fp + FP_OFFSET_TO_HOP_BACK] {
                        for _ in 0..hop_back {
                            if let Var::Int(prev_fp, _) = self.stack[fp] {
                                fp = prev_fp as usize; // go back a frame
                            } else {
                                return Err(Error::VM("fp is wrong type?".to_string()));
                            }
                        }
                        if let BytecodeArg::Int(arg1) = bc.arg1 {
                            let src = &self.stack[fp - arg1 as usize - 1];
                            self.stack[self.sp - 1] = src.clone();
                        }
                    } else {
                        return Err(Error::VM("fp is wrong type?".to_string()));
                    }
                }
                Mem::Local => {
                    // if we're referencing a LOCAL in-between CALL and CALL_0 make sure we
                    // use the right frame

                    let mut fp = self.fp;
                    if let Var::Int(hop_back, _) = self.stack[fp + FP_OFFSET_TO_HOP_BACK] {
                        for _ in 0..hop_back {
                            if let Var::Int(prev_fp, _) = self.stack[fp] {
                                fp = prev_fp as usize; // go back a frame
                            } else {
                                return Err(Error::VM("fp is wrong type?".to_string()));
                            }
                        }
                        self.local = fp + FP_OFFSET_TO_LOCALS; // get the correct frame's local

                        if let BytecodeArg::Int(arg1) = bc.arg1 {
                            let src = &self.stack[self.local + arg1 as usize];
                            self.stack[self.sp - 1] = src.clone();
                        }
                    } else {
                        return Err(Error::VM("fp is wrong type?".to_string()));
                    }
                }
                Mem::Global => {
                    if let BytecodeArg::Int(arg1) = bc.arg1 {
                        let src = &self.stack[self.global + arg1 as usize];
                        self.stack[self.sp - 1] = src.clone();
                    }
                }
                Mem::Constant => self.stack[self.sp - 1] = bytecode_arg_to_var(&bc.arg1)?,
                // Mem::Void => ,
                _ => {
                    return Err(Error::VM(format!(
                        "opcode_load unknown memory type: {}",
                        mem
                    )))
                }
            }
        } else {
            return Err(Error::VM("LOAD requires arg0 to be Mem".to_string()));
        }

        Ok(())
    }

    fn opcode_store(&mut self, bc: &Bytecode) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        let popped = &self.stack[self.sp];

        let mem;
        if let BytecodeArg::Mem(mem_) = bc.arg0 {
            mem = mem_;
        } else {
            return Err(Error::VM("opcode_store arg0 should be mem".to_string()));
        }

        match mem {
            Mem::Argument => {
                if let BytecodeArg::Int(offset) = bc.arg1 {
                    self.stack[self.fp - offset as usize - 1] = popped.clone();
                }
            }
            Mem::Local => {
                if let BytecodeArg::Int(offset) = bc.arg1 {
                    self.stack[self.local + offset as usize] = popped.clone();
                }
            }
            Mem::Global => {
                if let BytecodeArg::Int(offset) = bc.arg1 {
                    self.stack[self.global + offset as usize] = popped.clone();
                }
            }
            Mem::Void => {
                // pop from the stack and lose the value
            }
            _ => {
                return Err(Error::VM(format!(
                    "opcode_store unknown memory type: {}",
                    mem
                )))
            }
        }

        Ok(())
    }

    fn opcode_add(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2, _) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1, _) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?; // stack push
                self.stack[self.sp - 1] = Var::Float(f1 + f2, true);
            }
        }
        Ok(())
    }

    fn opcode_sub(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2, _) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1, _) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?; // stack push
                self.stack[self.sp - 1] = Var::Float(f1 - f2, true);
            }
        }
        Ok(())
    }

    fn opcode_mul(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2, _) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1, _) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?; // stack push
                self.stack[self.sp - 1] = Var::Float(f1 * f2, true);
            }
        }
        Ok(())
    }

    fn opcode_div(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2, _) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1, _) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?; // stack push
                self.stack[self.sp - 1] = Var::Float(f1 / f2, true);
            }
        }
        Ok(())
    }

    fn opcode_mod(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2, _) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1, _) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?; // stack push
                self.stack[self.sp - 1] = Var::Float((*f1 as i32 % *f2 as i32) as f32, true);
            }
        }
        Ok(())
    }

    fn opcode_sqrt(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f1, _) = &self.stack[self.sp] {
            self.sp = self.sp_inc()?; // stack push
            self.stack[self.sp - 1] = Var::Float(f1.sqrt(), true);
        }

        Ok(())
    }

    fn opcode_eq(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2, _) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1, _) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?; // stack push
                self.stack[self.sp - 1] = Var::Bool(f1 == f2, true);
            }
        }
        Ok(())
    }

    fn opcode_gt(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2, _) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1, _) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?; // stack push
                self.stack[self.sp - 1] = Var::Bool(f1 > f2, true);
            }
        }
        Ok(())
    }

    fn opcode_lt(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2, _) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1, _) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?; // stack push
                self.stack[self.sp - 1] = Var::Bool(f1 < f2, true);
            }
        }
        Ok(())
    }

    fn opcode_or(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Bool(b2, _) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Bool(b1, _) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?; // stack push
                self.stack[self.sp - 1] = Var::Bool(*b1 || *b2, true);
            }
        }
        Ok(())
    }

    fn opcode_not(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Bool(b1, _) = &self.stack[self.sp] {
            self.sp = self.sp_inc()?; // stack push
            self.stack[self.sp - 1] = Var::Bool(!*b1, true);
        }

        Ok(())
    }

    fn opcode_and(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Bool(b2, _) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Bool(b1, _) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?; // stack push
                self.stack[self.sp - 1] = Var::Bool(*b1 && *b2, true);
            }
        }
        Ok(())
    }

    fn opcode_jump(&mut self, bc: &Bytecode) -> Result<()> {
        if let BytecodeArg::Int(i) = bc.arg0 {
            self.ip += i as usize - 1;
        } else {
            return Err(Error::VM("opcode_jump".to_string()));
        }
        Ok(())
    }

    fn opcode_jump_if(&mut self, bc: &Bytecode) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Bool(b, _) = &self.stack[self.sp] {
            // jump if the top of the stack is false
            if *b == false {
                // assume that compiler will always emit a BytecodeArg::Int as arg0 for JUMP_IF
                if let BytecodeArg::Int(i) = bc.arg0 {
                    self.ip += i as usize - 1;
                }
            }
        }
        Ok(())
    }

    fn opcode_call(&mut self) -> Result<()> {
        let num_args;
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Int(num_args_, _) = &self.stack[self.sp] {
            num_args = *num_args_;
        } else {
            return Err(Error::VM("opcode_call num_args_".to_string()));
        }

        let addr;
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Int(addr_, _) = &self.stack[self.sp] {
            addr = *addr_;
        } else {
            return Err(Error::VM("opcode_call addr_".to_string()));
        }

        // make room for the labelled arguments
        self.sp = self.sp_inc_by(num_args as usize * 2)?;

        let fp = self.sp;

        // push the caller's fp
        self.sp = self.sp_inc()?; // stack push
        self.stack[self.sp - 1] = Var::Int(self.fp as i32, true);

        // push the ip
        self.sp = self.sp_inc()?; // stack push
        self.stack[self.sp - 1] = Var::Int(self.ip as i32, true);

        // push the num_args
        self.sp = self.sp_inc()?; // stack push
        self.stack[self.sp - 1] = Var::Int(num_args, true);

        // push hop back
        if let Var::Int(hop_back, _) = self.stack[self.fp + FP_OFFSET_TO_HOP_BACK] {
            self.sp = self.sp_inc()?; // stack push
            self.stack[self.sp - 1] = Var::Int(hop_back + 1, true);
        }

        self.ip = addr as usize;
        self.fp = fp;
        self.local = self.sp;

        // clear the memory that's going to be used for locals
        for _ in 0..MEMORY_LOCAL_SIZE {
            // setting all memory as VAR_INT will prevent any weird ref count
            // stuff when we deal with the RET opcodes later on
            self.sp = self.sp_inc()?;
            self.stack[self.sp - 1] = Var::Int(0, true);
        }

        Ok(())
    }

    fn opcode_call_0(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop

        let addr;
        if let Var::Int(addr_, _) = &self.stack[self.sp] {
            addr = *addr_;
        } else {
            return Err(Error::VM("opcode_call_0".to_string()));
        }

        // like CALL but keep the existing frame and just update the ip and return ip

        // set the correct return ip
        self.stack[self.fp + FP_OFFSET_TO_IP] = Var::Int(self.ip as i32, true);

        // leap to a location
        self.ip = addr as usize;

        // we're now executing the body of the function so don't
        // hop back when we push any arguments or locals onto the stack
        self.stack[self.fp + FP_OFFSET_TO_HOP_BACK] = Var::Int(0, true);

        Ok(())
    }

    fn opcode_ret(&mut self) -> Result<()> {
        // pop the frame
        //

        // grab whatever was the last value on the soon to be popped frame
        let src = &self.stack[self.sp - 1];

        let num_args: usize;
        if let Var::Int(num_args_, _) = &self.stack[self.fp + FP_OFFSET_TO_NUM_ARGS] {
            num_args = *num_args_ as usize;
        } else {
            return Err(Error::VM("opcode_ret num_args_".to_string()));
        }

        // update vm
        self.sp = self.fp - (num_args * 2);
        if let Var::Int(ip, _) = &self.stack[self.fp + FP_OFFSET_TO_IP] {
            self.ip = *ip as usize;
        } else {
            return Err(Error::VM("opcode_ret ip".to_string()));
        }
        if let Var::Int(fp, _) = &self.stack[self.fp] {
            self.fp = *fp as usize;
        } else {
            return Err(Error::VM("opcode_ret fp".to_string()));
        }
        self.local = self.fp + FP_OFFSET_TO_LOCALS;

        // copy the previous frame's top stack value onto the current frame's stack
        self.sp = self.sp_inc()?; // stack push
        self.stack[self.sp - 1] = src.clone();

        Ok(())
    }

    fn opcode_ret_0(&mut self) -> Result<()> {
        // leap to the return ip
        if let Var::Int(ip, _) = self.stack[self.fp + FP_OFFSET_TO_IP] {
            self.ip = ip as usize;
        } else {
            return Err(Error::VM("opcode_ret_0".to_string()));
        }
        Ok(())
    }

    fn opcode_call_f(&mut self, program: &Program) -> Result<()> {
        // like CALL but gets it's function information from program->fn_info

        // read the index into program->fn_name
        let fn_info_index;
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Int(fn_info_index_, _) = &self.stack[self.sp] {
            fn_info_index = *fn_info_index_ as usize;
        } else {
            return Err(Error::VM("opcode_call_f fn_info_index_".to_string()));
        }
        let fn_info = &program.fn_info[fn_info_index];

        let num_args = fn_info.num_args;
        let addr = fn_info.arg_address;

        // make room for the labelled arguments
        self.sp = self.sp_inc_by(num_args as usize * 2)?;

        let fp = self.sp;

        // push the caller's fp
        self.sp = self.sp_inc()?; // stack push
        self.stack[self.sp - 1] = Var::Int(self.fp as i32, true);

        // push the ip
        self.sp = self.sp_inc()?; // stack push
        self.stack[self.sp - 1] = Var::Int(self.ip as i32, true);

        // push the num_args
        self.sp = self.sp_inc()?; // stack push
        self.stack[self.sp - 1] = Var::Int(num_args, true);

        // push hop back
        if let Var::Int(hop_back, _) = self.stack[self.fp + FP_OFFSET_TO_HOP_BACK] {
            self.sp = self.sp_inc()?; // stack push
            self.stack[self.sp - 1] = Var::Int(hop_back + 1, true);
        }

        self.ip = addr as usize;
        self.fp = fp;
        self.local = self.sp;

        // clear the memory that's going to be used for locals
        for _ in 0..MEMORY_LOCAL_SIZE {
            // setting all memory as VAR_INT will prevent any weird ref count
            // stuff when we deal with the RET opcodes later on
            self.sp = self.sp_inc()?;
            self.stack[self.sp - 1] = Var::Int(0, true);
        }

        Ok(())
    }

    fn opcode_call_f_0(&mut self, program: &Program) -> Result<()> {
        // like CALL_0 but gets it's function information from program->fn_info
        let fn_info_index;
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Int(fn_info_index_, _) = &self.stack[self.sp] {
            fn_info_index = *fn_info_index_ as usize;
        } else {
            return Err(Error::VM("opcode_call_f fn_info_index_".to_string()));
        }
        let fn_info = &program.fn_info[fn_info_index];

        let addr = fn_info.body_address;

        // like CALL but keep the existing frame and just update the ip and return ip

        // set the correct return ip
        self.stack[self.fp + FP_OFFSET_TO_IP] = Var::Int(self.ip as i32, true);

        // leap to a location
        self.ip = addr as usize;

        // we're now executing the body of the function so don't
        // hop back when we push any arguments or locals onto the stack
        self.stack[self.fp + FP_OFFSET_TO_HOP_BACK] = Var::Int(0, true);

        Ok(())
    }

    fn opcode_squish2(&mut self) -> Result<()> {
        // combines two floats from the stack into a single Var::V2D

        self.sp = self.sp_dec()?; // stack pop
        let f2 = if let Var::Float(f2_, _) = &self.stack[self.sp] {
            *f2_
        } else {
            return Err(Error::VM(
                "opcode_squish2: f2 expected to be float".to_string(),
            ));
        };

        self.sp = self.sp_dec()?; // stack pop
        let f1 = if let Var::Float(f1_, _) = &self.stack[self.sp] {
            *f1_
        } else {
            return Err(Error::VM(
                "opcode_squish2: f1 expected to be float".to_string(),
            ));
        };

        self.sp = self.sp_inc()?; // stack push
        self.stack[self.sp - 1] = Var::V2D(f1, f2, true);

        Ok(())
    }

    // executes a program on a vm
    // returns Ok if we reached a STOP opcode
    pub fn interpret(&mut self, _env: &Env, program: &Program) -> Result<()> {
        // sp == next free stack index
        // do sp_inc or sp_dec before accessing values as these funcs do sanity checks
        // means that a pop (via sp_dec) can reference stack[sp]
        // and that a push (via sp_inc) requires stack[sp-1]

        // let mut bytecode_index: usize = 0;
        let mut bc;

        loop {
            // println!("{}: ip: {}", self.opcodes_executed, self.ip);
            // if self.opcodes_executed > 500 {
            //     return Err(Error::VM("too many opcode executed?".to_string()))
            // }

            self.opcodes_executed += 1;
            bc = &program.code[self.ip];
            self.ip += 1;

            match bc.op {
                Opcode::LOAD => self.opcode_load(bc)?,
                Opcode::STORE => self.opcode_store(bc)?,
                Opcode::ADD => self.opcode_add()?,
                Opcode::SUB => self.opcode_sub()?,
                Opcode::MUL => self.opcode_mul()?,
                Opcode::DIV => self.opcode_div()?,
                Opcode::MOD => self.opcode_mod()?,
                Opcode::SQRT => self.opcode_sqrt()?,
                Opcode::EQ => self.opcode_eq()?,
                Opcode::GT => self.opcode_gt()?,
                Opcode::LT => self.opcode_lt()?,
                Opcode::AND => self.opcode_and()?,
                Opcode::OR => self.opcode_or()?,
                Opcode::NOT => self.opcode_not()?,
                Opcode::JUMP => self.opcode_jump(bc)?,
                Opcode::JUMP_IF => self.opcode_jump_if(bc)?,
                Opcode::CALL => self.opcode_call()?,
                Opcode::CALL_0 => self.opcode_call_0()?,
                Opcode::RET => self.opcode_ret()?,
                Opcode::RET_0 => self.opcode_ret_0()?,
                Opcode::CALL_F => self.opcode_call_f(program)?,
                Opcode::CALL_F_0 => self.opcode_call_f_0(program)?,
                Opcode::SQUISH2 => self.opcode_squish2()?,
                Opcode::STOP => {
                    // todo: execution time
                    //
                    return Ok(());
                }
                _ => return Err(Error::VM(format!("unknown bytecode: {}", bc.op))),
            }
        }
    }

    pub fn top_stack_value(&self) -> Result<Var> {
        let var = &self.stack[self.sp - 1];
        Ok(var.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::compile_program;
    use crate::parser::parse;

    fn vm_exec(s: &str) -> Var {
        let mut vm = Vm::new();
        let env = Env::new();

        let (ast, _word_lut) = parse(s).unwrap();
        let program = compile_program(&ast).unwrap();

        vm.interpret(&env, &program).unwrap();

        vm.top_stack_value().unwrap()
    }

    fn is_float(s: &str, val: f32) {
        if let Var::Float(f, _) = vm_exec(s) {
            assert_eq!(f, val)
        }
    }

    fn is_bool(s: &str, val: bool) {
        if let Var::Bool(b, _) = vm_exec(s) {
            assert_eq!(b, val)
        }
    }

    #[test]
    fn test_vm_basics() {
        is_float("(+ 2 3)", 5.0);
        is_float("(- 10 3)", 7.0);
        is_float("(* 4 3)", 12.0);
        is_float("(/ 20 5)", 4.0);
        is_float("(% 10 3)", 1.0);
        is_float("(sqrt 144)", 12.0);

        is_bool("(= 4 4)", true);
        is_bool("(= 4 5)", false);

        is_bool("(> 100 99)", true);
        is_bool("(> 50 55)", false);
        is_bool("(> 50 50)", false);

        is_bool("(< 50 55)", true);
        is_bool("(< 100 99)", false);
        is_bool("(< 50 50)", false);

        is_bool("(and (> 1 0) (> 1 0))", true);
        is_bool("(and (> 1 0) (< 1 0))", false);
        is_bool("(and (< 1 0) (> 1 0))", false);

        is_bool("(or (> 1 0) (> 1 0))", true);
        is_bool("(or (> 1 0) (< 1 0))", true);
        is_bool("(or (< 1 0) (> 1 0))", true);
        is_bool("(or (< 1 0) (< 1 0))", false);

        is_bool("(not (> 1 0))", false);
        is_bool("(not (< 1 0))", true);

        is_float("(if (< 5 10) 2 3)", 2.0);
        is_float("(if (> 5 10) 2 3)", 3.0);

        is_float("(define a 42) (define b 52) 10", 10.0);
        is_float("(define a 6) (define b 7) (+ a b)", 13.0);
        is_float("(define a 8 b 9) (+ a b)", 17.0);

        // is_float("(loop (x from: 0 to: 5) (+ 42 38)) 9", 9.0);
        // is_float("(loop (x from: 0 to: 5) (loop (y from: 0 to: 5) (+ 3 4))) 9", 9.0);
    }

    #[test]
    fn test_vm_problem() {
        is_float("(loop (x from: 0 to: 5) (+ 42 38)) 9", 9.0);
    }

    #[test]
    fn test_vm_callret() {
        is_float("(fn (adder a: 9 b: 8) (+ a b)) (adder a: 5 b: 3)", 8.0);

        is_float(
            "(fn (adder a: 9 b: 8) (+ a b)) (adder a: 5 b: (+ 3 4))",
            12.0,
        ); // calc required for value
        is_float("(fn (adder a: 9 b: 8) (+ a b)) (adder a: 5 xxx: 3)", 13.0); // non-existent argument
        is_float("(fn (adder a: 9 b: 8) (+ a b)) (adder)", 17.0); // only default arguments
        is_float("(fn (adder a: 9 b: 8) (+ a b)) (adder a: 10)", 18.0); // missing argument
        is_float("(fn (adder a: 9 b: 8) (+ a b)) (adder b: 20)", 29.0); // missing argument

        is_float(
            "(fn (p2 a: 1) (+ a 2)) (fn (p3 a: 1) (+ a 3)) (+ (p2 a: 5) (p3 a: 10))",
            20.0,
        );
        is_float(
            "(fn (p2 a: 1) (+ a 2)) (fn (p3 a: 1) (+ a 3)) (p2 a: (p3 a: 10))",
            15.0,
        );
        is_float("(fn (p2 a: 2) (+ a 5))(fn (p3 a: 3) (+ a 6))(fn (p4 a: 4) (+ a 7))(p2 a: (p3 a: (p4 a: 20)))",
                 38.0);

        // functions calling functions
        is_float("(fn (z a: 1) (+ a 2)) (fn (x c: 3) (+ c (z))) (x)", 6.0);
        is_float(
            "(fn (z a: 1) (+ a 2)) (fn (x c: 3) (+ c (z a: 5))) (x)",
            10.0,
        );
        is_float(
            "(fn (z a: 1) (+ a 2)) (fn (x c: 3) (+ c (z a: 5))) (x c: 5)",
            12.0,
        );

        // function calling another function, passing on one of it's local variables
        // (make use of the hop_back method of referring to the correct LOCAL frame)
        is_float(
            "(fn (z a: 1) (+ a 5)) (fn (y) (define x 10) (z a: x)) (y)",
            15.0,
        );
        is_float("(fn (z a: 1) (+ a 5)) (fn (zz a: 1) (+ a 9))(fn (y) (define x 10) (z a: (zz a: x))) (y)", 24.0);

        // function referencing a global
        is_float("(define gs 30)(fn (foo at: 0) (+ at gs))(foo at: 10)", 40.0);

        // global references a function, function references a global
        is_float(
            "(define a 5 b (acc n: 2)) (fn (acc n: 0) (+ n a)) (+ a b)",
            12.0,
        );

        // using a function before it's been declared
        is_float(
            "(fn (x a: 33) (+ a (y c: 555))) (fn (y c: 444) c)  (x a: 66)",
            621.0,
        );

        // passing an argument to a function that isn't being used
        // produces POP with VOID -1 args
        is_float(
            "(fn (x a: 33) (+ a (y c: 555))) (fn (y c: 444) c)  (x a: 66 b: 8383)",
            621.0,
        );
    }
}
