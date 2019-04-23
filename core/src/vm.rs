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

use crate::colour::{Colour, ProcColourStateStruct};
use crate::compiler::{Bytecode, BytecodeArg, FnInfo, Mem, Program};
use crate::ease::Easing;
use crate::error::{Error, Result};
use crate::focal::FocalStateStruct;
use crate::geometry::Geometry;
use crate::interp::InterpStateStruct;
use crate::keywords::Keyword;
use crate::matrix::MatrixStack;
use crate::name::Name;
use crate::native::execute_native;
use crate::opcodes::Opcode;
use crate::packable::{Mule, Packable};
use crate::prng::PrngStateStruct;
use crate::uvmapper::{BrushType, Mappings};

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

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
    Keyword(Keyword),
    Long(u64),
    Name(Name),
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
            _ => return Err(Error::Packable("Var::pack".to_string())),
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
            let (val, rem) = Name::unpack(rem)?;
            Ok((Var::Name(val), rem))
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
            Err(Error::Packable("Var::unpack".to_string()))
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
            _ => Err(Error::NotedError("expected a Var::Float".to_string())),
        }
    }
}

/// The Seni VM
/// the c-impl of vm (sen_vm) had pointers to env and program. these were required
/// in case any of the native functions had to invoke vm_interpret.
/// the rust version should just pass in these 2 extra args into the native functions
pub struct Vm {
    pub matrix_stack: MatrixStack,
    /// only used when evaluating bracket bindings
    pub prng_state: PrngStateStruct,

    pub mappings: Mappings,
    pub geometry: Geometry,

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

    pub debug_str: String,
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
        let sp = base_offset;

        Vm {
            matrix_stack: MatrixStack::new(),

            prng_state: PrngStateStruct::new(10, 0.0, 1.0),

            mappings: Mappings::new(),
            geometry: Geometry::new(),

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

            building_with_trait_within_vector: false,
            trait_within_vector_index: 0,

            debug_str: "".to_string(),
        }
    }
}

fn bytecode_arg_to_var(bytecode_arg: &BytecodeArg) -> Result<Var> {
    match bytecode_arg {
        BytecodeArg::Int(i) => Ok(Var::Int(*i)),
        BytecodeArg::Float(f) => Ok(Var::Float(*f)),
        BytecodeArg::Name(iname) => Ok(Var::Name(*iname)),
        BytecodeArg::Keyword(keyword) => Ok(Var::Keyword(*keyword)),
        BytecodeArg::Mem(_mem) => Err(Error::VM(
            "bytecode_arg_to_var not implemented for BytecodeArg::Mem".to_string(),
        )),
        BytecodeArg::Native(_native) => Err(Error::VM(
            "bytecode_arg_to_var not implemented for BytecodeArg::Native".to_string(),
        )),
        BytecodeArg::Colour(col) => Ok(Var::Colour(*col)),
    }
}

impl Vm {
    pub fn new() -> Vm {
        Default::default()
    }

    pub fn debug_str_clear(&mut self) {
        self.debug_str = "".to_string();
    }

    pub fn debug_str_append(&mut self, text: &str) {
        if !self.debug_str.is_empty() {
            self.debug_str += &" ".to_string();
        }
        self.debug_str += &text.to_string();
    }

    pub fn render_line(
        &mut self,
        from: (f32, f32),
        to: (f32, f32),
        width: f32,
        from_col: &Colour,
        to_col: &Colour,
        brush_type: BrushType,
        brush_subtype: usize,
    ) -> Result<()> {
        if let Some(matrix) = self.matrix_stack.peek() {
            let uvm = self.mappings.get_uv_mapping(brush_type, brush_subtype);

            self.geometry
                .render_line(matrix, from, to, width, from_col, to_col, uvm)
        } else {
            Err(Error::VM("no matrix for render_line".to_string()))
        }
    }
    pub fn render_rect(
        &mut self,
        position: (f32, f32),
        width: f32,
        height: f32,
        colour: &Colour,
    ) -> Result<()> {
        if let Some(matrix) = self.matrix_stack.peek() {
            let uvm = self.mappings.get_uv_mapping(BrushType::Flat, 0);
            self.geometry
                .render_rect(matrix, position, width, height, colour, uvm)
        } else {
            Err(Error::VM("no matrix for render_rect".to_string()))
        }
    }

