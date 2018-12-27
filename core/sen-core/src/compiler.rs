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

use std::collections::HashMap;
use std::fmt;

use crate::error::{Error, Result};
use crate::keywords::{keyword_to_string, string_to_keyword_hash, Keyword};
use crate::opcodes::{opcode_stack_offset, Opcode};
use crate::parser::Node;

const TAU: f32 = 6.283_185_307_179_586; // todo: move TAU to math

const MEMORY_LOCAL_SIZE: usize = 40;

pub fn compile_preamble() -> Result<Vec<Bytecode>> {
    let mut compilation = Compilation::new();
    let compiler = Compiler::new();
    compiler.register_top_level_preamble(&mut compilation)?;
    compiler.compile_preamble(&mut compilation)?;

    Ok(compilation.code)
}

pub fn compile_program(complete_ast: &[Node]) -> Result<Program> {
    let mut compilation = Compilation::new();
    let compiler = Compiler::new();

    // clean the complete_ast of whitespace and comment nodes
    //
    let mut ast: Vec<Node> = Vec::new();
    for n in complete_ast.iter() {
        if let Some(useful_node) = clean_node(n) {
            ast.push(useful_node);
        }
    }

    compiler.compile_common(&mut compilation, &ast)?;

    Ok(Program::new(compilation.code, compilation.fn_info))
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Mem {
    Argument = 0, // store the function's arguments
    Local = 1,    // store the function's local arguments
    Global = 2,   // global variables shared by all functions
    Constant = 3, // pseudo-segment holds constants in range 0..32767
    Void = 4,     // nothing
}

impl fmt::Display for Mem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mem::Argument => write!(f, "ARG"),
            Mem::Local => write!(f, "LOCAL"),
            Mem::Global => write!(f, "GLOBAL"),
            Mem::Constant => write!(f, "CONST"),
            Mem::Void => write!(f, "VOID"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ColourFormat {
    Rgba,
}

impl fmt::Display for ColourFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ColourFormat::Rgba => write!(f, "rgba"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BytecodeArg {
    Int(i32),
    Float(f32),
    Name(i32),
    Mem(Mem),
    Colour(ColourFormat, f32, f32, f32, f32),
}

impl fmt::Display for BytecodeArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BytecodeArg::Int(i) => write!(f, "{}", i),
            BytecodeArg::Float(s) => write!(f, "{:.2}", s),
            BytecodeArg::Name(i) => write!(f, "Name({})", i),
            BytecodeArg::Mem(m) => write!(f, "{}", m),
            BytecodeArg::Colour(s, a, b, c, d) => write!(f, "{}({} {} {} {})", s, a, b, c, d),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Bytecode {
    pub op: Opcode,
    pub arg0: BytecodeArg,
    pub arg1: BytecodeArg,
}

impl fmt::Display for Bytecode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.op {
            Opcode::LOAD | Opcode::STORE | Opcode::STORE_F => {
                write!(f, "{}\t{}\t{}", self.op, self.arg0, self.arg1)?;
            }
            Opcode::JUMP | Opcode::JUMP_IF => {
                if let BytecodeArg::Int(i) = self.arg0 {
                    if i > 0 {
                        write!(f, "{}\t+{}", self.op, self.arg0)?
                    } else {
                        write!(f, "{}\t{}", self.op, self.arg0)?
                    }
                }
            }
            // todo: format NATIVE
            Opcode::NATIVE => write!(f, "{}\t{}\t{}", self.op, self.arg0, self.arg1)?,
            // todo: format PILE
            Opcode::PILE => write!(f, "{}\t{}\t{}", self.op, self.arg0, self.arg1)?,
            _ => write!(f, "{}", self.op)?,
        };
        Ok(())
    }
}

#[derive(Debug)]
pub struct FnInfo {
    pub fn_name: String,
    pub arg_address: usize,
    pub body_address: usize,
    pub num_args: i32,
    pub argument_offsets: Vec<i32>,
}

impl FnInfo {
    fn new(fn_name: String) -> FnInfo {
        FnInfo {
            fn_name,
            arg_address: 0,
            body_address: 0,
            num_args: 0,
            argument_offsets: Vec::new(),
        }
    }

    fn get_argument_mapping(&self, argument_iname: i32) -> Option<usize> {
        for (i, arg) in self.argument_offsets.iter().enumerate() {
            if *arg == argument_iname {
                return Some((i * 2) + 1);
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct Program {
    pub code: Vec<Bytecode>,
    pub fn_info: Vec<FnInfo>,
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, b) in self.code.iter().enumerate() {
            writeln!(f, "{}\t{}", i, b)?;
        }
        Ok(())
    }
}

impl Program {
    fn new(code: Vec<Bytecode>, fn_info: Vec<FnInfo>) -> Self {
        Program { code, fn_info }
    }
}

#[derive(Debug)]
struct Compilation {
    code: Vec<Bytecode>,

    fn_info: Vec<FnInfo>,
    current_fn_info_index: Option<usize>,
    opcode_offset: i32,

    global_mappings: HashMap<String, i32>,
    global_mapping_marker: i32,

    local_mappings: HashMap<String, i32>,
    local_mapping_marker: i32, // todo: check that it is < MEMORY_LOCAL_SIZE, as that constant is used in the interpreter
}

impl fmt::Display for Compilation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, b) in self.code.iter().enumerate() {
            writeln!(f, "{}\t{}", i, b)?;
        }
        Ok(())
    }
}

impl Compilation {
    fn new() -> Self {
        Compilation {
            code: Vec::new(),

            fn_info: Vec::new(),
            current_fn_info_index: None,
            opcode_offset: 0,

            global_mappings: HashMap::new(),
            global_mapping_marker: 0,

            local_mappings: HashMap::new(),
            local_mapping_marker: 0,
        }
    }

    fn clear_global_mappings(&mut self) -> Result<()> {
        self.global_mappings.clear();
        self.global_mapping_marker = 0;
        Ok(())
    }

    fn add_global_mapping(&mut self, s: String) -> Result<i32> {
        self.global_mappings.insert(s, self.global_mapping_marker);
        self.global_mapping_marker += 1;
        Ok(self.global_mapping_marker - 1)
    }

    fn clear_local_mappings(&mut self) -> Result<()> {
        self.local_mappings.clear();
        self.local_mapping_marker = 0;
        Ok(())
    }

    fn add_local_mapping(&mut self, s: String) -> Result<i32> {
        self.local_mappings.insert(s, self.local_mapping_marker);
        self.local_mapping_marker += 1;
        Ok(self.local_mapping_marker - 1)
    }

    // we want a local mapping that's going to be used to store an internal variable
    // (e.g. during a fence loop)
    // note: it's up to the caller to manage this reference
    fn add_internal_local_mapping(&mut self) -> Result<i32> {
        let s = "internal_local_mapping".to_string();
        self.local_mappings.insert(s, self.local_mapping_marker);
        self.local_mapping_marker += 1;
        Ok(self.local_mapping_marker - 1)
    }

    fn add_bytecode(&mut self, bc: Bytecode) -> Result<()> {
        self.code.push(bc);
        Ok(())
    }

    fn get_fn_info_index(&self, node: &Node) -> Option<usize> {
        if let Node::Name(text, _, _) = node {
            for (i, fi) in self.fn_info.iter().enumerate() {
                if fi.fn_name == *text {
                    return Some(i);
                }
            }
        }
        None
    }

    fn emit_opcode(&mut self, op: Opcode) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Int(0),
            arg1: BytecodeArg::Int(0),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }

    fn emit_opcode_i32_i32(&mut self, op: Opcode, arg0: i32, arg1: i32) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Int(arg0),
            arg1: BytecodeArg::Int(arg1),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }

    fn emit_opcode_mem_i32(&mut self, op: Opcode, arg0: Mem, arg1: i32) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Mem(arg0),
            arg1: BytecodeArg::Int(arg1),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }

    fn emit_opcode_mem_name(&mut self, op: Opcode, arg0: Mem, arg1: i32) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Mem(arg0),
            arg1: BytecodeArg::Name(arg1),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }

    fn emit_opcode_mem_f32(&mut self, op: Opcode, arg0: Mem, arg1: f32) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Mem(arg0),
            arg1: BytecodeArg::Float(arg1),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }

    fn emit_opcode_mem_rgba(
        &mut self,
        op: Opcode,
        arg0: Mem,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Mem(arg0),
            arg1: BytecodeArg::Colour(ColourFormat::Rgba, r, g, b, a),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }

    fn emit_opcode_i32_f32(&mut self, op: Opcode, arg0: i32, arg1: f32) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Int(arg0),
            arg1: BytecodeArg::Float(arg1),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }

    fn emit_opcode_i32_name(&mut self, op: Opcode, arg0: i32, arg1: i32) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Int(arg0),
            arg1: BytecodeArg::Name(arg1),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }

    fn emit_opcode_i32_rgba(
        &mut self,
        op: Opcode,
        arg0: i32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Int(arg0),
            arg1: BytecodeArg::Colour(ColourFormat::Rgba, r, g, b, a),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }

    fn bytecode_modify(&mut self, index: usize, op: Opcode, arg0: i32, arg1: i32) -> Result<()> {
        self.code[index] = Bytecode {
            op,
            arg0: BytecodeArg::Int(arg0),
            arg1: BytecodeArg::Int(arg1),
        };

        Ok(())
    }

    fn bytecode_modify_mem(
        &mut self,
        index: usize,
        op: Opcode,
        arg0: Mem,
        arg1: i32,
    ) -> Result<()> {
        self.code[index] = Bytecode {
            op,
            arg0: BytecodeArg::Mem(arg0),
            arg1: BytecodeArg::Int(arg1),
        };

        Ok(())
    }

    fn bytecode_modify_arg0_i32(&mut self, index: usize, arg0: i32) -> Result<()> {
        let arg1 = self.code[index].arg1;
        let op = self.code[index].op;

        self.code[index] = Bytecode {
            op,
            arg0: BytecodeArg::Int(arg0),
            arg1,
        };

        Ok(())
    }

    fn bytecode_modify_arg1_i32(&mut self, index: usize, arg1: i32) -> Result<()> {
        let arg0 = self.code[index].arg0;
        let op = self.code[index].op;

        self.code[index] = Bytecode {
            op,
            arg0,
            arg1: BytecodeArg::Int(arg1),
        };

        Ok(())
    }
}

