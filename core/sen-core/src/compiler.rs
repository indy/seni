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
use parser::Node;
use opcodes::{Opcode, opcode_stack_offset};
use keywords::{Keyword, keyword_to_string};

type Placeholder = i32;

const TAU: f32 = 6.283_185_307_179_586; // todo: move TAU to math

const PLACEHOLDER: Placeholder = 0;

const MEMORY_LOCAL_SIZE: usize = 40;

#[derive(Clone, Copy)]
pub enum MemorySegmentType {
    Argument = 0, // store the function's arguments
    Local = 1,    // store the function's local arguments
    Global = 2,   // global variables shared by all functions
    Constant = 3, // pseudo-segment holds constants in range 0..32767
    Void = 4,     // nothing
}

impl fmt::Display for MemorySegmentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MemorySegmentType::Argument => write!(f, "ARG"),
            MemorySegmentType::Local => write!(f, "LOCAL"),
            MemorySegmentType::Global => write!(f, "GLOBAL"),
            MemorySegmentType::Constant => write!(f, "CONST"),
            MemorySegmentType::Void => write!(f, "VOID"),
        }
    }
}

impl MemorySegmentType {
    fn from_bytecode_arg(b: &BytecodeArg) -> SenResult<MemorySegmentType> {
        match b {
            BytecodeArg::Int(i) => match i {
                0 => Ok(MemorySegmentType::Argument),
                1 => Ok(MemorySegmentType::Local),
                2 => Ok(MemorySegmentType::Global),
                3 => Ok(MemorySegmentType::Constant),
                4 => Ok(MemorySegmentType::Void),
                _ => Err(SenError::MemorySegmentTypeUnmappableI32)
            },
            _ => Err(SenError::MemorySegmentTypeUnmappableBytecodeArg)
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
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
            BytecodeArg::Colour(s, a, b, c, d) => write!(f, "Colour({} {} {} {} {})", s, a, b, c, d),
        }
    }
}

#[derive(Debug)]
pub struct Bytecode {
    pub op: Opcode,
    pub arg0: BytecodeArg,
    pub arg1: BytecodeArg,
}

impl fmt::Display for Bytecode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.op {
            Opcode::LOAD | Opcode::STORE | Opcode::STORE_F => {
                let mem = MemorySegmentType::from_bytecode_arg(&self.arg0).map_err(|_| ::std::fmt::Error)?;
                write!(f, "{}\t{}\t{}", self.op, mem, self.arg1)
            },
            // todo: format JUMP and JUMP_IF
            Opcode::JUMP | Opcode::JUMP_IF => write!(f, "{}\t{}\t{}", self.op, self.arg0, self.arg1),
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
    active: bool,               // todo: probably not needed anymore
    index: usize,
    fn_name_str: String,
    fn_name: i32,
    arg_address: i32,
    body_address: i32,
    num_args: i32,
    argument_offsets: Vec<String>,
}

impl FnInfo {
    fn new(fn_name_str: String, index: usize) -> FnInfo {
        FnInfo {
            active: true,
            index: index,
            fn_name_str: fn_name_str,
            fn_name: 0,   // todo: fix/remove?
            arg_address: 0,
            body_address: 0,
            num_args: 0,
            argument_offsets: Vec::new(),
        }
    }

