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

import PublicBinding from '../lang/PublicBinding';

const PI = Math.PI;
const twoPI = PI * 2;
const PIbyTwo = PI / 2;

function mc([xa, ya], [xb, yb]) {
  const m = (ya - yb) / (xa - xb);
  const c = ya - (m * xa);
  return [m, c];
}

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
  const s = Math.sin(x * PIbyTwo);
  return s * s * s * s;
}

function mapSlowEaseInEaseOut(x) {
  return x - (Math.sin(x * twoPI) / twoPI);
}

const mappingLookup = new Immutable.Map({'linear': mapLinear,
                                         'quick': mapQuickEase,
                                         'slow-in': mapSlowEaseIn,
                                         'slow-in-out': mapSlowEaseInEaseOut});

function remapFn(params) {

  const from = params.from || [0, 1];
  const to = params.to || [0, 100];
  const clamping = params.clamping || false;
  const mapping = params.mapping || 'linear';

  const [fromA, fromB] = from,
        [toA, toB] = to,
        [fromM, fromC] = mc([fromA, 0], [fromB, 1]),
        [toM, toC] = mc([0, toA], [1, toB]);

  let normalisedMappingFn = mappingLookup.get(mapping);

  if (normalisedMappingFn === undefined) {
    normalisedMappingFn = mappingLookup.get('linear');
  }

  return function(paramaters) {
    const val = paramaters.val || 0;
    const fromInterp = (fromM * val) + fromC,
          toInterp = normalisedMappingFn(fromInterp),
          res = (toM * toInterp) + toC;
    if (clamping) {
      return fromInterp < 0 ? toA : (fromInterp > 1) ? toB : res;
    } else {
      return res;
    }
  };
}

const MathUtil = {

  remapFn,

  PI: new PublicBinding(
    'PI',
    ``,
    {},
    () => PI
  ),

  twoPI: new PublicBinding(
    '2PI',
    ``,
    {},
    () => twoPI
  ),

  PIbyTwo: new PublicBinding(
    'PI/2',
    ``,
    {},
    () => PIbyTwo
  ),

  sin: new PublicBinding(
    'sin',
    ``,
    {angle: 0},
    (self) => function(params) {
      const {angle} = self.mergeWithDefaults(params);
      return Math.sin(angle);
    }
  ),

  cos: new PublicBinding(
    'cos',
    ``,
    {angle: 0},
    (self) => function(params) {
      const {angle} = self.mergeWithDefaults(params);
      return Math.cos(angle);
    }
  ),

  remapFnBinding: new PublicBinding(
    'remapFn',
    ``,
    {},
    () => remapFn
  ),

  distance2D: new PublicBinding(
    'math/distance2D',
    ``,
    {aX: 0, aY: 0, bX: 1, bY: 1},
    (self) => function(params) {
      const {aX, aY, bX, bY} = self.mergeWithDefaults(params);

      const xd = aX - bX;
      const yd = aY - bY;
      return Math.sqrt((xd * xd) + (yd * yd));
    }
  ),

  clamp: new PublicBinding(
    'math/clamp',
    ``,
    {val: 0, min: 0, max: 1},
    (self) => function(params) {
      const {val, min, max} = self.mergeWithDefaults(params);
      return val < min ? min : (val > max ? max : val);
    }
  ),

  stepsInclusive: function(start, end, num) {
    const unit = (end - start) / (num - 1);
    const res = [];
    for (let i = 0; i < num; i++) {
      res.push(start + (i * unit));
    }
    return res;
  },

  distance1d: function(a, b) {
    return Math.abs(a - b);
  },
  /*
   clamp: function(a, min, max) {
   return a < min ? min : (a > max ? max : a);
   },

   distance2d: function([xa, ya], [xb, yb]) {
   const xd = xa - xb;
   const yd = ya - yb;
   return Math.sqrt((xd * xd) + (yd * yd));
   },
   */

  interpolate: function(a, b, t) {
    return (a * (1 - t)) + (b * t);
  },

  normalize: function(x, y) {
    const len = Math.sqrt((x * x) + (y * y));
    return [(x / len), (y / len)];
  },

  mc: function([xa, ya], [xb, yb]) {
    const m = (ya - yb) / (xa - xb);
    const c = ya - (m * xa);
    return [m, c];
  }
};

export default MathUtil;
