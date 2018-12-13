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

use error::*;
use keywords::{keyword_to_string, string_to_keyword_hash, Keyword};
use opcodes::{opcode_stack_offset, Opcode};
use parser::Node;

const TAU: f32 = 6.283_185_307_179_586; // todo: move TAU to math

const MEMORY_LOCAL_SIZE: usize = 40;

#[derive(Clone, Copy)]
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

impl Mem {
    fn from_bytecode_arg(b: &BytecodeArg) -> SenResult<Mem> {
        match b {
            BytecodeArg::Int(i) => match i {
                0 => Ok(Mem::Argument),
                1 => Ok(Mem::Local),
                2 => Ok(Mem::Global),
                3 => Ok(Mem::Constant),
                4 => Ok(Mem::Void),
                _ => Err(SenError::MemUnmappableI32),
            },
            _ => Err(SenError::MemUnmappableBytecodeArg),
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
    Colour(ColourFormat, f32, f32, f32, f32),
}

impl fmt::Display for BytecodeArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BytecodeArg::Int(i) => write!(f, "Int({})", i),
            BytecodeArg::Float(s) => write!(f, "Float({})", s),
            BytecodeArg::Name(i) => write!(f, "Name({})", i),
            BytecodeArg::Colour(s, a, b, c, d) => {
                write!(f, "Colour({} {} {} {} {})", s, a, b, c, d)
            }
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
                let mem = Mem::from_bytecode_arg(&self.arg0).map_err(|_| ::std::fmt::Error)?;
                write!(f, "{}\t{}\t{}", self.op, mem, self.arg1)
            }
            // todo: format JUMP and JUMP_IF
            Opcode::JUMP | Opcode::JUMP_IF => {
                write!(f, "{}\t{}\t{}", self.op, self.arg0, self.arg1)
            }
            // todo: format NATIVE
            Opcode::NATIVE => write!(f, "{}\t{}\t{}", self.op, self.arg0, self.arg1),
            // todo: format PILE
            Opcode::PILE => write!(f, "{}\t{}\t{}", self.op, self.arg0, self.arg1),
            _ => write!(f, "{}", self.op),
        }
    }
}

#[derive(Debug)]
pub struct FnInfo {
    fn_name: String,
    arg_address: usize,
    body_address: usize,
    num_args: i32,
    argument_offsets: Vec<i32>,
}

impl FnInfo {
    fn new(fn_name: String) -> FnInfo {
        FnInfo {
            fn_name: fn_name,
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
    code: Vec<Bytecode>,
    fn_info: Vec<FnInfo>,
    // word_lut
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, b) in self.code.iter().enumerate() {
            write!(f, "{}\t{}\n", i, b)?;
        }
        Ok(())
    }
}

impl Program {
    fn new() -> Self {
        Program {
            code: Vec::new(),
            fn_info: Vec::new(),
        }
    }

    pub fn add_bytecode(&mut self, bc: Bytecode) -> SenResult<()> {
        self.code.push(bc);
        Ok(())
    }

    pub fn get_fn_info_index(&self, node: &Node) -> Option<usize> {
        if let Node::Name(text, _, _) = node {
            for (i, fi) in self.fn_info.iter().enumerate() {
                if fi.fn_name == *text {
                    return Some(i)
                }
            }
        }
        None
    }
}

struct Compilation<'a> {
    pub program: &'a mut Program,

    pub opcode_offset: i32,

    pub global_mappings: HashMap<String, i32>,
    pub global_mapping_marker: i32,

    pub local_mappings: HashMap<String, i32>,
    pub local_mapping_marker: i32, // todo: check that it is < MEMORY_LOCAL_SIZE, as that constant is used in the interpreter

    pub current_fn_info_index: Option<usize>,

    pub string_to_keyword: HashMap<String, Keyword>,
}

impl<'a> Compilation<'a> {
    pub fn new(program: &'a mut Program) -> Self {
        Compilation {
            program,

            opcode_offset: 0,

            global_mappings: HashMap::new(),
            global_mapping_marker: 0,

            local_mappings: HashMap::new(),
            local_mapping_marker: 0,

            current_fn_info_index: None,

            string_to_keyword: string_to_keyword_hash(),
        }
    }

    pub fn clear_global_mappings(&mut self) -> SenResult<()> {
        self.global_mappings.clear();
        self.global_mapping_marker = 0;
        Ok(())
    }

    fn add_global_mapping(&mut self, s: String) -> SenResult<i32> {
        self.global_mappings.insert(s, self.global_mapping_marker);
        self.global_mapping_marker += 1;
        Ok(self.global_mapping_marker - 1)
    }

    pub fn clear_local_mappings(&mut self) -> SenResult<()> {
        self.local_mappings.clear();
        self.local_mapping_marker = 0;
        Ok(())
    }

    fn add_local_mapping(&mut self, s: String) -> SenResult<i32> {
        self.local_mappings.insert(s, self.local_mapping_marker);
        self.local_mapping_marker += 1;
        Ok(self.local_mapping_marker - 1)
    }

    // we want a local mapping that's going to be used to store an internal variable
    // (e.g. during a fence loop)
    // note: it's up to the caller to manage this reference
    fn add_internal_local_mapping(&mut self) -> SenResult<i32> {
        let s = "internal_local_mapping".to_string();
        self.local_mappings.insert(s, self.local_mapping_marker);
        self.local_mapping_marker += 1;
        Ok(self.local_mapping_marker - 1)
    }

