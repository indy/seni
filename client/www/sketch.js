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
    gl.deleteShader(shader);
    return null;
  }
  return shader;
}

function setupSketchShaders(gl) {
  const shader = {};

  shader.program = gl.createProgram();

  // pre-multiply the alpha in the shader
  // see http://www.realtimerendering.com/blog/gpus-prefer-premultiplication/
  // this needs to happen in linear colour space
  const fragmentSrc = `
  precision highp float;
  varying vec4 vColour;
  varying highp vec2 vTextureCoord;

  uniform sampler2D uSampler;
  uniform bool uOutputLinearColourSpace;

  // https://en.wikipedia.org/wiki/SRGB
  vec3 srgb_to_linear(vec3 srgb) {
      float a = 0.055;
      float b = 0.04045;
      vec3 linear_lo = srgb / 12.92;
      vec3 linear_hi = pow((srgb + vec3(a)) / (1.0 + a), vec3(2.4));
      return vec3(
          srgb.r > b ? linear_hi.r : linear_lo.r,
          srgb.g > b ? linear_hi.g : linear_lo.g,
          srgb.b > b ? linear_hi.b : linear_lo.b);
  }

  void main(void) {
    vec4 tex = texture2D(uSampler, vTextureCoord);

    // note: you _never_ want uOutputLinearColourSpace to be set to true
    // it's only here because some of the older sketchs didn't correctly
    // convert from linear colour space to sRGB colour space during rendering
    // and this shader needs to reproduce them as intended at time of creation
    //
    if (uOutputLinearColourSpace) {
      gl_FragColor.r = tex.r * vColour.r * vColour.a;
      gl_FragColor.g = tex.r * vColour.g * vColour.a;
      gl_FragColor.b = tex.r * vColour.b * vColour.a;
      gl_FragColor.a = tex.r * vColour.a;
    } else {
      vec4 linearColour = vec4(srgb_to_linear(vColour.rgb), vColour.a);
      gl_FragColor.r = tex.r * linearColour.r * linearColour.a;
      gl_FragColor.g = tex.r * linearColour.g * linearColour.a;
      gl_FragColor.b = tex.r * linearColour.b * linearColour.a;
      gl_FragColor.a = tex.r * linearColour.a;
    }
  }
  `;

  const vertexSrc = `
  attribute vec2 aVertexPosition;
  attribute vec4 aVertexColour;
  attribute vec2 aVertexTexture;

  uniform mat4 uMVMatrix;
  uniform mat4 uPMatrix;

  varying vec4 vColour;
  varying highp vec2 vTextureCoord;

  void main(void) {
    gl_Position = uPMatrix * uMVMatrix * vec4(aVertexPosition, 0.0, 1.0);
    vColour = aVertexColour;
    vTextureCoord = aVertexTexture;
  }
  `;

  const vertexShader = compileShader(gl, gl.VERTEX_SHADER, vertexSrc);
  const fragmentShader = compileShader(gl, gl.FRAGMENT_SHADER, fragmentSrc);

  gl.attachShader(shader.program, vertexShader);
  gl.attachShader(shader.program, fragmentShader);

  gl.linkProgram(shader.program);

  if (!gl.getProgramParameter(shader.program, gl.LINK_STATUS)) {
    let lastError = gl.getProgramInfoLog(shader.program);

    alert(`Could not initialise shaders: ${lastError}`);;
    gl.deleteProgram(shader.program);
    return null;
  }

  shader.positionAttribute = gl.getAttribLocation(shader.program, 'aVertexPosition');
  shader.colourAttribute = gl.getAttribLocation(shader.program, 'aVertexColour');
  shader.textureAttribute = gl.getAttribLocation(shader.program, 'aVertexTexture');

  shader.pMatrixUniform = gl.getUniformLocation(shader.program, 'uPMatrix');
  shader.mvMatrixUniform = gl.getUniformLocation(shader.program, 'uMVMatrix');
  shader.textureUniform  = gl.getUniformLocation(shader.program, 'uSampler');

  // older versions of seni (pre 4.2.0) did not convert from sRGB space to linear before blending
  // in order to retain the look of these older sketchs we can't carry out the linear -> sRGB conversion
  //
  shader.outputLinearColourSpaceUniform = gl.getUniformLocation(shader.program, 'uOutputLinearColourSpace');

  return shader;
}

