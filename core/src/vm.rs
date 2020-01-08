// Copyright (C) 2020 Inderjit Gill <email@indy.io>

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::colour::{Colour, ProcColourStateStruct};
use crate::context::Context;
use crate::error::{Error, Result};
use crate::focal::FocalStateStruct;
use crate::iname::Iname;
use crate::interp::InterpStateStruct;
use crate::keywords::Keyword;
use crate::native::execute_native;
use crate::opcodes::Opcode;
use crate::packable::{Mule, Packable};
use crate::prng::PrngStateStruct;
use crate::program::{Bytecode, BytecodeArg, FnInfo, Mem, Program};

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use log::error;

const FP_OFFSET_TO_LOCALS: usize = 4;
const FP_OFFSET_TO_HOP_BACK: usize = 3;
const FP_OFFSET_TO_NUM_ARGS: usize = 2;
const FP_OFFSET_TO_IP: usize = 1;

const MEMORY_GLOBAL_SIZE: usize = 40;
pub const MEMORY_LOCAL_SIZE: usize = 40;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VMProfiling {
    On,
    Off,
}

// **************************************************
// VM bytecode interpreter
// **************************************************

#[derive(Clone, Debug)]
pub enum Var {
    Int(i32),
    Float(f32),
    Bool(bool),
    Keyword(Keyword),
    Long(u64),
    Name(Iname),
    String(Iname),
    Vector(Vec<Var>),
    Colour(Colour),
    V2D(f32, f32),
    Debug(String), // this is temporary REMOVE
    InterpState(InterpStateStruct),
    ProcColourState(ProcColourStateStruct),
    FocalState(FocalStateStruct),
    PrngState(Rc<RefCell<PrngStateStruct>>),
}

impl Packable for Var {
    fn pack(&self, cursor: &mut String) -> Result<()> {
        match self {
            Var::Int(i) => cursor.push_str(&format!("INT {}", i)),
            Var::Float(fl) => cursor.push_str(&format!("FLOAT {}", fl)),
            Var::Bool(b) => Mule::pack_label_bool(cursor, "BOOLEAN", *b),
            Var::Keyword(kw) => {
                cursor.push_str("KW ");
                kw.pack(cursor)?;
            }
            Var::Long(u) => cursor.push_str(&format!("LONG {}", u)),
            Var::Name(i) => cursor.push_str(&format!("NAME {}", i)),
            Var::String(i) => cursor.push_str(&format!("STRING {}", i)),
            Var::Vector(vars) => {
                cursor.push_str("VEC ");
                Mule::pack_usize(cursor, vars.len());

                for v in vars {
                    Mule::pack_space(cursor);
                    v.pack(cursor)?;
                }
            }
            Var::Colour(col) => {
                cursor.push_str("COLOUR ");
                col.pack(cursor)?;
            }
            Var::V2D(fl1, fl2) => cursor.push_str(&format!("2D {} {}", fl1, fl2)),
            _ => {
                error!("Var::pack");
                return Err(Error::Packable);
            }
        }

        Ok(())
    }

    fn unpack(cursor: &str) -> Result<(Self, &str)> {
        if cursor.starts_with("INT ") {
            let rem = Mule::skip_forward(cursor, "INT ".len());
            let (val, rem) = Mule::unpack_i32(rem)?;
            Ok((Var::Int(val), rem))
        } else if cursor.starts_with("FLOAT ") {
            let rem = Mule::skip_forward(cursor, "FLOAT ".len());
            let (val, rem) = Mule::unpack_f32(rem)?;
            Ok((Var::Float(val), rem))
        } else if cursor.starts_with("BOOLEAN ") {
            let rem = Mule::skip_forward(cursor, "BOOLEAN ".len());
            let (val, rem) = Mule::unpack_bool(rem)?;
            Ok((Var::Bool(val), rem))
        } else if cursor.starts_with("KW ") {
            let rem = Mule::skip_forward(cursor, "KW ".len());
            let (val, rem) = Keyword::unpack(rem)?;
            Ok((Var::Keyword(val), rem))
        } else if cursor.starts_with("LONG ") {
            let rem = Mule::skip_forward(cursor, "LONG ".len());
            let (val, rem) = Mule::unpack_u64(rem)?;
            Ok((Var::Long(val), rem))
        } else if cursor.starts_with("NAME ") {
            let rem = Mule::skip_forward(cursor, "NAME ".len());
            let (val, rem) = Iname::unpack(rem)?;
            Ok((Var::Name(val), rem))
        } else if cursor.starts_with("STRING ") {
            let rem = Mule::skip_forward(cursor, "STRING ".len());
            let (val, rem) = Iname::unpack(rem)?;
            Ok((Var::String(val), rem))
        } else if cursor.starts_with("COLOUR ") {
            let rem = Mule::skip_forward(cursor, "COLOUR ".len());
            let (val, rem) = Colour::unpack(rem)?;
            Ok((Var::Colour(val), rem))
        } else if cursor.starts_with("VEC ") {
            let rem = Mule::skip_forward(cursor, "VEC ".len());
            let mut var_list = vec![];
            let (num_vars, rem) = Mule::unpack_usize(rem)?;
            let mut r = rem;
            for _ in 0..num_vars {
                r = Mule::skip_space(r);
                let (a_var, rem) = Var::unpack(r)?;
                r = rem;
                var_list.push(a_var);
            }
            Ok((Var::Vector(var_list), r))
        } else if cursor.starts_with("2D ") {
            let rem = Mule::skip_forward(cursor, "2D ".len());
            let (val0, rem) = Mule::unpack_f32_sp(rem)?;
            let (val1, rem) = Mule::unpack_f32(rem)?;
            Ok((Var::V2D(val0, val1), rem))
        } else {
            error!("Var::unpack");
            Err(Error::Packable)
        }
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Var::Int(i) => write!(f, "Int({})", i),
            Var::Float(fl) => write!(f, "Float({})", fl),
            Var::Bool(b) => write!(f, "Bool({})", b),
            Var::Keyword(kw) => write!(f, "Keyword{}", kw),
            Var::Long(u) => write!(f, "Long({})", u),
            Var::Name(i) => write!(f, "Name({})", i),
            Var::String(i) => write!(f, "String({})", i),
            Var::Vector(_) => write!(f, "Vector(todo: implement Display)"),
            Var::Colour(col) => write!(f, "Colour({})", col),
            Var::V2D(fl1, fl2) => write!(f, "V2D({}, {})", fl1, fl2),
            Var::Debug(s) => write!(f, "DEBUG: {}", s),
            Var::InterpState(state) => write!(f, "InterpState({:?})", state),
            Var::ProcColourState(state) => write!(f, "ProcColourState({:?})", state),
            Var::FocalState(state) => write!(f, "FocalState({:?})", state),
            Var::PrngState(state) => write!(f, "PrngState({:?})", state),
        }
    }
}

