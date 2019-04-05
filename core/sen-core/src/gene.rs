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
use crate::compiler::compile_preamble;
use crate::error::{Error, Result};
use crate::keywords::Keyword;
use crate::packable::{Mule, Packable};
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

        // setup the env with the global variables in preamble
        let preamble = compile_preamble()?;
        vm.interpret(&env, &preamble)?;

        vm.ip = 0;

        vm.interpret(&env, &t.program)?;
        let var = vm.top_stack_value()?;

        vm.building_with_trait_within_vector = false;
        vm.trait_within_vector_index = 0;

        Gene::from_var(&var)
    }
}

impl Packable for Gene {
    fn pack(&self, cursor: &mut String) -> Result<()> {
        match self {
            Gene::Int(i) => cursor.push_str(&format!("INT {}", i)),
            Gene::Float(fl) => cursor.push_str(&format!("FLOAT {}", fl)),
            Gene::Bool(b) => Mule::pack_label_bool(cursor, "BOOLEAN", *b),
            Gene::Keyword(kw) => {
                cursor.push_str("KW ");
                kw.pack(cursor)?;
            }
            Gene::Long(u) => cursor.push_str(&format!("LONG {}", u)),
            Gene::Name(i) => cursor.push_str(&format!("NAME {}", i)),
            Gene::Colour(col) => {
                cursor.push_str("COLOUR ");
                col.pack(cursor)?;
            }
            Gene::V2D(fl1, fl2) => cursor.push_str(&format!("2D {} {}", fl1, fl2)),
        }

        Ok(())
    }

    fn unpack(cursor: &str) -> Result<(Self, &str)> {
        if cursor.starts_with("INT ") {
            let rem = Mule::skip_forward(cursor, "INT ".len());
            let (val, rem) = Mule::unpack_i32(rem)?;
            Ok((Gene::Int(val), rem))
        } else if cursor.starts_with("FLOAT ") {
            let rem = Mule::skip_forward(cursor, "FLOAT ".len());
            let (val, rem) = Mule::unpack_f32(rem)?;
            Ok((Gene::Float(val), rem))
        } else if cursor.starts_with("BOOLEAN ") {
            let rem = Mule::skip_forward(cursor, "BOOLEAN ".len());
            let (val, rem) = Mule::unpack_bool(rem)?;
            Ok((Gene::Bool(val), rem))
        } else if cursor.starts_with("KW ") {
            let rem = Mule::skip_forward(cursor, "KW ".len());
            let (val, rem) = Keyword::unpack(rem)?;
            Ok((Gene::Keyword(val), rem))
        } else if cursor.starts_with("LONG ") {
            let rem = Mule::skip_forward(cursor, "LONG ".len());
            let (val, rem) = Mule::unpack_u64(rem)?;
            Ok((Gene::Long(val), rem))
        } else if cursor.starts_with("NAME ") {
            let rem = Mule::skip_forward(cursor, "NAME ".len());
            let (val, rem) = Mule::unpack_i32(rem)?;
            Ok((Gene::Name(val), rem))
        } else if cursor.starts_with("COLOUR ") {
            let rem = Mule::skip_forward(cursor, "COLOUR ".len());
            let (val, rem) = Colour::unpack(rem)?;
            Ok((Gene::Colour(val), rem))
        } else if cursor.starts_with("2D ") {
            let rem = Mule::skip_forward(cursor, "2D ".len());
            let (val0, rem) = Mule::unpack_f32_sp(rem)?;
            let (val1, rem) = Mule::unpack_f32(rem)?;
            Ok((Gene::V2D(val0, val1), rem))
        } else {
            Err(Error::Packable("Gene::unpack".to_string()))
        }
    }
}

