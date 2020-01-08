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

use crate::colour::{Colour, ColourFormat};
use crate::constants;
use crate::error::{Error, Result};
use crate::gene::Genotype;
use crate::iname::Iname;
use crate::keywords::{name_to_keyword_hash, Keyword};
use crate::mathutil;
use crate::native::{name_to_native_hash, parameter_info, Native};
use crate::node::Node;
use crate::opcodes::{opcode_stack_offset, Opcode};
use crate::parser::WordLut;
use crate::program::{Bytecode, BytecodeArg, Data, FnInfo, Mem, Program};
use crate::vm::{Var, MEMORY_LOCAL_SIZE};
use log::error;
use std::collections::{BTreeMap, HashMap};
use std::fmt;

const NONSENSE: i32 = 666;

pub fn compile_preamble() -> Result<Program> {
    let mut c = Compilation::new();
    let compiler = Compiler::new();

    compiler.register_top_level_preamble(&mut c)?;
    compiler.compile_preamble(&mut c)?;

    Ok(Program {
        code: c.code,
        fn_info: c.fn_info,
        ..Default::default()
    })
}

pub fn compile_program(ast: &[Node], word_lut: &WordLut) -> Result<Program> {
    let mut c = Compilation::new();
    let compiler = Compiler::new();

    compiler.compile_common(&mut c, &ast)?;

    let mut data: Data = Default::default();
    data.strings = word_lut.get_script_inames();

    Ok(Program {
        code: c.code,
        fn_info: c.fn_info,
        data,
    })
}

pub fn compile_program_1(ast_node: &Node, word_lut: &WordLut) -> Result<Program> {
    let mut c = Compilation::new();
    let compiler = Compiler::new();

    compiler.compile_common_1(&mut c, &ast_node)?;

    let mut data: Data = Default::default();
    data.strings = word_lut.get_script_inames();

    Ok(Program {
        code: c.code,
        fn_info: c.fn_info,
        data,
    })
}

pub fn compile_program_for_trait(
    ast: &[Node],
    word_lut: &WordLut,
    global_mapping: &BTreeMap<Iname, usize>,
) -> Result<Program> {
    let mut c = Compilation::new();
    let compiler = Compiler::new();

    let ast = only_semantic_nodes(ast);

    compiler.compile_common_prologue(&mut c)?;
    compiler.register_top_level_fns(&mut c, &ast)?;
    compiler.register_top_level_defines(&mut c, &ast)?;

    compiler.compile_common_top_level_fns(&mut c, &ast)?;
    compiler.compile_global_mappings_for_trait(&mut c, global_mapping)?;
    compiler.compile_common_top_level_defines(&mut c, &ast)?;
    compiler.compile_common_top_level_forms(&mut c, &ast)?;
    compiler.compile_common_epilogue(&mut c)?;

    let mut data: Data = Default::default();
    data.strings = word_lut.get_script_inames();

    Ok(Program {
        code: c.code,
        fn_info: c.fn_info,
        data,
    })
}

pub fn compile_program_with_genotype(
    ast: &mut [Node],
    word_lut: &WordLut,
    genotype: &mut Genotype,
) -> Result<Program> {
    let mut c = Compilation::new();
    let mut compiler = Compiler::new();

    compiler.use_genes = true;
    assign_genotype_to_ast(ast, genotype)?;
    compiler.compile_common(&mut c, &ast)?;

    let mut data: Data = Default::default();
    data.strings = word_lut.get_script_inames();

    Ok(Program {
        code: c.code,
        fn_info: c.fn_info,
        data,
    })
}

fn assign_genotype_to_ast(ast: &mut [Node], genotype: &mut Genotype) -> Result<()> {
    genotype.reset_gene_index();

    for n in ast {
        assign_genes_to_nodes(n, genotype)?;
    }

    Ok(())
}

fn assign_gene(n: &Node, genotype: &mut Genotype) -> Result<Node> {
    match n {
        Node::Vector(meta, ns) => Ok(Node::Vector(
            meta.new_with_gene(genotype.clone_next_gene()?),
            ns.clone(),
        )),
        Node::Float(meta, f, s) => Ok(Node::Float(
            meta.new_with_gene(genotype.clone_next_gene()?),
            *f,
            s.to_string(),
        )),
        _ => {
            error!("assign_gene: element neither vector nor float");
            Err(Error::Compiler)
        }
    }
}

fn assign_genes_to_each_node_in_vector(
    elements: &mut Vec<Node>,
    genotype: &mut Genotype,
) -> Result<Vec<Node>> {
    elements
        .iter()
        .filter(|n| n.is_semantic())
        .map(|n| assign_gene(n, genotype))
        .collect()
}

fn assign_genes_to_nodes(node: &mut Node, genotype: &mut Genotype) -> Result<()> {
    match node {
        Node::List(meta, ref mut ns) => {
            if let Some(ref mut gene_info) = meta.gene_info {
                gene_info.gene = Some(genotype.clone_next_gene()?);
            }
            for n in ns {
                assign_genes_to_nodes(n, genotype)?;
            }
        }
        Node::Vector(meta, ref mut ns) => {
            if meta.gene_info.is_some() {
                let ns_with_genes = assign_genes_to_each_node_in_vector(ns, genotype)?;

                ns.clear();
                for n in ns_with_genes {
                    ns.push(n);
                }
            } else {
                for n in ns {
                    assign_genes_to_nodes(n, genotype)?;
                }
            }
        }
        Node::Float(meta, _, _)
        | Node::FromName(meta, _, _)
        | Node::Name(meta, _, _)
        | Node::Label(meta, _, _)
        | Node::String(meta, _, _)
        | Node::Tilde(meta)
        | Node::Whitespace(meta, _)
        | Node::Comment(meta, _) => {
            if let Some(ref mut gene_info) = meta.gene_info {
                gene_info.gene = Some(genotype.clone_next_gene()?);
            }
        }
    }

    Ok(())
}

fn is_node_colour_constructor(children: &[&Node]) -> bool {
    if !children.is_empty() {
        if let Node::Name(_, _, iname) = children[0] {
            let col_constructor_start = Iname::from(Native::ColConstructorStart_);
            let col_constructor_end = Iname::from(Native::ColConstructorEnd_);

            return iname.enclosed_by(col_constructor_start, col_constructor_end);
        }
    }
    false
}

#[derive(Debug)]
pub struct Compilation {
    code: Vec<Bytecode>,

    fn_info: Vec<FnInfo>,
    current_fn_info_index: Option<usize>,
    opcode_offset: i32,

    local_mappings: HashMap<Iname, usize>, // iname -> local mapping index
    local_mapping_marker: usize,

    global_mappings: HashMap<Iname, usize>, // iname -> global mapping index
    global_mapping_marker: usize,

    // using BTreeMap as this will be given to a TraitList which will be packed,
    // for testing purposes having a consistent ordering is important
    user_defined_globals: BTreeMap<Iname, usize>, // iname -> global mapping index
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
    pub fn new() -> Self {
        Compilation {
            code: Vec::new(),

            fn_info: Vec::new(),
            current_fn_info_index: None,
            opcode_offset: 0,

            local_mappings: HashMap::new(),
            local_mapping_marker: 0,

            global_mappings: HashMap::new(),
            global_mapping_marker: 0,

            // the subset of global_mappings that are defined by the user script
            user_defined_globals: BTreeMap::new(),
        }
    }

    fn clear_global_mappings(&mut self) -> Result<()> {
        self.global_mappings.clear();
        self.global_mapping_marker = 0;
        Ok(())
    }

    // used when adding explicit global mappings during a trait c
    fn add_explicit_global_mapping(&mut self, iname: Iname, map_val: usize) {
        self.global_mappings.insert(iname, map_val);
    }

    fn add_global_mapping(&mut self, iname: Iname) -> Result<usize> {
        self.user_defined_globals
            .insert(iname, self.global_mapping_marker);
        self.global_mappings
            .insert(iname, self.global_mapping_marker);
        self.global_mapping_marker += 1;
        Ok(self.global_mapping_marker - 1)
    }

    fn add_global_mapping_for_keyword(&mut self, kw: Keyword) -> Result<usize> {
        // self.add_global_mapping(kw as i32)
        self.global_mappings
            .insert(Iname::from(kw), self.global_mapping_marker);
        self.global_mapping_marker += 1;
        Ok(self.global_mapping_marker - 1)
    }

    fn get_global_mapping(&self, iname: Iname) -> Option<&usize> {
        self.global_mappings.get(&iname)
    }

    pub fn get_user_defined_globals(self) -> BTreeMap<Iname, usize> {
        self.user_defined_globals
    }

    fn clear_local_mappings(&mut self) -> Result<()> {
        self.local_mappings.clear();
        self.local_mapping_marker = 0;
        Ok(())
    }

    fn add_local_mapping(&mut self, iname: Iname) -> Result<usize> {
        self.local_mappings.insert(iname, self.local_mapping_marker);
        self.local_mapping_marker += 1;

        if self.local_mapping_marker >= MEMORY_LOCAL_SIZE {
            error!("add_local_mapping: exceeded MEMORY_LOCAL_SIZE");
            Err(Error::Compiler)
        } else {
            Ok(self.local_mapping_marker - 1)
        }
    }

    fn get_local_mapping(&self, iname: Iname) -> Option<&usize> {
        self.local_mappings.get(&iname)
    }

    // we want a local mapping that's going to be used to store an internal variable
    // (e.g. during a fence loop)
    // note: it's up to the caller to manage this reference
    fn add_internal_local_mapping(&mut self) -> Result<usize> {
        // todo: is this right???
        let i = 9999;
        let n = Iname::new(i);

        // let s = "internal_local_mapping".to_string();
        self.local_mappings.insert(n, self.local_mapping_marker);
        self.local_mapping_marker += 1;

        if self.local_mapping_marker >= MEMORY_LOCAL_SIZE {
            error!("add_internal_local_mapping: exceeded MEMORY_LOCAL_SIZE");
            Err(Error::Compiler)
        } else {
            Ok(self.local_mapping_marker - 1)
        }
    }

    fn add_bytecode(&mut self, bc: Bytecode) -> Result<()> {
        self.code.push(bc);
        Ok(())
    }

    fn get_fn_info_index(&self, node: &Node) -> Option<usize> {
        if let Node::Name(_, text, _) = node {
            for (i, fi) in self.fn_info.iter().enumerate() {
                if fi.fn_name == *text {
                    return Some(i);
                }
            }
        }
        None
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

    fn emit_name_as_string(&mut self, op: Opcode, arg0: Mem, arg1: Iname) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Mem(arg0),
            arg1: BytecodeArg::String(arg1),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }

    fn emit_squish(&mut self, num: i32) -> Result<()> {
        self.emit(Opcode::SQUISH, num, 0)?;
        self.opcode_offset -= num; // squish takes values from the stack
        self.opcode_offset += 1; // squish pushes result

        Ok(())
    }
}

trait EmitOpcode<T, U> {
    fn emit(&mut self, op: Opcode, arg0: T, arg1: U) -> Result<()>;
}

impl EmitOpcode<i32, i32> for Compilation {
    fn emit(&mut self, op: Opcode, arg0: i32, arg1: i32) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Int(arg0),
            arg1: BytecodeArg::Int(arg1),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }
}

impl EmitOpcode<usize, i32> for Compilation {
    fn emit(&mut self, op: Opcode, arg0: usize, arg1: i32) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Int(arg0 as i32),
            arg1: BytecodeArg::Int(arg1),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }
}

impl EmitOpcode<usize, usize> for Compilation {
    fn emit(&mut self, op: Opcode, arg0: usize, arg1: usize) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Int(arg0 as i32),
            arg1: BytecodeArg::Int(arg1 as i32),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }
}

impl EmitOpcode<Mem, i32> for Compilation {
    fn emit(&mut self, op: Opcode, arg0: Mem, arg1: i32) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Mem(arg0),
            arg1: BytecodeArg::Int(arg1),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }
}

impl EmitOpcode<Mem, usize> for Compilation {
    fn emit(&mut self, op: Opcode, arg0: Mem, arg1: usize) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Mem(arg0),
            arg1: BytecodeArg::Int(arg1 as i32),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }
}

impl EmitOpcode<Mem, Keyword> for Compilation {
    fn emit(&mut self, op: Opcode, arg0: Mem, arg1: Keyword) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Mem(arg0),
            arg1: BytecodeArg::Keyword(arg1),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }
}

impl EmitOpcode<Mem, Iname> for Compilation {
    fn emit(&mut self, op: Opcode, arg0: Mem, arg1: Iname) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Mem(arg0),
            arg1: BytecodeArg::Name(arg1),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }
}

impl EmitOpcode<Mem, f32> for Compilation {
    fn emit(&mut self, op: Opcode, arg0: Mem, arg1: f32) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Mem(arg0),
            arg1: BytecodeArg::Float(arg1),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }
}

impl EmitOpcode<Mem, Colour> for Compilation {
    fn emit(&mut self, op: Opcode, arg0: Mem, arg1: Colour) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Mem(arg0),
            arg1: BytecodeArg::Colour(arg1),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }
}

impl EmitOpcode<Native, i32> for Compilation {
    fn emit(&mut self, op: Opcode, arg0: Native, arg1: i32) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Native(arg0),
            arg1: BytecodeArg::Int(arg1),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }
}

impl EmitOpcode<Native, usize> for Compilation {
    fn emit(&mut self, op: Opcode, arg0: Native, arg1: usize) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Native(arg0),
            arg1: BytecodeArg::Int(arg1 as i32),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }
}

impl EmitOpcode<i32, f32> for Compilation {
    fn emit(&mut self, op: Opcode, arg0: i32, arg1: f32) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Int(arg0),
            arg1: BytecodeArg::Float(arg1),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }
}

impl EmitOpcode<usize, Iname> for Compilation {
    fn emit(&mut self, op: Opcode, arg0: usize, arg1: Iname) -> Result<()> {
        let b = Bytecode {
            op,
            arg0: BytecodeArg::Int(arg0 as i32),
            arg1: BytecodeArg::Name(arg1),
        };

        self.add_bytecode(b)?;
        self.opcode_offset += opcode_stack_offset(op);

        Ok(())
    }
}