impl Var {
    pub fn get_float_value(var: &Var) -> Result<f32> {
        match var {
            Var::Float(f) => Ok(*f),
            _ => {
                error!("expected a Var::Float");
                Err(Error::VM)
            }
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            Var::Float(_) => true,
            _ => false,
        }
    }
}

pub struct ProbeSample {
    /// frame pointer
    pub fp: usize,
    /// stack pointer (points to the next free stack index)
    pub sp: usize,
    /// instruction pointer
    pub ip: usize,

    pub scalar: Option<f32>,
    pub scalar_v2: Option<(f32, f32)>,
}

impl ProbeSample {
    pub fn new(vm: &Vm) -> ProbeSample {
        ProbeSample {
            fp: vm.fp,
            sp: vm.sp,
            ip: vm.ip,
            scalar: None,
            scalar_v2: None,
        }
    }

    pub fn new_scalar(vm: &Vm, scalar: f32) -> ProbeSample {
        ProbeSample {
            fp: vm.fp,
            sp: vm.sp,
            ip: vm.ip,
            scalar: Some(scalar),
            scalar_v2: None,
        }
    }

    pub fn new_scalar_v2(vm: &Vm, scalar_v2: (f32, f32)) -> ProbeSample {
        ProbeSample {
            fp: vm.fp,
            sp: vm.sp,
            ip: vm.ip,
            scalar: None,
            scalar_v2: Some(scalar_v2),
        }
    }
}

pub struct Vm {
    /// only used when evaluating bracket bindings
    pub prng_state: PrngStateStruct,

    pub profiling: VMProfiling,
    pub opcode_count: Vec<u64>,
    pub opcodes_executed: u64,
    pub execution_time: f32, // in msec

    pub stack: Vec<Var>,
    pub stack_size: usize,

    /// frame pointer
    pub fp: usize,
    /// stack pointer (points to the next free stack index)
    pub sp: usize,
    /// instruction pointer
    pub ip: usize,

    /// single segment of memory at top of stack
    pub global: usize,

    pub building_with_trait_within_vector: bool,
    pub trait_within_vector_index: usize,

    /// used during testing
    pub probe_samples: Vec<ProbeSample>,
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

        base_offset += MEMORY_LOCAL_SIZE;
        // Q: why are we adding MEMORY_LOCAL_SIZE to the base_offset when there
        // is no function being called and we're in a global scope?
        // A: Constructs like the loop keyword compile down to bytecode that
        // uses STORE LOCAL opcodes. Since it's very common for scripts to use
        // the loop keyword at the top-level we'll need to reserve some of the
        // stack for 'local' variables even on the global scope.
        let sp = base_offset;

        Vm {
            prng_state: PrngStateStruct::new(10, 0.0, 1.0),

            profiling: VMProfiling::Off,
            opcode_count: vec![],
            opcodes_executed: 0,
            execution_time: 0.0, // in msec

            stack,
            stack_size,

            fp, // frame pointer
            sp, // stack pointer
            ip: 0,

            global,

            building_with_trait_within_vector: false,
            trait_within_vector_index: 0,

            probe_samples: vec![],
        }
    }
}

