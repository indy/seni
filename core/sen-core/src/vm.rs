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

use crate::compiler::{Bytecode, FnInfo, Program};
use crate::error::{Error, Result};
use crate::opcodes::Opcode;
use crate::parser::WordLut;
use crate::placeholder::*;

const FP_OFFSET_TO_LOCALS: i32 = 4;
const FP_OFFSET_TO_HOP_BACK: i32 = 3;
const FP_OFFSET_TO_NUM_ARGS: i32 = 2;
const FP_OFFSET_TO_IP: i32 = 1;

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


pub enum Var {
    Int(i32, bool),
    Float(f32, bool),
    Bool(bool, bool),
    Long(u64, bool),
    Name(i32, bool),
    Vector(Box<Var>, bool),
    Colour(i32, f32, f32, f32, f32, bool),
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

impl Vm {
    pub fn new() -> Vm {
        Default::default()
    }

    pub fn stack_top_mut(&mut self) -> &mut Var {
        &mut self.stack[self.sp - 1]
    }

    pub fn stack_top(&mut self) -> &Var {
        &self.stack[self.sp - 1]
    }

    pub fn stack_push(&mut self) -> Result<()> {
        self.sp += 1;
        if self.sp >= self.stack_size {
            return Err(Error::VMStackOverflow)
        }
        Ok(())
    }

    pub fn stack_pop(&mut self) -> Result<()> {
        if self.sp == 0 {
            return Err(Error::VMStackUnderflow)
        }
        self.sp -= 1;
        Ok(())
    }

    pub fn interpret(&mut self, env: &Env, program: &Program) -> Result<()> {
        let mut bytecode_index: usize = 0;
        let mut bc = &program.code[bytecode_index];

        let mut stack_d: &Var = &self.stack[self.sp];
        let mut v: &Var = &self.stack[self.sp];

        loop {
            self.opcodes_executed += 1;

            self.ip += 1;
            bc = &program.code[self.ip];

            match bc.op {
                Opcode::ADD => {
                    // stack pop
                    {
                        if self.sp == 0 {
                            return Err(Error::VMStackUnderflow)
                        }
                        self.sp -= 1;
                        stack_d = &self.stack[self.sp];
                        v = stack_d;
                    }

                    if let Var::Float(f2, _) = v {
                        // stack pop
                        {
                            if self.sp == 0 {
                                return Err(Error::VMStackUnderflow)
                            }
                            self.sp -= 1;
                            stack_d = &self.stack[self.sp];
                            v = stack_d;
                        }
                        if let Var::Float(f1, _) = v {
                            // stack push
                            {
                                v = stack_d;
                                self.sp += 1;
                                if self.sp >= self.stack_size {
                                    return Err(Error::VMStackOverflow)
                                }
                                stack_d = &self.stack[self.sp];
                            }
                            self.stack[self.sp-1] = Var::Float(f1 + f2, true);
                        }
                    }
                },
                _ => return Err(Error::GeneralError)
            }
        }

        Ok(())
    }
}

impl Default for Vm {
    fn default() -> Vm {
        let stack_size = 50; // ???

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

            stack: Vec::with_capacity(stack_size),
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

// executes a program on a vm
// returns Ok if we reached a STOP opcode
pub fn vm_interpret(vm: &mut Vm, env: &Env, program: &Program) -> Result<()> {
    let mut bytecode_index: usize = 0;
    let mut bc = &program.code[bytecode_index];

    let mut ip = vm.ip;
    let mut sp = vm.sp;
    let mut stack_d: &Var = &vm.stack[sp];
    let mut v: &Var = &vm.stack[sp];

    /*
        // stack pop
        {
            if sp == 0 {
                return Err(Error::VMStackUnderflow)
            }
            sp -= 1;
            stack_d = &vm.stack[sp];
            v = stack_d;
        }

        // stack push
        {
            v = stack_d;
            sp += 1;
            if sp >= vm.stack_size {
                return Err(Error::VMStackOverflow)
            }
            stack_d = &vm.stack[sp];
        }
    */

    loop {
        vm.opcodes_executed += 1;

        ip += 1;
        bc = &program.code[ip];

        match bc.op {
            Opcode::ADD => {
                // stack pop
                {
                    if sp == 0 {
                        return Err(Error::VMStackUnderflow)
                    }
                    sp -= 1;
                    stack_d = &vm.stack[sp];
                    v = stack_d;
                }

                if let Var::Float(f2, _) = v {
                    // stack pop
                    {
                        if sp == 0 {
                            return Err(Error::VMStackUnderflow)
                        }
                        sp -= 1;
                        stack_d = &vm.stack[sp];
                        v = stack_d;
                    }
                    if let Var::Float(f1, _) = v {
                        // stack push
                        {
                            v = stack_d;
                            sp += 1;
                            if sp >= vm.stack_size {
                                return Err(Error::VMStackOverflow)
                            }
                            stack_d = &vm.stack[sp];
                        }
                        vm.stack[sp-1] = Var::Float(f1 + f2, true);
                    }
                }
            },
            _ => return Err(Error::GeneralError)
        }
    }

    Ok(())
}
