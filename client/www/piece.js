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

  drawBuffer(memory, buffer) {
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

    // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray#Syntax
    // a new typed array view is created that views the specified ArrayBuffer
    const gbuf = new Float32Array(memory, buffer.geo_ptr, buffer.geo_len);

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

/// /seni/Util

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

let gGLRenderer = undefined;


let gLogDebug = false;
let gTimeoutId = undefined;
let gSlideshowDelay = 5000;
let gDemandCanvasSize = 500;
let gMode = "normal";           // normal | slideshow
let gActiveImageElement = 0;
let gNumTransitions = 0;        // reset after every mode switch


function logDebug(msg) {
  if (gLogDebug) {
    const op0 = getRequiredElement('piece-img-0').style.opacity;
    const op1 = getRequiredElement('piece-img-1').style.opacity;

    console.log(`${msg} ${gMode} gNumTransitions: ${gNumTransitions} img-0 opacity: ${op0}, img-1 opacity: ${op1} activeImageElement: ${gActiveImageElement}`);
  }
}

function updatePieceDimensions(pieceImg, canvas, w, h) {
  pieceImg.style.top = canvas.offsetTop + "px";
  pieceImg.style.left = canvas.offsetLeft + "px";
  pieceImg.width = w;
  pieceImg.height = h;
}

function updatePieceData(pieceImg) {
  pieceImg.src = gGLRenderer.getImageData();
}

function displayOnImageElements() {
  const canvas = getRequiredElement('piece-canvas');
  const pieceImg0 = getRequiredElement('piece-img-0');
  const pieceImg1 = getRequiredElement('piece-img-1');

  if (gNumTransitions === 0) {
    // have just switched modes, so make sure the images are correctly positioned
    setOpacity('piece-img-0', 1);
    updatePieceDimensions(pieceImg0, canvas, gDemandCanvasSize, gDemandCanvasSize);
    updatePieceDimensions(pieceImg1, canvas, gDemandCanvasSize, gDemandCanvasSize);
  }

  if (gActiveImageElement === 0) {
    updatePieceData(pieceImg0);
    if (gNumTransitions > 0) {
      if (gMode === "normal") {
        addClass('piece-img-1', 'seni-fade-out');
      } else {
        addClass('piece-img-1', 'seni-fade-out-slideshow');
      }
    }

  } else {
    updatePieceData(pieceImg1);
    if (gNumTransitions > 0) {
      if (gMode === "normal") {
        addClass('piece-img-1', 'seni-fade-in');
      } else {
        addClass('piece-img-1', 'seni-fade-in-slideshow');
      }
    }
  }

  gActiveImageElement = 1 - gActiveImageElement;

  logDebug("displayOnImageElements");
}

function renderBuffers(memory, buffers, w, h) {
  // this will update the size of the piece-canvas element
  gGLRenderer.preDrawScene(w, h);

  buffers.forEach(buffer => {
    gGLRenderer.drawBuffer(memory, buffer);
  });

  displayOnImageElements();
}

function renderScript(config) {
  return Job.request(jobRender, config)
    .then(({ memory, buffers }) => {
      renderBuffers(memory, buffers, gDemandCanvasSize, gDemandCanvasSize);
    }).catch(error => {
      // handle error
      console.error(`worker: error of ${error}`);
    });
}

function buildTraits(config) {
  return Job.request(jobBuildTraits, config);
}

function buildGenotype(config) {
  return Job.request(jobSingleGenotypeFromSeed, config);
}

function unparse(config) {
  return Job.request(jobUnparse, config);
}

function getSeedValue(element) {
  const res = parseInt(element.value, 10);
  return res;
}

function fetchScript(id) {
  return fetch(`/gallery/${id}`).then(response => response.text());
}

function getRequiredElement(id) {
  const element = document.getElementById(id);
  if (!element) {
    console.error(`required element ${id} not found in dom`);
  }
  return element;
}

function showSimplifiedScript(fullScript) {
  Job.request(jobSimplifyScript, {
    script: fullScript
  }).then(({ script }) => {
    const simplifiedScriptElement =
          getRequiredElement('piece-simplified-script');
    console.log(fullScript);
    console.log(script);
    simplifiedScriptElement.textContent = script;
  }).catch(error => {
    // handle error
    console.error(`worker: error of ${error}`);
  });
}

function useLargeCanvas() {
  gDemandCanvasSize = window.innerWidth < window.innerHeight ? window.innerWidth : window.innerHeight;
  gDemandCanvasSize *= 0.9;
}

function useNormalCanvas() {
  gDemandCanvasSize = 500;
}

function addClass(id, clss) {
  const e = getRequiredElement(id);
  e.classList.add(clss);
}

function removeClass(id, clss) {
  const e = getRequiredElement(id);
  e.classList.remove(clss);
}

function showId(id) {
  removeClass(id, 'seni-hide');
}

function hideId(id) {
  addClass(id, 'seni-hide');
}

function setOpacity(id, opacity) {
  const e = getRequiredElement(id);
  e.style.opacity = opacity;
}

function performSlideshow() {
  gNumTransitions += 1;
  const scriptElement = getRequiredElement('piece-script');
  const seedElement = getRequiredElement('piece-seed');

  const scriptHash = hashCode('whatever');

  const script = scriptElement.textContent;
  const originalScript = script.slice();

  const newSeed = Math.random() * (1 << 30);
  seedElement.value = parseInt(newSeed, 10);

  const seedValue = getSeedValue(seedElement);
  buildTraits({ script: originalScript, scriptHash })
    .then(({ traits }) => buildGenotype({ traits, seed: seedValue }))
    .then(({ genotype }) => {
      const config = { script: originalScript, scriptHash };
      if (seedValue !== 0) {
        config.genotype = genotype;
      }
      return renderScript(config);
    })
    .then(() => {
      gTimeoutId = window.setTimeout(performSlideshow, gSlideshowDelay);
    })
    .catch(error => {
      console.error('performSlideshow error');
      console.error(error);
    });
}

// returns true if the mode was actually changed
//
function setMode(newMode) {
  if (newMode === "normal" && gMode !== "normal") {
    gMode = "normal";
    window.clearTimeout(gTimeoutId); // stop the slideshow
    addClass('piece-content', 'piece-content-wrap');
    useNormalCanvas();
    showId('header');
    showId('seni-piece-controls');
    showId('code-content-wrap');
    showId('seni-title');
    showId('seni-date');
    showId('piece-hideable-for-slideshow');
    removeClass('piece-canvas-container', 'seni-centre-canvas');

    setOpacity('piece-img-0', 0);
    setOpacity('piece-img-1', 0);
    gActiveImageElement = 0;
    gNumTransitions = 0;


    const originalButton = getRequiredElement('piece-eval-original');
    const scriptElement = getRequiredElement('piece-script');

    const scriptHash = hashCode('whatever');
    const script = scriptElement.textContent;
    const originalScript = script.slice();

    renderScript({ script: originalScript, scriptHash });
    scriptElement.textContent = originalScript;
    showSimplifiedScript(originalScript);
    originalButton.disabled = true;

    return true;
  } else if (newMode === "slideshow" && gMode !== "slideshow") {
    gMode = "slideshow";

    removeClass('piece-content', 'piece-content-wrap');
    useLargeCanvas();
    hideId('header');
    hideId('seni-piece-controls');
    hideId('code-content-wrap');
    hideId('seni-title');
    hideId('seni-date');
    hideId('piece-hideable-for-slideshow');
    addClass('piece-canvas-container', 'seni-centre-canvas');

    setOpacity('piece-img-0', 0);
    setOpacity('piece-img-1', 0);
    gActiveImageElement = 0;
    gNumTransitions = 0;

    gTimeoutId = window.setTimeout(performSlideshow, 1500);
    return true;
  }
  return false;
}

function animationEndListener(event, id) {
  if (event.animationName === 'senifadeout') {
    removeClass(id, 'seni-fade-out');
    removeClass(id, 'seni-fade-out-slideshow');
    setOpacity(id, 0);
  }

  if (event.animationName === 'senifadein') {
    removeClass(id, 'seni-fade-in');
    removeClass(id, 'seni-fade-in-slideshow');
    setOpacity(id, 1);
  }
}

function animationEndListener1(event) {
  animationEndListener(event, 'piece-img-1');
}

function compatibilityHacks() {
  // Safari doesn't have Number.parseInt (yet)
  // Safari is the new IE
  if (Number.parseInt === undefined) {
    Number.parseInt = parseInt;
  }
}

function main() {

  const texturePathElement = getRequiredElement('piece-texture-path');
  const workerPathElement = getRequiredElement('piece-worker-path');

  Job.setup(2, workerPathElement.textContent);

  const originalButton = getRequiredElement('piece-eval-original');
  const evalButton = getRequiredElement('piece-eval');
  const slideshowButton = getRequiredElement('piece-eval-slideshow');
  const scriptElement = getRequiredElement('piece-script');
  const canvasElement = getRequiredElement('piece-canvas');
  const canvasImageElement0 = getRequiredElement('piece-img-0');
  const canvasImageElement1 = getRequiredElement('piece-img-1');
  const seedElement = getRequiredElement('piece-seed');

  canvasImageElement1.addEventListener("animationend", animationEndListener1, false);
  setOpacity('piece-img-0', 0);
  setOpacity('piece-img-1', 0);

  const LOAD_FOR_SENI_APP_GALLERY = true;

  if (LOAD_FOR_SENI_APP_GALLERY === false) {
    // not really required, hack to load in other pieces
    const loadIdElement = getRequiredElement('piece-load-id');
    loadIdElement.addEventListener('change', event => {
      console.log('loadidelement');
      const iVal = parseInt(event.target.value, 10);

      fetchScript(iVal).then(code => {
        script = code;
        originalScript = script.slice();
        scriptElement.textContent = script;
        showSimplifiedScript(script);
        return renderScript({ script, scriptHash });
      }).catch(error => console.error(error));
    });
  }

  gMode = "normal";

  const scriptHash = hashCode('whatever');

  gGLRenderer = new GLRenderer(canvasElement);

  const script = scriptElement.textContent;
  const originalScript = script.slice();
  showSimplifiedScript(script);

  logDebug("init");

  gGLRenderer.loadTexture(texturePathElement.textContent)
    .then(() => renderScript({ script, scriptHash }))
    .catch(error => console.error(error));

  originalButton.addEventListener('click', () => {
    setMode("normal");
    renderScript({ script: originalScript, scriptHash });
    scriptElement.textContent = originalScript;
    showSimplifiedScript(originalScript);
    originalButton.disabled = true;
  });

  slideshowButton.addEventListener('click', () => {
    setMode("slideshow");
    renderScript({ script: originalScript, scriptHash });
    scriptElement.textContent = originalScript;
    showSimplifiedScript(originalScript);
    originalButton.disabled = true;
  });

  evalButton.addEventListener('click', () => {
    gNumTransitions += 1;
    originalButton.disabled = false;
    const newSeed = Math.random() * (1 << 30);
    seedElement.value = parseInt(newSeed, 10);

    const seedValue = getSeedValue(seedElement);
    buildTraits({ script: originalScript, scriptHash })
      .then(({ traits }) => buildGenotype({ traits, seed: seedValue }))
      .then(({ genotype }) => {
        const config = { script: originalScript, scriptHash };
        if (seedValue !== 0) {
          config.genotype = genotype;
        }
        renderScript(config);

        return unparse({ script: originalScript, genotype });
      })
      .then(({ script }) => {
        scriptElement.textContent = script;
        showSimplifiedScript(script);
      })
      .catch(error => {
        console.error('piece-eval click error');
        console.error(error);
      });
  });

  canvasImageElement1.addEventListener('click', () => {
    setMode("normal");
  });

  const escapeKey = 27;
  document.addEventListener('keydown', event => {
    if (event.keyCode === escapeKey && gMode === 'slideshow') {
      setMode('normal');
      event.preventDefault();
    }
  }, false);
}

document.addEventListener('DOMContentLoaded', () => {
  compatibilityHacks();
  main();
});