#[derive(Clone, Debug)]
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

    pub fn num_genes(&self) -> usize {
        self.genes.len()
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

    fn push_gene(&mut self, a_gene: Gene) {
        self.genes.push(a_gene);
    }

    pub fn crossover(&self, other: &Genotype, prng: &mut PrngStateStruct) -> Result<Self> {
        let mut child = Genotype::new();

        let num_genes = self.genes.len();
        let crossover_index: usize = prng.prng_usize_range(0, num_genes);

        for i in 0..crossover_index {
            child.push_gene(self.genes[i].clone())
        }

        for i in crossover_index..num_genes {
            child.push_gene(other.genes[i].clone())
        }

        Ok(child)
    }

    pub fn possibly_mutate(
        &mut self,
        mutation_rate: f32,
        prng: &mut PrngStateStruct,
        trait_list: &TraitList,
    ) -> Result<()> {
        let num_genes = self.genes.len();

        for i in 0..num_genes {
            let r = prng.prng_f32();
            if r < mutation_rate {
                self.gene_generate_new_var(i, prng, trait_list)?;
            }
        }

        Ok(())
    }

    fn gene_generate_new_var(
        &mut self,
        idx: usize,
        prng: &mut PrngStateStruct,
        trait_list: &TraitList,
    ) -> Result<()> {
        let mut vm = Vm::new();
        let t = trait_list.get_trait(idx);

        vm.set_prng_state(prng.clone());
        self.genes[idx] = Gene::build_from_trait(&mut vm, t)?;
        prng.clone_rng(vm.prng_state);

        Ok(())
    }
}

impl Packable for Genotype {
    fn pack(&self, cursor: &mut String) -> Result<()> {
        Mule::pack_usize(cursor, self.genes.len());

        for g in &self.genes {
            Mule::pack_space(cursor);
            g.pack(cursor)?;
        }

        Ok(())
    }

    fn unpack(cursor: &str) -> Result<(Self, &str)> {
        let mut genotype = Genotype::new();

        let (num_genes, rem) = Mule::unpack_usize(cursor)?;

        let mut r = rem;
        for _ in 0..num_genes {
            r = Mule::skip_space(r);
            let (a_gene, rem) = Gene::unpack(r)?;
            r = rem;
            genotype.push_gene(a_gene);
        }

        Ok((genotype, r))
    }
}

