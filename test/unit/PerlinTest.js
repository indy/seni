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

import Perlin from '../../src/seni/Perlin';

import chai from 'chai';
const expect = chai.expect;

describe('Perlin', () => {

  it('should output a number', () => {
    for (let i = 0; i < 1000; i++) {
      let binding = Perlin.perlin;
      let v = binding.create(binding)({});
      expect(v).to.be.at.least(0.0);
      expect(v).to.be.at.most(1.0);
    }
  });

  it('should output the same number given the same arguments', () => {
    let binding = Perlin.perlin;
    let v = binding.create(binding)({x: 0.1, y: 0.3, z: 0.5});
    let w = binding.create(binding)({x: 0.1, y: 0.3, z: 0.5});
    expect(v).to.be.closeTo(w, 3);
  });
});