fn bytecode_arg_to_var(bytecode_arg: &BytecodeArg) -> Result<Var> {
    match bytecode_arg {
        BytecodeArg::Int(i) => Ok(Var::Int(*i)),
        BytecodeArg::Float(f) => Ok(Var::Float(*f)),
        BytecodeArg::Name(iname) => Ok(Var::Name(*iname)),
        BytecodeArg::String(iname) => Ok(Var::String(*iname)),
        BytecodeArg::Keyword(keyword) => Ok(Var::Keyword(*keyword)),
        BytecodeArg::Mem(_mem) => {
            error!("bytecode_arg_to_var not implemented for BytecodeArg::Mem");
            Err(Error::VM)
        }
        BytecodeArg::Native(_native) => {
            error!("bytecode_arg_to_var not implemented for BytecodeArg::Native");
            Err(Error::VM)
        }
        BytecodeArg::Colour(col) => Ok(Var::Colour(*col)),
    }
}

impl Vm {
    pub fn probe_clear(&mut self) {
        self.probe_samples = vec![];
    }

    pub fn probe_scalar(&mut self, scalar: f32) {
        let sample = ProbeSample::new_scalar(self, scalar);
        self.probe_samples.push(sample);
    }

    pub fn probe_scalar_v2(&mut self, x: f32, y: f32) {
        let sample = ProbeSample::new_scalar_v2(self, (x, y));
        self.probe_samples.push(sample);
    }

    pub fn set_prng_state(&mut self, prng: PrngStateStruct) {
        self.prng_state = prng;
    }

    pub fn reset(&mut self) {
        let mut base_offset: usize = 0;
        self.global = base_offset;
        base_offset += MEMORY_GLOBAL_SIZE;

        self.ip = 0;
        self.fp = base_offset;
        self.stack[self.fp] = Var::Int(0);

        // add some offsets so that the memory after fp matches a standard format
        base_offset += 1; // the caller's frame pointer
        base_offset += 1; // the caller's ip
        base_offset += 1; // the num_args of the called function
        base_offset += 1; // the caller's hop back

        base_offset += MEMORY_LOCAL_SIZE;
        self.sp = base_offset;

        // todo
        // vm->building_with_trait_within_vector = 0;
        // vm->trait_within_vector_index         = 0;
    }

    pub fn function_call_default_arguments(
        &mut self,
        context: &mut Context,
        program: &Program,
        fn_info: &FnInfo,
    ) -> Result<()> {
        let stop_address = program.stop_location();

        // make room for the labelled arguments
        self.sp = self.sp_inc_by((fn_info.num_args * 2) as usize)?;

        let fp = self.sp;

        // push the caller's fp
        self.sp = self.sp_inc()?;
        self.stack[self.sp - 1] = Var::Int(self.fp as i32);

        // push stop address ip
        self.sp = self.sp_inc()?;
        self.stack[self.sp - 1] = Var::Int(stop_address as i32);

        // push num_args
        self.sp = self.sp_inc()?;
        self.stack[self.sp - 1] = Var::Int(fn_info.num_args);

        // push hop back
        self.sp = self.sp_inc()?;
        self.stack[self.sp - 1] = Var::Int(0);

        self.ip = fn_info.arg_address;
        self.fp = fp;

        // clear the memory that's going to be used for locals
        for _ in 0..MEMORY_LOCAL_SIZE {
            // setting all memory as VAR_INT will prevent any weird ref count
            // stuff when we deal with the RET opcodes later on
            self.sp = self.sp_inc()?;
            self.stack[self.sp - 1] = Var::Int(0);
        }

        self.interpret(context, program)?;

        Ok(())
    }

    pub fn function_call_body(
        &mut self,
        context: &mut Context,
        program: &Program,
        fn_info: &FnInfo,
    ) -> Result<()> {
        // push a frame onto the stack whose return address is the program's STOP
        // instruction
        let stop_address = program.stop_location();

        // set the correct return ip
        self.stack[self.fp + FP_OFFSET_TO_IP] = Var::Int(stop_address as i32);

        // leap to a location
        self.ip = fn_info.body_address;

        self.interpret(context, program)?;

        // the above vm_interpret will eventually hit a RET, pop the frame,
        // push the function's result onto the stack and then jump to the stop_address
        // so we'll need to pop that function's return value off the stack
        self.sp = self.sp_dec()?;

        Ok(())
    }

    fn arg_memory_from_iname(
        &self,
        fn_info: &FnInfo,
        iname: Iname,
        stack_offset: usize,
    ) -> Option<usize> {
        let mut offset = stack_offset;

        // search the ARG memory for iname
        for _ in 0..fn_info.num_args {
            if let Var::Name(vi) = self.stack[offset] {
                if vi == iname {
                    return Some(offset - 1); // move from the label onto the arg's default value
                }
            }
            offset -= 2; // skip past the value of the arg and the next label's iname
        }

        None
    }

    pub fn function_set_argument_to_col(
        &mut self,
        fn_info: &FnInfo,
        iname: Iname,
        colour: &Colour,
    ) {
        if let Some(offset) = self.arg_memory_from_iname(fn_info, iname, self.fp - 1) {
            self.stack[offset] = Var::Colour(*colour);
        }
    }

    pub fn function_set_argument_to_f32(&mut self, fn_info: &FnInfo, iname: Iname, f: f32) {
        if let Some(offset) = self.arg_memory_from_iname(fn_info, iname, self.fp - 1) {
            self.stack[offset] = Var::Float(f);
        }
    }

