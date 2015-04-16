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

const MatrixStackBindings = {

  pushMatrix: new PublicBinding(
    'push-matrix',
    ``,
    {},
    (self, matrixStack) => {
      return () => matrixStack.pushMatrix();
    }
  ),

  popMatrix: new PublicBinding(
    'pop-matrix',
    ``,
    {},
    (self, matrixStack) => {
      return () => matrixStack.popMatrix();
    }
  ),

  scale: new PublicBinding(
    'scale',
    ``,
    {x: 1, y: 1},
    (self, matrixStack) => {
      return (params) => {
        const {x, y} = self.mergeWithDefaults(params);
        return matrixStack.scale(x, y);
      };
    }
  ),

  translate: new PublicBinding(
    'translate',
    ``,
    {x: 0.0, y: 0.0},
    (self, matrixStack) => {
      return (params) => {
        const {x, y} = self.mergeWithDefaults(params);
        return matrixStack.translate(x, y);
      };
    }
  ),

  rotate: new PublicBinding(
    'rotate',
    ``,
    {angle: 0.0},
    (self, matrixStack) => {
      return (params) => {
        const {angle} = self.mergeWithDefaults(params);
        return matrixStack.rotate(angle);
      };
    }
  )
};

export default MatrixStackBindings;