struct Compiler {
    string_to_keyword: HashMap<String, Keyword>,
}

impl Compiler {
    fn new() -> Self {
        Compiler {
            string_to_keyword: string_to_keyword_hash(),
        }
    }

    fn correct_function_addresses(&self, compilation: &mut Compilation) -> Result<()> {
        let mut all_fixes: Vec<(usize, Opcode, Mem, i32)> = Vec::new(); // tuple of index, op, arg0, arg1
        let mut arg1_fixes: Vec<(usize, i32)> = Vec::new(); // tuple of index,value pairs

        // go through the bytecode fixing up function call addresses
        for (i, bc) in compilation.code.iter().enumerate() {
            // replace the temporarily stored index in the args of CALL and CALL_0 with
            // the actual values

            match bc.op {
                Opcode::CALL => {
                    if let BytecodeArg::Int(fn_info_index) = bc.arg0 {
                        let fn_info = &compilation.fn_info[fn_info_index as usize];

                        // the previous two bytecodes will be LOADs of CONST.
                        // i - 2 == the address to call
                        // i - 1 == the number of arguments used by the function
                        arg1_fixes.push((i - 2, fn_info.arg_address as i32));
                        arg1_fixes.push((i - 1, fn_info.num_args as i32));
                    }
                }
                Opcode::CALL_0 => {
                    if let BytecodeArg::Int(fn_info_index) = bc.arg0 {
                        let fn_info = &compilation.fn_info[fn_info_index as usize];
                        arg1_fixes.push((i - 1, fn_info.body_address as i32));
                    }
                }
                Opcode::PLACEHOLDER_STORE => {
                    // opcode's arg0 is the fn_info_index and arg1 is the label_value
                    if let BytecodeArg::Int(fn_info_index) = bc.arg0 {
                        let fn_info = &compilation.fn_info[fn_info_index as usize];
                        if let BytecodeArg::Int(label_value) = bc.arg1 {
                            if let Some(data_index) = fn_info.get_argument_mapping(label_value) {
                                all_fixes.push((
                                    i,
                                    Opcode::STORE,
                                    Mem::Argument,
                                    data_index as i32,
                                ));
                            } else {
                                all_fixes.push((i, Opcode::STORE, Mem::Void, 0));
                            }
                        }
                    }
                }
                _ => (),
            }
        }

        for (index, op, arg0, arg1) in all_fixes {
            compilation.bytecode_modify_mem(index, op, arg0, arg1)?;
        }
        for (index, arg1) in arg1_fixes {
            compilation.bytecode_modify_arg1_i32(index, arg1)?;
        }

        Ok(())
    }

    fn register_top_level_preamble(&self, compilation: &mut Compilation) -> Result<()> {
        compilation.add_global_mapping(keyword_to_string(Keyword::GenInitial))?;

        compilation.add_global_mapping(keyword_to_string(Keyword::CanvasWidth))?;
        compilation.add_global_mapping(keyword_to_string(Keyword::CanvasHeight))?;

        compilation.add_global_mapping(keyword_to_string(Keyword::MathTau))?;

        compilation.add_global_mapping(keyword_to_string(Keyword::White))?;
        compilation.add_global_mapping(keyword_to_string(Keyword::Black))?;
        compilation.add_global_mapping(keyword_to_string(Keyword::Red))?;
        compilation.add_global_mapping(keyword_to_string(Keyword::Green))?;
        compilation.add_global_mapping(keyword_to_string(Keyword::Blue))?;
        compilation.add_global_mapping(keyword_to_string(Keyword::Yellow))?;
        compilation.add_global_mapping(keyword_to_string(Keyword::Magenta))?;
        compilation.add_global_mapping(keyword_to_string(Keyword::Cyan))?;

        compilation.add_global_mapping(keyword_to_string(Keyword::ColProceduralFnPresets))?;
        compilation.add_global_mapping(keyword_to_string(Keyword::EasePresets))?;

        Ok(())
    }

