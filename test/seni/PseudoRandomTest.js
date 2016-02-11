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

import PseudoRandom from '../../app/js/seni/PseudoRandom';

import {expect} from 'chai';

describe('PseudoRandom', () => {

  const perlinSignedIndex = 1;

  it('Perlin: should output a number', () => {
    for (let i = 0; i < 1000; i++) {
      const binding = PseudoRandom.publicBindings[perlinSignedIndex];
      const v = binding.create(binding)({});
      expect(v).to.be.at.least(0.0);
      expect(v).to.be.at.most(1.0);
    }
  });

  it('Perlin: should output the same number given the same arguments', () => {
    const binding = PseudoRandom.publicBindings[perlinSignedIndex];
    const v = binding.create(binding)({x: 0.1, y: 0.3, z: 0.5});
    const w = binding.create(binding)({x: 0.1, y: 0.3, z: 0.5});
    expect(v).to.be.closeTo(w, 3);
  });

  it('should have replicable number generation', () => {
    const epsilon = 0.0001;

    const aa = PseudoRandom.buildUnsigned('hello.');
    expect(aa()).to.be.closeTo(0.9282578795792454, epsilon);
    expect(aa()).to.be.closeTo(0.3752569768646784, epsilon);

    const bb = PseudoRandom.buildUnsigned('hello.');
    expect(bb()).to.be.closeTo(0.9282578795792454, epsilon);
    expect(bb()).to.be.closeTo(0.3752569768646784, epsilon);
  });
});
