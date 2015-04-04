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

/* eslint-disable no-use-before-define */
// todo: look into no-unused-expressions
/* eslint-disable no-unused-expressions */

import Interpreter from '../../src/lang/Interpreter';
import Parser from '../../src/lang/Parser';
import Lexer from '../../src/lang/Lexer';
import Compiler from '../../src/lang/Compiler';
import Genetic from '../../src/lang/Genetic';

describe('eval', () => {

  let e;
  let key;
  let val;
  let epsilon = 0.01;

  beforeEach(() => {
    e = Interpreter.getBasicEnv();
    key = 'foo';
    val = 5;
    e = e.set(key, val);
  });

  it('should evaluate a bracketed form', () => {
    let res = evalForm(e, '(* 2 [4])');
    expect(res[1]).to.be.closeTo(8, epsilon);

    //res = evalForm(e, '(quote ["shabba"])');
    //expect(res[1]).to.be.equal('shabba');
  });

  it('should evaluate simple nodes', () => {
    let [newEnv, res] = Interpreter.evaluate(null, 4);

    expect(newEnv).to.equal(null);
    expect(res).to.equal(4);

    [newEnv, res] = Interpreter.evaluate(null, 12.34);
    expect(res).to.be.closeTo(12.34, epsilon);

    [newEnv, res] = Interpreter.evaluate(e, ['quote', 'some string']);
    expect(res).to.equal('some string');
  });

  it('should get names in the env', () => {
    let [newEnv, res] = Interpreter.evaluate(e, key);
    expect(res).to.equal(val);
    expect(newEnv).to.equal(e);
  });

  it('should test required mathematical functions', () => {
    let [newEnv, res] = evalForm(e, '(* 2 4)');
    expect(res).to.be.closeTo(8, epsilon);
    expect(newEnv).to.equal(e);

    [newEnv, res] = evalForm(e, '(+ 2 4)');
    expect(res).to.be.closeTo(6, epsilon);

    [newEnv, res] = evalForm(e, '(- 10 3)');
    expect(res).to.be.closeTo(7, epsilon);

    [newEnv, res] = evalForm(e, '(- 10 3 5)');
    expect(res).to.be.closeTo(2, epsilon);

    [newEnv, res] = evalForm(e, '(- 42)');
    expect(res).to.be.closeTo(-42, epsilon);

    [newEnv, res] = evalForm(e, '(+ 2 foo)');
    expect(res).to.be.closeTo(7, epsilon);

    [newEnv, res] = evalForm(e, '(+ (* 2 2) (* 3 3))');
    expect(res).to.be.closeTo(13, epsilon);

    [newEnv, res] = evalForm(e, '(/ 90 10)');
    expect(res).to.be.closeTo(9, epsilon);

    [newEnv, res] = evalForm(e, '(/ 90 10 3)');
    expect(res).to.be.closeTo(3, epsilon);
    expect(newEnv).to.equal(e);
  });

  it('should test required comparison functions', () => {
    let [newEnv, res] = evalForm(e, '(= 90 90)');
    expect(res).to.equal('#t');
    expect(newEnv).to.equal(e);

    [newEnv, res] = evalForm(e, '(= 90 90 90)');
    expect(res).to.equal('#t');

    [newEnv, res] = evalForm(e, '(= 90 3)');
    expect(res).to.equal('#f');

    [newEnv, res] = evalForm(e, '(< 54 30)');
    expect(res).to.equal('#t');

    [newEnv, res] = evalForm(e, '(< 54 30 20)');
    expect(res).to.equal('#t');

    [newEnv, res] = evalForm(e, '(< 54 54)');
    expect(res).to.equal('#f');

    [newEnv, res] = evalForm(e, '(< 54 540)');
    expect(res).to.equal('#f');

    [newEnv, res] = evalForm(e, '(> 54 30)');
    expect(res).to.equal('#f');

    [newEnv, res] = evalForm(e, '(> 54 62 72)');
    expect(res).to.equal('#t');

    [newEnv, res] = evalForm(e, '(> 54 54)');
    expect(res).to.equal('#f');

    [newEnv, res] = evalForm(e, '(> 54 540)');
    expect(res).to.equal('#t');
    expect(newEnv).to.equal(e);
  });

  it('should test list', () => {
    let [newEnv, res] = evalForm(e, '(list 90 90)');
    expect(res).to.deep.equal([90, 90]);
    expect(newEnv).to.equal(e);
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

  it('should test define', () => {
    let newEnv = evalForm(e, '(define monkey 42)')[0];
    expect(newEnv.has('monkey')).to.be.true;
    expect(newEnv.get('monkey')).to.equal(42);
  });

  it('should test define for a function2', () => {
    let [newEnv, res] = evalForm(e, '(define (addup x: 2) (+ x x))');
    expect(newEnv.has('addup')).to.be.true;

    [newEnv, res] = evalForm(newEnv, '(addup x: 5)');
    expect(res).to.equal(10);

    [newEnv, res] = evalForm(newEnv, '(addup)');
    expect(res).to.equal(4);
  });

  /*
   it('should test set!', () => {
   expect(e.has('foo')).to.be.true;
   expect(e.get('foo')).to.equal(5);

   let newEnv = evalForm(e, '(set! foo 42)')[0];

   expect(newEnv.has('foo')).to.be.true;
   expect(newEnv.get('foo')).to.equal(42);

   // todo: test that the e env still has foo bound to 5
   });
   */
  /* eslint-disable no-unused-vars */
  it('should test begin', () => {
    expect(e.has('foo')).to.be.true;
    expect(e.get('foo')).to.equal(5);
    let [newEnv, res] = evalForm(e, '(begin (+ 1 1) (+ 2 2))');
    expect(res).to.equal(4);

    [newEnv, res] = evalForm(e, '(begin (+ 1 1))');
    expect(res).to.equal(2);
  });
  /* eslint-enable no-unused-vars */

  it('should test let', () => {
    let [newEnv, res] = evalForm(e, '(let ((a 12) (b 24)) (+ a b foo))');

    expect(newEnv.has('foo')).to.be.true;
    expect(newEnv.get('foo')).to.equal(5);

    expect(res).to.equal(41);

    // let bindings can refer to earlier bindings
    [newEnv, res] = evalForm(e, '(let ((a 2) (b (+ a a))) (+ a b foo))');
    expect(res).to.equal(11);

    // the body of let can contain multiple forms
    [newEnv, res] =
      evalForm(e, '(let ((a 5) (b (+ a a))) (+ a a) (+ a b foo))');
    expect(res).to.equal(20);
  });

  it('should test destructuring let', () => {
    let r = evalForm(e, '(let (((x y) (list 3 4))) (+ x y foo))');
    expect(r[1]).to.equal(12);
  });

  /*
   it('should test loop', () => {
   e.add('bar', 0);
   let [newEnv, res] =
   evalForm(e, '(loop (a from: 0 to: 4 step: 1) (set! bar (+ bar a)))');
   expect(newEnv.get('bar')).to.equal(6);

   // ''until' for <= loop ('to' for < loop)
   newEnv.add('bar', 0);
   [newEnv, res] =
   evalForm(newEnv, '(loop (a from: 0 until: 4 step: 1) (set! bar (+ bar a)))');
   expect(newEnv.get('bar')).to.equal(10);

   newEnv.add('bar', 0);
   [newEnv, res] = evalForm(newEnv, '(loop (a to: 5) (set! bar (+ bar a)))');
   expect(newEnv.get('bar')).to.equal(10);

   newEnv.add('bar', 0);
   [newEnv, res] =
       evalForm(newEnv, '(loop (a to: 5 step: 2) (set! bar (+ bar a)))');
   expect(newEnv.get('bar')).to.equal(6);

   // loop should eval it's arguments
   newEnv.add('bar', 0);
   [newEnv, res] =
       evalForm(newEnv,
                '(let ((x 2)) (loop (a to: 5 step: x) (set! bar (+ bar a))))');
   expect(newEnv.get('bar')).to.equal(6);

   // loop's body should be treated as though it is wrapped in a 'begin'
   newEnv.add('bar', 0);
   [newEnv, res] =
       evalForm(newEnv,
  '(let ((x 2) (y 4)) (loop (a to: 5 step: x) (+ y y) (set! bar (+ bar a))))');
   expect(newEnv.get('bar')).to.equal(6);

   });*/

  function evalForm(env, form) {
    const ts = Lexer.tokenise(form).tokens;
    const ast = Parser.parse(ts).nodes;
    const traits = Genetic.buildTraits(ast);
    const genotype = Genetic.createGenotypeFromInitialValues(traits);
    const compiled = Compiler.compile(ast, genotype);

    // returns [newEnv, res]
    return Interpreter.evaluate(env, compiled.forms[0]);
  }
});
