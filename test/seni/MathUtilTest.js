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

import MathUtil from '../../app/js/seni/MathUtil';

import {expect} from 'chai';

describe(`MathUtil`, () => {

  const epsilon = 0.01;

  it(`stepsInclusive`, () => {
    const expected = [0.0, 0.25, 0.50, 0.75, 1.0];
    const res = MathUtil.stepsInclusive(0, 1, 5);

    expect(res.length).to.equal(5);
    for (let i = 0; i < 5; i++) {
      expect(res[i]).to.equal(expected[i]);
    }
  });

  /*
   it(`clamp`, () => {
   expect(MathUtil.clamp(5, 0, 10)).to.equal(5);
   expect(MathUtil.clamp(5, 7, 10)).to.equal(7);
   expect(MathUtil.clamp(5, 0, 4)).to.equal(4);
   });
   */

  it(`normalize`, () => {
    expect(MathUtil.normalize(32, 0)).to.eql([1, 0]);

    const res = MathUtil.normalize(81, 81);
    expect(res[0]).to.be.closeTo(0.707106, epsilon);
    expect(res[1]).to.be.closeTo(0.707106, epsilon);
  });
});
