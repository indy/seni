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


import { expect } from 'chai';

import Interpreter from '../../app/js/lang/Interpreter';
import { buildEnv, evalForm } from './eval_helper';

describe(`Interpreter`, () => {

  let e;
  let key;
  let val;
  const epsilon = 0.01;

  beforeEach(() => {
    e = buildEnv();
    key = `foo`;
    val = 5;
    e = e.set(key, { binding: val });
  });

  it(`evaluate a bracketed form`, () => {
    const [_env, res, _error] = evalForm(e, `(* 2 {4})`);
    expect(res).to.be.closeTo(8, epsilon);
  });

  it(`error when evaluating an unbound variable`, () => {
    const [_env, _form, error] = evalForm(e, `(* bar 2)`);
    expect(error).to.equal(`bar is undefined`);
  });

  it(`error when evaluating an unbound function`, () => {
    const [_env, _form, error] = evalForm(e, `(baq 1 2)`);
    expect(error).to.equal(`baq is undefined`);
  });


  it(`error when binding an undefined variable`, () => {
    const [_env, _form, error] = evalForm(e, `(define a 1 b 2 c d)`);
    expect(error).to.equal(`d is undefined`);
  });

  it(`error when binding an odd number of args`, () => {
    const [_env, _form, error] = evalForm(e, `(define a 1 b 2 c)`);
    expect(error).to.equal(`define should have an even number of args`);
  });

  it(`error when if condition is invalid`, () => {
    const [_env, _form, error] = evalForm(e, `(if z 1 2)`);
    expect(error).to.equal(`z is undefined`);
  });

  it(`evaluate simple nodes`, () => {
    let [newEnv, res, error] = Interpreter.evaluate(null, 4);

    expect(newEnv).to.equal(null);
    expect(res).to.equal(4);
    expect(error).to.equal(Interpreter.NO_ERROR);

    [newEnv, res, error] = Interpreter.evaluate(null, 12.34);
    expect(res).to.be.closeTo(12.34, epsilon);
    expect(error).to.equal(Interpreter.NO_ERROR);

    [newEnv, res, error] = Interpreter.evaluate(e, [`quote`, `some string`]);
    expect(res).to.equal(`some string`);
    expect(error).to.equal(Interpreter.NO_ERROR);
  });

  it(`get names in the env`, () => {
    const [newEnv, res, error] = Interpreter.evaluate(e, key);
    expect(res).to.equal(val);
    expect(newEnv).to.equal(e);
    expect(error).to.equal(Interpreter.NO_ERROR);
  });
});