    pub fn render_circle(
        &mut self,
        position: (f32, f32),
        width: f32,
        height: f32,
        colour: &Colour,
        tessellation: usize,
    ) -> Result<()> {
        if let Some(matrix) = self.matrix_stack.peek() {
            let uvm = self.mappings.get_uv_mapping(BrushType::Flat, 0);
            self.geometry
                .render_circle(matrix, position, width, height, colour, tessellation, uvm)
        } else {
            Err(Error::VM("no matrix for render_circle".to_string()))
        }
    }

    pub fn render_circle_slice(
        &mut self,
        position: (f32, f32),
        width: f32,
        height: f32,
        colour: &Colour,
        tessellation: usize,
        angle_start: f32,
        angle_end: f32,
        inner_width: f32,
        inner_height: f32,
    ) -> Result<()> {
        if let Some(matrix) = self.matrix_stack.peek() {
            let uvm = self.mappings.get_uv_mapping(BrushType::Flat, 0);
            self.geometry.render_circle_slice(
                matrix,
                position,
                width,
                height,
                colour,
                tessellation,
                angle_start,
                angle_end,
                inner_width,
                inner_height,
                uvm,
            )
        } else {
            Err(Error::VM("no matrix for render_circle_slice".to_string()))
        }
    }

    pub fn render_poly(&mut self, coords: &[Var], colours: &[Var]) -> Result<()> {
        if let Some(matrix) = self.matrix_stack.peek() {
            let uvm = self.mappings.get_uv_mapping(BrushType::Flat, 0);
            self.geometry.render_poly(matrix, coords, colours, uvm)
        } else {
            Err(Error::VM("no matrix for render_poly".to_string()))
        }
    }

    pub fn render_quadratic(
        &mut self,
        coords: &[f32; 6],
        width_start: f32,
        width_end: f32,
        width_mapping: Easing,
        t_start: f32,
        t_end: f32,
        colour: &Colour,
        tessellation: usize,
        brush_type: BrushType,
        brush_subtype: usize,
    ) -> Result<()> {
        if let Some(matrix) = self.matrix_stack.peek() {
            let uvm = self.mappings.get_uv_mapping(brush_type, brush_subtype);

            self.geometry.render_quadratic(
                matrix,
                coords,
                width_start,
                width_end,
                width_mapping,
                t_start,
                t_end,
                colour,
                tessellation,
                uvm,
            )
        } else {
            Err(Error::VM("no matrix for render_quadratic".to_string()))
        }
    }

    pub fn render_bezier(
        &mut self,
        coords: &[f32; 8],
        width_start: f32,
        width_end: f32,
        width_mapping: Easing,
        t_start: f32,
        t_end: f32,
        colour: &Colour,
        tessellation: usize,
        brush_type: BrushType,
        brush_subtype: usize,
    ) -> Result<()> {
        if let Some(matrix) = self.matrix_stack.peek() {
            let uvm = self.mappings.get_uv_mapping(brush_type, brush_subtype);

            self.geometry.render_bezier(
                matrix,
                coords,
                width_start,
                width_end,
                width_mapping,
                t_start,
                t_end,
                colour,
                tessellation,
                uvm,
            )
        } else {
            Err(Error::VM("no matrix for render_bezier".to_string()))
        }
    }

    pub fn render_bezier_bulging(
        &mut self,
        coords: &[f32; 8],
        line_width: f32,
        t_start: f32,
        t_end: f32,
        colour: &Colour,
        tessellation: usize,
        brush_type: BrushType,
        brush_subtype: usize,
    ) -> Result<()> {
        if let Some(matrix) = self.matrix_stack.peek() {
            let uvm = self.mappings.get_uv_mapping(brush_type, brush_subtype);

            self.geometry.render_bezier_bulging(
                matrix,
                coords,
                line_width,
                t_start,
                t_end,
                colour,
                tessellation,
                uvm,
            )
        } else {
            Err(Error::VM("no matrix for render_bezier_bulging".to_string()))
        }
    }

