// Copyright (C) 2019 Inderjit Gill

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

use crate::compiler::{Bytecode, BytecodeArg, ColourFormat, FnInfo, Mem, Program};
use crate::error::{Error, Result};
use crate::native::{build_native_fn_hash, Native};
use crate::opcodes::Opcode;
use crate::placeholder::*;

use std::collections::HashMap;
use std::fmt;

const FP_OFFSET_TO_LOCALS: usize = 4;
const FP_OFFSET_TO_HOP_BACK: usize = 3;
const FP_OFFSET_TO_NUM_ARGS: usize = 2;
const FP_OFFSET_TO_IP: usize = 1;

const MEMORY_GLOBAL_SIZE: usize = 40;
const MEMORY_LOCAL_SIZE: usize = 40;

// **************************************************
// VM bytecode interpreter
// **************************************************

#[derive(Clone, Debug)]
pub enum Var {
    Int(i32),
    Float(f32),
    Bool(bool),
    Long(u64),
    Name(i32),
    Vector(Vec<Var>),
    Colour(ColourFormat, f32, f32, f32, f32),
    V2D(f32, f32),
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Var::Int(i) => write!(f, "Int({})", i),
            Var::Float(fl) => write!(f, "Float({})", fl),
            Var::Bool(b) => write!(f, "Bool({})", b),
            Var::Long(u) => write!(f, "Long({})", u),
            Var::Name(i) => write!(f, "Name({})", i),
            Var::Vector(_) => write!(f, "Vector(todo: implement Display)"),
            Var::Colour(format, e0, e1, e2, e3) => {
                write!(f, "Colour({}, {}, {}, {}, {})", format, e0, e1, e2, e3)
            }
            Var::V2D(fl1, fl2) => write!(f, "V2D({}, {})", fl1, fl2),
        }
    }
}

pub struct Env {
    function_ptr: Placeholder,
    // word_lut: WordLut,
}

impl Env {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for Env {
    fn default() -> Env {
        Env { function_ptr: 42 }
    }
}

// the c-impl of vm (sen_vm) had pointers to env and program. these were required
// in case any of the native functions had to invoke vm_interpret.
// the rust version should just pass in these 2 extra args into the native functions
pub struct Vm {
    render_data: RenderData, // stores the generated vertex data
    matrix_stack: MatrixStack,
    prng_state: PrngState, // only used when evaluating bracket bindings

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

    native_fns: HashMap<Native, fn(&mut Vm, &Program, i32) -> Result<Var>>,
}

impl Default for Vm {
    fn default() -> Vm {
        let stack_size = 1024;
        let stack = vec![Var::Int(0); stack_size];

        let mut base_offset: usize = 0;
        let global = base_offset;
        base_offset += MEMORY_GLOBAL_SIZE;
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

            native_fns: build_native_fn_hash(),
        }
    }
}

