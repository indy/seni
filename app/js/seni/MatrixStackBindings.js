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

const publicBindings = [
  new PublicBinding(
    'push-matrix',
    { description: 'pushes the current matrix onto the stack' },
    {},
    (self, renderer) => () => renderer.cmdMatrixPush()
  ),

  new PublicBinding(
    'pop-matrix',
    { description: 'pops a matrix from the stack' },
    {},
    (self, renderer) => () => renderer.cmdMatrixPop()
  ),

  new PublicBinding(
    'scale',
    { description: 'apply scaling to the matrix',
      args: [['vector', '[0 1]'],
             ['scalar', '1']]
    },
    { vector: [1, 1],
      scalar: 1 },
    (self, renderer) => params => {
      let vector;
      if (params.scalar) {
        vector = [params.scalar, params.scalar];
      } else {
        const obj = self.mergeWithDefaults(params);
        vector = obj.vector;
      }

      return renderer.cmdMatrixScale(vector[0], vector[1]);
    }
  ),

  new PublicBinding(
    'translate',
    { description: 'apply translation to the matrix',
      args: [['vector', '[0 1]']] },
    { vector: [0, 0] },
    (self, renderer) => params => {
      const {vector} = self.mergeWithDefaults(params);
      return renderer.cmdMatrixTranslate(vector[0], vector[1]);
    }
  ),

  new PublicBinding(
    'rotate',
    { description: 'apply a rotation to the matrix',
      args: [['angle', '0']] },
    { angle: 0.0 },
    (self, renderer) => params => {
      const {angle} = self.mergeWithDefaults(params);
      // angle is going to be in degrees
      const radians = MathUtil.degreesToRadians(angle);
      return renderer.cmdMatrixRotate(radians);
    }
  )
];

export default {
  publicBindingType: 'binding',
  publicBindings
};