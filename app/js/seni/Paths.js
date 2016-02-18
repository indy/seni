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
import MathUtil from './MathUtil';

function emptyFn() {
  // an empty function that acts as the default value for fn arguments
}

function pathLinear(publicBinding, params) {
  const fullParams = publicBinding.mergeWithDefaults(params);

  const {coords, fn, steps} = fullParams;
  //const tStart = fullParams[`t-start`];
  //const tEnd = fullParams[`t-end`];

  const from = coords[0];
  const to = coords[coords.length - 1];

  const xUnit = (to[0] - from[0]) / (steps - 1);
  const yUnit = (to[1] - from[1]) / (steps - 1);

  for (let i = 0; i < steps; i++) {
    fn({
      step: i,
      position: [from[0] + (i * xUnit), from[1] + (i * yUnit)],
      t: i / steps
    });
  }
}

function pathCircle(publicBinding, params) {
  const fullParams = publicBinding.mergeWithDefaults(params);

  const {position, radius, fn, steps} = fullParams;
  const tStart = fullParams[`t-start`];
  const tEnd = fullParams[`t-end`];

  const [x, y] = position;
  const unit = (tEnd - tStart) / steps;
  const unitAngle = unit * MathUtil.TAU;

  for (let i = 0; i < steps; i++) {

    const angle = (unitAngle * i) + (tStart * MathUtil.TAU);
    const vx = (Math.sin(angle) * radius) + x;
    const vy = (Math.cos(angle) * radius) + y;

    fn({
      step: i,
      position: [vx, vy],
      t: tStart + (unit * i)
    });
  }
}

function pathCurve(publicBinding, params, coordFn) {
  const fullParams = publicBinding.mergeWithDefaults(params);

  const {coords, fn, steps} = fullParams;
  const tStart = fullParams[`t-start`];
  const tEnd = fullParams[`t-end`];

  const tVals = MathUtil.stepsInclusive(tStart, tEnd, steps);

  const {
    xs,
    ys
  } = coordFn(tVals, coords);

  for (let i = 0; i < steps; i++) {
    fn({
      step: i,
      position: [xs[i], ys[i]],
      t: tVals[i]
    });
  }
}

const publicBindings = [
  new PublicBinding(
    `path/linear`,
    { description:
      `invokes a given function with positions along a linear path`,
      args: [[`coords`, `a vector of 2 2D vectors`],
             [`fn`, `a function that accepts step, position and t params`],
             [`steps`, `10`],
             [`t-start`, `0`],
             [`t-end`, `1`]] },
    { coords: [[0, 0], [100, 100]],
      fn: emptyFn,
      steps: 10,
      't-start': 0.0,
      't-end': 1.0 },
    self => params => pathLinear(self, params)),

  new PublicBinding(
    `path/circle`,
    { description:
      `invokes a given function with positions along a circular path`,
      args: [[`position`, `the centre of the circle`],
             [`radius`, `the radius of the circle`],
             [`fn`, `a function that accepts step, position and t params`],
             [`steps`, `10`],
             [`t-start`, `0`],
             [`t-end`, `1`]] },
    { position: [500, 500],
      radius: 100,
      fn: emptyFn,
      steps: 10,
      't-start': 0.0,
      't-end': 1.0 },
    self => params => pathCircle(self, params)),

  new PublicBinding(
    `path/spline`,
    { description:
      `invokes a given function with positions along a quadratic spline path`,
      args: [[`coords`, `a vector of 3 2D vectors`],
             [`fn`, `a function that accepts step, position and t params`],
             [`steps`, `10`],
             [`t-start`, `0`],
             [`t-end`, `1`]] },
    { coords: [[0, 0], [30, 90], [100, 100]],
      fn: emptyFn,
      steps: 10,
      't-start': 0.0,
      't-end': 1.0 },
    self => params => pathCurve(self, params, MathUtil.quadraticCoordinates)),

  new PublicBinding(
    `path/bezier`,
    { description:
      `invokes a given function with positions along a Bezier spline path`,
      args: [[`coords`, `a vector of 4 2D vectors`],
             [`fn`, `a function that accepts step, position and t params`],
             [`steps`, `10`],
             [`t-start`, `0`],
             [`t-end`, `1`]] },
    { coords: [[0, 0], [30, 90], [60, 90], [100, 100]],
      fn: emptyFn,
      steps: 10,
      't-start': 0.0,
      't-end': 1.0 },
    self => params => pathCurve(self, params, MathUtil.bezierCoordinates))
];

export default {
  publicBindingType: `binding`,
  publicBindings
};