function setupBlitShaders(gl) {
  const shader = {};

  shader.program = gl.createProgram();

  const fragmentSrc = `
  precision highp float;

  varying vec2 vTextureCoord;

  uniform sampler2D uSampler;
  uniform bool uOutputLinearColourSpace;

  // https:en.wikipedia.org/wiki/SRGB
  vec3 linear_to_srgb(vec3 linear) {
      float a = 0.055;
      float b = 0.0031308;
      vec3 srgb_lo = 12.92 * linear;
      vec3 srgb_hi = (1.0 + a) * pow(linear, vec3(1.0/2.4)) - vec3(a);
      return vec3(
          linear.r > b ? srgb_hi.r : srgb_lo.r,
          linear.g > b ? srgb_hi.g : srgb_lo.g,
          linear.b > b ? srgb_hi.b : srgb_lo.b);
  }

  // https:twitter.com/jimhejl/status/633777619998130176
  vec3 ToneMapFilmic_Hejl2015(vec3 hdr, float whitePt) {
      vec4 vh = vec4(hdr, whitePt);
      vec4 va = 1.425 * vh + 0.05;
      vec4 vf = (vh * va + 0.004) / (vh * (va + 0.55) + 0.0491) - 0.0821;
      return vf.rgb / vf.www;
  }

  void main()
  {
     vec4 col = texture2D( uSampler, vTextureCoord );

     // note: you _never_ want uOutputLinearColourSpace to be set to true
     // it's only here because some of the older sketchs didn't correctly
     // convert from linear colour space to sRGB colour space during rendering
     // and this shader needs to reproduce them as intended at time of creation
     //
     if (uOutputLinearColourSpace) {
       gl_FragColor = col;
     } else {
       gl_FragColor = vec4(linear_to_srgb(col.rgb), 1.0);
     }
  }
  `;

  const vertexSrc = `
  attribute vec2 aVertexPosition;
  attribute vec2 aVertexTexture;

  uniform mat4 uMVMatrix;
  uniform mat4 uPMatrix;

  varying highp vec2 vTextureCoord;

  void main(void) {
    gl_Position = uPMatrix * uMVMatrix * vec4(aVertexPosition, 0.0, 1.0);
    vTextureCoord = aVertexTexture;
  }
  `;

  const vertexShader = compileShader(gl, gl.VERTEX_SHADER, vertexSrc);
  const fragmentShader = compileShader(gl, gl.FRAGMENT_SHADER, fragmentSrc);

  gl.attachShader(shader.program, vertexShader);
  gl.attachShader(shader.program, fragmentShader);

  gl.linkProgram(shader.program);

  if (!gl.getProgramParameter(shader.program, gl.LINK_STATUS)) {
    let lastError = gl.getProgramInfoLog(shader.program);

    alert(`Could not initialise shaders: ${lastError}`);;
    gl.deleteProgram(shader.program);
    return null;
  }

  shader.positionAttribute = gl.getAttribLocation(shader.program, 'aVertexPosition');
  shader.textureAttribute = gl.getAttribLocation(shader.program, 'aVertexTexture');

  shader.pMatrixUniform = gl.getUniformLocation(shader.program, 'uPMatrix');
  shader.mvMatrixUniform = gl.getUniformLocation(shader.program, 'uMVMatrix');
  shader.textureUniform  = gl.getUniformLocation(shader.program, 'uSampler');

  // older versions of seni (pre 4.2.0) did not convert from sRGB space to linear before blending
  // in order to retain the look of these older sketchs we can't carry out the linear -> sRGB conversion
  //
  shader.outputLinearColourSpaceUniform = gl.getUniformLocation(shader.program, 'uOutputLinearColourSpace');

  return shader;
}

function setupGLState(gl) {
  // clear colour alpha is 1.0 as we want to treat a blank canvas as opaque white
  gl.clearColor(1.0, 1.0, 1.0, 1.0);
  gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
  gl.enable(gl.BLEND);

  // assuming that we'll be using pre-multiplied alpha
  // see http://www.realtimerendering.com/blog/gpus-prefer-premultiplication/
  gl.blendEquation(gl.FUNC_ADD);
  gl.blendFunc(gl.ONE, gl.ONE_MINUS_SRC_ALPHA);

//  gl.disable(gl.DEPTH_TEST);
}


function handleTextureLoaded(gl, image, texture) {
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
}