    fn register_top_level_fns(&self, compilation: &mut Compilation, ast: &[Node]) -> Result<()> {
        // clear all data
        compilation.fn_info = Vec::new();

        // register top level fns
        for n in ast.iter() {
            if self.is_list_beginning_with(n, Keyword::Fn) {
                // get the name of the fn
                if let Node::List(nodes, _) = n {
                    if nodes.len() < 2 {
                        // a list with just the 'fn' keyword ???
                        return Err(Error::Compiler("malformed function definition".to_string()));
                    }
                    let name_and_params = &nodes[1];
                    if let Node::List(np_nodes, _) = name_and_params {
                        if !np_nodes.is_empty() {
                            let name_node = &np_nodes[0];
                            if let Node::Name(text, _, _) = name_node {
                                // we have a named top-level fn declaration
                                //
                                // create and add a top level fn
                                let fn_info = FnInfo::new(text.to_string());
                                compilation.fn_info.push(fn_info);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn register_names_in_define(&self, compilation: &mut Compilation, lhs: &Node) -> Result<()> {
        match lhs {
            Node::Name(name, _, _) => {
                // (define foo 42)
                let s = name.to_string();
                // todo: is this check necessary?
                if let Some(_i) = compilation.global_mappings.get(name) {
                    // name was already added to global_mappings
                    return Ok(());
                }

                if let Err(e) = compilation.add_global_mapping(s) {
                    return Err(e);
                }
            }
            Node::List(nodes, _) | Node::Vector(nodes, _) => {
                // (define [a b] (something))
                // (define [a [x y]] (something))
                for n in nodes.iter() {
                    if let Err(e) = self.register_names_in_define(compilation, n) {
                        return Err(e);
                    }
                }
            }
            _ => (),
        }
        Ok(())
    }

    fn register_top_level_defines(
        &self,
        compilation: &mut Compilation,
        ast: &[Node],
    ) -> Result<()> {
        let define_keyword_string = keyword_to_string(Keyword::Define);

        for n in ast.iter() {
            if let Node::List(nodes, _) = n {
                if !nodes.is_empty() {
                    let define_keyword = &nodes[0];
                    if let Node::Name(text, _, _) = define_keyword {
                        if text == &define_keyword_string {
                            let mut defs = &nodes[1..];
                            while defs.len() > 1 {
                                if let Err(e) = self.register_names_in_define(compilation, &defs[0])
                                {
                                    return Err(e);
                                }
                                defs = &defs[2..];
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn compile_preamble(&self, compilation: &mut Compilation) -> Result<()> {
        // ********************************************************************************
        // NOTE: each entry should have a corresponding entry in
        // register_top_level_preamble
        // ********************************************************************************
        self.compile_global_bind_i32(compilation, keyword_to_string(Keyword::GenInitial), 0)?;

        self.compile_global_bind_f32(compilation, keyword_to_string(Keyword::CanvasWidth), 1000.0)?;
        self.compile_global_bind_f32(
            compilation,
            keyword_to_string(Keyword::CanvasHeight),
            1000.0,
        )?;

        self.compile_global_bind_f32(compilation, keyword_to_string(Keyword::MathTau), TAU)?;

        self.compile_global_bind_col(
            compilation,
            keyword_to_string(Keyword::White),
            1.0,
            1.0,
            1.0,
            1.0,
        )?;
        self.compile_global_bind_col(
            compilation,
            keyword_to_string(Keyword::Black),
            0.0,
            0.0,
            0.0,
            1.0,
        )?;
        self.compile_global_bind_col(
            compilation,
            keyword_to_string(Keyword::Red),
            1.0,
            0.0,
            0.0,
            1.0,
        )?;
        self.compile_global_bind_col(
            compilation,
            keyword_to_string(Keyword::Green),
            0.0,
            1.0,
            0.0,
            1.0,
        )?;
        self.compile_global_bind_col(
            compilation,
            keyword_to_string(Keyword::Blue),
            0.0,
            0.0,
            1.0,
            1.0,
        )?;
        self.compile_global_bind_col(
            compilation,
            keyword_to_string(Keyword::Yellow),
            1.0,
            1.0,
            0.0,
            1.0,
        )?;
        self.compile_global_bind_col(
            compilation,
            keyword_to_string(Keyword::Magenta),
            1.0,
            0.0,
            1.0,
            1.0,
        )?;
        self.compile_global_bind_col(
            compilation,
            keyword_to_string(Keyword::Cyan),
            0.0,
            1.0,
            1.0,
            1.0,
        )?;

        self.compile_global_bind_procedural_presets(compilation)?;
        self.compile_global_bind_ease_presets(compilation)?;

        // ********************************************************************************
        // NOTE: each entry should have a corresponding entry in
        // register_top_level_preamble
        // ********************************************************************************

        // slap a stop onto the end of this compilation
        compilation.emit_opcode(Opcode::STOP)?;

        Ok(())
    }

    fn compile_global_bind_procedural_presets(&self, compilation: &mut Compilation) -> Result<()> {
        // create a vector
        compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Void, 0)?;

        // append the names
        self.append_keyword(compilation, Keyword::Chrome)?;
        self.append_keyword(compilation, Keyword::HotlineMiami)?;
        self.append_keyword(compilation, Keyword::KnightRider)?;
        self.append_keyword(compilation, Keyword::Mars)?;
        self.append_keyword(compilation, Keyword::Rainbow)?;
        self.append_keyword(compilation, Keyword::Robocop)?;
        self.append_keyword(compilation, Keyword::Transformers)?;

        self.store_globally(
            compilation,
            keyword_to_string(Keyword::ColProceduralFnPresets),
        )?;

        Ok(())
    }

    fn compile_global_bind_ease_presets(&self, compilation: &mut Compilation) -> Result<()> {
        // create a vector
        compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Void, 0)?;

        // append the names
        self.append_keyword(compilation, Keyword::Linear)?;
        self.append_keyword(compilation, Keyword::EaseQuick)?;
        self.append_keyword(compilation, Keyword::EaseSlowIn)?;
        self.append_keyword(compilation, Keyword::EaseSlowInOut)?;
        self.append_keyword(compilation, Keyword::EaseQuadraticIn)?;
        self.append_keyword(compilation, Keyword::EaseQuadraticOut)?;
        self.append_keyword(compilation, Keyword::EaseQuadraticInOut)?;
        self.append_keyword(compilation, Keyword::EaseCubicIn)?;
        self.append_keyword(compilation, Keyword::EaseCubicOut)?;
        self.append_keyword(compilation, Keyword::EaseCubicInOut)?;
        self.append_keyword(compilation, Keyword::EaseQuarticIn)?;
        self.append_keyword(compilation, Keyword::EaseQuarticOut)?;
        self.append_keyword(compilation, Keyword::EaseQuarticInOut)?;
        self.append_keyword(compilation, Keyword::EaseQuinticIn)?;
        self.append_keyword(compilation, Keyword::EaseQuinticOut)?;
        self.append_keyword(compilation, Keyword::EaseQuinticInOut)?;
        self.append_keyword(compilation, Keyword::EaseSinIn)?;
        self.append_keyword(compilation, Keyword::EaseSinOut)?;
        self.append_keyword(compilation, Keyword::EaseSinInOut)?;
        self.append_keyword(compilation, Keyword::EaseCircularIn)?;
        self.append_keyword(compilation, Keyword::EaseCircularOut)?;
        self.append_keyword(compilation, Keyword::EaseCircularInOut)?;
        self.append_keyword(compilation, Keyword::EaseExponentialIn)?;
        self.append_keyword(compilation, Keyword::EaseExponentialOut)?;
        self.append_keyword(compilation, Keyword::EaseExponentialInOut)?;
        self.append_keyword(compilation, Keyword::EaseElasticIn)?;
        self.append_keyword(compilation, Keyword::EaseElasticOut)?;
        self.append_keyword(compilation, Keyword::EaseElasticInOut)?;
        self.append_keyword(compilation, Keyword::EaseBackIn)?;
        self.append_keyword(compilation, Keyword::EaseBackOut)?;
        self.append_keyword(compilation, Keyword::EaseBackInOut)?;
        self.append_keyword(compilation, Keyword::EaseBounceIn)?;
        self.append_keyword(compilation, Keyword::EaseBounceOut)?;
        self.append_keyword(compilation, Keyword::EaseBounceInOut)?;

        self.store_globally(compilation, keyword_to_string(Keyword::EasePresets))?;

        Ok(())
    }

    fn compile_common(&self, compilation: &mut Compilation, ast: &[Node]) -> Result<()> {
        self.compile_common_prologue(compilation, ast)?;
        self.compile_common_top_level_fns(compilation, ast)?;
        self.compile_common_top_level_defines(compilation, ast)?;
        self.compile_common_top_level_forms(compilation, ast)?;
        self.compile_common_epilogue(compilation, ast)?;
        Ok(())
    }

    fn compile_common_prologue(&self, compilation: &mut Compilation, ast: &[Node]) -> Result<()> {
        compilation.clear_global_mappings()?;
        compilation.clear_local_mappings()?;
        // compilation->current_fn_info = NULL;

        self.register_top_level_preamble(compilation)?;
        self.register_top_level_fns(compilation, ast)?;
        self.register_top_level_defines(compilation, ast)?;

        Ok(())
    }

    fn compile_common_top_level_fns(
        &self,
        compilation: &mut Compilation,
        ast: &[Node],
    ) -> Result<()> {
        // a placeholder, filled in at the end of this function
        compilation.emit_opcode(Opcode::JUMP)?;
        let start_index = compilation.code.len() - 1;

        // compile the top-level functions
        for n in ast.iter() {
            if self.is_list_beginning_with(n, Keyword::Fn) {
                self.compile(compilation, n)?; // todo: the c-impl returns a node to continue from
            }
        }

        // jump to the compilation's starting address
        let jump_address = compilation.code.len() as i32;
        compilation.bytecode_modify_arg0_i32(start_index, jump_address)?;

        Ok(())
    }

    fn compile_common_top_level_defines(
        &self,
        compilation: &mut Compilation,
        ast: &[Node],
    ) -> Result<()> {
        for n in ast.iter() {
            if self.is_list_beginning_with(n, Keyword::Define) {
                if let Node::List(children, _) = n {
                    self.compile_define(compilation, &children[1..], Mem::Global)?;
                }
            }
        }
        Ok(())
    }

    fn compile_common_top_level_forms(
        &self,
        compilation: &mut Compilation,
        ast: &[Node],
    ) -> Result<()> {
        for n in ast.iter() {
            if !self.is_list_beginning_with(n, Keyword::Define)
                && !self.is_list_beginning_with(n, Keyword::Fn)
            {
                self.compile(compilation, n)?;
            }
        }
        Ok(())
    }

    fn compile_common_epilogue(&self, compilation: &mut Compilation, _ast: &[Node]) -> Result<()> {
        compilation.emit_opcode(Opcode::STOP)?;

        // now update the addreses used by CALL and CALL_0
        self.correct_function_addresses(compilation)?;

        Ok(())
    }

    fn compile(&self, compilation: &mut Compilation, ast: &Node) -> Result<()> {
        // todo: move this out of compile and into the compilation struct

        match ast {
            Node::List(children, _) => self.compile_list(compilation, children)?,
            Node::Float(f, _) => {
                return compilation.emit_opcode_mem_f32(Opcode::LOAD, Mem::Constant, *f)
            }
            Node::Vector(children, _) => {
                if children.len() == 2 {
                    return self.compile_2d(compilation, children);
                } else {
                    return self.compile_vector(compilation, children);
                }
            }
            Node::Name(text, iname, _) => {
                let found_name = self.compile_user_defined_name(compilation, &text, *iname)?;
                if found_name {
                    return Ok(());
                } else {
                    return Err(Error::Compiler(format!(
                        "compile: can't find user defined name: {}",
                        text
                    )));
                }
            }
            _ => return Err(Error::Compiler("compile".to_string())),
        }

        Ok(())
    }

    fn compile_list(&self, compilation: &mut Compilation, children: &[Node]) -> Result<()> {
        if children.is_empty() {
            // should this be an error?
            return Err(Error::Compiler(
                "compile_list no children (should this be an error?)".to_string(),
            ));
        }

        match &children[0] {
            Node::List(kids, _) => self.compile_list(compilation, &kids)?,
            Node::Name(text, iname, _) => {
                if let Some(fn_info_index) = compilation.get_fn_info_index(&children[0]) {
                    // todo: get_fn_info_index is re-checking that this is a Node::Name
                    self.compile_fn_invocation(compilation, &children[1..], fn_info_index)?;
                    return Ok(());
                }

                let found_name = self.compile_user_defined_name(compilation, &text, *iname)?;
                if found_name {
                    return Ok(());
                }

                let mut found_keyword: bool = false;
                let mut keyword: Keyword = Keyword::Define;
                if let Some(kw) = self.string_to_keyword.get(text) {
                    keyword = *kw;
                    found_keyword = true;
                }

                if found_keyword {
                    match keyword {
                        Keyword::Define => {
                            self.compile_define(compilation, &children[1..], Mem::Local)?
                        }
                        Keyword::If => self.compile_if(compilation, &children[1..])?,
                        Keyword::Each => self.compile_each(compilation, &children[1..])?,
                        Keyword::Loop => self.compile_loop(compilation, &children[1..])?,
                        Keyword::Fence => self.compile_fence(compilation, &children[1..])?,
                        Keyword::OnMatrixStack => {
                            self.compile_on_matrix_stack(compilation, &children[1..])?
                        }
                        Keyword::Fn => self.compile_fn(compilation, &children[1..])?,
                        Keyword::Plus => {
                            self.compile_math(compilation, &children[1..], Opcode::ADD)?
                        }
                        Keyword::Minus => {
                            self.compile_math(compilation, &children[1..], Opcode::SUB)?
                        }
                        Keyword::Mult => {
                            self.compile_math(compilation, &children[1..], Opcode::MUL)?
                        }
                        Keyword::Divide => {
                            self.compile_math(compilation, &children[1..], Opcode::DIV)?
                        }
                        Keyword::Mod => {
                            self.compile_math(compilation, &children[1..], Opcode::MOD)?
                        }
                        Keyword::Equal => {
                            self.compile_math(compilation, &children[1..], Opcode::EQ)?
                        }
                        Keyword::Lt => {
                            self.compile_math(compilation, &children[1..], Opcode::LT)?
                        }
                        Keyword::Gt => {
                            self.compile_math(compilation, &children[1..], Opcode::GT)?
                        }
                        Keyword::And => {
                            self.compile_math(compilation, &children[1..], Opcode::AND)?
                        }
                        Keyword::Or => {
                            self.compile_math(compilation, &children[1..], Opcode::OR)?
                        }
                        Keyword::Not => {
                            self.compile_next_one(compilation, &children[1..], Opcode::NOT)?
                        }
                        Keyword::Sqrt => {
                            self.compile_next_one(compilation, &children[1..], Opcode::SQRT)?
                        }
                        Keyword::AddressOf => {
                            self.compile_address_of(compilation, &children[1..])?
                        }
                        Keyword::FnCall => self.compile_fn_call(compilation, &children[1..])?,
                        Keyword::VectorAppend => {
                            self.compile_vector_append(compilation, &children[1..])?
                        }
                        Keyword::Quote => self.compile_quote(compilation, &children[1..])?,
                        _ => {
                            // look up the name as a user defined variable
                            // normally get here when a script contains variables
                            // that have the same name as common parameters.
                            // e.g. r, g, b, alpha
                            // or if we're passing a pre-defined argument value
                            // e.g. linear in (bezier line-width-mapping: linear)

                            // todo: some version of compile_user_defined_name that
                            // also looks at the string_to_keyword hash

                        }
                    }
                }
                // check native api set
            }
            _ => return Err(Error::Compiler("compile_list strange child".to_string())),
        }

        Ok(())
    }

    fn compile_define(
        &self,
        compilation: &mut Compilation,
        children: &[Node],
        mem: Mem,
    ) -> Result<()> {
        // children are an even number of nodes representing binding/value pairs
        // (define a 10 b 20 c 30) -> a 10 b 20 c 30

        let mut defs = children;

        if defs.len() % 2 != 0 {
            // log: should be an even number of elements
            return Err(Error::Compiler(
                "should be an even number of elements".to_string(),
            ));
        }

        while !defs.is_empty() {
            let lhs_node = &defs[0];
            let value_node = &defs[1];

            self.compile(compilation, &value_node)?;

            match lhs_node {
                Node::Name(_, _, _) => {
                    // define foo 10
                    self.store_from_stack_to_memory(compilation, &lhs_node, mem)?;
                }
                Node::Vector(kids, _) => {
                    // define [a b] (something-that-returns-a-vector ...)

                    // check if we can use the PILE opcode
                    if all_children_are_name_nodes(lhs_node) {
                        let num_kids = kids.len();

                        // PILE will stack the elements in the rhs vector in order,
                        // so the lhs values have to be popped in reverse order
                        compilation.emit_opcode_i32_i32(Opcode::PILE, num_kids as i32, 0)?;
                        compilation.opcode_offset = compilation.opcode_offset + num_kids as i32 - 1;

                        for k in kids.iter().rev() {
                            self.store_from_stack_to_memory(compilation, &k, mem)?;
                        }
                    } else {
                        // all nodes in lhs vector definition should be names
                        // note: this means that recursive name assignments aren't implemented
                        // e.g. (define [a [b c]] something)
                        return Err(Error::Compiler(
                            "recursive name assignments aren't implemented".to_string(),
                        ));
                    }
                }
                _ => return Err(Error::Compiler("compile_define".to_string())),
            }

            defs = &defs[2..];
        }

        Ok(())
    }

    fn compile_fence(&self, compilation: &mut Compilation, children: &[Node]) -> Result<()> {
        // (fence (x from: 0 to: 5 num: 5) (+ 42 38))
        if children.len() < 2 {
            return Err(Error::Compiler(
                "compile_fence requires at least 2 forms".to_string(),
            ));
        }

        let parameters_node = &children[0];
        // todo: warn if alterable
        if let Node::List(kids, _) = parameters_node {
            // the looping variable x
            let name_node = &kids[0];

            let mut maybe_from_node: Option<&Node> = None;
            let mut maybe_to_node: Option<&Node> = None;
            let mut maybe_num_node: Option<&Node> = None;

            let mut label_vals = &kids[1..];
            while label_vals.len() > 1 {
                let label = &label_vals[0];
                let value = &label_vals[1];

                if let Node::Label(_, iname, _) = label {
                    if *iname == Keyword::From as i32 {
                        maybe_from_node = Some(&value);
                    } else if *iname == Keyword::To as i32 {
                        maybe_to_node = Some(&value);
                    } else if *iname == Keyword::Num as i32 {
                        maybe_num_node = Some(&value);
                    }
                }

                label_vals = &label_vals[2..];
            }

            // store the quantity
            let num_address = compilation.add_internal_local_mapping()?;
            if let Some(num_node) = maybe_num_node {
                self.compile(compilation, num_node)?;
            } else {
                compilation.emit_opcode_mem_f32(Opcode::LOAD, Mem::Constant, 2.0)?;
            }

            compilation.emit_opcode_mem_i32(Opcode::STORE, Mem::Local, num_address)?;

            // reserve a memory location in local memory for a counter from 0 to quantity
            let counter_address = compilation.add_internal_local_mapping()?;

            compilation.emit_opcode_mem_f32(Opcode::LOAD, Mem::Constant, 0.0)?;
            compilation.emit_opcode_mem_i32(Opcode::STORE, Mem::Local, counter_address)?;

            // delta that needs to be added at every iteration
            //
            // (to - from) / (num - 1)
            if let Some(to_node) = maybe_to_node {
                self.compile(compilation, to_node)?;
            } else {
                // else default to 1
                compilation.emit_opcode_mem_f32(Opcode::LOAD, Mem::Constant, 1.0)?;
            }

            if let Some(from_node) = maybe_from_node {
                self.compile(compilation, from_node)?;
            } else {
                // else default to 0
                compilation.emit_opcode_mem_f32(Opcode::LOAD, Mem::Constant, 0.0)?;
            }

            compilation.emit_opcode(Opcode::SUB)?;

            if let Some(num_node) = maybe_num_node {
                self.compile(compilation, num_node)?;
            } else {
                // else default to 3
                compilation.emit_opcode_mem_f32(Opcode::LOAD, Mem::Constant, 3.0)?;
            }
            compilation.emit_opcode_mem_f32(Opcode::LOAD, Mem::Constant, 1.0)?;
            compilation.emit_opcode(Opcode::SUB)?;
            compilation.emit_opcode(Opcode::DIV)?;
            let delta_address = compilation.add_internal_local_mapping()?;
            compilation.emit_opcode_mem_i32(Opcode::STORE, Mem::Local, delta_address)?;

            // set looping variable x to 'from' value
            if let Some(from_node) = maybe_from_node {
                self.compile(compilation, from_node)?;
            } else {
                // else default to 0
                compilation.emit_opcode_mem_f32(Opcode::LOAD, Mem::Constant, 0.0)?;
            }

            let from_address = compilation.add_internal_local_mapping()?;

            compilation.emit_opcode_mem_i32(Opcode::STORE, Mem::Local, from_address)?;

            // store the starting 'from' value in the locally scoped variable
            compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Local, from_address)?;

            let loop_variable_address =
                self.store_from_stack_to_memory(compilation, name_node, Mem::Local)?;

            // compare looping variable against exit condition
            // and jump if looping variable >= exit value
            let addr_loop_start = compilation.code.len();

            compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Local, counter_address)?;
            compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Local, num_address)?;

            // exit check
            compilation.emit_opcode(Opcode::LT)?;

            let addr_exit_check = compilation.code.len();
            compilation.emit_opcode(Opcode::JUMP_IF)?;

            // looper = from + (counter * delta)
            compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Local, from_address)?;
            compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Local, counter_address)?;
            compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Local, delta_address)?;
            compilation.emit_opcode(Opcode::MUL)?;
            compilation.emit_opcode(Opcode::ADD)?;
            compilation.emit_opcode_mem_i32(Opcode::STORE, Mem::Local, loop_variable_address)?;

            let pre_body_opcode_offset = compilation.opcode_offset;

            // compile the body forms (woooaaaoohhh body form, body form for yoooouuuu)
            self.compile_rest(compilation, &children[1..])?;

            let post_body_opcode_offset = compilation.opcode_offset;
            let opcode_delta = post_body_opcode_offset - pre_body_opcode_offset;

            // pop off any values that the body might leave on the stack
            for _i in 0..opcode_delta {
                compilation.emit_opcode_mem_i32(Opcode::STORE, Mem::Void, 0)?;
            }

            // increment counter
            compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Local, counter_address)?;
            compilation.emit_opcode_mem_f32(Opcode::LOAD, Mem::Constant, 1.0)?;
            compilation.emit_opcode(Opcode::ADD)?;
            compilation.emit_opcode_mem_i32(Opcode::STORE, Mem::Local, counter_address)?;

            // loop back to the comparison
            let mut compilation_len = compilation.code.len() as i32;
            compilation.emit_opcode_i32_i32(
                Opcode::JUMP,
                -(compilation_len - addr_loop_start as i32),
                0,
            )?;

            compilation_len = compilation.code.len() as i32;
            compilation.bytecode_modify_arg0_i32(
                addr_exit_check,
                compilation_len - addr_exit_check as i32,
            )?;
        }
        Ok(())
    }

    fn compile_loop(&self, compilation: &mut Compilation, children: &[Node]) -> Result<()> {
        // (loop (x from: 0 upto: 120 inc: 30) (body))
        // compile_loop children == (x from: 0 upto: 120 inc: 30) (body)
        //
        if children.len() < 2 {
            return Err(Error::Compiler(
                "compile_loop requires at least 2 forms".to_string(),
            ));
        }

        let parameters_node = &children[0];
        // todo: warn if alterable
        if let Node::List(kids, _) = parameters_node {
            // the looping variable y
            let name_node = &kids[0];

            let mut maybe_from_node: Option<&Node> = None;
            let mut maybe_to_node: Option<&Node> = None;
            let mut maybe_upto_node: Option<&Node> = None;
            let mut maybe_increment_node: Option<&Node> = None;

            let mut label_vals = &kids[1..];
            while label_vals.len() > 1 {
                let label = &label_vals[0];
                let value = &label_vals[1];

                if let Node::Label(_, iname, _) = label {
                    if *iname == Keyword::From as i32 {
                        maybe_from_node = Some(&value);
                    } else if *iname == Keyword::To as i32 {
                        maybe_to_node = Some(&value);
                    } else if *iname == Keyword::Upto as i32 {
                        maybe_upto_node = Some(&value);
                    } else if *iname == Keyword::Inc as i32 {
                        maybe_increment_node = Some(&value);
                    }
                }

                label_vals = &label_vals[2..];
            }

            let mut use_to = false;
            if maybe_to_node.is_some() {
                use_to = true;
            } else if maybe_upto_node.is_none() {
                return Err(Error::Compiler(
                    "compile_loop requires either to or upto parameters".to_string(),
                ));
            }

            // set looping variable x to 'from' value
            if let Some(from_node) = maybe_from_node {
                self.compile(compilation, from_node)?;
            } else {
                // else default to 0
                compilation.emit_opcode_mem_f32(Opcode::LOAD, Mem::Constant, 0.0)?;
            }

            let loop_variable_address =
                self.store_from_stack_to_memory(compilation, name_node, Mem::Local)?;

            // compare looping variable against exit condition
            // and jump if looping variable >= exit value
            let addr_loop_start = compilation.code.len();

            compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Local, loop_variable_address)?;

            if use_to {
                // so jump if looping variable >= exit value
                if let Some(to_node) = maybe_to_node {
                    self.compile(compilation, to_node)?;
                    compilation.emit_opcode(Opcode::LT)?;
                }
            } else {
                // so jump if looping variable > exit value
                if let Some(upto_node) = maybe_upto_node {
                    self.compile(compilation, upto_node)?;
                    compilation.emit_opcode(Opcode::GT)?;
                    compilation.emit_opcode(Opcode::NOT)?;
                }
            }

            let addr_exit_check = compilation.code.len();
            compilation.emit_opcode(Opcode::JUMP_IF)?; // bc_exit_check

            let pre_body_opcode_offset = compilation.opcode_offset;

            // compile the body forms (woooaaaoohhh body form, body form for yoooouuuu)
            self.compile_rest(compilation, &children[1..])?;

            let post_body_opcode_offset = compilation.opcode_offset;
            let opcode_delta = post_body_opcode_offset - pre_body_opcode_offset;

            // pop off any values that the body might leave on the stack
            for _i in 0..opcode_delta {
                compilation.emit_opcode_mem_i32(Opcode::STORE, Mem::Void, 0)?;
            }

            // increment the looping variable
            compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Local, loop_variable_address)?;

            if let Some(increment_node) = maybe_increment_node {
                self.compile(compilation, increment_node)?;
            } else {
                compilation.emit_opcode_mem_f32(Opcode::LOAD, Mem::Constant, 1.0)?;
            }

            compilation.emit_opcode(Opcode::ADD)?;
            compilation.emit_opcode_mem_i32(Opcode::STORE, Mem::Local, loop_variable_address)?;
            // loop back to the comparison
            let mut compilation_len = compilation.code.len() as i32;
            compilation.emit_opcode_i32_i32(
                Opcode::JUMP,
                -(compilation_len - addr_loop_start as i32),
                0,
            )?;

            compilation_len = compilation.code.len() as i32;
            compilation.bytecode_modify_arg0_i32(
                addr_exit_check,
                compilation_len - addr_exit_check as i32,
            )?;
        }
        Ok(())
    }

    fn compile_each(&self, compilation: &mut Compilation, children: &[Node]) -> Result<()> {
        // (each (x from: [10 20 30 40 50])
        //       (+ x x))

        if children.len() < 2 {
            return Err(Error::Compiler(
                "compile_each requires at least 2 forms".to_string(),
            ));
        }

        let parameters_node = &children[0];
        if let Node::List(kids, _) = parameters_node {
            // the looping variable x
            let name_node = &kids[0];

            let mut maybe_from_node: Option<&Node> = None;

            let mut label_vals = &kids[1..];
            while label_vals.len() > 1 {
                let label = &label_vals[0];
                let value = &label_vals[1];

                if let Node::Label(_, iname, _) = label {
                    if *iname == Keyword::From as i32 {
                        maybe_from_node = Some(&value);
                    }
                }

                label_vals = &label_vals[2..];
            }

            // set looping variable x to 'from' value
            if let Some(from_node) = maybe_from_node {
                self.compile(compilation, from_node)?;
            } else {
                // todo: ignore this, each should always have a from parameter
                // else default to 0
                compilation.emit_opcode_mem_f32(Opcode::LOAD, Mem::Constant, 0.0)?;
            }

            compilation.emit_opcode(Opcode::VEC_NON_EMPTY)?;
            let addr_exit_check_is_vec = compilation.code.len();
            compilation.emit_opcode(Opcode::JUMP_IF)?;

            compilation.emit_opcode(Opcode::VEC_LOAD_FIRST)?;

            // compare looping variable against exit condition
            // and jump if looping variable >= exit value
            let addr_loop_start = compilation.code.len() as i32;

            let loop_variable_address =
                self.store_from_stack_to_memory(compilation, name_node, Mem::Local)?;

            let pre_body_opcode_offset = compilation.opcode_offset;

            // compile the body forms (woooaaaoohhh body form, body form for yoooouuuu)
            self.compile_rest(compilation, &children[1..])?;

            let post_body_opcode_offset = compilation.opcode_offset;
            let opcode_delta = post_body_opcode_offset - pre_body_opcode_offset;

            // pop off any values that the body might leave on the stack
            for _i in 0..opcode_delta {
                compilation.emit_opcode_mem_i32(Opcode::STORE, Mem::Void, 0)?;
            }

            compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Local, loop_variable_address)?;
            compilation.emit_opcode(Opcode::VEC_HAS_NEXT)?;

            let addr_exit_check = compilation.code.len();

            compilation.emit_opcode(Opcode::JUMP_IF)?;

            compilation.emit_opcode(Opcode::VEC_NEXT)?;

            // loop back to the comparison
            let mut compilation_len = compilation.code.len() as i32;
            compilation.emit_opcode_i32_i32(
                Opcode::JUMP,
                -(compilation_len - addr_loop_start),
                0,
            )?;

            compilation_len = compilation.code.len() as i32;
            compilation.bytecode_modify_arg0_i32(
                addr_exit_check,
                compilation_len - addr_exit_check as i32,
            )?;
            // fill in jump distance for the IS_VEC check
            compilation.bytecode_modify_arg0_i32(
                addr_exit_check_is_vec,
                compilation_len - addr_exit_check_is_vec as i32,
            )?;
        } else {
            return Err(Error::Compiler(
                "compile_each expected a list that defines parameters".to_string(),
            ));
        }
        Ok(())
    }

    fn compile_vector_in_quote(
        &self,
        compilation: &mut Compilation,
        list_node: &Node,
    ) -> Result<()> {
        // pushing from the VOID means creating a new, empty vector
        compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Void, 0)?;

        // todo: warn if alterable

        if let Node::List(children, _) = list_node {
            // slightly hackish
            // if this is a form like: '(red green blue)
            // the compiler should output the names rather than the colours that are
            // actually referenced (compile_user_defined_name would genereate a
            // MEM_SEG_GLOBAL LOAD code)
            //

            for n in children {
                if let Node::Name(_, iname, _) = n {
                    compilation.emit_opcode_mem_name(Opcode::LOAD, Mem::Constant, *iname)?;
                } else {
                    self.compile(compilation, n)?;
                }
                compilation.emit_opcode(Opcode::APPEND)?;
            }
            return Ok(());
        }
        Err(Error::Compiler(
            "compile_vector_in_quote expected a Node::List".to_string(),
        ))
    }

    fn compile_quote(&self, compilation: &mut Compilation, children: &[Node]) -> Result<()> {
        let quoted_form = &children[0];
        match quoted_form {
            Node::List(_, _) => self.compile_vector_in_quote(compilation, quoted_form)?,
            Node::Name(_, iname, _) => {
                compilation.emit_opcode_mem_name(Opcode::LOAD, Mem::Constant, *iname)?
            }
            _ => self.compile(compilation, quoted_form)?,
        }
        Ok(())
    }

    // (++ vector value)
    fn compile_vector_append(
        &self,
        compilation: &mut Compilation,
        children: &[Node],
    ) -> Result<()> {
        if children.len() != 2 {
            return Err(Error::Compiler(
                "compile_vector_append requires 2 args".to_string(),
            ));
        }

        let vector = &children[0];
        self.compile(compilation, vector)?;

        let value = &children[1];
        self.compile(compilation, value)?;

        compilation.emit_opcode(Opcode::APPEND)?;

        if let Node::Name(text, _, _) = vector {
            let mut mem_addr: Option<(Mem, i32)> = None;

            if let Some(address) = compilation.local_mappings.get(text) {
                mem_addr = Some((Mem::Local, *address));
            }
            if let Some(address) = compilation.global_mappings.get(text) {
                mem_addr = Some((Mem::Global, *address));
            }

            if let Some((mem, addr)) = mem_addr {
                compilation.emit_opcode_mem_i32(Opcode::STORE, mem, addr)?;
            }
        }

        Ok(())
    }

    // (fn-call (aj z: 44))
    fn compile_fn_call(&self, compilation: &mut Compilation, children: &[Node]) -> Result<()> {
        // fn_name should be a defined function's name
        // it will be known at compile time

        if let Node::List(kids, _) = &children[0] {
            // todo: warn if alterable

            let fn_info_index = &kids[0];
            // place the fn_info_index onto the stack so that CALL_F can find the function
            // offset and num args
            self.compile(compilation, fn_info_index)?;
            compilation.emit_opcode(Opcode::CALL_F)?;

            // compile the rest of the arguments

            // overwrite the default arguments with the actual arguments given by the fn invocation
            let mut label_vals = &kids[1..];
            while label_vals.len() > 1 {
                let label = &label_vals[0];
                let value = &label_vals[1];

                // push value
                self.compile(compilation, &value)?;

                // push the actual fn_info index so that the _FLU opcode can find it
                self.compile(compilation, fn_info_index)?;

                if let Node::Label(_, iname, _) = label {
                    compilation.emit_opcode_mem_i32(Opcode::STORE_F, Mem::Argument, *iname)?;
                } else {
                    return Err(Error::Compiler(
                        "compile_fn_call: label required".to_string(),
                    ));
                }

                label_vals = &label_vals[2..];
            }

            // place the fn_info_index onto the stack so that CALL_F_0 can find the
            // function's body offset
            self.compile(compilation, fn_info_index)?;
            compilation.emit_opcode(Opcode::CALL_F)?;

            return Ok(());
        }

        Err(Error::Compiler(
            "compile_fn_call should be given a list as the first parameter".to_string(),
        ))
    }

    fn compile_address_of(&self, compilation: &mut Compilation, children: &[Node]) -> Result<()> {
        // fn_name should be a defined function's name, it will be known at compile time
        if let Some(fn_info_index) = compilation.get_fn_info_index(&children[0]) {
            // store the index into compilation->fn_info in the compilation
            compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Constant, fn_info_index as i32)?;
            return Ok(());
        }

        Err(Error::Compiler("compile_address_of".to_string()))
    }