    pub fn render_stroked_bezier(
        &mut self,
        tessellation: usize,
        coords: &[f32; 8],
        stroke_tessellation: usize,
        stroke_noise: f32,
        stroke_line_width_start: f32,
        stroke_line_width_end: f32,
        colour: &Colour,
        colour_volatility: f32,
        seed: f32,
        mapping: Easing,
        brush_type: BrushType,
        brush_subtype: usize,
    ) -> Result<()> {
        if let Some(matrix) = self.matrix_stack.peek() {
            let uvm = self.mappings.get_uv_mapping(brush_type, brush_subtype);

            self.geometry.render_stroked_bezier(
                matrix,
                tessellation,
                coords,
                stroke_tessellation,
                stroke_noise,
                stroke_line_width_start,
                stroke_line_width_end,
                colour,
                colour_volatility,
                seed,
                mapping,
                uvm,
            )
        } else {
            Err(Error::VM("no matrix for render_stroked_bezier".to_string()))
        }
    }

    pub fn render_stroked_bezier_rect(
        &mut self,
        position: (f32, f32),
        width: f32,
        height: f32,
        volatility: f32,
        overlap: f32,
        iterations: f32,
        seed: i32,
        tessellation: usize,
        stroke_tessellation: usize,
        stroke_noise: f32,
        colour: &Colour,
        colour_volatility: f32,
        brush_type: BrushType,
        brush_subtype: usize,
    ) -> Result<()> {
        if let Some(matrix) = self.matrix_stack.peek() {
            let uvm = self.mappings.get_uv_mapping(brush_type, brush_subtype);

            self.geometry.render_stroked_bezier_rect(
                matrix,
                position,
                width,
                height,
                volatility,
                overlap,
                iterations,
                seed,
                tessellation,
                stroke_tessellation,
                stroke_noise,
                colour,
                colour_volatility,
                uvm,
            )
        } else {
            Err(Error::VM(
                "no matrix for render_stroked_bezier_rect".to_string(),
            ))
        }
    }

    pub fn set_prng_state(&mut self, prng: PrngStateStruct) {
        self.prng_state = prng;
    }

    pub fn get_render_packet_geo_len(&self, packet_number: usize) -> usize {
        self.geometry.get_render_packet_geo_len(packet_number)
    }

    pub fn get_render_packet_geo_ptr(&self, packet_number: usize) -> *const f32 {
        self.geometry.get_render_packet_geo_ptr(packet_number)
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

        self.matrix_stack.reset();
        self.geometry.reset();

        // todo
        // vm->building_with_trait_within_vector = 0;
        // vm->trait_within_vector_index         = 0;
    }

    pub fn function_call_default_arguments(
        &mut self,
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

        self.interpret(program)?;

        Ok(())
    }

    pub fn function_call_body(&mut self, program: &Program, fn_info: &FnInfo) -> Result<()> {
        // push a frame onto the stack whose return address is the program's STOP
        // instruction
        let stop_address = program.stop_location();

        // set the correct return ip
        self.stack[self.fp + FP_OFFSET_TO_IP] = Var::Int(stop_address as i32);

        // leap to a location
        self.ip = fn_info.body_address;

        self.interpret(program)?;

        // the above vm_interpret will eventually hit a RET, pop the frame,
        // push the function's result onto the stack and then jump to the stop_address
        // so we'll need to pop that function's return value off the stack
        self.sp = self.sp_dec()?;

        Ok(())
    }

