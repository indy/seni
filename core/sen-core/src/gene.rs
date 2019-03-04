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

use crate::colour::Colour;
use crate::error::{Error, Result};
use crate::keywords::Keyword;
use crate::prng::PrngStateStruct;
use crate::trait_list::{Trait, TraitList};
use crate::vm::{Env, Var, Vm};

/*
GeneVar is a subset of the Var enum. Since Gene is a member of NodeMeta it
needs the PartialEq trait and prng (which is used in the Var enum) uses
XorShiftRng which doesn't implement PartialEq
*/

#[derive(Clone, Debug, PartialEq)]
pub enum Gene {
    Int(i32),
    Float(f32),
    Bool(bool),
    Keyword(Keyword),
    Long(u64),
    Name(i32), // todo: how do names work with genes? should the String also be here?
    Colour(Colour),
    V2D(f32, f32),
}

impl Gene {
    pub fn from_var(var: &Var) -> Result<Self> {
        match var {
            Var::Int(i) => Ok(Gene::Int(*i)),
            Var::Float(fl) => Ok(Gene::Float(*fl)),
            Var::Bool(b) => Ok(Gene::Bool(*b)),
            Var::Keyword(kw) => Ok(Gene::Keyword(*kw)),
            Var::Long(u) => Ok(Gene::Long(*u)),
            Var::Name(i) => Ok(Gene::Name(*i)),
            Var::Colour(col) => Ok(Gene::Colour(*col)),
            Var::V2D(fl1, fl2) => Ok(Gene::V2D(*fl1, *fl2)),
            _ => Err(Error::Gene("from_var: incompatible input var".to_string())),
        }
    }

    pub fn build_from_trait(vm: &mut Vm, t: &Trait) -> Result<Self> {
        let env = Env::new();

        vm.reset();

        vm.building_with_trait_within_vector = t.within_vector;
        vm.trait_within_vector_index = t.index;

        vm.interpret(&env, &t.program)?;
        let var = vm.top_stack_value()?;

        vm.building_with_trait_within_vector = false;
        vm.trait_within_vector_index = 0;

        Gene::from_var(&var)
    }
}

#[derive(Debug)]
pub struct Genotype {
    pub genes: Vec<Gene>,
    pub current_gene_index: usize,
}

impl Genotype {
    pub fn new() -> Self {
        Genotype {
            genes: Vec::new(),
            current_gene_index: 0,
        }
    }

    pub fn build_genotypes(
        trait_list: &TraitList,
        population_size: i32,
        seed: i32,
    ) -> Result<Vec<Self>> {
        let mut genotypes: Vec<Genotype> = Vec::new();
        let mut prng = PrngStateStruct::new(seed, 1.0, 655536.0);

        for _ in 0..population_size {
            let genotype_seed = prng.prng_f32_defined_range() as i32;
            genotypes.push(Genotype::build_from_seed(trait_list, genotype_seed)?);
        }

        Ok(genotypes)
    }

    // build_from_trait_list
    pub fn build_from_seed(trait_list: &TraitList, seed: i32) -> Result<Self> {
        let mut vm = Vm::new();
        let mut genotype = Genotype::new();

        // the seed is set once per genotype (should it be once per-gene?)
        //
        vm.prng_state.set_state(seed);

        for t in &trait_list.traits {
            genotype.genes.push(Gene::build_from_trait(&mut vm, t)?);
        }

        Ok(genotype)
    }

    pub fn build_from_initial_values(trait_list: &TraitList) -> Result<Self> {
        let mut genotype = Genotype::new();

        for t in &trait_list.traits {
            genotype.genes.push(Gene::from_var(&t.initial_value)?);
        }

        Ok(genotype)
    }

    // crossover
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colour::*;
    use crate::parser::parse;
    use crate::{compile_and_execute, compile_program_with_genotype, run_program_with_preamble};

    pub fn run_with_seeded_genotype(s: &str, seed: i32) -> Result<(Var, Genotype)> {
        let mut vm = Vm::new();
        // todo: cache the preamble program
        let (mut ast, _word_lut) = parse(s)?;

        let trait_list = TraitList::compile(&ast)?;
        let mut genotype = Genotype::build_from_seed(&trait_list, seed)?;
        let program = compile_program_with_genotype(&mut ast, &mut genotype)?;

        let var = run_program_with_preamble(&mut vm, &program)?;

        Ok((var, genotype))
    }