    fn compile_on_matrix_stack(
        &self,
        compilation: &mut Compilation,
        children: &[Node],
    ) -> Result<()> {
        compilation.emit_opcode(Opcode::MTX_LOAD)?;
        self.compile_rest(compilation, children)?;
        compilation.emit_opcode(Opcode::MTX_STORE)?;
        Ok(())
    }

    fn compile_if(&self, compilation: &mut Compilation, children: &[Node]) -> Result<()> {
        let if_node: &Node;
        let then_node: &Node;
        let else_node: Option<&Node>;

        let num_children = children.len();

        if num_children == 2 {
            if_node = &children[0];
            then_node = &children[1];
            else_node = None;
        } else if num_children == 3 {
            if_node = &children[0];
            then_node = &children[1];
            else_node = Some(&children[2]);
        } else {
            return Err(Error::Compiler(format!(
                "if clause requires 2 or 3 forms (given {})",
                num_children
            )));
        }

        self.compile(compilation, if_node)?;

        // insert jump to after the 'then' node if not true
        let addr_jump_then = compilation.code.len();
        compilation.emit_opcode(Opcode::JUMP_IF)?;

        // the offset after the if
        let offset_after_if = compilation.opcode_offset;

        self.compile(compilation, then_node)?;

        let offset_after_then = compilation.opcode_offset;

        if let Some(else_node) = else_node {
            // logically we're now going to go down one of possibly two paths
            // so we can't just continue to add the compilation->opcode_offset since
            // that would result in the offset taking both of the conditional's paths
            compilation.opcode_offset = offset_after_if;

            // insert a bc_jump_else opcode
            let addr_jump_else = compilation.code.len();

            compilation.emit_opcode(Opcode::JUMP)?;

            let addr_jump_then_offset = compilation.code.len() as i32 - addr_jump_then as i32;
            compilation.bytecode_modify_arg0_i32(addr_jump_then, addr_jump_then_offset)?;

            self.compile(compilation, else_node)?;

            let offset_after_else = compilation.opcode_offset;

            if offset_after_then != offset_after_else {
                // is this case actually going to happen?
                // if so we can check which of the two paths has the lower opcode offset
                // and pad out that path by inserting some LOAD CONST 9999 into the
                // compilation
                return Err(Error::Compiler(
                    "different opcode_offsets for the two paths in a conditional".to_string(),
                ));
            }

            let addr_jump_else_offset = compilation.code.len() as i32 - addr_jump_else as i32;
            compilation.bytecode_modify_arg0_i32(addr_jump_else, addr_jump_else_offset)?;
        } else {
            let addr_jump_then_offset = compilation.code.len() as i32 - addr_jump_then as i32;
            compilation.bytecode_modify_arg0_i32(addr_jump_then, addr_jump_then_offset)?;
        }

        Ok(())
    }