    fn get_argument_mapping(&self, argument_name: &String) -> Option<usize> {
        for (i, arg) in self.argument_offsets.iter().enumerate() {
            if arg == argument_name {
                return Some((i * 2) + 1)
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
        unimplemented!();
    }

    pub fn get_fn_info(&self, node: &Node) -> SenResult<&FnInfo> {
        unimplemented!();
    }
}

struct Compilation<'a> {
    pub program: &'a mut Program,

    pub opcode_offset: i32,

    pub global_mappings: HashMap<String, i32>,
    pub global_mapping_marker: i32,

    pub local_mappings: HashMap<String, i32>,
    pub local_mapping_marker: i32, // todo: check that it is < MEMORY_LOCAL_SIZE, as that constant is used in the interpreter

    pub current_fn_info: Placeholder,
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

            current_fn_info: PLACEHOLDER,
        }
    }

    pub fn clear_global_mappings(&mut self) {
        self.global_mappings.clear();
        self.global_mapping_marker = 0;
    }

    fn add_global_mapping(&mut self, s: String) -> SenResult<i32> {
        self.global_mappings.insert(s, self.global_mapping_marker);
        self.global_mapping_marker += 1;
        Ok(self.global_mapping_marker - 1)
    }

    pub fn clear_local_mappings(&mut self) {
        self.local_mappings.clear();
        self.local_mapping_marker = 0;
    }

    fn add_local_mapping(&mut self, s: String) -> SenResult<i32> {
        self.local_mappings.insert(s, self.local_mapping_marker);
        self.local_mapping_marker += 1;
        Ok(self.local_mapping_marker - 1)
    }

    fn add_internal_local_mapping(&mut self, s: String) -> SenResult<i32> {
        unimplemented!();
    }

    fn correct_function_addresses(&mut self) -> SenResult<()> {
        unimplemented!();
    }

    fn get_fn_info_index(&self, node: &Node) -> Option<usize> {
        unimplemented!();
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
        self.emit_opcode_i32(Opcode::STOP, 0, 0)?;

        Ok(())
    }

    fn compile_global_bind_procedural_presets(&mut self) -> SenResult<()> {
        // create a vector
        self.emit_opcode_i32(Opcode::LOAD, MemorySegmentType::Void as i32, 0)?;

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
        self.emit_opcode_i32(Opcode::LOAD, MemorySegmentType::Void as i32, 0)?;

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
        self.clear_global_mappings();
        self.clear_local_mappings();
        // compilation->current_fn_info = NULL;

        self.register_top_level_preamble()?;
        self.register_top_level_fns(ast)?;
        self.register_top_level_defines(ast)?;

        Ok(())
    }

    fn compile_common_top_level_fns(&mut self, ast: &Vec<Node>) -> SenResult<()> {
        // a placeholder, filled in at the end of this function
        self.emit_opcode_i32(Opcode::JUMP, 0, 0)?;
        let start_index = self.program.code.len() - 1;

        // compile the top-level functions
        let fn_keyword_string = keyword_to_string(Keyword::Fn);
        for n in ast.iter() {
            if is_list_beginning_with(n, &fn_keyword_string) {
                self.compile(n)?;    // todo: the c-impl returns a node to continue from
            }
        }

        // jump to the program's starting address
        self.program.code[start_index] = Bytecode {
            op: Opcode::JUMP,
            arg0: BytecodeArg::Int(self.program.code.len() as i32),
            arg1: BytecodeArg::Int(0),
        };

        Ok(())
    }

    fn compile_common_top_level_defines(&mut self, ast: &Vec<Node>) -> SenResult<()> {
        let define_keyword_string = keyword_to_string(Keyword::Define);
        for n in ast.iter() {
            if is_list_beginning_with(n, &define_keyword_string) {
                self.compile_define(n, MemorySegmentType::Global)?;
            }
        }
        Ok(())
    }

    fn compile_common_top_level_forms(&mut self, ast: &Vec<Node>) -> SenResult<()> {
        let define_keyword_string = keyword_to_string(Keyword::Define);
        let fn_keyword_string = keyword_to_string(Keyword::Fn);

        for n in ast.iter() {
            if !is_list_beginning_with(n, &define_keyword_string) && !is_list_beginning_with(n, &fn_keyword_string) {
                self.compile(n)?;
            }
        }
        Ok(())
    }

    fn compile_common_epilogue(&mut self, ast: &Vec<Node>) -> SenResult<()> {
        self.emit_opcode_i32(Opcode::STOP, 0, 0)?;

        // now update the addreses used by CALL and CALL_0
        self.correct_function_addresses()?;

        Ok(())
    }

    fn register_top_level_fns(&mut self, ast: &Vec<Node>) -> SenResult<()> {
        let fn_keyword_string = keyword_to_string(Keyword::Fn);

        let mut num_fns: usize = 0;

        // clear all data
        self.program.fn_info = Vec::new();

        // register top level fns
        for n in ast.iter() {
            if is_list_beginning_with(n, &fn_keyword_string) {
                // get the name of the fn
                if let Node::List(nodes, _) = n {
                    if nodes.len() < 2 {
                        // a list with just the 'fn' keyword ???
                        return Err(SenError::GeneralError) // malformed function definition
                    }
                    let name_and_params = &nodes[1];
                    if let Node::List(np_nodes, _) = name_and_params {
                        if np_nodes.len() > 0 {
                            let name_node = &np_nodes[0];
                            if let Node::Name(text, _) = name_node {
                                // we have a named top-level fn declaration
                                //
                                // create and add a top level fn
                                let fn_info = FnInfo::new(text.to_string(), num_fns);
                                num_fns += 1;
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
            Node::Name(name, _) => {
                // (define foo 42)
                if let Err(e) = self.add_global_mapping(name.to_string()) {
                    return Err(e)
                }
            },
            Node::List(nodes, _) | Node::Vector(nodes, _) => {
                // (define [a b] (something))
                // (define [a [x y]] (something))
                for n in nodes.iter() {
                    if let Err(e) = self.register_names_in_define(n) {
                        return Err(e)
                    }
                }
            },
            _ => ()

        }
        Ok(())
    }

    fn register_top_level_defines(&mut self, ast: &Vec<Node>) -> SenResult<()> {
        let define_keyword_string = keyword_to_string(Keyword::Define);

        for n in ast.iter() {
            if let Node::List(nodes, _) = n {
                if nodes.len() > 0 {
                    let define_keyword = &nodes[0];
                    if let Node::Name(text, _) = define_keyword {
                        if text == &define_keyword_string {
                            let mut defs = &nodes[1..];
                            while defs.len() > 1 {
                                if let Err(e) = self.register_names_in_define(&defs[0]) {
                                    return Err(e)
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

    fn compile_define(&mut self, ast: &Node, mem: MemorySegmentType) -> SenResult<()> {
        // ast is a list beginning with 'define'

        // (define a 10 b 20 c 30)
        if let Node::List(children, _) = ast {
            let mut defs = &children[1..]; // remove the initial 'define'

            if defs.len() % 2 != 0 {
                // log: should be an even number of elements
                return Err(SenError::GeneralError)
            }

            while defs.len() > 0 {
                let lhs_node = &defs[0];
                let value_node = &defs[1];

                self.compile(&value_node)?;

                match lhs_node {
                    Node::Name(_, _) => {
                        // define foo 10
                        self.store_from_stack_to_memory(&lhs_node, mem)?;
                    },
                    Node::Vector(children, _) => {
                        // define [a b] (something-that-returns-a-vector ...)

                        // check if we can use the PILE opcode
                        if all_children_are_name_nodes(lhs_node) {

                            let num_children = children.len();

                            // PILE will stack the elements in the rhs vector in order,
                            // so the lhs values have to be popped in reverse order
                            self.emit_opcode_i32(Opcode::PILE, num_children as i32, 0)?;
                            self.opcode_offset = self.opcode_offset + num_children as i32 - 1;

                            for c in children.iter().rev() {
                                self.store_from_stack_to_memory(&c, mem)?;
                            }
                        } else {
                            // all nodes in lhs vector definition should be names
                            // note: this means that recursive name assignments aren't implemented
                            // e.g. (define [a [b c]] something)
                            return Err(SenError::GeneralError)
                        }
                    },
                    _ => return Err(SenError::GeneralError)
                }

                defs = &defs[2..];
            }
        } else {
            // expected a list beginning with 'define'
            // should never get here because of earlier check
            return Err(SenError::GeneralError)
        }

        Ok(())
    }

    fn compile(&mut self, _ast: &Node) -> SenResult<()> {
        Ok(())
    }


    fn compile_global_bind_i32(&mut self, s: String, value: i32) -> SenResult<()> {
        self.emit_opcode_i32(Opcode::LOAD, MemorySegmentType::Constant as i32, value)?;
        self.store_globally(s)?;
        Ok(())
    }

    fn compile_global_bind_f32(&mut self, s: String, value: f32) -> SenResult<()> {
        self.emit_opcode_i32_f32(Opcode::LOAD, MemorySegmentType::Constant as i32, value)?;
        self.store_globally(s)?;
        Ok(())
    }

    fn compile_global_bind_col(&mut self, s: String, r: f32, g: f32, b: f32, a: f32) -> SenResult<()> {
        self.emit_opcode_i32_rgba(Opcode::LOAD, MemorySegmentType::Constant as i32, r, g, b, a)?;
        self.store_globally(s)?;
        Ok(())
    }

    fn append_keyword(&mut self, kw: Keyword) -> SenResult<()> {
        self.emit_opcode_i32_name(Opcode::LOAD, MemorySegmentType::Constant as i32, kw as i32)?;
        self.emit_opcode_i32(Opcode::APPEND, 0, 0)?;
        Ok(())
    }

    fn store_locally(&mut self, s: String) -> SenResult<i32> {
        let address: i32 = match self.local_mappings.get(&s) {
            Some(&local_mapping) => local_mapping, // already storing the binding name
            None => self.add_local_mapping(s)?
        };

        self.emit_opcode_i32(Opcode::STORE, MemorySegmentType::Local as i32, address)?;

        Ok(address)
    }

    fn store_globally(&mut self, s: String) -> SenResult<i32> {
        let address: i32 = match self.global_mappings.get(&s) {
            Some(&global_mapping) => global_mapping, // already storing the binding name
            None => self.add_global_mapping(s)?
        };

        self.emit_opcode_i32(Opcode::STORE, MemorySegmentType::Global as i32, address)?;

        Ok(address)
    }

    fn store_from_stack_to_memory(&mut self, node: &Node, mem: MemorySegmentType) -> SenResult<i32> {
        if let Node::Name(text, _) = node {
            match mem {
                MemorySegmentType::Local => self.store_locally(text.to_string()),
                MemorySegmentType::Global => self.store_globally(text.to_string()),
                _ => Err(SenError::GeneralError)
            }
        } else {
            Err(SenError::GeneralError)
        }
    }

    fn emit_opcode_i32(&mut self, op: Opcode, arg0: i32, arg1: i32) -> SenResult<()> {
        let b = Bytecode {
            op: op,
            arg0: BytecodeArg::Int(arg0),
            arg1: BytecodeArg::Int(arg1),
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

    fn emit_opcode_i32_rgba(&mut self, op: Opcode, arg0: i32, r: f32, g: f32, b: f32, a: f32) -> SenResult<()> {
        let b = Bytecode {
            op: op,
            arg0: BytecodeArg::Int(arg0),
            arg1: BytecodeArg::Colour(ColourFormat::Rgba, r, g, b, a),
        };

        self.program.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }
}

fn is_list_beginning_with(n: &Node, s: &String) -> bool {
    if let Node::List(nodes, _) = n {
        if nodes.len() > 0 {
            if let Node::Name(ref text, _) = nodes[0] {
                return text == s
            }
        }
    }
    false
}

// renamed all_children_have_type as it's only used with children of type NAME
fn all_children_are_name_nodes(parent: &Node) -> bool {
    match parent {
        Node::List(children, _) | Node::Vector(children, _) => {
            for n in children.iter() {
                if let Node::Name(_, _) = n {
                    continue
                } else {
                    return false
                }
            }
            true
        },
        _ => false
    }
}

fn count_children(parent: &Node) ->SenResult<usize> {
    match parent {
        Node::List(children, _) | Node::Vector(children, _) => {
            Ok(children.len())
        },
        _ => Err(SenError::GeneralError)
    }
}

pub struct Compiler {
    preamble: Option<Program>,
}

fn clean_node(node: &Node) -> Option<Node> {
    match node {
        Node::List(nodes, _) => {
            let mut vn: Vec<Node> = Vec::new();
            for n in nodes.iter() {
                if let Some(cleaned) = clean_node(n) {
                    vn.push(cleaned);
                }
            };
            Some(Node::List(vn, None))
        },
        Node::Vector(nodes, _) => {
            let mut vn: Vec<Node> = Vec::new();
            for n in nodes.iter() {
                if let Some(cleaned) = clean_node(n) {
                    vn.push(cleaned);
                }
            };
            Some(Node::Vector(vn, None))
        },
        Node::Float(f, _) => Some(Node::Float(*f, None)),
        Node::Name(text, _) => Some(Node::Name(text.to_string(), None)),
        Node::Label(text, _) => Some(Node::Label(text.to_string(), None)),
        Node::String(text, _) => Some(Node::String(text.to_string(), None)),
        Node::Whitespace(_, _) => None,
        Node::Comment(_, _) => None,
    }
}

impl Compiler {
    fn new() -> Self {
        Compiler {
            preamble: None
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

}

pub fn compile_program(_ast: Vec<Node>) -> SenResult<Program> {
    //    Ok(Program { placeholder: 0 })
    Err(SenError::GeneralError)
}
