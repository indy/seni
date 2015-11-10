/*
 *  Seni
 *  Copyright (C) 2015  Inderjit Gill <email@indy.io>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

import PublicBinding from './PublicBinding';
import Immutable from 'immutable';
import MathUtil from './MathUtil';


// the following map* functions work in the range 0..1:

function mapLinear(x) {
  return x;
}

function mapQuickEase(x) {
  const x2 = x * x;
  const x3 = x * x * x;
  return (3 * x2) - (2 * x3);
}

function mapSlowEaseIn(x) {
  const s = Math.sin(x * MathUtil.PIbyTwo);
  return s * s * s * s;
}

function mapSlowEaseInEaseOut(x) {
  return x - (Math.sin(x * MathUtil.TAU) / MathUtil.TAU);
}

const remappingFn = new Immutable.Map({'linear': mapLinear,
                                       'quick': mapQuickEase,
                                       'slow-in': mapSlowEaseIn,
                                       'slow-in-out': mapSlowEaseInEaseOut});

function remapFn(params) {

  const from = params.from || [0, 1];
  const to = params.to || [0, 100];
  const clamping = params.clamping || false;
  const mapping = params.mapping || 'linear';

  const [fromA, fromB] = from;
  const [toA, toB] = to;
  const [fromM, fromC] = MathUtil.mc([fromA, 0], [fromB, 1]);
  const [toM, toC] = MathUtil.mc([0, toA], [1, toB]);

  let normalisedMappingFn = remappingFn.get(mapping);

  if (normalisedMappingFn === undefined) {
    normalisedMappingFn = remappingFn.get('linear');
  }

  return function(parameters) {
    const val = parameters.val || 0;
    const fromInterp = (fromM * val) + fromC;
    const toInterp = normalisedMappingFn(fromInterp);
    const res = (toM * toInterp) + toC;

    if (clamping) {
      return fromInterp < 0 ? toA : (fromInterp > 1) ? toB : res;
    } else {
      return res;
    }
  };
}


const Interp = {
  publicBindings: [
    new PublicBinding(
      'interp/fn',
      ``,
      {},
      () => remapFn
    ),

    new PublicBinding(
      'interp/bezier',
      ``,
      {coords: [[440, 400],
                [533, 700],
                [766, 200],
                [900, 500]],
       t: 1},
      (self) => function(params) {
        const {coords, t} = self.mergeWithDefaults(params);
        let x = MathUtil.bezierPoint(coords[0][0],
                                     coords[1][0],
                                     coords[2][0],
                                     coords[3][0],
                                     t);
        let y = MathUtil.bezierPoint(coords[0][1],
                                     coords[1][1],
                                     coords[2][1],
                                     coords[3][1],
                                     t);
        return [x, y];
      }
    ),

    new PublicBinding(
      'interp/bezier-tangent',
      ``,
      {coords: [[440, 400],
                [533, 700],
                [766, 200],
                [900, 500]],
       t: 1},
      (self) => function(params) {
        const {coords, t} = self.mergeWithDefaults(params);
        let x = MathUtil.bezierTangent(coords[0][0],
                                       coords[1][0],
                                       coords[2][0],
                                       coords[3][0],
                                       t);
        let y = MathUtil.bezierTangent(coords[0][1],
                                       coords[1][1],
                                       coords[2][1],
                                       coords[3][1],
                                       t);
        return [x, y];
      }
    ),

  ],

  interpolate: function(a, b, t) {
    return (a * (1 - t)) + (b * t);
  },

  remapFn
};

export default Interp;
