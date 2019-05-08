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

use crate::compiler::{compile_program_1, compile_program_for_trait, Compilation, Compiler};
use crate::context::Context;
use crate::iname::Iname;
use crate::packable::{Mule, Packable};
use crate::parser::{Node, NodeMeta, WordLut};
use crate::program::Program;
use crate::result::Result;
use crate::run_program_with_preamble;
use crate::vm::{Var, Vm};

use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Trait {
    // todo: replace within_vector and index with a single Option<usize>

    // true == instantiated as one of multiple traits within a vector
    pub within_vector: bool,

    // if within_vector then this is the index within the parent vector
    pub index: usize,

    pub initial_value: Var,
    pub program: Program,
}

impl Packable for Trait {
    fn pack(&self, cursor: &mut String) -> Result<()> {
        Mule::pack_bool_sp(cursor, self.within_vector);
        Mule::pack_usize_sp(cursor, self.index);

        self.initial_value.pack(cursor)?;
        Mule::pack_space(cursor);

        self.program.pack(cursor)?;

        Ok(())
    }
    fn unpack(cursor: &str) -> Result<(Self, &str)> {
        let (within_vector, rem) = Mule::unpack_bool_sp(cursor)?;

        let (index, rem) = Mule::unpack_usize_sp(rem)?;

        let (initial_value, rem) = Var::unpack(rem)?;
        let rem = Mule::skip_space(rem);

        let (program, rem) = Program::unpack(rem)?;

        let new_trait = if within_vector {
            Trait::new(initial_value, program, Some(index))
        } else {
            Trait::new(initial_value, program, None)
        };

        Ok((new_trait, rem))
    }
}

impl Trait {
    fn new(initial_value: Var, program: Program, index_in_vec: Option<usize>) -> Self {
        let index: usize;
        let within_vector: bool;
        if let Some(i) = index_in_vec {
            index = i;
            within_vector = true;
        } else {
            index = 0;
            within_vector = false;
        }

        Trait {
            within_vector,
            index,
            initial_value,
            program,
        }
    }

    fn compile(
        node: &Node,
        word_lut: &WordLut,
        parameter_ast: &[Node],
        global_mapping: &BTreeMap<Iname, i32>,
        index: Option<usize>,
    ) -> Result<Self> {
        // todo: what about NODE_LABEL and NODE_STRING?
        /*
         * Why is NODE_NAME a special case?
         *
         * It doesn't make sense to set the trait->initial_value to the result of
         * compiling just a name. Something like the 'focal/build-*' functions return
         * a structure that can be used by 'focal/value', but using that structure
         * as the initial_value makes unparsing impossible.
         *
         * So here we're just manually converting a sen_node name into a sen_var name
         * and normal execution of the program will do the appropriate evaluation.
         */

        let initial_value = match node {
            Node::Name(_, i, _) => Var::Name(*i),
            _ => {
                let program = compile_program_1(node, word_lut)?;
                let mut vm: Vm = Default::default();
                let mut context: Context = Default::default();

                run_program_with_preamble(&mut vm, &mut context, &program)?
            }
        };

        let program = compile_program_for_trait(parameter_ast, word_lut, global_mapping)?;

        Ok(Trait::new(initial_value, program, index))
    }
}

#[derive(Debug)]
pub struct TraitList {
    pub traits: Vec<Trait>,
    pub seed_value: i32,
    pub global_mapping: BTreeMap<Iname, i32>,
}

impl TraitList {
    fn new() -> Self {
        TraitList {
            traits: Vec::new(),
            // todo: when is the seed_value set properly?
            seed_value: 42,
            global_mapping: BTreeMap::new(),
        }
    }

    pub fn get_trait(&self, idx: usize) -> &Trait {
        &self.traits[idx]
    }

    pub fn compile(ast: &[Node], word_lut: &WordLut) -> Result<Self> {
        // this top-level compilation is only to get the user defined global mappings
        let mut trait_list = TraitList::new();
        let mut compilation = Compilation::new();
        let compiler = Compiler::new();

        compiler.compile_common(&mut compilation, &ast)?;
        trait_list.global_mapping = compilation.get_user_defined_globals();
        for n in ast {
            trait_list.ga_traverse(&n, word_lut)?;
        }

        Ok(trait_list)
    }

