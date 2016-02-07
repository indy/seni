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

function interpolate(a, b, t) {
  return (a * (1 - t)) + (b * t);
}


function makeBezierFn(coords) {
  return parameters => {
    const t = parameters.t || 0;
    const x = MathUtil.bezierPoint(coords[0][0],
                                   coords[1][0],
                                   coords[2][0],
                                   coords[3][0],
                                   t);
    const y = MathUtil.bezierPoint(coords[0][1],
                                   coords[1][1],
                                   coords[2][1],
                                   coords[3][1],
                                   t);
    return [x, y];
  };
}

function makeBezierTangentFn(coords) {
  return parameters => {
    const t = parameters.t || 0;
    const x = MathUtil.bezierTangent(coords[0][0],
                                     coords[1][0],
                                     coords[2][0],
                                     coords[3][0],
                                     t);
    const y = MathUtil.bezierTangent(coords[0][1],
                                     coords[1][1],
                                     coords[2][1],
                                     coords[3][1],
                                     t);
    return [x, y];
  };
}

const defaultCoords = [[440, 400],
                       [533, 700],
                       [766, 200],
                       [900, 500]];


const publicBindings = [
  new PublicBinding(
    'interp/fn',
    {
      description: 'a function for remapping values',
      args: [['from', '[0 1]'],
             ['to', '[0 100]'],
             ['clamping', 'false'],
             ['mapping', `one of 'linear', 'quick', 'slow-in', 'slow-in-out'`]],
      returns: `a function which accepts a 'val' argument`
    },
    {},
    () => remapFn
  ),

  new PublicBinding(
    'interp/cos',
    {
      description: 'calculate cosine value of t',
      args: [['amplitude', '1'],
             ['frequency', '1'],
             ['t', '1 (note: t goes from 0 to math/TAU)']],
      returns: 'the cosine'
    },
    {
      amplitude: 1,
      frequency: 1,
      t: 1
    },
    self => params => {
      const {amplitude, frequency, t} = self.mergeWithDefaults(params);
      // make a cosine fn and then invoke it with the t value
      return amplitude * Math.cos(t * frequency);
    }
  ),

  new PublicBinding(
    'interp/sin',
    {
      description: 'calculate sine value of t',
      args: [['amplitude', '1'],
             ['frequency', '1'],
             ['t', '1 (note: t goes from 0 to math/TAU)']],
      returns: 'the sin'
    },
    {
      amplitude: 1,
      frequency: 1,
      t: 1
    },
    self => params => {
      const {amplitude, frequency, t} = self.mergeWithDefaults(params);
      // make a sin fn and then invoke it with the t value
      return amplitude * Math.sin(t * frequency);
    }
  ),

  new PublicBinding(
    'interp/bezier',
    /* eslint-disable max-len */
    {
      description: 'interpolates across a Bezier curve',
      args: [['coords', 'four vectors representing control points on a Bezier curve'],
             ['t', 'the t value along the curve']],
      returns: 'a point on a Bezier curve'
    },
    /* eslint-enable max-len */
    {
      coords: defaultCoords,
      t: 1
    },
    self => params => {
      const {coords, t} = self.mergeWithDefaults(params);
      // make a Bezier fn and then invoke it straight away with the t value
      return makeBezierFn(coords)({t});
    }
  ),

  new PublicBinding(
    'interp/bezier-fn',
    /* eslint-disable max-len */
    {
      description: 'creates a function which calculates points on a Bezier curve',
      args: [['coords', 'four vectors representing control points on a Bezier curve']],
      returns: `returns a function which, when given 't' returns the point on the curve`
    },
    /* eslint-enable max-len */
    {
      coords: defaultCoords
    },
    self => params => {
      const {coords} = self.mergeWithDefaults(params);
      return makeBezierFn(coords);
    }
  ),

  new PublicBinding(
    'interp/bezier-tangent',
    /* eslint-disable max-len */
    {
      description: 'calculate the tangent vector for a point on the Bezier curve',
      args: [['coords', 'four vectors representing control points on a Bezier curve'],
             ['t', 'the t value along the curve']],
      returns: 'a tangent vector on the Bezier curve'
    },
    /* eslint-enable max-len */
    {
      coords: defaultCoords,
      t: 1
    },
    self => params => {
      const {coords, t} = self.mergeWithDefaults(params);
      return makeBezierTangentFn(coords)({t});
    }
  ),

  new PublicBinding(
    'interp/bezier-tangent-fn',
    /* eslint-disable max-len */
    {
      description: 'create a function calculates tangents for a point on the Bezier curve',
      args: [['coords', 'four vectors representing control points on a Bezier curve']],
      returns: `returns a function which, given 't' returns the tangent for the curve`
    },
    /* eslint-enable max-len */
    {
      coords: defaultCoords
    },
    self => params => {
      const {coords} = self.mergeWithDefaults(params);
      return makeBezierTangentFn(coords);
    }
  ),

  new PublicBinding(
    'interp/circle',
    {
      description: 'calculate a point on a circle',
      args: [['position', 'vector for the position of the circle'],
             ['radius', 'radius of the circle'],
             ['t', 'parametric value along the circle']],
      returns: 'a point on a circle'
    },
    {
      position: [0, 0],
      radius: 1,
      t: 0
    },
    self => params => {
      const {position, radius, t} = self.mergeWithDefaults(params);

      const [x, y] = position;

      const angle = t * MathUtil.TAU;

      const vx = (Math.sin(angle) * radius) + x;
      const vy = (Math.cos(angle) * radius) + y;

      return [vx, vy];
    }
  )
];

export default {
  publicBindings,
  interpolate,
  remapFn
};