    pub fn function_set_argument_to_2d(&mut self, fn_info: &FnInfo, iname: Iname, x: f32, y: f32) {
        if let Some(offset) = self.arg_memory_from_iname(fn_info, iname, self.fp - 1) {
            self.stack[offset] = Var::V2D(x, y);
        }
    }

    fn sp_inc_by(&self, delta: usize) -> Result<usize> {
        if self.sp + delta >= self.stack_size {
            error!("StackOverflow");
            return Err(Error::VM);
        }
        Ok(self.sp + delta)
    }

    fn sp_inc(&self) -> Result<usize> {
        if self.sp + 1 >= self.stack_size {
            error!("StackOverflow");
            return Err(Error::VM);
        }
        Ok(self.sp + 1)
    }

    fn sp_dec_by(&self, delta: usize) -> Result<usize> {
        if delta > self.sp {
            error!("StackUnderflow");
            return Err(Error::VM);
        }
        Ok(self.sp - delta)
    }

    fn sp_dec(&self) -> Result<usize> {
        if self.sp == 0 {
            error!("StackUnderflow");
            return Err(Error::VM);
        }
        Ok(self.sp - 1)
    }

    fn opcode_load(&mut self, bc: &Bytecode) -> Result<()> {
        let arg0 = bc.arg0;
        let arg1 = bc.arg1;

        self.sp = self.sp_inc()?; // stack push

        if let BytecodeArg::Mem(mem) = arg0 {
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
                                error!("Mem::Argument (hopback) fp is not Var::Int?");
                                return Err(Error::VM);
                            }
                        }
                        let src = &self.stack[fp - arg1.get_int()? as usize - 1];
                        self.stack[self.sp - 1] = src.clone();
                    } else {
                        error!("Mem::Argument: fp is not Var::Int?");
                        return Err(Error::VM);
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
                                error!("Mem::Local (hopback): fp is not Var::Int?");
                                return Err(Error::VM);
                            }
                        }
                        let local = fp + FP_OFFSET_TO_LOCALS; // get the correct frame's local
                        let src = &self.stack[local + arg1.get_int()? as usize];

                        self.stack[self.sp - 1] = src.clone();
                    } else {
                        error!("Mem::Local: fp is not Var::Int?");
                        return Err(Error::VM);
                    }
                }
                Mem::Global => {
                    let src = &self.stack[self.global + arg1.get_int()? as usize];
                    self.stack[self.sp - 1] = src.clone();
                }
                Mem::Constant => self.stack[self.sp - 1] = bytecode_arg_to_var(&arg1)?,
                Mem::Void => {
                    // pushing from the void. i.e. create this object
                    self.stack[self.sp - 1] = Var::Vector(Vec::new());
                }
            }
        } else {
            error!("LOAD requires arg0 to be Mem");
            return Err(Error::VM);
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
            error!("opcode_store arg0 should be mem");
            return Err(Error::VM);
        }

        let arg1 = bc.arg1.get_int()? as usize;

        match mem {
            Mem::Argument => {
                self.stack[self.fp - arg1 - 1] = popped.clone();
            }
            Mem::Local => {
                let local = self.fp + FP_OFFSET_TO_LOCALS; // get the correct frame's local
                self.stack[local + arg1] = popped.clone();
            }
            Mem::Global => {
                self.stack[self.global + arg1] = popped.clone();
            }
            Mem::Void => {
                // pop from the stack and lose the value
            }
            _ => {
                error!("opcode_store unknown memory type: {}", mem);
                return Err(Error::VM);
            }
        }

        Ok(())
    }

    pub fn show_stack_around_sp(&self) {
        // dbg!(self.sp);

        // let lowest = if self.sp > 4 {
        //     self.sp - 4
        // } else {
        //     0
        // };

        // for i in (lowest..self.sp).rev() {
        //     dbg!(i);
        //     dbg!(&self.stack[i]);
        // }
    }

    fn opcode_native(
        &mut self,
        context: &mut Context,
        program: &Program,
        bc: &Bytecode,
    ) -> Result<()> {
        let num_args = bc.arg1.get_int().map_err(|_| {
            error!("opcode native requires arg1 to be num_args");
            Error::VM
        })? as usize;

        let res = if let BytecodeArg::Native(native) = bc.arg0 {
            execute_native(self, context, program, native)?
        } else {
            error!("opcode_native");
            return Err(Error::VM);
        };

        // pop all of the arguments off the stack as well as the default mask
        self.sp -= num_args + 1;

        if let Some(var) = res {
            // push var onto the stack
            self.sp = self.sp_inc()?;
            self.stack[self.sp - 1] = var;
        }

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
            error!("store_f");
            return Err(Error::VM);
        }

        // pop the value
        self.sp = self.sp_dec()?;

        let mem;
        if let BytecodeArg::Mem(mem_) = bc.arg0 {
            mem = mem_;
        } else {
            error!("opcode_store_f arg0 should be mem");
            return Err(Error::VM);
        }

        match mem {
            Mem::Argument => {
                if let BytecodeArg::Name(iname) = bc.arg1 {
                    let fn_info = &program.fn_info[i as usize];
                    if let Some(dest_index) =
                        self.arg_memory_from_iname(fn_info, iname, self.fp - 1)
                    {
                        // copy popped into stack[dest_index]
                        self.stack[dest_index] = self.stack[self.sp].clone();
                    }
                    // else this is trying to assign a parameter that doesn't exist for the function
                }
            }
            _ => {
                error!("opcode_store_f unknown memory type: {}", mem);
                return Err(Error::VM);
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
            } else {
                error!("opcode_lt expected float at top-1 of stack");
                return Err(Error::VM);
            }
        } else {
            error!("opcode_lt expected float at top of stack");
            return Err(Error::VM);
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
        let jump = bc.arg0.get_int()?;

        self.ip = (self.ip as i32 + jump - 1) as usize;

        Ok(())
    }

    fn opcode_jump_if(&mut self, bc: &Bytecode) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Bool(b) = &self.stack[self.sp] {
            // jump if the top of the stack is false
            if !(*b) {
                // assume that compiler will always emit a BytecodeArg::Int as arg0 for JUMP_IF
                self.ip += bc.arg0.get_int()? as usize - 1;
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
            error!("opcode_call num_args_");
            return Err(Error::VM);
        }

        let addr;
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Int(addr_) = &self.stack[self.sp] {
            addr = *addr_;
        } else {
            error!("opcode_call addr_");
            return Err(Error::VM);
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
            error!("opcode_call_0");
            return Err(Error::VM);
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
            error!("opcode_ret num_args");
            return Err(Error::VM);
        }

        // update vm
        self.sp = self.fp - (num_args * 2);
        if let Var::Int(ip) = &self.stack[self.fp + FP_OFFSET_TO_IP] {
            self.ip = *ip as usize;
        } else {
            error!("opcode_ret ip");
            return Err(Error::VM);
        }
        if let Var::Int(fp) = &self.stack[self.fp] {
            self.fp = *fp as usize;
        } else {
            error!("opcode_ret fp");
            return Err(Error::VM);
        }

        // copy the previous frame's top stack value onto the current frame's stack
        self.sp = self.sp_inc()?; // stack push
        self.stack[self.sp - 1] = src.clone();

        Ok(())
    }

    fn opcode_ret_0(&mut self) -> Result<()> {
        // leap to the return ip
        if let Var::Int(ip) = self.stack[self.fp + FP_OFFSET_TO_IP] {
            self.ip = ip as usize;
            Ok(())
        } else {
            error!("opcode_ret_0");
            Err(Error::VM)
        }
    }

    fn opcode_call_f(&mut self, program: &Program) -> Result<()> {
        // like CALL but gets it's function information from program->fn_info

        // read the index into program->fn_name
        let fn_info_index;
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Int(fn_info_index_) = &self.stack[self.sp] {
            fn_info_index = *fn_info_index_ as usize;
        } else {
            error!("opcode_call_f fn_info_index_");
            return Err(Error::VM);
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
            error!("opcode_call_f fn_info_index_");
            return Err(Error::VM);
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

    fn opcode_squish(&mut self, bc: &Bytecode) -> Result<()> {
        if bc.arg0.is_int(2)
            && self.stack[self.sp - 1].is_float()
            && self.stack[self.sp - 2].is_float()
        {
            // combines two floats from the stack into a single Var::V2D

            self.sp = self.sp_dec()?; // stack pop
            let f2 = Var::get_float_value(&self.stack[self.sp])?;

            self.sp = self.sp_dec()?; // stack pop
            let f1 = Var::get_float_value(&self.stack[self.sp])?;

            self.sp = self.sp_inc()?; // stack push
            self.stack[self.sp - 1] = Var::V2D(f1, f2);
        } else {
            // pop the vars and combine them into a Var::Vec

            let num = bc.arg0.get_int()? as usize;
            let mut v: Vec<Var> = Vec::with_capacity(num);

            for i in 0..num {
                v.push(self.stack[(self.sp - num) + i].clone());
            }
            self.sp = self.sp_dec_by(num)?;

            self.sp = self.sp_inc()?; // stack push
            self.stack[self.sp - 1] = Var::Vector(v);
        }
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
            error!("append requires either a Vector or V2D");
            return Err(Error::VM);
        }

        Ok(())
    }

    fn opcode_pile(&mut self, bc: &Bytecode) -> Result<()> {
        // pops the V2D/Vector from the top of the stack and pushes the
        // given number of elements from the V2D/Vector onto the stack

        let num_args = bc.arg0.get_int().map_err(|_| {
            error!("opcode_pile arg0 should be Int");
            Error::VM
        })? as usize;

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
                error!("PILE: V2D num_args = {}, requires 2", num_args);
                return Err(Error::VM);
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
            error!("opcode_pile");
            return Err(Error::VM);
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
            error!("VEC_NON_EMPTY requires either Vector or V2D on the stack");
            return Err(Error::VM);
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
            error!("VEC_LOAD_FIRST requires either Vector or V2D on the stack");
            return Err(Error::VM);
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
            error!("VEC_HAS_NEXT requires either Vector or V2D on the stack");
            return Err(Error::VM);
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
                error!("VEC_NEXT impossible situation");
                return Err(Error::VM);
            }
        } else {
            error!("VEC_NEXT requires either Vector or V2D on the stack");
            return Err(Error::VM);
        }

        Ok(())
    }

    // called before the top level interpret is invoked
    pub fn init_for_main_program(
        &mut self,
        program: &Program,
        profiling: VMProfiling,
    ) -> Result<()> {
        self.profiling = profiling;
        self.ip = 0;

        if self.profiling == VMProfiling::On {
            self.opcodes_executed = 0;
            self.opcode_count = Vec::with_capacity(program.code.len());
            for _ in 0..program.code.len() {
                self.opcode_count.push(0);
            }
        }

        Ok(())
    }

    // executes a program on a vm
    // returns Ok if we reached a STOP opcode
    pub fn interpret(&mut self, context: &mut Context, program: &Program) -> Result<()> {
        // sp == next free stack index
        // do sp_inc or sp_dec before accessing values as these funcs do sanity checks
        // means that a pop (via sp_dec) can reference stack[sp]
        // and that a push (via sp_inc) requires stack[sp-1]
        let mut bc;

        loop {
            if self.profiling == VMProfiling::On {
                self.opcodes_executed += 1;
                self.opcode_count[self.ip] += 1;
            }

            bc = &program.code[self.ip];
            self.ip += 1;

            match bc.op {
                Opcode::LOAD => self.opcode_load(bc)?,
                Opcode::STORE => self.opcode_store(bc)?,
                Opcode::NATIVE => self.opcode_native(context, program, bc)?,
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
                Opcode::SQUISH => self.opcode_squish(bc)?,
                Opcode::APPEND => self.opcode_append()?,
                Opcode::PILE => self.opcode_pile(bc)?,
                Opcode::VEC_NON_EMPTY => self.opcode_vec_non_empty()?,
                Opcode::VEC_LOAD_FIRST => self.opcode_vec_load_first()?,
                Opcode::VEC_HAS_NEXT => self.opcode_vec_has_next()?,
                Opcode::VEC_NEXT => self.opcode_vec_next()?,
                Opcode::STOP => {
                    // todo: execution time
                    //
                    return Ok(());
                }
                _ => {
                    error!("Invalid Opcode: {}", bc.op);
                    return Err(Error::VM);
                }
            }
        }
    }

    pub fn println_profiling(&self, program: &Program) -> Result<()> {
        for (i, line) in self.opcode_count.iter().enumerate() {
            println!("{:>4}: {:>6}:      {}", i + 1, line, program.code[i]);
        }

        Ok(())
    }

    pub fn top_stack_value(&self) -> Result<Var> {
        let var = &self.stack[self.sp - 1];
        Ok(var.clone())
    }
}

