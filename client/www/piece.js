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

const Matrix = {
  create,
  ortho,
};

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

  copyImageDataTo(elem) {
    return new Promise((resolve, reject) => {
      try {
        this.glDomElement.toBlob(blob => {
          elem.src = window.URL.createObjectURL(blob);
          return resolve();
        });
      } catch (error) {
        return reject(error);
      }
    });
  }

  localDownload(filename) {
    this.glDomElement.toBlob(function(blob) {

      const url = window.URL.createObjectURL(blob);

      let element = document.createElement('a');
      element.setAttribute('href', url);
      // element.setAttribute('target', '_blank');
      element.setAttribute('download', filename);

      element.style.display = 'none';
      document.body.appendChild(element);

      element.click();

      document.body.removeChild(element);

    });
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

  setupBuffer(memory, buffer) {
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
  }

  drawBuffer(buffer) {
    const gl = this.gl;

    const vertexItemSize = 2;
    const colourItemSize = 4;
    const textureItemSize = 2;

    const totalSize = (vertexItemSize + colourItemSize + textureItemSize);

    gl.drawArrays(gl.TRIANGLE_STRIP, 0, buffer.geo_len / totalSize);
  }

  drawBufferPartial(buffer, first, count) {
    const gl = this.gl;

    const vertexItemSize = 2;
    const colourItemSize = 4;
    const textureItemSize = 2;

    const totalSize = (vertexItemSize + colourItemSize + textureItemSize);

    gl.drawArrays(gl.TRIANGLE_STRIP, first / totalSize, count / totalSize);
  }
}

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
// job

let numWorkers = 0;
const promiseWorkers = [];

