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

/* eslint-disable no-use-before-define */

import Interpreter from '../../app/js/lang/Interpreter';
import Parser from '../../app/js/lang/Parser';
import Lexer from '../../app/js/lang/Lexer';
import Compiler from '../../app/js/lang/Compiler';
import Genetic from '../../app/js/lang/Genetic';
import Bind from '../../app/js/lang/Bind';

import {expect} from 'chai';

describe('Interpreter', () => {

  let e;
  let key;
  let val;
  const epsilon = 0.01;

  beforeEach(() => {
    e = Bind.addClassicBindings(
      Bind.addSpecialDebugBindings(
        Bind.addSpecialBindings(
          Interpreter.getBasicEnv())));
    key = 'foo';
    val = 5;
    e = e.set(key, { binding: val });
  });

  it('should evaluate a bracketed form', () => {
    const [_env, res, error] = evalForm(e, '(* 2 {4})');
    expect(res).to.be.closeTo(8, epsilon);
  });

  it('should error when evaluating an unbound variable', () => {
    const [_env, _form, error] = evalForm(e, '(* bar 2)');
    expect(error).to.equal('bar is undefined');
  });

  it('should error when evaluating an unbound function', () => {
    const [_env, _form, error] = evalForm(e, '(baq 1 2)');
    expect(error).to.equal('baq is undefined');
  });

  it('should error', () => {
    const [_env, _form, error] = evalForm(e, '(+ 1 "a")');
    expect(error).to.equal('all arguments to + should be numbers');
  });

  it('should error when binding an undefined variable', () => {
    const [_env, _form, error] = evalForm(e, '(define a 1 b 2 c d)');
    expect(error).to.equal('d is undefined');
  });

  it('should error when binding an odd number of args', () => {
    const [_env, _form, error] = evalForm(e, '(define a 1 b 2 c)');
    expect(error).to.equal('define should have an even number of args');
  });

  it('should error when if condition is invalid', () => {
    const [_env, _form, error] = evalForm(e, '(if z 1 2)');
    expect(error).to.equal('z is undefined');
  });

  it('should evaluate simple nodes', () => {
    let [newEnv, res, error] = Interpreter.evaluate(null, 4);

    expect(newEnv).to.equal(null);
    expect(res).to.equal(4);
    expect(error).to.equal(Interpreter.NO_ERROR);

    [newEnv, res, error] = Interpreter.evaluate(null, 12.34);
    expect(res).to.be.closeTo(12.34, epsilon);
    expect(error).to.equal(Interpreter.NO_ERROR);

    [newEnv, res, error] = Interpreter.evaluate(e, ['quote', 'some string']);
    expect(res).to.equal('some string');
    expect(error).to.equal(Interpreter.NO_ERROR);
  });

  it('should get names in the env', () => {
    const [newEnv, res, error] = Interpreter.evaluate(e, key);
    expect(res).to.equal(val);
    expect(newEnv).to.equal(e);
    expect(error).to.equal(Interpreter.NO_ERROR);
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

  function evalForm(env, form) {

    const ts = Lexer.tokenise(form).tokens;
    const ast = Parser.parse(ts).nodes;
    const traits = Genetic.buildTraits(ast);
    const genotype = Genetic.createGenotypeFromInitialValues(traits);
    const backAst = Compiler.compileBackAst(ast);
    const simplifiedAst = Compiler.compileWithGenotype(backAst, genotype);

    // returns [newEnv, res, error]
    return Interpreter.evaluate(env, simplifiedAst[0]);
  }
});
