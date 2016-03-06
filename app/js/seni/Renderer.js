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

import RenderPacket from './RenderPacket';
import GLRenderer from './GLRenderer';
import MatrixStack from './MatrixStack';
import MathUtil from './MathUtil';
import Interp from './Interp';
import Colour from './Colour';
import Util from './Util';
import { mat4 } from 'gl-matrix';

const Format = Colour.Format;

// command constants for the commandBuffer
const opMatrixPush = 0;
const opMatrixPop = 1;
const opMatrixScale = 2;
const opMatrixTranslate = 3;
const opMatrixRotate = 4;
const opRenderLine = 5;
const opRenderRect = 6;
const opRenderCircle = 7;
const opRenderCircleSlice = 8;
const opRenderPoly = 9;
const opRenderBezier = 10;
const opRenderQuadratic = 11;

export default class Renderer {
  constructor(canvasElement) {
    this.glRenderer = new GLRenderer(canvasElement);

    // matrix setup
    this.matrixStack = new MatrixStack();
    this.mvMatrix = mat4.create();
    this.pMatrix = mat4.create();
    mat4.ortho(this.pMatrix, 0, 1000, 0, 1000, 10, -10);

    this.renderPackets = [];
    this.renderPacket = new RenderPacket();
  }

  executeCommandBuffer(commandBuffer) {
    const hackColour = c => Colour.construct(c.format, c.elements);

    const hackParams = p => {
      if (p.colour) {
        p.colour = hackColour(p.colour);
      }

      if (p.colours) {
        p.colours = p.colours.map(hackColour);
      }

      return p;
    };

    commandBuffer.forEach(cmd => {
      const op = cmd[0];

      let params = undefined;
      if (op >= opRenderLine) {
        // make sure that colours are Immutable objects
        params = hackParams(cmd[1]);
      }

      switch (op) {
      case opMatrixPush:
        this.matrixStack.pushMatrix();
        break;
      case opMatrixPop:
        this.matrixStack.popMatrix();
        break;
      case opMatrixScale:
        this.matrixStack.scale(cmd[1], cmd[2]);
        break;
      case opMatrixTranslate:
        this.matrixStack.translate(cmd[1], cmd[2]);
        break;
      case opMatrixRotate:
        this.matrixStack.rotate(cmd[1]);
        break;
      case opRenderLine:
        this.renderLine(params);
        break;
      case opRenderRect:
        this.renderRect(params);
        break;
      case opRenderCircle:
        this.renderCircle(params);
        break;
      case opRenderCircleSlice:
        this.renderCircleSlice(params);
        break;
      case opRenderPoly:
        this.renderPoly(params);
        break;
      case opRenderBezier:
        this.renderCurve(params, MathUtil.bezierCoordinates);
        break;
      case opRenderQuadratic:
        this.renderCurve(params, MathUtil.quadraticCoordinates);
        break;
      }
    });
  }

  // todo: remove this once all rendering goes through ProxyRenderer
  vectorToCanvasSpace(v2) {
    const res = this.matrixStack.transform2DVector(v2);
    // destructuring Float32Array as Arrays doesn't work in safari
    // so we have to build and return a normal JS array
    return [res[0], res[1]];
  }

  renderLine(params) {
    const {
      from,
      to,
      width,
      colour
    } = params;

    const [x1, y1] = from;
    const [x2, y2] = to;
    const hw = width / 2;

    const [[n1x, n1y], [n2x, n2y]] = MathUtil.normals(x1, y1, x2, y2);

    const colourArray = Colour.elementArray(Colour.cloneAs(colour, Format.RGB));

    this.prepareToAddTriangleStrip(4, [x1 + (hw * n1x), y1 + (hw * n1y)]);
    this.addVertex([x1 + (hw * n1x), y1 + (hw * n1y)], colourArray);
    this.addVertex([x1 + (hw * n2x), y1 + (hw * n2y)], colourArray);
    this.addVertex([x2 + (hw * n1x), y2 + (hw * n1y)], colourArray);
    this.addVertex([x2 + (hw * n2x), y2 + (hw * n2y)], colourArray);
  }

  renderRect(params) {
    const {
      position,
      width,
      height,
      colour
    } = params;

    const [x, y] = position;
    const halfWidth = width / 2;
    const halfHeight = height / 2;

    const colourArray = Colour.elementArray(Colour.cloneAs(colour, Format.RGB));

    this.prepareToAddTriangleStrip(4, [x - halfWidth, y - halfHeight]);
    this.addVertex([x - halfWidth, y - halfHeight], colourArray);
    this.addVertex([x + halfWidth, y - halfHeight], colourArray);
    this.addVertex([x - halfWidth, y + halfHeight], colourArray);
    this.addVertex([x + halfWidth, y + halfHeight], colourArray);
  }

