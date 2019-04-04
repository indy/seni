/*
 *  Seni
 *  Copyright (C) 2019 Inderjit Gill <email@indy.io>
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

const logToConsole = false;

// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

// matrix

function create() {
  const out = new Float32Array(16);
  out[0] = 1;
  out[1] = 0;
  out[2] = 0;
  out[3] = 0;
  out[4] = 0;
  out[5] = 1;
  out[6] = 0;
  out[7] = 0;
  out[8] = 0;
  out[9] = 0;
  out[10] = 1;
  out[11] = 0;
  out[12] = 0;
  out[13] = 0;
  out[14] = 0;
  out[15] = 1;

  return out;
}

function identity(out) {
  out[0] = 1;
  out[1] = 0;
  out[2] = 0;
  out[3] = 0;
  out[4] = 0;
  out[5] = 1;
  out[6] = 0;
  out[7] = 0;
  out[8] = 0;
  out[9] = 0;
  out[10] = 1;
  out[11] = 0;
  out[12] = 0;
  out[13] = 0;
  out[14] = 0;
  out[15] = 1;

  return out;
}

function ortho(out, left, right, bottom, top, near, far) {
  const lr = 1 / (left - right);
  const bt = 1 / (bottom - top);
  const nf = 1 / (near - far);

  out[0] = -2 * lr;
  out[1] = 0;
  out[2] = 0;
  out[3] = 0;
  out[4] = 0;
  out[5] = -2 * bt;
  out[6] = 0;
  out[7] = 0;
  out[8] = 0;
  out[9] = 0;
  out[10] = 2 * nf;
  out[11] = 0;
  out[12] = (left + right) * lr;
  out[13] = (top + bottom) * bt;
  out[14] = (far + near) * nf;
  out[15] = 1;

  return out;
}

function clone(a) {
  const out = new Float32Array(16);
  out[0] = a[0];
  out[1] = a[1];
  out[2] = a[2];
  out[3] = a[3];
  out[4] = a[4];
  out[5] = a[5];
  out[6] = a[6];
  out[7] = a[7];
  out[8] = a[8];
  out[9] = a[9];
  out[10] = a[10];
  out[11] = a[11];
  out[12] = a[12];
  out[13] = a[13];
  out[14] = a[14];
  out[15] = a[15];

  return out;
}

function scale(out, a, v) {
  const x = v[0], y = v[1], z = v[2];

  out[0] = a[0] * x;
  out[1] = a[1] * x;
  out[2] = a[2] * x;
  out[3] = a[3] * x;
  out[4] = a[4] * y;
  out[5] = a[5] * y;
  out[6] = a[6] * y;
  out[7] = a[7] * y;
  out[8] = a[8] * z;
  out[9] = a[9] * z;
  out[10] = a[10] * z;
  out[11] = a[11] * z;
  out[12] = a[12];
  out[13] = a[13];
  out[14] = a[14];
  out[15] = a[15];

  return out;
}

function multiply(out, a, b) {
  const a00 = a[0], a01 = a[1], a02 = a[2], a03 = a[3];
  const a10 = a[4], a11 = a[5], a12 = a[6], a13 = a[7];
  const a20 = a[8], a21 = a[9], a22 = a[10], a23 = a[11];
  const a30 = a[12], a31 = a[13], a32 = a[14], a33 = a[15];

  // Cache only the current line of the second matrix
  const b0  = b[0], b1 = b[1], b2 = b[2], b3 = b[3];
  out[0] = b0*a00 + b1*a10 + b2*a20 + b3*a30;
  out[1] = b0*a01 + b1*a11 + b2*a21 + b3*a31;
  out[2] = b0*a02 + b1*a12 + b2*a22 + b3*a32;
  out[3] = b0*a03 + b1*a13 + b2*a23 + b3*a33;

  b0 = b[4]; b1 = b[5]; b2 = b[6]; b3 = b[7];
  out[4] = b0*a00 + b1*a10 + b2*a20 + b3*a30;
  out[5] = b0*a01 + b1*a11 + b2*a21 + b3*a31;
  out[6] = b0*a02 + b1*a12 + b2*a22 + b3*a32;
  out[7] = b0*a03 + b1*a13 + b2*a23 + b3*a33;

  b0 = b[8]; b1 = b[9]; b2 = b[10]; b3 = b[11];
  out[8] = b0*a00 + b1*a10 + b2*a20 + b3*a30;
  out[9] = b0*a01 + b1*a11 + b2*a21 + b3*a31;
  out[10] = b0*a02 + b1*a12 + b2*a22 + b3*a32;
  out[11] = b0*a03 + b1*a13 + b2*a23 + b3*a33;

  b0 = b[12]; b1 = b[13]; b2 = b[14]; b3 = b[15];
  out[12] = b0*a00 + b1*a10 + b2*a20 + b3*a30;
  out[13] = b0*a01 + b1*a11 + b2*a21 + b3*a31;
  out[14] = b0*a02 + b1*a12 + b2*a22 + b3*a32;
  out[15] = b0*a03 + b1*a13 + b2*a23 + b3*a33;

  return out;
}

function translate(out, a, v) {
  const x = v[0], y = v[1], z = v[2];
  // let a00, a01, a02, a03, a10, a11, a12, a13, a20, a21, a22, a23;

  if (a === out) {
    out[12] = a[0] * x + a[4] * y + a[8] * z + a[12];
    out[13] = a[1] * x + a[5] * y + a[9] * z + a[13];
    out[14] = a[2] * x + a[6] * y + a[10] * z + a[14];
    out[15] = a[3] * x + a[7] * y + a[11] * z + a[15];
  } else {
    const a00 = a[0], a01 = a[1], a02 = a[2], a03 = a[3];
    const a10 = a[4], a11 = a[5], a12 = a[6], a13 = a[7];
    const a20 = a[8], a21 = a[9], a22 = a[10], a23 = a[11];

    out[0] = a00; out[1] = a01; out[2] = a02; out[3] = a03;
    out[4] = a10; out[5] = a11; out[6] = a12; out[7] = a13;
    out[8] = a20; out[9] = a21; out[10] = a22; out[11] = a23;

    out[12] = a00 * x + a10 * y + a20 * z + a[12];
    out[13] = a01 * x + a11 * y + a21 * z + a[13];
    out[14] = a02 * x + a12 * y + a22 * z + a[14];
    out[15] = a03 * x + a13 * y + a23 * z + a[15];
  }

  return out;
}

function rotateZ(out, a, rad) {
  const s = Math.sin(rad), c = Math.cos(rad);
  const a00 = a[0], a01 = a[1], a02 = a[2], a03 = a[3];
  const a10 = a[4], a11 = a[5], a12 = a[6], a13 = a[7];

  if (a !== out) {
    out[8] = a[8];
    out[9] = a[9];
    out[10] = a[10];
    out[11] = a[11];
    out[12] = a[12];
    out[13] = a[13];
    out[14] = a[14];
    out[15] = a[15];
  }

  // Perform axis-specific matrix multiplication
  out[0] = a00 * c + a10 * s;
  out[1] = a01 * c + a11 * s;
  out[2] = a02 * c + a12 * s;
  out[3] = a03 * c + a13 * s;
  out[4] = a10 * c - a00 * s;
  out[5] = a11 * c - a01 * s;
  out[6] = a12 * c - a02 * s;
  out[7] = a13 * c - a03 * s;

  return out;
}

function transformVec2(out, a, m) {
  const x = a[0];
  const y = a[1];
  out[0] = m[0] * x + m[4] * y + m[12];
  out[1] = m[1] * x + m[5] * y + m[13];

  return out;
}

function transformVec3(out, a, m) {
  const x = a[0], y = a[1], z = a[2];
  let w = m[3] * x + m[7] * y + m[11] * z + m[15];
  w = w || 1.0;
  out[0] = (m[0] * x + m[4] * y + m[8] * z + m[12]) / w;
  out[1] = (m[1] * x + m[5] * y + m[9] * z + m[13]) / w;
  out[2] = (m[2] * x + m[6] * y + m[10] * z + m[14]) / w;

  return out;
}

const Matrix = {
  create,
  identity,
  ortho,
  clone,
  scale,
  multiply,
  translate,
  rotateZ,
  transformVec2,
  transformVec3
};



// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

// renderer


function memorySubArray(mem, ptr, length) {
  const nByte = 4;
  const pos = ptr / nByte;
  return mem.subarray(pos, pos + length);
}

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
  varying highp vec2 vTextureCoord;

  uniform sampler2D uSampler;

  void main(void) {
    vec4 tex = texture2D(uSampler, vTextureCoord);

    gl_FragColor.r = tex.r * vColor.r;
    gl_FragColor.g = tex.r * vColor.g;
    gl_FragColor.b = tex.r * vColor.b;
    gl_FragColor.a = tex.r * vColor.a;

  }
  `;

  const vertexSrc = `
  attribute vec2 aVertexPosition;
  attribute vec4 aVertexColor;
  attribute vec2 aVertexTexture;

  uniform mat4 uMVMatrix;
  uniform mat4 uPMatrix;

  varying vec4 vColor;
  varying highp vec2 vTextureCoord;

  void main(void) {
    gl_Position = uPMatrix * uMVMatrix * vec4(aVertexPosition, 0.0, 1.0);
    vColor = aVertexColor;
    vTextureCoord = aVertexTexture;
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

  shaderProgram.textureAttribute =
    gl.getAttribLocation(shaderProgram, 'aVertexTexture');
  gl.enableVertexAttribArray(shaderProgram.textureAttribute);

  shaderProgram.pMatrixUniform =
    gl.getUniformLocation(shaderProgram, 'uPMatrix');
  shaderProgram.mvMatrixUniform =
    gl.getUniformLocation(shaderProgram, 'uMVMatrix');

  return shaderProgram;
}

function setupGLState(gl) {
  gl.clearColor(1.0, 1.0, 1.0, 1.0);
  gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
  gl.enable(gl.BLEND);

  // assuming that we'll be using pre-multiplied alpha
  // see http://www.realtimerendering.com/blog/gpus-prefer-premultiplication/
  gl.blendEquation(gl.FUNC_ADD);
  gl.blendFunc(gl.ONE, gl.ONE_MINUS_SRC_ALPHA);

//  gl.disable(gl.DEPTH_TEST);
}


function handleTextureLoaded(gl, image, texture, shaderProgram) {
  gl.bindTexture(gl.TEXTURE_2D, texture);
  gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, true);
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, image);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER,
                   gl.LINEAR_MIPMAP_NEAREST);
  gl.generateMipmap(gl.TEXTURE_2D);
  gl.bindTexture(gl.TEXTURE_2D, null);

  gl.activeTexture(gl.TEXTURE0);
  gl.bindTexture(gl.TEXTURE_2D, texture);
  gl.uniform1i(gl.getUniformLocation(shaderProgram, 'uSampler'), 0);
}

class GLRenderer {
  constructor(canvasElement) {
    this.glDomElement = canvasElement;

    // webgl setup
    const gl = initGL(this.glDomElement);
    this.gl = gl;

    this.shaderProgram = setupShaders(gl);
    setupGLState(gl);

    this.glVertexBuffer = gl.createBuffer();
    this.glColourBuffer = gl.createBuffer();
    this.glTextureBuffer = gl.createBuffer();

    this.mvMatrix = Matrix.create();
    this.pMatrix = Matrix.create();
    Matrix.ortho(this.pMatrix, 0, 1000, 0, 1000, 10, -10);
  }

  loadTexture(src) {
    let that = this;

    return new Promise(function(resolve, reject) {

      const gl = that.gl;
      that.texture = gl.createTexture();
      const image = new Image();

      image.addEventListener('load', () => {
        handleTextureLoaded(that.gl, image, that.texture, that.shaderProgram);
        resolve();
      });

      image.addEventListener('error', () => {
        reject();
      });

      image.src = src;
    });
  }

  getImageData() {
    return this.glDomElement.toDataURL();
  }

  preDrawScene(destWidth, destHeight, section) {
    const gl = this.gl;
    const domElement = this.glDomElement;

    if (domElement.width !== destWidth) {
      if (logToConsole) {
        console.log('GL width from', domElement.width, 'to', destWidth);
      }
      domElement.width = destWidth;
    }
    if (this.glDomElement.height !== destHeight) {
      if (logToConsole) {
        console.log('GL height from', domElement.height, 'to', destHeight);
      }
      domElement.height = destHeight;
    }
    // gl.drawingBufferWidth, gl.drawingBufferHeight hold the actual
    // size of the rendering element

    gl.viewport(0, 0, gl.drawingBufferWidth, gl.drawingBufferHeight);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);


    if (section === undefined) {
      // render the entirety of the scene
      Matrix.ortho(this.pMatrix, 0, 1000, 0, 1000, 10, -10);
    } else {
      switch (section) {
        // bottom left
      case 0: Matrix.ortho(this.pMatrix,   0,  500,   0,  500, 10, -10); break;
        // bottom right
      case 1: Matrix.ortho(this.pMatrix, 500, 1000,   0,  500, 10, -10); break;
        // top left
      case 2: Matrix.ortho(this.pMatrix,   0,  500, 500, 1000, 10, -10); break;
        // top right
      case 3: Matrix.ortho(this.pMatrix, 500, 1000, 500, 1000, 10, -10); break;
      }
    }

    gl.uniformMatrix4fv(this.shaderProgram.pMatrixUniform,
                        false,
                        this.pMatrix);

    gl.uniformMatrix4fv(this.shaderProgram.mvMatrixUniform,
                        false,
                        this.mvMatrix);
  }

  drawBuffer(memoryF32, buffer) {
    const gl = this.gl;
    const shaderProgram = this.shaderProgram;

    const glVertexBuffer = this.glVertexBuffer;
    const glColourBuffer = this.glColourBuffer;
    const glTextureBuffer = this.glTextureBuffer;

    const bytesin32bit = 4;

    const vertexItemSize = 2;
    const colourItemSize = 4;
    const textureItemSize = 2;

    const totalSize = (vertexItemSize + colourItemSize + textureItemSize);


    const gbuf = memorySubArray(memoryF32, buffer.geo_ptr, buffer.geo_len);

    gl.bindBuffer(gl.ARRAY_BUFFER, glVertexBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, gbuf, gl.STATIC_DRAW);
    gl.vertexAttribPointer(shaderProgram.positionAttribute,
                           vertexItemSize,
                           gl.FLOAT, false, totalSize * bytesin32bit,
                           0 * bytesin32bit);

    gl.bindBuffer(gl.ARRAY_BUFFER, glColourBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, gbuf, gl.STATIC_DRAW);
    gl.vertexAttribPointer(shaderProgram.colourAttribute,
                           colourItemSize,
                           gl.FLOAT, false, totalSize * bytesin32bit,
                           vertexItemSize * bytesin32bit);

    gl.bindBuffer(gl.ARRAY_BUFFER, glTextureBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, gbuf, gl.STATIC_DRAW);
    gl.vertexAttribPointer(shaderProgram.textureAttribute,
                           textureItemSize,
                           gl.FLOAT, false, totalSize * bytesin32bit,
                           (vertexItemSize + colourItemSize) * bytesin32bit);

    gl.drawArrays(gl.TRIANGLE_STRIP, 0, buffer.geo_len / totalSize);

  }
}



// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

// history

function senModeAsString(state) {
  const mode = state.currentMode;

  switch (mode) {
  case SeniMode.gallery:
    return 'gallery';
  case SeniMode.edit:
    return state.scriptId;
  case SeniMode.evolve:
    return 'evolve';
  default:
    return 'error unknown SeniMode value';
  }
}

function buildState(appState) {
  const state = appState;
  const uri = `#${senModeAsString(state)}`;

  return [state, uri];
}

const History = {
  pushState: function(appState) {
    const [state, uri] = buildState(appState);
    if (logToConsole) {
      console.log('historyPushState', state);
    }
    history.pushState(state, null, uri);
  },
  replaceState: function(appState) {
    const [state, uri] = buildState(appState);
    if (logToConsole) {
      console.log('historyReplace', state);
    }
    history.replaceState(state, null, uri);
  },
  restoreState: function(state) {
    if (logToConsole) {
      console.log('historyRestore', state);
    }

    return state;
  }
};

// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

// codemirrorSeni

function seniMode() {
  const BUILTIN = 'builtin';
  const COMMENT = 'comment';
  const STRING = 'string';
  const ATOM = 'atom';
  const NUMBER = 'number';
  const PAREN = 'paren';      // ()
  const CURLY = 'curly';    // {}
  const BRACKET = 'bracket';    // []
  const SENICOMMON = 'seni-common';
  const PARAMETER = 'seni-parameter';

  const INDENT_WORD_SKIP = 2;

  function makeKeywords(str) {
    const obj = {}, words = str.split(/\s+/);
    for (let i = 0; i < words.length; ++i) obj[words[i]] = true;
    return obj;
  }

  // keywords are core to the seni language
  const keywords =
        makeKeywords('begin define fn if fence loop on-matrix-stack quote');
  const indentKeys = makeKeywords('define fence loop on-matrix-stack fn');

  // functions from the common seni library
  const seniCommon = makeKeywords(`* + - / < = > append begin bezier
bezier-bulging bezier-trailing box canvas/centre canvas/height canvas/width
circle circle-slice col/analagous col/bezier-fn col/complementary col/convert
col/darken col/get-alpha col/get-lab-a col/get-lab-b col/get-lab-l
col/get-rgb-b col/get-rgb-g col/get-rgb-r col/hsl col/hsluv col/hsv col/lab col/lighten
col/procedural-fn col/quadratic-fn col/rgb col/set-alpha col/set-lab-a
col/set-lab-b col/set-lab-l col/set-rgb-b col/set-rgb-g col/set-rgb-r
col/split-complementary col/triad define degrees->radians fence fn focal/hline
focal/point focal/vline if interp/bezier interp/bezier-fn interp/bezier-tangent
interp/bezier-tangent-fn interp/circle interp/fn line list list/get list/length
log loop math/PI math/TAU math/atan2 math/clamp math/cos math/distance-2d
math/sin mod on-matrix-stack path/bezier path/circle path/linear path/spline
poly pop-matrix print prng/perlin-signed prng/perlin-unsigned prng/range
push-matrix quote radians->degrees rect repeat/rotate repeat/rotate-mirrored
repeat/symmetry-4 repeat/symmetry-8 repeat/symmetry-horizontal
repeat/symmetry-vertical rotate scale spline sqrt stroked-bezier
stroked-bezier-rect take translate`);

  function StateStack(indent, type, prev) { // represents a state stack object
    this.indent = indent;
    this.type = type;
    this.prev = prev;
  }

  function pushStack(state, indent, type) {
    state.indentStack = new StateStack(indent, type, state.indentStack);
  }

  function popStack(state) {
    state.indentStack = state.indentStack.prev;
  }

  const decimalMatcher = new RegExp(/^([-+]?\d*\.?\d*)/);

  function isDecimalNumber(stream, backup) {
    if (backup === true) {
      stream.backUp(1);
    }
    return stream.match(decimalMatcher);
  }

  function isParameter(word) {
    return word.slice(-1) === ':';
  }

  function tokenType(token, state, ch) {
    const prefix = 'geno-';
    let usePrefix = false;

    if (state.insideCurly) {
      // leave the first element inside curlys as is.

      if (state.curlyCounter === 1) {
        usePrefix = false;
        // this is the first element in the curlys
        state.curlyedFirstChildIsParen = (token === PAREN);
        if (state.curlyedFirstChildIsParen) {
          // special case of the first child in curlys being a s-exp.
          // we'll need to keep count of parenDepth
          state.firstParenCurlyDepth = state.parenDepth;
        }
      } else {
        // normally grey out, except if we're curlyedFirstChildIsParen
        if (state.curlyedFirstChildIsParen &&
            state.firstParenCurlyDepth <= state.parenDepth) {
          // keep on colouring as normal
          usePrefix = false;

          // if this is a closing parens then we've processed the first s-exp
          // and can start using prefix
          // (i.e. start greying out the remainder of the curly contents)
          if (state.firstParenCurlyDepth === state.parenDepth && ch === ')') {
            state.curlyedFirstChildIsParen = false;
          }
        } else {
          usePrefix = true;
        }
      }

      state.curlyCounter++;
    }

    return usePrefix ? prefix + token : token;
  }

  function setInsideCurly(value, state) {
    if (value === true) {
      state.curlyCounter = 0;
    }
    state.insideCurly = value;
  }

  return {
    startState: () => {
      const state = {
        indentStack: null,
        indentation: 0,
        mode: false,
        sExprComment: false,

        parenDepth: 0,

        insideCurly: false,
        curlyCounter: 0,
        firstParenCurlyDepth: 0,
        curlyedFirstChildIsParen: false
      };
      return state;
    },

    token: (stream, state) => {
      if (state.indentStack === null && stream.sol()) {
        // update indentation, but only if indentStack is empty
        state.indentation = stream.indentation();
      }

      // skip spaces
      if (stream.eatSpace()) {
        return null;
      }
      let returnType = null;
      let next;

      switch (state.mode) {
      case 'string': // multi-line string parsing mode
        let escaped = false;
        while ((next = stream.next()) != null) {
          if (next === '\"' && !escaped) {
            state.mode = false;
            break;
          }
          escaped = !escaped && next === '\\';
        }
        // continue on in scheme-string mode
        returnType = tokenType(STRING, state);
        break;
      case 'comment': // comment parsing mode
        let maybeEnd = false;
        while ((next = stream.next()) != null) {
          if (next === '#' && maybeEnd) {
            state.mode = false;
            break;
          }
          maybeEnd = (next === '|');
        }
        returnType = tokenType(COMMENT, state);
        break;
      default: // default parsing mode
        const ch = stream.next();

        if (ch === '\"') {
          state.mode = 'string';
          returnType = tokenType(STRING, state);
        } else if (ch === '\'') {
          returnType = tokenType(ATOM, state);
        } else if (/^[-+0-9.]/.test(ch) && isDecimalNumber(stream, true)) {
          // match non-prefixed number, must be decimal
          returnType = tokenType(NUMBER, state);
        } else if (ch === ';') { // comment
          stream.skipToEnd(); // rest of the line is a comment
          returnType = tokenType(COMMENT, state);
        } else if (ch === '[') { // bracket
          pushStack(state, stream.column() + 1, ch);
          returnType = tokenType(BRACKET, state);
        } else if (ch === ']') { // bracket
          popStack(state);
          returnType = tokenType(BRACKET, state);
        } else if (ch === '(' || ch === '{') {
          let keyWord = '', letter;
          const indentTemp = stream.column();

          if (ch === '{') {
            setInsideCurly(true, state);
          } else {
            state.parenDepth++;
          }

          while ((letter = stream.eat(/[^\s\(\)\[\]\{\}\;]/)) != null) {
            keyWord += letter;
          }

          if (keyWord.length > 0 && indentKeys.propertyIsEnumerable(keyWord)) {
            // indent-word
            pushStack(state, indentTemp + INDENT_WORD_SKIP, ch);
          } else { // non-indent word
            // we continue eating the spaces
            stream.eatSpace();
            if (stream.eol() || stream.peek() === ';') {
              // nothing significant after
              // we restart indentation 1 space after
              pushStack(state, indentTemp + 1, ch);
            } else {
              pushStack(state, indentTemp + stream.current().length, ch);
              // else we match
            }
          }
          stream.backUp(stream.current().length - 1); // undo all the eating

          if (typeof state.sExprComment === 'number') state.sExprComment++;

          returnType = tokenType(ch === '{' ? CURLY : PAREN, state, ch);
        } else if (ch === ')' || ch === '}') {
          returnType = tokenType(ch === '}' ? CURLY : PAREN, state, ch);
          if (state.indentStack != null &&
              state.indentStack.type === (ch === ')' ? '(' : '{')) {
            popStack(state);

            if (typeof state.sExprComment === 'number') {
              if (--state.sExprComment === 0) {
                returnType = tokenType(COMMENT, state); // final closing curly
                state.sExprComment = false; // turn off s-expr commenting mode
              }
            }
          }
          if (ch === '}') {
            setInsideCurly(false, state);
          } else {
            state.parenDepth--;
          }
        } else {
          stream.eatWhile(/[\w\$_\-!$%&*+\.\/:<=>?@\^~]/);

          if (keywords.propertyIsEnumerable(stream.current())) {
            returnType = tokenType(BUILTIN, state);
          } else if (seniCommon.propertyIsEnumerable(stream.current())) {
            returnType = tokenType(SENICOMMON, state);
          } else if (isParameter(stream.current())) {
            returnType = tokenType(PARAMETER, state);
          } else returnType = tokenType('variable', state);
        }
      }
      return (typeof state.sExprComment === 'number') ? COMMENT : returnType;
    },

    indent: state => {
      if (state.indentStack === null) return state.indentation;
      return state.indentStack.indent;
    },

    closeBrackets: {pairs: '()[]{}\"\"'},
    lineComment: ';;'
  };
}

const CodeMirrorSeni = {
  seniMode
};


// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

// editor

const modeName = 'seni';

function defineSeniMode() {
  // return an instance of CodeMirror with Seni mode defined
  CodeMirror.defineMode(modeName, CodeMirrorSeni.seniMode);
  return CodeMirror;
}

const Editor = {
  createEditor: function(element, customConfig) {
    const codeMirrorSeniMode = defineSeniMode();
    const defaultConfig = {
      lineNumbers: false,
      mode: modeName,
      autoCloseBrackets: true,
      matchBrackets: true
    };
    const res = Object.assign({}, defaultConfig, customConfig);

    return codeMirrorSeniMode.fromTextArea(element, res);
  }
};



// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

// store

let currentState = undefined;

function cloneState(state) {
  const clone = {};

  clone.highResolution = state.highResolution;
  clone.placeholder = state.placeholder;
  clone.populationSize = state.populationSize;
  clone.mutationRate = state.mutationRate;

  clone.currentMode = state.currentMode;
  clone.galleryLoaded = state.galleryLoaded;
  clone.previouslySelectedGenotypes = state.previouslySelectedGenotypes;
  clone.selectedIndices = state.selectedIndices;
  clone.scriptId = state.scriptId;
  clone.script = state.script;
  clone.scriptHash = state.scriptHash;
  clone.genotypes = state.genotypes;
  clone.traits = state.traits;

  return clone;
}

function resolveAsCurrentState(resolve, state) {
  currentState = state;
  resolve(currentState);
}

function actionSetMode(state, { mode }) {
  return new Promise((resolve, _reject) => {
    const newState = cloneState(state);
    newState.currentMode = mode;
    resolveAsCurrentState(resolve, newState);
  });
}

// from http://werxltd.com/wp/2010/05/13/ (cont'd next line)
// javascript-implementation-of-javas-string-hashcode-method/
function hashCode(string) {
  let hash = 0, i, len;
  if (string.length === 0) return hash;
  for (i = 0, len = string.length; i < len; i++) {
    const chr = string.charCodeAt(i);
    hash = ((hash << 5) - hash) + chr;
    hash |= 0; // Convert to 32bit integer
  }
  return hash;
}

function actionSetScript(state, { script }) {
  return new Promise((resolve, reject) => {
    const newState = cloneState(state);
    newState.script = script;
    newState.scriptHash = hashCode(script);

    Job.request(jobBuildTraits, {
      script: newState.script,
      scriptHash: newState.scriptHash
    }).then(({ validTraits, traits }) => {
      if (validTraits) {
        newState.traits = traits;
      } else {
        newState.traits = [];
      }

      resolveAsCurrentState(resolve, newState);
    }).catch(error => {
      // handle error
      console.error(`worker: error of ${error}`);
      reject(error);
    });
  });
}

function actionSetScriptId(state, { id }) {
  return new Promise((resolve, _reject) => {
    const newState = cloneState(state);
    newState.scriptId = id;
    resolveAsCurrentState(resolve, newState);
  });
}

function actionSetSelectedIndices(state, { selectedIndices }) {
  return new Promise((resolve, _reject) => {
    const newState = cloneState(state);
    newState.selectedIndices = selectedIndices || [];
    resolveAsCurrentState(resolve, newState);
  });
}

// todo: should populationSize be passed in the action?
function actionInitialGeneration(state) {
  return new Promise((resolve, reject) => {
    const newState = cloneState(state);
    Job.request(jobInitialGeneration, {
      traits: newState.traits,
      populationSize: newState.populationSize
    }).then(({ genotypes }) => {
      newState.genotypes = genotypes;
      newState.previouslySelectedGenotypes = [];
      newState.selectedIndices = [];
      resolveAsCurrentState(resolve, newState);
    }).catch(error => {
      // handle error
      console.error(`worker: error of ${error}`);
      reject(error);
    });
  });
}

function actionGalleryIsLoaded(state) {
  return new Promise((resolve, _reject) => {
    const newState = cloneState(state);
    newState.galleryLoaded = true;
    resolveAsCurrentState(resolve, newState);
  });
}

function actionShuffleGeneration(state, { rng }) {
  return new Promise((resolve, reject) => {
    const newState = cloneState(state);
    const prev = newState.previouslySelectedGenotypes;

    if (prev.length === 0) {
      actionInitialGeneration(newState).then(s => {
        resolveAsCurrentState(resolve, s);
      }).catch(error1 => {
        // handle error
        console.error(`worker: error of ${error1}`);
        reject(error1);
      });
    } else {
      Job.request(jobNewGeneration, {
        genotypes: prev,
        populationSize: newState.populationSize,
        traits: newState.traits,
        mutationRate: newState.mutationRate,
        rng
      }).then(({ genotypes }) => {
        newState.genotypes = genotypes;
        newState.selectedIndices = [];
        resolveAsCurrentState(resolve, newState);
      }).catch(error => {
        // handle error
        console.error(`worker: error of ${error}`);
        reject(error);
      });
    }
  });
}

function actionNextGeneration(state, { rng }) {
  return new Promise((resolve, reject) => {
    const newState = cloneState(state);
    const pg = newState.genotypes;
    const selectedIndices = newState.selectedIndices;
    const selectedGenos = [];
    for (let i = 0; i < selectedIndices.length; i++) {
      selectedGenos.push(pg[selectedIndices[i]]);
    }

    Job.request(jobNewGeneration, {
      genotypes: selectedGenos,
      populationSize: newState.populationSize,
      traits: newState.traits,
      mutationRate: newState.mutationRate,
      rng
    }).then(({ genotypes }) => {
      const previouslySelectedGenotypes =
            genotypes.slice(0, selectedIndices.length);

      newState.genotypes = genotypes;
      newState.previouslySelectedGenotypes = previouslySelectedGenotypes;
      newState.selectedIndices = [];

      resolveAsCurrentState(resolve, newState);
    }).catch(error => {
      // handle error
      console.error(`worker: error of ${error}`);
      reject(error);
    });
  });
}

function wrapInPromise(state) {
  return new Promise((resolve, _reject) => {
    resolveAsCurrentState(resolve, state);
  });
}

function logMode(mode) {
  let name = '';
  switch (mode) {
  case SeniMode.gallery:
    name = 'gallery';
    break;
  case SeniMode.edit:
    name = 'edit';
    break;
  case SeniMode.evolve:
    name = 'evolve';
    break;
  default:
    name = 'unknown';
    break;
  }
  console.log(`SET_MODE: ${name}`);
}

function createInitialState() {
  return {
    // the resolution of the high res image
    highResolution: [2048, 2048], //[4096, 4096],
    placeholder: 'img/spinner.gif',
    populationSize: 24,
    mutationRate: 0.1,

    currentMode: SeniMode.gallery,
    galleryLoaded: false,
    previouslySelectedGenotypes: [],
    selectedIndices: [],
    scriptId: undefined,
    script: undefined,
    scriptHash: undefined,
    genotypes: [],
    traits: []
  };
}

function createStore(initialState) {
  currentState = initialState;

  function reducer(state, action) {
    switch (action.type) {
    case 'SET_MODE':
      if (logToConsole) {
        logMode(action.mode);
      }
      return actionSetMode(state, action);
    case 'SET_SCRIPT':
      return actionSetScript(state, action);
    case 'SET_SCRIPT_ID':
      return actionSetScriptId(state, action);
    case 'SET_SELECTED_INDICES':
      return actionSetSelectedIndices(state, action);
    case 'INITIAL_GENERATION':
      return actionInitialGeneration(state);
    case 'NEXT_GENERATION':
      return actionNextGeneration(state, action);
    case 'SHUFFLE_GENERATION':
      return actionShuffleGeneration(state, action);
    case 'SET_STATE':
      if (logToConsole) {
        console.log(`SET_STATE: ${action.state}`);
      }
      return wrapInPromise(action.state);
    case 'GALLERY_LOADED':
      return actionGalleryIsLoaded(state, action);
    default:
      return wrapInPromise(state);
    }
  }

  function getState() {
    return currentState;
  }

  function dispatch(action) {
    if (logToConsole) {
      console.log(`dispatch: action = ${action.type}`);
    }
    return reducer(currentState, action);
  }

  return {
    getState,
    dispatch
  };
}


// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

// timer

const db = {};
const printPrecision = 2;

function getStats(entry) {
  if (entry.num === 0) {
    return null;
  }
  return {
    current: entry.last,
    average: (entry.sum / entry.num),
    min: entry.min,
    max: entry.max,
    num: entry.num
  };
}


function addTiming(entry, duration) {
  entry.num += 1;
  entry.sum += duration;
  entry.last = duration;
  if (duration < entry.min) {
    entry.min = duration;
  }
  if (duration > entry.max) {
    entry.max = duration;
  }
  return entry;
}

function useDBEntry(id) {
  if (!db[id]) {
    db[id] = {
      id,
      num: 0,
      sum: 0,
      last: 0,
      min: 100000,
      max: 0
    };
  }

  return db[id];
}

function startTiming() {
  const before = performance.now();
  // return the 'stop' function
  return (id, konsole) => {
    const entry = useDBEntry(id);

    const after = performance.now();
    const duration = after - before;

    addTiming(entry, duration);

    const stats = getStats(entry);

    if (konsole && stats) {
      const eid = entry.id;
      const cur = stats.current.toFixed(printPrecision);
      const avg = stats.average.toFixed(printPrecision);
      const min = stats.min.toFixed(printPrecision);
      const max = stats.max.toFixed(printPrecision);
      const num = stats.num;

      const msg1 = `${eid}: ${cur}ms `;
      const msg2 = `(Mean: ${avg}, Min: ${min}, Max: ${max} N:${num})`;

      konsole.log(msg1 + msg2);
    }
  };
}

function getTimingEntry(id) {
  return db[id];
}

// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

// SeniMode

const SeniMode = {
  gallery: 0,
  edit: 1,
  evolve: 2,
  numSeniModes: 3
};

// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------


// job

let numWorkers = 0;
const promiseWorkers = [];

class PromiseWorker {
  constructor(id, workerUrl) {
    const self = this;

    this.worker = new Worker(workerUrl);
    this.id = id;
    this.initialised = false; // true when the worker has loaded it's wasm file
    this.working = false;
    this.reject = undefined;
    this.resolve = undefined;

    this.worker.addEventListener('message', event => {
      // string data is always going to be in JSON formation
      // otherwise it will be a string encoded in an ArrayBuffer
      let status = undefined;
      let result = undefined;

      if (typeof(event.data) === 'string') {
        [status, result] = JSON.parse(event.data);

        if (status.systemInitialised) {
          self.initialised = true;
          return;
        }
      } else {                  // ArrayBuffer
        [status, result] = event.data;
      }

      if (status.logMessages && status.logMessages.length > 0) {
        console.log(status.logMessages);
      }

      if (status.error) {
        self.reject(new Error(status.error.message));
      } else {
        self.resolve(result);
      }
    });
  }

  postMessage(type, data) {
    const self = this;

    return new Promise((resolve, reject) => {
      self.resolve = resolve;
      self.reject = reject;
      self.worker.postMessage(JSON.stringify({ type, data }));
    });
  }

  employ() {
    this.working = true;
    return this;
  }

  release() {
    this.working = false;
    return this;
  }

  isInitialised() {
    return this.initialised;
  }

  isWorking() {
    return this.working;
  }

  getId() {
    return this.id;
  }
}

function findAvailableWorker() {
  return new Promise((resolve, _reject) => {
    setTimeout(function go() {
      for (let i=0;i<numWorkers;i++) {
        if (promiseWorkers[i].isInitialised() === true &&
            promiseWorkers[i].isWorking() === false) {
          resolve(promiseWorkers[i].employ());
          return;
        }
      }
      // todo?: reject if waiting too long?
      setTimeout(go, 100);
    });
  });
}

const Job = {
  request: function(type, data) {
    return new Promise((resolve, reject) => {
      let worker = undefined;

      findAvailableWorker().then(worker_ => {
        worker = worker_;
        if (logToConsole) {
          console.log(`assigning ${type} to worker ${worker.getId()}`);
        }
        return worker.postMessage(type, data);
      }).then(result => {
        if (logToConsole) {
          console.log(`result ${type} id:${worker.getId()}`);
        }
        // console.log(`job:request received: ${result}`);
        worker.release();
        resolve(result);
      }).catch(error => {
        if (worker !== undefined) {
          worker.release();
        }
        // handle error
        console.error(`worker (job:${type}): error of ${error}`);
        reject(error);
      });
    });
  },

  setup: function(numWorkersParam, path) {
    numWorkers = numWorkersParam;

    if (logToConsole) {
      console.log(`workers::path = ${path}`);
      console.log(`workers::numWorkers = ${numWorkers}`);
    }

    for (let i = 0; i < numWorkers; i++) {
      promiseWorkers[i] = new PromiseWorker(i, path);
    }
  },
};


// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

// jobTypes

const jobRender = 'RENDER';
const jobUnparse = 'UNPARSE';
const jobBuildTraits = 'BUILD_TRAITS';
const jobInitialGeneration = 'INITIAL_GENERATION';
const jobNewGeneration = 'NEW_GENERATION';
const jobGenerateHelp = 'GENERATE_HELP';
const jobSingleGenotypeFromSeed = 'SINGLE_GENOTYPE_FROM_SEED';
const jobSimplifyScript = 'SIMPLIFY_SCRIPT';


// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

let gUI = {};
let gGLRenderer = undefined;

function get(url) {
  return new Promise((resolve, reject) => {
    const req = new XMLHttpRequest();
    req.open('GET', url);

    req.onload = () => {
      // This is called even on 404 etc
      // so check the status
      if (req.status === 200) {
        // Resolve the promise with the response text
        resolve(req.response);
      } else {
        // Otherwise reject with the status text
        // which will hopefully be a meaningful error
        reject(Error(req.statusText));
      }
    };

    // Handle network errors
    req.onerror = () => {
      reject(Error('Network Error'));
    };

    // Make the request
    req.send();
  });
}

function getJSON(url) {
  return get(url).then(JSON.parse);
}

function getScriptFromEditor() {
  return gUI.editor.getValue();
}

function showButtonsFor(mode) {
  const evalBtn = document.getElementById('eval-btn');
  const evolveBtn = document.getElementById('evolve-btn');
  const renderBtn = document.getElementById('render-btn');

  const nextBtn = document.getElementById('next-btn');
  const shuffleBtn = document.getElementById('shuffle-btn');

  switch (mode) {
  case SeniMode.gallery :
    evalBtn.classList.add('hidden');
    evolveBtn.classList.add('hidden');
    renderBtn.classList.add('hidden');

    nextBtn.classList.add('hidden');
    shuffleBtn.classList.add('hidden');
    break;
  case SeniMode.edit :
    evalBtn.classList.remove('hidden');
    evolveBtn.classList.remove('hidden');
    renderBtn.classList.remove('hidden');

    nextBtn.classList.add('hidden');
    shuffleBtn.classList.add('hidden');
    break;
  case SeniMode.evolve :
    evalBtn.classList.add('hidden');
    evolveBtn.classList.add('hidden');
    renderBtn.classList.add('hidden');

    nextBtn.classList.remove('hidden');
    shuffleBtn.classList.remove('hidden');
    break;
  default:
    console.log('unknown sen mode');
    break;
  }
}

function showCurrentMode(state) {
  // show the current container, hide the others
  const containers = gUI.containers;
  const currentMode = state.currentMode;

  for (let i = 0; i < SeniMode.numSeniModes; i++) {
    containers[i].className = i === currentMode ? '' : 'hidden';
  }
  showButtonsFor(currentMode);
}

function showPlaceholderImages(state) {
  const placeholder = state.placeholder;
  const populationSize = state.populationSize;
  const phenotypes = gUI.phenotypes;

  for (let i = 0; i < populationSize; i++) {
    const imageElement = phenotypes[i].imageElement;
    imageElement.src = placeholder;
  }
}

// needs the store since imageLoadHandler rebinds store.getState()
// on every image load
//
function afterLoadingPlaceholderImages(state) {
  const allImagesLoadedSince = timeStamp => {
    const phenotypes = gUI.phenotypes;

    return phenotypes.every(phenotype => {
      const imageElement = phenotype.imageElement;
      const loaded = imageElement.getAttribute('data-image-load-timestamp');
      return loaded > timeStamp;
    });
  };

  const initialTimeStamp = performance.now();

  showPlaceholderImages(state);

  return new Promise((resolve, _) => { // todo: implement reject
    setTimeout(function go() {
      // wait until all of the placeholder load events have been received
      // otherwise there may be image sizing issues, especially with the
      // first img element
      if (allImagesLoadedSince(initialTimeStamp)) {
        resolve(state);
      } else {
        setTimeout(go, 20);
      }
    });
  });
}

// update the selected phenotypes in the evolve screen according to the
// values in selectedIndices
function updateSelectionUI(state) {
  const selectedIndices = state.selectedIndices;
  const populationSize = state.populationSize;
  const phenotypes = gUI.phenotypes;

  for (let i = 0; i < populationSize; i++) {
    const element = phenotypes[i].phenotypeElement;
    element.classList.remove('selected');
  }

  selectedIndices.forEach(i => {
    const element = phenotypes[i].phenotypeElement;
    element.classList.add('selected');
    return true;
  });
}

function renderGeometryBuffers(memory, buffers, imageElement, w, h) {
  let destWidth = undefined;
  let destHeight = undefined;
  if (w !== undefined && h !== undefined) {
    destWidth = w;
    destHeight = h;
  } else {
    destWidth = imageElement.clientWidth;
    destHeight = imageElement.clientHeight;
  }

  gGLRenderer.preDrawScene(destWidth, destHeight);

  const memoryF32 = new Float32Array(memory);

  buffers.forEach(buffer => {
    gGLRenderer.drawBuffer(memoryF32, buffer);
  });

  imageElement.src = gGLRenderer.getImageData();
}

function renderGeometryBuffersSection(memory, buffers, imageElement, w, h, section) {
  let destWidth = undefined;
  let destHeight = undefined;
  if (w !== undefined && h !== undefined) {
    destWidth = w;
    destHeight = h;
  } else {
    destWidth = imageElement.clientWidth;
    destHeight = imageElement.clientHeight;
  }

  gGLRenderer.preDrawScene(destWidth, destHeight, section);

  const memoryF32 = new Float32Array(memory);

  buffers.forEach(buffer => {
    gGLRenderer.drawBuffer(memoryF32, buffer);
  });

  imageElement.src = gGLRenderer.getImageData();
}

function renderGeneration(state) {
  return new Promise((resolve, _reject) => {
    const script = state.script;
    const scriptHash = state.scriptHash;

    const genotypes = state.genotypes;

    // TODO: stop generating  if the user has switched to edit mode
    const phenotypes = gUI.phenotypes;

    let hackTitle = scriptHash;

    const promises = [];

    const stopFn = startTiming();

    for (let i = 0; i < phenotypes.length; i++) {
      const workerJob = Job.request(jobRender, {
        script,
        scriptHash,
        genotype: genotypes[i]
      }).then(({ title , memory, buffers }) => {
        const imageElement = phenotypes[i].imageElement;
        renderGeometryBuffers(memory, buffers, imageElement);
        hackTitle = title;
      }).catch(error => {
        // handle error
        console.error(`worker: error of ${error}`);
      });

      promises.push(workerJob);
    }

    Promise.all(promises).then(() => {
      stopFn(`renderGeneration-${hackTitle}`, console);
    }).catch(error => console.error(`renderGeneration error: ${error}`));

    resolve();
  });
}

// invoked when the evolve screen is displayed after the edit screen
function setupEvolveUI(store) {
  return new Promise((resolve, reject) => {
    afterLoadingPlaceholderImages(store.getState()).then(() => {
      return store.dispatch({type: 'INITIAL_GENERATION'});
    }).then(state => {
      // render the phenotypes
      updateSelectionUI(state);
      renderGeneration(state);
      return state;
    }).then(state => {
      return resolve(state);
    }).catch(error => {
      console.error(`setupEvolveUI error: ${error}`);
      reject(error);
    });
  });
}

function showScriptInEditor(state) {
  const editor = gUI.editor;

  editor.getDoc().setValue(state.script);
  editor.refresh();
}

function renderScript(state, imageElement) {
  const stopFn = startTiming();

  Job.request(jobRender, {
    script: state.script,
    scriptHash: state.scriptHash
  }).then(({ title, memory, buffers }) => {
    renderGeometryBuffers(memory, buffers, imageElement);
    if (title === '') {
      stopFn(`renderScript`, console);
    } else {
      stopFn(`renderScript-${title}`, console);
    }
  }).catch(error => {
    // handle error
    console.error(`worker: error of ${error}`);
  });
}

// function that takes a read-only state and updates the UI
//
function updateUI(state) {
  showCurrentMode(state);

  switch (state.currentMode) {
  case SeniMode.gallery :
    break;
  case SeniMode.edit :
    showScriptInEditor(state);
    renderScript(state, gUI.renderImage);
    break;
  case SeniMode.evolve :
    // will only get here from History.restoreState
    // NOTE: the popstate event listener is handling this case
    break;
  default:
    console.log('unknown SeniMode');
    break;
  }
}

function ensureMode(store, mode) {

  if (mode === SeniMode.gallery && store.getState().galleryLoaded === false) {
    // want to show the gallery but it hasn't been loaded yet. This occurs when
    // editing a particular piece by loading it's id directly into the URL
    // e.g. http://localhost:3210/#61
    //
    return getGallery(store).then(() => {
      // gallery is loaded now so call this again to return the Promise below
      return ensureMode(store, mode);
    });
  }

  return new Promise((resolve, reject) => {
    if (store.getState().currentMode === mode) {
      resolve();
      return;
    }

    store.dispatch({type: 'SET_MODE', mode}).then(state => {
      History.pushState(state);

      if (mode === SeniMode.evolve) {
        showCurrentMode(state);
        setupEvolveUI(store).then(latestState => {
          // make sure that the history for the first evolve generation
          // has the correct genotypes
          History.replaceState(latestState);
          resolve();
        }).catch(error => console.error(`ensureMode error: ${error}`));
      } else {
        updateUI(state);
        resolve();
      }
    }).catch(error => {
      console.error(`ensureMode error: ${error}`);
      reject(error);
    });
  });
}

function addClickEvent(id, fn) {
  const element = document.getElementById(id);

  if (element) {
    element.addEventListener('click', fn);
  } else {
    console.error('cannot addClickEvent for', id);
  }
}

function getIdNumberFromDom(element, regexp) {
  let e = element;
  while (e) {
    if (!e.id) {
      e = e.parentNode;
    } else {
      const m = e.id.match(regexp);
      if (m && m.length === 2) {
        const index = Number.parseInt(m[1], 10);
        return [index, e];
      } else {
        e = e.parentNode;
      }
    }
  }
  return [-1, null];
}

// when user has clicked on a phenotype in the evolve UI,
// traverse up the card until we get to a dom element that
// contains the phenotype's index number in it's id
function getPhenoIdFromDom(element) {
  return getIdNumberFromDom(element, /pheno-(\d+)/);
}

function renderHighRes(state, genotype) {
  const container = document.getElementById('high-res-container');
  const loader = document.getElementById('high-res-loader');
  const image = document.getElementById('high-res-image');

  container.classList.remove('hidden');
  loader.classList.remove('hidden');
  image.classList.add('hidden');

  const stopFn = startTiming();

  Job.request(jobRender, {
    script: state.script,
    scriptHash: state.scriptHash,
    genotype: genotype ? genotype : undefined
  }).then(({ title, memory, buffers }) => {
    const [width, height] = state.highResolution;

    renderGeometryBuffers(memory, buffers, image, width, height);

    stopFn(`renderHighRes-${title}`, console);

    image.classList.remove('hidden');
    loader.classList.add('hidden');
  }).catch(error => {
    // handle error
    console.error(`worker: error of ${error}`);
    console.error(error);
    image.classList.remove('hidden');
    loader.classList.add('hidden');
  });
}

function renderHighResSection(state, section) {
  const container = document.getElementById('high-res-container');
  const loader = document.getElementById('high-res-loader');
  const image = document.getElementById('high-res-image');

  container.classList.remove('hidden');
  loader.classList.remove('hidden');
  image.classList.add('hidden');

  const stopFn = startTiming();

  Job.request(jobRender, {
    script: state.script,
    scriptHash: state.scriptHash,
    genotype: undefined
  }).then(({ title, memory, buffers }) => {
    const [width, height] = state.highResolution;

    renderGeometryBuffersSection(memory, buffers, image, width, height, section);

    stopFn(`renderHighResSection-${title}-${section}`, console);

    image.classList.remove('hidden');
    const link = document.getElementById('high-res-link');
    link.href = image.src;
    loader.classList.add('hidden');
  }).catch(error => {
    // handle error
    console.error(`worker: error of ${error}`);
    console.error(error);
    image.classList.remove('hidden');
    loader.classList.add('hidden');
  });
}

// updates the store's script variable and then generates the traits
// in a ww and updates the store again
//
function setScript(store, script) {
  return store.dispatch({type: 'SET_SCRIPT', script});
}

function setScriptId(store, id) {
  return store.dispatch({type: 'SET_SCRIPT_ID', id});
}

function showEditFromEvolve(store, element) {
  return new Promise((resolve, reject) => {
    const [index, _] = getPhenoIdFromDom(element);
    if (index !== -1) {
      const state = store.getState();
      const genotypes = state.genotypes;

      Job.request(jobUnparse, {
        script: state.script,
        scriptHash: state.scriptHash,
        genotype: genotypes[index]
      }).then(({ script }) => {
        setScript(store, script).then(() => {
          return ensureMode(store, SeniMode.edit);
        }).then(resolve).catch(e => {
          // handle error
          console.error(`worker: error of ${e}`);
          reject(e);
        });
      }).catch(error => {
        // handle error
        console.error(`worker: error of ${error}`);
        reject(error);
      });
    } else {
      resolve();
    }
  });
}

function onNextGen(store) {
  // get the selected genotypes for the next generation
  const populationSize = store.getState().populationSize;
  const phenotypes = gUI.phenotypes;
  const selectedIndices = [];

  for (let i = 0; i < populationSize; i++) {
    const element = phenotypes[i].phenotypeElement;
    if (element.classList.contains('selected')) {
      selectedIndices.push(i);
    }
  }

  const command = {type: 'SET_SELECTED_INDICES', selectedIndices};
  store.dispatch(command).then(state => {
    if (selectedIndices.length === 0) {
      // no phenotypes were selected
      return undefined;
    }

    // update the last history state
    History.replaceState(state);

    showPlaceholderImages(state);

    return store.dispatch({type: 'NEXT_GENERATION', rng: 4242});
  }).then(state => {
    if (state === undefined) {
      return;
    }

    History.pushState(state);
    // render the genotypes
    updateSelectionUI(state);
    renderGeneration(state);
  }).catch(error => {
    // handle error
    console.error(`error of ${error}`);
  });
}

function createPhenotypeElement(id, placeholderImage) {
  const container = document.createElement('div');

  container.className = 'card-holder';
  container.id = `pheno-${id}`;
  container.innerHTML = `
      <a href="#">
        <img class="card-image phenotype"
             data-id="${id}" src="${placeholderImage}">
      </a>
      <div class="card-action">
        <a href="#" class="render left-side">Render</a>
        <a href="#" class="edit right-side">Edit</a>
      </div>`;

  return container;
}

// invoked when restoring the evolve screen from the history api
function restoreEvolveUI(store) {
  return new Promise((resolve, reject) => { // todo: implement reject
    afterLoadingPlaceholderImages(store.getState()).then(() => {
      // render the phenotypes
      updateSelectionUI(store.getState());
      return renderGeneration(store.getState());
    }).then(resolve).catch(error => {
      // handle error
      console.error(`restoreEvolveUI: error of ${error}`);
      reject(error);
    });
  });
}

function loadScriptWithId(store, id) {
  return new Promise((resolve, reject) => {
    const url = `gallery/${id}`;
    get(url).catch(() => {
      reject(Error(`cannot connect to ${url}`));
    }).then(data => {
      return setScript(store, data);
    }).then(() => {
      return setScriptId(store, id);
    }).then(() => {
      return ensureMode(store, SeniMode.edit);
    }).then(resolve).catch(error => {
      console.error(`loadScriptWithId error ${error}`);
      reject(error);
    });
  });
}

function showEditFromGallery(store, element) {
  return new Promise((resolve, reject) => {
    const [index, _] = getIdNumberFromDom(element, /gallery-item-(\d+)/);
    if (index !== -1) {
      return loadScriptWithId(store, index);
    } else {
      resolve();
    }
  });
}
/* eslint-enable no-unused-vars */

