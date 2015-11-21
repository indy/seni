/*
    Seni
    Copyright (C) 2015 Inderjit Gill <email@indy.io>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program. If not, see <http://www.gnu.org/licenses/>.
*/

// todo: look into no-unused-expressions
/* eslint-disable no-unused-expressions */

import Lexer from '../../src/lang/Lexer';
import Parser from '../../src/lang/Parser';
import Unparser from '../../src/lang/Unparser';
import Genetic from '../../src/lang/Genetic';
import Compiler from '../../src/lang/Compiler';

import chai from 'chai';
const expect = chai.expect;

describe('Unparser', () => {

  function compileAst(form) {
    const ts = Lexer.tokenise(form).tokens;
    const frontAst = Parser.parse(ts).nodes;
    const backAst = Compiler.compileBackAst(frontAst);

    const traits = Genetic.buildTraits(backAst);

    return [frontAst, traits];
  }

  function simpleUnparse(form) {
    const [ast, traits] = compileAst(form);
    const genotype = Genetic.createGenotypeFromInitialValues(traits);

    return Unparser.unparse(ast, genotype);
  }

  function seededUnparse(form, seed) {
    const [ast, traits] = compileAst(form);
    const genotype = Genetic.createGenotypeFromTraits(traits, seed);

    return Unparser.unparse(ast, genotype);
  }

  function expectToUnparse(form) {
    expect(simpleUnparse(form)).to.equal(form);
  }

  it('should unparse', () => {
    expectToUnparse('4');
    expectToUnparse('4.2');
    expectToUnparse('hello');
    expectToUnparse('"some string"');
    expectToUnparse('label:');
    expectToUnparse('true');
    expectToUnparse('4 2 0');
    expectToUnparse('(1)');
    expectToUnparse('(foo 1)');
    expectToUnparse('(foo "hello")');
    expectToUnparse('(list "string")');
    expectToUnparse('(list (list "a"))');
    expectToUnparse('(fn (bar x: 3) (+ x x))');
  });

  it('should unparse alterable expressions', () => {
    expectToUnparse('[(list (list "a") (list "a"))]');
    expectToUnparse('(foo f [(list (list "a"))])');
    expectToUnparse('background [(col/rgb r: 1 g: 1 b: 1 alpha: 1) (col)]');
    expectToUnparse('["hello" (something "foo" "bar")]');
    expectToUnparse('[true (something)]');
    expectToUnparse('(foo ["hello"])');
    expectToUnparse('(foo ["hello" (something "foo" "bar")])');
    expectToUnparse('(+ 1 2 [3 (int)])');
    expectToUnparse('(+ 1 [ 3 (int)])');
  });

  it('should unparse with different genotypes', () => {
    expect(seededUnparse('(+ [1 (int)] [3 (int)])', 32))
      .to.equal('(+ [51 (int)] [79 (int)])');
    expect(seededUnparse('(+ [1 (int)])', 32))
      .to.equal('(+ [51 (int)])');
  });

  it('should unparse map alterables', () => {
    let f = '(define f [(list 1 2) map (select from: (list 1 2 3 4))])';
    let g = '(define f [(list 4 1) map (select from: (list 1 2 3 4))])';

    expectToUnparse(f);
    expect(seededUnparse(f, 33)).to.equal(g);

    expectToUnparse('(define f [(list "a") map (l from: (list "a" "b"))])');
  });
});