  renderCircle(params) {
    const {
      position,
      radius,
      tessellation,
      colour
    } = params;

    let {
      width,
      height
    } = params;

    const [x, y] = position;

    if (radius !== undefined) {
      // use the radius for both width and height if it's given
      width = radius;
      height = radius;
    }

    const colourArray = Colour.elementArray(Colour.cloneAs(colour, Format.RGB));

    this.prepareToAddTriangleStrip((tessellation * 2) + 2, [x, y]);

    const tau = Math.PI * 2;
    const unitAngle = tau / tessellation;
    let angle, vx, vy;

    for (let i = 0; i < tessellation; i++) {
      angle = unitAngle * i;
      vx = (Math.sin(angle) * width) + x;
      vy = (Math.cos(angle) * height) + y;

      this.addVertex([x, y], colourArray);
      this.addVertex([vx, vy], colourArray);
    }

    // close up the polygon
    angle = 0.0;
    vx = (Math.sin(angle) * width) + x;
    vy = (Math.cos(angle) * height) + y;

    this.addVertex([x, y], colourArray);
    this.addVertex([vx, vy], colourArray);
  }

  renderCircleSlice(params) {
    const {
      position,
      radius,
      tessellation,
      colour
    } = params;

    let {
      width,
      height
    } = params;

    const angleStart = params[`angle-start`];
    const angleEnd = params[`angle-end`];
    const innerWidth = params[`inner-width`];
    const innerHeight = params[`inner-height`];

    const degToRad = MathUtil.TAU / 360;

    const [x, y] = position;

    if (radius !== undefined) {
      // use the radius for both width and height if it's given
      width = radius;
      height = radius;
    }

    if (angleStart > angleEnd) {
      console.warn(`angleStart: ${angleStart} > angleEnd: ${angleEnd}`);
    }

    const rStart = angleStart * degToRad;
    const rEnd = angleEnd * degToRad;

    const colourArray = Colour.elementArray(Colour.cloneAs(colour, Format.RGB));

    //let tau = Math.PI * 2;
    const unitAngle = (rEnd - rStart) / tessellation;
    let angle, vx, vy, innervx, innervy;

    angle = rStart;
    innervx = (Math.sin(angle) * innerWidth) + x;
    innervy = (Math.cos(angle) * innerHeight) + y;
    this.prepareToAddTriangleStrip((tessellation * 2) + 2, [innervx, innervy]);

    for (let i = 0; i < tessellation; i++) {
      angle = rStart + (unitAngle * i);

      innervx = (Math.sin(angle) * innerWidth) + x;
      innervy = (Math.cos(angle) * innerHeight) + y;

      vx = (Math.sin(angle) * width) + x;
      vy = (Math.cos(angle) * height) + y;

      this.addVertex([innervx, innervy], colourArray);
      this.addVertex([vx, vy], colourArray);
    }

    // close up the polygon
    angle = rEnd;

    innervx = (Math.sin(angle) * innerWidth) + x;
    innervy = (Math.cos(angle) * innerHeight) + y;

    vx = (Math.sin(angle) * width) + x;
    vy = (Math.cos(angle) * height) + y;

    this.addVertex([innervx, innervy], colourArray);
    this.addVertex([vx, vy], colourArray);
  }

  renderPoly(params) {
    const {
      coords,
      colours
    } = params;

    const n = coords.length;
    // todo: check that the colours array is the same size as the coords array

    this.prepareToAddTriangleStrip(n, coords[0]);
    for (let i = 0; i < n; i++) {
      const c = Colour.elementArray(Colour.cloneAs(colours[i], Format.RGB));
      this.addVertex(coords[i], c);
    }
  }

  renderCurve(params, coordFn) {
    const {
      colour,
      coords,
      tessellation
    } = params;
    const tStart = params[`t-start`];
    const tEnd = params[`t-end`];

    const tVals = MathUtil.stepsInclusive(tStart, tEnd, tessellation);

    const {
      xs,
      ys
    } = coordFn(tVals, coords);

    const {
      halfWidthEnd,
      remap
    } = this.getRemapAndHalfWidthEnd(params);

    this.addVerticesAsStrip({
      tVals,
      xs,
      ys,
      tessellation,
      remap,
      colour,
      halfWidthEnd
    });
  }

