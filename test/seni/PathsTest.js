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

import Paths from '../../src/seni/Paths';

import chai from 'chai';
const expect = chai.expect;
/* eslint-disable no-debugger */
describe('Paths', () => {

  function getPublicBinding(namespace, name) {
    for (let i = 0; i < namespace.publicBindings.length; i++) {
      let binding = namespace.publicBindings[i];
      if (binding.name === name) {
        return binding;
      }
    }
    return undefined;
  }

  beforeEach(() => {
  });

  it('should invoke callback with linear values', () => {

    let count = 0;
    let positions = [];

    let f = function(p) {
      count++;
      positions.push(p.position);
    };

    let linearBinding = getPublicBinding(Paths, 'path/linear');
    let linear = linearBinding.create(linearBinding);

    let params = {
      coords: [[0, 0], [100, 100]],
      fn: f,
      steps: 10
    };

    // invoke the path/linear
    linear(params);

    expect(count, 'callback invoked params.steps times').to.equal(params.steps);

    expect(positions.length).to.equal(params.steps);

    expect(positions[0][0]).to.be.closeTo(0, 1);
    expect(positions[0][1]).to.be.closeTo(0, 1);
    expect(positions[1][0]).to.be.closeTo(11.1, 1);
    expect(positions[1][1]).to.be.closeTo(11.1, 1);
    expect(positions[2][0]).to.be.closeTo(22.2, 1);
    expect(positions[2][1]).to.be.closeTo(22.2, 1);
    expect(positions[3][0]).to.be.closeTo(33.3, 1);
    expect(positions[3][1]).to.be.closeTo(33.3, 1);
    expect(positions[4][0]).to.be.closeTo(44.4, 1);
    expect(positions[4][1]).to.be.closeTo(44.4, 1);
    expect(positions[5][0]).to.be.closeTo(55.5, 1);
    expect(positions[5][1]).to.be.closeTo(55.5, 1);
    expect(positions[6][0]).to.be.closeTo(66.6, 1);
    expect(positions[6][1]).to.be.closeTo(66.6, 1);
    expect(positions[7][0]).to.be.closeTo(77.7, 1);
    expect(positions[7][1]).to.be.closeTo(77.7, 1);
    expect(positions[8][0]).to.be.closeTo(88.8, 1);
    expect(positions[8][1]).to.be.closeTo(88.8, 1);
    expect(positions[9][0]).to.be.closeTo(99.9, 1);
    expect(positions[9][1]).to.be.closeTo(99.9, 1);
  });
});
