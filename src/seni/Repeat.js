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
import MathUtil from './MathUtil';

function emptyFn() {
  // an empty function that acts as the default value for fn arguments
}

const vertical = [-1, 1];
const horizontal = [1, -1];

function mirror(renderer, drawFn, scaling) {
  let [x, y] = scaling;

  renderer.cmdMatrixPush();
  drawFn();
  renderer.cmdMatrixPop();

  renderer.cmdMatrixPush();
  renderer.cmdMatrixScale(x, y);
  drawFn();
  renderer.cmdMatrixPop();
}

// draws once then again with x,y swapped
function rotated90(renderer, drawFn) {
  renderer.cmdMatrixPush();
  drawFn();
  renderer.cmdMatrixPop();

  renderer.cmdMatrixPush();
  renderer.cmdMatrixRotate(MathUtil.PIbyTwo);
  drawFn();
  renderer.cmdMatrixPop();

}

const symmetryVertical = new PublicBinding(
  'repeat/symmetry-vertical',
  'renders the draw fn twice (mirrored vertically)',
  {
    draw: emptyFn
  },
  (self, renderer) => {
    return (params) => {
      let { draw } = self.mergeWithDefaults(params);
      mirror(renderer, draw, vertical);
    };
  }
);

const symmetryHorizontal = new PublicBinding(
  'repeat/symmetry-horizontal',
  'renders the draw fn twice (mirrored horizontally)',
  {
    draw: emptyFn
  },
  (self, renderer) => {
    return (params) => {
      let { draw } = self.mergeWithDefaults(params);
      mirror(renderer, draw, horizontal);
    };
  }
);

const symmetry4 = new PublicBinding(
  'repeat/symmetry-4',
  'renders the draw fn reflected along both the horizontal and vertical axis',
  {
    draw: emptyFn
  },
  (self, renderer) => {
    return (params) => {
      let { draw } = self.mergeWithDefaults(params);
      mirror(renderer, () => {
        mirror(renderer, draw, vertical);
      }, horizontal);
    };
  }
);

const symmetry8 = new PublicBinding(
  'repeat/symmetry-8',
  'renders the draw fn reflected 8 times', // todo: better doc
  {
    draw: emptyFn
  },
  (self, renderer) => {
    return (params) => {
      let { draw } = self.mergeWithDefaults(params);
      mirror(renderer, () => {
        mirror(renderer, () => {
          rotated90(renderer, draw);
        }, vertical);
      }, horizontal);
    };
  }
);

const rotate = new PublicBinding(
  'repeat/rotate',
  'renders multiple times by rotation',
  {
    draw: emptyFn,
    copies: 3
  },
  (self, renderer) => {
    return (params) => {
      let { draw, copies } = self.mergeWithDefaults(params);

      const delta = MathUtil.TAU / copies;

      for(let i = 0; i < copies; i++) {
        renderer.cmdMatrixPush();
        renderer.cmdMatrixRotate(delta * i);
        draw();
        renderer.cmdMatrixPop();
      }
    };
  }
);

const rotateMirrored = new PublicBinding(
  'repeat/rotate-mirrored',
  'renders multiple times by rotation',
  {
    draw: emptyFn,
    copies: 3
  },
  (self, renderer) => {
    return (params) => {
      let { draw, copies } = self.mergeWithDefaults(params);

      const delta = MathUtil.TAU / copies;

      for(let i = 0; i < copies; i++) {
        renderer.cmdMatrixPush();
        renderer.cmdMatrixRotate(delta * i);
        draw();
        renderer.cmdMatrixPop();
      }

      renderer.cmdMatrixPush();
      renderer.cmdMatrixScale(-1, 1);
      for(let i = 0; i < copies; i++) {
        renderer.cmdMatrixPush();
        renderer.cmdMatrixRotate(delta * i);
        draw();
        renderer.cmdMatrixPop();
      }
      renderer.cmdMatrixPop();

    };
  }
);

const Repeat = {
  publicBindings: [
    symmetryVertical,
    symmetryHorizontal,
    symmetry4,
    symmetry8,
    rotate,
    rotateMirrored
  ]
};

export default Repeat;
