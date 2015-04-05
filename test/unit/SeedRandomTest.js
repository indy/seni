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

import SeedRandom from '../../src/seni/SeedRandom';

describe('SeedRandom', () => {

  it('should have replicable number generation', () => {
    let epsilon = 0.0001;

    let aa = SeedRandom.buildUnsigned('hello.');
    expect(aa()).to.be.closeTo(0.9282578795792454, epsilon);
    expect(aa()).to.be.closeTo(0.3752569768646784, epsilon);

    let bb = SeedRandom.buildUnsigned('hello.');
    expect(bb()).to.be.closeTo(0.9282578795792454, epsilon);
    expect(bb()).to.be.closeTo(0.3752569768646784, epsilon);
  });
});