pub trait StackPeek<T> {
    fn stack_peek(&self, offset: usize) -> Result<T>;
}

impl StackPeek<Keyword> for Vm {
    fn stack_peek(&self, offset: usize) -> Result<Keyword> {
        if let Var::Keyword(kw) = &self.stack[self.sp - offset] {
            Ok(*kw)
        } else {
            error!("stack_peek expected Var::Keyword");
            Err(Error::VM)
        }
    }
}

impl StackPeek<i32> for Vm {
    fn stack_peek(&self, offset: usize) -> Result<i32> {
        if let Var::Int(i) = &self.stack[self.sp - offset] {
            Ok(*i)
        } else {
            error!("stack_peek expected Var::Int");
            Err(Error::VM)
        }
    }
}

impl StackPeek<f32> for Vm {
    fn stack_peek(&self, offset: usize) -> Result<f32> {
        if let Var::Float(f) = &self.stack[self.sp - offset] {
            Ok(*f)
        } else {
            error!("stack_peek expected Var::Float");
            Err(Error::VM)
        }
    }
}

impl StackPeek<usize> for Vm {
    fn stack_peek(&self, offset: usize) -> Result<usize> {
        if let Var::Float(f) = &self.stack[self.sp - offset] {
            Ok(*f as usize)
        } else {
            error!("stack_peek expected Var::Float");
            Err(Error::VM)
        }
    }
}