    fn ga_traverse(&mut self, node: &Node, word_lut: &WordLut) -> Result<()> {
        match node {
            Node::List(ns, meta) => {
                if let Some(meta) = meta {
                    self.add_single_trait(&node, &meta, word_lut)?;
                };

                for n in ns {
                    self.ga_traverse(n, word_lut)?;
                }
            }
            Node::Vector(ns, meta) => {
                if let Some(meta) = meta {
                    self.add_multiple_traits(ns, &meta, word_lut)?;
                }

                for n in ns {
                    self.ga_traverse(n, word_lut)?;
                }
            }
            Node::Float(_, _, meta) => {
                if let Some(meta) = meta {
                    self.add_single_trait(&node, &meta, word_lut)?;
                }
            }
            Node::Name(_, _, meta) => {
                if let Some(meta) = meta {
                    self.add_single_trait(&node, &meta, word_lut)?;
                }
            }
            Node::Label(_, _, meta) => {
                if let Some(meta) = meta {
                    self.add_single_trait(&node, &meta, word_lut)?;
                }
            }
            _ => {}
        };

        Ok(())
    }

    fn add_single_trait(&mut self, node: &Node, meta: &NodeMeta, word_lut: &WordLut) -> Result<()> {
        let t = Trait::compile(
            node,
            word_lut,
            &meta.parameter_ast,
            &self.global_mapping,
            None,
        )?;

        self.traits.push(t);

        Ok(())
    }

    fn add_multiple_traits(
        &mut self,
        nodes: &[Node],
        meta: &NodeMeta,
        word_lut: &WordLut,
    ) -> Result<()> {
        let mut i: usize = 0;
        for n in nodes.iter() {
            match n {
                Node::Whitespace(_, _) | Node::Comment(_, _) => {
                    // ignoring whitespace and comments
                }
                _ => {
                    let t = Trait::compile(
                        n,
                        word_lut,
                        &meta.parameter_ast,
                        &self.global_mapping,
                        Some(i),
                    )?;
                    self.traits.push(t);
                    i += 1;
                }
            }
        }

        Ok(())
    }

    fn push_trait_during_unpack(&mut self, a_trait: Trait) {
        self.traits.push(a_trait);
    }

    fn set_seed_during_unpack(&mut self, seed_value: i32) {
        self.seed_value = seed_value;
    }
}

impl Packable for TraitList {
    fn pack(&self, cursor: &mut String) -> Result<()> {
        Mule::pack_i32_sp(cursor, self.seed_value);

        Mule::pack_usize_sp(cursor, self.global_mapping.len());
        for (iname, map_val) in &self.global_mapping {
            iname.pack(cursor)?;
            Mule::pack_space(cursor);

            Mule::pack_i32_sp(cursor, *map_val);
        }

        Mule::pack_usize(cursor, self.traits.len());
        for t in &self.traits {
            Mule::pack_space(cursor);
            t.pack(cursor)?;
        }

        Ok(())
    }