pub struct Compiler {
    name_to_keyword: HashMap<Iname, Keyword>,
    name_to_native: HashMap<Iname, Native>,
    use_genes: bool,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            name_to_keyword: name_to_keyword_hash(),
            name_to_native: name_to_native_hash(),
            use_genes: false,
        }
    }

    fn correct_function_addresses(&self, c: &mut Compilation) -> Result<()> {
        let mut all_fixes: Vec<(usize, Opcode, Mem, i32)> = Vec::new(); // tuple of index, op, arg0, ???
        let mut arg1_fixes: Vec<(usize, i32)> = Vec::new(); // tuple of index,value pairs

        // go through the bytecode fixing up function call addresses
        for (i, bc) in c.code.iter().enumerate() {
            // replace the temporarily stored index in the args of CALL and CALL_0 with
            // the actual values

            match bc.op {
                Opcode::CALL => {
                    if let BytecodeArg::Int(fn_info_index) = bc.arg0 {
                        let fn_info = &c.fn_info[fn_info_index as usize];

                        // the previous two bytecodes will be LOADs of CONST.
                        // i - 2 == the address to call
                        // i - 1 == the number of arguments used by the function
                        arg1_fixes.push((i - 2, fn_info.arg_address as i32));
                        arg1_fixes.push((i - 1, fn_info.num_args as i32));
                    }
                }
                Opcode::CALL_0 => {
                    if let BytecodeArg::Int(fn_info_index) = bc.arg0 {
                        let fn_info = &c.fn_info[fn_info_index as usize];
                        arg1_fixes.push((i - 1, fn_info.body_address as i32));
                    }
                }
                Opcode::PLACEHOLDER_STORE => {
                    // opcode's arg0 is the fn_info_index and arg1 is the label_value
                    if let BytecodeArg::Int(fn_info_index) = bc.arg0 {
                        let fn_info = &c.fn_info[fn_info_index as usize];
                        if let BytecodeArg::Name(label_value) = bc.arg1 {
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
            c.bytecode_modify_mem(index, op, arg0, arg1)?;
        }
        for (index, arg1) in arg1_fixes {
            c.bytecode_modify_arg1_i32(index, arg1)?;
        }

        Ok(())
    }

    fn register_top_level_preamble(&self, c: &mut Compilation) -> Result<()> {
        c.add_global_mapping_for_keyword(Keyword::CanvasCentre)?;
        c.add_global_mapping_for_keyword(Keyword::CanvasWidth)?;
        c.add_global_mapping_for_keyword(Keyword::CanvasHeight)?;
        c.add_global_mapping_for_keyword(Keyword::CanvasSize)?;

        c.add_global_mapping_for_keyword(Keyword::MathPi)?;
        c.add_global_mapping_for_keyword(Keyword::MathTau)?;

        c.add_global_mapping_for_keyword(Keyword::White)?;
        c.add_global_mapping_for_keyword(Keyword::Black)?;
        c.add_global_mapping_for_keyword(Keyword::Red)?;
        c.add_global_mapping_for_keyword(Keyword::Green)?;
        c.add_global_mapping_for_keyword(Keyword::Blue)?;
        c.add_global_mapping_for_keyword(Keyword::Yellow)?;
        c.add_global_mapping_for_keyword(Keyword::Magenta)?;
        c.add_global_mapping_for_keyword(Keyword::Cyan)?;

        c.add_global_mapping_for_keyword(Keyword::ColProceduralFnPresets)?;
        c.add_global_mapping_for_keyword(Keyword::EaseAll)?;
        c.add_global_mapping_for_keyword(Keyword::BrushAll)?;

        Ok(())
    }

    fn register_top_level_fns(&self, c: &mut Compilation, ast: &[&Node]) -> Result<()> {
        // clear all data
        c.fn_info = Vec::new();

        // register top level fns
        for n in ast.iter() {
            if let Some(fn_info) = self.register_top_level_fns_1(n)? {
                c.fn_info.push(fn_info);
            }
        }

        Ok(())
    }

    fn register_top_level_fns_1(&self, n: &Node) -> Result<Option<FnInfo>> {
        if self.is_list_beginning_with(n, Keyword::Fn) {
            // get the name of the fn
            if let Node::List(_, nodes) = n {
                let nodes = only_semantic_nodes(nodes);

                if nodes.len() < 2 {
                    // a list with just the 'fn' keyword ???
                    n.error_here("malformed function definition");
                    return Err(Error::Compiler);
                }
                let name_and_params = nodes[1];
                if let Node::List(_, np_nodes) = name_and_params {
                    let np_nodes = only_semantic_nodes(np_nodes);

                    if !np_nodes.is_empty() {
                        let name_node = &np_nodes[0];
                        if let Node::Name(_, text, _) = name_node {
                            // we have a named top-level fn declaration
                            //
                            // create and add a top level fn
                            let fn_info = FnInfo {
                                fn_name: text.to_string(),
                                ..Default::default()
                            };
                            return Ok(Some(fn_info));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    fn register_names_in_define(&self, c: &mut Compilation, lhs: &Node) -> Result<()> {
        error_if_alterable(&lhs, "register_names_in_define")?;

        match lhs {
            Node::Name(_, _, _) => {
                // (define foo 42)
                //let s = name.to_string();
                let iname = self.get_iname(lhs)?;
                if let Some(_i) = c.get_global_mapping(iname) {
                    // name was already added to global_mappings
                    return Ok(());
                }

                if let Err(e) = c.add_global_mapping(iname) {
                    return Err(e);
                }
            }
            Node::List(_, nodes) | Node::Vector(_, nodes) => {
                // (define [a b] (something))
                // (define [a [x y]] (something))
                let nodes = only_semantic_nodes(nodes);

                for n in nodes {
                    if let Err(e) = self.register_names_in_define(c, n) {
                        return Err(e);
                    }
                }
            }
            _ => (),
        }
        Ok(())
    }

    fn register_top_level_defines(&self, c: &mut Compilation, ast: &[&Node]) -> Result<()> {
        for n in ast.iter() {
            self.register_top_level_defines_1(c, n)?;
        }

        Ok(())
    }

    fn register_top_level_defines_1(&self, c: &mut Compilation, n: &Node) -> Result<()> {
        let define_keyword_string = Keyword::Define.to_string();

        if let Node::List(_, nodes) = n {
            let nodes = only_semantic_nodes(nodes);
            if !nodes.is_empty() {
                let define_keyword = &nodes[0];
                if let Node::Name(_, text, _) = define_keyword {
                    if text == &define_keyword_string {
                        let mut defs = &nodes[1..];
                        while defs.len() > 1 {
                            if let Err(e) = self.register_names_in_define(c, &defs[0]) {
                                return Err(e);
                            }
                            defs = &defs[2..];
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn compile_preamble(&self, c: &mut Compilation) -> Result<()> {
        let dim = constants::CANVAS_DIM;
        let half_dim = constants::CANVAS_DIM / 2.0;

        // ********************************************************************************
        // NOTE: each entry should have a corresponding entry in
        // register_top_level_preamble
        // ********************************************************************************
        self.compile_global_bind_kw_v2d(c, Keyword::CanvasCentre, half_dim, half_dim)?;
        self.compile_global_bind_kw_f32(c, Keyword::CanvasWidth, dim)?;
        self.compile_global_bind_kw_f32(c, Keyword::CanvasHeight, dim)?;
        self.compile_global_bind_kw_f32(c, Keyword::CanvasSize, dim)?;

        self.compile_global_bind_kw_f32(c, Keyword::MathPi, mathutil::PI)?;
        self.compile_global_bind_kw_f32(c, Keyword::MathTau, mathutil::TAU)?;

        self.compile_global_bind_kw_col(c, Keyword::White, 1.0, 1.0, 1.0, 1.0)?;
        self.compile_global_bind_kw_col(c, Keyword::Black, 0.0, 0.0, 0.0, 1.0)?;
        self.compile_global_bind_kw_col(c, Keyword::Red, 1.0, 0.0, 0.0, 1.0)?;
        self.compile_global_bind_kw_col(c, Keyword::Green, 0.0, 1.0, 0.0, 1.0)?;
        self.compile_global_bind_kw_col(c, Keyword::Blue, 0.0, 0.0, 1.0, 1.0)?;
        self.compile_global_bind_kw_col(c, Keyword::Yellow, 1.0, 1.0, 0.0, 1.0)?;
        self.compile_global_bind_kw_col(c, Keyword::Magenta, 1.0, 0.0, 1.0, 1.0)?;
        self.compile_global_bind_kw_col(c, Keyword::Cyan, 0.0, 1.0, 1.0, 1.0)?;

        self.compile_global_bind_procedural_presets(c)?;
        self.compile_global_bind_ease_all(c)?;
        self.compile_global_bind_brush_all(c)?;

        // ********************************************************************************
        // NOTE: each entry should have a corresponding entry in
        // register_top_level_preamble
        // ********************************************************************************

        // slap a stop onto the end of this c
        c.emit(Opcode::STOP, 0, 0)?;

        Ok(())
    }

    fn store_global_keyword_vector(
        &self,
        c: &mut Compilation,
        global_name: Keyword,
        kws: Vec<Keyword>,
    ) -> Result<()> {
        let len = kws.len() as i32;
        for kw in kws {
            c.emit(Opcode::LOAD, Mem::Constant, Iname::from(kw))?;
        }
        c.emit_squish(len)?;

        self.store_globally_kw(c, global_name)?;

        Ok(())
    }

    fn compile_global_bind_procedural_presets(&self, c: &mut Compilation) -> Result<()> {
        self.store_global_keyword_vector(
            c,
            Keyword::ColProceduralFnPresets,
            vec![
                Keyword::Chrome,
                Keyword::HotlineMiami,
                Keyword::KnightRider,
                Keyword::Mars,
                Keyword::Rainbow,
                Keyword::Robocop,
                Keyword::Transformers,
            ],
        )?;
        Ok(())
    }

    fn compile_global_bind_ease_all(&self, c: &mut Compilation) -> Result<()> {
        self.store_global_keyword_vector(
            c,
            Keyword::EaseAll,
            vec![
                Keyword::Linear,
                Keyword::EaseQuick,
                Keyword::EaseSlowIn,
                Keyword::EaseSlowInOut,
                Keyword::EaseQuadraticIn,
                Keyword::EaseQuadraticOut,
                Keyword::EaseQuadraticInOut,
                Keyword::EaseCubicIn,
                Keyword::EaseCubicOut,
                Keyword::EaseCubicInOut,
                Keyword::EaseQuarticIn,
                Keyword::EaseQuarticOut,
                Keyword::EaseQuarticInOut,
                Keyword::EaseQuinticIn,
                Keyword::EaseQuinticOut,
                Keyword::EaseQuinticInOut,
                Keyword::EaseSinIn,
                Keyword::EaseSinOut,
                Keyword::EaseSinInOut,
                Keyword::EaseCircularIn,
                Keyword::EaseCircularOut,
                Keyword::EaseCircularInOut,
                Keyword::EaseExponentialIn,
                Keyword::EaseExponentialOut,
                Keyword::EaseExponentialInOut,
                Keyword::EaseElasticIn,
                Keyword::EaseElasticOut,
                Keyword::EaseElasticInOut,
                Keyword::EaseBackIn,
                Keyword::EaseBackOut,
                Keyword::EaseBackInOut,
                Keyword::EaseBounceIn,
                Keyword::EaseBounceOut,
                Keyword::EaseBounceInOut,
            ],
        )?;
        Ok(())
    }

    fn compile_global_bind_brush_all(&self, c: &mut Compilation) -> Result<()> {
        self.store_global_keyword_vector(
            c,
            Keyword::BrushAll,
            vec![
                Keyword::BrushFlat,
                Keyword::BrushA,
                Keyword::BrushB,
                Keyword::BrushC,
                Keyword::BrushD,
                Keyword::BrushE,
                Keyword::BrushF,
                Keyword::BrushG,
            ],
        )?;
        Ok(())
    }

    pub fn compile_common(&self, c: &mut Compilation, ast: &[Node]) -> Result<()> {
        let ast = only_semantic_nodes(ast);
        self.compile_common_prologue(c)?;

        self.register_top_level_fns(c, &ast)?;
        self.register_top_level_defines(c, &ast)?;

        self.compile_common_top_level_fns(c, &ast)?;
        self.compile_common_top_level_defines(c, &ast)?;
        self.compile_common_top_level_forms(c, &ast)?;
        self.compile_common_epilogue(c)?;

        Ok(())
    }

    fn compile_common_1(&self, c: &mut Compilation, n: &Node) -> Result<()> {
        self.compile_common_prologue(c)?;

        // single node version of self.register_top_level_fns(c, ast)?;
        c.fn_info = Vec::new();
        if let Some(fn_info) = self.register_top_level_fns_1(n)? {
            c.fn_info.push(fn_info);
        }

        // single node version of self.register_top_level_defines(c, ast)?;
        self.register_top_level_defines_1(c, n)?;

        //// single node version of self.compile_common_top_level_fns(c, ast)?;
        {
            // a placeholder, filled in at the end of this function
            c.emit(Opcode::JUMP, 0, 0)?;
            let start_index = c.code.len() - 1;

            // compile the top-level functions
            if self.is_list_beginning_with(n, Keyword::Fn) {
                self.compile(c, n)?; // todo: the c-impl returns a node to continue from
            }

            // jump to the c's starting address
            let jump_address = c.code.len() as i32;
            c.bytecode_modify_arg0_i32(start_index, jump_address)?;
        }

        //// single node version of self.compile_common_top_level_defines(c, ast)?;
        {
            if self.is_list_beginning_with(n, Keyword::Define) {
                if let Node::List(_, children) = n {
                    let children = only_semantic_nodes(children);
                    self.compile_define(c, &children[1..], Mem::Global)?;
                }
            }
        }

        //// single node version of self.compile_common_top_level_forms(c, ast)?;
        {
            if !self.is_list_beginning_with(n, Keyword::Define)
                && !self.is_list_beginning_with(n, Keyword::Fn)
            {
                self.compile(c, n)?;
            }
        }

        self.compile_common_epilogue(c)?;

        Ok(())
    }

    fn compile_common_prologue(&self, c: &mut Compilation) -> Result<()> {
        c.clear_global_mappings()?;
        c.clear_local_mappings()?;
        // c->current_fn_info = NULL;

        self.register_top_level_preamble(c)?;

        Ok(())
    }

    fn compile_common_top_level_fns(&self, c: &mut Compilation, ast: &[&Node]) -> Result<()> {
        // a placeholder, filled in at the end of this function
        c.emit(Opcode::JUMP, 0, 0)?;
        let start_index = c.code.len() - 1;

        // compile the top-level functions
        for n in ast.iter() {
            if self.is_list_beginning_with(n, Keyword::Fn) {
                self.compile(c, n)?; // todo: the c-impl returns a node to continue from
            }
        }

        // jump to the c's starting address
        let jump_address = c.code.len() as i32;
        c.bytecode_modify_arg0_i32(start_index, jump_address)?;

        Ok(())
    }

    fn compile_global_mappings_for_trait(
        &self,
        c: &mut Compilation,
        global_mapping: &BTreeMap<Iname, usize>,
    ) -> Result<()> {
        for (iname, map_val) in global_mapping {
            c.add_explicit_global_mapping(*iname, *map_val);
        }
        Ok(())
    }

    fn compile_common_top_level_defines(&self, c: &mut Compilation, ast: &[&Node]) -> Result<()> {
        for n in ast.iter() {
            if self.is_list_beginning_with(n, Keyword::Define) {
                if let Node::List(_, children) = n {
                    let children = only_semantic_nodes(children);
                    self.compile_define(c, &children[1..], Mem::Global)?;
                }
            }
        }
        Ok(())
    }

    fn compile_common_top_level_forms(&self, c: &mut Compilation, ast: &[&Node]) -> Result<()> {
        for n in ast.iter() {
            if !self.is_list_beginning_with(n, Keyword::Define)
                && !self.is_list_beginning_with(n, Keyword::Fn)
            {
                self.compile(c, n)?;
            }
        }
        Ok(())
    }

    fn compile_common_epilogue(&self, c: &mut Compilation) -> Result<()> {
        c.emit(Opcode::STOP, 0, 0)?;

        // now update the addreses used by CALL and CALL_0
        self.correct_function_addresses(c)?;

        Ok(())
    }

    fn compile(&self, c: &mut Compilation, ast: &Node) -> Result<()> {
        // todo: move this out of compile and into the c struct
        match ast {
            Node::List(meta, children) => {
                let children = only_semantic_nodes(children);

                if self.use_genes
                    && meta.gene_info.is_some()
                    && is_node_colour_constructor(&children[..])
                {
                    // we have an alterable colour constructor so just load in the colour value stored in the gene
                    //
                    let col = self.get_colour(ast)?;
                    c.emit(Opcode::LOAD, Mem::Constant, col)?;
                } else {
                    if self.use_genes && meta.gene_info.is_some() {
                        ast.error_here(
                            "given an alterable list that wasn't a colour constructor???",
                        );
                        return Err(Error::Compiler);
                    }
                    self.compile_list(c, &children[..])?
                }
            }
            Node::Float(_, _, _) => {
                let f = self.get_float(ast)?;
                return c.emit(Opcode::LOAD, Mem::Constant, f);
            }
            Node::Vector(_, children) => {
                let children = only_semantic_nodes(children);

                if children.len() == 2 {
                    return self.compile_2d(c, ast, &children[..]);
                } else {
                    return self.compile_vector(c, ast, &children[..]);
                }
            }
            Node::String(_, _, _) => {
                let iname = self.get_iname(ast)?;
                return c.emit_name_as_string(Opcode::LOAD, Mem::Constant, iname);
            }
            Node::Name(_, text, _) => {
                let iname = self.get_iname(ast)?;

                return if self.compile_user_defined_name(c, iname)? {
                    Ok(())
                } else if let Some(kw) = self.name_to_keyword.get(&iname) {
                    c.emit(Opcode::LOAD, Mem::Constant, *kw)?;
                    Ok(())
                } else if let Some(native) = self.name_to_native.get(&iname) {
                    c.emit(Opcode::NATIVE, *native, 0)?;
                    Ok(())
                } else {
                    ast.error_here(&format!(
                        "compile: can't find user defined name or keyword: {}",
                        text
                    ));
                    Err(Error::Compiler)
                };
            }
            Node::FromName(_, text, _) => {
                let iname = self.get_iname(ast)?;

                return if self.compile_user_defined_name(c, iname)? {
                    Ok(())
                } else if let Some(kw) = self.name_to_keyword.get(&iname) {
                    c.emit(Opcode::LOAD, Mem::Constant, *kw)?;
                    Ok(())
                } else if let Some(native) = self.name_to_native.get(&iname) {
                    c.emit(Opcode::NATIVE, *native, 0)?;
                    Ok(())
                } else {
                    ast.error_here(&format!(
                        "compile: can't find user defined name or keyword: {}",
                        text
                    ));
                    Err(Error::Compiler)
                };
            }
            _ => {
                ast.error_here(&format!("compile ast: {:?}", ast));
                return Err(Error::Compiler);
            }
        }

        Ok(())
    }

    fn compile_list(&self, c: &mut Compilation, children: &[&Node]) -> Result<()> {
        if children.is_empty() {
            // should this be an error?
            error!("compile_list no children (should this be an error?)");
            return Err(Error::Compiler);
        }

        match &children[0] {
            Node::List(_, kids) => {
                let kids = only_semantic_nodes(kids);
                self.compile_list(c, &kids[..])?
            }
            Node::FromName(_, _, _) => {
                // syntax sugar for the 'from' parameter
                // e.g. (some-vec.vector/length)
                // is equivalent to (vector/length from: some-vec)

                if children.len() == 1 || !children[1].is_name() {
                    children[1]
                        .error_here("Node::FromName should always be followed by a Node::Name");
                    return Err(Error::Compiler);
                }

                let iname = self.get_iname(&children[1])?;

                if let Some(fn_info_index) = c.get_fn_info_index(&children[1]) {
                    self.compile_fn_invocation_prologue(c, fn_info_index)?;
                    self.compile_fn_invocation_implicit_from(c, fn_info_index, &children[0])?;
                    self.compile_fn_invocation_args(
                        c,
                        &children[0],
                        &children[2..],
                        fn_info_index,
                    )?;
                    self.compile_fn_invocation_epilogue(c, fn_info_index)?;
                } else if let Some(native) = self.name_to_native.get(&iname) {
                    // get the list of arguments
                    // match up the nodes and compile them in argument order

                    let (args, stack_offset) = parameter_info(*native)?;

                    let num_args = args.len();
                    let label_vals = &children[2..];

                    // write the default_mask at the bottom of the stack
                    let mut default_mask: i32 = 0;
                    for (i, (kw, _)) in args.iter().enumerate() {
                        // ignore the from keyword, we're going to set it later on
                        if *kw != Keyword::From
                            && self.get_parameter_index(label_vals, *kw).is_none()
                        {
                            default_mask |= 1 << i;
                        }
                    }
                    c.emit(Opcode::LOAD, Mem::Constant, default_mask)?;

                    // iterating in reverse so that when the native function
                    // is run it can pop the arguments from the stack in the
                    // order it specified
                    // now add the arguments to the stack
                    for (kw, default_value) in args.iter().rev() {
                        if *kw == Keyword::From {
                            self.compile(c, &children[0])?;
                        } else if let Some(idx) = self.get_parameter_index(label_vals, *kw) {
                            // todo: does this need to be self?
                            // compile the node at the given index
                            self.compile(c, label_vals[idx])?;
                        } else {
                            // compile the default argument value from the Var in args
                            self.compile_var_as_load(c, default_value)?;
                        }
                    }

                    c.emit(Opcode::NATIVE, *native, num_args)?;

                    // the vm's opcode_native will modify the stack, no need for the compiler to add STORE VOID opcodes
                    // subtract num_args and the default_mask, also take into account that a value might be returned
                    c.opcode_offset -= (num_args as i32 + 1) - stack_offset;
                }
            }
            Node::Name(_, _, _) => {
                let iname = self.get_iname(&children[0])?;

                if let Some(fn_info_index) = c.get_fn_info_index(&children[0]) {
                    // todo: get_fn_info_index is re-checking that this is a Node::Name

                    self.compile_fn_invocation_prologue(c, fn_info_index)?;
                    self.compile_fn_invocation_args(
                        c,
                        &children[0],
                        &children[1..],
                        fn_info_index,
                    )?;
                    self.compile_fn_invocation_epilogue(c, fn_info_index)?;
                } else if let Some(kw) = self.name_to_keyword.get(&iname) {
                    match *kw {
                        Keyword::Define => self.compile_define(c, &children[1..], Mem::Local)?,
                        Keyword::If => self.compile_if(c, &children[0], &children[1..])?,
                        Keyword::Each => self.compile_each(c, &children[1..])?,
                        Keyword::Loop => self.compile_loop(c, &children[1..])?,
                        Keyword::Fence => self.compile_fence(c, &children[1..])?,
                        Keyword::OnMatrixStack => {
                            self.compile_on_matrix_stack(c, &children[1..])?
                        }
                        Keyword::Fn => self.compile_fn(c, &children[1..])?,
                        Keyword::Plus => self.compile_math(c, &children[1..], Opcode::ADD)?,
                        Keyword::Minus => self.compile_math(c, &children[1..], Opcode::SUB)?,
                        Keyword::Mult => self.compile_math(c, &children[1..], Opcode::MUL)?,
                        Keyword::Divide => self.compile_math(c, &children[1..], Opcode::DIV)?,
                        Keyword::Mod => self.compile_math(c, &children[1..], Opcode::MOD)?,
                        Keyword::Equal => self.compile_math(c, &children[1..], Opcode::EQ)?,
                        Keyword::Lt => self.compile_math(c, &children[1..], Opcode::LT)?,
                        Keyword::Gt => self.compile_math(c, &children[1..], Opcode::GT)?,
                        Keyword::And => self.compile_math(c, &children[1..], Opcode::AND)?,
                        Keyword::Or => self.compile_math(c, &children[1..], Opcode::OR)?,
                        Keyword::Not => {
                            self.compile_next_one(c, &children[0], &children[1..], Opcode::NOT)?
                        }
                        Keyword::Sqrt => {
                            self.compile_next_one(c, &children[0], &children[1..], Opcode::SQRT)?
                        }
                        Keyword::AddressOf => {
                            self.compile_address_of(c, &children[0], &children[1..])?
                        }
                        Keyword::FnCall => self.compile_fn_call(c, &children[1..])?,
                        Keyword::VectorAppend => self.compile_vector_append(c, &children[1..])?,
                        Keyword::Quote => self.compile_quote(c, &children[1..])?,
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
                } else if let Some(native) = self.name_to_native.get(&iname) {
                    // get the list of arguments
                    // match up the nodes and compile them in argument order

                    let (args, stack_offset) = parameter_info(*native)?;

                    let num_args = args.len();
                    let label_vals = &children[1..];

                    // write the default_mask at the bottom of the stack
                    let mut default_mask: i32 = 0;
                    for (i, (kw, _)) in args.iter().enumerate() {
                        if self.get_parameter_index(label_vals, *kw).is_none() {
                            default_mask |= 1 << i;
                        }
                    }
                    c.emit(Opcode::LOAD, Mem::Constant, default_mask)?;

                    // iterating in reverse so that when the native function
                    // is run it can pop the arguments from the stack in the
                    // order it specified
                    // now add the arguments to the stack
                    for (kw, default_value) in args.iter().rev() {
                        if let Some(idx) = self.get_parameter_index(label_vals, *kw) {
                            // todo: does this need to be self?
                            // compile the node at the given index
                            self.compile(c, label_vals[idx])?;
                        } else {
                            // compile the default argument value from the Var in args
                            self.compile_var_as_load(c, default_value)?;
                        }
                    }

                    c.emit(Opcode::NATIVE, *native, num_args)?;

                    // the vm's opcode_native will modify the stack, no need for the compiler to add STORE VOID opcodes
                    // subtract num_args and the default_mask, also take into account that a value might be returned
                    c.opcode_offset -= (num_args as i32 + 1) - stack_offset;
                }
            }
            _ => {
                children[0].error_here("compile_list strange child");
                return Err(Error::Compiler);
            }
        }

        Ok(())
    }

    fn get_parameter_index(&self, label_vals: &[&Node], kw: Keyword) -> Option<usize> {
        let kw_name = Iname::from(kw);

        let mut preceding_node_was_label = false;

        for (i, node) in label_vals.iter().enumerate() {
            // a label
            if let Node::Label(_, _, iname) = node {
                if *iname == kw_name {
                    return Some(i + 1);
                }
                preceding_node_was_label = true;
            } else if let Node::Name(_, _, iname) = node {
                // possibly using the shortened syntax:
                // (some-fn arg1) that's equivalent to (some-fn arg1: arg1)
                //
                if !preceding_node_was_label && *iname == kw_name {
                    return Some(i);
                }
                preceding_node_was_label = false;
            } else {
                preceding_node_was_label = false;
            }
        }

        None
    }

    fn compile_var_as_load(&self, c: &mut Compilation, var: &Var) -> Result<()> {
        match var {
            Var::Float(f) => c.emit(Opcode::LOAD, Mem::Constant, *f)?,
            Var::V2D(x, y) => {
                c.emit(Opcode::LOAD, Mem::Constant, *x)?;
                c.emit(Opcode::LOAD, Mem::Constant, *y)?;
                c.emit_squish(2)?;
            }
            Var::Colour(colour) => {
                // the default_mask
                c.emit(Opcode::LOAD, Mem::Constant, 0)?;
                c.emit(Opcode::LOAD, Mem::Constant, colour.e0)?;
                c.emit(Opcode::LOAD, Mem::Constant, colour.e1)?;
                c.emit(Opcode::LOAD, Mem::Constant, colour.e2)?;
                c.emit(Opcode::LOAD, Mem::Constant, colour.e3)?;
                c.emit(Opcode::NATIVE, Native::ColRGB, 4)?;
                // now update the opcode offset since this is using the NATIVE
                let num_args = 4;
                // subtract the 4 args + 1 default mask, but then add back the return value
                c.opcode_offset -= (num_args + 1) - 1;
            }
            Var::Keyword(kw) => c.emit(Opcode::LOAD, Mem::Constant, *kw)?,
            // Var::Vector(vs) => {
            //     // pushing from the VOID means creating a new, empty vector
            //     c.emit(Opcode::LOAD, Mem::Void, 0)?;
            //     for v in vs {
            //         self.compile_var_as_load(c, v)?;
            //         c.emit(Opcode::APPEND, 0, 0)?;
            //     }
            // }
            _ => {
                error!("unimplemented var c {:?}", var);
                return Err(Error::Compiler);
            }
        };

        Ok(())
    }

    fn compile_define(&self, c: &mut Compilation, children: &[&Node], mem: Mem) -> Result<()> {
        let mut defs = children;
        // defs are an even number of nodes representing binding/value pairs
        // (define a 10 b 20 c 30) -> a 10 b 20 c 30

        if defs.len() % 2 != 0 {
            defs[0].error_here("should be an even number of elements");
            return Err(Error::Compiler);
        }

        while !defs.is_empty() {
            let lhs_node = &defs[0];
            let value_node = &defs[1];

            self.compile(c, &value_node)?;

            match lhs_node {
                Node::Name(_, _, _) => {
                    // define foo 10
                    self.store_from_stack_to_memory(c, &lhs_node, mem)?;
                }
                Node::Vector(_, kids) => {
                    let kids = only_semantic_nodes(kids);

                    // define [a b] (something-that-returns-a-vector ...)

                    // check if we can use the PILE opcode
                    if all_children_are_name_nodes(lhs_node) {
                        let num_kids = kids.len();

                        // PILE will stack the elements in the rhs vector in order,
                        // so the lhs values have to be popped in reverse order
                        c.emit(Opcode::PILE, num_kids, 0)?;
                        c.opcode_offset = c.opcode_offset + num_kids as i32 - 1;

                        for k in kids.iter().rev() {
                            self.store_from_stack_to_memory(c, &k, mem)?;
                        }
                    } else {
                        // all nodes in lhs vector definition should be names
                        // note: this means that recursive name assignments aren't implemented
                        // e.g. (define [a [b c]] something)

                        lhs_node.error_here("recursive name assignments aren't implemented");
                        return Err(Error::Compiler);
                    }
                }
                _ => {
                    lhs_node.error_here("compile_define");
                    return Err(Error::Compiler);
                }
            }

            defs = &defs[2..];
        }

        Ok(())
    }

    fn compile_fence(&self, c: &mut Compilation, children: &[&Node]) -> Result<()> {
        // (fence (x from: 0 to: 5 num: 5) (+ 42 38))
        if children.len() < 2 {
            if children.is_empty() {
                error!("compile_fence requires at least 2 forms");
            } else {
                children[0].error_here("compile_fence requires at least 2 forms");
            }
            return Err(Error::Compiler);
        }

        let parameters_node = &children[0];
        error_if_alterable(&parameters_node, "compile_fence")?;

        if let Node::List(_, kids) = parameters_node {
            let kids = only_semantic_nodes(kids);

            // the looping variable x
            let name_node = &kids[0];

            let mut maybe_from_node: Option<&Node> = None;
            let mut maybe_to_node: Option<&Node> = None;
            let mut maybe_num_node: Option<&Node> = None;

            let mut label_vals = &kids[1..];
            while label_vals.len() > 1 {
                let label = &label_vals[0];
                let value = &label_vals[1];

                if let Node::Label(_, _, iname) = label {
                    if *iname == Iname::from(Keyword::From) {
                        maybe_from_node = Some(&value);
                    } else if *iname == Iname::from(Keyword::To) {
                        maybe_to_node = Some(&value);
                    } else if *iname == Iname::from(Keyword::Num) {
                        maybe_num_node = Some(&value);
                    }
                }

                label_vals = &label_vals[2..];
            }

            // store the quantity
            let num_address = c.add_internal_local_mapping()?;
            if let Some(num_node) = maybe_num_node {
                self.compile(c, num_node)?;
            } else {
                c.emit(Opcode::LOAD, Mem::Constant, 2.0)?;
            }

            c.emit(Opcode::STORE, Mem::Local, num_address)?;

            // reserve a memory location in local memory for a counter from 0 to quantity
            let counter_address = c.add_internal_local_mapping()?;

            c.emit(Opcode::LOAD, Mem::Constant, 0.0)?;
            c.emit(Opcode::STORE, Mem::Local, counter_address)?;

            // delta that needs to be added at every iteration
            //
            // (to - from) / (num - 1)
            if let Some(to_node) = maybe_to_node {
                self.compile(c, to_node)?;
            } else {
                // else default to 1
                c.emit(Opcode::LOAD, Mem::Constant, 1.0)?;
            }

            if let Some(from_node) = maybe_from_node {
                self.compile(c, from_node)?;
            } else {
                // else default to 0
                c.emit(Opcode::LOAD, Mem::Constant, 0.0)?;
            }

            c.emit(Opcode::SUB, 0, 0)?;

            if let Some(num_node) = maybe_num_node {
                self.compile(c, num_node)?;
            } else {
                // else default to 3
                c.emit(Opcode::LOAD, Mem::Constant, 3.0)?;
            }
            c.emit(Opcode::LOAD, Mem::Constant, 1.0)?;
            c.emit(Opcode::SUB, 0, 0)?;
            c.emit(Opcode::DIV, 0, 0)?;
            let delta_address = c.add_internal_local_mapping()?;
            c.emit(Opcode::STORE, Mem::Local, delta_address)?;

            // set looping variable x to 'from' value
            if let Some(from_node) = maybe_from_node {
                self.compile(c, from_node)?;
            } else {
                // else default to 0
                c.emit(Opcode::LOAD, Mem::Constant, 0.0)?;
            }

            let from_address = c.add_internal_local_mapping()?;

            c.emit(Opcode::STORE, Mem::Local, from_address)?;

            // store the starting 'from' value in the locally scoped variable
            c.emit(Opcode::LOAD, Mem::Local, from_address)?;

            let loop_variable_address =
                self.store_from_stack_to_memory(c, name_node, Mem::Local)?;

            // compare looping variable against exit condition
            // and jump if looping variable >= exit value
            let addr_loop_start = c.code.len();

            c.emit(Opcode::LOAD, Mem::Local, counter_address)?;
            c.emit(Opcode::LOAD, Mem::Local, num_address)?;

            // exit check
            c.emit(Opcode::LT, 0, 0)?;

            let addr_exit_check = c.code.len();
            c.emit(Opcode::JUMP_IF, 0, 0)?;

            // looper = from + (counter * delta)
            c.emit(Opcode::LOAD, Mem::Local, from_address)?;
            c.emit(Opcode::LOAD, Mem::Local, counter_address)?;
            c.emit(Opcode::LOAD, Mem::Local, delta_address)?;
            c.emit(Opcode::MUL, 0, 0)?;
            c.emit(Opcode::ADD, 0, 0)?;
            c.emit(Opcode::STORE, Mem::Local, loop_variable_address)?;

            let pre_body_opcode_offset = c.opcode_offset;

            // compile the body forms (woooaaaoohhh body form, body form for yoooouuuu)
            self.compile_rest(c, &children[1..])?;

            let post_body_opcode_offset = c.opcode_offset;
            let opcode_delta = post_body_opcode_offset - pre_body_opcode_offset;

            // pop off any values that the body might leave on the stack
            for _i in 0..opcode_delta {
                c.emit(Opcode::STORE, Mem::Void, 0)?;
            }

            // increment counter
            c.emit(Opcode::LOAD, Mem::Local, counter_address)?;
            c.emit(Opcode::LOAD, Mem::Constant, 1.0)?;
            c.emit(Opcode::ADD, 0, 0)?;
            c.emit(Opcode::STORE, Mem::Local, counter_address)?;

            // loop back to the comparison
            let mut c_len = c.code.len() as i32;
            c.emit(Opcode::JUMP, -(c_len - addr_loop_start as i32), 0)?;

            c_len = c.code.len() as i32;
            c.bytecode_modify_arg0_i32(addr_exit_check, c_len - addr_exit_check as i32)?;
        }
        Ok(())
    }

    fn compile_loop(&self, c: &mut Compilation, children: &[&Node]) -> Result<()> {
        // (loop (x from: 0 upto: 120 inc: 30) (body))
        // compile_loop children == (x from: 0 upto: 120 inc: 30) (body)
        //
        if children.len() < 2 {
            if children.is_empty() {
                error!("compile_loop requires at least 2 forms");
            } else {
                children[0].error_here("compile_loop requires at least 2 forms");
            }
            return Err(Error::Compiler);
        }

        let parameters_node = &children[0];
        error_if_alterable(&parameters_node, "compile_loop")?;

        if let Node::List(_, kids) = parameters_node {
            let kids = only_semantic_nodes(kids);

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

                if let Node::Label(_, _, iname) = label {
                    if *iname == Iname::from(Keyword::From) {
                        maybe_from_node = Some(&value);
                    } else if *iname == Iname::from(Keyword::To) {
                        maybe_to_node = Some(&value);
                    } else if *iname == Iname::from(Keyword::Upto) {
                        maybe_upto_node = Some(&value);
                    } else if *iname == Iname::from(Keyword::Inc) {
                        maybe_increment_node = Some(&value);
                    }
                }

                label_vals = &label_vals[2..];
            }

            let mut use_to = false;
            if maybe_to_node.is_some() {
                use_to = true;
            } else if maybe_upto_node.is_none() {
                parameters_node.error_here("compile_loop requires either to or upto parameters");
                return Err(Error::Compiler);
            }

            // set looping variable x to 'from' value
            if let Some(from_node) = maybe_from_node {
                self.compile(c, from_node)?;
            } else {
                // else default to 0
                c.emit(Opcode::LOAD, Mem::Constant, 0.0)?;
            }

            let loop_variable_address =
                self.store_from_stack_to_memory(c, name_node, Mem::Local)?;

            // compare looping variable against exit condition
            // and jump if looping variable >= exit value
            let addr_loop_start = c.code.len();

            c.emit(Opcode::LOAD, Mem::Local, loop_variable_address)?;

            if use_to {
                // so jump if looping variable >= exit value
                if let Some(to_node) = maybe_to_node {
                    self.compile(c, to_node)?;
                    c.emit(Opcode::LT, 0, 0)?;
                }
            } else {
                // so jump if looping variable > exit value
                if let Some(upto_node) = maybe_upto_node {
                    self.compile(c, upto_node)?;
                    c.emit(Opcode::GT, 0, 0)?;
                    c.emit(Opcode::NOT, 0, 0)?;
                }
            }

            let addr_exit_check = c.code.len();
            c.emit(Opcode::JUMP_IF, 0, 0)?; // bc_exit_check

            let pre_body_opcode_offset = c.opcode_offset;

            // compile the body forms (woooaaaoohhh body form, body form for yoooouuuu)
            self.compile_rest(c, &children[1..])?;

            let post_body_opcode_offset = c.opcode_offset;
            let opcode_delta = post_body_opcode_offset - pre_body_opcode_offset;

            // pop off any values that the body might leave on the stack
            for _i in 0..opcode_delta {
                c.emit(Opcode::STORE, Mem::Void, 0)?;
            }

            // increment the looping variable
            c.emit(Opcode::LOAD, Mem::Local, loop_variable_address)?;

            if let Some(increment_node) = maybe_increment_node {
                self.compile(c, increment_node)?;
            } else {
                c.emit(Opcode::LOAD, Mem::Constant, 1.0)?;
            }

            c.emit(Opcode::ADD, 0, 0)?;
            c.emit(Opcode::STORE, Mem::Local, loop_variable_address)?;
            // loop back to the comparison
            let mut c_len = c.code.len() as i32;
            c.emit(Opcode::JUMP, -(c_len - addr_loop_start as i32), 0)?;

            c_len = c.code.len() as i32;
            c.bytecode_modify_arg0_i32(addr_exit_check, c_len - addr_exit_check as i32)?;
        }
        Ok(())
    }

    fn compile_each(&self, c: &mut Compilation, children: &[&Node]) -> Result<()> {
        // (each (x from: [10 20 30 40 50])
        //       (+ x x))
        if children.len() < 2 {
            if children.is_empty() {
                error!("compile_each requires at least 2 forms");
            } else {
                children[0].error_here("compile_each requires at least 2 forms");
            }
            return Err(Error::Compiler);
        }

        let parameters_node = &children[0];
        error_if_alterable(&parameters_node, "compile_each")?;

        if let Node::List(_, kids) = parameters_node {
            let kids = only_semantic_nodes(kids);

            // the looping variable x
            let name_node = &kids[0];

            let mut maybe_from_node: Option<&Node> = None;

            let mut label_vals = &kids[1..];
            while label_vals.len() > 1 {
                let label = &label_vals[0];
                let value = &label_vals[1];

                if let Node::Label(_, _, iname) = label {
                    if *iname == Iname::from(Keyword::From) {
                        maybe_from_node = Some(&value);
                    }
                }

                label_vals = &label_vals[2..];
            }

            // set looping variable x to 'from' value
            if let Some(from_node) = maybe_from_node {
                self.compile(c, from_node)?;
            } else {
                // todo: ignore this, each should always have a from parameter
                // else default to 0
                c.emit(Opcode::LOAD, Mem::Constant, 0.0)?;
            }

            c.emit(Opcode::VEC_NON_EMPTY, 0, 0)?;
            let addr_exit_check_is_vec = c.code.len();
            c.emit(Opcode::JUMP_IF, 0, 0)?;

            c.emit(Opcode::VEC_LOAD_FIRST, 0, 0)?;

            // compare looping variable against exit condition
            // and jump if looping variable >= exit value
            let addr_loop_start = c.code.len() as i32;

            let loop_variable_address =
                self.store_from_stack_to_memory(c, name_node, Mem::Local)?;

            let pre_body_opcode_offset = c.opcode_offset;

            // compile the body forms (woooaaaoohhh body form, body form for yoooouuuu)
            self.compile_rest(c, &children[1..])?;

            let post_body_opcode_offset = c.opcode_offset;
            let opcode_delta = post_body_opcode_offset - pre_body_opcode_offset;

            // pop off any values that the body might leave on the stack
            for _i in 0..opcode_delta {
                c.emit(Opcode::STORE, Mem::Void, 0)?;
            }

            c.emit(Opcode::LOAD, Mem::Local, loop_variable_address)?;
            c.emit(Opcode::VEC_HAS_NEXT, 0, 0)?;

            let addr_exit_check = c.code.len();

            c.emit(Opcode::JUMP_IF, 0, 0)?;

            c.emit(Opcode::VEC_NEXT, 0, 0)?;

            // loop back to the comparison
            let mut c_len = c.code.len() as i32;
            c.emit(Opcode::JUMP, -(c_len - addr_loop_start), 0)?;

            c_len = c.code.len() as i32;
            c.bytecode_modify_arg0_i32(addr_exit_check, c_len - addr_exit_check as i32)?;
            // fill in jump distance for the IS_VEC check
            c.bytecode_modify_arg0_i32(
                addr_exit_check_is_vec,
                c_len - addr_exit_check_is_vec as i32,
            )?;
        } else {
            parameters_node.error_here("compile_each expected a list that defines parameters");
            return Err(Error::Compiler);
        }
        Ok(())
    }

    fn compile_vector_in_quote(&self, c: &mut Compilation, list_node: &Node) -> Result<()> {
        if let Node::List(_, children) = list_node {
            error_if_alterable(list_node, "compile_vector_in_quote")?;

            // slightly hackish
            // if this is a form like: '(red green blue)
            // the compiler should output the names rather than the colours that are
            // actually referenced (compile_user_defined_name would genereate a
            // MEM_SEG_GLOBAL LOAD code)
            //

            let children = only_semantic_nodes(children);
            let len = children.len() as i32;
            for n in children {
                if let Node::Name(_, _, iname) = n {
                    c.emit(Opcode::LOAD, Mem::Constant, *iname)?;
                } else {
                    self.compile(c, n)?;
                }
            }

            c.emit_squish(len)?;

            Ok(())
        } else {
            list_node.error_here("compile_vector_in_quote expected a Node::List");
            Err(Error::Compiler)
        }
    }

    fn compile_quote(&self, c: &mut Compilation, children: &[&Node]) -> Result<()> {
        let quoted_form = &children[0];
        match quoted_form {
            Node::List(_, _) => self.compile_vector_in_quote(c, quoted_form)?,
            Node::Name(_, _, iname) => c.emit(Opcode::LOAD, Mem::Constant, *iname)?,
            _ => self.compile(c, quoted_form)?,
        }
        Ok(())
    }

    // (++ vector value)
    fn compile_vector_append(&self, c: &mut Compilation, children: &[&Node]) -> Result<()> {
        if children.len() != 2 {
            if children.is_empty() {
                error!("compile_vector_append requires 2 args");
            } else {
                children[0].error_here("compile_vector_append requires 2 args");
            }
            return Err(Error::Compiler);
        }

        let vector = &children[0];
        self.compile(c, vector)?;

        let value = &children[1];
        self.compile(c, value)?;

        c.emit(Opcode::APPEND, 0, 0)?;

        if let Node::Name(_, _, iname) = vector {
            let mut mem_addr: Option<(Mem, usize)> = None;

            if let Some(address) = c.get_local_mapping(*iname) {
                mem_addr = Some((Mem::Local, *address));
            }
            if let Some(address) = c.get_global_mapping(*iname) {
                mem_addr = Some((Mem::Global, *address));
            }

            if let Some((mem, addr)) = mem_addr {
                c.emit(Opcode::STORE, mem, addr)?;
            }
        }

        Ok(())
    }

    // (fn-call (aj z: 44))
    fn compile_fn_call(&self, c: &mut Compilation, children: &[&Node]) -> Result<()> {
        // fn_name should be a defined function's name
        // it will be known at compile time

        if let Node::List(_, kids) = &children[0] {
            error_if_alterable(&children[0], "compile_fn_call")?;

            let kids = only_semantic_nodes(kids);

            // todo: warn if alterable

            let fn_info_index = &kids[0];
            // place the fn_info_index onto the stack so that CALL_F can find the function
            // offset and num args
            self.compile(c, fn_info_index)?;
            c.emit(Opcode::CALL_F, 0, 0)?;

            // compile the rest of the arguments

            // overwrite the default arguments with the actual arguments given by the fn invocation
            let mut label_vals = &kids[1..];
            while label_vals.len() > 1 {
                let label = &label_vals[0];
                let value = &label_vals[1];

                // push value
                self.compile(c, &value)?;

                // push the actual fn_info index so that the _FLU opcode can find it
                self.compile(c, fn_info_index)?;

                if let Node::Label(_, _, iname) = label {
                    c.emit(Opcode::STORE_F, Mem::Argument, *iname)?;
                } else {
                    label.error_here("compile_fn_call: label required");
                    return Err(Error::Compiler);
                }

                label_vals = &label_vals[2..];
            }

            // place the fn_info_index onto the stack so that CALL_F_0 can find the
            // function's body offset
            self.compile(c, fn_info_index)?;
            c.emit(Opcode::CALL_F_0, 0, 0)?;

            return Ok(());
        }

        children[0].error_here("compile_fn_call should be given a list as the first parameter");
        Err(Error::Compiler)
    }

    fn compile_address_of(
        &self,
        c: &mut Compilation,
        parent: &Node,
        children: &[&Node],
    ) -> Result<()> {
        // fn_name should be a defined function's name, it will be known at compile time
        if let Some(fn_info_index) = c.get_fn_info_index(&children[0]) {
            // store the index into c->fn_info in the c
            c.emit(Opcode::LOAD, Mem::Constant, fn_info_index as i32)?;
            Ok(())
        } else {
            parent.error_here("address-of function not found");
            Err(Error::Compiler)
        }
    }

    fn compile_explicit_0_arg_native_call(
        &self,
        c: &mut Compilation,
        native: Native,
    ) -> Result<()> {
        let default_mask = 0;

        let (args, stack_offset) = parameter_info(native)?;
        let num_args = args.len();

        c.emit(Opcode::LOAD, Mem::Constant, default_mask)?;
        c.emit(Opcode::NATIVE, native, num_args)?;

        // the vm's opcode_native will modify the stack, no need for the compiler to add STORE VOID opcodes
        // subtract num_args and the default_mask, also take into account that a value might be returned
        c.opcode_offset -= (num_args as i32 + 1) - stack_offset; // should be -= 1

        Ok(())
    }

    fn compile_on_matrix_stack(&self, c: &mut Compilation, children: &[&Node]) -> Result<()> {
        self.compile_explicit_0_arg_native_call(c, Native::MatrixPush)?;
        self.compile_rest(c, children)?;
        self.compile_explicit_0_arg_native_call(c, Native::MatrixPop)?;
        Ok(())
    }

    fn compile_if(&self, c: &mut Compilation, parent: &Node, children: &[&Node]) -> Result<()> {
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
            parent.error_here(&format!(
                "if clause requires 2 or 3 forms (given {})",
                num_children
            ));
            return Err(Error::Compiler);
        }

        self.compile(c, if_node)?;

        // insert jump to after the 'then' node if not true
        let addr_jump_then = c.code.len();
        c.emit(Opcode::JUMP_IF, 0, 0)?;

        // the offset after the if
        let offset_after_if = c.opcode_offset;

        self.compile(c, then_node)?;

        let offset_after_then = c.opcode_offset;

        if let Some(else_node) = else_node {
            // logically we're now going to go down one of possibly two paths
            // so we can't just continue to add the c->opcode_offset since
            // that would result in the offset taking both of the conditional's paths
            c.opcode_offset = offset_after_if;

            // insert a bc_jump_else opcode
            let addr_jump_else = c.code.len();

            c.emit(Opcode::JUMP, 0, 0)?;

            let addr_jump_then_offset = c.code.len() as i32 - addr_jump_then as i32;
            c.bytecode_modify_arg0_i32(addr_jump_then, addr_jump_then_offset)?;

            self.compile(c, else_node)?;

            let offset_after_else = c.opcode_offset;

            if offset_after_then != offset_after_else {
                // is this case actually going to happen?
                // if so we can check which of the two paths has the lower opcode offset
                // and pad out that path by inserting some LOAD CONST 9999 into the
                // c
                parent.error_here("different opcode_offsets for the two paths in a conditional");
                return Err(Error::Compiler);
            }

            let addr_jump_else_offset = c.code.len() as i32 - addr_jump_else as i32;
            c.bytecode_modify_arg0_i32(addr_jump_else, addr_jump_else_offset)?;
        } else {
            let addr_jump_then_offset = c.code.len() as i32 - addr_jump_then as i32;
            c.bytecode_modify_arg0_i32(addr_jump_then, addr_jump_then_offset)?;
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
    fn compile_fn(&self, c: &mut Compilation, children: &[&Node]) -> Result<()> {
        // fn (adder a: 0 b: 0) (+ a b)
        c.clear_local_mappings()?;

        let signature = &children[0]; // (addr a: 0 b: 0)
        error_if_alterable(&signature, "compile_fn")?;

        if let Node::List(_, kids) = signature {
            let kids = only_semantic_nodes(kids);

            if kids.is_empty() {
                // no fn name given
                signature.error_here("FnWithoutName");
                return Err(Error::Compiler);
            }

            let fn_name = &kids[0];
            if let Some(index) = c.get_fn_info_index(&fn_name) {
                c.current_fn_info_index = Some(index);

                // -------------
                // the arguments
                // -------------
                let mut updated_fn_info: FnInfo;
                {
                    let fn_info: &FnInfo = &c.fn_info[index];
                    updated_fn_info = FnInfo {
                        fn_name: fn_info.fn_name.to_string(),
                        ..Default::default()
                    }
                }

                updated_fn_info.arg_address = c.code.len();

                // pairs of label/value declarations
                let mut var_decls = &kids[1..];
                let mut num_args = 0;
                let mut counter = 0;

                if var_decls.len() % 2 != 0 {
                    fn_name.error_here("fn declaration doesn't have matching arg/value pairs");
                    return Err(Error::Compiler);
                }

                while !var_decls.is_empty() {
                    let label_node = &var_decls[0];
                    let value_node = &var_decls[1];

                    // get argument mapping
                    let iname = self.get_label_iname(label_node)?;

                    updated_fn_info.argument_offsets.push(iname);

                    // if let Some(label_i) = c.global_mappings.get(text) {
                    // } else {
                    //     // should be impossible to get here, the global mappings for the
                    //     // fn args should all have been registered in the
                    //     // register_top_level_fns function
                    // }

                    c.emit(Opcode::LOAD, Mem::Constant, iname)?;

                    c.emit(Opcode::STORE, Mem::Argument, counter)?;
                    counter += 1;

                    self.compile(c, value_node)?;
                    c.emit(Opcode::STORE, Mem::Argument, counter)?;
                    counter += 1;

                    num_args += 1;
                    var_decls = &var_decls[2..];
                }
                updated_fn_info.num_args = num_args;

                c.emit(Opcode::RET_0, 0, 0)?;

                // --------
                // the body
                // --------

                updated_fn_info.body_address = c.code.len();

                c.fn_info[index] = updated_fn_info;

                // compile the body forms (woooaaaoohhh body form, body form for yoooouuuu)
                self.compile_rest(c, &children[1..])?;

                // Don't need any STORE, MEM_SEG_VOID instructions as the RET will
                // pop the frame and blow the stack
                c.emit(Opcode::RET, 0, 0)?;

                c.current_fn_info_index = None;
            } else {
                fn_name.error_here("cannot find fn_info for node");
                return Err(Error::Compiler);
            }
        } else {
            // first item in fn declaration needs to be a list of function name and args
            signature.error_here("FnDeclIncomplete");
            return Err(Error::Compiler);
        }

        Ok(())
    }

    fn compile_fn_invocation_prologue(
        &self,
        c: &mut Compilation,
        fn_info_index: usize,
    ) -> Result<()> {
        // NOTE: CALL and CALL_0 get their function offsets and num args from the
        // stack so add some placeholder LOAD CONST opcodes and fill the CALL, CALL_0
        // with fn_info indexes that can later be used to fill in the LOAD CONST
        // opcodes with their correct offsets doing it this way enables functions to
        // call other functions that are declared later in the script

        // prepare the MEM_SEG_ARGUMENT with default values

        // for the function address
        c.emit(Opcode::LOAD, Mem::Constant, NONSENSE)?;
        // for the num args
        c.emit(Opcode::LOAD, Mem::Constant, NONSENSE)?;

        c.emit(Opcode::CALL, fn_info_index, fn_info_index)?;

        Ok(())
    }

    fn compile_fn_invocation_implicit_from(
        &self,
        c: &mut Compilation,
        fn_info_index: usize,
        from_name: &Node,
    ) -> Result<()> {
        self.compile(c, from_name)?;
        let from_iname = Iname::from(Keyword::From);
        c.emit(Opcode::PLACEHOLDER_STORE, fn_info_index, from_iname)?;

        Ok(())
    }

    fn compile_fn_invocation_args(
        &self,
        c: &mut Compilation,
        parent: &Node,
        children: &[&Node],
        fn_info_index: usize,
    ) -> Result<()> {
        // overwrite the default arguments with the actual arguments given by the fn invocation
        let mut arg_vals = &children[..];
        while !arg_vals.is_empty() {
            let arg = &arg_vals[0];
            if let Node::Label(_, _, iname) = arg {
                let val = &arg_vals[1];
                self.compile(c, val)?;
                c.emit(Opcode::PLACEHOLDER_STORE, fn_info_index, *iname)?;
                arg_vals = &arg_vals[2..];
            } else if let Node::Name(_, _, iname) = arg {
                let val = &arg_vals[0];
                self.compile(c, val)?;
                c.emit(Opcode::PLACEHOLDER_STORE, fn_info_index, *iname)?;
                arg_vals = &arg_vals[1..];
            } else {
                parent.error_here("compile_fn_invocation");
                return Err(Error::Compiler);
            }
        }
        Ok(())
    }

    fn compile_fn_invocation_epilogue(
        &self,
        c: &mut Compilation,
        fn_info_index: usize,
    ) -> Result<()> {
        // call the body of the function
        c.emit(Opcode::LOAD, Mem::Constant, NONSENSE)?;
        c.emit(Opcode::CALL_0, fn_info_index, fn_info_index)?;

        Ok(())
    }

    fn compile_rest(&self, c: &mut Compilation, children: &[&Node]) -> Result<()> {
        for n in children {
            self.compile(c, n)?;
        }
        Ok(())
    }

    fn compile_next_one(
        &self,
        c: &mut Compilation,
        parent: &Node,
        children: &[&Node],
        op: Opcode,
    ) -> Result<()> {
        if children.is_empty() {
            parent.error_here("compile_next_one");
            return Err(Error::Compiler);
        }

        self.compile(c, &children[0])?;
        c.emit(op, 0, 0)?;

        Ok(())
    }

    fn compile_math(&self, c: &mut Compilation, children: &[&Node], op: Opcode) -> Result<()> {
        self.compile(c, children[0])?;
        for n in &children[1..] {
            self.compile(c, n)?;
            c.emit(op, 0, 0)?;
        }
        Ok(())
    }

    fn compile_alterable_element(&self, c: &mut Compilation, node: &Node) -> Result<()> {
        match node {
            Node::Float(_, _, _) => {
                let f = self.get_float(node)?;
                c.emit(Opcode::LOAD, Mem::Constant, f)?;
            }
            Node::Vector(_, _elements) => {
                unimplemented!();
            }
            _ => {
                node.error_here(
                    "compile_alterable_element: expected either a float element or a vector",
                );
                return Err(Error::Compiler);
            }
        }

        Ok(())
    }

    fn compile_2d(&self, c: &mut Compilation, node: &Node, children: &[&Node]) -> Result<()> {
        // the node may contain alterable info
        let use_gene = node.is_alterable() && self.use_genes;

        if node.has_gene() && use_gene {
            let (a, b) = self.get_2d(node)?;
            c.emit(Opcode::LOAD, Mem::Constant, a)?;
            c.emit(Opcode::LOAD, Mem::Constant, b)?;
        } else {
            for n in children {
                if use_gene {
                    self.compile_alterable_element(c, n)?;
                } else {
                    self.compile(c, n)?;
                }
            }
        }
        c.emit_squish(2)?;

        Ok(())
    }

    fn compile_vector(&self, c: &mut Compilation, node: &Node, children: &[&Node]) -> Result<()> {
        // if this is an alterable vector, we'll have to pull values for each element from the genes
        let use_gene = node.has_gene() && self.use_genes;
        let len = children.len() as i32;

        for n in children {
            if use_gene {
                self.compile_alterable_element(c, n)?;
            } else {
                self.compile(c, n)?;
            }
        }

        c.emit_squish(len)?;

        Ok(())
    }

    fn compile_global_bind_kw_v2d(
        &self,
        c: &mut Compilation,
        kw: Keyword,
        value0: f32,
        value1: f32,
    ) -> Result<()> {
        c.emit(Opcode::LOAD, Mem::Constant, value0)?;
        c.emit(Opcode::LOAD, Mem::Constant, value1)?;
        c.emit_squish(2)?;
        self.store_globally_kw(c, kw)?;
        Ok(())
    }

    fn compile_global_bind_kw_f32(
        &self,
        c: &mut Compilation,
        kw: Keyword,
        value: f32,
    ) -> Result<()> {
        c.emit(Opcode::LOAD, Mem::Constant, value)?;
        self.store_globally_kw(c, kw)?;
        Ok(())
    }

    fn compile_global_bind_kw_col(
        &self,
        c: &mut Compilation,
        kw: Keyword,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) -> Result<()> {
        c.emit(
            Opcode::LOAD,
            Mem::Constant,
            Colour::new(ColourFormat::Rgb, r, g, b, a),
        )?;
        self.store_globally_kw(c, kw)?;
        Ok(())
    }

    fn store_locally(&self, c: &mut Compilation, iname: Iname) -> Result<usize> {
        let address: usize = match c.get_local_mapping(iname) {
            Some(&local_mapping) => local_mapping, // already storing the binding name
            None => c.add_local_mapping(iname)?,
        };

        c.emit(Opcode::STORE, Mem::Local, address)?;

        Ok(address)
    }

    fn store_globally_kw(&self, c: &mut Compilation, kw: Keyword) -> Result<usize> {
        let iname = Iname::from(kw);
        let address: usize = match c.get_global_mapping(iname) {
            Some(&global_mapping) => global_mapping, // already storing the binding name
            None => c.add_global_mapping_for_keyword(kw)?,
        };

        c.emit(Opcode::STORE, Mem::Global, address)?;

        Ok(address)
    }

    fn store_globally(&self, c: &mut Compilation, iname: Iname) -> Result<usize> {
        let address: usize = match c.get_global_mapping(iname) {
            Some(&global_mapping) => global_mapping, // already storing the binding name
            None => c.add_global_mapping(iname)?,
        };

        c.emit(Opcode::STORE, Mem::Global, address)?;

        Ok(address)
    }

    fn store_from_stack_to_memory(
        &self,
        c: &mut Compilation,
        node: &Node,
        mem: Mem,
    ) -> Result<usize> {
        if let Node::Name(_, _, iname) = node {
            match mem {
                Mem::Local => self.store_locally(c, *iname),
                // a call with mem == global means that this is a user defined global
                Mem::Global => self.store_globally(c, *iname),
                _ => {
                    node.error_here("store_from_stack_to_memory invalid memory type");
                    Err(Error::Compiler)
                }
            }
        } else {
            node.error_here("store_from_stack_to_memory");
            Err(Error::Compiler)
        }
    }

    fn compile_user_defined_name(&self, c: &mut Compilation, iname: Iname) -> Result<bool> {
        if let Some(local_mapping) = c.get_local_mapping(iname) {
            let val = *local_mapping;
            c.emit(Opcode::LOAD, Mem::Local, val)?;
            return Ok(true);
        }

        // check arguments if we're in a function
        if let Some(current_fn_info_index) = c.current_fn_info_index {
            let maybe_argument_mapping;
            {
                let fn_info = &c.fn_info[current_fn_info_index];
                maybe_argument_mapping = fn_info.get_argument_mapping(iname);
            }
            if let Some(argument_mapping) = maybe_argument_mapping {
                c.emit(Opcode::LOAD, Mem::Argument, argument_mapping as i32)?;
                return Ok(true);
            }
        }

        if let Some(global_mapping) = c.get_global_mapping(iname) {
            let val = *global_mapping;
            c.emit(Opcode::LOAD, Mem::Global, val)?;
            return Ok(true);
        }

        Ok(false)
    }

    fn is_list_beginning_with(&self, n: &Node, kw: Keyword) -> bool {
        if let Node::List(_, nodes) = n {
            let nodes = only_semantic_nodes(nodes);

            if !nodes.is_empty() {
                if let Node::Name(_, _, iname) = nodes[0] {
                    // todo: could just cast kw to i32 and compare directly with iname
                    if let Some(name_kw) = self.name_to_keyword.get(iname) {
                        return *name_kw == kw;
                    }
                }
            }
        }
        false
    }

    fn get_float(&self, n: &Node) -> Result<f32> {
        n.get_float(self.use_genes)
    }

    fn get_iname(&self, n: &Node) -> Result<Iname> {
        n.get_iname(self.use_genes)
    }

    fn get_label_iname(&self, n: &Node) -> Result<Iname> {
        n.get_label_iname(self.use_genes)
    }

    fn get_colour(&self, n: &Node) -> Result<Colour> {
        n.get_colour(self.use_genes)
    }

    fn get_2d(&self, n: &Node) -> Result<(f32, f32)> {
        n.get_2d(self.use_genes)
    }
}

fn error_if_alterable(n: &Node, s: &str) -> Result<()> {
    if n.is_alterable() {
        n.error_here(&format!("Alterable error: {} {:?}", s, n));
        Err(Error::Compiler)
    } else {
        Ok(())
    }
}

// renamed all_children_have_type as it's only used with children of type NAME
fn all_children_are_name_nodes(parent: &Node) -> bool {
    match parent {
        Node::List(_, children) | Node::Vector(_, children) => {
            let children = only_semantic_nodes(children);

            for n in children {
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

fn only_semantic_nodes(children: &[Node]) -> Vec<&Node> {
    let ns: Vec<&Node> = children.iter().filter(|n| n.is_semantic()).collect();
    ns
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::program::Mem;

    fn compile(s: &str) -> Vec<Bytecode> {
        let (ast, word_lut) = parse(s).unwrap();
        let program = compile_program(&ast, &word_lut).unwrap();
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

    fn load_const_keyword(val: Keyword) -> Bytecode {
        Bytecode {
            op: Opcode::LOAD,
            arg0: BytecodeArg::Mem(Mem::Constant),
            arg1: BytecodeArg::Name(Iname::new(val as i32)),
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

    fn native(n: Native, args: i32) -> Bytecode {
        Bytecode {
            op: Opcode::NATIVE,
            arg0: BytecodeArg::Native(n),
            arg1: BytecodeArg::Int(args),
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

    fn squish(num: i32) -> Bytecode {
        Bytecode {
            op: Opcode::SQUISH,
            arg0: BytecodeArg::Int(num),
            arg1: BytecodeArg::Int(0),
        }
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
        let preamble_program = compile_preamble().unwrap();
        assert_eq!(preamble_program.code.len(), 86);
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
                squish(2),
                stop(),
            ]
        );

        assert_eq!(
            compile("[23 45 67 89]"),
            vec![
                jump(1),
                load_const_f32(23.0),
                load_const_f32(45.0),
                load_const_f32(67.0),
                load_const_f32(89.0),
                squish(4),
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
                store_global(17),
                load_const_f32(10.0),
                store_global(18),
                stop(),
            ]
        );

        assert_eq!(
            compile("(define brush 9 b 10) (+ brush b)"),
            vec![
                jump(1),
                load_const_f32(9.0),
                store_global(17),
                load_const_f32(10.0),
                store_global(18),
                load_global_i32(17),
                load_global_i32(18),
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
                load_const_keyword(Keyword::A),
                store_arg(0),
                load_const_f32(0.0),
                store_arg(1),
                load_const_keyword(Keyword::B),
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
    fn test_adding_multiple_numbers() {
        assert_eq!(
            compile("(+ 5 6 7 8 9)"),
            vec![
                jump(1),
                load_const_f32(5.0),
                load_const_f32(6.0),
                add(),
                load_const_f32(7.0),
                add(),
                load_const_f32(8.0),
                add(),
                load_const_f32(9.0),
                add(),
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
                load_const_keyword(Keyword::A),
                store_arg(0),
                load_const_f32(99.0),
                store_arg(1),
                load_const_keyword(Keyword::B),
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
                squish(0),
                store_global(17),
                load_global_i32(17),
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
                jump(30),
                load_const_keyword(Keyword::By),
                store_arg(0),
                load_const_f32(4.0),
                store_arg(1),
                ret_0(),
                load_const_f32(7.0),
                load_const_f32(8.0),
                load_const_f32(9.0),
                squish(3),
                store_local(0),
                squish(0),
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

    #[test]
    fn test_native() {
        let expected_bytecode = vec![
            jump(1),
            load_const_i32(15),
            load_const_f32(1.0),
            load_const_f32(0.0),
            load_const_f32(0.0),
            load_const_f32(0.0),
            native(Native::ColRGB, 4),
            store_global(17),
            load_const_i32(0),
            load_const_f32(0.5),
            load_global_i32(17),
            native(Native::ColSetAlpha, 2),
            stop(),
        ];

        assert_eq!(
            compile("(define c (col/rgb)) (col/set-alpha from: c value: 0.5)"),
            expected_bytecode
        );

        // the implied label name syntax should produce the same output
        assert_eq!(
            compile("(define from (col/rgb)) (col/set-alpha from value: 0.5)"),
            expected_bytecode
        );

        // compiler should handle cases where an argument value has the same
        // name as another argument
        assert_eq!(
            compile("(define value (col/rgb)) (col/set-alpha from: value value: 0.5)"),
            expected_bytecode
        );
    }

    #[test]
    fn test_fn_invocation_2() {
        let expected_bytecode = vec![
            jump(14),
            load_const_keyword(Keyword::A),
            store_arg(0),
            load_const_f32(99.0),
            store_arg(1),
            load_const_keyword(Keyword::B),
            store_arg(2),
            load_const_f32(88.0),
            store_arg(3),
            ret_0(),
            load_arg(1),
            load_arg(3),
            add(),
            ret(),
            load_const_f32(1.0),
            store_global(17),
            load_const_f32(2.0),
            store_global(18),
            load_const_i32(1),
            load_const_i32(2),
            call(),
            load_global_i32(17),
            store_arg(1),
            load_global_i32(18),
            store_arg(3),
            load_const_i32(10),
            call_0(),
            stop(),
        ];

        assert_eq!(
            compile(
                "(define x 1 y 2)
                 (fn (adder a: 99 b: 88) (+ a b))
                 (adder a: x b: y)"
            ),
            expected_bytecode
        );

        assert_eq!(
            compile(
                "(define a 1 b 2)
                 (fn (adder a: 99 b: 88) (+ a b))
                 (adder a: a b: b)"
            ),
            expected_bytecode
        );

        assert_eq!(
            compile(
                "(define a 1 b 2)
                 (fn (adder a: 99 b: 88) (+ a b))
                 (adder a b)"
            ),
            expected_bytecode
        );
    }

    #[test]
    fn test_fromname_fn_invocation() {
        let expected_bytecode = vec![
            jump(14),
            load_const_keyword(Keyword::From),
            store_arg(0),
            load_const_f32(99.0),
            store_arg(1),
            load_const_keyword(Keyword::By),
            store_arg(2),
            load_const_f32(88.0),
            store_arg(3),
            ret_0(),
            load_arg(1),
            load_arg(3),
            add(),
            ret(),
            load_const_f32(33.0),
            store_global(17),
            load_const_i32(1),
            load_const_i32(2),
            call(),
            load_global_i32(17),
            store_arg(1),
            load_const_f32(10.0),
            store_arg(3),
            load_const_i32(10),
            call_0(),
            stop(),
        ];

        // traditional syntax
        assert_eq!(
            compile(
                "(define x 33)
                 (fn (increase from: 99 by: 88) (+ from by))
                 (increase from: x by: 10)"
            ),
            expected_bytecode
        );

        // fromname syntax
        assert_eq!(
            compile(
                "(define x 33)
                 (fn (increase from: 99 by: 88) (+ from by))
                 (x.increase by: 10)"
            ),
            expected_bytecode
        );
    }
}