    /*
    - invoking code will first CALL into the arg_address to setup the
      default values for all args
    - the fn code will then return back to the invoking code
    - invoking code will then overwrite specific data in arg memory
    - invoking code will then CALL into the body_address
     */
    fn compile_fn(&self, compilation: &mut Compilation, children: &[Node]) -> Result<()> {
        // fn (adder a: 0 b: 0) (+ a b)
        compilation.clear_local_mappings()?;

        let signature = &children[0]; // (addr a: 0 b: 0)
        if let Node::List(kids, _) = signature {
            if kids.is_empty() {
                // no fn name given
                return Err(Error::CompilerFnWithoutName);
            }

            let fn_name = &kids[0];
            if let Some(index) = compilation.get_fn_info_index(&fn_name) {
                compilation.current_fn_info_index = Some(index);

                // -------------
                // the arguments
                // -------------
                let mut updated_fn_info: FnInfo;
                {
                    let fn_info: &FnInfo = &compilation.fn_info[index];
                    updated_fn_info = FnInfo::new(fn_info.fn_name.to_string());
                }

                updated_fn_info.arg_address = compilation.code.len();

                // pairs of label/value declarations
                let mut var_decls = &kids[1..];
                let mut num_args = 0;
                let mut counter = 0;

                if var_decls.len() % 2 != 0 {
                    return Err(Error::Compiler(
                        "fn declaration doesn't have matching arg/value pairs".to_string(),
                    ));
                }

                while !var_decls.is_empty() {
                    let label_node = &var_decls[0];
                    let value_node = &var_decls[1];

                    // get argument mapping
                    if let Node::Label(_, iname, _) = label_node {
                        updated_fn_info.argument_offsets.push(*iname);

                        // if let Some(label_i) = compilation.global_mappings.get(text) {
                        // } else {
                        //     // should be impossible to get here, the global mappings for the
                        //     // fn args should all have been registered in the
                        //     // register_top_level_fns function
                        // }

                        compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Constant, *iname)?;
                    }

                    compilation.emit_opcode_mem_i32(Opcode::STORE, Mem::Argument, counter)?;
                    counter += 1;

                    self.compile(compilation, value_node)?;
                    compilation.emit_opcode_mem_i32(Opcode::STORE, Mem::Argument, counter)?;
                    counter += 1;

                    num_args += 1;
                    var_decls = &var_decls[2..];
                }
                updated_fn_info.num_args = num_args;

                compilation.emit_opcode(Opcode::RET_0)?;

                // --------
                // the body
                // --------

                updated_fn_info.body_address = compilation.code.len();

                compilation.fn_info[index] = updated_fn_info;

                // compile the body forms (woooaaaoohhh body form, body form for yoooouuuu)
                self.compile_rest(compilation, &children[1..])?;

                // Don't need any STORE, MEM_SEG_VOID instructions as the RET will
                // pop the frame and blow the stack
                compilation.emit_opcode(Opcode::RET)?;

                compilation.current_fn_info_index = None;
            } else {
                // todo: implement Display for Node
                // return Err(Error::Compiler(format!("cannot find fn_info for {}", fn_name)))
                return Err(Error::Compiler("cannot find fn_info for node".to_string()));
            }
        } else {
            // first item in fn declaration needs to be a list of function name and args
            return Err(Error::CompilerFnDeclIncomplete);
        }