  getRemapAndHalfWidthEnd(params) {
    const lineWidth = params[`line-width`];
    const lineWidthStart = params[`line-width-start`];
    const lineWidthEnd = params[`line-width-end`];
    const tStart = params[`t-start`];
    const tEnd = params[`t-end`];
    const lineWidthMapping = params[`line-width-mapping`];

    let halfWidthEnd, remap;

    if (lineWidth !== undefined) {
      // user has given a constant lineWidth parameter
      halfWidthEnd = lineWidth / 2.0;
      remap = () => halfWidthEnd;
    } else {
      // use the default start and end line widths
      const halfWidthStart  = lineWidthStart / 2.0;
      halfWidthEnd = lineWidthEnd / 2.0;
      remap = Interp.remapFn({from: [tStart, tEnd],
                              to: [halfWidthStart, halfWidthEnd],
                              mapping: lineWidthMapping});
    }

    return {halfWidthEnd, remap};
  }

  addVerticesAsStrip(args) {
    const {
      tVals,
      xs,
      ys,
      tessellation,
      remap,
      colour,
      halfWidthEnd
    } = args;

    const colourArray = Colour.elementArray(Colour.cloneAs(colour, Format.RGB));

    let i, ix, iy, v1, v2, xn1, yn1, xn2, yn2;
    let t = undefined;

    for (i = 0; i < tVals.length - 1; i++) {
      [[xn1, yn1], [xn2, yn2]] =
        MathUtil.normals(xs[i], ys[i], xs[i + 1], ys[i + 1]);

      ix = xs[i];
      iy = ys[i];

      t = tVals[i];

      v1 = [(xn1 * remap({val: t})) + ix, (yn1 * remap({val: t})) + iy];
      v2 = [(xn2 * remap({val: t})) + ix, (yn2 * remap({val: t})) + iy];

      if (i === 0) {
        this.prepareToAddTriangleStrip(tessellation * 2, v1);
      }

      this.addVertex(v1, colourArray);
      this.addVertex(v2, colourArray);
    }

    // final 2 vertices for the end point
    i = tVals.length - 2;
    [[xn1, yn1], [xn2, yn2]] =
      MathUtil.normals(xs[i], ys[i], xs[i + 1], ys[i + 1]);

    ix = xs[i + 1];
    iy = ys[i + 1];

    v1 = [(xn1 * halfWidthEnd) + ix, (yn1 * halfWidthEnd) + iy];
    v2 = [(xn2 * halfWidthEnd) + ix, (yn2 * halfWidthEnd) + iy];

    this.addVertex(v1, colourArray);
    this.addVertex(v2, colourArray);
  }

  getImageData() {
    return this.glRenderer.getImageData();
  }

  preDrawScene(destWidth, destHeight) {
    this.glRenderer.preDrawScene(destWidth, destHeight,
                                 this.pMatrix, this.mvMatrix);

    this.matrixStack.reset();

    this.renderPackets = [];
    this.renderPacket = new RenderPacket();
  }

  postDrawScene() {
    this.flushTriangles();

    Util.withTiming(`drawRenderPackets`, () => {
      this.glRenderer.drawRenderPackets(this.renderPackets);
    }, false);
  }

  // --------------------------------------------------------------------------

  // buffer code...
  /**
   * make sure the buffer has enough space to add n vertices
   * which will be rendered as a triangle strip
   * @param numVertices
   * @param p0 the first vertex position
   */
  prepareToAddTriangleStrip(numVertices, p0) {
    if (this.renderPacket.canVerticesFit(numVertices) === false) {
      this.flushTriangles();
    }

    if (this.renderPacket.isRenderPacketEmpty() === false) {
      const res = this.matrixStack.transform2DVector(p0);
      this.renderPacket.formDegenerateTriangle(res);
    }
  }

  /**
   * this assumes that the buffer will have enough space to add the vertex
   * @param p
   * @param c
   */
  addVertex(p, c) {
    const res = this.matrixStack.transform2DVector(p);
    this.renderPacket.addVertex(res, c);
  }

  flushTriangles() {
    if (this.renderPacket.isRenderPacketEmpty()) {
      return;
    }

    // add the current renderpacket into the renderpackets array
    this.renderPackets.push(this.renderPacket);
    this.renderPacket = new RenderPacket();
  }
}