function createRenderTexture(gl) {
  // create to render to
  const targetTextureWidth = gState.render_texture_width;
  const targetTextureHeight = gState.render_texture_height;

  const targetTexture = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, targetTexture);

  {
    // define size and format of level 0
    const level = 0;
    const internalFormat = gl.RGBA;
    const border = 0;
    const format = gl.RGBA;
    const type = gl.UNSIGNED_BYTE;
    const data = null;
    gl.texImage2D(gl.TEXTURE_2D, level, internalFormat,
                  targetTextureWidth, targetTextureHeight, border,
                  format, type, data);

    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    // gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    // gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  }

  return targetTexture;
}

function createFrameBuffer(gl, targetTexture) {
  // Create and bind the framebuffer
  const fb = gl.createFramebuffer();
  gl.bindFramebuffer(gl.FRAMEBUFFER, fb);

  // attach the texture as the first color attachment
  const attachmentPoint = gl.COLOR_ATTACHMENT0;
  const level = 0;
  gl.framebufferTexture2D(gl.FRAMEBUFFER, attachmentPoint, gl.TEXTURE_2D, targetTexture, level);

  return fb;
}

function checkFramebufferStatus(gl) {
  let res = gl.checkFramebufferStatus(gl.FRAMEBUFFER);
  switch(res) {
  case gl.FRAMEBUFFER_COMPLETE: console.log("gl.FRAMEBUFFER_COMPLETE"); break;
  case gl.FRAMEBUFFER_INCOMPLETE_ATTACHMENT: console.log("gl.FRAMEBUFFER_INCOMPLETE_ATTACHMENT"); break;
  case gl.FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT: console.log("gl.FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT"); break;
  case gl.FRAMEBUFFER_INCOMPLETE_DIMENSIONS: console.log("gl.FRAMEBUFFER_INCOMPLETE_DIMENSIONS"); break;
  case gl.FRAMEBUFFER_UNSUPPORTED: console.log("gl.FRAMEBUFFER_UNSUPPORTED"); break;
  case gl.FRAMEBUFFER_INCOMPLETE_MULTISAMPLE: console.log("gl.FRAMEBUFFER_INCOMPLETE_MULTISAMPLE"); break;
  case gl.RENDERBUFFER_SAMPLES: console.log("gl.RENDERBUFFER_SAMPLE"); break;
  }
}

class GLRenderer {
  constructor(canvasElement) {
    this.glDomElement = canvasElement;

    // webgl setup
    const gl = initGL(this.glDomElement);
    this.gl = gl;

    this.sketchShader = setupSketchShaders(gl);
    this.blitShader = setupBlitShaders(gl);

    setupGLState(gl);

    this.glVertexBuffer = gl.createBuffer();
    this.glColourBuffer = gl.createBuffer();
    this.glTextureBuffer = gl.createBuffer();

    this.mvMatrix = Matrix.create();
    this.pMatrix = Matrix.create();

    this.renderTexture = createRenderTexture(gl);
    this.framebuffer = createFrameBuffer(gl, this.renderTexture);

    // render to the canvas
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
  }

  // isg
  clear() {
    this.gl.clear(this.gl.COLOR_BUFFER_BIT | this.gl.DEPTH_BUFFER_BIT);
  }

  loadTexture(src) {
    let that = this;

    return new Promise(function(resolve, reject) {

      const gl = that.gl;
      that.texture = gl.createTexture();
      const image = new Image();

      image.addEventListener('load', () => {
        handleTextureLoaded(that.gl, image, that.texture);
        resolve();
      });

      image.addEventListener('error', () => {
        reject();
      });

      image.src = src;
    });
  }

