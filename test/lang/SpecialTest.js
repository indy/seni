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

import Interpreter from '../../app/js/lang/Interpreter';
import Parser from '../../app/js/lang/Parser';
import Lexer from '../../app/js/lang/Lexer';
import Compiler from '../../app/js/lang/Compiler';
import Genetic from '../../app/js/lang/Genetic';
import Bind from '../../app/js/lang/Bind';

import {expect} from 'chai';

describe('Special', () => {

  let e;
  let key;
  let val;
  const epsilon = 0.01;

  beforeEach(() => {
    e = Bind.addClassicBindings(
      Bind.addSpecialBindings(
        Interpreter.getBasicEnv()));
  });

  it('should test if', () => {
    let [newEnv, res] = evalForm(e, '(if true 2 4)');
    expect(res).to.equal(2);
    expect(newEnv).to.equal(e);

    [newEnv, res] = evalForm(e, '(if false 2 4)');
    expect(res).to.equal(4);
    expect(newEnv).to.equal(e);
  });

  it('should test quote', () => {
    let [newEnv, res] = evalForm(e, '(quote something)');
    expect(res).to.equal('something');
    expect(newEnv).to.equal(e);

    [newEnv, res] = evalForm(e, '(quote (+ 4 2))');
    expect(res).to.deep.equal(['+', 4, 2]);
    expect(newEnv).to.equal(e);
  });

  it('should test fn', () => {
    let [newEnv, res] = evalForm(e, '(fn (addup x: 2) (+ x x))');
    expect(newEnv.has('addup')).to.equal(true);

    [newEnv, res] = evalForm(newEnv, '(addup x: 5)');
    expect(res).to.equal(10);

    [newEnv, res] = evalForm(newEnv, '(addup)');
    expect(res).to.equal(4);
  });

  it('should be able to invoke a function while defining it', () => {
    // invoke while defining
    let [newEnv, res] = evalForm(e, '((fn (addup x: 2) (+ x x)))');
    expect(newEnv.has('addup')).to.equal(true);
    expect(res).to.equal(4);

    [newEnv, res] = evalForm(e, '((fn (addup x: 2) (+ x x)) x: 7)');
    expect(newEnv.has('addup')).to.equal(true);
    expect(res).to.equal(14);
  });

  it('should test define', () => {
    const [newEnv, res] = evalForm(e, '(define monkey 42)');
    expect(newEnv.has('monkey')).to.equal(true);
    expect(newEnv.get('monkey').binding).to.equal(42);
    // define should also evaluate to it's set values
    expect(res).to.equal(42);
  });

  it('should test defining multiple values', () => {
    const [newEnv, res] = evalForm(e, '(define monkey 42 ape (+ 6 6))');
    expect(newEnv.has('monkey')).to.equal(true);
    expect(newEnv.get('monkey').binding).to.equal(42);
    expect(newEnv.has('ape')).to.equal(true);
    expect(newEnv.get('ape').binding).to.equal(12);
    expect(res).to.equal(12);
  });

  it('should test defining destructured values', () => {
    const [newEnv, res] = evalForm(e, '(define [a b] [2 (+ 3 4)])');
    expect(newEnv.has('a')).to.equal(true);
    expect(newEnv.get('a').binding).to.equal(2);
    expect(newEnv.has('b')).to.equal(true);
    expect(newEnv.get('b').binding).to.equal(7);
    expect(res).to.equal(7);
  });

  it('should test begin', () => {
    let [_, res] = evalForm(e, '(begin (+ 1 1) (+ 2 2))');
    expect(res).to.equal(4);

    [_, res] = evalForm(e, '(begin (+ 1 1))');
    expect(res).to.equal(2);
  });

  it('should test loop: from/to', () => {
    let [env, res] = evalForm(e,`
(define bar (list))
(loop (a from: 0 to: 4 increment: 1) (append bar a))`);

    const bar = env.get('bar');
    expect(bar.binding).to.deep.equal([0, 1, 2, 3]);
  });

  it('should test loop: from/upto', () => {
    let [env, res] = evalForm(e,`
(define bar (list))
(loop (a from: 0 upto: 4 increment: 1) (append bar a))`);

    const bar = env.get('bar');
    expect(bar.binding).to.deep.equal([0, 1, 2, 3, 4]);
  });

  it('should test loop: increment', () => {
    let [env, res] = evalForm(e,`
(define bar (list))
(loop (a from: 0 to: 12 increment: 2) (append bar a))`);

    const bar = env.get('bar');
    expect(bar.binding).to.deep.equal([0, 2, 4, 6, 8, 10]);
  });

  // todo: steps should only apply when upto is used
  // otherwise the 'to' parameter becomes inclusive
  it('should test loop: from/to steps', () => {
    let [env, res] = evalForm(e,`
(define bar (list))
(loop (a from: 0 to: 40 steps: 4) (append bar a))`);

    const bar = env.get('bar');
    expect(bar.binding[0]).to.be.closeTo(0.00, epsilon);
    expect(bar.binding[1]).to.be.closeTo(13.33, epsilon);
    expect(bar.binding[2]).to.be.closeTo(26.66, epsilon);
    expect(bar.binding[3]).to.be.closeTo(40.00, epsilon);
  });
  //   it('should test loop: negative increment', () => {
  //     let [env, res] = evalForm(e,`
  // (define bar (list))
  // (loop (a from: 12 to: 0 increment: -2) (append bar a))`);
  //
  //     const bar = env.get('bar');
  //     expect(bar.binding).to.deep.equal([12, 10, 8, 6, 4, 2]);
  //   });

  function evalForm(env, form) {

    const ts = Lexer.tokenise(form).tokens;
    const ast = Parser.parse(ts).nodes;
    const traits = Genetic.buildTraits(ast);
    const genotype = Genetic.createGenotypeFromInitialValues(traits);
    const backAst = Compiler.compileBackAst(ast);
    const astList = Compiler.compileWithGenotype(backAst, genotype);

    return astList.reduce(([e, res, err], ast) => Interpreter.evaluate(e, ast),
                          [env, undefined, Interpreter.NO_ERROR]);
  }
});
