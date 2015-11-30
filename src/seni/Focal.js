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
import MathUtil from './MathUtil';
import Interp from './Interp';

function setupFocalParameters(publicBinding, params) {
  const fullParams = publicBinding.mergeWithDefaults(params);

  const {
    position,                   // the position of the point of interest
    distance,
    falloff
  } = fullParams;

  if (falloff !== 'linear' &&
      falloff !== 'quick' &&
      falloff !== 'slow-in' &&
      falloff !== 'slow-in-out') {
    console.log('invalid falloff value');
  }

  const fn = Interp.remapFn({
    from: [0, distance],
    to: [1, 0],
    mapping: falloff,
    clamping: true
  });

  return {
    position,
    fn
  };
}

function point(publicBinding, params, renderer) {

  const {
    position,
    fn
  } = setupFocalParameters(publicBinding, params);

  // returns a function that given a v2 returns how 'interesting' it should be
  return function(parameters) {
    const v = parameters.position || [0, 0];
    const p = renderer.vectorToCanvasSpace(v);
    const d = MathUtil.distance2d(position, p);

    return fn({val: d});
  };
}

const pointBinding = new PublicBinding(
  'focal/point',
  `returns a function that takes a v2 and returns the 'interest' at that point`,
  {
    position: [0, 0],
    distance: 100,
    falloff: 'linear'
  },
  (self, renderer) => params => point(self, params, renderer)
);


function vline(publicBinding, params, renderer) {
  const {
    position,
    fn
  } = setupFocalParameters(publicBinding, params);

  // returns a function that given a v2 returns how 'interesting' it should be
  return function(parameters) {
    const v = parameters.position || [0, 0];
    const p = renderer.vectorToCanvasSpace(v);
    const d = MathUtil.distance1d(position[0], p[0]);

    return fn({val: d});
  };
}

const vlineBinding = new PublicBinding(
  'focal/vline',
  `returns a function that takes a v2 and returns the 'interest' at that point`,
  {
    position: [500, 500],
    distance: 100,
    falloff: 'linear'
  },
  (self, renderer) => params => vline(self, params, renderer)
);


function hline(publicBinding, params, renderer) {
  const {
    position,
    fn
  } = setupFocalParameters(publicBinding, params);

  // returns a function that given a v2 returns how 'interesting' it should be
  return parameters => {
    const v = parameters.position || [0, 0];
    const p = renderer.vectorToCanvasSpace(v);
    const d = MathUtil.distance1d(position[1], p[1]);

    return fn({val: d});
  };
}

const hlineBinding = new PublicBinding(
  'focal/hline',
  `returns a function that takes a v2 and returns the 'interest' at that point`,
  {
    position: [500, 500],
    distance: 100,
    falloff: 'linear'
  },
  (self, renderer) => params => hline(self, params, renderer)
);

const Focal = {
  publicBindings: [
    pointBinding,
    vlineBinding,
    hlineBinding
  ]
};

export default Focal;
