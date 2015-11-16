/*
 *  Seni
 *  Copyright (C) 2015 Inderjit Gill <email@indy.io>
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

const MathUtil = {

  publicBindings: [
    new PublicBinding(
      'math/PI',
      ``,
      {},
      () => PI
    ),

    new PublicBinding(
      'math/TAU',
      ``,
      {},
      () => TAU
    ),

    new PublicBinding(
      'math/sin',
      ``,
      {angle: 0},
      (self) => function(params) {
        const {angle} = self.mergeWithDefaults(params);
        return Math.sin(angle);
      }
    ),

    new PublicBinding(
      'math/cos',
      ``,
      {angle: 0},
      (self) => function(params) {
        const {angle} = self.mergeWithDefaults(params);
        return Math.cos(angle);
      }
    ),
    new PublicBinding(
      'math/atan2',
      `Calculates the arc tangent of the two variables y and x. It is similar
to calculating the arc tangent of y / x, except that the signs of
both arguments are used to determine the quadrant of the result`,
      {x: 0,
       y: 0},
      (self) => function(params) {
        const {x, y} = self.mergeWithDefaults(params);
        return Math.atan2(y, x); // this is correct, y is given before x
      }
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
    ),

    new PublicBinding(
      'degrees->radians',
      `A helper function that converts angles in degrees to radians`,
      {angle: 0.0},
      (self) => function(params) {
        const {angle} = self.mergeWithDefaults(params);
        return degreesToRadians(angle);
      }
    ),

    new PublicBinding(
      'radians->degrees',
      `A helper function that converts angles in radians to degrees`,
      {angle: 0.0},
      (self) => function(params) {
        const {angle} = self.mergeWithDefaults(params);
        return radiansToDegrees(angle);
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
/*
  interpolate: function(a, b, t) {
    return (a * (1 - t)) + (b * t);
  },

  mc: function([xa, ya], [xb, yb]) {
    const m = (ya - yb) / (xa - xb);
    const c = ya - (m * xa);
    return [m, c];
  }, ?????
 */

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

export default MathUtil;
