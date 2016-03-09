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

describe('HigherOrder', () => {

  let e;
  const epsilon = 0.01;

  beforeEach(() => {
    e = buildEnv();

    const key = 'foo';
    const val = 5;
    e = e.set(key, { binding: val });
  });


  it('map', () => {
    let [_, res] = evalForm(e, `(map
fn: (fn (triple x: 1)
        (+ x x x))
vector: [1 2 3 4]
bind: 'x)`);
    expect(res).to.deep.equal([3, 6, 9, 12]);
  });

  it('filter', () => {
    let [_, res] = evalForm(e, `(filter
fn: (fn (more-than-4 count: 1)
        (> count 4))
vector: [2 3 4 5 6 7]
bind: 'count)`);
    expect(res).to.deep.equal([5, 6, 7]);
  });

});
