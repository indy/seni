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

use crate::compiler::{compile_program_1, compile_program_for_trait, Program};
use crate::error::Result;
use crate::parser::{Node, NodeMeta};
use crate::run_program_with_preamble;
use crate::vm::{Var, Vm};

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

    fn compile(node: &Node, parameter_ast: &Vec<Node>, index: Option<usize>) -> Result<Self> {
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
                let program = compile_program_1(node)?;
                let mut vm = Vm::new();

                run_program_with_preamble(&mut vm, &program)?
            }
        };

        let program = compile_program_for_trait(parameter_ast, node)?;

        Ok(Trait::new(initial_value, program, index))
    }
}

#[derive(Debug)]
pub struct TraitList {
    pub traits: Vec<Trait>,
    pub seed_value: i32,
}

impl TraitList {
    fn new() -> Self {
        TraitList {
            traits: Vec::new(),
            // todo: when is the seed_value set properly?
            seed_value: 42,
        }
    }

    pub fn compile(ast: &[Node]) -> Result<Self> {
        let mut trait_list = TraitList::new();

        for n in ast {
            trait_list.ga_traverse(&n)?;
        }

        Ok(trait_list)
    }

    fn ga_traverse(&mut self, node: &Node) -> Result<()> {
        match node {
            Node::List(ns, meta) => {
                if let Some(meta) = meta {
                    self.add_single_trait(&node, &meta)?;
                };

                for n in ns {
                    self.ga_traverse(n)?;
                }
            }
            Node::Vector(ns, meta) => {
                if let Some(meta) = meta {
                    self.add_multiple_traits(ns, &meta)?;
                }

                for n in ns {
                    self.ga_traverse(n)?;
                }
            }
            Node::Float(_, meta) => {
                if let Some(meta) = meta {
                    self.add_single_trait(&node, &meta)?;
                }
            }
            Node::Name(_, _, meta) => {
                if let Some(meta) = meta {
                    self.add_single_trait(&node, &meta)?;
                }
            }
            Node::Label(_, _, meta) => {
                if let Some(meta) = meta {
                    self.add_single_trait(&node, &meta)?;
                }
            }
            // Node::String(s, meta) => {
            // },
            // Node::Whitespace(s, meta) => if let Some(meta) = meta {
            // },
            // Node::Comment(s, meta) => if let Some(meta) = meta {
            // },
            _ => {}
        };

        Ok(())
    }

    fn add_single_trait(&mut self, node: &Node, meta: &NodeMeta) -> Result<()> {
        let t = Trait::compile(node, &meta.parameter_ast, None)?;

        self.traits.push(t);

        Ok(())
    }

    fn add_multiple_traits(&mut self, nodes: &Vec<Node>, meta: &NodeMeta) -> Result<()> {
        let mut i: usize = 0;
        for n in nodes.iter() {
            match n {
                Node::Whitespace(_, _) | Node::Comment(_, _) => {
                    // ignoring whitespace and comments
                }
                _ => {
                    let t = Trait::compile(n, &meta.parameter_ast, Some(i))?;
                    self.traits.push(t);
                    i += 1;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    fn compile_trait_list(s: &str) -> Result<TraitList> {
        let (ast, _) = parse(s).unwrap();
        TraitList::compile(&ast)
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
}