    fn correct_function_addresses(&mut self) -> SenResult<()> {
        let mut all_fixes: Vec<(usize, Opcode, i32, i32)> = Vec::new(); // tuple of index, op, arg0, arg1
        let mut arg1_fixes: Vec<(usize, i32)> = Vec::new(); // tuple of index,value pairs

        // go through the bytecode fixing up function call addresses
        for (i, bc) in self.program.code.iter().enumerate() {
            // replace the temporarily stored index in the args of CALL and CALL_0 with
            // the actual values

            match bc.op {
                Opcode::CALL => {
                    if let BytecodeArg::Int(fn_info_index) = bc.arg0 {
                        let fn_info = &self.program.fn_info[fn_info_index as usize];

                        // the previous two bytecodes will be LOADs of CONST.
                        // i - 2 == the address to call
                        // i - 1 == the number of arguments used by the function
                        arg1_fixes.push((i-2, fn_info.arg_address as i32));
                        arg1_fixes.push((i-1, fn_info.num_args as i32));
                    }
                },
                Opcode::CALL_0 => {
                    if let BytecodeArg::Int(fn_info_index) = bc.arg0 {
                        let fn_info = &self.program.fn_info[fn_info_index as usize];
                        arg1_fixes.push((i-1, fn_info.body_address as i32));
                    }
                },
                Opcode::PLACEHOLDER_STORE => {
                    // opcode's arg0 is the fn_info_index and arg1 is the label_value
                    if let BytecodeArg::Int(fn_info_index) = bc.arg0 {
                        let fn_info = &self.program.fn_info[fn_info_index as usize];
                        if let BytecodeArg::Int(label_value) = bc.arg1 {
                            if let Some(data_index) = fn_info.get_argument_mapping(label_value) {
                                all_fixes.push((i, Opcode::STORE, Mem::Argument as i32, data_index as i32));
                            } else {
                                all_fixes.push((i, Opcode::STORE, Mem::Void as i32, 0));
                            }
                        }
                    }
                },
                _ => ()
            }
        }

        for (index, op, arg0, arg1) in all_fixes {
            self.bytecode_modify(index, op, arg0, arg1)?;
        }
        for (index, arg1) in arg1_fixes {
            self.bytecode_modify_arg1_i32(index, arg1)?;
        }

        Ok(())
    }

    fn register_top_level_preamble(&mut self) -> SenResult<()> {
        self.add_global_mapping(keyword_to_string(Keyword::GenInitial))?;

        self.add_global_mapping(keyword_to_string(Keyword::CanvasWidth))?;
        self.add_global_mapping(keyword_to_string(Keyword::CanvasHeight))?;

        self.add_global_mapping(keyword_to_string(Keyword::MathTau))?;

        self.add_global_mapping(keyword_to_string(Keyword::White))?;
        self.add_global_mapping(keyword_to_string(Keyword::Black))?;
        self.add_global_mapping(keyword_to_string(Keyword::Red))?;
        self.add_global_mapping(keyword_to_string(Keyword::Green))?;
        self.add_global_mapping(keyword_to_string(Keyword::Blue))?;
        self.add_global_mapping(keyword_to_string(Keyword::Yellow))?;
        self.add_global_mapping(keyword_to_string(Keyword::Magenta))?;
        self.add_global_mapping(keyword_to_string(Keyword::Cyan))?;

        self.add_global_mapping(keyword_to_string(Keyword::ColProceduralFnPresets))?;
        self.add_global_mapping(keyword_to_string(Keyword::EasePresets))?;

        Ok(())
    }

