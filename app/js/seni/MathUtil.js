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

import PublicBinding from '../lang/PublicBinding';

const PI = Math.PI;
const TAU = PI * 2;
const PIbyTwo = PI / 2;

const degToRad = TAU / 360;
const radToDeg = 360 / TAU;

function degreesToRadians(angle) {
  return (angle % 360) * degToRad;
}

function radiansToDegrees(angle) {
  return (angle % TAU) * radToDeg;
}

function mc([xa, ya], [xb, yb]) {
  const m = (ya - yb) / (xa - xb);
  const c = ya - (m * xa);
  return [m, c];
}

function normalize(x, y) {
  const len = Math.sqrt((x * x) + (y * y));
  return [(x / len), (y / len)];
}

function normals(x1, y1, x2, y2) {
  const dx = x2 - x1;
  const dy = y2 - y1;

  const [nx, ny] = normalize(-dy, dx);
  return [[nx, ny], [-nx, -ny]];
}

function quadraticPoint(a, b, c, t) {
  const r = ((b - a) - 0.5 * (c - a)) / (0.5 * (0.5 - 1));
  const s = c - a - r;

  return (r * t * t) + (s * t) + a;
}

function bezierPoint(a, b, c, d, t) {
  const t1 = 1 - t;

  return (a * t1 * t1 * t1) +
    (3 * b * t * t1 * t1) +
    (3 * c * t * t * t1) +
    (d * t * t * t);
}

function bezierTangent(a, b, c, d, t) {
  return (3 * t * t * (-a + 3 * b - 3 * c + d) +
          6 * t * (a - 2 * b + c) +
          3 * (-a + b));
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

const publicBindings = [
  new PublicBinding(
    'math/PI',
    {},
    {},
    () => PI
  ),

  new PublicBinding(
    'math/TAU',
    {},
    {},
    () => TAU
  ),

  new PublicBinding(
    'math/sin',
    {},
    {angle: 0},
    self => params => {
      const {angle} = self.mergeWithDefaults(params);
      return Math.sin(angle);
    }
  ),

  new PublicBinding(
    'math/cos',
    {},
    {angle: 0},
    self => params => {
      const {angle} = self.mergeWithDefaults(params);
      return Math.cos(angle);
    }
  ),
  new PublicBinding(
    'math/atan2',
    { description: `Calculates the arc tangent of the two variables y and x.
It is similar to calculating the arc tangent of y / x, except that the signs of
both arguments are used to determine the quadrant of the result`,
      args: [['x', ''],
             ['y', '']] },
    { x: 0, y: 0 },
    self => params => {
      const {x, y} = self.mergeWithDefaults(params);
      return Math.atan2(y, x); // this is correct, y is given before x
    }
  ),

  // todo: make this work with vectors of any dimension
  new PublicBinding(
    'math/distance',
    { description: 'returns the distance between 2 vectors',
      args: [['vec1', '[0 0]'],
             ['vec2', '[100 100]']] },
    { vec1: [0, 0], vec2: [100, 100] },
    self => params => {
      const {vec1, vec2} = self.mergeWithDefaults(params);
      return distance2d(vec1, vec2);
    }
  ),

  new PublicBinding(
    'math/clamp',
    { description: 'clamps a value between min and max',
      args: [['val', '0'],
             ['min', ''],
             ['max', '']] },
    { val: 0, min: 0, max: 1 },
    self => params => {
      const {val, min, max} = self.mergeWithDefaults(params);
      return clamp(val, min, max);
    }
  ),

  new PublicBinding(
    'degrees->radians',
    { description:
      'A helper function that converts angles in degrees to radians',
      args: [['angle', '0.0']] },
    { angle: 0.0 },
    self => params => {
      const {angle} = self.mergeWithDefaults(params);
      return degreesToRadians(angle);
    }
  ),

  new PublicBinding(
    'radians->degrees',
    { description:
      'A helper function that converts angles in radians to degrees',
      args: [['angle', '0.0']] },
    { angle: 0.0 },
    self => params => {
      const {angle} = self.mergeWithDefaults(params);
      return radiansToDegrees(angle);
    }
  )
];

export default {
  publicBindingType: 'binding',
  publicBindings,

  stepsInclusive: (start, end, num) => {
    const unit = (end - start) / (num - 1);
    const res = [];
    for (let i = 0; i < num; i++) {
      res.push(start + (i * unit));
    }
    return res;
  },

  distance1d: (a, b) => Math.abs(a - b),

  mc,

  PI,
  TAU,
  PIbyTwo,

  degreesToRadians,
  radiansToDegrees,
  distance2d,

  clamp,

  normalize,
  normals,
  bezierPoint,
  bezierTangent,
  quadraticPoint,
  bezierCoordinates,
  quadraticCoordinates
};
