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

import PublicBinding from './PublicBinding';
import Immutable from 'immutable';

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

const remappingFn = new Immutable.Map({'linear': mapLinear,
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

  let normalisedMappingFn = remappingFn.get(mapping);

  if (normalisedMappingFn === undefined) {
    normalisedMappingFn = remappingFn.get('linear');
  }

  return function(parameters) {
    const val = parameters.val || 0;
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

function normalize(x, y) {
  const len = Math.sqrt((x * x) + (y * y));
  return [(x / len), (y / len)];
}

function normals(x1, y1, x2, y2) {
  const dx = x2 - x1;
  const dy = y2 - y1;

  return [normalize(-dy, dx), normalize(dy, -dx)];
}

function bezierPoint(a, b, c, d, t) {
  const t1 = 1 - t;

  return (a * t1 * t1 * t1) +
    (3 * b * t * t1 * t1) +
    (3 * c * t * t * t1) +
    (d * t * t * t);
}

function quadraticPoint(a, b, c, t) {
  const r = ((b - a) - 0.5 * (c - a)) / (0.5 * (0.5 - 1));
  const s = c - a - r;

  return (r * t * t) + (s * t) + a;
}

function bezierCoordinates(tVals, controlPoints) {
  const xs = tVals.map(t => bezierPoint(controlPoints[0][0],
                                        controlPoints[1][0],
                                        controlPoints[2][0],
                                        controlPoints[3][0], t));
  const ys = tVals.map(t => bezierPoint(controlPoints[0][1],
                                        controlPoints[1][1],
                                        controlPoints[2][1],
                                        controlPoints[3][1], t));

  return {xs, ys};
}

function quadraticCoordinates(tVals, controlPoints) {
  const xs = tVals.map(t => quadraticPoint(controlPoints[0][0],
                                           controlPoints[1][0],
                                           controlPoints[2][0], t));
  const ys = tVals.map(t => quadraticPoint(controlPoints[0][1],
                                           controlPoints[1][1],
                                           controlPoints[2][1], t));

  return {xs, ys};
}

function distance2d([x1, y1], [x2, y2]) {
  const xd = x1 - x2;
  const yd = y1 - y2;
  return Math.sqrt((xd * xd) + (yd * yd));
}

function clamp(a, min, max) {
  return a < min ? min : (a > max ? max : a);
}

const MathUtil = {

  publicBindings: [
    new PublicBinding(
      'PI',
      ``,
      {},
      () => PI
    ),

    new PublicBinding(
      'PIPI',
      ``,
      {},
      () => twoPI
    ),

    new PublicBinding(
      'PI/2',
      ``,
      {},
      () => PIbyTwo
    ),

    new PublicBinding(
      'sin',
      ``,
      {angle: 0},
      (self) => function(params) {
        const {angle} = self.mergeWithDefaults(params);
        return Math.sin(angle);
      }
    ),

    new PublicBinding(
      'cos',
      ``,
      {angle: 0},
      (self) => function(params) {
        const {angle} = self.mergeWithDefaults(params);
        return Math.cos(angle);
      }
    ),

    new PublicBinding(
      'remap-fn',
      ``,
      {},
      () => remapFn
    ),

    new PublicBinding(
      'math/distance-2d',
      ``,
      {x1: 0, y1: 0, x2: 1, y2: 1},
      (self) => function(params) {
        const {x1, y1, x2, y2} = self.mergeWithDefaults(params);
        return distance2d([x1, y1], [x2, y2]);
      }
    ),

    new PublicBinding(
      'math/clamp',
      ``,
      {val: 0, min: 0, max: 1},
      (self) => function(params) {
        const {val, min, max} = self.mergeWithDefaults(params);
        return clamp(val, min, max);
      }
    )
  ],

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

  interpolate: function(a, b, t) {
    return (a * (1 - t)) + (b * t);
  },

  mc: function([xa, ya], [xb, yb]) {
    const m = (ya - yb) / (xa - xb);
    const c = ya - (m * xa);
    return [m, c];
  },

  distance2d,

  remapFn,
  clamp,

  normalize,
  normals,
  bezierPoint,
  quadraticPoint,
  bezierCoordinates,
  quadraticCoordinates,

  mapLinear,
  mapQuickEase,
  mapSlowEaseIn,
  mapSlowEaseInEaseOut
};

export default MathUtil;
