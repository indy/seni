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

import MatrixStack from './MatrixStack';
import { opMatrixPush,
         opMatrixPop,
         opMatrixScale,
         opMatrixTranslate,
         opMatrixRotate,
         opRenderLine,
         opRenderRect,
         opRenderCircle,
         opRenderCircleSlice,
         opRenderPoly,
         opRenderBezier,
         opRenderQuadratic } from './RenderOps';

// todo: preDraw resets the matrix stack, will this behaviour
// be required in the ProxyRenderer?

export default class ProxyRenderer {
  constructor() {
    this.reset();
  }

  reset() {
    // matrix setup
    this.matrixStack = new MatrixStack();
    this.commandBuffer = [];
  }

  getCommandBuffer() {
    return this.commandBuffer;
  }

  vectorToCanvasSpace(v2) {
    const res = this.matrixStack.transform2DVector(v2);
    // destructuring Float32Array as Arrays doesn't work in safari
    // so we have to build and return a normal JS array
    return [res[0], res[1]];
  }

  cmdMatrixPush() {
    this.commandBuffer.push([opMatrixPush]);
    return this.matrixStack.pushMatrix();
  }

  cmdMatrixPop() {
    this.commandBuffer.push([opMatrixPop]);
    return this.matrixStack.popMatrix();
  }

  cmdMatrixScale(x, y) {
    this.commandBuffer.push([opMatrixScale, x, y]);
    return this.matrixStack.scale(x, y);
  }

  cmdMatrixTranslate(x, y) {
    this.commandBuffer.push([opMatrixTranslate, x, y]);
    return this.matrixStack.translate(x, y);
  }

  cmdMatrixRotate(angle) {
    this.commandBuffer.push([opMatrixRotate, angle]);
    return this.matrixStack.rotate(angle);
  }

  cmdRenderLine(params) {
    this.commandBuffer.push([opRenderLine, params]);
  }

  cmdRenderRect(params) {
    this.commandBuffer.push([opRenderRect, params]);
  }

  cmdRenderCircle(params) {
    this.commandBuffer.push([opRenderCircle, params]);
  }

  cmdRenderCircleSlice(params) {
    this.commandBuffer.push([opRenderCircleSlice, params]);
  }

  cmdRenderPoly(params) {
    this.commandBuffer.push([opRenderPoly, params]);
  }

  cmdRenderBezier(params) {
    this.commandBuffer.push([opRenderBezier, params]);
  }

  cmdRenderQuadratic(params) {
    this.commandBuffer.push([opRenderQuadratic, params]);
  }
}
