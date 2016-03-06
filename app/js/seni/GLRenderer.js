/*
 Seni
 Copyright (C) 2016 Inderjit Gill <email@indy.io>

 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.

 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

const logToConsole = false;

function initGL(canvas) {
  try {
    const gl = canvas.getContext(`experimental-webgl`, {
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
    gl.getAttribLocation(shaderProgram, `aVertexPosition`);
  gl.enableVertexAttribArray(shaderProgram.positionAttribute);

  shaderProgram.colourAttribute =
    gl.getAttribLocation(shaderProgram, `aVertexColor`);
  gl.enableVertexAttribArray(shaderProgram.colourAttribute);

  shaderProgram.pMatrixUniform =
    gl.getUniformLocation(shaderProgram, `uPMatrix`);
  shaderProgram.mvMatrixUniform =
    gl.getUniformLocation(shaderProgram, `uMVMatrix`);

  return shaderProgram;
}

function setupGLState(gl) {
  gl.clearColor(1.0, 1.0, 1.0, 1.0);
  gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
  gl.enable(gl.BLEND);

  /**
   * Note: blendEquationSeparate and blendFuncSeparate behave exactly like the
   * original blendEquation and blendFunc functions. It looks like the extra
   * arguments in the *Separate functions are ignored on my dev machine
   */
  gl.blendEquationSeparate(gl.FUNC_ADD, gl.FUNC_ADD);
  gl.blendFuncSeparate(gl.SRC_ALPHA,
                       gl.ONE_MINUS_SRC_ALPHA,
                       gl.ONE_MINUS_DST_ALPHA,
                       gl.ONE);
  // original
  // http://www.andersriggelsen.dk/glblendfunc.php
  // gl.blendEquation(gl.FUNC_ADD);
  // gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

  gl.disable(gl.DEPTH_TEST);
}

export default class GLRenderer {
  constructor(canvasElement) {
    this.glDomElement = canvasElement;

    // webgl setup
    const gl = initGL(this.glDomElement);
    this.gl = gl;
    this.shaderProgram = setupShaders(gl);
    setupGLState(gl);
    this.glVertexBuffer = gl.createBuffer();
    this.glColourBuffer = gl.createBuffer();
  }

  getImageData() {
    return this.glDomElement.toDataURL();
  }

  preDrawScene(destWidth, destHeight, pMatrix, mvMatrix) {
    const gl = this.gl;
    const shaderProgram = this.shaderProgram;
    const domElement = this.glDomElement;

    if (domElement.width !== destWidth) {
      if (logToConsole) {
        console.log(`GL width from`, domElement.width, `to`, destWidth);
      }
      domElement.width = destWidth;
    }
    if (this.glDomElement.height !== destHeight) {
      if (logToConsole) {
        console.log(`GL height from`, domElement.height, `to`, destHeight);
      }
      domElement.height = destHeight;
    }
    // gl.drawingBufferWidth, gl.drawingBufferHeight hold the actual
    // size of the rendering element

    gl.viewport(0, 0, gl.drawingBufferWidth, gl.drawingBufferHeight);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

    gl.uniformMatrix4fv(shaderProgram.pMatrixUniform, false, pMatrix);
    gl.uniformMatrix4fv(shaderProgram.mvMatrixUniform, false, mvMatrix);
  }

  drawRenderPackets(renderPackets) {
    const gl = this.gl;
    const shaderProgram = this.shaderProgram;

    const glVertexBuffer = this.glVertexBuffer;
    const glColourBuffer = this.glColourBuffer;

    let sum = 0;
    const numPackets = renderPackets.length;
    for (let i = 0; i < numPackets; i++) {
      const renderPacket = renderPackets[i];
      sum += renderPacket.bufferLevel;

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
    if (logToConsole) {
      console.log(`rendered ${sum} vertices in ${numPackets} renderPackets`);
    }
  }
}