    fn geno_test(
        expr: &str,
        seed: i32,
        genotype_length: usize,
        expected_normal: f32,
        expected_variant: f32,
    ) {
        let res = compile_and_execute(expr).unwrap();
        is_float(&res, expected_normal);

        let (res, genotype) = run_with_seeded_genotype(expr, seed).unwrap();
        assert_eq!(genotype.genes.len(), genotype_length);
        is_float(&res, expected_variant);
    }

    fn geno_test_2d(
        expr: &str,
        seed: i32,
        genotype_length: usize,
        expected_normal: (f32, f32),
        expected_variant: (f32, f32),
    ) {
        let res = compile_and_execute(expr).unwrap();
        is_2d(&res, expected_normal);

        let (res, genotype) = run_with_seeded_genotype(expr, seed).unwrap();
        assert_eq!(genotype.genes.len(), genotype_length);
        is_2d(&res, expected_variant);
    }

    fn compile_trait_list(s: &str) -> Result<TraitList> {
        let (ast, _) = parse(s).unwrap();
        TraitList::compile(&ast)
    }

    fn gene_float(g: &Gene, expected: f32) {
        if let Gene::Float(f) = g {
            assert_eq!(*f, expected);
        } else {
            assert!(false);
        }
    }

    fn gene_2d(g: &Gene, expected_x: f32, expected_y: f32) {
        if let Gene::V2D(x, y) = g {
            assert_eq!(*x, expected_x);
            assert_eq!(*y, expected_y);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn genotype_from_initial_values_1() {
        let trait_list =
            compile_trait_list("(+ {4 (gen/scalar min: 2 max: 9)}) {6 (gen/scalar min: 2 max: 9)}")
                .unwrap();

        let genotype = Genotype::build_from_initial_values(&trait_list).unwrap();

        assert_eq!(genotype.genes.len(), 2);
        gene_float(&genotype.genes[0], 4.0);
        gene_float(&genotype.genes[1], 6.0);
    }

    #[test]
    fn genotype_from_initial_values_2() {
        let trait_list =
            compile_trait_list("{[[0.1 0.2] [0.3 0.4]] (gen/2d min: 50 max: 60)}").unwrap();

        let genotype = Genotype::build_from_initial_values(&trait_list).unwrap();

        assert_eq!(genotype.genes.len(), 2);
        gene_2d(&genotype.genes[0], 0.1, 0.2);
        gene_2d(&genotype.genes[1], 0.3, 0.4);
    }

    #[test]
    fn genotype_from_seed_1() {
        let trait_list =
            compile_trait_list("(+ {4 (gen/scalar min: 2 max: 9)}) {6 (gen/scalar min: 2 max: 9)}")
                .unwrap();

        let genotype = Genotype::build_from_seed(&trait_list, 432).unwrap();

        assert_eq!(genotype.genes.len(), 2);
        gene_float(&genotype.genes[0], 8.82988);
        gene_float(&genotype.genes[1], 6.2474613);
    }

    #[test]
    fn genotype_from_seed_2() {
        let trait_list =
            compile_trait_list("{[[0.1 0.2] [0.3 0.4]] (gen/2d min: 50 max: 60)}").unwrap();

        let genotype = Genotype::build_from_seed(&trait_list, 432).unwrap();

        assert_eq!(genotype.genes.len(), 2);
        gene_2d(&genotype.genes[0], 59.75697, 56.067802);
        gene_2d(&genotype.genes[1], 55.85068, 57.474014);
    }

    fn is_float(var: &Var, expected: f32) {
        if let Var::Float(f) = var {
            assert_eq!(*f, expected);
        } else {
            assert!(false);
        }
    }

    fn is_2d(var: &Var, expected: (f32, f32)) {
        if let Var::V2D(x, y) = var {
            assert_eq!(*x, expected.0);
            assert_eq!(*y, expected.1);
        } else {
            assert!(false);
        }
    }

    fn is_col(var: &Var, expected: &Colour) {
        if let Var::Colour(col) = var {
            assert_eq!(col.format, expected.format);
            assert_eq!(col.e0, expected.e0);
            assert_eq!(col.e1, expected.e1);
            assert_eq!(col.e2, expected.e2);
            assert_eq!(col.e3, expected.e3);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn genotype_col() {
        let s = "{(col/rgb r: 0.1) (gen/col alpha: 0.3)}";

        let res = compile_and_execute(s).unwrap();
        is_col(&res, &Colour::new(ColourFormat::Rgb, 0.1, 0.0, 0.0, 1.0));

        let (res, genotype) = run_with_seeded_genotype(s, 432).unwrap();
        is_col(
            &res,
            &Colour::new(ColourFormat::Rgb, 0.97569704, 0.6067802, 0.585068, 0.3),
        );
        assert_eq!(genotype.genes.len(), 1);
    }

    #[test]
    fn genotype_compile() {
        geno_test(
            "(+ {3 (gen/scalar min: 10 max: 20)} {4 (gen/scalar min: 100 max: 105)})",
            432,
            2,
            7.0,
            122.79086,
        );
        geno_test("(+ 6 {3 (gen/int min: 1 max: 100)})", 432, 1, 9.0, 104.0);
        geno_test(
            "(+ 6 {3 (gen/scalar min: 1 max: 100)})",
            432,
            1,
            9.0,
            103.59401,
        );
        geno_test("(+ 6 {3 (gen/int min: 1 max: 100)})", 874, 1, 9.0, 60.0);
        geno_test(
            "(+ 6 {3 (gen/scalar min: 1 max: 100)})",
            874,
            1,
            9.0,
            59.47561,
        );
    }

    #[test]
    fn genotype_compile_stray() {
        geno_test("{3 (gen/stray from: 3 by: 0.5)}", 432, 1, 3.0, 3.475697);
        geno_test("{3 (gen/stray-int from: 3 by: 0.5)}", 432, 1, 3.0, 3.0);
    }

    #[test]
    fn genotype_compile_stray_2d() {
        // genotype has a length of 2
        geno_test_2d(
            "{[100 200] (gen/stray-2d from: [100 200] by: [10 10])}",
            7524,
            2,
            (100.0, 200.0),
            (93.04805, 197.49728),
        );
    }

    #[test]
    fn genotype_compile_vectors() {
        // gen/2d in this expr will produce a genotype with 2 genes, each gene will be a V2D

        {
            let expr = "{[[0.1 0.2] [0.3 0.4]] (gen/2d)}";
            let seed = 752;

            // assert the default case [0.1 0.2] [0.3 0.4]:
            let res = compile_and_execute(expr).unwrap();
            if let Var::Vector(vs) = res {
                assert_eq!(vs.len(), 2);
                is_2d(&vs[0], (0.1, 0.2));
                is_2d(&vs[1], (0.3, 0.4));
            } else {
                assert!(false);
            }

            let (res, genotype) = run_with_seeded_genotype(expr, seed).unwrap();
            if let Var::Vector(vs) = res {
                assert_eq!(vs.len(), 2);
                is_2d(&vs[0], (0.9825403, 0.85869956));
                is_2d(&vs[1], (0.59191173, 0.999328));
            } else {
                assert!(false);
            }

            assert_eq!(genotype.genes.len(), 2);
        }

        {
            let expr = "{[[0.1 0.2] [0.3 0.4]] (gen/2d min: 50 max: 60)}";
            let seed = 752;

            // assert the default case [0.1 0.2] [0.3 0.4]:
            let res = compile_and_execute(expr).unwrap();
            if let Var::Vector(vs) = res {
                assert_eq!(vs.len(), 2);
                is_2d(&vs[0], (0.1, 0.2));
                is_2d(&vs[1], (0.3, 0.4));
            } else {
                assert!(false);
            }

            let (res, genotype) = run_with_seeded_genotype(expr, seed).unwrap();
            if let Var::Vector(vs) = res {
                assert_eq!(vs.len(), 2);
                is_2d(&vs[0], (59.8254, 58.586998));
                is_2d(&vs[1], (55.919117, 59.99328));
            } else {
                assert!(false);
            }

            assert_eq!(genotype.genes.len(), 2);
        }
    }

    #[test]
    fn genotype_compile_multiple_floats() {
        // gen/2d in this expr will produce a genotype with 2 genes, each gene will be a V2D

        {
            let expr = "{[0.977 0.416 0.171] (gen/scalar)}";
            let seed = 922;

            let res = compile_and_execute(expr).unwrap();
            if let Var::Vector(vs) = res {
                assert_eq!(vs.len(), 3);
                is_float(&vs[0], 0.977);
                is_float(&vs[1], 0.416);
                is_float(&vs[2], 0.171);
            } else {
                assert!(false);
            }

            let (res, genotype) = run_with_seeded_genotype(expr, seed).unwrap();
            if let Var::Vector(vs) = res {
                assert_eq!(vs.len(), 3);
                is_float(&vs[0], 0.6279464);
                is_float(&vs[1], 0.46001887);
                is_float(&vs[2], 0.51953447);
            } else {
                assert!(false);
            }

            assert_eq!(genotype.genes.len(), 3);
        }
    }

}