class PromiseWorker {
  constructor(id, workerUrl) {
    const self = this;

    // <2019-04-13 Sat>
    // would be good to use module syntax in the worker.js file.
    // this would enable a more modern way of instantiating the wasm module
    // see https://rustwasm.github.io/docs/wasm-bindgen/examples/without-a-bundler.html?highlight=export,memory#without-a-bundler
    //
    // This should be enabled with:
    // this.worker = new Worker(workerUrl, {type:'module'});
    //
    // unfortunately there is a bug in Chromium preventing this:
    // https://bugs.chromium.org/p/chromium/issues/detail?id=680046
    // original info from:
    // https://www.codedread.com/blog/archives/2017/10/19/web-workers-can-be-es6-modules-too/

    this.worker = new Worker(workerUrl);
    this.id = id;
    this.initialised = false; // true when the worker has loaded it's wasm file
    this.working = false;
    this.reject = undefined;
    this.resolve = undefined;

    this.worker.addEventListener('message', event => {

      const [status, result] = event.data;

      if (status.systemInitialised) {
        self.initialised = true;
        console.log(`worker ${self.id} initialised`);
        return;
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

      if (type === jobRender_2_ReceiveBitmapData) {
        // ImageData is not a transferrable type
        self.worker.postMessage({ type, data });
      } else {
        self.worker.postMessage({ type, data });
      }
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
  request: async function(type, data, worker_id) {
    try {
      let worker = undefined;
      if (worker_id === undefined) {
        worker = await findAvailableWorker();
        if (logToConsole) {
          console.log(`assigning ${type} to worker ${worker.getId()}`);
        }
      } else {
        worker = promiseWorkers[worker_id];
        if (logToConsole) {
          console.log(`explicitly assigning ${type} to worker ${worker.getId()}`);
        }
      }

      const result = await worker.postMessage(type, data);
      if (logToConsole) {
        console.log(`result ${type} id:${worker.getId()}`);
      }

      if(!data.__retain) {
        worker.release();
      }

      result.__worker_id = worker.getId();

      return result;
    } catch (error) {
      // handle error
      console.error(`worker (job:${type}): error of ${error}`);
      return undefined;         // ???
    }
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
// jobTypes

const jobRender_1_Compile = 'RENDER_1_COMPILE';
const jobRender_2_ReceiveBitmapData = 'RENDER_2_RECEIVEBITMAPDATA';
const jobRender_3_RenderPackets = 'RENDER_3_RENDERPACKETS';
const jobUnparse = 'UNPARSE';
const jobBuildTraits = 'BUILD_TRAITS';
const jobInitialGeneration = 'INITIAL_GENERATION';
const jobNewGeneration = 'NEW_GENERATION';
const jobGenerateHelp = 'GENERATE_HELP';
const jobSingleGenotypeFromSeed = 'SINGLE_GENOTYPE_FROM_SEED';
const jobSimplifyScript = 'SIMPLIFY_SCRIPT';

// --------------------------------------------------------------------------------

let gGLRenderer = undefined;

let gLogDebug = false;
let gTimeoutId = undefined;
let gSlideshowDelay = 5000;
let gDemandCanvasSize = 500;
let gMode = "normal";           // normal | slideshow
let gActiveImageElement = 0;
let gNumTransitions = 0;        // reset after every mode switch

let gSketchMemory = undefined;
let gSketchBuffers = undefined;
let gSketchBufferIndex = 0;
let gSketchGeoIndex = 0;
let gSketchAmount = 0;
let gSketchTimeStart = 0;
// note: desiredDuration will always be a slight underestimation of actual duration
// as some transitions aren't going to be using the full amount of vertices
let gSketchDesiredDuration = 40;

function logDebug(msg) {
  if (gLogDebug) {
    const op0 = getRequiredElement('piece-img-0').style.opacity;
    const op1 = getRequiredElement('piece-img-1').style.opacity;

    console.log(`${msg} ${gMode} gNumTransitions: ${gNumTransitions} img-0 opacity: ${op0}, img-1 opacity: ${op1} activeImageElement: ${gActiveImageElement}`);
  }
}

function updatePieceDimensions(pieceImg, canvas, w, h) {
  // console.log(`top: ${canvas.offsetTop} left: ${canvas.offsetLeft} width: ${w} height: ${h}`);

  pieceImg.style.top = canvas.offsetTop + "px";
  pieceImg.style.left = canvas.offsetLeft + "px";
  pieceImg.width = w;
  pieceImg.height = h;
}

async function displayOnImageElements() {
  if (gActiveImageElement === 0) {
    const pieceImg0 = getRequiredElement('piece-img-0');
    await gGLRenderer.copyImageDataTo(pieceImg0);

    if (gNumTransitions > 0) {
      if (gMode === "normal") {
        addClass('piece-img-1', 'seni-fade-out');
      } else if (gMode === "slideshow") {
        addClass('piece-img-1', 'seni-fade-out-slideshow');
      } else if (gMode === "sketch") {
        addClass('piece-img-1', 'seni-fade-out-sketch');
      }
    }
  } else {
    const pieceImg1 = getRequiredElement('piece-img-1');
    await gGLRenderer.copyImageDataTo(pieceImg1);
    if (gNumTransitions > 0) {
      if (gMode === "normal") {
        addClass('piece-img-1', 'seni-fade-in');
      } else if (gMode === "slideshow") {
        addClass('piece-img-1', 'seni-fade-in-slideshow');
      } else if (gMode === "sketch") {
        addClass('piece-img-1', 'seni-fade-in-sketch');
      }
    }
  }

  gActiveImageElement = 1 - gActiveImageElement;

  logDebug("displayOnImageElements");
}

async function renderBuffers(memory, buffers, w, h) {
  // this will update the size of the piece-canvas element
  gGLRenderer.preDrawScene(w, h);

  buffers.forEach(buffer => {
    gGLRenderer.setupBuffer(memory, buffer);
    gGLRenderer.drawBuffer(buffer);
  });

  await displayOnImageElements();
}

// based on code from:
// https://hackernoon.com/functional-javascript-resolving-promises-sequentially-7aac18c4431e
function sequentialPromises(funcs) {
  return funcs.reduce((promise, func) =>
    promise.then(result => func().then(Array.prototype.concat.bind(result))),
    Promise.resolve([]));
}

// todo: is this the best way of getting image data for a web worker?
// is there a way for the webworker to do this without having to interact with the DOM?
// note: don't call this on a sequence of bitmaps
function loadBitmapImageData(url) {
  return new Promise(function(resolve, reject) {
    const element = document.getElementById('bitmap-canvas');
    const context = element.getContext('2d');
    const img = new Image();

    img.onload = () => {
      element.width = img.width;
      element.height = img.height;

      context.drawImage(img, 0, 0);

      const imageData = context.getImageData(0, 0, element.width, element.height);

      resolve(imageData);
    };
    img.onerror = () => {
      reject();
    };

    img.src = normalize_bitmap_url(url);
  });
}

function normalize_bitmap_url(url) {
  // todo: this should:
  // 1. do nothing if the url is a valid url
  // 2. if it's just a filename, prefix the img/ path (specific to seni web app)
  return "img/" + url;
}

async function renderJob(parameters) {
  // 1. compile the program in a web worker
  // 2. (retain the id for this worker)
  // 3. after compilation, the worker will return a list of bitmaps that are
  //    required by the program and are not in the web worker's bitmap-cache
  // 4. sequentially load in the bitmaps and send their data to the worker
  // 5. can now request a render which will return the render packets

  // request a compile job but make sure to retain the worker as it will be performing the rendering
  //
  parameters.__retain = true;
  const { bitmapsToTransfer, __worker_id } = await Job.request(jobRender_1_Compile, parameters);

  // convert each bitmap path to a function that returns a promise
  //
  const bitmap_loading_funcs = bitmapsToTransfer.map(filename => async () => {
    const imageData = await loadBitmapImageData(filename);
    console.log(`worker ${__worker_id}: bitmap request: ${filename}`);
    // make an explicit job request to the same worker
    return Job.request(jobRender_2_ReceiveBitmapData, { filename, imageData, __retain: true }, __worker_id);
  });

  // seqentially execute the promises that load in bitmaps and send the bitmap data to a particular worker
  //
  await sequentialPromises(bitmap_loading_funcs);

  // now make an explicit job request to the same worker that has recieved the bitmap data
  // note: no __retain as we want the worker to be returned to the available pool
  const renderPacketsResult = await Job.request(jobRender_3_RenderPackets, {}, __worker_id);

  return renderPacketsResult;
}

async function renderScript(parameters) {
  let { title, memory, buffers } = await renderJob(parameters);
  await renderBuffers(memory, buffers, gDemandCanvasSize, gDemandCanvasSize);
}

function getSeedValue(element) {
  const res = parseInt(element.value, 10);
  return res;
}

async function fetchScript(id) {
  const response = await fetch(`/gallery/${id}`);
  return response.text();
}

function getRequiredElement(id) {
  const element = document.getElementById(id);
  if (!element) {
    console.error(`required element ${id} not found in dom`);
  }
  return element;
}

async function showSimplifiedScript(fullScript) {
  const { script } = await Job.request(jobSimplifyScript, {
    script: fullScript
  });

  const simplifiedScriptElement = getRequiredElement('piece-simplified-script');
  //    console.log(fullScript);
  //    console.log(script);
  simplifiedScriptElement.textContent = script;
}

function updatePiecesUsingCanvasSize() {
  const canvas = getRequiredElement('piece-canvas');
  const pieceImg0 = getRequiredElement('piece-img-0');
  const pieceImg1 = getRequiredElement('piece-img-1');
  updatePieceDimensions(pieceImg0, canvas, gDemandCanvasSize, gDemandCanvasSize);
  updatePieceDimensions(pieceImg1, canvas, gDemandCanvasSize, gDemandCanvasSize);
}

function useLargeCanvas() {
  gDemandCanvasSize = window.innerWidth < window.innerHeight ? window.innerWidth : window.innerHeight;
  gDemandCanvasSize *= 0.9;

  updatePiecesUsingCanvasSize();
}

function useNormalCanvas() {
  gDemandCanvasSize = 500;
  updatePiecesUsingCanvasSize();
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

async function performSlideshow() {
  gNumTransitions += 1;
  const scriptElement = getRequiredElement('piece-script');
  const seedElement = getRequiredElement('piece-seed');

  const scriptHash = hashCode('whatever');

  const script = scriptElement.textContent;
  const originalScript = script.slice();

  const newSeed = Math.random() * (1 << 30);
  seedElement.value = parseInt(newSeed, 10);

  const seedValue = getSeedValue(seedElement);

  const { traits } = await Job.request(jobBuildTraits, { script: originalScript, scriptHash });
  const { genotype } = await Job.request(jobSingleGenotypeFromSeed, { traits, seed: seedValue });

  const config = { script: originalScript, scriptHash };
  if (seedValue !== 0) {
    config.genotype = genotype;
  }

  await renderScript(config);

  gTimeoutId = window.setTimeout(performSlideshow, gSlideshowDelay);
}

function getCSSAnimationDuration(className) {
  const indyioCSSStylesheet = 0; // note: update this if more than one stylesheet is used

  const styleSheet = document.styleSheets[indyioCSSStylesheet];

  let cssRules = undefined;
  for(let i = 0; i < styleSheet.cssRules.length; i++) {
    if (styleSheet.cssRules[i].selectorText === className) {
      cssRules = styleSheet.cssRules[i];
      return parseFloat(cssRules.style.animationDuration);
    }
  }
  return undefined;
}

async function performSketch() {
  const scriptElement = getRequiredElement('piece-script');
  const seedElement = getRequiredElement('piece-seed');

  const scriptHash = hashCode('whatever');

  const script = scriptElement.textContent;
  const originalScript = script.slice();

  const config = { script: originalScript, scriptHash };

  let { title, memory, buffers } = await renderJob(config);

  gSketchMemory = memory;
  gSketchBuffers = buffers;
  gSketchBufferIndex = 0;
  gSketchGeoIndex = 0;

  let i = 0;
  buffers.forEach(b => {
    console.log(`buffer ${i} size: ${b.geo_len}`);
    i += 1;
  });

  let numElements = buffers.reduce((acc, buffer) => acc + buffer.geo_len, 0);
  let numVertices = numElements / 8;

  let cssTimeFadeIn = getCSSAnimationDuration(".seni-fade-in-sketch");
  let cssTimeFadeOut = getCSSAnimationDuration(".seni-fade-out-sketch");
  let transitionTime = cssTimeFadeIn > cssTimeFadeOut ? cssTimeFadeIn : cssTimeFadeOut;
  // the number of transitions that the piece should be divided into
  let numTransitions = gSketchDesiredDuration / transitionTime;
  let vertsPerTransition = numVertices / numTransitions;


  // divide numVertices by the animation length and round up to be divisible by 8
  // gSketchAmount = 2000 * 8;
  gSketchAmount = Math.round(vertsPerTransition) * 8;
  console.log(`gSketchAmount = ${gSketchAmount}`);


  const canvas = getRequiredElement('piece-canvas');
  const pieceImg0 = getRequiredElement('piece-img-0');
  const pieceImg1 = getRequiredElement('piece-img-1');

  gGLRenderer.preDrawScene(gDemandCanvasSize, gDemandCanvasSize);
  await gGLRenderer.copyImageDataTo(pieceImg0);
  await gGLRenderer.copyImageDataTo(pieceImg1);

  if(buffers.length > 0) {
    gSketchTimeStart = performance.now();
    gGLRenderer.setupBuffer(memory, buffers[0]);
    gTimeoutId = window.setTimeout(animateSketch);
  }
}

// the animationEndListener is responsible for calling animateSketch.
// This way we can specify the length of each fade in css
async function animateSketch() {
  let currentBuffer = gSketchBuffers[gSketchBufferIndex];

  if (gSketchGeoIndex + gSketchAmount < currentBuffer.geo_len) {
    // can draw geometry from the current buffer
    gGLRenderer.drawBufferPartial(currentBuffer, gSketchGeoIndex, gSketchAmount);
    console.log(`animateSketch ${gSketchBufferIndex} gSketchGeoIndex: ${gSketchGeoIndex} gSketchAmount: ${gSketchAmount} sum: ${gSketchGeoIndex + gSketchAmount}`);
    gSketchGeoIndex += gSketchAmount;
  } else {
    // render the remaining geometry
    const remaining = currentBuffer.geo_len - gSketchGeoIndex;
    gGLRenderer.drawBufferPartial(currentBuffer, gSketchGeoIndex, remaining);
    console.log(`animateSketch ${gSketchBufferIndex} gSketchGeoIndex: ${gSketchGeoIndex} remaining: ${remaining} sum: ${gSketchGeoIndex + remaining}`);
    console.log("");

    // move onto the next buffer
    gSketchBufferIndex += 1;
    gSketchGeoIndex = 0;

    if (gSketchBufferIndex < gSketchBuffers.length) {
      gGLRenderer.setupBuffer(gSketchMemory, gSketchBuffers[gSketchBufferIndex]);
    } else {
      // finished the animation
      let endTime = performance.now();
      let duration = endTime - gSketchTimeStart;
      console.log(`finished. duration: ${duration}`);
      return;
    }
  }

  gNumTransitions += 1;

  await displayOnImageElements();
}


function resetImageElements() {
  setOpacity('piece-img-1', 0);
  gActiveImageElement = 0;
  gNumTransitions = 0;
}

function styleForNormalPiece() {
  addClass('piece-content', 'piece-content-wrap');
  showId('header');
  showId('seni-piece-controls');
  showId('code-content-wrap');
  showId('seni-title');
  showId('seni-date');
  showId('piece-hideable-for-slideshow');
  removeClass('piece-canvas-container', 'seni-centre-canvas');

  useNormalCanvas();

  resetImageElements();
}

function styleForLargePiece() {
  removeClass('piece-content', 'piece-content-wrap');
  hideId('header');
  hideId('seni-piece-controls');
  hideId('code-content-wrap');
  hideId('seni-title');
  hideId('seni-date');
  hideId('piece-hideable-for-slideshow');
  addClass('piece-canvas-container', 'seni-centre-canvas');

  useLargeCanvas();

  resetImageElements();
}


function updateToMode(newMode) {
  if (gMode === newMode) {
    return false;
  }

  gMode = newMode;
  return true;
}

// returns true if the mode was actually changed
//
async function setMode(newMode) {
  console.log("setMode " + newMode);
  if (newMode === "normal" && updateToMode(newMode)) {
    window.clearTimeout(gTimeoutId); // stop the slideshow/sketch

    styleForNormalPiece();

    const originalButton = getRequiredElement('piece-eval-original');
    const scriptElement = getRequiredElement('piece-script');

    const scriptHash = hashCode('whatever');
    const script = scriptElement.textContent;
    const originalScript = script.slice();

    await renderScript({ script: originalScript, scriptHash });
    scriptElement.textContent = originalScript;
    await showSimplifiedScript(originalScript);
    originalButton.disabled = true;

    return true;
  } else if (newMode === "slideshow" && updateToMode(newMode)) {
    styleForLargePiece();
    gTimeoutId = window.setTimeout(performSlideshow, 1500);
    return true;
  } else if (newMode === "sketch" && updateToMode(newMode)) {
    styleForLargePiece();
    await performSketch();
    return true;
  }

  return false;
}

function animationEndListener(event, id) {
  if (event.animationName === 'senifadeout') {
    removeClass(id, 'seni-fade-out');
    removeClass(id, 'seni-fade-out-slideshow');
    removeClass(id, 'seni-fade-out-sketch');
    setOpacity(id, 0);
  }

  if (event.animationName === 'senifadein') {
    removeClass(id, 'seni-fade-in');
    removeClass(id, 'seni-fade-in-slideshow');
    removeClass(id, 'seni-fade-in-sketch');
    setOpacity(id, 1);
  }

  if (gMode === "sketch") {
    gTimeoutId = window.setTimeout(animateSketch);
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

async function main() {

  const texturePathElement = getRequiredElement('piece-texture-path');
  const workerPathElement = getRequiredElement('piece-worker-path');

  Job.setup(2, workerPathElement.textContent);

  const originalButton = getRequiredElement('piece-eval-original');
  const evalButton = getRequiredElement('piece-eval');
  const slideshowButton = getRequiredElement('piece-eval-slideshow');
  const sketchButton = getRequiredElement('piece-eval-sketch');
  const scriptElement = getRequiredElement('piece-script');
  const canvasElement = getRequiredElement('piece-canvas');
  const canvasImageElement0 = getRequiredElement('piece-img-0');
  const canvasImageElement1 = getRequiredElement('piece-img-1');
  const seedElement = getRequiredElement('piece-seed');

  canvasImageElement1.addEventListener("animationend", animationEndListener1, false);
  setOpacity('piece-img-1', 0);

  const LOAD_FOR_SENI_APP_GALLERY = true;

  if (LOAD_FOR_SENI_APP_GALLERY === false) {
    // not really required, hack to load in other pieces
    const loadIdElement = getRequiredElement('piece-load-id');
    loadIdElement.addEventListener('change', async event => {
      console.log('loadidelement');
      const iVal = parseInt(event.target.value, 10);

      const code = await fetchScript(iVal);

      script = code;
      originalScript = script.slice();
      scriptElement.textContent = script;
      await showSimplifiedScript(script);
      await renderScript({ script, scriptHash });

    });
  }

  gMode = "normal";
  useNormalCanvas();

  const scriptHash = hashCode('whatever');

  gGLRenderer = new GLRenderer(canvasElement);

  const script = scriptElement.textContent;
  const originalScript = script.slice();
  await showSimplifiedScript(script);

  logDebug("init");

  gGLRenderer.loadTexture(texturePathElement.textContent)
    .then(async () => await renderScript({ script, scriptHash }))
    .catch(error => console.error(error));

  originalButton.addEventListener('click', async () => {
    originalButton.disabled = true;
    await setMode("normal");
    await renderScript({ script: originalScript, scriptHash });
    scriptElement.textContent = originalScript;
    await showSimplifiedScript(originalScript);
  });

  slideshowButton.addEventListener('click', async () => {
    originalButton.disabled = true;
    await setMode("slideshow");
    await renderScript({ script: originalScript, scriptHash });
    scriptElement.textContent = originalScript;
    await showSimplifiedScript(originalScript);
  });

  sketchButton.addEventListener('click', async () => {
    originalButton.disabled = true;
    await setMode("sketch");
    scriptElement.textContent = originalScript;
    await showSimplifiedScript(originalScript);
  });

  evalButton.addEventListener('click', async () => {
    gNumTransitions += 1;
    originalButton.disabled = false;
    const newSeed = Math.random() * (1 << 30);
    seedElement.value = parseInt(newSeed, 10);

    const seedValue = getSeedValue(seedElement);

    const { traits } = await Job.request(jobBuildTraits, { script: originalScript, scriptHash });
    const { genotype } = await Job.request(jobSingleGenotypeFromSeed, { traits, seed: seedValue });

    const config = { script: originalScript, scriptHash };
    if (seedValue !== 0) {
      config.genotype = genotype;
    }

    await renderScript(config);

    const { script } = await Job.request(jobUnparse, { script: originalScript, genotype });

    scriptElement.textContent = script;
    await showSimplifiedScript(script);
  });

  canvasImageElement1.addEventListener('click', async () => {
    await setMode("normal");
  });

  const escapeKey = 27;
  document.addEventListener('keydown', async event => {
    if (event.keyCode === escapeKey && gMode === 'slideshow') {
      await setMode('normal');
      event.preventDefault();
    }
  }, false);
}

document.addEventListener('DOMContentLoaded', () => {
  compatibilityHacks();
  main();
});