    fn register_top_level_fns(&mut self, ast: &Vec<Node>) -> SenResult<()> {
        // clear all data
        self.program.fn_info = Vec::new();

        // register top level fns
        for n in ast.iter() {
            if self.is_list_beginning_with(n, Keyword::Fn) {
                // get the name of the fn
                if let Node::List(nodes, _) = n {
                    if nodes.len() < 2 {
                        // a list with just the 'fn' keyword ???
                        return Err(SenError::Compiler(format!("malformed function definition")));
                    }
                    let name_and_params = &nodes[1];
                    if let Node::List(np_nodes, _) = name_and_params {
                        if np_nodes.len() > 0 {
                            let name_node = &np_nodes[0];
                            if let Node::Name(text, _, _) = name_node {
                                // we have a named top-level fn declaration
                                //
                                // create and add a top level fn
                                let fn_info = FnInfo::new(text.to_string());
                                self.program.fn_info.push(fn_info);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn register_names_in_define(&mut self, lhs: &Node) -> SenResult<()> {
        match lhs {
            Node::Name(name, _, _) => {
                // (define foo 42)
                let s = name.to_string();
                // todo: is this check necessary?
                if let Some(_i) = self.global_mappings.get(name) {
                    // name was already added to global_mappings
                    return Ok(())
                }

                if let Err(e) = self.add_global_mapping(s) {
                    return Err(e);
                }
            }
            Node::List(nodes, _) | Node::Vector(nodes, _) => {
                // (define [a b] (something))
                // (define [a [x y]] (something))
                for n in nodes.iter() {
                    if let Err(e) = self.register_names_in_define(n) {
                        return Err(e);
                    }
                }
            }
            _ => (),
        }
        Ok(())
    }

    fn register_top_level_defines(&mut self, ast: &Vec<Node>) -> SenResult<()> {
        let define_keyword_string = keyword_to_string(Keyword::Define);

        for n in ast.iter() {
            if let Node::List(nodes, _) = n {
                if nodes.len() > 0 {
                    let define_keyword = &nodes[0];
                    if let Node::Name(text, _, _) = define_keyword {
                        if text == &define_keyword_string {
                            let mut defs = &nodes[1..];
                            while defs.len() > 1 {
                                if let Err(e) = self.register_names_in_define(&defs[0]) {
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

    fn compile_preamble(&mut self) -> SenResult<()> {
        // ********************************************************************************
        // NOTE: each entry should have a corresponding entry in
        // register_top_level_preamble
        // ********************************************************************************
        self.compile_global_bind_i32(keyword_to_string(Keyword::GenInitial), 0)?;

        self.compile_global_bind_f32(keyword_to_string(Keyword::CanvasWidth), 1000.0)?;
        self.compile_global_bind_f32(keyword_to_string(Keyword::CanvasHeight), 1000.0)?;

        self.compile_global_bind_f32(keyword_to_string(Keyword::MathTau), TAU)?;

        self.compile_global_bind_col(keyword_to_string(Keyword::White), 1.0, 1.0, 1.0, 1.0)?;
        self.compile_global_bind_col(keyword_to_string(Keyword::Black), 0.0, 0.0, 0.0, 1.0)?;
        self.compile_global_bind_col(keyword_to_string(Keyword::Red), 1.0, 0.0, 0.0, 1.0)?;
        self.compile_global_bind_col(keyword_to_string(Keyword::Green), 0.0, 1.0, 0.0, 1.0)?;
        self.compile_global_bind_col(keyword_to_string(Keyword::Blue), 0.0, 0.0, 1.0, 1.0)?;
        self.compile_global_bind_col(keyword_to_string(Keyword::Yellow), 1.0, 1.0, 0.0, 1.0)?;
        self.compile_global_bind_col(keyword_to_string(Keyword::Magenta), 1.0, 0.0, 1.0, 1.0)?;
        self.compile_global_bind_col(keyword_to_string(Keyword::Cyan), 0.0, 1.0, 1.0, 1.0)?;

        self.compile_global_bind_procedural_presets()?;
        self.compile_global_bind_ease_presets()?;

        // ********************************************************************************
        // NOTE: each entry should have a corresponding entry in
        // register_top_level_preamble
        // ********************************************************************************

        // slap a stop onto the end of this program
        self.emit_opcode(Opcode::STOP)?;

        Ok(())
    }

    fn compile_global_bind_procedural_presets(&mut self) -> SenResult<()> {
        // create a vector
        self.emit_opcode_mem_i32(Opcode::LOAD, Mem::Void, 0)?;

        // append the names
        self.append_keyword(Keyword::Chrome)?;
        self.append_keyword(Keyword::HotlineMiami)?;
        self.append_keyword(Keyword::KnightRider)?;
        self.append_keyword(Keyword::Mars)?;
        self.append_keyword(Keyword::Rainbow)?;
        self.append_keyword(Keyword::Robocop)?;
        self.append_keyword(Keyword::Transformers)?;

        self.store_globally(keyword_to_string(Keyword::ColProceduralFnPresets))?;

        Ok(())
    }

    fn compile_global_bind_ease_presets(&mut self) -> SenResult<()> {
        // create a vector
        self.emit_opcode_mem_i32(Opcode::LOAD, Mem::Void, 0)?;

        // append the names
        self.append_keyword(Keyword::Linear)?;
        self.append_keyword(Keyword::EaseQuick)?;
        self.append_keyword(Keyword::EaseSlowIn)?;
        self.append_keyword(Keyword::EaseSlowInOut)?;
        self.append_keyword(Keyword::EaseQuadraticIn)?;
        self.append_keyword(Keyword::EaseQuadraticOut)?;
        self.append_keyword(Keyword::EaseQuadraticInOut)?;
        self.append_keyword(Keyword::EaseCubicIn)?;
        self.append_keyword(Keyword::EaseCubicOut)?;
        self.append_keyword(Keyword::EaseCubicInOut)?;
        self.append_keyword(Keyword::EaseQuarticIn)?;
        self.append_keyword(Keyword::EaseQuarticOut)?;
        self.append_keyword(Keyword::EaseQuarticInOut)?;
        self.append_keyword(Keyword::EaseQuinticIn)?;
        self.append_keyword(Keyword::EaseQuinticOut)?;
        self.append_keyword(Keyword::EaseQuinticInOut)?;
        self.append_keyword(Keyword::EaseSinIn)?;
        self.append_keyword(Keyword::EaseSinOut)?;
        self.append_keyword(Keyword::EaseSinInOut)?;
        self.append_keyword(Keyword::EaseCircularIn)?;
        self.append_keyword(Keyword::EaseCircularOut)?;
        self.append_keyword(Keyword::EaseCircularInOut)?;
        self.append_keyword(Keyword::EaseExponentialIn)?;
        self.append_keyword(Keyword::EaseExponentialOut)?;
        self.append_keyword(Keyword::EaseExponentialInOut)?;
        self.append_keyword(Keyword::EaseElasticIn)?;
        self.append_keyword(Keyword::EaseElasticOut)?;
        self.append_keyword(Keyword::EaseElasticInOut)?;
        self.append_keyword(Keyword::EaseBackIn)?;
        self.append_keyword(Keyword::EaseBackOut)?;
        self.append_keyword(Keyword::EaseBackInOut)?;
        self.append_keyword(Keyword::EaseBounceIn)?;
        self.append_keyword(Keyword::EaseBounceOut)?;
        self.append_keyword(Keyword::EaseBounceInOut)?;

        self.store_globally(keyword_to_string(Keyword::EasePresets))?;

        Ok(())
    }

    fn compile_common(&mut self, ast: &Vec<Node>) -> SenResult<()> {
        self.compile_common_prologue(ast)?;
        self.compile_common_top_level_fns(ast)?;
        self.compile_common_top_level_defines(ast)?;
        self.compile_common_top_level_forms(ast)?;
        self.compile_common_epilogue(ast)?;
        Ok(())
    }

    fn compile_common_prologue(&mut self, ast: &Vec<Node>) -> SenResult<()> {
        self.clear_global_mappings()?;
        self.clear_local_mappings()?;
        // compilation->current_fn_info = NULL;

        self.register_top_level_preamble()?;
        self.register_top_level_fns(ast)?;
        self.register_top_level_defines(ast)?;

        Ok(())
    }

    fn compile_common_top_level_fns(&mut self, ast: &Vec<Node>) -> SenResult<()> {
        // a placeholder, filled in at the end of this function
        self.emit_opcode(Opcode::JUMP)?;
        let start_index = self.program.code.len() - 1;

        // compile the top-level functions
        for n in ast.iter() {
            if self.is_list_beginning_with(n, Keyword::Fn) {
                self.compile(n)?; // todo: the c-impl returns a node to continue from
            }
        }

        // jump to the program's starting address
        let jump_address = self.program.code.len() as i32;
        self.bytecode_modify_arg0_i32(start_index, jump_address)?;

        Ok(())
    }

    fn compile_common_top_level_defines(&mut self, ast: &Vec<Node>) -> SenResult<()> {
        for n in ast.iter() {
            if self.is_list_beginning_with(n, Keyword::Define) {
                if let Node::List(children, _) = n {
                    return self.compile_define(&children[1..], Mem::Global);
                }
                return Err(SenError::Compiler(format!("can never get here")))
            }
        }
        Ok(())
    }

    fn compile_common_top_level_forms(&mut self, ast: &Vec<Node>) -> SenResult<()> {
        for n in ast.iter() {
            if !self.is_list_beginning_with(n, Keyword::Define)
                && !self.is_list_beginning_with(n, Keyword::Fn)
            {
                self.compile(n)?;
            }
        }
        Ok(())
    }

    fn compile_common_epilogue(&mut self, _ast: &Vec<Node>) -> SenResult<()> {
        self.emit_opcode(Opcode::STOP)?;

        // now update the addreses used by CALL and CALL_0
        self.correct_function_addresses()?;

        Ok(())
    }

    fn compile_define(&mut self, children: &[Node], mem: Mem) -> SenResult<()> {
        // children are an even number of nodes representing binding/value pairs
        // (define a 10 b 20 c 30) -> a 10 b 20 c 30

        let mut defs = children;

        if defs.len() % 2 != 0 {
            // log: should be an even number of elements
            return Err(SenError::Compiler(format!("should be an even number of elements")));
        }

        while defs.len() > 0 {
            let lhs_node = &defs[0];
            let value_node = &defs[1];

            self.compile(&value_node)?;

            match lhs_node {
                Node::Name(_, _, _) => {
                    // define foo 10
                    self.store_from_stack_to_memory(&lhs_node, mem)?;
                }
                Node::Vector(kids, _) => {
                    // define [a b] (something-that-returns-a-vector ...)

                    // check if we can use the PILE opcode
                    if all_children_are_name_nodes(lhs_node) {
                        let num_kids = kids.len();

                        // PILE will stack the elements in the rhs vector in order,
                        // so the lhs values have to be popped in reverse order
                        self.emit_opcode_i32_i32(Opcode::PILE, num_kids as i32, 0)?;
                        self.opcode_offset = self.opcode_offset + num_kids as i32 - 1;

                        for k in kids.iter().rev() {
                            self.store_from_stack_to_memory(&k, mem)?;
                        }
                    } else {
                        // all nodes in lhs vector definition should be names
                        // note: this means that recursive name assignments aren't implemented
                        // e.g. (define [a [b c]] something)
                        return Err(SenError::Compiler(format!("recursive name assignments aren't implemented")));
                    }
                }
                _ => return Err(SenError::Compiler(format!("compile_define"))),
            }

            defs = &defs[2..];
        }


        Ok(())
    }

    fn compile(&mut self, ast: &Node) -> SenResult<()> {
        // todo: move this out of compile and into the compilation struct

        match ast {
            Node::List(children, _) => self.compile_list(children)?,
            Node::Float(f, _) => return self.emit_opcode_mem_f32(Opcode::LOAD, Mem::Constant, *f),
            Node::Vector(children, _) => {
                if children.len() == 2 {
                    return self.compile_2d(children);
                } else {
                    return self.compile_vector(children);
                }
            }
            Node::Name(text, iname, _) => {
                let found_name = self.compile_user_defined_name(&text, *iname)?;
                if found_name {
                    return Ok(())
                } else {
                    return Err(SenError::Compiler(format!("compile: can't find user defined name: {}", text)))
                }
            },
            _ => return Err(SenError::Compiler(format!("compile"))),
        }

        Ok(())
    }

    fn compile_list(&mut self, children: &[Node]) -> SenResult<()> {
        if children.len() == 0 {
            // should this be an error?
            return Err(SenError::Compiler(format!("compile_list no children (should this be an error?)")));
        }

        match &children[0] {
            Node::List(kids, _) => self.compile_list(&kids)?,
            Node::Name(text, iname, _) => {

                if let Some(fn_info_index) = self.program.get_fn_info_index(&children[0]) {
                    // todo: get_fn_info_index is re-checking that this is a Node::Name
                    self.compile_fn_invocation(&children[1..], fn_info_index)?;
                    return Ok(())
                }

                let found_name = self.compile_user_defined_name(&text, *iname)?;
                if found_name {
                    return Ok(())
                }

                let mut found_keyword: bool = false;
                let mut keyword: Keyword = Keyword::Define;
                if let Some(kw) = self.string_to_keyword.get(text) {
                    keyword = *kw;
                    found_keyword = true;
                }

                if found_keyword {
                    match keyword {
                        Keyword::Define => self.compile_define(&children[1..], Mem::Local)?,
                        Keyword::If => self.compile_if(&children[1..])?,
                        Keyword::Each => unimplemented!(),
                        Keyword::Loop => unimplemented!(),
                        Keyword::Fence => unimplemented!(),
                        Keyword::OnMatrixStack => self.compile_on_matrix_stack(&children[1..])?,
                        Keyword::Fn => self.compile_fn(&children[1..])?,
                        Keyword::Plus => self.compile_math(&children[1..], Opcode::ADD)?,
                        Keyword::Minus => self.compile_math(&children[1..], Opcode::SUB)?,
                        Keyword::Mult => self.compile_math(&children[1..], Opcode::MUL)?,
                        Keyword::Divide => self.compile_math(&children[1..], Opcode::DIV)?,
                        Keyword::Mod => self.compile_math(&children[1..], Opcode::MOD)?,
                        Keyword::Equal => self.compile_math(&children[1..], Opcode::EQ)?,
                        Keyword::Lt => self.compile_math(&children[1..], Opcode::LT)?,
                        Keyword::Gt => self.compile_math(&children[1..], Opcode::GT)?,
                        Keyword::And => self.compile_math(&children[1..], Opcode::AND)?,
                        Keyword::Or => self.compile_math(&children[1..], Opcode::OR)?,
                        Keyword::Not => self.compile_next_one(&children[1..], Opcode::NOT)?,
                        Keyword::Sqrt => self.compile_next_one(&children[1..], Opcode::SQRT)?,
                        Keyword::AddressOf => self.compile_address_of(&children[1..])?,
                        Keyword::FnCall => unimplemented!(),
                        Keyword::VectorAppend => unimplemented!(),
                        Keyword::Quote => unimplemented!(),
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

            },
            _ => return Err(SenError::Compiler(format!("compile_list strange child")))
        }

        Ok(())
    }

    fn compile_address_of(&mut self, children: &[Node]) -> SenResult<()> {
        // fn_name should be a defined function's name, it will be known at compile time
        if let Some(fn_info_index) = self.program.get_fn_info_index(&children[0]) {
            // store the index into program->fn_info in the program
            self.emit_opcode_mem_i32(Opcode::LOAD, Mem::Constant, fn_info_index as i32)?;
            return Ok(())
        }


        Err(SenError::Compiler(format!("compile_address_of")))
    }

    fn compile_on_matrix_stack(&mut self, children: &[Node]) -> SenResult<()> {
        self.emit_opcode(Opcode::MTX_LOAD)?;
        self.compile_rest(children)?;
        self.emit_opcode(Opcode::MTX_STORE)?;
        Ok(())
    }

    fn compile_if(&mut self, children: &[Node]) -> SenResult<()> {
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
            return Err(SenError::Compiler(format!("if clause requires 2 or 3 forms (given {})", num_children)))
        }

        self.compile(if_node)?;

        // insert jump to after the 'then' node if not true
        let addr_jump_then = self.program.code.len();
        self.emit_opcode(Opcode::JUMP_IF)?;

        // the offset after the if
        let offset_after_if = self.opcode_offset;

        self.compile(then_node)?;

        let offset_after_then = self.opcode_offset;

        if let Some(else_node) = else_node {
            // logically we're now going to go down one of possibly two paths
            // so we can't just continue to add the compilation->opcode_offset since
            // that would result in the offset taking both of the conditional's paths
            self.opcode_offset = offset_after_if;

            // insert a bc_jump_else opcode
            let addr_jump_else = self.program.code.len();

            self.emit_opcode(Opcode::JUMP)?;

            let addr_jump_then_offset = self.program.code.len() as i32 - addr_jump_then as i32;
            self.bytecode_modify_arg0_i32(addr_jump_then, addr_jump_then_offset)?;

            self.compile(else_node)?;

            let offset_after_else = self.opcode_offset;

            if offset_after_then != offset_after_else {
                // is this case actually going to happen?
                // if so we can check which of the two paths has the lower opcode offset
                // and pad out that path by inserting some LOAD CONST 9999 into the
                // program
                return Err(SenError::Compiler(format!("different opcode_offsets for the two paths in a conditional")))
            }

            let addr_jump_else_offset = self.program.code.len() as i32 - addr_jump_else as i32;
            self.bytecode_modify_arg0_i32(addr_jump_else, addr_jump_else_offset)?;
        } else {
            let addr_jump_then_offset = self.program.code.len() as i32 - addr_jump_then as i32;
            self.bytecode_modify_arg0_i32(addr_jump_then, addr_jump_then_offset)?;
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
    fn compile_fn(&mut self, children: &[Node]) -> SenResult<()> {
        // fn (adder a: 0 b: 0) (+ a b)
        self.clear_local_mappings()?;

        let signature = &children[0]; // (addr a: 0 b: 0)
        if let Node::List(kids, _) = signature {
            if kids.len() == 0 {
                // no fn name given
                return Err(SenError::CompilerFnWithoutName)
            }

            let fn_name = &kids[0];
            if let Some(index) = self.program.get_fn_info_index(&fn_name) {
                self.current_fn_info_index = Some(index);

                // -------------
                // the arguments
                // -------------
                let mut updated_fn_info: FnInfo;
                {
                    let fn_info: &FnInfo = &self.program.fn_info[index];
                    updated_fn_info = FnInfo::new(fn_info.fn_name.to_string());
                }

                updated_fn_info.arg_address = self.program.code.len();

                // pairs of label/value declarations
                let mut var_decls = &kids[1..];
                let mut num_args = 0;
                let mut counter = 0;

                if var_decls.len() % 2 != 0 {
                    return Err(SenError::Compiler(format!("fn declaration doesn't have matching arg/value pairs")))
                }

                while var_decls.len() > 0 {
                    let label_node = &var_decls[0];
                    let value_node = &var_decls[1];

                    // get argument mapping
                    if let Node::Label(_, label_i, _) = label_node {
                        updated_fn_info.argument_offsets.push(*label_i);

                        // if let Some(label_i) = self.global_mappings.get(text) {
                        // } else {
                        //     // should be impossible to get here, the global mappings for the
                        //     // fn args should all have been registered in the
                        //     // register_top_level_fns function
                        // }

                        self.emit_opcode_mem_i32(Opcode::LOAD, Mem::Constant, *label_i)?;
                    }

                    self.emit_opcode_mem_i32(Opcode::STORE, Mem::Argument, counter)?;
                    counter += 1;

                    self.compile(value_node)?;
                    self.emit_opcode_mem_i32(Opcode::STORE, Mem::Argument, counter)?;
                    counter += 1;

                    num_args += 1;
                    var_decls = &var_decls[2..];
                }
                updated_fn_info.num_args = num_args;

                self.emit_opcode(Opcode::RET_0)?;

                // --------
                // the body
                // --------

                updated_fn_info.body_address = self.program.code.len();

                self.program.fn_info[index] = updated_fn_info;

                // compile the body forms (woooaaaoohhh body form, body form for yoooouuuu)
                self.compile_rest(&children[1..])?;

                // Don't need any STORE, MEM_SEG_VOID instructions as the RET will
                // pop the frame and blow the stack
                self.emit_opcode(Opcode::RET)?;

                self.current_fn_info_index = None;



            } else {
                // todo: implement Display for Node
                // return Err(SenError::Compiler(format!("cannot find fn_info for {}", fn_name)))
                return Err(SenError::Compiler(format!("cannot find fn_info for node")))
            }
        } else {
            // first item in fn declaration needs to be a list of function name and args
            return Err(SenError::CompilerFnDeclIncomplete)
        }


        Ok(())
    }

    // if (adder a: 10 b: 20) then children == a: 10 b: 20
    fn compile_fn_invocation(&mut self, children: &[Node], fn_info_index: usize) -> SenResult<()> {

        // NOTE: CALL and CALL_0 get their function offsets and num args from the
        // stack so add some placeholder LOAD CONST opcodes and fill the CALL, CALL_0
        // with fn_info indexes that can later be used to fill in the LOAD CONST
        // opcodes with their correct offsets doing it this way enables functions to
        // call other functions that are declared later in the script

        // prepare the MEM_SEG_ARGUMENT with default values

        // for the function address
        self.emit_opcode_mem_i32(Opcode::LOAD, Mem::Constant, 666)?;
        // for the num args
        self.emit_opcode_mem_i32(Opcode::LOAD, Mem::Constant, 667)?;

        self.emit_opcode_i32_i32(Opcode::CALL, fn_info_index as i32, fn_info_index as i32)?;

        // overwrite the default arguments with the actual arguments given by the fn invocation
        let mut arg_vals = &children[..];
        while arg_vals.len() > 1 {
            let arg = &arg_vals[0];
            if let Node::Label(_, i_name, _) = arg {
                let val = &arg_vals[1];
                self.compile(val)?;
                self.emit_opcode_i32_i32(Opcode::PLACEHOLDER_STORE, fn_info_index as i32, *i_name)?;
            } else {
                return Err(SenError::Compiler(format!("compile_fn_invocation")))
            }

            arg_vals = &arg_vals[2..];
        }

        // call the body of the function
        self.emit_opcode_mem_i32(Opcode::LOAD, Mem::Constant, 668)?;
        self.emit_opcode_i32_i32(Opcode::CALL_0, fn_info_index as i32, fn_info_index as i32)?;

        Ok(())
    }

    fn compile_rest(&mut self, children: &[Node]) -> SenResult<()> {
        for n in children {
            self.compile(n)?;
        }
        Ok(())
    }

    fn compile_next_one(&mut self, children: &[Node], op: Opcode) -> SenResult<()> {
        if children.len() < 1 {
            return Err(SenError::Compiler(format!("compile_next_one")))
        }

        self.compile(&children[0])?;
        self.emit_opcode(op)?;

        Ok(())
    }

    fn compile_math(&mut self, children: &[Node], op: Opcode) -> SenResult<()> {
        for n in children {
            self.compile(n)?;
        }
        self.emit_opcode(op)?;
        Ok(())
    }

    fn compile_2d(&mut self, children: &[Node]) -> SenResult<()> {
        for n in children {
            self.compile(n)?;
        }
        self.emit_opcode(Opcode::SQUISH2)?;
        Ok(())
    }

    fn compile_vector(&mut self, children: &[Node]) -> SenResult<()> {
        // pushing from the VOID means creating a new, empty vector
        self.emit_opcode_mem_i32(Opcode::LOAD, Mem::Void, 0)?;
        for n in children {
            self.compile(n)?;
            self.emit_opcode(Opcode::APPEND)?;
        }

        Ok(())
    }

    fn compile_global_bind_i32(&mut self, s: String, value: i32) -> SenResult<()> {
        self.emit_opcode_mem_i32(Opcode::LOAD, Mem::Constant, value)?;
        self.store_globally(s)?;
        Ok(())
    }

    fn compile_global_bind_f32(&mut self, s: String, value: f32) -> SenResult<()> {
        self.emit_opcode_mem_f32(Opcode::LOAD, Mem::Constant, value)?;
        self.store_globally(s)?;
        Ok(())
    }

    fn compile_global_bind_col(
        &mut self,
        s: String,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) -> SenResult<()> {
        self.emit_opcode_mem_rgba(Opcode::LOAD, Mem::Constant, r, g, b, a)?;
        self.store_globally(s)?;
        Ok(())
    }

    fn append_keyword(&mut self, kw: Keyword) -> SenResult<()> {
        self.emit_opcode_mem_name(Opcode::LOAD, Mem::Constant, kw as i32)?;
        self.emit_opcode(Opcode::APPEND)?;
        Ok(())
    }

    fn store_locally(&mut self, s: String) -> SenResult<i32> {
        let address: i32 = match self.local_mappings.get(&s) {
            Some(&local_mapping) => local_mapping, // already storing the binding name
            None => self.add_local_mapping(s)?,
        };

        self.emit_opcode_mem_i32(Opcode::STORE, Mem::Local, address)?;

        Ok(address)
    }

    fn store_globally(&mut self, s: String) -> SenResult<i32> {
        let address: i32 = match self.global_mappings.get(&s) {
            Some(&global_mapping) => global_mapping, // already storing the binding name
            None => self.add_global_mapping(s)?,
        };

        self.emit_opcode_mem_i32(Opcode::STORE, Mem::Global, address)?;

        Ok(address)
    }

    fn store_from_stack_to_memory(&mut self, node: &Node, mem: Mem) -> SenResult<i32> {
        if let Node::Name(text, _, _) = node {
            match mem {
                Mem::Local => self.store_locally(text.to_string()),
                Mem::Global => self.store_globally(text.to_string()),
                _ => Err(SenError::Compiler(format!("store_from_stack_to_memory invalid memory type"))),
            }
        } else {
            Err(SenError::Compiler(format!("store_from_stack_to_memory")))
        }
    }

    fn compile_user_defined_name(&mut self, s: &str, iname: i32) -> SenResult<bool> {

        let mut val: i32 = 0;
        let mut found = false;

        if let Some(local_mapping) = self.local_mappings.get(s) {
            val = *local_mapping;
            found = true;
        }

        if found {
            self.emit_opcode_mem_i32(Opcode::LOAD, Mem::Local, val)?;
            return Ok(true)
        }


        // check arguments if we're in a function
        if let Some(current_fn_info_index) = self.current_fn_info_index {
            let maybe_argument_mapping;
            {
                let fn_info = &self.program.fn_info[current_fn_info_index];
                maybe_argument_mapping = fn_info.get_argument_mapping(iname);
            }
            if let Some(argument_mapping) = maybe_argument_mapping {
                self.emit_opcode_mem_i32(Opcode::LOAD, Mem::Argument, argument_mapping as i32)?;
                return Ok(true)
            }
        }


        if let Some(global_mapping) = self.global_mappings.get(s) {
            val = *global_mapping;
            found = true;
        }
        if found {
            self.emit_opcode_mem_i32(Opcode::LOAD, Mem::Global, val)?;
            return Ok(true)
        }



        // // could be a keyword such as linear, ease-in etc
        // if let Some(keyword) = self.string_to_keyword.get(s) {
        //     val = *keyword as i32;
        //     found = true;
        // }
        // if found {
        //     self.emit_opcode_mem_i32(Opcode::LOAD, Mem::Constant, val)?;
        //     return Ok(true)
        // }

        // todo: log unknown mapping for s

        Ok(false)
    }

    fn emit_opcode(&mut self, op: Opcode) -> SenResult<()> {
        let b = Bytecode {
            op: op,
            arg0: BytecodeArg::Int(0),
            arg1: BytecodeArg::Int(0),
        };

        self.program.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }

    fn emit_opcode_i32_i32(&mut self, op: Opcode, arg0: i32, arg1: i32) -> SenResult<()> {
        let b = Bytecode {
            op: op,
            arg0: BytecodeArg::Int(arg0),
            arg1: BytecodeArg::Int(arg1),
        };

        self.program.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }

    fn emit_opcode_mem_i32(&mut self, op: Opcode, arg0: Mem, arg1: i32) -> SenResult<()> {
        let b = Bytecode {
            op: op,
            arg0: BytecodeArg::Int(arg0 as i32),
            arg1: BytecodeArg::Int(arg1),
        };

        self.program.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }

    fn emit_opcode_mem_name(&mut self, op: Opcode, arg0: Mem, arg1: i32) -> SenResult<()> {
        let b = Bytecode {
            op: op,
            arg0: BytecodeArg::Int(arg0 as i32),
            arg1: BytecodeArg::Name(arg1),
        };

        self.program.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }

    fn emit_opcode_mem_f32(&mut self, op: Opcode, arg0: Mem, arg1: f32) -> SenResult<()> {
        let b = Bytecode {
            op: op,
            arg0: BytecodeArg::Int(arg0 as i32),
            arg1: BytecodeArg::Float(arg1),
        };

        self.program.add_bytecode(b)?;
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
    ) -> SenResult<()> {
        let b = Bytecode {
            op: op,
            arg0: BytecodeArg::Int(arg0 as i32),
            arg1: BytecodeArg::Colour(ColourFormat::Rgba, r, g, b, a),
        };

        self.program.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }

    fn emit_opcode_i32_f32(&mut self, op: Opcode, arg0: i32, arg1: f32) -> SenResult<()> {
        let b = Bytecode {
            op: op,
            arg0: BytecodeArg::Int(arg0),
            arg1: BytecodeArg::Float(arg1),
        };

        self.program.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }

    fn emit_opcode_i32_name(&mut self, op: Opcode, arg0: i32, arg1: i32) -> SenResult<()> {
        let b = Bytecode {
            op: op,
            arg0: BytecodeArg::Int(arg0),
            arg1: BytecodeArg::Name(arg1),
        };

        self.program.add_bytecode(b)?;
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
    ) -> SenResult<()> {
        let b = Bytecode {
            op: op,
            arg0: BytecodeArg::Int(arg0),
            arg1: BytecodeArg::Colour(ColourFormat::Rgba, r, g, b, a),
        };

        self.program.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }

    fn bytecode_modify(&mut self, index: usize, op: Opcode, arg0: i32, arg1: i32) -> SenResult<()> {
        self.program.code[index] = Bytecode {
            op: op,
            arg0: BytecodeArg::Int(arg0),
            arg1: BytecodeArg::Int(arg1),
        };

        Ok(())
    }

    fn bytecode_modify_arg0_i32(&mut self, index: usize, arg0: i32) -> SenResult<()> {
        let arg1 = self.program.code[index].arg1;
        let op = self.program.code[index].op;

        self.program.code[index] = Bytecode {
            op: op,
            arg0: BytecodeArg::Int(arg0),
            arg1: arg1,
        };

        Ok(())
    }

    fn bytecode_modify_arg1_i32(&mut self, index: usize, arg1: i32) -> SenResult<()> {
        let arg0 = self.program.code[index].arg0;
        let op = self.program.code[index].op;

        self.program.code[index] = Bytecode {
            op: op,
            arg0: arg0,
            arg1: BytecodeArg::Int(arg1),
        };

        Ok(())
    }

    fn is_list_beginning_with(&self, n: &Node, kw: Keyword) -> bool {
        if let Node::List(nodes, _) = n {
            if nodes.len() > 0 {
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

fn count_children(parent: &Node) -> SenResult<usize> {
    match parent {
        Node::List(children, _) | Node::Vector(children, _) => Ok(children.len()),
        _ => Err(SenError::Compiler(format!("count_children"))),
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
        Node::Name(text, i_text, _) => Some(Node::Name(text.to_string(), *i_text, None)),
        Node::Label(text, i_text, _) => Some(Node::Label(text.to_string(), *i_text, None)),
        Node::String(text, _) => Some(Node::String(text.to_string(), None)),
        Node::Whitespace(_, _) => None,
        Node::Comment(_, _) => None,
    }
}

pub fn compile_preamble() -> SenResult<Program> {
    let mut program = Program::new();
    {
        let mut compilation = Compilation::new(&mut program);
        compilation.register_top_level_preamble()?;
        compilation.compile_preamble()?;
    }

    Ok(program)
}

pub fn compile_program(complete_ast: &Vec<Node>) -> SenResult<Program> {
    let mut program = Program::new();
    {
        let mut compilation = Compilation::new(&mut program);

        // clean the complete_ast of whitespace and comment nodes
        //
        let mut ast: Vec<Node> = Vec::new();
        for n in complete_ast.iter() {
            if let Some(useful_node) = clean_node(n) {
                ast.push(useful_node);
            }
        }

        compilation.compile_common(&ast)?;
    }

    Ok(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::parse;

    fn compile(s: &str) -> Program {
        let (ast, _word_lut) = parse(s).unwrap();
        compile_program(&ast).unwrap()
    }

    fn bytecode_from_opcode(op: Opcode) -> Bytecode {
        Bytecode {
            op: op,
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
            arg0: BytecodeArg::Int(Mem::Argument as i32),
            arg1: BytecodeArg::Int(val),
        }
    }

    fn load_const_f32(val: f32) -> Bytecode {
        Bytecode {
            op: Opcode::LOAD,
            arg0: BytecodeArg::Int(Mem::Constant as i32),
            arg1: BytecodeArg::Float(val),
        }
    }

    fn load_const_i32(val: i32) -> Bytecode {
        Bytecode {
            op: Opcode::LOAD,
            arg0: BytecodeArg::Int(Mem::Constant as i32),
            arg1: BytecodeArg::Int(val),
        }
    }

    fn load_global_i32(val: i32) -> Bytecode {
        Bytecode {
            op: Opcode::LOAD,
            arg0: BytecodeArg::Int(Mem::Global as i32),
            arg1: BytecodeArg::Int(val),
        }
    }

    fn load_void() -> Bytecode {
        Bytecode {
            op: Opcode::LOAD,
            arg0: BytecodeArg::Int(Mem::Void as i32),
            arg1: BytecodeArg::Int(0),
        }
    }

    fn lt() -> Bytecode {
        bytecode_from_opcode(Opcode::LT)
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
            arg0: BytecodeArg::Int(Mem::Argument as i32),
            arg1: BytecodeArg::Int(val),
        }
    }

    fn store_global(val: i32) -> Bytecode {
        Bytecode {
            op: Opcode::STORE,
            arg0: BytecodeArg::Int(Mem::Global as i32),
            arg1: BytecodeArg::Int(val),
        }
    }

    #[test]
    fn sanity_check_compile_preamble() {
        // stupid, brittle test just to check that the preamble is creating something
        let preamble = compile_preamble().unwrap();
        assert_eq!(preamble.code.len(), 111);
    }

    #[test]
    fn test_basics() {
        // f32
        assert_eq!(compile("34").code, [jump(1), load_const_f32(34.0), stop()]);
        // 2d vector of f32
        assert_eq!(
            compile("[23 45]").code,
            [
                jump(1),
                load_const_f32(23.0),
                load_const_f32(45.0),
                squish2(),
                stop(),
            ]
        );
        // vector of f32
        assert_eq!(
            compile("[23 45 67 89]").code,
            [
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
            compile("(sqrt 144)").code,
            [
                jump(1),
                load_const_f32(144.0),
                sqrt(),
                stop(),
            ]
        );

        assert_eq!(
            compile("(define brush 9 b 10)").code,
            [
                jump(1),
                load_const_f32(9.0),
                store_global(14),
                load_const_f32(10.0),
                store_global(15),
                stop(),
            ]
        );

        assert_eq!(
            compile("(define brush 9 b 10) (+ brush b)").code,
            [
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
        assert_eq!(compile("(fn (foo a: 0 b: 0) (+ a b))").code,
                   [
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
                   ]);
    }

    #[test]
    fn test_if() {
        assert_eq!(compile("(if (< 3 23) 4 5)").code,
                   [
                       jump(1),
                       load_const_f32(3.0),
                       load_const_f32(23.0),
                       lt(),
                       jump_if(3),
                       load_const_f32(4.00),
                       jump(2),
                       load_const_f32(5.00),
                       stop()
                   ]);
    }

    #[test]
    fn test_fn_invocation() {
        assert_eq!(compile("(fn (adder a: 99 b: 88)
                                (+ a b))
                            (adder a: 3 b: 7)").code,
                   [
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
                   ]);
    }
}