    fn arg_memory_from_iname(
        &self,
        fn_info: &FnInfo,
        iname: Name,
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

    pub fn function_set_argument_to_f32(&mut self, fn_info: &FnInfo, iname: Name, f: f32) {
        if let Some(offset) = self.arg_memory_from_iname(fn_info, iname, self.fp - 1) {
            self.stack[offset] = Var::Float(f);
        }
    }

    pub fn function_set_argument_to_2d(&mut self, fn_info: &FnInfo, iname: Name, x: f32, y: f32) {
        if let Some(offset) = self.arg_memory_from_iname(fn_info, iname, self.fp - 1) {
            self.stack[offset] = Var::V2D(x, y);
        }
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
                    if let Var::Int(hop_back) = self.stack[fp + FP_OFFSET_TO_HOP_BACK] {
                        for _ in 0..hop_back {
                            if let Var::Int(prev_fp) = self.stack[fp] {
                                fp = prev_fp as usize; // go back a frame
                            } else {
                                return Err(Error::VM(
                                    "Mem::Argument (hopback) fp is not Var::Int?".to_string(),
                                ));
                            }
                        }
                        if let BytecodeArg::Int(arg1) = bc.arg1 {
                            let src = &self.stack[fp - arg1 as usize - 1];
                            self.stack[self.sp - 1] = src.clone();
                        }
                    } else {
                        return Err(Error::VM("Mem::Argument: fp is not Var::Int?".to_string()));
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
                                return Err(Error::VM(
                                    "Mem::Local (hopback): fp is not Var::Int?".to_string(),
                                ));
                            }
                        }
                        let local = fp + FP_OFFSET_TO_LOCALS; // get the correct frame's local

                        if let BytecodeArg::Int(offset) = bc.arg1 {
                            let src = &self.stack[local + offset as usize];
                            self.stack[self.sp - 1] = src.clone();
                        }
                    } else {
                        return Err(Error::VM("Mem::Local: fp is not Var::Int?".to_string()));
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
                let local = self.fp + FP_OFFSET_TO_LOCALS; // get the correct frame's local

                if let BytecodeArg::Int(offset) = bc.arg1 {
                    self.stack[local + offset as usize] = popped.clone();
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
                )));
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

    fn opcode_native(&mut self, program: &Program, bc: &Bytecode) -> Result<()> {
        let num_args = if let BytecodeArg::Int(num_args_) = bc.arg1 {
            num_args_ as usize
        } else {
            return Err(Error::VM(
                "opcode native requires arg1 to be num_args".to_string(),
            ));
        };

        let res = if let BytecodeArg::Native(native) = bc.arg0 {
            execute_native(self, program, native)?
        } else {
            return Err(Error::VM("opcode_native".to_string()));
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
                return Err(Error::VM(format!(
                    "opcode_store_f unknown memory type: {}",
                    mem
                )));
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
                return Err(Error::VM(
                    "opcode_lt expected float at top-1 of stack".to_string(),
                ));
            }
        } else {
            return Err(Error::VM(
                "opcode_lt expected float at top of stack".to_string(),
            ));
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
        // println!("opcode_jump_if {}", self.sp);
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

    fn opcode_mtx_push(&mut self) -> Result<()> {
        self.matrix_stack.push();
        Ok(())
    }

    fn opcode_mtx_pop(&mut self) -> Result<()> {
        self.matrix_stack.pop();
        Ok(())
    }

    // executes a program on a vm
    // returns Ok if we reached a STOP opcode
    pub fn interpret(&mut self, program: &Program) -> Result<()> {
        // sp == next free stack index
        // do sp_inc or sp_dec before accessing values as these funcs do sanity checks
        // means that a pop (via sp_dec) can reference stack[sp]
        // and that a push (via sp_inc) requires stack[sp-1]
        let mut bc;

        loop {
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
                Opcode::MTX_PUSH => self.opcode_mtx_push()?,
                Opcode::MTX_POP => self.opcode_mtx_pop()?,
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

pub trait StackPeek<T> {
    fn stack_peek(&self, offset: usize) -> Result<T>;
}

impl StackPeek<Keyword> for Vm {
    fn stack_peek(&self, offset: usize) -> Result<Keyword> {
        if let Var::Keyword(kw) = &self.stack[self.sp - offset] {
            Ok(*kw)
        } else {
            Err(Error::VM("stack_peek expected Var::Keyword".to_string()))
        }
    }
}

impl StackPeek<i32> for Vm {
    fn stack_peek(&self, offset: usize) -> Result<i32> {
        if let Var::Int(i) = &self.stack[self.sp - offset] {
            Ok(*i)
        } else {
            Err(Error::VM("stack_peek expected Var::Int".to_string()))
        }
    }
}

impl StackPeek<f32> for Vm {
    fn stack_peek(&self, offset: usize) -> Result<f32> {
        if let Var::Float(f) = &self.stack[self.sp - offset] {
            Ok(*f)
        } else {
            Err(Error::VM("stack_peek expected Var::Float".to_string()))
        }
    }
}

impl StackPeek<usize> for Vm {
    fn stack_peek(&self, offset: usize) -> Result<usize> {
        if let Var::Float(f) = &self.stack[self.sp - offset] {
            Ok(*f as usize)
        } else {
            Err(Error::VM("stack_peek expected Var::Float".to_string()))
        }
    }
}

impl StackPeek<(f32, f32)> for Vm {
    fn stack_peek(&self, offset: usize) -> Result<(f32, f32)> {
        if let Var::V2D(x, y) = &self.stack[self.sp - offset] {
            Ok((*x, *y))
        } else {
            Err(Error::VM("stack_peek expected Var::V2D".to_string()))
        }
    }
}

impl StackPeek<Colour> for Vm {
    fn stack_peek(&self, offset: usize) -> Result<Colour> {
        if let Var::Colour(col) = &self.stack[self.sp - offset] {
            Ok(*col)
        } else {
            Err(Error::VM("stack_peek expected Var::Colour".to_string()))
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::compiler::compile_program;
    use crate::parser::parse;

    pub fn vm_run(vm: &mut Vm, s: &str) {
        let (ast, _word_lut) = parse(s).unwrap();
        let program = compile_program(&ast).unwrap();

        vm.reset();
        vm.interpret(&program).unwrap();
    }

    pub fn vm_exec(mut vm: &mut Vm, s: &str) -> Var {
        vm_run(&mut vm, s);
        vm.top_stack_value().unwrap()
    }

    pub fn is_float(s: &str, val: f32) {
        let mut vm = Vm::new();
        if let Var::Float(f) = vm_exec(&mut vm, s) {
            assert_eq!(f, val)
        }
    }

    pub fn is_int(s: &str, val: i32) {
        let mut vm = Vm::new();
        if let Var::Int(i) = vm_exec(&mut vm, s) {
            assert_eq!(i, val)
        }
    }

    pub fn is_bool(s: &str, val: bool) {
        let mut vm = Vm::new();
        if let Var::Bool(b) = vm_exec(&mut vm, s) {
            assert_eq!(b, val)
        }
    }

    pub fn is_vec_of_f32(s: &str, val: Vec<f32>) {
        let mut vm = Vm::new();
        if let Var::Vector(vec_vec) = vm_exec(&mut vm, s) {
            assert_eq!(vec_vec.len(), val.len());
            for (i, f) in val.iter().enumerate() {
                if let Some(Var::Float(ff)) = vec_vec.get(i) {
                    assert_eq!(ff, f);
                }
            }
        }
    }

    pub fn is_debug_str(s: &str, val: &str) {
        let mut vm = Vm::new();
        vm_exec(&mut vm, s);
        assert_eq!(vm.debug_str, val);
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

    fn pack_compare(var: Var, expected: &str) {
        let mut res: String = "".to_string();
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

    fn unpack_compare_var_name(inp: &str, expected_val: Name, expected_rem: &str) {
        let (res, actual_rem) = Var::unpack(inp).unwrap();

        if let Var::Name(actual_val) = res {
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
        pack_compare(Var::Name(Name::new(65)), "NAME 65");
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

        unpack_compare_var_name("NAME 42", Name::new(42), "");
        unpack_compare_var_name("NAME 42 ", Name::new(42), " ");
        unpack_compare_var_name("NAME 42 shabba", Name::new(42), " shabba");

        unpack_compare_var_v2d("2D 1.23 4.56", 1.23, 4.56, "");
    }
}
