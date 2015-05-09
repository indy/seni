/*
    Seni
    Copyright (C) 2015  Inderjit Gill <email@indy.io>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

import Parser from '../../src/lang/Parser';
import Lexer from '../../src/lang/Lexer';
import Genetic from '../../src/lang/Genetic';

import chai from 'chai';
const expect = chai.expect;

describe('Genetic', () => {

  function simpleBuildTraits(form) {
    // assumes that the form will compile into a single list
    let ts = Lexer.tokenise(form).tokens;
    let ast = Parser.parse(ts).nodes;

    return Genetic.buildTraits(ast);
  }

  it('should build a traits array from an ast', () => {
    let res = simpleBuildTraits('(+ 3 [4 (int min: 0 max: 8)])');
    expect(res.length).to.equal(1);
    expect(res[0].initialValue).to.equal(4);
    // the ast should be in compiled form
    expect(res[0].simplifiedAst.length).to.equal(1);
    expect(res[0].simplifiedAst[0][0]).to.equal('int');
  });

  it('should default bracketed forms to have an identity function', () => {
    let res = simpleBuildTraits('(+ 2 [1])');
    expect(res.length).to.equal(1);
    expect(res[0].initialValue).to.equal(1);
    expect(res[0].simplifiedAst.length).to.equal(1);
    expect(res[0].simplifiedAst[0][0]).to.equal('identity');
  });

  it('should createGenotypeFromTraits', () => {
    let ts = Lexer.tokenise('(+ 2 [44])').tokens;
    let ast = Parser.parse(ts).nodes;

    let traits = Genetic.buildTraits(ast);

    let genotype = Genetic.createGenotypeFromTraits(traits, 100);

    expect(genotype.size).to.equal(1);
    expect(genotype.get(0).get('value')).to.equal(44);
  });

  it('should createGenotypeFromTraits 2', () => {
    let ts = Lexer.tokenise('(+ 2 [44 (int min: 10 max: 56)])').tokens;
    let ast = Parser.parse(ts).nodes;

    let traits = Genetic.buildTraits(ast);

    let genotype = Genetic.createGenotypeFromTraits(traits, 100);

    expect(genotype.size).to.equal(1);
    // the 11 is returned from an rng with a seed of 100
    expect(genotype.get(0).get('value')).to.equal(11);
  });

  it('should create the same genotype', () => {
    let ts = Lexer.tokenise('(+ 2 [44 (int min: 10 max: 56)])').tokens;
    let ast = Parser.parse(ts).nodes;

    let traits = Genetic.buildTraits(ast);

    let genotype = Genetic.createGenotypeFromTraits(traits, 33);
    expect(genotype.get(0).get('value')).to.equal(49);
    // the same seed should generate the same genotype
    genotype = Genetic.createGenotypeFromTraits(traits, 33);
    expect(genotype.get(0).get('value')).to.equal(49);
  });
});
