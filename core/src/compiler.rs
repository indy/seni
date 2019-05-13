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

use std::collections::{BTreeMap, HashMap};
use std::fmt;

use crate::colour::{Colour, ColourFormat};
use crate::error::Error;
use crate::gene::Genotype;
use crate::iname::Iname;
use crate::keywords::{name_to_keyword_hash, Keyword};
use crate::mathutil;
use crate::native::{name_to_native_hash, parameter_info, Native};
use crate::opcodes::{opcode_stack_offset, Opcode};
use crate::parser::{Node, NodeMeta, WordLut};
use crate::program::{Bytecode, BytecodeArg, Data, FnInfo, Mem, Program};
use crate::result::Result;
use crate::vm::Var;

use log::warn;

const MEMORY_LOCAL_SIZE: usize = 40;

pub fn compile_preamble() -> Result<Program> {
    let mut compilation = Compilation::new();
    let compiler = Compiler::new();

    compiler.register_top_level_preamble(&mut compilation)?;
    compiler.compile_preamble(&mut compilation)?;

    Ok(Program {
        code: compilation.code,
        fn_info: compilation.fn_info,
        ..Default::default()
    })
}

pub fn compile_program(ast: &[Node], word_lut: &WordLut) -> Result<Program> {
    let mut compilation = Compilation::new();
    let compiler = Compiler::new();

    compiler.compile_common(&mut compilation, &ast)?;

    let mut data: Data = Default::default();
    data.strings = word_lut.get_script_inames();

    Ok(Program {
        code: compilation.code,
        fn_info: compilation.fn_info,
        data,
        ..Default::default()
    })
}

pub fn compile_program_1(ast_node: &Node, word_lut: &WordLut) -> Result<Program> {
    let mut compilation = Compilation::new();
    let compiler = Compiler::new();

    compiler.compile_common_1(&mut compilation, &ast_node)?;

    let mut data: Data = Default::default();
    data.strings = word_lut.get_script_inames();

    Ok(Program {
        code: compilation.code,
        fn_info: compilation.fn_info,
        data,
        ..Default::default()
    })
}

pub fn compile_program_for_trait(
    ast: &[Node],
    word_lut: &WordLut,
    global_mapping: &BTreeMap<Iname, i32>,
) -> Result<Program> {
    let mut compilation = Compilation::new();
    let compiler = Compiler::new();

    let ast = only_semantic_nodes(ast);

    compiler.compile_common_prologue(&mut compilation)?;
    compiler.register_top_level_fns(&mut compilation, &ast)?;
    compiler.register_top_level_defines(&mut compilation, &ast)?;

    compiler.compile_common_top_level_fns(&mut compilation, &ast)?;
    compiler.compile_global_mappings_for_trait(&mut compilation, global_mapping)?;
    compiler.compile_common_top_level_defines(&mut compilation, &ast)?;
    compiler.compile_common_top_level_forms(&mut compilation, &ast)?;
    compiler.compile_common_epilogue(&mut compilation)?;

    let mut data: Data = Default::default();
    data.strings = word_lut.get_script_inames();

    Ok(Program {
        code: compilation.code,
        fn_info: compilation.fn_info,
        data,
        ..Default::default()
    })
}

pub fn compile_program_with_genotype(
    ast: &mut [Node],
    word_lut: &WordLut,
    genotype: &mut Genotype,
) -> Result<Program> {
    let mut compilation = Compilation::new();
    let mut compiler = Compiler::new();
    compiler.use_genes = true;

    assign_genotype_to_ast(ast, genotype)?;

    compiler.compile_common(&mut compilation, &ast)?;

    let mut data: Data = Default::default();
    data.strings = word_lut.get_script_inames();

    Ok(Program {
        code: compilation.code,
        fn_info: compilation.fn_info,
        data,
        ..Default::default()
    })
}

// todo: don't make public
// todo: return errors when applicable
pub fn assign_genotype_to_ast(ast: &mut [Node], genotype: &mut Genotype) -> Result<()> {
    genotype.current_gene_index = 0;

    for n in ast {
        assign_genes_to_nodes(n, genotype)?;
    }

    Ok(())
}

fn hacky_assign_genes_to_each_node_in_vector(
    elements: &mut Vec<Node>,
    genotype: &mut Genotype,
) -> Vec<Node> {
    let mut res: Vec<Node> = Vec::new();

    for n in elements {
        match n {
            Node::Vector(ns, _) => {
                res.push(Node::Vector(
                    ns.clone(),
                    Some(NodeMeta::new_with_gene(
                        genotype.genes[genotype.current_gene_index].clone(),
                    )),
                ));
                genotype.current_gene_index += 1;
            }
            Node::Float(f, s, _) => {
                res.push(Node::Float(
                    *f,
                    s.to_string(),
                    Some(NodeMeta::new_with_gene(
                        genotype.genes[genotype.current_gene_index].clone(),
                    )),
                ));
                genotype.current_gene_index += 1;
            }
            _ => {}
        }
    }

    res
}

fn hacky_assign_genes_to_each_node_in_vector2(ns: &mut Vec<Node>, res: Vec<Node>) {
    ns.clear();
    for n in res {
        ns.push(n);
    }
}

fn assign_genes_to_nodes(node: &mut Node, genotype: &mut Genotype) -> Result<()> {
    match node {
        Node::List(ref mut ns, meta) => {
            if let Some(ref mut node_meta) = meta {
                node_meta.gene = Some(genotype.genes[genotype.current_gene_index].clone());
                genotype.current_gene_index += 1;
            }
            for n in ns {
                assign_genes_to_nodes(n, genotype)?;
            }
        }
        Node::Vector(ref mut ns, meta) => {
            if meta.is_some() {
                let res = hacky_assign_genes_to_each_node_in_vector(ns, genotype);
                hacky_assign_genes_to_each_node_in_vector2(ns, res);
            } else {
                for n in ns {
                    assign_genes_to_nodes(n, genotype)?;
                }
            }
        }
        Node::Float(_, _, ref mut meta) => {
            if let Some(ref mut node_meta) = meta {
                node_meta.gene = Some(genotype.genes[genotype.current_gene_index].clone());
                genotype.current_gene_index += 1;
            }
        }
        Node::Name(_, _, meta) => {
            if let Some(ref mut node_meta) = meta {
                node_meta.gene = Some(genotype.genes[genotype.current_gene_index].clone());
                genotype.current_gene_index += 1;
            }
        }
        Node::Label(_, _, meta) => {
            if let Some(ref mut node_meta) = meta {
                node_meta.gene = Some(genotype.genes[genotype.current_gene_index].clone());
                genotype.current_gene_index += 1;
            }
        }
        Node::String(_, _, meta) => {
            if let Some(ref mut node_meta) = meta {
                node_meta.gene = Some(genotype.genes[genotype.current_gene_index].clone());
                genotype.current_gene_index += 1;
            }
        }
        Node::Whitespace(_, meta) => {
            if let Some(ref mut node_meta) = meta {
                node_meta.gene = Some(genotype.genes[genotype.current_gene_index].clone());
                genotype.current_gene_index += 1;
            }
        }
        Node::Comment(_, meta) => {
            if let Some(ref mut node_meta) = meta {
                node_meta.gene = Some(genotype.genes[genotype.current_gene_index].clone());
                genotype.current_gene_index += 1;
            }
        }
    }

    Ok(())
}

