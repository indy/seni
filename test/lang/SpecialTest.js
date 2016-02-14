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

import { expect } from 'chai';

import { buildEnv, evalForm } from './eval_helper';

describe('Special', () => {

  let e;
  let key;
  let val;
  const epsilon = 0.01;

  beforeEach(() => {
    e = buildEnv();
  });

  it('if', () => {
    let [newEnv, res] = evalForm(e, '(if true 2 4)');
    expect(res).to.equal(2);
    expect(newEnv).to.equal(e);

    [newEnv, res] = evalForm(e, '(if false 2 4)');
    expect(res).to.equal(4);
    expect(newEnv).to.equal(e);
  });

  it('quote', () => {
    let [newEnv, res] = evalForm(e, '(quote something)');
    expect(res).to.equal('something');
    expect(newEnv).to.equal(e);

    [newEnv, res] = evalForm(e, '(quote (+ 4 2))');
    expect(res).to.deep.equal(['+', 4, 2]);
    expect(newEnv).to.equal(e);
  });

  it('fn', () => {
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

  it('define', () => {
    const [newEnv, res] = evalForm(e, '(define monkey 42)');
    expect(newEnv.has('monkey')).to.equal(true);
    expect(newEnv.get('monkey').binding).to.equal(42);
    // define should also evaluate to it's set values
    expect(res).to.equal(42);
  });

  it('defining multiple values', () => {
    const [newEnv, res] = evalForm(e, '(define monkey 42 ape (+ 6 6))');
    expect(newEnv.has('monkey')).to.equal(true);
    expect(newEnv.get('monkey').binding).to.equal(42);
    expect(newEnv.has('ape')).to.equal(true);
    expect(newEnv.get('ape').binding).to.equal(12);
    expect(res).to.equal(12);
  });

  it('defining destructured values', () => {
    const [newEnv, res] = evalForm(e, '(define [a b] [2 (+ 3 4)])');
    expect(newEnv.has('a')).to.equal(true);
    expect(newEnv.get('a').binding).to.equal(2);
    expect(newEnv.has('b')).to.equal(true);
    expect(newEnv.get('b').binding).to.equal(7);
    expect(res).to.equal(7);
  });

  it('begin', () => {
    let [_, res] = evalForm(e, '(begin (+ 1 1) (+ 2 2))');
    expect(res).to.equal(4);

    [_, res] = evalForm(e, '(begin (+ 1 1))');
    expect(res).to.equal(2);
  });

  it('loop: from/to', () => {
    let [env, res] = evalForm(e,`
(define bar (list))
(loop (a from: 0 to: 4 increment: 1) (append bar a))`);

    const bar = env.get('bar');
    expect(bar.binding).to.deep.equal([0, 1, 2, 3]);
  });

  it('loop: from/to high to low', () => {
    let [env, res] = evalForm(e,`
(define bar (list))
(loop (a from: 4 to: 0 increment: -1) (append bar a))`);

    const bar = env.get('bar');
    expect(bar.binding).to.deep.equal([4, 3, 2, 1]);
  });

  it('loop: from/to high to low, positive increment', () => {
    let [env, res] = evalForm(e,`
(define bar (list))
(loop (a from: 4 to: 0 increment: 1) (append bar a))`);

    const bar = env.get('bar');
    expect(bar.binding).to.deep.equal([4, 3, 2, 1]);
  });

  it('loop: from/upto', () => {
    let [env, res] = evalForm(e,`
(define bar (list))
(loop (a from: 0 upto: 4 increment: 1) (append bar a))`);

    const bar = env.get('bar');
    expect(bar.binding).to.deep.equal([0, 1, 2, 3, 4]);
  });

  it('loop: from/upto high to low', () => {
    let [env, res] = evalForm(e,`
(define bar (list))
(loop (a from: 4 upto: 0 increment: -1) (append bar a))`);

    const bar = env.get('bar');
    expect(bar.binding).to.deep.equal([4, 3, 2, 1, 0]);
  });

  it('loop: from/upto high to low, positive increment', () => {
    let [env, res] = evalForm(e,`
(define bar (list))
(loop (a from: 4 upto: 0 increment: 1) (append bar a))`);

    const bar = env.get('bar');
    expect(bar.binding).to.deep.equal([4, 3, 2, 1, 0]);
  });

  it('loop: to increment', () => {
    let [env, res] = evalForm(e,`
(define bar (list))
(loop (a from: 0 to: 12 increment: 2) (append bar a))`);

    const bar = env.get('bar');
    expect(bar.binding).to.deep.equal([0, 2, 4, 6, 8, 10]);
  });

  it('loop: upto increment', () => {
    let [env, res] = evalForm(e,`
(define bar (list))
(loop (a from: 0 upto: 12 increment: 2) (append bar a))`);

    const bar = env.get('bar');
    expect(bar.binding).to.deep.equal([0, 2, 4, 6, 8, 10, 12]);
  });

  it('loop: from/to steps', () => {
    let [env, res] = evalForm(e,`
(define bar (list))
(loop (a from: 0 to: 10 steps: 3) (append bar a))`);

    const bar = env.get('bar');
    expect(bar.binding[0]).to.be.closeTo(0.00, epsilon);
    expect(bar.binding[1]).to.be.closeTo(3.333, epsilon);
    expect(bar.binding[2]).to.be.closeTo(6.666, epsilon);
  });

  it('loop: from/to steps high to low', () => {
    let [env, res] = evalForm(e,`
(define bar (list))
(loop (a from: 10 to: 0 steps: 3) (append bar a))`);

    const bar = env.get('bar');
    expect(bar.binding[0]).to.be.closeTo(10.000, epsilon);
    expect(bar.binding[1]).to.be.closeTo(6.666, epsilon);
    expect(bar.binding[2]).to.be.closeTo(3.333, epsilon);
  });

  it('loop: from/upto steps', () => {
    let [env, res] = evalForm(e,`

      (define bar (list))
      (loop (a from: 0 upto: 10 steps: 3) (append bar a))

    `);

    const bar = env.get('bar');
    expect(bar.binding[0]).to.be.closeTo(0.00, epsilon);
    expect(bar.binding[1]).to.be.closeTo(5.0, epsilon);
    expect(bar.binding[2]).to.be.closeTo(10.0, epsilon);
  });

  it('loop: from/upto steps high to low', () => {
    let [env, res] = evalForm(e,`

      (define bar (list))
      (loop (a from: 10 upto: 0 steps: 3) (append bar a))

    `);

    const bar = env.get('bar');
    expect(bar.binding[0]).to.be.closeTo(10.0, epsilon);
    expect(bar.binding[1]).to.be.closeTo(5.0, epsilon);
    expect(bar.binding[2]).to.be.closeTo(0.00, epsilon);
  });



  //   it('loop: negative increment', () => {
  //     let [env, res] = evalForm(e,`
  // (define bar (list))
  // (loop (a from: 12 to: 0 increment: -2) (append bar a))`);
  //
  //     const bar = env.get('bar');
  //     expect(bar.binding).to.deep.equal([12, 10, 8, 6, 4, 2]);
  //   });

});
