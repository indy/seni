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

import {expect} from 'chai';

import { buildEnv, evalForm } from './eval_helper';

describe('Classic', () => {
  let e;
  const epsilon = 0.01;

  beforeEach(() => {
    e = buildEnv();

    const key = 'foo';
    const val = 5;
    e = e.set(key, { binding: val });
  });

  it('error', () => {
    const [_env, _form, error] = evalForm(e, '(+ 1 "a")');
    expect(error).to.equal('all arguments to + should be numbers');
  });

  it('+', () => {
    let [_, res] = evalForm(e, '(+ 2 4)');
    expect(res).to.be.closeTo(6, epsilon);

    [_, res] = evalForm(e, '(+ 2 foo)');
    expect(res).to.be.closeTo(7, epsilon);
  });

  it('-', () => {
    let [_, res] = evalForm(e, '(- 10 3)');
    expect(res).to.be.closeTo(7, epsilon);

    [_, res] = evalForm(e, '(- 10 3 5)');
    expect(res).to.be.closeTo(2, epsilon);

    [_, res] = evalForm(e, '(- 42)');
    expect(res).to.be.closeTo(-42, epsilon);
  });

  it('*', () => {
    const [_, res] = evalForm(e, '(* 2 4)');
    expect(res).to.be.closeTo(8, epsilon);
  });

  it('/', () => {
    let [_, res] = evalForm(e, '(/ 90 10)');
    expect(res).to.be.closeTo(9, epsilon);

    [_, res] = evalForm(e, '(/ 90 10 3)');
    expect(res).to.be.closeTo(3, epsilon);
  });

  it('combined mathematical functions', () => {
    const [_, res] = evalForm(e, '(+ (* 2 2) (* 3 3))');
    expect(res).to.be.closeTo(13, epsilon);
  });

  it('mod', () => {
    const [_, res] = evalForm(e, '(mod 10 3)');
    expect(res).to.equal(1);
  });

  it('sqrt', () => {
    const [_, res] = evalForm(e, '(sqrt 81)');
    expect(res).to.equal(9);
  });

  it('=', () => {
    let [_, res] = evalForm(e, '(= 90 90)');
    expect(res).to.equal('#t');

    [_, res] = evalForm(e, '(= 90 3)');
    expect(res).to.equal('#f');

    [_, res] = evalForm(e, '(= 90 90 90)');
    expect(res).to.equal('#t');
  });

  it('>', () => {
    let [_, res] = evalForm(e, '(> 54 30)');
    expect(res).to.equal('#t');

    [_, res] = evalForm(e, '(> 54 540)');
    expect(res).to.equal('#f');

    [_, res] = evalForm(e, '(> 54 54)');
    expect(res).to.equal('#f');

    [_, res] = evalForm(e, '(> 54 30 20)');
    expect(res).to.equal('#t');
  });

  it('<', () => {
    let [_, res] = evalForm(e, '(< 54 30)');
    expect(res).to.equal('#f');

    [_, res] = evalForm(e, '(< 54 540)');
    expect(res).to.equal('#t');

    [_, res] = evalForm(e, '(< 54 54)');
    expect(res).to.equal('#f');

    [_, res] = evalForm(e, '(< 54 62 72)');
    expect(res).to.equal('#t');
  });

  it('vector', () => {
    const [newEnv, res] = evalForm(e, '(vector 90 90)');
    expect(res).to.deep.equal([90, 90]);
    expect(newEnv).to.equal(e);
  });

  it('vector/append', () => {
    let [_, res] = evalForm(e, '(vector/append (vector 10 20) 30)');
    expect(res).to.deep.equal([10, 20, 30]);

    [_, res] = evalForm(e, '(vector/append (vector 10 20) 30 40 50 60)');
    expect(res).to.deep.equal([10, 20, 30, 40, 50, 60]);
  });
});
