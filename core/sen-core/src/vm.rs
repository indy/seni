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

use crate::compiler::{Bytecode, Program, Mem, BytecodeArg, ColourFormat};
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
        BytecodeArg::Mem(_mem) => Err(Error::VM("bytecode_arg_to_var not implemented for BytecodeArg::Mem".to_string())),
        BytecodeArg::Colour(format, e0, e1, e2, e3) => Ok(Var::Colour(*format, *e0, *e1, *e2, *e3, true)),
    }
}

impl Vm {
    pub fn new() -> Vm {
        Default::default()
    }

    fn sp_inc(&self) -> Result<usize> {
        if self.sp + 1 >= self.stack_size {
            return Err(Error::VMStackOverflow)
        }
        Ok(self.sp + 1)
    }

    fn sp_dec(&self) -> Result<usize> {
        if self.sp == 0 {
            return Err(Error::VMStackUnderflow)
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
                    if let Var::Int(hop_back, _) = self.stack[self.fp + FP_OFFSET_TO_HOP_BACK] {
                        for _ in 0..hop_back {
                            if let Var::Int(prev_fp, _) = self.stack[self.fp] {
                                self.fp = prev_fp as usize; // go back a frame
                            } else {
                                return Err(Error::VM("fp is wrong type?".to_string()))
                            }
                        }
                        if let BytecodeArg::Int(arg1) = bc.arg1 {
                            let src = &self.stack[self.fp - arg1 as usize - 1];
                            self.stack[self.sp - 1] = src.clone();
                        }

                    } else {
                        return Err(Error::VM("fp is wrong type?".to_string()))
                    }
                },
                //Mem::Local => ,
                //Mem::Global => ,
                Mem::Constant => self.stack[self.sp-1] = bytecode_arg_to_var(&bc.arg1)?,
                //Mem::Void => ,
                _ => return Err(Error::VM("fp is wrong type?".to_string()))
            }
        } else {
            return Err(Error::VM("LOAD requires arg0 to be Mem".to_string()))
        }

        Ok(())
    }

    fn opcode_add(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2, _) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1, _) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?;  // stack push
                self.stack[self.sp-1] = Var::Float(f1 + f2, true);
            }
        }
        Ok(())
    }

    fn opcode_sub(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2, _) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1, _) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?;  // stack push
                self.stack[self.sp-1] = Var::Float(f1 - f2, true);
            }
        }
        Ok(())
    }

    fn opcode_mul(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2, _) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1, _) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?;  // stack push
                self.stack[self.sp-1] = Var::Float(f1 * f2, true);
            }
        }
        Ok(())
    }

    fn opcode_div(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2, _) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1, _) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?;  // stack push
                self.stack[self.sp-1] = Var::Float(f1 / f2, true);
            }
        }
        Ok(())
    }

    fn opcode_mod(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2, _) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1, _) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?;  // stack push
                self.stack[self.sp-1] = Var::Float((*f1 as i32 % *f2 as i32) as f32, true);
            }
        }
        Ok(())
    }

    fn opcode_sqrt(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f1, _) = &self.stack[self.sp] {
            self.sp = self.sp_inc()?;  // stack push
            self.stack[self.sp-1] = Var::Float(f1.sqrt(), true);
        }

        Ok(())
    }

    fn opcode_eq(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2, _) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1, _) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?;  // stack push
                self.stack[self.sp-1] = Var::Bool(f1 == f2, true);
            }
        }
        Ok(())
    }

    fn opcode_gt(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2, _) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1, _) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?;  // stack push
                self.stack[self.sp-1] = Var::Bool(f1 > f2, true);
            }
        }
        Ok(())
    }

    fn opcode_lt(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Float(f2, _) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Float(f1, _) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?;  // stack push
                self.stack[self.sp-1] = Var::Bool(f1 < f2, true);
            }
        }
        Ok(())
    }

    fn opcode_or(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Bool(b2, _) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Bool(b1, _) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?;  // stack push
                self.stack[self.sp-1] = Var::Bool(*b1 || *b2, true);
            }
        }
        Ok(())
    }

    fn opcode_not(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Bool(b1, _) = &self.stack[self.sp] {
            self.sp = self.sp_inc()?;  // stack push
            self.stack[self.sp-1] = Var::Bool(!*b1, true);
        }

        Ok(())
    }
    fn opcode_and(&mut self) -> Result<()> {
        self.sp = self.sp_dec()?; // stack pop
        if let Var::Bool(b2, _) = &self.stack[self.sp] {
            self.sp = self.sp_dec()?; // stack pop
            if let Var::Bool(b1, _) = &self.stack[self.sp] {
                self.sp = self.sp_inc()?;  // stack push
                self.stack[self.sp-1] = Var::Bool(*b1 && *b2, true);
            }
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
            self.opcodes_executed += 1;

            self.ip += 1;
            bc = &program.code[self.ip];

            match bc.op {
                Opcode::LOAD => self.opcode_load(bc)?,
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
                Opcode::STOP => {
                    // todo: execution time
                    //
                    return Ok(())
                }
                _ => return Err(Error::GeneralError)
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
    use crate::parser::parse;
    use crate::compiler::{compile_program};

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
    fn test_vm() {
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
    }

}