// take the height of the navbar into consideration
function resizeContainers() {
  const navbar = document.getElementById('seni-navbar');

  const edit = document.getElementById('edit-container');
  edit.style.height = `${window.innerHeight - navbar.offsetHeight}px`;

  const evolve = document.getElementById('evolve-container');
  evolve.style.height = `${window.innerHeight - navbar.offsetHeight}px`;
}

function createEditor(store, editorTextArea) {
  const blockIndent = function (editor, from, to) {
    editor.operation(() => {
      for (let i = from; i < to; ++i) {
        editor.indentLine(i, 'smart');
      }
    });
  };

  const extraKeys = {
    'Ctrl-E': () => {
      setScript(store, getScriptFromEditor()).then(state => {
        return renderScript(state, gUI.renderImage);
      }).catch(error => {
        console.error(`worker setScript error: ${error}`);
      });
      return false;
    },
    'Ctrl-I': () => {
      const editor = gUI.editor;
      const numLines = editor.doc.size;
      blockIndent(editor, 0, numLines);
      console.log(`indenting ${numLines} lines`);
      return false;
    }
  };

  return Editor.createEditor(editorTextArea, {
    theme: 'default',
    extraKeys
  });
}

function setupUI(store) {
  const d = document;
  const editorTextArea = d.getElementById('edit-textarea');

  gUI = {
    containers: [d.getElementById('gallery-container'),
                 d.getElementById('edit-container'),
                 d.getElementById('evolve-container')],
    // the top nav bar across the state
    navbar: d.getElementById('seni-navbar'),
    // the img destination that shows the rendered script in edit mode
    renderImage: d.getElementById('render-img'),
    // console CodeMirror element in the edit screen
    editor: createEditor(store, editorTextArea)
  };

  showButtonsFor(SeniMode.gallery);

  addClickEvent('home', event => {
    ensureMode(store, SeniMode.gallery);
    event.preventDefault();
  });

  addClickEvent('evolve-btn', event => {
    // get the latest script from the editor
    setScript(store, getScriptFromEditor()).then(state => {
      History.replaceState(state);
      ensureMode(store, SeniMode.evolve);
    }).catch(error => {
      // handle error
      console.error(`evolve-btn:click : error of ${error}`);
    });
    event.preventDefault();
  });

  addClickEvent('render-btn', event => {
    renderHighRes(store.getState());
    event.preventDefault();
  });

  addClickEvent('shuffle-btn', event => {
    showPlaceholderImages(store.getState());
    store.dispatch({type: 'SHUFFLE_GENERATION', rng: 11}).then(state => {
      updateSelectionUI(state);
      renderGeneration(state);
    }).catch(error => {
      // handle error
      console.error(`shuffle-btn:click : error of ${error}`);
    });
    event.preventDefault();
  });

  addClickEvent('eval-btn', event => {
    setScript(store, getScriptFromEditor()).then(state => {
      renderScript(state, gUI.renderImage);
    }).catch(error => {
      // handle error
      console.error(`eval-btn:click : error of ${error}`);
    });
    event.preventDefault();
  });

  addClickEvent('gallery-container', event => {
    const target = event.target;
    if (target.classList.contains('show-edit')) {
      showEditFromGallery(store, target).catch(error => {
        console.error(error);
      });
    }
    event.preventDefault();
  });

  addClickEvent('evolve-container', event => {
    const target = event.target;
    const [index, phenoElement] = getPhenoIdFromDom(target);

    if (target.classList.contains('render')) {
      if (index !== -1) {
        const genotypes = store.getState().genotypes;
        const genotype = genotypes[index];
        renderHighRes(store.getState(), genotype);
      }
    } else if (target.classList.contains('edit')) {
      showEditFromEvolve(store, target);
    } else {
      if (index !== -1) {
        phenoElement.classList.toggle('selected');
      }
    }
    event.preventDefault();
  });

  addClickEvent('next-btn', () => {
    onNextGen(store);
  });

  addClickEvent('high-res-link', event => {
    const image = document.getElementById('high-res-image');
    const win = window.open();
    win.document.open();
    win.document.write('<iframe src="' + image.src + '" frameborder="0" style="border:0; top:0px; left:0px; bottom:0px; right:0px; width:100%; height:100%;" allowfullscreen></iframe>');
    win.document.close();
    event.preventDefault();
  });

  addClickEvent('high-res-download', event => {
    const highResLink = document.getElementById('high-res-link');

    // remove target='_blank' and add a download attribute
    highResLink.removeAttribute('target');
    highResLink.setAttribute('download', 'seni-image.png');

    highResLink.click();

    // restore attributes
    highResLink.removeAttribute('download');
    highResLink.setAttribute('target', '_blank');

    event.preventDefault();
  });

  addClickEvent('high-res-close', event => {
    const highResContainer = document.getElementById('high-res-container');
    highResContainer.classList.add('hidden');
    event.preventDefault();
  });

  // Ctrl-D renders the next generation
  const dKey = 68;
  document.addEventListener('keydown', event => {
    if (event.ctrlKey && event.keyCode === dKey &&
        store.getState().currentMode === SeniMode.evolve) {
      event.preventDefault();
      onNextGen(store);
    }
  }, false);

  // invoked on every load event for an img tag
  const imageLoadHandler = event => {
    event.target.setAttribute('data-image-load-timestamp', event.timeStamp);
  };

  // setup the evolve-container
  const evolveGallery = document.getElementById('evolve-gallery');
  evolveGallery.innerHTML = '';

  const row = document.createElement('div');
  row.className = 'cards';
  evolveGallery.appendChild(row);

  const populationSize = store.getState().populationSize;
  const phenotypes = [];
  for (let i = 0; i < populationSize; i++) {
    const phenotypeElement = createPhenotypeElement(i, '');

    // get the image element
    const imageElement =
          phenotypeElement.getElementsByClassName('phenotype')[0];
    imageElement.addEventListener('load', imageLoadHandler, false);
    imageElement.setAttribute('data-image-load-timestamp', 0);

    row.appendChild(phenotypeElement);

    phenotypes.push({
      phenotypeElement,
      imageElement
    });
  }

  gUI.phenotypes = phenotypes;

  window.addEventListener('popstate', event => {
    if (event.state) {
      const savedState = History.restoreState(event.state);
      store.dispatch({type: 'SET_STATE', state: savedState}).then(state => {
        updateUI(state);
        if (state.currentMode === SeniMode.evolve) {
          restoreEvolveUI(store);
        }
      }).catch(error => {
        // handle error
        console.error(`SET_STATE: error of ${error}`);
      });
    } else {
      // no event.state so behave as if the user has visited
      // the '/' of the state
      ensureMode(store, SeniMode.gallery);
    }
  });

  return store;
}