fn bytecode_arg_to_var(bytecode_arg: &BytecodeArg) -> Result<Var> {
    match bytecode_arg {
        BytecodeArg::Int(i) => Ok(Var::Int(*i)),
        BytecodeArg::Float(f) => Ok(Var::Float(*f)),
        BytecodeArg::Name(iname) => Ok(Var::Name(*iname)),
        BytecodeArg::Mem(_mem) => Err(Error::VM(
            "bytecode_arg_to_var not implemented for BytecodeArg::Mem".to_string(),
        )),
        BytecodeArg::Native(_native) => Err(Error::VM(
            "bytecode_arg_to_var not implemented for BytecodeArg::Native".to_string(),
        )),
        BytecodeArg::Colour(format, e0, e1, e2, e3) => Ok(Var::Colour(*format, *e0, *e1, *e2, *e3)),
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

    fn arg_memory_from_iname(
        &self,
        fn_info: &FnInfo,
        iname: usize,
        stack_index: usize,
    ) -> Option<usize> {
        let mut args = stack_index;

        for _ in 0..fn_info.num_args {
            if let Var::Int(ina) = self.stack[args] {
                if ina as usize == iname {
                    return Some(args - 1); // move from the label onto the arg's default value
                }
            }

            args -= 2; // move past this arg and the next arg's value
        }

        None
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
                    if let Var::Int(hop_back) = self.stack[fp + FP_OFFSET_TO_HOP_BACK] {
                        for _ in 0..hop_back {
                            if let Var::Int(prev_fp) = self.stack[fp] {
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
                    if let Var::Int(hop_back) = self.stack[fp + FP_OFFSET_TO_HOP_BACK] {
                        for _ in 0..hop_back {
                            if let Var::Int(prev_fp) = self.stack[fp] {
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
                Mem::Void => {
                    // pushing from the void. i.e. create this object
                    self.stack[self.sp - 1] = Var::Vector(Vec::new());
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

    fn opcode_native(&mut self, program: &Program, bc: &Bytecode) -> Result<()> {
        let num_args = if let BytecodeArg::Int(num_args_) = bc.arg1 {
            num_args_
        } else {
            return Err(Error::VM(
                "opcode native requires arg1 to be num_args".to_string(),
            ));
        };

        let native = if let BytecodeArg::Native(native_) = bc.arg0 {
            native_
        } else {
            return Err(Error::VM(
                "opcode native requires arg0 to be a BytecodeArg::Native".to_string(),
            ));
        };

        let var = if let Some(function) = self.native_fns.get(&native) {
            function(self, program, num_args)?
        } else {
            return Err(Error::VM(
                "opcode native can't find native function".to_string(),
            ));
        };

        // push var onto the stack
        self.sp = self.sp_inc()?;
        self.stack[self.sp - 1] = var;

        Ok(())
    }

    fn opcode_store_f(&mut self, program: &Program, bc: &Bytecode) -> Result<()> {
        // function look-up version of STORE
        // pops the fn_info_index from the stack in order to determine the
        // correct location to store an argument parameter

        // pop
        self.sp = self.sp_dec()?;
        let i;
        if let Var::Int(i_) = self.stack[self.sp] {
            i = i_;
        } else {
            return Err(Error::VM("store_f".to_string()));
        }

        // pop the value
        self.sp = self.sp_dec()?;

        let mem;
        if let BytecodeArg::Mem(mem_) = bc.arg0 {
            mem = mem_;
        } else {
            return Err(Error::VM("opcode_store_f arg0 should be mem".to_string()));
        }

        match mem {
            Mem::Argument => {
                if let BytecodeArg::Int(iname) = bc.arg1 {
                    let fn_info = &program.fn_info[i as usize];
                    if let Some(dest_index) =
                        self.arg_memory_from_iname(fn_info, iname as usize, self.fp - 1)
                    {
                        // copy popped into stack[dest_index]
                        self.stack[dest_index] = self.stack[self.sp].clone();
                    }
                    // else this is trying to assign a parameter that doesn't exist for the function
                }
            }
            _ => {
                return Err(Error::VM(format!(
                    "opcode_store_f unknown memory type: {}",
                    mem
                )))
            }
        }

        Ok(())
    }

    fn opcode_add(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?; // stack push
                self.stack[self.sp - 1] = Var::Float(f1 + f2);
            }
        }
        Ok(())
    }

    fn opcode_sub(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?; // stack push
                self.stack[self.sp - 1] = Var::Float(f1 - f2);
            }
        }
        Ok(())
    }

    fn opcode_mul(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?; // stack push
                self.stack[self.sp - 1] = Var::Float(f1 * f2);
            }
        }
        Ok(())
    }

    fn opcode_div(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?; // stack push
                self.stack[self.sp - 1] = Var::Float(f1 / f2);
            }
        }
        Ok(())
    }

    fn opcode_mod(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?; // stack push
                self.stack[self.sp - 1] = Var::Float((*f1 as i32 % *f2 as i32) as f32);
            }
        }
        Ok(())
    }

    fn opcode_sqrt(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f1) = &self.stack[self.sp] {
            self.sp = self.sp_inc()?; // stack push
            self.stack[self.sp - 1] = Var::Float(f1.sqrt());
        }

        Ok(())
    }

    fn opcode_eq(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?; // stack push
                self.stack[self.sp - 1] = Var::Bool(f1 == f2);
            }
        }
        Ok(())
    }

    fn opcode_gt(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?; // stack push
                self.stack[self.sp - 1] = Var::Bool(f1 > f2);
            }
        }
        Ok(())
    }

    fn opcode_lt(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?; // stack push
                self.stack[self.sp - 1] = Var::Bool(f1 < f2);
            }
        }
        Ok(())
    }

    fn opcode_or(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Bool(b2) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Bool(b1) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?; // stack push
                self.stack[self.sp - 1] = Var::Bool(*b1 || *b2);
            }
        }
        Ok(())
    }

    fn opcode_not(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Bool(b1) = &self.stack[self.sp] {
            self.sp = self.sp_inc()?; // stack push
            self.stack[self.sp - 1] = Var::Bool(!*b1);
        }

        Ok(())
    }

    fn opcode_and(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Bool(b2) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Bool(b1) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?; // stack push
                self.stack[self.sp - 1] = Var::Bool(*b1 && *b2);
            }
        }
        Ok(())
    }

    fn opcode_jump(&mut self, bc: &Bytecode) -> Result<()> {
        if let BytecodeArg::Int(i) = bc.arg0 {
            self.ip = (self.ip as i32 + i - 1) as usize;
        } else {
            return Err(Error::VM("opcode_jump".to_string()));
        }
        Ok(())
    }

    fn opcode_jump_if(&mut self, bc: &Bytecode) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Bool(b) = &self.stack[self.sp] {
            // jump if the top of the stack is false
            if !(*b) {
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
        if let Var::Int(num_args_) = &self.stack[self.sp] {
            num_args = *num_args_;
        } else {
            return Err(Error::VM("opcode_call num_args_".to_string()));
        }

        let addr;
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Int(addr_) = &self.stack[self.sp] {
            addr = *addr_;
        } else {
            return Err(Error::VM("opcode_call addr_".to_string()));
        }

        // make room for the labelled arguments
        self.sp = self.sp_inc_by(num_args as usize * 2)?;

        let fp = self.sp;

        // push the caller's fp
        self.sp = self.sp_inc()?; // stack push
        self.stack[self.sp - 1] = Var::Int(self.fp as i32);

        // push the ip
        self.sp = self.sp_inc()?; // stack push
        self.stack[self.sp - 1] = Var::Int(self.ip as i32);

        // push the num_args
        self.sp = self.sp_inc()?; // stack push
        self.stack[self.sp - 1] = Var::Int(num_args);

        // push hop back
        if let Var::Int(hop_back) = self.stack[self.fp + FP_OFFSET_TO_HOP_BACK] {
            self.sp = self.sp_inc()?; // stack push
            self.stack[self.sp - 1] = Var::Int(hop_back + 1);
        }

        self.ip = addr as usize;
        self.fp = fp;
        self.local = self.sp;

        // clear the memory that's going to be used for locals
        for _ in 0..MEMORY_LOCAL_SIZE {
            // setting all memory as VAR_INT will prevent any weird ref count
            // stuff when we deal with the RET opcodes later on
            self.sp = self.sp_inc()?;
            self.stack[self.sp - 1] = Var::Int(0);
        }

        Ok(())
    }

    fn opcode_call_0(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop

        let addr;
        if let Var::Int(addr_) = &self.stack[self.sp] {
            addr = *addr_;
        } else {
            return Err(Error::VM("opcode_call_0".to_string()));
        }

        // like CALL but keep the existing frame and just update the ip and return ip

        // set the correct return ip
        self.stack[self.fp + FP_OFFSET_TO_IP] = Var::Int(self.ip as i32);

        // leap to a location
        self.ip = addr as usize;

        // we're now executing the body of the function so don't
        // hop back when we push any arguments or locals onto the stack
        self.stack[self.fp + FP_OFFSET_TO_HOP_BACK] = Var::Int(0);

        Ok(())
    }

    fn opcode_ret(&mut self) -> Result<()> {
        // pop the frame
        //

        // grab whatever was the last value on the soon to be popped frame
        let src = &self.stack[self.sp - 1];

        let num_args: usize;
        if let Var::Int(num_args_) = &self.stack[self.fp + FP_OFFSET_TO_NUM_ARGS] {
            num_args = *num_args_ as usize;
        } else {
            return Err(Error::VM("opcode_ret num_args_".to_string()));
        }

        // update vm
        self.sp = self.fp - (num_args * 2);
        if let Var::Int(ip) = &self.stack[self.fp + FP_OFFSET_TO_IP] {
            self.ip = *ip as usize;
        } else {
            return Err(Error::VM("opcode_ret ip".to_string()));
        }
        if let Var::Int(fp) = &self.stack[self.fp] {
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
        if let Var::Int(ip) = self.stack[self.fp + FP_OFFSET_TO_IP] {
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
        if let Var::Int(fn_info_index_) = &self.stack[self.sp] {
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
        self.stack[self.sp - 1] = Var::Int(self.fp as i32);

        // push the ip
        self.sp = self.sp_inc()?; // stack push
        self.stack[self.sp - 1] = Var::Int(self.ip as i32);

        // push the num_args
        self.sp = self.sp_inc()?; // stack push
        self.stack[self.sp - 1] = Var::Int(num_args);

        // push hop back
        if let Var::Int(hop_back) = self.stack[self.fp + FP_OFFSET_TO_HOP_BACK] {
            self.sp = self.sp_inc()?; // stack push
            self.stack[self.sp - 1] = Var::Int(hop_back + 1);
        }

        self.ip = addr as usize;
        self.fp = fp;
        self.local = self.sp;

        // clear the memory that's going to be used for locals
        for _ in 0..MEMORY_LOCAL_SIZE {
            // setting all memory as VAR_INT will prevent any weird ref count
            // stuff when we deal with the RET opcodes later on
            self.sp = self.sp_inc()?;
            self.stack[self.sp - 1] = Var::Int(0);
        }

        Ok(())
    }

    fn opcode_call_f_0(&mut self, program: &Program) -> Result<()> {
        // like CALL_0 but gets it's function information from program->fn_info
        let fn_info_index;
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Int(fn_info_index_) = &self.stack[self.sp] {
            fn_info_index = *fn_info_index_ as usize;
        } else {
            return Err(Error::VM("opcode_call_f fn_info_index_".to_string()));
        }
        let fn_info = &program.fn_info[fn_info_index];

        let addr = fn_info.body_address;

        // like CALL but keep the existing frame and just update the ip and return ip

        // set the correct return ip
        self.stack[self.fp + FP_OFFSET_TO_IP] = Var::Int(self.ip as i32);

        // leap to a location
        self.ip = addr as usize;

        // we're now executing the body of the function so don't
        // hop back when we push any arguments or locals onto the stack
        self.stack[self.fp + FP_OFFSET_TO_HOP_BACK] = Var::Int(0);

        Ok(())
    }

    fn opcode_squish2(&mut self) -> Result<()> {
        // combines two floats from the stack into a single Var::V2D

        self.sp = self.sp_dec()?; // stack pop
        let f2 = if let Var::Float(f2_) = &self.stack[self.sp] {
            *f2_
        } else {
            return Err(Error::VM(
                "opcode_squish2: f2 expected to be float".to_string(),
            ));
        };

        self.sp = self.sp_dec()?; // stack pop
        let f1 = if let Var::Float(f1_) = &self.stack[self.sp] {
            *f1_
        } else {
            return Err(Error::VM(
                "opcode_squish2: f1 expected to be float".to_string(),
            ));
        };

        self.sp = self.sp_inc()?; // stack push
        self.stack[self.sp - 1] = Var::V2D(f1, f2);

        Ok(())
    }

    fn opcode_append(&mut self) -> Result<()> {
        // pops top two values: a value and a vector appends the value onto the vector

        self.sp = self.sp_dec()?; // stack pop
        let cloned_var_value = self.stack[self.sp].clone();

        // a pop here to get the vector
        // a push here to place the updated vector
        // both of the above cancel out

        if let Var::V2D(a, b) = &self.stack[self.sp - 1] {
            // convert the VAR_2D into a VAR_VECTOR
            self.stack[self.sp - 1] =
                Var::Vector(vec![Var::Float(*a), Var::Float(*b), cloned_var_value]);
        } else if let Var::Vector(ref mut vec_vec) = &mut self.stack[self.sp - 1] {
            vec_vec.push(cloned_var_value);
        } else {
            return Err(Error::VM(
                "append requires either a Vector or V2D".to_string(),
            ));
        }

        Ok(())
    }

    fn opcode_pile(&mut self, bc: &Bytecode) -> Result<()> {
        // pops the V2D/Vector from the top of the stack and pushes the
        // given number of elements from the V2D/Vector onto the stack

        let num_args;
        if let BytecodeArg::Int(num_args_) = bc.arg0 {
            num_args = num_args_ as usize;
        } else {
            return Err(Error::VM("opcode_pile arg0 should be Int".to_string()));
        }

        self.sp = self.sp_dec()?; // stack pop

        if let Var::V2D(a, b) = &self.stack[self.sp] {
            // top of the stack is a var_2d
            let x = *a;
            let y = *b;
            if num_args == 2 {
                // push both floats onto the stack
                self.sp = self.sp_inc_by(2)?;
                self.stack[self.sp - 2] = Var::Float(x);
                self.stack[self.sp - 1] = Var::Float(y);
            } else {
                // note: is this really an error? what if only 1 value from V2D is required?
                return Err(Error::VM(format!(
                    "PILE: V2D num_args = {}, requires 2",
                    num_args
                )));
            }
            return Ok(());
        }

        // note: extra clone here, is there a way of taking the vec from the Var::Vector() ?
        let mut elems: Vec<Var> = Vec::with_capacity(num_args);
        if let Var::Vector(vec_vec) = &self.stack[self.sp] {
            for v in vec_vec.iter().take(num_args) {
                elems.push(v.clone());
            }
        } else {
            return Err(Error::VM("opcode_pile".to_string()));
        }

        for i in 0..num_args {
            self.sp = self.sp_inc()?;
            if let Some(var) = elems.get(i) {
                self.stack[self.sp - 1] = var.clone()
            }
        }

        Ok(())
    }

    fn opcode_vec_non_empty(&mut self) -> Result<()> {
        let top = &self.stack[self.sp - 1]; // peek

        let non_empty = if let Var::Vector(vec_vec) = top {
            !vec_vec.is_empty()
        } else if let Var::V2D(_, _) = top {
            // pretend that VAR_2D is a vector and special case all the VEC_* opcodes
            true
        } else {
            return Err(Error::VM(
                "VEC_NON_EMPTY requires either Vector or V2D on the stack".to_string(),
            ));
        };

        // push
        self.sp = self.sp_inc()?;
        self.stack[self.sp - 1] = Var::Bool(non_empty);

        Ok(())
    }

    fn opcode_vec_load_first(&mut self) -> Result<()> {
        // top of the stack has a vector (currently self.sp - 1)
        // going to push 2 elements onto the stack (index and first element)
        // so the vector will be at self.sp - 3

        // push the index (0)
        self.sp = self.sp_inc()?;
        self.stack[self.sp - 1] = Var::Int(0);

        // push the first element to the top
        self.sp = self.sp_inc()?;

        if let Var::Vector(vec_vec) = &self.stack[self.sp - 3] {
            if let Some(elem) = vec_vec.get(0) {
                self.stack[self.sp - 1] = elem.clone();
            }
        } else if let Var::V2D(a, _b) = &self.stack[self.sp - 3] {
            self.stack[self.sp - 1] = Var::Float(*a);
        } else {
            return Err(Error::VM(
                "VEC_LOAD_FIRST requires either Vector or V2D on the stack".to_string(),
            ));
        }

        Ok(())
    }

    fn opcode_vec_has_next(&mut self) -> Result<()> {
        // self.sp - 3 == the vector
        // self.sp - 2 == the index
        // self.sp - 1 == the element

        let mut index = 0;
        if let Var::Int(index_) = self.stack[self.sp - 2] {
            index = index_;
        }

        let mut has_next = false;
        if let Var::Vector(vec_vec) = &self.stack[self.sp - 3] {
            if vec_vec.len() > index as usize + 1 {
                has_next = true;
            }
        } else if let Var::V2D(_a, _b) = &self.stack[self.sp - 3] {
            if index == 0 {
                has_next = true;
            }
        } else {
            return Err(Error::VM(
                "VEC_HAS_NEXT requires either Vector or V2D on the stack".to_string(),
            ));
        }

        self.sp = self.sp_inc()?;
        self.stack[self.sp - 1] = Var::Bool(has_next);

        Ok(())
    }

    fn opcode_vec_next(&mut self) -> Result<()> {
        // self.sp - 3 == the vector
        // self.sp - 2 == the index
        // self.sp - 1 == the element

        // increment the index
        let mut index = 0;
        if let Var::Int(index_) = self.stack[self.sp - 2] {
            index = index_;
        }
        index += 1;
        self.stack[self.sp - 2] = Var::Int(index);

        // update the element at the top of the stack
        if let Var::Vector(vec_vec) = &self.stack[self.sp - 3] {
            if let Some(elem) = vec_vec.get(index as usize) {
                self.stack[self.sp - 1] = elem.clone();
            }
        } else if let Var::V2D(_a, b) = &self.stack[self.sp - 3] {
            if index == 1 {
                self.stack[self.sp - 1] = Var::Float(*b);
            } else {
                return Err(Error::VM("VEC_NEXT impossible situation".to_string()));
            }
        } else {
            return Err(Error::VM(
                "VEC_NEXT requires either Vector or V2D on the stack".to_string(),
            ));
        }

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
                Opcode::NATIVE => self.opcode_native(program, bc)?,
                Opcode::STORE_F => self.opcode_store_f(program, bc)?,
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
                Opcode::APPEND => self.opcode_append()?,
                Opcode::PILE => self.opcode_pile(bc)?,
                Opcode::VEC_NON_EMPTY => self.opcode_vec_non_empty()?,
                Opcode::VEC_LOAD_FIRST => self.opcode_vec_load_first()?,
                Opcode::VEC_HAS_NEXT => self.opcode_vec_has_next()?,
                Opcode::VEC_NEXT => self.opcode_vec_next()?,
                Opcode::MTX_LOAD => unimplemented!(),
                Opcode::MTX_STORE => unimplemented!(),
                Opcode::STOP => {
                    // todo: execution time
                    //
                    return Ok(());
                }
                _ => return Err(Error::VM(format!("Invalid Opcode: {}", bc.op))),
            }
        }
    }

    pub fn top_stack_value(&self) -> Result<Var> {
        let var = &self.stack[self.sp - 1];
        Ok(var.clone())
    }
}

#[cfg(test)]
pub mod tests {
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
        if let Var::Float(f) = vm_exec(s) {
            assert_eq!(f, val)
        }
    }

    pub fn is_int(s: &str, val: i32) {
        if let Var::Int(i) = vm_exec(s) {
            assert_eq!(i, val)
        }
    }

    fn is_bool(s: &str, val: bool) {
        if let Var::Bool(b) = vm_exec(s) {
            assert_eq!(b, val)
        }
    }

    fn is_vec_of_f32(s: &str, val: Vec<f32>) {
        if let Var::Vector(vec_vec) = vm_exec(s) {
            assert_eq!(vec_vec.len(), val.len());
            for (i, f) in val.iter().enumerate() {
                if let Some(Var::Float(ff)) = vec_vec.get(i) {
                    assert_eq!(ff, f);
                }
            }
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

        is_float("(loop (x from: 0 to: 5) (+ 42 38)) 9", 9.0);
        is_float(
            "(loop (x from: 0 to: 5) (loop (y from: 0 to: 5) (+ 3 4))) 9",
            9.0,
        );
    }

    #[test]
    fn test_vm_callret() {
        is_float("(fn (adder a: 9 b: 8) (+ a b)) (adder a: 5 b: 3)", 8.0);

        is_float(
            "(fn (adder a: 9 b: 8)
                 (+ a b))
             (adder a: 5 b: (+ 3 4))",
            12.0,
        ); // calc required for value
        is_float("(fn (adder a: 9 b: 8) (+ a b)) (adder a: 5 xxx: 3)", 13.0); // non-existent argument
        is_float("(fn (adder a: 9 b: 8) (+ a b)) (adder)", 17.0); // only default arguments
        is_float("(fn (adder a: 9 b: 8) (+ a b)) (adder a: 10)", 18.0); // missing argument
        is_float("(fn (adder a: 9 b: 8) (+ a b)) (adder b: 20)", 29.0); // missing argument

        is_float(
            "(fn (p2 a: 1) (+ a 2))
             (fn (p3 a: 1) (+ a 3))
             (+ (p2 a: 5) (p3 a: 10))",
            20.0,
        );
        is_float(
            "(fn (p2 a: 1) (+ a 2))
             (fn (p3 a: 1) (+ a 3))
             (p2 a: (p3 a: 10))",
            15.0,
        );
        is_float(
            "(fn (p2 a: 2) (+ a 5))
             (fn (p3 a: 3) (+ a 6))
             (fn (p4 a: 4) (+ a 7))
             (p2 a: (p3 a: (p4 a: 20)))",
            38.0,
        );

        // functions calling functions
        is_float(
            "(fn (z a: 1) (+ a 2))
             (fn (x c: 3) (+ c (z)))
             (x)",
            6.0,
        );
        is_float(
            "(fn (z a: 1) (+ a 2))
             (fn (x c: 3) (+ c (z a: 5)))
             (x)",
            10.0,
        );
        is_float(
            "(fn (z a: 1) (+ a 2))
             (fn (x c: 3) (+ c (z a: 5)))
             (x c: 5)",
            12.0,
        );
        // function calling another function, passing on one of it's local variables
        // (make use of the hop_back method of referring to the correct LOCAL frame)
        is_float(
            "(fn (z a: 1) (+ a 5))
             (fn (y)
                 (define x 10)
                 (z a: x))
             (y)",
            15.0,
        );
        is_float(
            "(fn (z a: 1) (+ a 5))
             (fn (zz a: 1) (+ a 9))
             (fn (y)
                 (define x 10)
                 (z a: (zz a: x)))
             (y)",
            24.0,
        );
        // function referencing a global
        is_float(
            "(define gs 30)
             (fn (foo at: 0) (+ at gs))
             (foo at: 10)",
            40.0,
        );
        // global references a function, function references a global
        is_float(
            "(define a 5 b (acc n: 2))
             (fn (acc n: 0) (+ n a))
             (+ a b)",
            12.0,
        );
        // using a function before it's been declared
        is_float(
            "(fn (x a: 33) (+ a (y c: 555)))
             (fn (y c: 444) c)
             (x a: 66)",
            621.0,
        );
        // passing an argument to a function that isn't being used
        // produces POP with VOID -1 args
        is_float(
            "(fn (x a: 33) (+ a (y c: 555)))
             (fn (y c: 444) c)
             (x a: 66 b: 8383)",
            621.0,
        );
    }

    #[test]
    fn test_vm_hop_back() {
        /*
        call-a invokes call-b and call-b's arguments invoke call-c and call-d.
        the process of two function calls whilst setting up call-b makes use of the hop_back
        mechanism in the interpreter
         */
        is_float(
            "(fn (call-d lambda: 994 omega: 993 theta: 992)
                 (* (+ lambda omega) theta))
             (fn (call-c epsilon: 995)
                 (+ epsilon epsilon))
             (fn (call-b delta: 997 gamma: 996)
                 (- gamma delta))
             (fn (call-a alpha: 999 beta: 998)
                 (call-b delta: (call-c epsilon: alpha)
                         gamma: (call-d lambda: 8 omega: beta theta: alpha)))
             (define res (call-a alpha: 2 beta: 5))
             res",
            22.0,
        );
    }

    #[test]
    fn text_vm_vector() {
        is_vec_of_f32("[4 5 6 7 8]", vec![4.0, 5.0, 6.0, 7.0, 8.0]);

        is_float("(loop (x from: 0 to: 5) [1 2 3 4 5]) 9", 9.0);

        // explicitly defined vector is returned
        is_vec_of_f32(
            "(fn (f a: 3) [1 2 3 4 5]) (fn (x) (f)) (x)",
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
        );

        // local var in function is returned
        is_vec_of_f32(
            "(fn (f a: 3) (define b [1 2 3 4 5]) b) (fn (x) (f)) (x)",
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
        );

        // local var in function is not returned
        is_float(
            "(fn (f a: 3) (define b [1 2 3 4 5]) 55) (fn (x) (f)) (x)",
            55.0,
        );

        // default argument for function is returned
        is_vec_of_f32(
            "(fn (f a: [1 2 3 4 5]) a) (fn (x) (f)) (x)",
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
        );

        // default argument for function is not returned
        is_float("(fn (f a: [1 2 3 4 5]) 3) (fn (x) (f)) (x)", 3.0);

        // default argument for function is not returned and
        // it's called with an explicitly declared vector
        is_float("(fn (f a: [1 2 3 4 5]) 3) (fn (x) (f a: [3 4])) (x)", 3.0);

        // default argument for function is not returned and
        // it's called with an unused argument
        is_float("(fn (f a: [1 2 3 4 5]) 3) (fn (x) (f z: [3 4])) (x)", 3.0);

        // default argument for function is not returned
        is_float("(fn (f a: [1 2 3 4 5]) a) (fn (x) (f a: 5)) (x)", 5.0);

        // argument into function is returned
        is_vec_of_f32(
            "(fn (f a: [3 4 5 6 7]) a) (fn (x) (f a: [1 2 3 4 5])) (x)",
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
        );
    }

    #[test]
    fn test_vm_vector_append() {
        is_vec_of_f32("(define v []) (++ v 100) v", vec![100.0]);
        is_vec_of_f32("(define v [1]) (++ v 100) v", vec![1.0, 100.0]);
        is_vec_of_f32("(define v [1 2]) (++ v 100) v", vec![1.0, 2.0, 100.0]);
        is_vec_of_f32(
            "(define v [1 2 3]) (++ v 100) v",
            vec![1.0, 2.0, 3.0, 100.0],
        );
        is_vec_of_f32(
            "(define v [1 2 3 4]) (++ v 100) v",
            vec![1.0, 2.0, 3.0, 4.0, 100.0],
        );
    }

    #[test]
    fn test_vm_fence() {
        is_vec_of_f32(
            "(define v []) (fence (x from: 0 to: 10 num: 3) (++ v x)) v",
            vec![0.0, 5.0, 10.0],
        );
        is_vec_of_f32(
            "(define v []) (fence (x from: 10 to: 0 num: 3) (++ v x)) v",
            vec![10.0, 5.0, 0.0],
        );
        is_vec_of_f32(
            "(define v []) (fence (x num: 5) (++ v x)) v",
            vec![0.0, 0.25, 0.5, 0.75, 1.0],
        );

        is_vec_of_f32(
            "(define v []) (fence (x from: 100 to: 900 num: 10) (++ v x)) v",
            vec![
                100.0000, 188.88889, 277.77777, 366.66666, 455.55554, 544.44446, 633.33333,
                722.22217, 811.1111, 900.0000,
            ],
        );
    }

    #[test]
    fn test_vm_loop() {
        is_vec_of_f32(
            "(define v []) (loop (x from: 0 to: 4) (++ v x)) v",
            vec![0.0, 1.0, 2.0, 3.0],
        );
        is_vec_of_f32(
            "(define v []) (loop (x from: 0 upto: 4) (++ v x)) v",
            vec![0.0, 1.0, 2.0, 3.0, 4.0],
        );
        is_vec_of_f32(
            "(define v []) (loop (x from: 0 to: 10 inc: 2) (++ v x)) v",
            vec![0.0, 2.0, 4.0, 6.0, 8.0],
        );
        is_vec_of_f32(
            "(define v []) (loop (x from: 0 upto: 10 inc: 2) (++ v x)) v",
            vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0],
        );
    }

    #[test]
    fn test_vm_each() {
        is_vec_of_f32(
            "(define inp [] v []) (each (x from: inp) (++ v x)) v",
            vec![],
        );
        is_vec_of_f32(
            "(define inp [99] v []) (each (x from: inp) (++ v x)) v",
            vec![99.0],
        );
        // this tests the special case of VAR_2D rather than the default VAR_VECTOR:
        is_vec_of_f32(
            "(define inp [42 43] v []) (each (x from: inp) (++ v x)) v",
            vec![42.0, 43.0],
        );
        is_vec_of_f32(
            "(define inp [0 1 2 3] v []) (each (x from: inp) (++ v x)) v",
            vec![0.0, 1.0, 2.0, 3.0],
        );
    }

    #[test]
    fn test_vm_function_address() {
        is_float(
            "(fn (k a: 5) (+ a a))
             (fn (l a: 5) (+ a a))
             (define foo (address-of l))
             (fn-call (foo a: 99 b: 88))",
            198.0,
        );

        // normal
        is_float(
            "(fn (dbl a: 5) (* a 2))
             (fn (trp a: 5) (* a 3))
             (define foo (address-of dbl))
             (fn-call (foo a: 44))",
            88.0,
        );
        is_float(
            "(fn (dbl a: 5) (* a 2))
             (fn (trp a: 5) (* a 3))
             (define foo (address-of trp))
             (fn-call (foo a: 44))",
            132.0,
        );

        // invalid arguments - use defaults
        is_float(
            "(fn (dbl a: 5) (* a 2))
             (fn (trp a: 5) (* a 3))
             (define foo (address-of dbl))
             (fn-call (foo z: 44))",
            10.0,
        );
        is_float(
            "(fn (dbl a: 5) (* a 2))
             (fn (trp a: 5) (* a 3))
             (define foo (address-of trp))
             (fn-call (foo z: 44))",
            15.0,
        );

        // some invalid arguments
        is_float(
            "(fn (dbl a: 5) (* a 2))
             (fn (trp a: 5) (* a 3))
             (define foo (address-of dbl))
             (fn-call (foo z: 100 a: 44))",
            88.0,
        );
        is_float(
            "(fn (dbl a: 5) (* a 2))
             (fn (trp a: 5) (* a 3))
             (define foo (address-of trp))
             (fn-call (foo z: 41 a: 44))",
            132.0,
        );
    }
}