impl StackPeek<(f32, f32)> for Vm {
    fn stack_peek(&self, offset: usize) -> Result<(f32, f32)> {
        if let Var::V2D(x, y) = &self.stack[self.sp - offset] {
            Ok((*x, *y))
        } else {
            error!("stack_peek expected Var::V2D");
            Err(Error::VM)
        }
    }
}

impl StackPeek<Iname> for Vm {
    fn stack_peek(&self, offset: usize) -> Result<Iname> {
        if let Var::Name(n) = &self.stack[self.sp - offset] {
            Ok(*n)
        } else if let Var::String(n) = &self.stack[self.sp - offset] {
            Ok(*n)
        } else {
            error!("stack_peek expected Var::Name or Var::String");
            Err(Error::VM)
        }
    }
}

impl StackPeek<Colour> for Vm {
    fn stack_peek(&self, offset: usize) -> Result<Colour> {
        if let Var::Colour(col) = &self.stack[self.sp - offset] {
            Ok(*col)
        } else {
            error!("stack_peek expected Var::Colour");
            Err(Error::VM)
        }
    }
}

impl StackPeek<bool> for Vm {
    fn stack_peek(&self, offset: usize) -> Result<bool> {
        if let Var::Bool(b) = &self.stack[self.sp - offset] {
            Ok(*b)
        } else {
            error!("stack_peek expected Var::Bool");
            Err(Error::VM)
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::compiler::compile_program;
    use crate::parser::parse;

    pub fn vm_run(vm: &mut Vm, context: &mut Context, s: &str) {
        let (ast, word_lut) = parse(s).unwrap();
        let program = compile_program(&ast, &word_lut).unwrap();

        context.reset_for_piece();
        vm.reset();
        vm.interpret(context, &program).unwrap();
    }

    pub fn vm_exec(vm: &mut Vm, context: &mut Context, s: &str) -> Var {
        vm_run(vm, context, s);
        vm.top_stack_value().unwrap()
    }

    pub fn is_float(s: &str, val: f32) {
        let mut vm: Vm = Default::default();
        let mut context: Context = Default::default();

        if let Var::Float(f) = vm_exec(&mut vm, &mut context, s) {
            assert_eq!(f, val)
        }
    }

    pub fn is_int(s: &str, val: i32) {
        let mut vm: Vm = Default::default();
        let mut context: Context = Default::default();

        if let Var::Int(i) = vm_exec(&mut vm, &mut context, s) {
            assert_eq!(i, val)
        }
    }

    pub fn is_bool(s: &str, val: bool) {
        let mut vm: Vm = Default::default();
        let mut context: Context = Default::default();

        if let Var::Bool(b) = vm_exec(&mut vm, &mut context, s) {
            assert_eq!(b, val)
        }
    }

    pub fn is_vec_of_f32(s: &str, val: Vec<f32>) {
        let mut vm: Vm = Default::default();
        let mut context: Context = Default::default();

        if let Var::Vector(vec_vec) = vm_exec(&mut vm, &mut context, s) {
            assert_eq!(vec_vec.len(), val.len());
            for (i, f) in val.iter().enumerate() {
                if let Some(Var::Float(ff)) = vec_vec.get(i) {
                    assert_eq!(ff, f);
                }
            }
        }
    }

    pub fn probe_has_scalars(s: &str, expected_scalars: Vec<f32>) {
        let mut vm: Vm = Default::default();
        let mut context: Context = Default::default();

        vm_exec(&mut vm, &mut context, s);

        for (i, sample) in vm.probe_samples.iter().enumerate() {
            if let Some(scalar) = sample.scalar {
                assert_eq!(scalar, expected_scalars[i], "mismatch at index {}", i);
            } else {
                assert!(false, "expected a scalar in the sample");
            }
        }
    }

    pub fn probe_has_scalars_v2(s: &str, expected_scalars_v2: Vec<(f32, f32)>) {
        let mut vm: Vm = Default::default();
        let mut context: Context = Default::default();

        vm_exec(&mut vm, &mut context, s);

        for (i, sample) in vm.probe_samples.iter().enumerate() {
            if let Some(scalar_v2) = sample.scalar_v2 {
                assert_eq!(
                    scalar_v2.0, expected_scalars_v2[i].0,
                    "mismatch for 0 element at index {}",
                    i
                );
                assert_eq!(
                    scalar_v2.1, expected_scalars_v2[i].1,
                    "mismatch for 1 element at index {}",
                    i
                );
            } else {
                assert!(false, "expected a scalar in the sample");
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
    fn test_vm_implied_args() {
        // explicit argument syntax
        is_float(
            "(define x 1 y 2)
             (fn (adder a: 99 b: 88) (+ a b))
             (adder a: x b: y)",
            3.0,
        );

        // explicit argument syntax, same names
        is_float(
            "(define a 1 b 2)
             (fn (adder a: 99 b: 88) (+ a b))
             (adder a: a b: b)",
            3.0,
        );

        // implied argument syntax
        is_float(
            "(define a 1 b 2)
             (fn (adder a: 99 b: 88) (+ a b))
             (adder a b)",
            3.0,
        );

        // default argument value plus the value of b is set to the global a
        // if implied argument syntax is going to mess up, it will mess up here
        //
        is_float(
            "(define a 5 b 10)
             (fn (adder a: 99 b: 88) (+ a b))
             (adder b: a)",
            104.0,
        );
    }

    #[test]
    fn test_vm_redefine() {
        // can use a define and then redefine it
        is_float(
            "(fn (something)
                 (define a 10)
                 (define b (+ a 2))
                 (define a 100)
                 b)
             (something)",
            12.0,
        );

        is_float(
            "(fn (something)
                 (define a 10)
                 (define b (+ a 2))
                 (define a 100)
                 (+ a b))
             (something)",
            112.0,
        );
    }

    #[test]
    fn test_vm_implicit_from() {
        // implicit from for a user defined function
        is_float(
            "(define x 33)
             (fn (increase from: 99 by: 88) (+ from by))
             (x.increase by: 10)",
            43.0,
        );

        // implicit from for a native function
        is_float(
            "(define some-vec [10 20 30 40 50 60 70 80])
             (some-vec.nth n: 2)",
            30.0,
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

    fn pack_compare(var: Var, expected: &str) {
        let mut res: String = "".into();
        var.pack(&mut res).unwrap();
        assert_eq!(expected, res);
    }

    fn unpack_compare_var_int(inp: &str, expected_val: i32, expected_rem: &str) {
        let (res, actual_rem) = Var::unpack(inp).unwrap();

        if let Var::Int(actual_val) = res {
            assert_eq!(actual_val, expected_val);
            assert_eq!(actual_rem, expected_rem);
        } else {
            assert_eq!(false, true);
        }
    }

    fn unpack_compare_var_float(inp: &str, expected_val: f32, expected_rem: &str) {
        let (res, actual_rem) = Var::unpack(inp).unwrap();

        if let Var::Float(actual_val) = res {
            assert_eq!(actual_val, expected_val);
            assert_eq!(actual_rem, expected_rem);
        } else {
            assert_eq!(false, true);
        }
    }

    fn unpack_compare_var_bool(inp: &str, expected_val: bool, expected_rem: &str) {
        let (res, actual_rem) = Var::unpack(inp).unwrap();

        if let Var::Bool(actual_val) = res {
            assert_eq!(actual_val, expected_val);
            assert_eq!(actual_rem, expected_rem);
        } else {
            assert_eq!(false, true);
        }
    }

    fn unpack_compare_var_long(inp: &str, expected_val: u64, expected_rem: &str) {
        let (res, actual_rem) = Var::unpack(inp).unwrap();

        if let Var::Long(actual_val) = res {
            assert_eq!(actual_val, expected_val);
            assert_eq!(actual_rem, expected_rem);
        } else {
            assert_eq!(false, true);
        }
    }

    fn unpack_compare_var_name(inp: &str, expected_val: Iname, expected_rem: &str) {
        let (res, actual_rem) = Var::unpack(inp).unwrap();

        if let Var::Name(actual_val) = res {
            assert_eq!(actual_val, expected_val);
            assert_eq!(actual_rem, expected_rem);
        } else {
            assert_eq!(false, true);
        }
    }

    fn unpack_compare_var_string(inp: &str, expected_val: Iname, expected_rem: &str) {
        let (res, actual_rem) = Var::unpack(inp).unwrap();

        if let Var::String(actual_val) = res {
            assert_eq!(actual_val, expected_val);
            assert_eq!(actual_rem, expected_rem);
        } else {
            assert_eq!(false, true);
        }
    }

    fn unpack_compare_var_v2d(
        inp: &str,
        expected_val0: f32,
        expected_val1: f32,
        expected_rem: &str,
    ) {
        let (res, actual_rem) = Var::unpack(inp).unwrap();

        if let Var::V2D(actual_val0, actual_val1) = res {
            assert_eq!(actual_val0, expected_val0);
            assert_eq!(actual_val1, expected_val1);
            assert_eq!(actual_rem, expected_rem);
        } else {
            assert_eq!(false, true);
        }
    }

    #[test]
    fn test_var_pack() {
        pack_compare(Var::Int(42), "INT 42");
        pack_compare(Var::Float(3.14), "FLOAT 3.14");
        pack_compare(Var::Bool(true), "BOOLEAN 1");
        pack_compare(Var::Bool(false), "BOOLEAN 0");
        pack_compare(Var::Long(544), "LONG 544");
        pack_compare(Var::Name(Iname::new(65)), "NAME 65");
        pack_compare(Var::String(Iname::new(33)), "STRING 33");
        pack_compare(Var::V2D(5.67, 8.90), "2D 5.67 8.9");
    }

    #[test]
    fn test_var_unpack() {
        unpack_compare_var_int("INT 42", 42, "");
        unpack_compare_var_int("INT 42 ", 42, " ");
        unpack_compare_var_int("INT 42 shabba", 42, " shabba");

        unpack_compare_var_float("FLOAT 42.33", 42.33, "");
        unpack_compare_var_float("FLOAT 42.33 ", 42.33, " ");
        unpack_compare_var_float("FLOAT 42.33 shabba", 42.33, " shabba");

        unpack_compare_var_bool("BOOLEAN 0", false, "");
        unpack_compare_var_bool("BOOLEAN 1", true, "");

        unpack_compare_var_long("LONG 42", 42, "");
        unpack_compare_var_long("LONG 42 ", 42, " ");
        unpack_compare_var_long("LONG 42 shabba", 42, " shabba");

        unpack_compare_var_name("NAME 42", Iname::new(42), "");
        unpack_compare_var_name("NAME 42 ", Iname::new(42), " ");
        unpack_compare_var_name("NAME 42 shabba", Iname::new(42), " shabba");

        unpack_compare_var_string("STRING 42", Iname::new(42), "");
        unpack_compare_var_string("STRING 42 ", Iname::new(42), " ");
        unpack_compare_var_string("STRING 42 shabba", Iname::new(42), " shabba");

        unpack_compare_var_v2d("2D 1.23 4.56", 1.23, 4.56, "");
    }
}