function getGallery(store) {
  const createGalleryElement = galleryItem => {
    const container = document.createElement('div');

    container.className = 'card-holder';
    container.id = `gallery-item-${galleryItem.id}`;

    container.innerHTML = `
      <a href="#" class="show-edit">
        <img class="card-image show-edit"
             src="${galleryItem.image}">
      </a>
      <div class="card-action">
        <span>${galleryItem.name}</span>
      </div>`;

    return container;
  };

  return new Promise((resolve, reject) => {
    const list = document.getElementById('gallery-container');
    list.innerHTML = '';

    const row = document.createElement('div');
    row.className = 'cards';
    list.appendChild(row);

    const url = 'gallery';
    getJSON(url).then(galleryItems => {
      // gets an array of gallery items
      galleryItems.forEach(item => {
        const e = createGalleryElement(item);
        row.appendChild(e);
      });

      return store.dispatch({type: 'GALLERY_LOADED'});
    }).then(() => {
      resolve();
    }).catch(() => {
      reject(Error(`cannot connect to ${url}`));
    });
  });
}

function allocateWorkers(state) {
  const defaultNumWorkers = 4;
  let numWorkers = navigator.hardwareConcurrency || defaultNumWorkers;
  // console.log("setting numWorkers to 1");
  // let numWorkers = 1;
  if (numWorkers > state.populationSize) {
    // don't allocate more workers than necessary
    numWorkers = state.populationSize;
  }
  Job.setup(numWorkers, 'worker.js');
}

