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

import RenderPacket from './RenderPacket';
import MatrixStack from './MatrixStack';
import MathUtil from './MathUtil';
import Colour from './Colour';
import { mat4 } from 'gl-matrix';

const Format = Colour.Format;

function initGL(canvas) {
  try {
    const gl = canvas.getContext('experimental-webgl', {
      alpha: false,
      preserveDrawingBuffer: true
    });
    // commented out because of jshint
    //    if (!gl) {
    //alert('Could not initialise WebGL, sorry :-(');
    //    }

    return gl;
  } catch (e) {
    return undefined;
  }
}

function compileShader(gl, type, src) {
  const shader = gl.createShader(type);
  gl.shaderSource(shader, src);
  gl.compileShader(shader);

  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    //alert(gl.getShaderInfoLog(shader));
    return null;
  }
  return shader;
}

function setupShaders(gl) {

  const shaderProgram = gl.createProgram();

  const fragmentSrc = `
  precision mediump float;
  varying vec4 vColor;

  void main(void) {
    gl_FragColor = vColor;
  }
  `;

  const vertexSrc = `
  attribute vec2 aVertexPosition;
  attribute vec4 aVertexColor;

  uniform mat4 uMVMatrix;
  uniform mat4 uPMatrix;

  varying vec4 vColor;

  void main(void) {
    gl_Position = uPMatrix * uMVMatrix * vec4(aVertexPosition, 0.0, 1.0);
    vColor = aVertexColor;
  }
  `;

  const vertexShader = compileShader(gl, gl.VERTEX_SHADER, vertexSrc);
  const fragmentShader = compileShader(gl, gl.FRAGMENT_SHADER, fragmentSrc);

  gl.attachShader(shaderProgram, vertexShader);
  gl.attachShader(shaderProgram, fragmentShader);

  gl.linkProgram(shaderProgram);

  // commented out because of jshint
  //  if (!gl.getProgramParameter(shaderProgram, gl.LINK_STATUS)) {
  //alert('Could not initialise shaders');
  //  }

  gl.useProgram(shaderProgram);

  shaderProgram.positionAttribute =
    gl.getAttribLocation(shaderProgram, 'aVertexPosition');
  gl.enableVertexAttribArray(shaderProgram.positionAttribute);

  shaderProgram.colourAttribute =
    gl.getAttribLocation(shaderProgram, 'aVertexColor');
  gl.enableVertexAttribArray(shaderProgram.colourAttribute);

  shaderProgram.pMatrixUniform =
    gl.getUniformLocation(shaderProgram, 'uPMatrix');
  shaderProgram.mvMatrixUniform =
    gl.getUniformLocation(shaderProgram, 'uMVMatrix');

  return shaderProgram;
}

function setupGLState(gl) {
  gl.clearColor(1.0, 1.0, 1.0, 1.0);

  gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
  //gl.clearColor(0.0, 0.0, 0.0, 1.0);

  // http://www.andersriggelsen.dk/glblendfunc.php
  //gl.blendFunc(gl.SRC_ALPHA, gl.ONE);
  //  gl.blendFunc(gl.GL_SRC_ALPHA, gl.GL_ONE_MINUS_SRC_ALPHA);
  //gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_DST_COLOR);

  //gl.blendFunc(gl.SRC_COLOR, gl.ONE_MINUS_SRC_COLOR);
  gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

  //  gl.blendFuncSeparate(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA,
  //                       gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
  gl.blendEquation(gl.FUNC_ADD);
  gl.enable(gl.BLEND);

  gl.disable(gl.DEPTH_TEST);
}

class Renderer {
  constructor(canvasElement) {
    this.glDomElement = document.getElementById(canvasElement);

    // webgl setup
    const gl = initGL(this.glDomElement);
    this.gl = gl;
    this.shaderProgram = setupShaders(gl);
    setupGLState(gl);
    this.glVertexBuffer = gl.createBuffer();
    this.glColourBuffer = gl.createBuffer();

    // matrix setup
    this.matrixStack = new MatrixStack();
    this.mvMatrix = mat4.create();
    this.pMatrix = mat4.create();
    mat4.ortho(this.pMatrix, 0, 1000, 0, 1000, 10, -10);

    this.renderPackets = [];
    this.renderPacket = new RenderPacket();
  }

  vectorToCanvasSpace(v2) {
    let res = this.matrixStack.transform2DVector(v2);
    // destructuring Float32Array as Arrays doesn't work in safari
    // so we have to build and return a normal JS array
    return [res[0], res[1]];
  }

  // ----------------------------------------------------------------------
  // functions beginning with cmd are commands.
  // perhaps split this out into a separate class?
  cmdMatrixPush() {
    return this.matrixStack.pushMatrix();
  }

  cmdMatrixPop() {
    return this.matrixStack.popMatrix();
  }

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

  cmdMatrixScale(x, y) {
    return this.matrixStack.scale(x, y);
  }

  cmdMatrixTranslate(x, y) {
    return this.matrixStack.translate(x, y);
  }

  cmdMatrixRotate(angle) {
    return this.matrixStack.rotate(angle);
  }

  cmdRenderRect(params) {
    return this.renderRect(params);
  }

  cmdRenderPoly(params) {
    return this.renderPoly(params);
  }

