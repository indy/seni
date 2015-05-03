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

    // anti-pattern:
    // http://webglfundamentals.org/webgl/lessons/webgl-anti-patterns.html
    gl.viewportWidth = canvas.width;
    gl.viewportHeight = canvas.height;

    console.log(gl.drawingBufferWidth, gl.drawingBufferHeight);

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

    // buffer code...
    // each buffer can hold 1000 'items' where an item is a vertex, colour etc
    this.bufferSize = 1000;
    this.vertexItemSize = 2; // xy
    this.colourItemSize = 4; // rgba
    this.vertexBuffer = new Float32Array(this.vertexItemSize * this.bufferSize);
    this.colourBuffer = new Float32Array(this.colourItemSize * this.bufferSize);
    // the level of both the vertex and colour buffer
    // to find the actual index position multiply bufferLevel
    // by the relevant itemSize of the buffer
    this.bufferLevel = 0;
    this.flushCount = 0;
  }

  vectorToCanvasSpace(v2) {
    let v3 = [v2[0], v2[1], 0];
    return this.matrixStack.transformVector(v3);
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

  preDrawScene() {
    const gl = this.gl;
    const shaderProgram = this.shaderProgram;

    gl.viewport(0, 0, gl.viewportWidth, gl.viewportHeight);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

    gl.uniformMatrix4fv(shaderProgram.pMatrixUniform, false, this.pMatrix);
    gl.uniformMatrix4fv(shaderProgram.mvMatrixUniform, false, this.mvMatrix);

    this.matrixStack.reset();
  }

  postDrawScene() {
    this.flushTriangles();
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

    if (this.bufferLevel >= this.bufferSize - (numVertices + 2)) {
      this.flushTriangles();
    }

    if (this.bufferLevel !== 0) {
      // add two vertex entries which will form degenerate triangles
      const lastVertexIndex = (this.bufferLevel - 1) * this.vertexItemSize;
      // just copy the previous entries
      // note: colour doesn't matter since these triangles won't be rendered
      this.addVertexWithoutMatrixMultiply(
        [this.vertexBuffer[lastVertexIndex + 0],
         this.vertexBuffer[lastVertexIndex + 1]],
        [0, 0, 0, 0]);

      this.addVertex(p0, [0, 0, 0, 0]);

      // Note: still need to call addVertex on the first
      // vertex when we 'really' render the strip
    }
  }

  /**
   * this assumes that the buffer will have enough space to add the vertex
   * @param p
   * @param c
   */
  addVertex(p, c) {
    const res = this.matrixStack.transform2DVector(p);

    let bl = this.bufferLevel * this.vertexItemSize;
    this.vertexBuffer[bl + 0] = res[0];
    this.vertexBuffer[bl + 1] = res[1];

    bl = this.bufferLevel * this.colourItemSize;
    this.colourBuffer[bl + 0] = c[0];
    this.colourBuffer[bl + 1] = c[1];
    this.colourBuffer[bl + 2] = c[2];
    this.colourBuffer[bl + 3] = c[3];

    this.bufferLevel += 1;
  }

  addVertexWithoutMatrixMultiply(p, c) {
    let bl = this.bufferLevel * this.vertexItemSize;
    this.vertexBuffer[bl + 0] = p[0];
    this.vertexBuffer[bl + 1] = p[1];
//    this.vertexBuffer[bl + 2] = p[2];

    bl = this.bufferLevel * this.colourItemSize;
    this.colourBuffer[bl + 0] = c[0];
    this.colourBuffer[bl + 1] = c[1];
    this.colourBuffer[bl + 2] = c[2];
    this.colourBuffer[bl + 3] = c[3];

    this.bufferLevel += 1;
  }

  flushTriangles() {

    if (this.bufferLevel === 0) {
      return;
    }

    this.flushCount += 1;

    const gl = this.gl;
    const shaderProgram = this.shaderProgram;

    const glVertexBuffer = this.glVertexBuffer;
    const glColourBuffer = this.glColourBuffer;

    gl.bindBuffer(gl.ARRAY_BUFFER, glVertexBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, this.vertexBuffer, gl.STATIC_DRAW);
    gl.vertexAttribPointer(shaderProgram.positionAttribute,
                           this.vertexItemSize, gl.FLOAT, false, 0, 0);

    gl.bindBuffer(gl.ARRAY_BUFFER, glColourBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, this.colourBuffer, gl.STATIC_DRAW);
    gl.vertexAttribPointer(shaderProgram.colourAttribute,
                           this.colourItemSize, gl.FLOAT, false, 0, 0);

    gl.drawArrays(gl.TRIANGLE_STRIP, 0, this.bufferLevel);

    this.bufferLevel = 0;
  }

}

export default Renderer;