  renderGeometryToTexture(meta, destTextureWidth, destTextureHeight, memoryF32, buffers, section) {
    const gl = this.gl;

    let shader = this.sketchShader;

    // render to texture attached to framebuffer

    gl.bindFramebuffer(gl.FRAMEBUFFER, this.framebuffer);
    //gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    gl.bindTexture(gl.TEXTURE_2D, this.texture);
    gl.viewport(0, 0, destTextureWidth, destTextureHeight);

    gl.useProgram(shader.program);

    gl.enableVertexAttribArray(shader.positionAttribute);
    gl.enableVertexAttribArray(shader.colourAttribute);
    gl.enableVertexAttribArray(shader.textureAttribute);

    // gl.clearColor(0.0, 0.0, 1.0, 1.0);
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

    gl.uniformMatrix4fv(shader.pMatrixUniform,
                        false,
                        this.pMatrix);

    gl.uniformMatrix4fv(shader.mvMatrixUniform,
                        false,
                        this.mvMatrix);

    gl.uniform1i(shader.textureUniform, 0);

    gl.uniform1i(shader.outputLinearColourSpaceUniform, meta.output_linear_colour_space);

    const glVertexBuffer = this.glVertexBuffer;
    const glColourBuffer = this.glColourBuffer;
    const glTextureBuffer = this.glTextureBuffer;

    const bytesin32bit = 4;

    const vertexItemSize = 2;
    const colourItemSize = 4;
    const textureItemSize = 2;
    const totalSize = (vertexItemSize + colourItemSize + textureItemSize);


    buffers.forEach(buffer => {
      // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray#Syntax
      // a new typed array view is created that views the specified ArrayBuffer
      const gbuf = new Float32Array(memoryF32, buffer.geo_ptr, buffer.geo_len);

      //const gbuf = memorySubArray(memoryF32, geo_ptr, geo_len);

      gl.bindBuffer(gl.ARRAY_BUFFER, glVertexBuffer);
      gl.bufferData(gl.ARRAY_BUFFER, gbuf, gl.STATIC_DRAW);
      gl.vertexAttribPointer(shader.positionAttribute,
                             vertexItemSize,
                             gl.FLOAT, false, totalSize * bytesin32bit,
                             0);

      gl.bindBuffer(gl.ARRAY_BUFFER, glColourBuffer);
      gl.bufferData(gl.ARRAY_BUFFER, gbuf, gl.STATIC_DRAW);
      gl.vertexAttribPointer(shader.colourAttribute,
                             colourItemSize,
                             gl.FLOAT, false, totalSize * bytesin32bit,
                             vertexItemSize * bytesin32bit);

      gl.bindBuffer(gl.ARRAY_BUFFER, glTextureBuffer);
      gl.bufferData(gl.ARRAY_BUFFER, gbuf, gl.STATIC_DRAW);
      gl.vertexAttribPointer(shader.textureAttribute,
                             textureItemSize,
                             gl.FLOAT, false, totalSize * bytesin32bit,
                             (vertexItemSize + colourItemSize) * bytesin32bit);

      gl.drawArrays(gl.TRIANGLE_STRIP, 0, buffer.geo_len / totalSize);

    });

  }