  cmdRenderGradientPoly(params, sides) {
    return this.renderGradientPoly(params, sides);
  }

  cmdRenderBezier(params) {
    return this.renderCurve(params, MathUtil.bezierCoordinates);
  }

  cmdRenderQuadratic(params) {
    return this.renderCurve(params, MathUtil.quadraticCoordinates);
  }

  // ----------------------------------------------------------------------

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

  renderPoly(params) {
    let {
      position,
      width,
      height,
      radius,
      tessellation,
      colour
    } = params;

    const [x, y] = position;

    if (radius !== undefined) {
      // use the radius for both width and height if it's given
      width = radius;
      height = radius;
    }

    const colourArray = Colour.elementArray(Colour.cloneAs(colour, Format.RGB));

    this.prepareToAddTriangleStrip((tessellation * 2) + 2, [x, y]);

    let twoPI = Math.PI * 2;
    let unitAngle = twoPI / tessellation;
    let angle, vx, vy;

    for(let i = 0; i < tessellation; i++) {

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

  renderGradientPoly(params, n) {
    const {
      coords,
      colours
    } = params;

    let c;

    this.prepareToAddTriangleStrip(n, coords[0]);
    for(let i = 0; i < n; i++) {
      c = Colour.elementArray(Colour.cloneAs(colours[i], Format.RGB));
      this.addVertex(coords[i], c);
    }
  }

  renderCurve(params, coordFn) {

    const {
      colour,
      coords,
      tessellation
    } = params;
    const tStart = params['t-start'];
    const tEnd = params['t-end'];

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

    const lineWidth = params['line-width'];
    const lineWidthStart = params['line-width-start'];
    const lineWidthEnd = params['line-width-end'];
    const tStart = params['t-start'];
    const tEnd = params['t-end'];
    const lineWidthMapping = params['line-width-mapping'];

    let halfWidthEnd, remap;

    if (lineWidth !== undefined) {
      // user has given a constant lineWidth parameter
      halfWidthEnd = lineWidth / 2.0;
      remap = () => halfWidthEnd;
    } else {
      // use the default start and end line widths
      const halfWidthStart  = lineWidthStart / 2.0;
      halfWidthEnd = lineWidthEnd / 2.0;
      remap = MathUtil.remapFn({from: [tStart, tEnd],
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

    let i, ix, iy, v1, v2, t, xn1, yn1, xn2, yn2;

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
    return this.glDomElement.toDataURL();
  }

  preDrawScene(destWidth, destHeight) {
    const gl = this.gl;
    const shaderProgram = this.shaderProgram;
    const domElement = this.glDomElement;

    if(domElement.width !== destWidth) {
      console.log('GL width from', domElement.width, 'to', destWidth);
      domElement.width = destWidth;
    }
    if(this.glDomElement.height !== destHeight) {
      console.log('GL height from', domElement.height, 'to', destHeight);
      domElement.height = destHeight;
    }
    // gl.drawingBufferWidth, gl.drawingBufferHeight hold the actual
    // size of the rendering element

    gl.viewport(0, 0, gl.drawingBufferWidth, gl.drawingBufferHeight);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

    gl.uniformMatrix4fv(shaderProgram.pMatrixUniform, false, this.pMatrix);
    gl.uniformMatrix4fv(shaderProgram.mvMatrixUniform, false, this.mvMatrix);

    this.matrixStack.reset();

    this.renderPackets = [];
    this.renderPacket = new RenderPacket();
  }

  renderRenderPackets() {
    const gl = this.gl;
    const shaderProgram = this.shaderProgram;

    const glVertexBuffer = this.glVertexBuffer;
    const glColourBuffer = this.glColourBuffer;

    let renderPacket;
    for(let i = 0; i < this.renderPackets.length; i++) {
      renderPacket = this.renderPackets[i];
      console.log('rendering render packet ', i,
                  'level:', renderPacket.bufferLevel);

      gl.bindBuffer(gl.ARRAY_BUFFER, glVertexBuffer);
      gl.bufferData(gl.ARRAY_BUFFER,
                    renderPacket.vertexBuffer, gl.STATIC_DRAW);
      gl.vertexAttribPointer(shaderProgram.positionAttribute,
                             renderPacket.vertexItemSize,
                             gl.FLOAT, false, 0, 0);

      gl.bindBuffer(gl.ARRAY_BUFFER, glColourBuffer);
      gl.bufferData(gl.ARRAY_BUFFER,
                    renderPacket.colourBuffer, gl.STATIC_DRAW);
      gl.vertexAttribPointer(shaderProgram.colourAttribute,
                             renderPacket.colourItemSize,
                             gl.FLOAT, false, 0, 0);

      gl.drawArrays(gl.TRIANGLE_STRIP, 0, renderPacket.bufferLevel);
    }
  }

  postDrawScene() {
    this.flushTriangles();

    this.renderRenderPackets();
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

    if (this.renderPacket.isRenderPacketEmpty() === false){
      const res = this.matrixStack.transform2DVector(p0);
      this.renderPacket.appendDegenerateVertices(res);
    }
  }

  /**
   * this assumes that the buffer will have enough space to add the vertex
   * @param p
   * @param c
   */
  addVertex(p, c) {
    const res = this.matrixStack.transform2DVector(p);
    this.renderPacket.appendVertex(res, c);
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

export default Renderer;
