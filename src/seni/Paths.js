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
import MathUtil from './MathUtil';

function emptyFn() {
  // an empty function that acts as the default value for fn arguments
}

function pathLinear(publicBinding, params) {
  let fullParams = publicBinding.mergeWithDefaults(params);

  let {coords, fn, steps} = fullParams;
  //const tStart = fullParams['t-start'];
  //const tEnd = fullParams['t-end'];

  let from = coords[0];
  let to = coords[coords.length - 1];

  const xUnit = (to[0] - from[0]) / (steps - 1);
  const yUnit = (to[1] - from[1]) / (steps - 1);

  for(let i = 0; i < steps; i++) {
    fn({
      step: i,
      position: [from[0] + (i * xUnit), from[1] + (i * yUnit)],
      't-value': i / steps
    });
  }
}

const linearBinding = new PublicBinding(
  'path/linear',
  `
  a linear path
  `,
  {
    coords: [[0, 0], [100, 100]],
    fn: emptyFn,
    steps: 10,
    't-start': 0.0,
    't-end': 1.0
  },
  (self) => {
    return (params) => pathLinear(self, params);
  });

function pathCircle(publicBinding, params) {
  let fullParams = publicBinding.mergeWithDefaults(params);

  let {position, radius, fn, steps} = fullParams;
  const tStart = fullParams['t-start'];
  const tEnd = fullParams['t-end'];

  let [x, y] = position;
  let twoPI = Math.PI * 2;
  let unit = (tEnd - tStart) / steps;
  let unitAngle = unit * twoPI;

  let angle, vx, vy;

  for(let i = 0; i < steps; i++) {

    angle = (unitAngle * i) + (tStart * twoPI);
    vx = (Math.sin(angle) * radius) + x;
    vy = (Math.cos(angle) * radius) + y;

    fn({
      step: i,
      position: [vx, vy],
      't-value': tStart + (unit * i)
    });
  }
}

const circleBinding = new PublicBinding(
  'path/circle',
  `
  a circular path
  `,
  {
    position: [500, 500],
    radius: 100,
    fn: emptyFn,
    steps: 10,
    't-start': 0.0,
    't-end': 1.0
  },
  (self) => {
    return (params) => pathCircle(self, params);
  });

function pathCurve(publicBinding, params, coordFn) {
  let fullParams = publicBinding.mergeWithDefaults(params);

  let {coords, fn, steps} = fullParams;
  const tStart = fullParams['t-start'];
  const tEnd = fullParams['t-end'];

  const tVals = MathUtil.stepsInclusive(tStart, tEnd, steps);

  const {
    xs,
    ys
  } = coordFn(tVals, coords);

  for(let i = 0; i < steps; i++) {
    fn({
      step: i,
      position: [xs[i], ys[i]],
      't-value': tVals[i]
    });
  }
}

const splineBinding = new PublicBinding(
  'path/spline',
  `
  a quadratic spline path
  `,
  {
    coords: [[0, 0], [30, 90], [100, 100]],
    fn: emptyFn,
    steps: 10,
    't-start': 0.0,
    't-end': 1.0
  },
  (self) => {
    return (params) => pathCurve(self, params, MathUtil.quadraticCoordinates);
  });

const bezierBinding = new PublicBinding(
  'path/bezier',
  `
  a bezier spline path
  `,
  {
    coords: [[0, 0], [30, 90], [60, 90], [100, 100]],
    fn: emptyFn,
    steps: 10,
    't-start': 0.0,
    't-end': 1.0
  },
  (self) => {
    return (params) => pathCurve(self, params, MathUtil.bezierCoordinates);
  });

const Paths = {
  linear: linearBinding,
  circle: circleBinding,
  spline: splineBinding,
  bezier: bezierBinding
};

export default Paths;