pub fn next_generation(
    parents: &Vec<Genotype>,
    population_size: usize,
    mutation_rate: f32,
    rng_seed: i32,
    trait_list: &TraitList,
) -> Result<Vec<Genotype>> {
    if parents.is_empty() {
        return Err(Error::NotedError(
            "next_generation: no parents given".to_string(),
        ));
    }

    // todo: should the children vector be declared with capacity of population_size?

    // copy the parents onto the new generation
    let num_parents = parents.len();
    let mut children: Vec<Genotype> = parents[..].to_vec();

    let mut rng = PrngStateStruct::new(rng_seed, 0.0, 1.0);
    let retry_count = 10;

    for _ in 0..(population_size - num_parents) {
        // select 2 different parents
        let a_index = rng.prng_usize_range(0, num_parents - 1);

        let mut b_index = a_index;
        for _ in 0..retry_count {
            b_index = rng.prng_usize_range(0, num_parents - 1);
            if b_index != a_index {
                break;
            }
        }
        if b_index == a_index {
            b_index = (a_index + 1) % num_parents;
        }

        let a = &parents[a_index];
        let b = &parents[b_index];

        let mut child = a.crossover(&b, &mut rng)?;
        child.possibly_mutate(mutation_rate, &mut rng, trait_list)?;

        children.push(child);
    }

    Ok(children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colour::*;
    use crate::parser::parse;
    use crate::{compile_and_execute, compile_program_with_genotype, run_program_with_preamble};

    pub fn run_with_seeded_genotype(s: &str, seed: i32) -> Result<(Var, Genotype)> {
        // todo: cache the preamble program
        let (mut ast, _word_lut) = parse(s)?;

        let trait_list = TraitList::compile(&ast)?;
        let mut genotype = Genotype::build_from_seed(&trait_list, seed)?;
        let program = compile_program_with_genotype(&mut ast, &mut genotype)?;

        let mut vm = Vm::new();
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

    fn is_keyword(var: &Var, expected: Keyword) {
        if let Var::Keyword(kw) = var {
            assert_eq!(*kw, expected);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn gen_select_preamble_variable() {
        let s = "{transformers (gen/select from: col/procedural-fn-presets)}";
        let res = compile_and_execute(s).unwrap();

        is_keyword(&res, Keyword::Transformers);

        {
            let (res, genotype) = run_with_seeded_genotype(s, 56534).unwrap();
            assert_eq!(genotype.genes.len(), 1);
            is_keyword(&res, Keyword::KnightRider);
        }
        {
            let (res, genotype) = run_with_seeded_genotype(s, 6534).unwrap();
            assert_eq!(genotype.genes.len(), 1);
            is_keyword(&res, Keyword::HotlineMiami);
        }
        {
            let (res, genotype) = run_with_seeded_genotype(s, 2009).unwrap();
            assert_eq!(genotype.genes.len(), 1);
            is_keyword(&res, Keyword::Rainbow);
        }
    }

    #[test]
    fn gen_select_variable() {
        let s = "(define a 2.3 b 3.4 c 4.5 d 5.6)
                 (+ 10.0 {a (gen/select from: '(a b c d))})";
        let res = compile_and_execute(s).unwrap();
        is_float(&res, 12.3);

        // a = 222 b = 223 c = 224 d = 225
        {
            let (res, genotype) = run_with_seeded_genotype(s, 211).unwrap(); // 222
            assert_eq!(genotype.genes.len(), 1);
            dbg!(&genotype.genes[0]);
            is_float(&res, 12.3);

            let (res, genotype) = run_with_seeded_genotype(s, 25).unwrap(); // 224
            assert_eq!(genotype.genes.len(), 1);
            dbg!(&genotype.genes[0]);
            is_float(&res, 14.5);

            let (res, genotype) = run_with_seeded_genotype(s, 37).unwrap(); // 223
            assert_eq!(genotype.genes.len(), 1);
            dbg!(&genotype.genes[0]);
            is_float(&res, 13.4);

            let (res, genotype) = run_with_seeded_genotype(s, 45).unwrap(); // 225
            assert_eq!(genotype.genes.len(), 1);
            dbg!(&genotype.genes[0]);
            is_float(&res, 15.6);
        }
    }

    // #[test]
    fn bug_gen_select_variable_2() {
        /*
         compiling traits will require access to the entire ast, not just the parts inside the {}
         the iname from the trait needs to match up with the correct iname from the whole script

        e.g.
        (define aa 2.3 bb 3.4 cc 4.5 dd 5.6)
        {bb (gen/select from: '(bb cc dd))}

        if just the form in brackets was evaluated, then the inames wouldn't match when compared to the whole ast
         */

        let s = "(define aa 2.3 bb 3.4 cc 4.5 dd 5.6)
                 (+ 10.0 {aa (gen/select from: '(aa bb cc dd))})";
        let res = compile_and_execute(s).unwrap();

        is_float(&res, 12.3);

        {
            let (res, genotype) = run_with_seeded_genotype(s, 211).unwrap();
            assert_eq!(genotype.genes.len(), 1);
            dbg!(&genotype.genes[0]);
            is_float(&res, 12.3);

            let (res, genotype) = run_with_seeded_genotype(s, 25).unwrap();
            assert_eq!(genotype.genes.len(), 1);
            dbg!(&genotype.genes[0]);
            is_float(&res, 14.5);

            let (res, genotype) = run_with_seeded_genotype(s, 37).unwrap();
            assert_eq!(genotype.genes.len(), 1);
            dbg!(&genotype.genes[0]);
            is_float(&res, 13.4);

            let (res, genotype) = run_with_seeded_genotype(s, 45).unwrap();
            assert_eq!(genotype.genes.len(), 1);
            dbg!(&genotype.genes[0]);
            is_float(&res, 15.6);
        }
    }

    #[test]
    fn gen_select_explicit_list() {
        let s = "{1.23 (gen/select from: '(1.1 2.2 3.3 4.4))}";
        let res = compile_and_execute(s).unwrap();

        is_float(&res, 1.23);

        let seed = 6445;
        let genotype_length = 1;

        let (res, genotype) = run_with_seeded_genotype(s, seed).unwrap();
        assert_eq!(genotype.genes.len(), genotype_length);

        is_float(&res, 4.4);
        // dbg!(&genotype.genes[0]);
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
            let expr = "{[[0.1 0.2] [0.3 0.4] [0.5 0.6]] (gen/2d)}";
            let seed = 752;

            // assert the default case [0.1 0.2] [0.3 0.4] [0.5 0.6]:
            let res = compile_and_execute(expr).unwrap();
            if let Var::Vector(vs) = res {
                assert_eq!(vs.len(), 3);
                is_2d(&vs[0], (0.1, 0.2));
                is_2d(&vs[1], (0.3, 0.4));
                is_2d(&vs[2], (0.5, 0.6));
            } else {
                assert!(false);
            }

            let (res, genotype) = run_with_seeded_genotype(expr, seed).unwrap();
            if let Var::Vector(vs) = res {
                assert_eq!(vs.len(), 3);
                is_2d(&vs[0], (0.9825403, 0.85869956));
                is_2d(&vs[1], (0.59191173, 0.999328));
                is_2d(&vs[2], (0.22858822, 0.4880673));
            } else {
                assert!(false);
            }

            assert_eq!(genotype.genes.len(), 3);
        }

        {
            let expr = "{[[0.1 0.2] [0.3 0.4] [0.5 0.6]] (gen/2d min: 50 max: 60)}";
            let seed = 752;

            // assert the default case [0.1 0.2] [0.3 0.4]:
            let res = compile_and_execute(expr).unwrap();
            if let Var::Vector(vs) = res {
                assert_eq!(vs.len(), 3);
                is_2d(&vs[0], (0.1, 0.2));
                is_2d(&vs[1], (0.3, 0.4));
                is_2d(&vs[2], (0.5, 0.6));
            } else {
                assert!(false);
            }

            let (res, genotype) = run_with_seeded_genotype(expr, seed).unwrap();
            if let Var::Vector(vs) = res {
                assert_eq!(vs.len(), 3);
                is_2d(&vs[0], (59.8254, 58.586998));
                is_2d(&vs[1], (55.919117, 59.99328));
                is_2d(&vs[2], (52.28588, 54.880672));
            } else {
                assert!(false);
            }

            assert_eq!(genotype.genes.len(), 3);
        }
    }

    #[test]
    fn genotype_compile_multiple_floats() {
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

    #[test]
    fn next_generation_test() {
        let expr = "{[0.977 0.416 0.171] (gen/scalar)}";

        let (ast, _) = parse(expr).unwrap();
        let trait_list = TraitList::compile(&ast).unwrap();

        let seed_a = 9876;
        let seed_b = 1234;

        let (_, genotype_a) = run_with_seeded_genotype(expr, seed_a).unwrap();
        let (_, genotype_b) = run_with_seeded_genotype(expr, seed_b).unwrap();

        assert_eq!(genotype_a.genes.len(), 3);
        gene_float(&genotype_a.genes[0], 0.000778846);
        gene_float(&genotype_a.genes[1], 0.5599265);
        gene_float(&genotype_a.genes[2], 0.8937246);

        assert_eq!(genotype_b.genes.len(), 3);
        gene_float(&genotype_b.genes[0], 0.1344259);
        gene_float(&genotype_b.genes[1], 0.052326918);
        gene_float(&genotype_b.genes[2], 0.024050714);

        let parents = vec![genotype_a, genotype_b];
        let children = next_generation(&parents, 5, 0.2, 234, &trait_list).unwrap();

        // first 2 children should be clones of the parents
        assert_eq!(children[0].genes.len(), 3);
        gene_float(&children[0].genes[0], 0.000778846);
        gene_float(&children[0].genes[1], 0.5599265);
        gene_float(&children[0].genes[2], 0.8937246);

        assert_eq!(children[1].genes.len(), 3);
        gene_float(&children[1].genes[0], 0.1344259);
        gene_float(&children[1].genes[1], 0.052326918);
        gene_float(&children[1].genes[2], 0.024050714);

        // 3 children
        assert_eq!(children[2].genes.len(), 3);
        gene_float(&children[2].genes[0], 0.6867611); // mutation
        gene_float(&children[2].genes[1], 0.052326918); // b
        gene_float(&children[2].genes[2], 0.024050714); // b

        assert_eq!(children[3].genes.len(), 3);
        gene_float(&children[3].genes[0], 0.000778846); // a
        gene_float(&children[3].genes[1], 0.5599265); // a
        gene_float(&children[3].genes[2], 0.024050714); // b

        assert_eq!(children[4].genes.len(), 3);
        gene_float(&children[4].genes[0], 0.000778846); // a
        gene_float(&children[4].genes[1], 0.052326918); // b
        gene_float(&children[4].genes[2], 0.024050714); // b

        assert_eq!(children.len(), 5);
    }
}
