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

import Interp from '../../src/seni/Interp';

import chai from 'chai';
const expect = chai.expect;

describe('Interp', () => {

  const epsilon = 0.01;

  it('remapFn', () => {
    let res = Interp.remapFn({from: [0, 1], to: [0, 100], clamping: false});
    expect(res({val: 0})).to.be.closeTo(0, epsilon);
    expect(res({val: 1})).to.be.closeTo(100, epsilon);
    expect(res({val: 0.4})).to.be.closeTo(40, epsilon);

    res = Interp.remapFn({from: [1, 0], to: [0, 100], clamping: true});
    expect(res({val: 0})).to.be.closeTo(100, epsilon);
    expect(res({val: 1})).to.be.closeTo(0, epsilon);
    expect(res({val: 0.4})).to.be.closeTo(60, epsilon);
    expect(res({val: 2})).to.be.closeTo(0, epsilon);
    expect(res({val: -7})).to.be.closeTo(100, epsilon);
  });
});