  renderTextureToScreen(meta, canvasWidth, canvasHeight) {
    const gl = this.gl;
    const domElement = this.glDomElement;

    if (domElement.width !== canvasWidth) {
      domElement.width = canvasWidth;
    }
    if (domElement.height !== canvasHeight) {
      domElement.height = canvasHeight;
    }

    let shader = this.blitShader;

    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    gl.bindTexture(gl.TEXTURE_2D, this.renderTexture);
    gl.viewport(0, 0, gl.drawingBufferWidth, gl.drawingBufferHeight);

    gl.useProgram(shader.program);

    // console.log(shader);
    gl.enableVertexAttribArray(shader.positionAttribute);
    gl.enableVertexAttribArray(shader.textureAttribute);

    // gl.clearColor(0.0, 0.0, 1.0, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

    // render the entirety of the scene
    Matrix.ortho(this.pMatrix, 0, canvasWidth, 0, canvasHeight, 10, -10);

    // add some uniforms for canvas width and height

    gl.uniformMatrix4fv(shader.pMatrixUniform,
                        false,
                        this.pMatrix);

    gl.uniformMatrix4fv(shader.mvMatrixUniform,
                        false,
                        this.mvMatrix);

    gl.uniform1i(shader.textureUniform, 0);

    gl.uniform1i(shader.outputLinearColourSpaceUniform, meta.output_linear_colour_space);


    const glVertexBuffer = this.glVertexBuffer;
    const glColourBuffer = this.glColourBuffer;
    const glTextureBuffer = this.glTextureBuffer;

    // x, y, u, v
    const jsData = [
      0.0, 0.0, 0.0, 0.0,
      canvasWidth, 0.0, 1.0, 0.0,
      0.0, canvasHeight, 0.0, 1.0,
      canvasWidth, canvasHeight, 1.0, 1.0
    ];
    const data = Float32Array.from(jsData);


    const bytesin32bit = 4;

    const vertexItemSize = 2;
    const textureItemSize = 2;
    const totalSize = (vertexItemSize + textureItemSize);

    gl.bindBuffer(gl.ARRAY_BUFFER, glVertexBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);
    gl.vertexAttribPointer(shader.positionAttribute,
                           vertexItemSize,
                           gl.FLOAT, false, totalSize * 4,
                           0);

    gl.bindBuffer(gl.ARRAY_BUFFER, glTextureBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);
    gl.vertexAttribPointer(shader.textureAttribute,
                           textureItemSize,
                           gl.FLOAT, false, totalSize * 4,
                           (vertexItemSize) * 4);

    gl.drawArrays(gl.TRIANGLE_STRIP, 0, jsData.length / totalSize);
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

const RUNNING_ON_STATIC_SITE = false;
const PAGE = RUNNING_ON_STATIC_SITE ? "index.html" : "sketch.html";

const URI_SEED = "seed";
const URI_MODE = "mode";

const MODE_NORMAL = "normal";
const MODE_SLIDESHOW = "slideshow";

// either display the generated image asap or fade it in
const DISPLAY_SNAP = 0;
const DISPLAY_FADE = 1;

const IMG_0 = 'sketch-img-0';
const IMG_1 = 'sketch-img-1';

let gState = {
  glRenderer: undefined,
  logDebug: false,
  timoutId: undefined,

  slideshowDelay: 5000,
  demandCanvasSize: 500,
  mode: MODE_NORMAL,
  seed: undefined,
  activeImageElement: 0,
  lastDisplay: DISPLAY_SNAP,

  render_texture_width: 1024,
  render_texture_height: 1024,
};

function logDebug(msg) {
  if (gState.logDebug) {
    const op0 = getRequiredElement(IMG_0).style.opacity;
    const op1 = getRequiredElement(IMG_1).style.opacity;

    console.log(`${msg} ${gState.mode} img-0 opacity: ${op0}, img-1 opacity: ${op1} activeImageElement: ${gState.activeImageElement}`);
  }
}

async function displayOnImageElements(display) {
  // required to check that an endAnimation doesn't fade in sketch-img-1
  gState.lastDisplay = display;

  if (display === DISPLAY_SNAP) {
    resetImageElements();

    const sketchImg0 = getRequiredElement(IMG_0);
    await gState.glRenderer.copyImageDataTo(sketchImg0);
  } else {
    if (gState.activeImageElement === 0) {
      const sketchImg0 = getRequiredElement(IMG_0);
      await gState.glRenderer.copyImageDataTo(sketchImg0);

      if (gState.mode === MODE_NORMAL) {
        addClass(IMG_1, 'seni-fade-out');
      } else if (gState.mode === MODE_SLIDESHOW) {
        addClass(IMG_1, 'seni-fade-out-slideshow');
      }
    } else {
      const sketchImg1 = getRequiredElement(IMG_1);
      await gState.glRenderer.copyImageDataTo(sketchImg1);

      if (gState.mode === MODE_NORMAL) {
        addClass(IMG_1, 'seni-fade-in');
      } else if (gState.mode === MODE_SLIDESHOW) {
        addClass(IMG_1, 'seni-fade-in-slideshow');
      }
    }

    gState.activeImageElement = 1 - gState.activeImageElement;
  }

  logDebug("displayOnImageElements");
}

async function renderGeometryBuffers(meta, memory, buffers, destWidth, destHeight, display) {
  gState.glRenderer.renderGeometryToTexture(meta, gState.render_texture_width, gState.render_texture_height, memory, buffers);
  gState.glRenderer.renderTextureToScreen(meta, destWidth, destHeight);

  await displayOnImageElements(display);
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

  // note: setting an absolute path as this works with static site generation for seni.indy.io
  return "/img/" + url;
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

async function renderScript(parameters, display) {
  console.log(`renderScript  (demandCanvasSize = ${gState.demandCanvasSize})`);
  let { meta, memory, buffers } = await renderJob(parameters);
  await renderGeometryBuffers(meta, memory, buffers, gState.demandCanvasSize, gState.demandCanvasSize, display);
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

  const simplifiedScriptElement = getRequiredElement('sketch-simplified-script');
  simplifiedScriptElement.textContent = script;
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
  if (gState.mode === MODE_SLIDESHOW) {
    const scriptElement = getRequiredElement('sketch-script');
    const seedElement = getRequiredElement('sketch-seed');
    const script = scriptElement.textContent;
    const originalScript = script.slice();

    const newSeed = Math.random() * (1 << 30);
    seedElement.value = parseInt(newSeed, 10);
    gState.seed = getSeedValue(seedElement);

    updateURIFromGlobals(false);
    await updateSketch(DISPLAY_FADE);
    gState.timeoutId = window.setTimeout(performSlideshow, gState.slideshowDelay);
  }
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

function resetImageElements() {
  setOpacity(IMG_1, 0);
  gState.activeImageElement = 1;

  removeClass(IMG_1, 'seni-fade-in');
  removeClass(IMG_1, 'seni-fade-in-slideshow');
}




function moveContainerInsideParent(parentId, forceLargest) {
  const canvasContainerId = 'sketch-canvas-container';
  const canvasContainer = getRequiredElement(canvasContainerId);

  const parent = getRequiredElement(parentId);
  parent.appendChild(canvasContainer);

  let dim = 0;
  if (forceLargest) {
    let forceWidth = document.documentElement.clientWidth;
    let forceHeight = document.documentElement.clientHeight;
    dim = forceWidth < forceHeight ? forceWidth : forceHeight;


    let marginLeft = (forceWidth - dim) / 2;
    canvasContainer.style.marginLeft = "" + marginLeft + "px";

  } else {
    dim = parent.clientWidth < parent.clientHeight ? parent.clientWidth : parent.clientHeight;
    canvasContainer.style.marginLeft = "0px";
  }

  canvasContainer.width = dim;
  canvasContainer.height = dim;
  gState.demandCanvasSize = dim;

  const img0 = getRequiredElement('sketch-img-0');
  img0.width = dim;
  img0.height = dim;

  const img1 = getRequiredElement('sketch-img-1');
  img1.width = dim;
  img1.height = dim;
}

function styleForNormalSketch() {
  showId('header');
  showId('main');

  moveContainerInsideParent('sketch-normal-anchor');

  resetImageElements();
}

function styleForLargeSketch() {
  hideId('header');
  hideId('main');

  moveContainerInsideParent('sketch-large-anchor', true);

  resetImageElements();
}


async function updateToMode(newMode) {
  if (gState.mode === newMode) {
    return false;
  }

  gState.mode = newMode;

  gState.glRenderer.clear();

  const sketchImg0 = getRequiredElement(IMG_0);
  await gState.glRenderer.copyImageDataTo(sketchImg0);
  const sketchImg1 = getRequiredElement(IMG_1);
  await gState.glRenderer.copyImageDataTo(sketchImg1);

  if (gState.mode === MODE_SLIDESHOW) {
    styleForLargeSketch();
  } else if (gState.mode === MODE_NORMAL) {
    window.clearTimeout(gState.timeoutId); // stop the slideshow
    styleForNormalSketch();
  }

  return true;
}

function animationEndListener1(event) {
  if (event.animationName === 'senifadeout') {
    removeClass(IMG_1, 'seni-fade-out');
    removeClass(IMG_1, 'seni-fade-out-slideshow');
    setOpacity(IMG_1, 0);
  }

  if (event.animationName === 'senifadein') {
    removeClass(IMG_1, 'seni-fade-in');
    removeClass(IMG_1, 'seni-fade-in-slideshow');
    if (gState.lastDisplay === DISPLAY_SNAP) {
      // if we were in a slideshow and the user pressed escape to go back to a normal render
      // the fade animation that was playing for the previous mode has now finished
      setOpacity(IMG_1, 0);
    } else {
      setOpacity(IMG_1, 1);
    }
  }
}

function compatibilityHacks() {
  // Safari doesn't have Number.parseInt (yet)
  // Safari is the new IE
  if (Number.parseInt === undefined) {
    Number.parseInt = parseInt;
  }
}

function getURIParameters() {
  const argPairs = window.location.search.substring(1).split("&");

  return argPairs.reduce((acc, kv) => {
    let [key, value] = kv.split("=");
    if (key === URI_SEED) {
      acc[key] = parseInt(value, 10);
    } else {
      acc[key] = value;
    }

    return acc;
  }, {});
}

function updateGlobalsFromURI() {
  const uriParameters = getURIParameters();

  if (uriParameters.hasOwnProperty(URI_SEED)) {
    gState.seed = uriParameters[URI_SEED];
  } else {
    gState.seed = undefined;
  }

  if (uriParameters[URI_MODE] === MODE_SLIDESHOW) {
    updateToMode(MODE_SLIDESHOW);
  } else {
    // absence of mode parameter in URI means MODE_NORMAL
    updateToMode(MODE_NORMAL);
  }
}

function updateURIFromGlobals(updateHistory) {
  let params = [];
  if (gState.mode != MODE_NORMAL) {
    params.push("mode=" + gState.mode);
  }
  if (gState.seed !== undefined) {
    params.push("seed=" + gState.seed);
  }

  let search = "";
  if (params.length > 0) {
    search = "?" + params.join("&");
  }

  if (updateHistory && window.location.search !== search) {
    // desired uri is different from current one
    const page_uri = PAGE + search;
    history.pushState({}, null, page_uri);
  }
}

async function renderNormal(display) {
  const scriptElement = getRequiredElement('sketch-script');
  const script = scriptElement.textContent.slice();

  if (gState.seed === undefined) {
    await showSimplifiedScript(script);
    await renderScript({ script }, display);
  } else {
    const { traits } = await Job.request(jobBuildTraits, { script });
    const { genotype } = await Job.request(jobSingleGenotypeFromSeed, { traits, seed: gState.seed });

    const unparsed = await Job.request(jobUnparse, { script, genotype });
    await showSimplifiedScript(unparsed.script);
    await renderScript({ script, genotype }, display);
  }
}

async function updateSketch(display) {
  await renderNormal(display);
}

async function main() {
  updateGlobalsFromURI();

  const texturePathElement = getRequiredElement('sketch-texture-path');
  const workerPathElement = getRequiredElement('sketch-worker-path');

  Job.setup(2, workerPathElement.textContent);

  const originalButton = getRequiredElement('sketch-eval-original');
  const variationButton = getRequiredElement('sketch-eval-variation');
  const slideshowButton = getRequiredElement('sketch-eval-slideshow');
  const scriptElement = getRequiredElement('sketch-script');
  const canvasElement = getRequiredElement('sketch-canvas');
  const canvasImageElement0 = getRequiredElement(IMG_0);
  const canvasImageElement1 = getRequiredElement(IMG_1);

  canvasImageElement1.addEventListener("animationend", animationEndListener1, false);
  setOpacity(IMG_1, 0);

  gState.glRenderer = new GLRenderer(canvasElement);

  const script = scriptElement.textContent;
  const originalScript = script.slice();

  logDebug("init");

  gState.glRenderer.loadTexture(texturePathElement.textContent)
    .then(async () => await updateSketch(DISPLAY_SNAP))
    .catch(error => console.error(error));

  originalButton.addEventListener('click', async () => {
    originalButton.disabled = true;

    gState.seed = undefined;
    updateToMode(MODE_NORMAL);

    updateURIFromGlobals(true);

    await updateSketch(DISPLAY_FADE);
  });

  slideshowButton.addEventListener('click', async () => {
    originalButton.disabled = true;

    if (updateToMode(MODE_SLIDESHOW)) {
      await updateSketch(DISPLAY_SNAP);
      const sketchImg1 = getRequiredElement(IMG_1);
      await gState.glRenderer.copyImageDataTo(sketchImg1);

      // only call updateSketch if we're actually switching to SLIDESHOW mode as this will create a settimeout
      gState.timeoutId = window.setTimeout(performSlideshow, 0);
    }
    updateURIFromGlobals(true);

  });

  variationButton.addEventListener('click', async () => {
    originalButton.disabled = false;

    const seedElement = getRequiredElement('sketch-seed');
    const newSeed = Math.random() * (1 << 30);
    seedElement.value = parseInt(newSeed, 10);
    gState.seed = getSeedValue(seedElement);

    updateToMode(MODE_NORMAL);

    updateURIFromGlobals(true);

    await updateSketch(DISPLAY_FADE);
  });

  window.addEventListener('popstate', async event => {
    updateGlobalsFromURI();
    await updateSketch(DISPLAY_SNAP);
  });

  canvasImageElement1.addEventListener('click', async () => {
    updateToMode(MODE_NORMAL);

    updateURIFromGlobals(true);

    await updateSketch(DISPLAY_SNAP);
  });

  const escapeKey = 27;
  document.addEventListener('keydown', async event => {
    if (event.keyCode === escapeKey && gState.mode !== MODE_NORMAL) {

      updateToMode(MODE_NORMAL);

      updateURIFromGlobals(true);

      await updateSketch(DISPLAY_SNAP);

      event.preventDefault();
    }
  }, false);
}

document.addEventListener('DOMContentLoaded', () => {
  compatibilityHacks();
  main();
});
