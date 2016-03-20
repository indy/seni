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

import Matrix from './Matrix';

export default class MatrixStack {

  constructor() {
    this.reset();
    this.transform = Matrix.create();
  }

  reset() {
    this.stack = [Matrix.create()];
    // only pay the cost of construction once
    this.out3 = new Float32Array(3);
    this.out2 = new Float32Array(2);
  }

  getHead() {
    const stack = this.stack;
    return stack[stack.length - 1];
  }

  pushMatrix() {
    const m = this.getHead();
    this.stack.push(Matrix.clone(m));
  }

  popMatrix() {
    this.stack.pop();
  }

  scale(sx, sy) {
    Matrix.identity(this.transform);
    Matrix.scale(this.transform, this.transform, [sx, sy, 1.0]);

    const m = this.getHead();
    Matrix.multiply(m, m, this.transform);
  }

  translate(tx, ty) {
    Matrix.identity(this.transform);
    Matrix.translate(this.transform, this.transform, [tx, ty, 0.0]);


    const m = this.getHead();
    Matrix.multiply(m, m, this.transform);
  }

  rotate(a) {
    Matrix.identity(this.transform);
    Matrix.rotateZ(this.transform, this.transform, a);

    const m = this.getHead();
    Matrix.multiply(m, m, this.transform);
  }

  transformVector(v) {
    const m = this.getHead();
    return Matrix.transformVec3(this.out3, v, m);
  }

  transform2DVector(v) {
    const m = this.getHead();
    return Matrix.transformVec2(this.out2, v, m);
  }
}
