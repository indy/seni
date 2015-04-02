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
    gl.viewportWidth = canvas.width;
    gl.viewportHeight = canvas.height;

    return gl;
  } catch (e) {
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

function initShaders(gl) {

  const shaderProgram = gl.createProgram();

  const fragmentSrc = `
  precision mediump float;
  varying vec4 vColor;

  void main(void) {
    gl_FragColor = vColor;
  }
  `;

  const vertexSrc = `
  attribute vec3 aVertexPosition;
  attribute vec4 aVertexColor;

  uniform mat4 uMVMatrix;
  uniform mat4 uPMatrix;

  varying vec4 vColor;

  void main(void) {
    gl_Position = uPMatrix * uMVMatrix * vec4(aVertexPosition, 1.0);
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

class GLContainer {
  constructor(canvas) {
    const gl = initGL(canvas);
    this.gl = gl;
    this.shaderProgram = initShaders(gl);
  }
}

export default GLContainer;