        Ok(())
    }

    // if (adder a: 10 b: 20) then children == a: 10 b: 20
    fn compile_fn_invocation(
        &self,
        compilation: &mut Compilation,
        children: &[Node],
        fn_info_index: usize,
    ) -> Result<()> {
        // NOTE: CALL and CALL_0 get their function offsets and num args from the
        // stack so add some placeholder LOAD CONST opcodes and fill the CALL, CALL_0
        // with fn_info indexes that can later be used to fill in the LOAD CONST
        // opcodes with their correct offsets doing it this way enables functions to
        // call other functions that are declared later in the script

        // prepare the MEM_SEG_ARGUMENT with default values

        // for the function address
        compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Constant, 666)?;
        // for the num args
        compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Constant, 667)?;

        compilation.emit_opcode_i32_i32(
            Opcode::CALL,
            fn_info_index as i32,
            fn_info_index as i32,
        )?;

        // overwrite the default arguments with the actual arguments given by the fn invocation
        let mut arg_vals = &children[..];
        while arg_vals.len() > 1 {
            let arg = &arg_vals[0];
            if let Node::Label(_, iname, _) = arg {
                let val = &arg_vals[1];
                self.compile(compilation, val)?;
                compilation.emit_opcode_i32_i32(
                    Opcode::PLACEHOLDER_STORE,
                    fn_info_index as i32,
                    *iname,
                )?;
            } else {
                return Err(Error::Compiler("compile_fn_invocation".to_string()));
            }

            arg_vals = &arg_vals[2..];
        }

        // call the body of the function
        compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Constant, 668)?;
        compilation.emit_opcode_i32_i32(
            Opcode::CALL_0,
            fn_info_index as i32,
            fn_info_index as i32,
        )?;

        Ok(())
    }

    fn compile_rest(&self, compilation: &mut Compilation, children: &[Node]) -> Result<()> {
        for n in children {
            self.compile(compilation, n)?;
        }
        Ok(())
    }

    fn compile_next_one(
        &self,
        compilation: &mut Compilation,
        children: &[Node],
        op: Opcode,
    ) -> Result<()> {
        if children.is_empty() {
            return Err(Error::Compiler("compile_next_one".to_string()));
        }

        self.compile(compilation, &children[0])?;
        compilation.emit_opcode(op)?;

        Ok(())
    }

    fn compile_math(
        &self,
        compilation: &mut Compilation,
        children: &[Node],
        op: Opcode,
    ) -> Result<()> {
        for n in children {
            self.compile(compilation, n)?;
        }
        compilation.emit_opcode(op)?;
        Ok(())
    }

    fn compile_2d(&self, compilation: &mut Compilation, children: &[Node]) -> Result<()> {
        for n in children {
            self.compile(compilation, n)?;
        }
        compilation.emit_opcode(Opcode::SQUISH2)?;
        Ok(())
    }

    fn compile_vector(&self, compilation: &mut Compilation, children: &[Node]) -> Result<()> {
        // pushing from the VOID means creating a new, empty vector
        compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Void, 0)?;
        for n in children {
            self.compile(compilation, n)?;
            compilation.emit_opcode(Opcode::APPEND)?;
        }

        Ok(())
    }

    fn compile_global_bind_i32(
        &self,
        compilation: &mut Compilation,
        s: String,
        value: i32,
    ) -> Result<()> {
        compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Constant, value)?;
        self.store_globally(compilation, s)?;
        Ok(())
    }

    fn compile_global_bind_f32(
        &self,
        compilation: &mut Compilation,
        s: String,
        value: f32,
    ) -> Result<()> {
        compilation.emit_opcode_mem_f32(Opcode::LOAD, Mem::Constant, value)?;
        self.store_globally(compilation, s)?;
        Ok(())
    }

    fn compile_global_bind_col(
        &self,
        compilation: &mut Compilation,
        s: String,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) -> Result<()> {
        compilation.emit_opcode_mem_rgba(Opcode::LOAD, Mem::Constant, r, g, b, a)?;
        self.store_globally(compilation, s)?;
        Ok(())
    }

    fn append_keyword(&self, compilation: &mut Compilation, kw: Keyword) -> Result<()> {
        compilation.emit_opcode_mem_name(Opcode::LOAD, Mem::Constant, kw as i32)?;
        compilation.emit_opcode(Opcode::APPEND)?;
        Ok(())
    }

    fn store_locally(&self, compilation: &mut Compilation, s: String) -> Result<i32> {
        let address: i32 = match compilation.local_mappings.get(&s) {
            Some(&local_mapping) => local_mapping, // already storing the binding name
            None => compilation.add_local_mapping(s)?,
        };

        compilation.emit_opcode_mem_i32(Opcode::STORE, Mem::Local, address)?;

        Ok(address)
    }

    fn store_globally(&self, compilation: &mut Compilation, s: String) -> Result<i32> {
        let address: i32 = match compilation.global_mappings.get(&s) {
            Some(&global_mapping) => global_mapping, // already storing the binding name
            None => compilation.add_global_mapping(s)?,
        };

        compilation.emit_opcode_mem_i32(Opcode::STORE, Mem::Global, address)?;

        Ok(address)
    }

    fn store_from_stack_to_memory(
        &self,
        compilation: &mut Compilation,
        node: &Node,
        mem: Mem,
    ) -> Result<i32> {
        if let Node::Name(text, _, _) = node {
            match mem {
                Mem::Local => self.store_locally(compilation, text.to_string()),
                Mem::Global => self.store_globally(compilation, text.to_string()),
                _ => Err(Error::Compiler(
                    "store_from_stack_to_memory invalid memory type".to_string(),
                )),
            }
        } else {
            Err(Error::Compiler("store_from_stack_to_memory".to_string()))
        }
    }

    fn compile_user_defined_name(
        &self,
        compilation: &mut Compilation,
        s: &str,
        iname: i32,
    ) -> Result<bool> {
        if let Some(local_mapping) = compilation.local_mappings.get(s) {
            let val = *local_mapping;
            compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Local, val)?;
            return Ok(true);
        }

        // check arguments if we're in a function
        if let Some(current_fn_info_index) = compilation.current_fn_info_index {
            let maybe_argument_mapping;
            {
                let fn_info = &compilation.fn_info[current_fn_info_index];
                maybe_argument_mapping = fn_info.get_argument_mapping(iname);
            }
            if let Some(argument_mapping) = maybe_argument_mapping {
                compilation.emit_opcode_mem_i32(
                    Opcode::LOAD,
                    Mem::Argument,
                    argument_mapping as i32,
                )?;
                return Ok(true);
            }
        }

        if let Some(global_mapping) = compilation.global_mappings.get(s) {
            let val = *global_mapping;
            compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Global, val)?;
            return Ok(true);
        }

        // // could be a keyword such as linear, ease-in etc
        // if let Some(keyword) = self.string_to_keyword.get(s) {
        //     val = *keyword as i32;
        //     found = true;
        // }
        // if found {
        //     compilation.emit_opcode_mem_i32(Opcode::LOAD, Mem::Constant, val)?;
        //     return Ok(true)
        // }

        // todo: log unknown mapping for s

        Ok(false)
    }

    fn is_list_beginning_with(&self, n: &Node, kw: Keyword) -> bool {
        if let Node::List(nodes, _) = n {
            if !nodes.is_empty() {
                if let Node::Name(ref text, _, _) = nodes[0] {
                    if let Some(name_kw) = self.string_to_keyword.get(text) {
                        return *name_kw == kw;
                    }
                }
            }
        }
        false
    }
}

