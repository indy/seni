/*
 *  Seni
 *  Copyright (C) 2016 Inderjit Gill <email@indy.io>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

import Lexer from '../../app/js/lang/Lexer';
import Parser from '../../app/js/lang/Parser';
import Unparser from '../../app/js/lang/Unparser';
import Genetic from '../../app/js/lang/Genetic';
import Compiler from '../../app/js/lang/Compiler';

import {expect} from 'chai';

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

  it('unparse', () => {
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
    expectToUnparse('(quote ("hello"))');
    expectToUnparse('\'bye');
    expectToUnparse('\'("hello")');
    expectToUnparse('(hello \'(a b c) \'(1 2 3) \'(a: b: c:))');
  });

  it('unparse alterable expressions', () => {
    expectToUnparse('background {(col/rgb r: 1 g: 1 b: 1 alpha: 1) (col)}');
    expectToUnparse('{"hello" (something "foo" "bar")}');
    expectToUnparse('{true (something)}');
    expectToUnparse('(foo {"hello"})');
    expectToUnparse('(foo {"hello" (something "foo" "bar")})');
    expectToUnparse('(+ 1 2 {3 (int)})');
    expectToUnparse('(+ 1 { 3 (int)})');
    expectToUnparse('{["a"]}');
    expectToUnparse('{[["a"] ["a"]]}');
    expectToUnparse(`({focal/vline (select from: (list 'focal/point
                                                       'focal/hline
                                                       'focal/vline))})`);
    expectToUnparse(`({red (select from: ['black
                                          'red
                                          'white])})`);
    expectToUnparse(`({[red green blue] (select from: ['black
                                                       'red
                                                       'white])})`);
  });

  it('unparse with different genotypes', () => {
    expect(seededUnparse('(+ {1 (int)} {3 (int)})', 32))
      .to.equal('(+ {51 (int)} {79 (int)})');
    expect(seededUnparse('(+ {1 (int)})', 32))
      .to.equal('(+ {51 (int)})');
  });

  it('unparse vectors', () => {
    const f = '(define f {[1 2] (select from: [1 2 3 4])})';
    const g = '(define f {[4 1] (select from: [1 2 3 4])})';

    expectToUnparse(f);
    expect(seededUnparse(f, 33)).to.equal(g);

    expectToUnparse('(define f {["a"] (l from: ["a" "b"])})');
  });

});
