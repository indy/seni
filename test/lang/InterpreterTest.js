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

/* eslint-disable no-use-before-define */
// todo: look into no-unused-expressions
/* eslint-disable no-unused-expressions */

import Interpreter from '../../app/js/lang/Interpreter';
import Parser from '../../app/js/lang/Parser';
import Lexer from '../../app/js/lang/Lexer';
import Compiler from '../../app/js/lang/Compiler';
import Genetic from '../../app/js/lang/Genetic';

import chai from 'chai';
const expect = chai.expect;

describe('Interpreter', () => {

  let e;
  let key;
  let val;
  const epsilon = 0.01;

  beforeEach(() => {
    e = Interpreter.getBasicEnv();
    key = 'foo';
    val = 5;
    e = e.set(key, { binding: val });
  });

  it('should evaluate a bracketed form', () => {
    const res = evalForm(e, '(* 2 {4})');
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
    const [newEnv, res] = Interpreter.evaluate(e, key);
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

    [newEnv, res] = evalForm(e, '(> 54 30)');
    expect(res).to.equal('#t');

    [newEnv, res] = evalForm(e, '(> 54 30 20)');
    expect(res).to.equal('#t');

    [newEnv, res] = evalForm(e, '(> 54 54)');
    expect(res).to.equal('#f');

    [newEnv, res] = evalForm(e, '(> 54 540)');
    expect(res).to.equal('#f');

    [newEnv, res] = evalForm(e, '(< 54 30)');
    expect(res).to.equal('#f');

    [newEnv, res] = evalForm(e, '(< 54 62 72)');
    expect(res).to.equal('#t');

    [newEnv, res] = evalForm(e, '(< 54 54)');
    expect(res).to.equal('#f');

    [newEnv, res] = evalForm(e, '(< 54 540)');
    expect(res).to.equal('#t');
    expect(newEnv).to.equal(e);
  });

  it('should test list', () => {
    const [newEnv, res] = evalForm(e, '(list 90 90)');
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
    const [newEnv, res] = evalForm(e, '(define monkey 42)');
    expect(newEnv.has('monkey')).to.be.true;
    expect(newEnv.get('monkey').binding).to.equal(42);
    // define should also evaluate to it's set values
    expect(res).to.equal(42);
  });

  it('should test defining multiple values', () => {
    const [newEnv, res] = evalForm(e, '(define monkey 42 ape (+ 6 6))');
    expect(newEnv.has('monkey')).to.be.true;
    expect(newEnv.get('monkey').binding).to.equal(42);
    expect(newEnv.has('ape')).to.be.true;
    expect(newEnv.get('ape').binding).to.equal(12);
    expect(res).to.equal(12);
  });

  it('should test defining destructured values', () => {
    const [newEnv, res] = evalForm(e, '(define [a b] [2 (+ 3 4)])');
    expect(newEnv.has('a')).to.be.true;
    expect(newEnv.get('a').binding).to.equal(2);
    expect(newEnv.has('b')).to.be.true;
    expect(newEnv.get('b').binding).to.equal(7);
    expect(res).to.equal(7);
  });

  it('should test fn', () => {
    let [newEnv, res] = evalForm(e, '(fn (addup x: 2) (+ x x))');
    expect(newEnv.has('addup')).to.be.true;

    [newEnv, res] = evalForm(newEnv, '(addup x: 5)');
    expect(res).to.equal(10);

    [newEnv, res] = evalForm(newEnv, '(addup)');
    expect(res).to.equal(4);
  });

  it('should be able to invoke a functions while defining it', () => {
    // invoke while defining
    let [newEnv, res] = evalForm(e, '((fn (addup x: 2) (+ x x)))');
    expect(newEnv.has('addup')).to.be.true;
    expect(res).to.equal(4);

    [newEnv, res] = evalForm(e, '((fn (addup x: 2) (+ x x)) x: 7)');
    expect(newEnv.has('addup')).to.be.true;
    expect(res).to.equal(14);
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
    expect(e.get('foo').binding).to.equal(5);
    let [newEnv, res] = evalForm(e, '(begin (+ 1 1) (+ 2 2))');
    expect(res).to.equal(4);

    [newEnv, res] = evalForm(e, '(begin (+ 1 1))');
    expect(res).to.equal(2);
  });

  /* eslint-enable no-unused-vars */
  /*
  it('should test let', () => {
    let [newEnv, res] = evalForm(e, '(let ((a 12) (b 24)) (+ a b foo))');

    expect(newEnv.has('foo')).to.be.true;
    expect(newEnv.get('foo').binding).to.equal(5);

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
    */
  /*
   it('should test loop', () => {
   e.add('bar', 0);
   let [newEnv, res] =
   evalForm(e, '(loop (a from: 0 to: 4 increment: 1) (set! bar (+ bar a)))');

   expect(newEnv.get('bar')).to.equal(6);

   // ''upto' for <= loop ('to' for < loop)
   newEnv.add('bar', 0);
   [newEnv, res] =
   evalForm(newEnv, '(loop (a from: 0 upto: 4 increment: 1)
   (set! bar (+ bar a)))');
   expect(newEnv.get('bar')).to.equal(10);

   newEnv.add('bar', 0);
   [newEnv, res] = evalForm(newEnv, '(loop (a to: 5) (set! bar (+ bar a)))');
   expect(newEnv.get('bar')).to.equal(10);

   newEnv.add('bar', 0);
   [newEnv, res] =
       evalForm(newEnv, '(loop (a to: 5 increment: 2)
                (set! bar (+ bar a)))');
   expect(newEnv.get('bar')).to.equal(6);

   // loop should eval it's arguments
   newEnv.add('bar', 0);
   [newEnv, res] =
       evalForm(newEnv,
                '(let ((x 2)) (loop (a to: 5 incremenet: x)
                (set! bar (+ bar a))))');
   expect(newEnv.get('bar')).to.equal(6);

   // loop's body should be treated as though it is wrapped in a 'begin'
   newEnv.add('bar', 0);
   [newEnv, res] =
       evalForm(newEnv,
  '(let ((x 2) (y 4)) (loop (a to: 5 increment: x)
                (+ y y) (set! bar (+ bar a))))');
   expect(newEnv.get('bar')).to.equal(6);

   });*/

  function evalForm(env, form) {
    const ts = Lexer.tokenise(form).tokens;
    const ast = Parser.parse(ts).nodes;
    const traits = Genetic.buildTraits(ast);
    const genotype = Genetic.createGenotypeFromInitialValues(traits);
    const backAst = Compiler.compileBackAst(ast);
    const simplifiedAst = Compiler.compileWithGenotype(backAst, genotype);

    // returns [newEnv, res]
    return Interpreter.evaluate(env, simplifiedAst[0]);
  }
});