fn is_node_colour_constructor(children: &[&Node]) -> bool {
    if !children.is_empty() {
        if let Node::Name(_, iname, _) = children[0] {
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

    local_mappings: HashMap<Iname, i32>, // iname -> local mapping index
    local_mapping_marker: i32, // todo: check that it is < MEMORY_LOCAL_SIZE, as that constant is used in the interpreter

    global_mappings: HashMap<Iname, i32>, // iname -> global mapping index
    global_mapping_marker: i32,

    // using BTreeMap as this will be given to a TraitList which will be packed,
    // for testing purposes having a consistent ordering is important
    user_defined_globals: BTreeMap<Iname, i32>, // iname -> global mapping index
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

    // used when adding explicit global mappings during a trait compilation
    fn add_explicit_global_mapping(&mut self, iname: Iname, map_val: i32) {
        self.global_mappings.insert(iname, map_val);
    }

    fn add_global_mapping(&mut self, iname: Iname) -> Result<i32> {
        self.user_defined_globals
            .insert(iname, self.global_mapping_marker);
        self.global_mappings
            .insert(iname, self.global_mapping_marker);
        self.global_mapping_marker += 1;
        Ok(self.global_mapping_marker - 1)
    }

    fn add_global_mapping_for_keyword(&mut self, kw: Keyword) -> Result<i32> {
        // self.add_global_mapping(kw as i32)
        self.global_mappings
            .insert(Iname::from(kw), self.global_mapping_marker);
        self.global_mapping_marker += 1;
        Ok(self.global_mapping_marker - 1)
    }

    fn get_global_mapping(&self, iname: Iname) -> Option<&i32> {
        self.global_mappings.get(&iname)
    }

    pub fn get_user_defined_globals(self) -> BTreeMap<Iname, i32> {
        self.user_defined_globals
    }

    fn clear_local_mappings(&mut self) -> Result<()> {
        self.local_mappings.clear();
        self.local_mapping_marker = 0;
        Ok(())
    }

    fn add_local_mapping(&mut self, iname: Iname) -> Result<i32> {
        self.local_mappings.insert(iname, self.local_mapping_marker);
        self.local_mapping_marker += 1;
        Ok(self.local_mapping_marker - 1)
    }

    fn get_local_mapping(&self, iname: Iname) -> Option<&i32> {
        self.local_mappings.get(&iname)
    }

    // we want a local mapping that's going to be used to store an internal variable
    // (e.g. during a fence loop)
    // note: it's up to the caller to manage this reference
    fn add_internal_local_mapping(&mut self) -> Result<i32> {
        // todo: is this right???
        let i = 9999;
        let n = Iname::new(i);

        // let s = "internal_local_mapping".to_string();
        self.local_mappings.insert(n, self.local_mapping_marker);
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

    fn correct_function_addresses(&self, compilation: &mut Compilation) -> Result<()> {
        let mut all_fixes: Vec<(usize, Opcode, Mem, i32)> = Vec::new(); // tuple of index, op, arg0, ???
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
            compilation.bytecode_modify_mem(index, op, arg0, arg1)?;
        }
        for (index, arg1) in arg1_fixes {
            compilation.bytecode_modify_arg1_i32(index, arg1)?;
        }

        Ok(())
    }

    fn register_top_level_preamble(&self, compilation: &mut Compilation) -> Result<()> {
        compilation.add_global_mapping_for_keyword(Keyword::CanvasWidth)?;
        compilation.add_global_mapping_for_keyword(Keyword::CanvasHeight)?;

        compilation.add_global_mapping_for_keyword(Keyword::MathPi)?;
        compilation.add_global_mapping_for_keyword(Keyword::MathTau)?;

        compilation.add_global_mapping_for_keyword(Keyword::White)?;
        compilation.add_global_mapping_for_keyword(Keyword::Black)?;
        compilation.add_global_mapping_for_keyword(Keyword::Red)?;
        compilation.add_global_mapping_for_keyword(Keyword::Green)?;
        compilation.add_global_mapping_for_keyword(Keyword::Blue)?;
        compilation.add_global_mapping_for_keyword(Keyword::Yellow)?;
        compilation.add_global_mapping_for_keyword(Keyword::Magenta)?;
        compilation.add_global_mapping_for_keyword(Keyword::Cyan)?;

        compilation.add_global_mapping_for_keyword(Keyword::ColProceduralFnPresets)?;
        compilation.add_global_mapping_for_keyword(Keyword::EaseAll)?;
        compilation.add_global_mapping_for_keyword(Keyword::BrushAll)?;

        Ok(())
    }

    fn register_top_level_fns(&self, compilation: &mut Compilation, ast: &[&Node]) -> Result<()> {
        // clear all data
        compilation.fn_info = Vec::new();

        // register top level fns
        for n in ast.iter() {
            if let Some(fn_info) = self.register_top_level_fns_1(n)? {
                compilation.fn_info.push(fn_info);
            }
        }

        Ok(())
    }

    fn register_top_level_fns_1(&self, n: &Node) -> Result<Option<FnInfo>> {
        if self.is_list_beginning_with(n, Keyword::Fn) {
            // get the name of the fn
            if let Node::List(nodes, _) = n {
                let nodes = only_semantic_nodes(nodes);

                if nodes.len() < 2 {
                    // a list with just the 'fn' keyword ???
                    return Err(Error::Compiler("malformed function definition".to_string()));
                }
                let name_and_params = nodes[1];
                if let Node::List(np_nodes, _) = name_and_params {
                    let np_nodes = only_semantic_nodes(np_nodes);

                    if !np_nodes.is_empty() {
                        let name_node = &np_nodes[0];
                        if let Node::Name(text, _, _) = name_node {
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

    fn register_names_in_define(&self, compilation: &mut Compilation, lhs: &Node) -> Result<()> {
        error_if_alterable(&lhs, "register_names_in_define")?;

        match lhs {
            Node::Name(_, _, _) => {
                // (define foo 42)
                //let s = name.to_string();
                let iname = self.get_iname(lhs)?;
                if let Some(_i) = compilation.get_global_mapping(iname) {
                    // name was already added to global_mappings
                    return Ok(());
                }

                if let Err(e) = compilation.add_global_mapping(iname) {
                    return Err(e);
                }
            }
            Node::List(nodes, _) | Node::Vector(nodes, _) => {
                // (define [a b] (something))
                // (define [a [x y]] (something))
                let nodes = only_semantic_nodes(nodes);

                for n in nodes {
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
        ast: &[&Node],
    ) -> Result<()> {
        let define_keyword_string = Keyword::Define.to_string();

        for n in ast.iter() {
            self.register_top_level_defines_1(compilation, n, &define_keyword_string)?;
        }

        Ok(())
    }

    fn register_top_level_defines_1(
        &self,
        compilation: &mut Compilation,
        n: &Node,
        define_keyword_string: &str,
    ) -> Result<()> {
        if let Node::List(nodes, _) = n {
            let nodes = only_semantic_nodes(nodes);
            if !nodes.is_empty() {
                let define_keyword = &nodes[0];
                if let Node::Name(text, _, _) = define_keyword {
                    if text == define_keyword_string {
                        let mut defs = &nodes[1..];
                        while defs.len() > 1 {
                            if let Err(e) = self.register_names_in_define(compilation, &defs[0]) {
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

    fn compile_preamble(&self, compilation: &mut Compilation) -> Result<()> {
        // ********************************************************************************
        // NOTE: each entry should have a corresponding entry in
        // register_top_level_preamble
        // ********************************************************************************
        self.compile_global_bind_kw_f32(compilation, Keyword::CanvasWidth, 1000.0)?;
        self.compile_global_bind_kw_f32(compilation, Keyword::CanvasHeight, 1000.0)?;

        self.compile_global_bind_kw_f32(compilation, Keyword::MathPi, mathutil::PI)?;
        self.compile_global_bind_kw_f32(compilation, Keyword::MathTau, mathutil::TAU)?;

        self.compile_global_bind_kw_col(compilation, Keyword::White, 1.0, 1.0, 1.0, 1.0)?;
        self.compile_global_bind_kw_col(compilation, Keyword::Black, 0.0, 0.0, 0.0, 1.0)?;
        self.compile_global_bind_kw_col(compilation, Keyword::Red, 1.0, 0.0, 0.0, 1.0)?;
        self.compile_global_bind_kw_col(compilation, Keyword::Green, 0.0, 1.0, 0.0, 1.0)?;
        self.compile_global_bind_kw_col(compilation, Keyword::Blue, 0.0, 0.0, 1.0, 1.0)?;
        self.compile_global_bind_kw_col(compilation, Keyword::Yellow, 1.0, 1.0, 0.0, 1.0)?;
        self.compile_global_bind_kw_col(compilation, Keyword::Magenta, 1.0, 0.0, 1.0, 1.0)?;
        self.compile_global_bind_kw_col(compilation, Keyword::Cyan, 0.0, 1.0, 1.0, 1.0)?;

        self.compile_global_bind_procedural_presets(compilation)?;
        self.compile_global_bind_ease_all(compilation)?;
        self.compile_global_bind_brush_all(compilation)?;

        // ********************************************************************************
        // NOTE: each entry should have a corresponding entry in
        // register_top_level_preamble
        // ********************************************************************************

        // slap a stop onto the end of this compilation
        compilation.emit(Opcode::STOP, 0, 0)?;

        Ok(())
    }

    fn compile_global_bind_procedural_presets(&self, compilation: &mut Compilation) -> Result<()> {
        // create a vector
        compilation.emit(Opcode::LOAD, Mem::Void, 0)?;

        // append the names
        self.append_keyword(compilation, Keyword::Chrome)?;
        self.append_keyword(compilation, Keyword::HotlineMiami)?;
        self.append_keyword(compilation, Keyword::KnightRider)?;
        self.append_keyword(compilation, Keyword::Mars)?;
        self.append_keyword(compilation, Keyword::Rainbow)?;
        self.append_keyword(compilation, Keyword::Robocop)?;
        self.append_keyword(compilation, Keyword::Transformers)?;

        self.store_globally_kw(compilation, Keyword::ColProceduralFnPresets)?;

        Ok(())
    }

    fn compile_global_bind_ease_all(&self, compilation: &mut Compilation) -> Result<()> {
        // create a vector
        compilation.emit(Opcode::LOAD, Mem::Void, 0)?;

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

        self.store_globally_kw(compilation, Keyword::EaseAll)?;

        Ok(())
    }

    fn compile_global_bind_brush_all(&self, compilation: &mut Compilation) -> Result<()> {
        // create a vector
        compilation.emit(Opcode::LOAD, Mem::Void, 0)?;

        // append the names
        self.append_keyword(compilation, Keyword::BrushFlat)?;
        self.append_keyword(compilation, Keyword::BrushA)?;
        self.append_keyword(compilation, Keyword::BrushB)?;
        self.append_keyword(compilation, Keyword::BrushC)?;
        self.append_keyword(compilation, Keyword::BrushD)?;
        self.append_keyword(compilation, Keyword::BrushE)?;
        self.append_keyword(compilation, Keyword::BrushF)?;
        self.append_keyword(compilation, Keyword::BrushG)?;

        self.store_globally_kw(compilation, Keyword::BrushAll)?;

        Ok(())
    }

    pub fn compile_common(&self, compilation: &mut Compilation, ast: &[Node]) -> Result<()> {
        let ast = only_semantic_nodes(ast);
        self.compile_common_prologue(compilation)?;

        self.register_top_level_fns(compilation, &ast)?;
        self.register_top_level_defines(compilation, &ast)?;

        self.compile_common_top_level_fns(compilation, &ast)?;
        self.compile_common_top_level_defines(compilation, &ast)?;
        self.compile_common_top_level_forms(compilation, &ast)?;
        self.compile_common_epilogue(compilation)?;
        Ok(())
    }

    fn compile_common_1(&self, compilation: &mut Compilation, n: &Node) -> Result<()> {
        self.compile_common_prologue(compilation)?;

        // single node version of self.register_top_level_fns(compilation, ast)?;
        compilation.fn_info = Vec::new();
        if let Some(fn_info) = self.register_top_level_fns_1(n)? {
            compilation.fn_info.push(fn_info);
        }

        // single node version of self.register_top_level_defines(compilation, ast)?;
        let define_keyword_string = Keyword::Define.to_string();
        self.register_top_level_defines_1(compilation, n, &define_keyword_string)?;

        //// single node version of self.compile_common_top_level_fns(compilation, ast)?;
        {
            // a placeholder, filled in at the end of this function
            compilation.emit(Opcode::JUMP, 0, 0)?;
            let start_index = compilation.code.len() - 1;

            // compile the top-level functions
            if self.is_list_beginning_with(n, Keyword::Fn) {
                self.compile(compilation, n)?; // todo: the c-impl returns a node to continue from
            }

            // jump to the compilation's starting address
            let jump_address = compilation.code.len() as i32;
            compilation.bytecode_modify_arg0_i32(start_index, jump_address)?;
        }

        //// single node version of self.compile_common_top_level_defines(compilation, ast)?;
        {
            if self.is_list_beginning_with(n, Keyword::Define) {
                if let Node::List(children, _) = n {
                    let children = only_semantic_nodes(children);
                    self.compile_define(compilation, &children[1..], Mem::Global)?;
                }
            }
        }

        //// single node version of self.compile_common_top_level_forms(compilation, ast)?;
        {
            if !self.is_list_beginning_with(n, Keyword::Define)
                && !self.is_list_beginning_with(n, Keyword::Fn)
            {
                self.compile(compilation, n)?;
            }
        }

        self.compile_common_epilogue(compilation)?;

        Ok(())
    }

    fn compile_common_prologue(&self, compilation: &mut Compilation) -> Result<()> {
        compilation.clear_global_mappings()?;
        compilation.clear_local_mappings()?;
        // compilation->current_fn_info = NULL;

        self.register_top_level_preamble(compilation)?;

        Ok(())
    }

    fn compile_common_top_level_fns(
        &self,
        compilation: &mut Compilation,
        ast: &[&Node],
    ) -> Result<()> {
        // a placeholder, filled in at the end of this function
        compilation.emit(Opcode::JUMP, 0, 0)?;
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

    fn compile_global_mappings_for_trait(
        &self,
        compilation: &mut Compilation,
        global_mapping: &BTreeMap<Iname, i32>,
    ) -> Result<()> {
        for (iname, map_val) in global_mapping {
            compilation.add_explicit_global_mapping(*iname, *map_val);
        }
        Ok(())
    }

    fn compile_common_top_level_defines(
        &self,
        compilation: &mut Compilation,
        ast: &[&Node],
    ) -> Result<()> {
        for n in ast.iter() {
            if self.is_list_beginning_with(n, Keyword::Define) {
                if let Node::List(children, _) = n {
                    let children = only_semantic_nodes(children);
                    self.compile_define(compilation, &children[1..], Mem::Global)?;
                }
            }
        }
        Ok(())
    }

    fn compile_common_top_level_forms(
        &self,
        compilation: &mut Compilation,
        ast: &[&Node],
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

    fn compile_common_epilogue(&self, compilation: &mut Compilation) -> Result<()> {
        compilation.emit(Opcode::STOP, 0, 0)?;

        // now update the addreses used by CALL and CALL_0
        self.correct_function_addresses(compilation)?;

        Ok(())
    }

    fn compile(&self, compilation: &mut Compilation, ast: &Node) -> Result<()> {
        // todo: move this out of compile and into the compilation struct
        match ast {
            Node::List(children, meta) => {
                let children = only_semantic_nodes(children);

                if self.use_genes && meta.is_some() && is_node_colour_constructor(&children[..]) {
                    // we have an alterable colour constructor so just load in the colour value stored in the gene
                    //
                    let col = self.get_colour(ast)?;
                    compilation.emit(Opcode::LOAD, Mem::Constant, col)?;
                } else {
                    if self.use_genes && meta.is_some() {
                        return Err(Error::Compiler(
                            "given an alterable list that wasn't a colour constructor???"
                                .to_string(),
                        ));
                    }
                    self.compile_list(compilation, &children[..])?
                }
            }
            Node::Float(_, _, _) => {
                let f = self.get_float(ast)?;
                return compilation.emit(Opcode::LOAD, Mem::Constant, f);
            }
            Node::Vector(children, _) => {
                let children = only_semantic_nodes(children);

                if children.len() == 2 {
                    return self.compile_2d(compilation, ast, &children[..]);
                } else {
                    return self.compile_vector(compilation, ast, &children[..]);
                }
            }
            Node::String(_, _, _) => {
                let iname = self.get_iname(ast)?;
                return compilation.emit_name_as_string(Opcode::LOAD, Mem::Constant, iname);
            }
            Node::Name(text, _, _) => {
                let iname = self.get_iname(ast)?;

                return if self.compile_user_defined_name(compilation, iname)? {
                    Ok(())
                } else if let Some(kw) = self.name_to_keyword.get(&iname) {
                    compilation.emit(Opcode::LOAD, Mem::Constant, *kw)?;
                    Ok(())
                } else if let Some(native) = self.name_to_native.get(&iname) {
                    compilation.emit(Opcode::NATIVE, *native, 0)?;
                    Ok(())
                } else {
                    Err(Error::Compiler(format!(
                        "compile: can't find user defined name or keyword: {}",
                        text
                    )))
                };
            }
            _ => return Err(Error::Compiler(format!("compile ast: {:?}", ast))),
        }

        Ok(())
    }

    fn compile_list(&self, compilation: &mut Compilation, children: &[&Node]) -> Result<()> {
        if children.is_empty() {
            // should this be an error?
            return Err(Error::Compiler(
                "compile_list no children (should this be an error?)".to_string(),
            ));
        }

        match &children[0] {
            Node::List(kids, _) => {
                let kids = only_semantic_nodes(kids);
                self.compile_list(compilation, &kids[..])?
            }
            Node::Name(_, _, _) => {
                let iname = self.get_iname(&children[0])?;

                if let Some(fn_info_index) = compilation.get_fn_info_index(&children[0]) {
                    // todo: get_fn_info_index is re-checking that this is a Node::Name
                    self.compile_fn_invocation(compilation, &children[1..], fn_info_index)?;
                    return Ok(());
                }

                if self.compile_user_defined_name(compilation, iname)? {
                    return Ok(());
                } else if let Some(kw) = self.name_to_keyword.get(&iname) {
                    match *kw {
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
                    compilation.emit(Opcode::LOAD, Mem::Constant, default_mask)?;

                    // iterating in reverse so that when the native function
                    // is run it can pop the arguments from the stack in the
                    // order it specified
                    // now add the arguments to the stack
                    for (kw, default_value) in args.iter().rev() {
                        if let Some(idx) = self.get_parameter_index(label_vals, *kw) {
                            // todo: does this need to be self?
                            // compile the node at the given index
                            self.compile(compilation, label_vals[idx])?;
                        } else {
                            // compile the default argument value from the Var in args
                            self.compile_var_as_load(compilation, default_value)?;
                        }
                    }

                    compilation.emit(Opcode::NATIVE, *native, num_args)?;

                    // the vm's opcode_native will modify the stack, no need for the compiler to add STORE VOID opcodes
                    // subtract num_args and the default_mask, also take into account that a value might be returned
                    compilation.opcode_offset -= (num_args as i32 + 1) - stack_offset;
                }
            }
            _ => return Err(Error::Compiler("compile_list strange child".to_string())),
        }

        Ok(())
    }

    fn get_parameter_index(&self, label_vals: &[&Node], kw: Keyword) -> Option<usize> {
        let kw_name = Iname::from(kw);

        for (i, node) in label_vals.iter().enumerate() {
            if i & 1 == 0 {
                // a label
                if let Node::Label(_, iname, _) = node {
                    if *iname == kw_name {
                        return Some(i + 1);
                    }
                } else {
                    warn!("expected a label node in every odd position");
                }
            } else {
                // a value
            }
        }

        None
    }

    fn compile_var_as_load(&self, compilation: &mut Compilation, var: &Var) -> Result<()> {
        match var {
            Var::Float(f) => compilation.emit(Opcode::LOAD, Mem::Constant, *f)?,
            Var::V2D(x, y) => {
                compilation.emit(Opcode::LOAD, Mem::Constant, *x)?;
                compilation.emit(Opcode::LOAD, Mem::Constant, *y)?;
                compilation.emit(Opcode::SQUISH2, 0, 0)?;
            }
            Var::Colour(colour) => {
                // the default_mask
                compilation.emit(Opcode::LOAD, Mem::Constant, 0)?;
                compilation.emit(Opcode::LOAD, Mem::Constant, colour.e0)?;
                compilation.emit(Opcode::LOAD, Mem::Constant, colour.e1)?;
                compilation.emit(Opcode::LOAD, Mem::Constant, colour.e2)?;
                compilation.emit(Opcode::LOAD, Mem::Constant, colour.e3)?;
                compilation.emit(Opcode::NATIVE, Native::ColRGB, 4)?;
                // now update the opcode offset since this is using the NATIVE
                let num_args = 4;
                // subtract the 4 args + 1 default mask, but then add back the return value
                compilation.opcode_offset -= (num_args + 1) - 1;
            }
            Var::Keyword(kw) => compilation.emit(Opcode::LOAD, Mem::Constant, *kw)?,
            // Var::Vector(vs) => {
            //     // pushing from the VOID means creating a new, empty vector
            //     compilation.emit(Opcode::LOAD, Mem::Void, 0)?;
            //     for v in vs {
            //         self.compile_var_as_load(compilation, v)?;
            //         compilation.emit(Opcode::APPEND, 0, 0)?;
            //     }
            // }
            _ => {
                return Err(Error::Compiler(format!(
                    "unimplemented var compilation {:?}",
                    var
                )))
            }
        };

        Ok(())
    }

    fn compile_define(
        &self,
        compilation: &mut Compilation,
        children: &[&Node],
        mem: Mem,
    ) -> Result<()> {
        let mut defs = children;
        // defs are an even number of nodes representing binding/value pairs
        // (define a 10 b 20 c 30) -> a 10 b 20 c 30

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
                    let kids = only_semantic_nodes(kids);

                    // define [a b] (something-that-returns-a-vector ...)

                    // check if we can use the PILE opcode
                    if all_children_are_name_nodes(lhs_node) {
                        let num_kids = kids.len();

                        // PILE will stack the elements in the rhs vector in order,
                        // so the lhs values have to be popped in reverse order
                        compilation.emit(Opcode::PILE, num_kids, 0)?;
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

    fn compile_fence(&self, compilation: &mut Compilation, children: &[&Node]) -> Result<()> {
        // (fence (x from: 0 to: 5 num: 5) (+ 42 38))
        if children.len() < 2 {
            return Err(Error::Compiler(
                "compile_fence requires at least 2 forms".to_string(),
            ));
        }

        let parameters_node = &children[0];
        error_if_alterable(&parameters_node, "compile_fence")?;

        if let Node::List(kids, _) = parameters_node {
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

                if let Node::Label(_, iname, _) = label {
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
            let num_address = compilation.add_internal_local_mapping()?;
            if let Some(num_node) = maybe_num_node {
                self.compile(compilation, num_node)?;
            } else {
                compilation.emit(Opcode::LOAD, Mem::Constant, 2.0)?;
            }

            compilation.emit(Opcode::STORE, Mem::Local, num_address)?;

            // reserve a memory location in local memory for a counter from 0 to quantity
            let counter_address = compilation.add_internal_local_mapping()?;

            compilation.emit(Opcode::LOAD, Mem::Constant, 0.0)?;
            compilation.emit(Opcode::STORE, Mem::Local, counter_address)?;

            // delta that needs to be added at every iteration
            //
            // (to - from) / (num - 1)
            if let Some(to_node) = maybe_to_node {
                self.compile(compilation, to_node)?;
            } else {
                // else default to 1
                compilation.emit(Opcode::LOAD, Mem::Constant, 1.0)?;
            }

            if let Some(from_node) = maybe_from_node {
                self.compile(compilation, from_node)?;
            } else {
                // else default to 0
                compilation.emit(Opcode::LOAD, Mem::Constant, 0.0)?;
            }

            compilation.emit(Opcode::SUB, 0, 0)?;

            if let Some(num_node) = maybe_num_node {
                self.compile(compilation, num_node)?;
            } else {
                // else default to 3
                compilation.emit(Opcode::LOAD, Mem::Constant, 3.0)?;
            }
            compilation.emit(Opcode::LOAD, Mem::Constant, 1.0)?;
            compilation.emit(Opcode::SUB, 0, 0)?;
            compilation.emit(Opcode::DIV, 0, 0)?;
            let delta_address = compilation.add_internal_local_mapping()?;
            compilation.emit(Opcode::STORE, Mem::Local, delta_address)?;

            // set looping variable x to 'from' value
            if let Some(from_node) = maybe_from_node {
                self.compile(compilation, from_node)?;
            } else {
                // else default to 0
                compilation.emit(Opcode::LOAD, Mem::Constant, 0.0)?;
            }

            let from_address = compilation.add_internal_local_mapping()?;

            compilation.emit(Opcode::STORE, Mem::Local, from_address)?;

            // store the starting 'from' value in the locally scoped variable
            compilation.emit(Opcode::LOAD, Mem::Local, from_address)?;

            let loop_variable_address =
                self.store_from_stack_to_memory(compilation, name_node, Mem::Local)?;

            // compare looping variable against exit condition
            // and jump if looping variable >= exit value
            let addr_loop_start = compilation.code.len();

            compilation.emit(Opcode::LOAD, Mem::Local, counter_address)?;
            compilation.emit(Opcode::LOAD, Mem::Local, num_address)?;

            // exit check
            compilation.emit(Opcode::LT, 0, 0)?;

            let addr_exit_check = compilation.code.len();
            compilation.emit(Opcode::JUMP_IF, 0, 0)?;

            // looper = from + (counter * delta)
            compilation.emit(Opcode::LOAD, Mem::Local, from_address)?;
            compilation.emit(Opcode::LOAD, Mem::Local, counter_address)?;
            compilation.emit(Opcode::LOAD, Mem::Local, delta_address)?;
            compilation.emit(Opcode::MUL, 0, 0)?;
            compilation.emit(Opcode::ADD, 0, 0)?;
            compilation.emit(Opcode::STORE, Mem::Local, loop_variable_address)?;

            let pre_body_opcode_offset = compilation.opcode_offset;

            // compile the body forms (woooaaaoohhh body form, body form for yoooouuuu)
            self.compile_rest(compilation, &children[1..])?;

            let post_body_opcode_offset = compilation.opcode_offset;
            let opcode_delta = post_body_opcode_offset - pre_body_opcode_offset;

            // pop off any values that the body might leave on the stack
            for _i in 0..opcode_delta {
                compilation.emit(Opcode::STORE, Mem::Void, 0)?;
            }

            // increment counter
            compilation.emit(Opcode::LOAD, Mem::Local, counter_address)?;
            compilation.emit(Opcode::LOAD, Mem::Constant, 1.0)?;
            compilation.emit(Opcode::ADD, 0, 0)?;
            compilation.emit(Opcode::STORE, Mem::Local, counter_address)?;

            // loop back to the comparison
            let mut compilation_len = compilation.code.len() as i32;
            compilation.emit(Opcode::JUMP, -(compilation_len - addr_loop_start as i32), 0)?;

            compilation_len = compilation.code.len() as i32;
            compilation.bytecode_modify_arg0_i32(
                addr_exit_check,
                compilation_len - addr_exit_check as i32,
            )?;
        }
        Ok(())
    }

    fn compile_loop(&self, compilation: &mut Compilation, children: &[&Node]) -> Result<()> {
        // (loop (x from: 0 upto: 120 inc: 30) (body))
        // compile_loop children == (x from: 0 upto: 120 inc: 30) (body)
        //
        if children.len() < 2 {
            return Err(Error::Compiler(
                "compile_loop requires at least 2 forms".to_string(),
            ));
        }

        let parameters_node = &children[0];
        error_if_alterable(&parameters_node, "compile_loop")?;

        if let Node::List(kids, _) = parameters_node {
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

                if let Node::Label(_, iname, _) = label {
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
                return Err(Error::Compiler(
                    "compile_loop requires either to or upto parameters".to_string(),
                ));
            }

            // set looping variable x to 'from' value
            if let Some(from_node) = maybe_from_node {
                self.compile(compilation, from_node)?;
            } else {
                // else default to 0
                compilation.emit(Opcode::LOAD, Mem::Constant, 0.0)?;
            }

            let loop_variable_address =
                self.store_from_stack_to_memory(compilation, name_node, Mem::Local)?;

            // compare looping variable against exit condition
            // and jump if looping variable >= exit value
            let addr_loop_start = compilation.code.len();

            compilation.emit(Opcode::LOAD, Mem::Local, loop_variable_address)?;

            if use_to {
                // so jump if looping variable >= exit value
                if let Some(to_node) = maybe_to_node {
                    self.compile(compilation, to_node)?;
                    compilation.emit(Opcode::LT, 0, 0)?;
                }
            } else {
                // so jump if looping variable > exit value
                if let Some(upto_node) = maybe_upto_node {
                    self.compile(compilation, upto_node)?;
                    compilation.emit(Opcode::GT, 0, 0)?;
                    compilation.emit(Opcode::NOT, 0, 0)?;
                }
            }

            let addr_exit_check = compilation.code.len();
            compilation.emit(Opcode::JUMP_IF, 0, 0)?; // bc_exit_check

            let pre_body_opcode_offset = compilation.opcode_offset;

            // compile the body forms (woooaaaoohhh body form, body form for yoooouuuu)
            self.compile_rest(compilation, &children[1..])?;

            let post_body_opcode_offset = compilation.opcode_offset;
            let opcode_delta = post_body_opcode_offset - pre_body_opcode_offset;

            // pop off any values that the body might leave on the stack
            for _i in 0..opcode_delta {
                compilation.emit(Opcode::STORE, Mem::Void, 0)?;
            }

            // increment the looping variable
            compilation.emit(Opcode::LOAD, Mem::Local, loop_variable_address)?;

            if let Some(increment_node) = maybe_increment_node {
                self.compile(compilation, increment_node)?;
            } else {
                compilation.emit(Opcode::LOAD, Mem::Constant, 1.0)?;
            }

            compilation.emit(Opcode::ADD, 0, 0)?;
            compilation.emit(Opcode::STORE, Mem::Local, loop_variable_address)?;
            // loop back to the comparison
            let mut compilation_len = compilation.code.len() as i32;
            compilation.emit(Opcode::JUMP, -(compilation_len - addr_loop_start as i32), 0)?;

            compilation_len = compilation.code.len() as i32;
            compilation.bytecode_modify_arg0_i32(
                addr_exit_check,
                compilation_len - addr_exit_check as i32,
            )?;
        }
        Ok(())
    }

    fn compile_each(&self, compilation: &mut Compilation, children: &[&Node]) -> Result<()> {
        // (each (x from: [10 20 30 40 50])
        //       (+ x x))

        if children.len() < 2 {
            return Err(Error::Compiler(
                "compile_each requires at least 2 forms".to_string(),
            ));
        }

        let parameters_node = &children[0];
        error_if_alterable(&parameters_node, "compile_each")?;

        if let Node::List(kids, _) = parameters_node {
            let kids = only_semantic_nodes(kids);

            // the looping variable x
            let name_node = &kids[0];

            let mut maybe_from_node: Option<&Node> = None;

            let mut label_vals = &kids[1..];
            while label_vals.len() > 1 {
                let label = &label_vals[0];
                let value = &label_vals[1];

                if let Node::Label(_, iname, _) = label {
                    if *iname == Iname::from(Keyword::From) {
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
                compilation.emit(Opcode::LOAD, Mem::Constant, 0.0)?;
            }

            compilation.emit(Opcode::VEC_NON_EMPTY, 0, 0)?;
            let addr_exit_check_is_vec = compilation.code.len();
            compilation.emit(Opcode::JUMP_IF, 0, 0)?;

            compilation.emit(Opcode::VEC_LOAD_FIRST, 0, 0)?;

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
                compilation.emit(Opcode::STORE, Mem::Void, 0)?;
            }

            compilation.emit(Opcode::LOAD, Mem::Local, loop_variable_address)?;
            compilation.emit(Opcode::VEC_HAS_NEXT, 0, 0)?;

            let addr_exit_check = compilation.code.len();

            compilation.emit(Opcode::JUMP_IF, 0, 0)?;

            compilation.emit(Opcode::VEC_NEXT, 0, 0)?;

            // loop back to the comparison
            let mut compilation_len = compilation.code.len() as i32;
            compilation.emit(Opcode::JUMP, -(compilation_len - addr_loop_start), 0)?;

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
        compilation.emit(Opcode::LOAD, Mem::Void, 0)?;

        if let Node::List(children, _) = list_node {
            error_if_alterable(list_node, "compile_vector_in_quote")?;

            // slightly hackish
            // if this is a form like: '(red green blue)
            // the compiler should output the names rather than the colours that are
            // actually referenced (compile_user_defined_name would genereate a
            // MEM_SEG_GLOBAL LOAD code)
            //

            let children = only_semantic_nodes(children);
            for n in children {
                if let Node::Name(_, iname, _) = n {
                    compilation.emit(Opcode::LOAD, Mem::Constant, *iname)?;
                } else {
                    self.compile(compilation, n)?;
                }
                compilation.emit(Opcode::APPEND, 0, 0)?;
            }
            return Ok(());
        }
        Err(Error::Compiler(
            "compile_vector_in_quote expected a Node::List".to_string(),
        ))
    }

    fn compile_quote(&self, compilation: &mut Compilation, children: &[&Node]) -> Result<()> {
        let quoted_form = &children[0];
        match quoted_form {
            Node::List(_, _) => self.compile_vector_in_quote(compilation, quoted_form)?,
            Node::Name(_, iname, _) => compilation.emit(Opcode::LOAD, Mem::Constant, *iname)?,
            _ => self.compile(compilation, quoted_form)?,
        }
        Ok(())
    }

    // (++ vector value)
    fn compile_vector_append(
        &self,
        compilation: &mut Compilation,
        children: &[&Node],
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

        compilation.emit(Opcode::APPEND, 0, 0)?;

        if let Node::Name(_, iname, _) = vector {
            let mut mem_addr: Option<(Mem, i32)> = None;

            if let Some(address) = compilation.get_local_mapping(*iname) {
                mem_addr = Some((Mem::Local, *address));
            }
            if let Some(address) = compilation.get_global_mapping(*iname) {
                mem_addr = Some((Mem::Global, *address));
            }

            if let Some((mem, addr)) = mem_addr {
                compilation.emit(Opcode::STORE, mem, addr)?;
            }
        }

        Ok(())
    }

    // (fn-call (aj z: 44))
    fn compile_fn_call(&self, compilation: &mut Compilation, children: &[&Node]) -> Result<()> {
        // fn_name should be a defined function's name
        // it will be known at compile time

        if let Node::List(kids, _) = &children[0] {
            error_if_alterable(&children[0], "compile_fn_call")?;

            let kids = only_semantic_nodes(kids);

            // todo: warn if alterable

            let fn_info_index = &kids[0];
            // place the fn_info_index onto the stack so that CALL_F can find the function
            // offset and num args
            self.compile(compilation, fn_info_index)?;
            compilation.emit(Opcode::CALL_F, 0, 0)?;

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
                    compilation.emit(Opcode::STORE_F, Mem::Argument, *iname)?;
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
            compilation.emit(Opcode::CALL_F_0, 0, 0)?;

            return Ok(());
        }

        Err(Error::Compiler(
            "compile_fn_call should be given a list as the first parameter".to_string(),
        ))
    }

    fn compile_address_of(&self, compilation: &mut Compilation, children: &[&Node]) -> Result<()> {
        // fn_name should be a defined function's name, it will be known at compile time
        if let Some(fn_info_index) = compilation.get_fn_info_index(&children[0]) {
            // store the index into compilation->fn_info in the compilation
            compilation.emit(Opcode::LOAD, Mem::Constant, fn_info_index as i32)?;
            return Ok(());
        }

        Err(Error::Compiler("compile_address_of".to_string()))
    }

    fn compile_explicit_0_arg_native_call(
        &self,
        compilation: &mut Compilation,
        native: Native,
    ) -> Result<()> {
        let default_mask = 0;

        let (args, stack_offset) = parameter_info(native)?;
        let num_args = args.len();

        compilation.emit(Opcode::LOAD, Mem::Constant, default_mask)?;
        compilation.emit(Opcode::NATIVE, native, num_args)?;

        // the vm's opcode_native will modify the stack, no need for the compiler to add STORE VOID opcodes
        // subtract num_args and the default_mask, also take into account that a value might be returned
        compilation.opcode_offset -= (num_args as i32 + 1) - stack_offset; // should be -= 1

        Ok(())
    }

    fn compile_on_matrix_stack(
        &self,
        compilation: &mut Compilation,
        children: &[&Node],
    ) -> Result<()> {
        self.compile_explicit_0_arg_native_call(compilation, Native::MatrixPush)?;
        self.compile_rest(compilation, children)?;
        self.compile_explicit_0_arg_native_call(compilation, Native::MatrixPop)?;
        Ok(())
    }

    fn compile_if(&self, compilation: &mut Compilation, children: &[&Node]) -> Result<()> {
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
        compilation.emit(Opcode::JUMP_IF, 0, 0)?;

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

            compilation.emit(Opcode::JUMP, 0, 0)?;

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
    fn compile_fn(&self, compilation: &mut Compilation, children: &[&Node]) -> Result<()> {
        // fn (adder a: 0 b: 0) (+ a b)
        compilation.clear_local_mappings()?;

        let signature = &children[0]; // (addr a: 0 b: 0)
        error_if_alterable(&signature, "compile_fn")?;

        if let Node::List(kids, _) = signature {
            let kids = only_semantic_nodes(kids);

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
                    updated_fn_info = FnInfo {
                        fn_name: fn_info.fn_name.to_string(),
                        ..Default::default()
                    }
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
                    let iname = self.get_label_iname(label_node)?;

                    updated_fn_info.argument_offsets.push(iname);

                    // if let Some(label_i) = compilation.global_mappings.get(text) {
                    // } else {
                    //     // should be impossible to get here, the global mappings for the
                    //     // fn args should all have been registered in the
                    //     // register_top_level_fns function
                    // }

                    compilation.emit(Opcode::LOAD, Mem::Constant, iname)?;

                    compilation.emit(Opcode::STORE, Mem::Argument, counter)?;
                    counter += 1;

                    self.compile(compilation, value_node)?;
                    compilation.emit(Opcode::STORE, Mem::Argument, counter)?;
                    counter += 1;

                    num_args += 1;
                    var_decls = &var_decls[2..];
                }
                updated_fn_info.num_args = num_args;

                compilation.emit(Opcode::RET_0, 0, 0)?;

                // --------
                // the body
                // --------

                updated_fn_info.body_address = compilation.code.len();

                compilation.fn_info[index] = updated_fn_info;

                // compile the body forms (woooaaaoohhh body form, body form for yoooouuuu)
                self.compile_rest(compilation, &children[1..])?;

                // Don't need any STORE, MEM_SEG_VOID instructions as the RET will
                // pop the frame and blow the stack
                compilation.emit(Opcode::RET, 0, 0)?;

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
        children: &[&Node],
        fn_info_index: usize,
    ) -> Result<()> {
        // NOTE: CALL and CALL_0 get their function offsets and num args from the
        // stack so add some placeholder LOAD CONST opcodes and fill the CALL, CALL_0
        // with fn_info indexes that can later be used to fill in the LOAD CONST
        // opcodes with their correct offsets doing it this way enables functions to
        // call other functions that are declared later in the script

        // prepare the MEM_SEG_ARGUMENT with default values

        // for the function address
        compilation.emit(Opcode::LOAD, Mem::Constant, 666)?;
        // for the num args
        compilation.emit(Opcode::LOAD, Mem::Constant, 667)?;

        compilation.emit(Opcode::CALL, fn_info_index, fn_info_index)?;

        // overwrite the default arguments with the actual arguments given by the fn invocation
        let mut arg_vals = &children[..];
        while arg_vals.len() > 1 {
            let arg = &arg_vals[0];
            if let Node::Label(_, iname, _) = arg {
                let val = &arg_vals[1];
                self.compile(compilation, val)?;
                compilation.emit(Opcode::PLACEHOLDER_STORE, fn_info_index, *iname)?;
            } else {
                return Err(Error::Compiler("compile_fn_invocation".to_string()));
            }

            arg_vals = &arg_vals[2..];
        }

        // call the body of the function
        compilation.emit(Opcode::LOAD, Mem::Constant, 668)?;
        compilation.emit(Opcode::CALL_0, fn_info_index, fn_info_index)?;

        Ok(())
    }

    fn compile_rest(&self, compilation: &mut Compilation, children: &[&Node]) -> Result<()> {
        for n in children {
            self.compile(compilation, n)?;
        }
        Ok(())
    }

    fn compile_next_one(
        &self,
        compilation: &mut Compilation,
        children: &[&Node],
        op: Opcode,
    ) -> Result<()> {
        if children.is_empty() {
            return Err(Error::Compiler("compile_next_one".to_string()));
        }

        self.compile(compilation, &children[0])?;
        compilation.emit(op, 0, 0)?;

        Ok(())
    }

    fn compile_math(
        &self,
        compilation: &mut Compilation,
        children: &[&Node],
        op: Opcode,
    ) -> Result<()> {
        self.compile(compilation, children[0])?;
        for n in &children[1..] {
            self.compile(compilation, n)?;
            compilation.emit(op, 0, 0)?;
        }
        Ok(())
    }

    fn compile_alterable_element(&self, compilation: &mut Compilation, node: &Node) -> Result<()> {
        match node {
            Node::Float(_, _, _) => {
                let f = self.get_float(node)?;
                compilation.emit(Opcode::LOAD, Mem::Constant, f)?;
            }
            Node::Vector(_elements, _) => {
                unimplemented!();
            }
            _ => {
                return Err(Error::Compiler(
                    "compile_alterable_element: expected either a float element or a vector"
                        .to_string(),
                ));
            }
        }

        Ok(())
    }

    fn compile_2d(
        &self,
        compilation: &mut Compilation,
        node: &Node,
        children: &[&Node],
    ) -> Result<()> {
        // the node may contain alterable info
        let use_gene = node.is_alterable() && self.use_genes;

        if node.has_gene() && use_gene {
            let (a, b) = self.get_2d(node)?;
            compilation.emit(Opcode::LOAD, Mem::Constant, a)?;
            compilation.emit(Opcode::LOAD, Mem::Constant, b)?;
        } else {
            for n in children {
                if use_gene {
                    self.compile_alterable_element(compilation, n)?;
                } else {
                    self.compile(compilation, n)?;
                }
            }
        }
        compilation.emit(Opcode::SQUISH2, 0, 0)?;

        Ok(())
    }

    fn compile_vector(
        &self,
        compilation: &mut Compilation,
        node: &Node,
        children: &[&Node],
    ) -> Result<()> {
        // pushing from the VOID means creating a new, empty vector
        compilation.emit(Opcode::LOAD, Mem::Void, 0)?;

        // if this is an alterable vector, we'll have to pull values for each element from the genes
        let use_gene = node.has_gene() && self.use_genes;

        for n in children {
            if use_gene {
                self.compile_alterable_element(compilation, n)?;
            } else {
                self.compile(compilation, n)?;
            }
            compilation.emit(Opcode::APPEND, 0, 0)?;
        }

        Ok(())
    }

    fn compile_global_bind_kw_i32(
        &self,
        compilation: &mut Compilation,
        kw: Keyword,
        value: i32,
    ) -> Result<()> {
        compilation.emit(Opcode::LOAD, Mem::Constant, value)?;
        self.store_globally_kw(compilation, kw)?;
        Ok(())
    }

    fn compile_global_bind_kw_f32(
        &self,
        compilation: &mut Compilation,
        kw: Keyword,
        value: f32,
    ) -> Result<()> {
        compilation.emit(Opcode::LOAD, Mem::Constant, value)?;
        self.store_globally_kw(compilation, kw)?;
        Ok(())
    }

    fn compile_global_bind_kw_col(
        &self,
        compilation: &mut Compilation,
        kw: Keyword,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) -> Result<()> {
        compilation.emit(
            Opcode::LOAD,
            Mem::Constant,
            Colour::new(ColourFormat::Rgb, r, g, b, a),
        )?;
        self.store_globally_kw(compilation, kw)?;
        Ok(())
    }

    fn append_keyword(&self, compilation: &mut Compilation, kw: Keyword) -> Result<()> {
        compilation.emit(Opcode::LOAD, Mem::Constant, Iname::from(kw))?;
        compilation.emit(Opcode::APPEND, 0, 0)?;
        Ok(())
    }

    fn store_locally(&self, compilation: &mut Compilation, iname: Iname) -> Result<i32> {
        let address: i32 = match compilation.get_local_mapping(iname) {
            Some(&local_mapping) => local_mapping, // already storing the binding name
            None => compilation.add_local_mapping(iname)?,
        };

        compilation.emit(Opcode::STORE, Mem::Local, address)?;

        Ok(address)
    }

    fn store_globally_kw(&self, compilation: &mut Compilation, kw: Keyword) -> Result<i32> {
        let iname = Iname::from(kw);
        let address: i32 = match compilation.get_global_mapping(iname) {
            Some(&global_mapping) => global_mapping, // already storing the binding name
            None => compilation.add_global_mapping_for_keyword(kw)?,
        };

        compilation.emit(Opcode::STORE, Mem::Global, address)?;

        Ok(address)
    }

    fn store_globally(&self, compilation: &mut Compilation, iname: Iname) -> Result<i32> {
        let address: i32 = match compilation.get_global_mapping(iname) {
            Some(&global_mapping) => global_mapping, // already storing the binding name
            None => compilation.add_global_mapping(iname)?,
        };

        compilation.emit(Opcode::STORE, Mem::Global, address)?;

        Ok(address)
    }

    fn store_from_stack_to_memory(
        &self,
        compilation: &mut Compilation,
        node: &Node,
        mem: Mem,
    ) -> Result<i32> {
        if let Node::Name(_, iname, _) = node {
            match mem {
                Mem::Local => self.store_locally(compilation, *iname),
                // a call with mem == global means that this is a user defined global
                Mem::Global => self.store_globally(compilation, *iname),
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
        iname: Iname,
    ) -> Result<bool> {
        if let Some(local_mapping) = compilation.get_local_mapping(iname) {
            let val = *local_mapping;
            compilation.emit(Opcode::LOAD, Mem::Local, val)?;
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
                compilation.emit(Opcode::LOAD, Mem::Argument, argument_mapping as i32)?;
                return Ok(true);
            }
        }

        if let Some(global_mapping) = compilation.get_global_mapping(iname) {
            let val = *global_mapping;
            compilation.emit(Opcode::LOAD, Mem::Global, val)?;
            return Ok(true);
        }

        Ok(false)
    }

    fn is_list_beginning_with(&self, n: &Node, kw: Keyword) -> bool {
        if let Node::List(nodes, _) = n {
            let nodes = only_semantic_nodes(nodes);

            if !nodes.is_empty() {
                if let Node::Name(_, iname, _) = nodes[0] {
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
        Err(Error::Compiler(format!("Alterable error: {} {:?}", s, n)))
    } else {
        Ok(())
    }
}

// renamed all_children_have_type as it's only used with children of type NAME
fn all_children_are_name_nodes(parent: &Node) -> bool {
    match parent {
        Node::List(children, _) | Node::Vector(children, _) => {
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

fn count_children(parent: &Node) -> Result<usize> {
    match parent {
        Node::List(children, _) | Node::Vector(children, _) => {
            let children = only_semantic_nodes(children);
            Ok(children.len())
        }
        _ => Err(Error::Compiler("count_children".to_string())),
    }
}

fn only_semantic_nodes(children: &[Node]) -> Vec<&Node> {
    let ns: Vec<&Node> = children.iter().filter(|n| n.is_semantic()).collect();
    ns
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packable::Packable;
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

    fn load_const_name(val: i32) -> Bytecode {
        Bytecode {
            op: Opcode::LOAD,
            arg0: BytecodeArg::Mem(Mem::Constant),
            arg1: BytecodeArg::Name(Iname::new(val)),
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
        let preamble_program = compile_preamble().unwrap();
        assert_eq!(preamble_program.code.len(), 129);
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
                store_global(15),
                load_const_f32(10.0),
                store_global(16),
                stop(),
            ]
        );

        assert_eq!(
            compile("(define brush 9 b 10) (+ brush b)"),
            vec![
                jump(1),
                load_const_f32(9.0),
                store_global(15),
                load_const_f32(10.0),
                store_global(16),
                load_global_i32(15),
                load_global_i32(16),
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
                load_const_name(224),
                store_arg(0),
                load_const_f32(0.0),
                store_arg(1),
                load_const_name(225),
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
                load_const_name(224),
                store_arg(0),
                load_const_f32(99.0),
                store_arg(1),
                load_const_name(225),
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
                store_global(15),
                load_global_i32(15),
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
                load_const_name(246),
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