    fn unpack(cursor: &str) -> Result<(Self, &str)> {
        let mut trait_list = TraitList::new();

        let (seed_value, rem) = Mule::unpack_i32_sp(cursor)?;
        trait_list.set_seed_during_unpack(seed_value);

        let mut global_mapping = BTreeMap::new();
        let (num_mappings, rem) = Mule::unpack_usize_sp(rem)?;
        let mut r = rem;
        for _ in 0..num_mappings {
            let (iname, rem) = Iname::unpack(r)?;
            let rem = Mule::skip_space(rem);
            r = rem;
            let (map_val, rem) = Mule::unpack_i32_sp(r)?;
            r = rem;
            global_mapping.insert(iname, map_val);
        }
        trait_list.global_mapping = global_mapping;

        let (num_traits, rem) = Mule::unpack_usize(r)?;

        let mut r = rem;
        for _ in 0..num_traits {
            r = Mule::skip_space(r);
            let (a_trait, rem) = Trait::unpack(r)?;
            r = rem;
            trait_list.push_trait_during_unpack(a_trait);
        }

        Ok((trait_list, r))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    fn compile_trait_list(s: &str) -> Result<TraitList> {
        let (ast, word_lut) = parse(s).unwrap();
        TraitList::compile(&ast, &word_lut)
    }

    fn trait_single_float(t: &Trait, expected: f32) {
        assert_eq!(t.within_vector, false);
        if let Var::Float(f) = t.initial_value {
            assert_eq!(f, expected);
        } else {
            assert!(false);
        }
    }

    fn trait_multiple_float(t: &Trait, expected: f32, index: usize) {
        assert_eq!(t.within_vector, true);
        assert_eq!(t.index, index);
        if let Var::Float(f) = t.initial_value {
            assert_eq!(f, expected);
        } else {
            assert!(false);
        }
    }

    fn trait_multiple_v2d(t: &Trait, expected_x: f32, expected_y: f32, index: usize) {
        assert_eq!(t.within_vector, true);
        assert_eq!(t.index, index);
        if let Var::V2D(x, y) = t.initial_value {
            assert_eq!(x, expected_x);
            assert_eq!(y, expected_y);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_trait_list() {
        let trait_list =
            compile_trait_list("(+ {4 (gen/scalar min: 2 max: 9)}) {6 (gen/scalar min: 2 max: 9)}")
                .unwrap();
        assert_eq!(trait_list.traits.len(), 2);
        trait_single_float(&trait_list.traits[0], 4.0);
        trait_single_float(&trait_list.traits[1], 6.0);
    }

    #[test]
    fn test_trait_list_2() {
        // this will create 2 genes, each one for a V2D
        match compile_trait_list("{[[0.1 0.2] [0.3 0.4]] (gen/2d min: 50 max: 60)}") {
            Ok(trait_list) => {
                assert_eq!(trait_list.traits.len(), 2);
                trait_multiple_v2d(&trait_list.traits[0], 0.1, 0.2, 0);
                trait_multiple_v2d(&trait_list.traits[1], 0.3, 0.4, 1);
            }
            Err(_) => assert!(false),
        };
    }

    #[test]
    fn pack_unpack_trait_list() {
        let trait_list = compile_trait_list(
            "(bezier tessellation: 30
        line-width-start: {50 (gen/scalar min: 30 max: 60)}
        line-width-end: {10 (gen/scalar min: 5 max: 20)}
        brush: brush-c
        coords: [[0 500] [200 900] [400 100] [600 500]]
        colour: (col/rgb r: 1 g: 0.3 b: 0 alpha: 1))",
        )
        .unwrap();

        assert_eq!(trait_list.traits.len(), 2);
        trait_single_float(&trait_list.traits[0], 50.0);
        trait_single_float(&trait_list.traits[1], 10.0);

        let mut packed = "".to_string();
        let packed_res = trait_list.pack(&mut packed);
        assert!(packed_res.is_ok());

        assert_eq!(packed, "42 0 2 0 0 FLOAT 50 0 6 JUMP INT 1 INT 0 LOAD MEM 3 INT 0 LOAD MEM 3 FLOAT 60 LOAD MEM 3 FLOAT 30 NATIVE NATIVE gen/scalar INT 2 STOP INT 0 INT 0 0 0 FLOAT 10 0 6 JUMP INT 1 INT 0 LOAD MEM 3 INT 0 LOAD MEM 3 FLOAT 20 LOAD MEM 3 FLOAT 5 NATIVE NATIVE gen/scalar INT 2 STOP INT 0 INT 0");

        let res = TraitList::unpack(&packed);
        match res {
            Ok((unpacked_trait_list, _)) => {
                assert_eq!(unpacked_trait_list.traits.len(), 2);
                trait_single_float(&unpacked_trait_list.traits[0], 50.0);
                trait_single_float(&unpacked_trait_list.traits[1], 10.0);
            }
            Err(e) => {
                println!("{:?}", e);
                assert_eq!(false, true);
            }
        }
    }

    #[test]
    fn pack_unpack_trait_list_2() {
        // this contains some global defines which will be packed
        let trait_list = compile_trait_list(
            "(define aaa 97 bbb 98 ccc 99)
        (bezier tessellation: 30
        line-width-start: {50 (gen/scalar min: 30 max: 60)}
        line-width-end: {10 (gen/scalar min: 5 max: 20)}
        brush: brush-c
        coords: [[0 500] [200 900] [400 100] [600 500]]
        colour: (col/rgb r: 1 g: 0.3 b: 0 alpha: 1))",
        )
        .unwrap();

        assert_eq!(trait_list.traits.len(), 2);
        trait_single_float(&trait_list.traits[0], 50.0);
        trait_single_float(&trait_list.traits[1], 10.0);

        let mut packed = "".to_string();
        let packed_res = trait_list.pack(&mut packed);
        assert!(packed_res.is_ok());

        assert_eq!(packed, "42 3 0 13 1 14 2 15 2 0 0 FLOAT 50 3 0 3 aaa 1 3 bbb 2 3 ccc 6 JUMP INT 1 INT 0 LOAD MEM 3 INT 0 LOAD MEM 3 FLOAT 60 LOAD MEM 3 FLOAT 30 NATIVE NATIVE gen/scalar INT 2 STOP INT 0 INT 0 0 0 FLOAT 10 3 0 3 aaa 1 3 bbb 2 3 ccc 6 JUMP INT 1 INT 0 LOAD MEM 3 INT 0 LOAD MEM 3 FLOAT 20 LOAD MEM 3 FLOAT 5 NATIVE NATIVE gen/scalar INT 2 STOP INT 0 INT 0");

        let res = TraitList::unpack(&packed);
        match res {
            Ok((unpacked_trait_list, _)) => {
                assert_eq!(unpacked_trait_list.traits.len(), 2);
                trait_single_float(&unpacked_trait_list.traits[0], 50.0);
                trait_single_float(&unpacked_trait_list.traits[1], 10.0);
            }
            Err(e) => {
                println!("{:?}", e);
                assert_eq!(false, true);
            }
        }
    }
}