// https://developer.mozilla.org/en-US/docs/Web/Events/resize
function throttle(type, name, obj) {
  const obj2 = obj || window;
  let running = false;
  const func = () => {
    if (running) { return; }
    running = true;
    requestAnimationFrame(() => {
      obj2.dispatchEvent(new CustomEvent(name));
      running = false;
    });
  };
  obj2.addEventListener(type, func);
}

function setupResizeability() {
  // define a version of the resize event which fires less frequently
  throttle('resize', 'throttledResize');

  window.addEventListener('throttledResize', () => {
    resizeContainers();
  });

  resizeContainers();
}

function compatibilityHacks() {
  // Safari doesn't have Number.parseInt (yet)
  // Safari is the new IE
  if (Number.parseInt === undefined) {
    Number.parseInt = parseInt;
  }
}

function main() {
  setupResizeability();

  const state = createInitialState();
  const store = createStore(state);

  allocateWorkers(state);

  const canvasElement = document.getElementById('render-canvas');
  gGLRenderer = new GLRenderer(canvasElement);

  gGLRenderer.loadTexture('img/texture.png').then(() => {
    setupUI(store);

    const matched = window.location.hash.match(/^\#(\d+)/);
    if (window.location.pathname === '/' && matched) {
      const id = parseInt(matched[1], 10);
      return loadScriptWithId(store, id);
    } else {
      return ensureMode(store, SeniMode.gallery);
    }
  }).catch(error => console.error(error));
}

document.addEventListener('DOMContentLoaded', () => {
  compatibilityHacks();
  main();
});
