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

import { vec2, vec3, mat4 } from 'gl-matrix';

class MatrixStack {

  constructor() {
    this.reset();
  }

  reset() {
    this.stack = [mat4.create()];
    this.out = vec3.create();   // only pay the cost of construction once
  }

  getHead() {
    const stack = this.stack;
    return stack[stack.length - 1];
  }

  pushMatrix() {
    const m = this.getHead();
    this.stack.push(mat4.clone(m));
  }

  popMatrix() {
    this.stack.pop();
  }

  scale(sx, sy) {
    const r = mat4.create();
    mat4.scale(r, r, [sx, sy, 1.0]);

    const m = this.getHead();
    mat4.mul(m, m, r);
  }

  translate(tx, ty) {
    const r = mat4.create();
    mat4.translate(r, r, [tx, ty, 0.0]);

    const m = this.getHead();
    mat4.mul(m, m, r);
  }

  rotate(a) {
    const r = mat4.create();
    mat4.rotateZ(r, r, a);

    const m = this.getHead();
    mat4.mul(m, m, r);
  }

  transformVector(v) {
    const m = this.getHead();
    return vec3.transformMat4(this.out, v, m);
  }

  transform2DVector(v) {
    const m = this.getHead();
    return vec2.transformMat4(this.out, v, m);
  }
}

export default MatrixStack;