// renamed all_children_have_type as it's only used with children of type NAME
fn all_children_are_name_nodes(parent: &Node) -> bool {
    match parent {
        Node::List(children, _) | Node::Vector(children, _) => {
            for n in children.iter() {
                if let Node::Name(_, _, _) = n {
                    continue;
                } else {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

fn count_children(parent: &Node) -> Result<usize> {
    match parent {
        Node::List(children, _) | Node::Vector(children, _) => Ok(children.len()),
        _ => Err(Error::Compiler("count_children".to_string())),
    }
}

fn clean_node(node: &Node) -> Option<Node> {
    match node {
        Node::List(nodes, _) => {
            let mut vn: Vec<Node> = Vec::new();
            for n in nodes.iter() {
                if let Some(cleaned) = clean_node(n) {
                    vn.push(cleaned);
                }
            }
            Some(Node::List(vn, None))
        }
        Node::Vector(nodes, _) => {
            let mut vn: Vec<Node> = Vec::new();
            for n in nodes.iter() {
                if let Some(cleaned) = clean_node(n) {
                    vn.push(cleaned);
                }
            }
            Some(Node::Vector(vn, None))
        }
        Node::Float(f, _) => Some(Node::Float(*f, None)),
        Node::Name(text, iname, _) => Some(Node::Name(text.to_string(), *iname, None)),
        Node::Label(text, iname, _) => Some(Node::Label(text.to_string(), *iname, None)),
        Node::String(text, _) => Some(Node::String(text.to_string(), None)),
        Node::Whitespace(_, _) => None,
        Node::Comment(_, _) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    fn compile(s: &str) -> Vec<Bytecode> {
        let (ast, _word_lut) = parse(s).unwrap();
        let program = compile_program(&ast).unwrap();
        program.code
    }

    fn bytecode_from_opcode(op: Opcode) -> Bytecode {
        Bytecode {
            op,
            arg0: BytecodeArg::Int(0),
            arg1: BytecodeArg::Int(0),
        }
    }

    fn add() -> Bytecode {
        bytecode_from_opcode(Opcode::ADD)
    }

    fn append() -> Bytecode {
        bytecode_from_opcode(Opcode::APPEND)
    }

    fn call() -> Bytecode {
        bytecode_from_opcode(Opcode::CALL)
    }

    fn call_0() -> Bytecode {
        bytecode_from_opcode(Opcode::CALL_0)
    }

    fn div() -> Bytecode {
        bytecode_from_opcode(Opcode::DIV)
    }

    fn gt() -> Bytecode {
        bytecode_from_opcode(Opcode::GT)
    }

    fn jump(delta: i32) -> Bytecode {
        Bytecode {
            op: Opcode::JUMP,
            arg0: BytecodeArg::Int(delta),
            arg1: BytecodeArg::Int(0),
        }
    }

    fn jump_if(delta: i32) -> Bytecode {
        Bytecode {
            op: Opcode::JUMP_IF,
            arg0: BytecodeArg::Int(delta),
            arg1: BytecodeArg::Int(0),
        }
    }

    fn load_arg(val: i32) -> Bytecode {
        Bytecode {
            op: Opcode::LOAD,
            arg0: BytecodeArg::Mem(Mem::Argument),
            arg1: BytecodeArg::Int(val),
        }
    }

    fn load_const_f32(val: f32) -> Bytecode {
        Bytecode {
            op: Opcode::LOAD,
            arg0: BytecodeArg::Mem(Mem::Constant),
            arg1: BytecodeArg::Float(val),
        }
    }

    fn load_const_i32(val: i32) -> Bytecode {
        Bytecode {
            op: Opcode::LOAD,
            arg0: BytecodeArg::Mem(Mem::Constant),
            arg1: BytecodeArg::Int(val),
        }
    }

    fn load_global_i32(val: i32) -> Bytecode {
        Bytecode {
            op: Opcode::LOAD,
            arg0: BytecodeArg::Mem(Mem::Global),
            arg1: BytecodeArg::Int(val),
        }
    }

    fn load_local_i32(val: i32) -> Bytecode {
        Bytecode {
            op: Opcode::LOAD,
            arg0: BytecodeArg::Mem(Mem::Local),
            arg1: BytecodeArg::Int(val),
        }
    }

    fn load_void() -> Bytecode {
        Bytecode {
            op: Opcode::LOAD,
            arg0: BytecodeArg::Mem(Mem::Void),
            arg1: BytecodeArg::Int(0),
        }
    }

    fn lt() -> Bytecode {
        bytecode_from_opcode(Opcode::LT)
    }

    fn mul() -> Bytecode {
        bytecode_from_opcode(Opcode::MUL)
    }

    fn not() -> Bytecode {
        bytecode_from_opcode(Opcode::NOT)
    }

    fn ret() -> Bytecode {
        bytecode_from_opcode(Opcode::RET)
    }

    fn ret_0() -> Bytecode {
        bytecode_from_opcode(Opcode::RET_0)
    }

    fn sqrt() -> Bytecode {
        bytecode_from_opcode(Opcode::SQRT)
    }

    fn squish2() -> Bytecode {
        bytecode_from_opcode(Opcode::SQUISH2)
    }

    fn stop() -> Bytecode {
        bytecode_from_opcode(Opcode::STOP)
    }

    fn store_arg(val: i32) -> Bytecode {
        Bytecode {
            op: Opcode::STORE,
            arg0: BytecodeArg::Mem(Mem::Argument),
            arg1: BytecodeArg::Int(val),
        }
    }

    fn store_global(val: i32) -> Bytecode {
        Bytecode {
            op: Opcode::STORE,
            arg0: BytecodeArg::Mem(Mem::Global),
            arg1: BytecodeArg::Int(val),
        }
    }

    fn store_local(val: i32) -> Bytecode {
        Bytecode {
            op: Opcode::STORE,
            arg0: BytecodeArg::Mem(Mem::Local),
            arg1: BytecodeArg::Int(val),
        }
    }

    fn store_void(val: i32) -> Bytecode {
        Bytecode {
            op: Opcode::STORE,
            arg0: BytecodeArg::Mem(Mem::Void),
            arg1: BytecodeArg::Int(val),
        }
    }

    fn sub() -> Bytecode {
        bytecode_from_opcode(Opcode::SUB)
    }

    fn vec_has_next() -> Bytecode {
        bytecode_from_opcode(Opcode::VEC_HAS_NEXT)
    }

    fn vec_load_first() -> Bytecode {
        bytecode_from_opcode(Opcode::VEC_LOAD_FIRST)
    }

    fn vec_next() -> Bytecode {
        bytecode_from_opcode(Opcode::VEC_NEXT)
    }

    fn vec_non_empty() -> Bytecode {
        bytecode_from_opcode(Opcode::VEC_NON_EMPTY)
    }

    #[test]
    fn sanity_check_compile_preamble() {
        // stupid, brittle test just to check that the preamble is creating something
        let preamble = compile_preamble().unwrap();
        assert_eq!(preamble.len(), 111);
    }

    #[test]
    fn test_basics() {
        // f32
        assert_eq!(compile("34"), vec![jump(1), load_const_f32(34.0), stop()]);
        // 2d vector of f32
        assert_eq!(
            compile("[23 45]"),
            vec![
                jump(1),
                load_const_f32(23.0),
                load_const_f32(45.0),
                squish2(),
                stop(),
            ]
        );
        // vector of f32
        assert_eq!(
            compile("[23 45 67 89]"),
            vec![
                jump(1),
                load_void(),
                load_const_f32(23.0),
                append(),
                load_const_f32(45.0),
                append(),
                load_const_f32(67.0),
                append(),
                load_const_f32(89.0),
                append(),
                stop(),
            ]
        );

        assert_eq!(
            compile("(sqrt 144)"),
            vec![jump(1), load_const_f32(144.0), sqrt(), stop(),]
        );

        assert_eq!(
            compile("(define brush 9 b 10)"),
            vec![
                jump(1),
                load_const_f32(9.0),
                store_global(14),
                load_const_f32(10.0),
                store_global(15),
                stop(),
            ]
        );

        assert_eq!(
            compile("(define brush 9 b 10) (+ brush b)"),
            vec![
                jump(1),
                load_const_f32(9.0),
                store_global(14),
                load_const_f32(10.0),
                store_global(15),
                load_global_i32(14),
                load_global_i32(15),
                add(),
                stop(),
            ]
        );
    }

    #[test]
    fn test_fn_declaration() {
        assert_eq!(
            compile("(fn (foo a: 0 b: 0) (+ a b))"),
            vec![
                jump(14),
                load_const_i32(222),
                store_arg(0),
                load_const_f32(0.0),
                store_arg(1),
                load_const_i32(223),
                store_arg(2),
                load_const_f32(0.0),
                store_arg(3),
                ret_0(),
                load_arg(1),
                load_arg(3),
                add(),
                ret(),
                stop()
            ]
        );
    }

    #[test]
    fn test_if() {
        assert_eq!(
            compile("(if (< 3 23) 4 5)"),
            vec![
                jump(1),
                load_const_f32(3.0),
                load_const_f32(23.0),
                lt(),
                jump_if(3),
                load_const_f32(4.00),
                jump(2),
                load_const_f32(5.00),
                stop()
            ]
        );
    }

    #[test]
    fn test_fn_invocation() {
        assert_eq!(
            compile(
                "(fn (adder a: 99 b: 88)
                                (+ a b))
                            (adder a: 3 b: 7)"
            ),
            vec![
                jump(14),
                load_const_i32(222),
                store_arg(0),
                load_const_f32(99.0),
                store_arg(1),
                load_const_i32(223),
                store_arg(2),
                load_const_f32(88.0),
                store_arg(3),
                ret_0(),
                load_arg(1),
                load_arg(3),
                add(),
                ret(),
                load_const_i32(1),
                load_const_i32(2),
                call(),
                load_const_f32(3.0),
                store_arg(1),
                load_const_f32(7.0),
                store_arg(3),
                load_const_i32(10),
                call_0(),
                stop()
            ]
        );
    }

    #[test]
    fn test_each() {
        assert_eq!(
            compile("(define data []) (each (x from: data) (+ x x))"),
            vec![
                jump(1),
                load_void(),
                store_global(14),
                load_global_i32(14),
                vec_non_empty(),
                jump_if(12),
                vec_load_first(),
                store_local(0),
                load_local_i32(0),
                load_local_i32(0),
                add(),
                store_void(0),
                load_local_i32(0),
                vec_has_next(),
                jump_if(3),
                vec_next(),
                jump(-9),
                stop()
            ]
        );

        assert_eq!(
            compile(
                "(fn (add-each by: 4)
                                (define
                                  data [7 8 9]
                                  res [])
                                (each (x from: data) (++ res (+ by x))))
                            (add-each by: 99)"
            ),
            vec![
                jump(33),
                load_const_i32(244),
                store_arg(0),
                load_const_f32(4.0),
                store_arg(1),
                ret_0(),
                load_void(),
                load_const_f32(7.0),
                append(),
                load_const_f32(8.0),
                append(),
                load_const_f32(9.0),
                append(),
                store_local(0),
                load_void(),
                store_local(1),
                load_local_i32(0),
                vec_non_empty(),
                jump_if(14),
                vec_load_first(),
                store_local(2),
                load_local_i32(1),
                load_arg(1),
                load_local_i32(2),
                add(),
                append(),
                store_local(1),
                load_local_i32(2),
                vec_has_next(),
                jump_if(3),
                vec_next(),
                jump(-11),
                ret(),
                load_const_i32(1),
                load_const_i32(1),
                call(),
                load_const_f32(99.0),
                store_arg(1),
                load_const_i32(6),
                call_0(),
                stop()
            ]
        );
    }

    #[test]
    fn test_loop() {
        assert_eq!(
            compile(
                "(loop (y from: 0 upto: 10 inc: 2)
                                  (+ y 3))"
            ),
            vec![
                jump(1),
                load_const_f32(0.0),
                store_local(0),
                load_local_i32(0),
                load_const_f32(10.0),
                gt(),
                not(),
                jump_if(10),
                load_local_i32(0),
                load_const_f32(3.0),
                add(),
                store_void(0),
                load_local_i32(0),
                load_const_f32(2.0),
                add(),
                store_local(0),
                jump(-13),
                stop()
            ]
        );

        assert_eq!(
            compile(
                "(loop (y from: 2 to: 10)
                                  (+ y 45))"
            ),
            vec![
                jump(1),
                load_const_f32(2.0),
                store_local(0),
                load_local_i32(0),
                load_const_f32(10.0),
                lt(),
                jump_if(10),
                load_local_i32(0),
                load_const_f32(45.0),
                add(),
                store_void(0),
                load_local_i32(0),
                load_const_f32(1.0),
                add(),
                store_local(0),
                jump(-12),
                stop()
            ]
        );
    }

    #[test]
    fn test_fence() {
        assert_eq!(
            compile("(fence (x from: 0 to: 5 num: 5) (+ x x))"),
            vec![
                jump(1),
                load_const_f32(5.0),
                store_local(0),
                load_const_f32(0.0),
                store_local(1),
                load_const_f32(5.0),
                load_const_f32(0.0),
                sub(),
                load_const_f32(5.0),
                load_const_f32(1.0),
                sub(),
                div(),
                store_local(2),
                load_const_f32(0.0),
                store_local(3),
                load_local_i32(3),
                store_local(4),
                load_local_i32(1),
                load_local_i32(0),
                lt(),
                jump_if(16),
                load_local_i32(3),
                load_local_i32(1),
                load_local_i32(2),
                mul(),
                add(),
                store_local(4),
                load_local_i32(4),
                load_local_i32(4),
                add(),
                store_void(0),
                load_local_i32(1),
                load_const_f32(1.0),
                add(),
                store_local(1),
                jump(-18),
                stop()
            ]
        );
    }
}