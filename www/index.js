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

// --------------------------------------------------------------------------------
// log

let logToConsole = false;

function log(msg) {
  if (logToConsole) {
    console.log(msg);
  }
}

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
    const gl = canvas.getContext('webgl', {
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
    console.log(gl.getShaderInfoLog(shader));
    gl.deleteShader(shader);
    return null;
  }
  return shader;
}

function setupSketchShaders(gl, vertexSrc, fragmentSrc) {
  const shader = {};

  shader.program = gl.createProgram();

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
  shader.textureUniform  = gl.getUniformLocation(shader.program, 'texture');

  // older versions of seni (pre 4.2.0) did not convert from sRGB space to linear before blending
  // in order to retain the look of these older sketchs we can't carry out the linear -> sRGB conversion
  //
  shader.outputLinearColourSpaceUniform = gl.getUniformLocation(shader.program, 'uOutputLinearColourSpace');

  return shader;
}

function setupBlitShaders(gl, vertexSrc, fragmentSrc) {
  const shader = {};

  shader.program = gl.createProgram();

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
  shader.textureUniform  = gl.getUniformLocation(shader.program, 'texture');

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

  // assuming that we'll be using pre-multiplied alpha
  // see http://www.realtimerendering.com/blog/gpus-prefer-premultiplication/
  gl.enable(gl.BLEND);
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
  gl.bindTexture(gl.TEXTURE_2D, null);  /// ?????

  gl.activeTexture(gl.TEXTURE0);
  gl.bindTexture(gl.TEXTURE_2D, texture);
}

function createRenderTexture(gl, config) {
  // create to render to
  const targetTextureWidth = config.render_texture_width;
  const targetTextureHeight = config.render_texture_height;

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

const gConfig = {
  render_texture_width: 2048,
  render_texture_height: 2048,
};

class GLRenderer {
  constructor(canvasElement, shaders) {
    this.glDomElement = canvasElement;

    // webgl setup
    const gl = initGL(this.glDomElement);
    this.gl = gl;

    // note: constructors can't be async so the shaders should already have been loaded by loadShaders
    this.sketchShader = setupSketchShaders(gl, shaders['shader/main-vert.glsl'], shaders['shader/main-frag.glsl']);
    this.blitShader = setupBlitShaders(gl, shaders['shader/blit-vert.glsl'], shaders['shader/blit-frag.glsl']);

    setupGLState(gl);

    this.glVertexBuffer = gl.createBuffer();

    this.mvMatrix = Matrix.create();
    this.pMatrix = Matrix.create();

    this.renderTexture = createRenderTexture(gl, gConfig);
    this.framebuffer = createFrameBuffer(gl, this.renderTexture);

    // render to the canvas
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
  }

  loadTexture(src) {
    let that = this;

    return new Promise((resolve, reject) => {

      const gl = that.gl;
      // todo: why is texture attached to that?
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
      gl.vertexAttribPointer(shader.colourAttribute,
                             colourItemSize,
                             gl.FLOAT, false, totalSize * bytesin32bit,
                             vertexItemSize * bytesin32bit);
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
          // console.log(this.glDomElement);
          // console.log(blob);
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

      downloadDialogHide();
    });
  }

}

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
    log('historyPushState', state);
    history.pushState(state, null, uri);
  },
  replaceState: function(appState) {
    const [state, uri] = buildState(appState);
    log('historyReplace', state);
    history.replaceState(state, null, uri);
  },
  restoreState: function(state) {
    log('historyRestore', state);
    return state;
  }
};

// --------------------------------------------------------------------------------
// codemirrorSeni

function seniMode() {
  const BUILTIN = 'builtin';
  const COMMENT = 'comment';
  const STRING = 'string';
  const ATOM = 'atom';
  const NUMBER = 'number';
  const TILDE = 'tilde';     // ~
  const PAREN = 'paren';     // ()
  const BRACKET = 'bracket'; // []
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
        makeKeywords('begin define fn if fence loop on-matrix-stack quote meta');
  const indentKeys = makeKeywords('define fence loop on-matrix-stack fn');

  // functions from the common seni library
  const seniCommon = makeKeywords(`* + - / < = > append begin bezier
bezier-bulging bezier-trailing box canvas/centre canvas/height canvas/width
circle circle-slice col/analagous col/bezier-fn col/complementary col/convert
col/darken col/alpha col/hsl col/hsluv col/hsv col/lab col/lighten
col/procedural-fn col/quadratic-fn col/rgb col/set-alpha col/e0 col/e1 col/e2 col/set-e0 col/set-e1 col/set-e2
col/split-complementary col/triad define math/degrees->radians fence fn focal/hline
focal/point focal/vline if interp/bezier interp/bezier-fn interp/bezier-tangent
interp/bezier-tangent-fn interp/circle interp/fn line list list/get list/length
log loop math/PI math/TAU math/atan2 math/clamp math/cos math/distance-2d
math/sin mod on-matrix-stack path/bezier path/circle path/linear path/spline
poly pop-matrix print prng/perlin-signed prng/perlin-unsigned prng/range
push-matrix quote radians->degrees rect repeat/rotate repeat/rotate-mirrored
repeat/symmetry-4 repeat/symmetry-8 repeat/symmetry-horizontal
repeat/symmetry-vertical rotate scale spline sqrt stroked-bezier
take translate`);

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
    return state.afterTilde === true ? 'geno-' + token : token;
  }


  function setAfterTilde(value, state) {
    if (value === true) {
      // switch off afterTilde when we get to a closing paren of parenDepth + 1
      state.afterTildeParenDepth = state.parenDepth + 1;
    }
    state.afterTilde = value;
  }

  return {
    startState: () => {
      const state = {
        indentStack: null,
        indentation: 0,
        mode: false,
        sExprComment: false,

        parenDepth: 0,

        afterTilde: false,
        afterTildeParenDepth: 0,
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
        } else if (ch === '~') {
          setAfterTilde(true, state);
          returnType = tokenType(TILDE, state);
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
        } else if (ch === '(') {
          let keyWord = '', letter;
          const indentTemp = stream.column();

          state.parenDepth++;

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

          returnType = tokenType(PAREN, state, ch);
        } else if (ch === ')') {
          returnType = tokenType(PAREN, state, ch);
          if (state.indentStack != null && state.indentStack.type === '(') {
            popStack(state);

            if (typeof state.sExprComment === 'number') {
              if (--state.sExprComment === 0) {
                returnType = tokenType(COMMENT, state);
                state.sExprComment = false; // turn off s-expr commenting mode
              }
            }
          }

          if (state.afterTilde === true && state.parenDepth === state.afterTildeParenDepth) {
            setAfterTilde(false, state);
          }

          state.parenDepth--;
        } else {
          stream.eatWhile(/[\w\$_\-!$%&*+\.\/:<=>?@\^]/);

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
// controller

const actionSetMode = 'SET_MODE';
const actionSetGenotype = 'SET_GENOTYPE';
const actionSetScript = 'SET_SCRIPT';
const actionSetScriptId = 'SET_SCRIPT_ID';
const actionSetSelectedIndices = 'SET_SELECTED_INDICES';
const actionInitialGeneration = 'INITIAL_GENERATION';
const actionNextGeneration = 'NEXT_GENERATION';
const actionShuffleGeneration = 'SHUFFLE_GENERATION';
const actionSetState = 'SET_STATE';
const actionSetGalleryItems = "SET_GALLERY_ITEMS";
const actionGalleryOldestToDisplay = 'GALLERY_OLDEST_TO_DISPLAY';

function createInitialState() {
  return {
    // the resolution of the high res image
    highResolution: [2048, 2048], // [4096, 4096],
    placeholder: 'img/spinner.gif',
    populationSize: 24,
    mutationRate: 0.1,

    currentMode: SeniMode.gallery,

    galleryLoaded: false,
    galleryOldestToDisplay: 9999,
    galleryItems: {},
    galleryDisplaySize: 20,     // the number of gallery sketchs to display everytime 'load more' is clicked

    previouslySelectedGenotypes: [],
    selectedIndices: [],
    scriptId: undefined,
    script: undefined,
    genotypes: [],
    traits: [],

    genotype: undefined,
  };
}

class Controller {
  constructor(initialState) {
    this.currentState = initialState;
  }

  cloneState(state) {
    const clone = {};

    clone.highResolution = state.highResolution;
    clone.placeholder = state.placeholder;
    clone.populationSize = state.populationSize;
    clone.mutationRate = state.mutationRate;

    clone.currentMode = state.currentMode;

    clone.galleryLoaded = state.galleryLoaded;
    clone.galleryOldestToDisplay = state.galleryOldestToDisplay;
    clone.galleryItems = state.galleryItems;
    clone.galleryDisplaySize = state.galleryDisplaySize;

    clone.previouslySelectedGenotypes = state.previouslySelectedGenotypes;
    clone.selectedIndices = state.selectedIndices;
    clone.scriptId = state.scriptId;
    clone.script = state.script;
    clone.genotypes = state.genotypes;
    clone.traits = state.traits;

    return clone;
  }

  async applySetMode(state, { mode }) { // note: this doesn't need to be async?
    const newState = this.cloneState(state);
    newState.currentMode = mode;

    this.currentState = newState;
    return this.currentState;
  }

  async applySetGenotype(state, { genotype }) {
    const newState = this.cloneState(state);
    newState.genotype = genotype;

    this.currentState = newState;
    return this.currentState;
  }

  async applySetScript(state, { script }) {

    const newState = this.cloneState(state);
    newState.script = script;

    const { validTraits, traits } = await Job.request(jobBuildTraits, {
      script: newState.script
    });

    if (validTraits) {
      newState.traits = traits;
    } else {
      newState.traits = [];
    }

    this.currentState = newState;
    return this.currentState;
  }

  async applySetScriptId(state, { id }) {
    const newState = this.cloneState(state);
    newState.scriptId = id;

    this.currentState = newState;
    return this.currentState;
  }

  async applySetSelectedIndices(state, { selectedIndices }) {
    const newState = this.cloneState(state);
    newState.selectedIndices = selectedIndices || [];

    this.currentState = newState;
    return this.currentState;
  }

  // todo: should populationSize be passed in the action?
  async applyInitialGeneration(state) {
    const newState = this.cloneState(state);
    let { genotypes } = await Job.request(jobInitialGeneration, {
      traits: newState.traits,
      populationSize: newState.populationSize
    });

    newState.genotypes = genotypes;
    newState.previouslySelectedGenotypes = [];
    newState.selectedIndices = [];

    this.currentState = newState;
    return this.currentState;
  }

  async applyGalleryOldestToDisplay(state, { oldestId }) {
    const newState = this.cloneState(state);
    newState.galleryOldestToDisplay = oldestId;

    this.currentState = newState;
    return this.currentState;
  }

  async applySetGalleryItems(state, { galleryItems }) {
    const newState = this.cloneState(state);

    newState.galleryItems = {};
    galleryItems.forEach(item => {
      newState.galleryItems[item.id] = item;
    });
    if (galleryItems.length === 0)  {
      console.error("galleryItems is empty?");
    }

    newState.galleryLoaded = true;
    newState.galleryOldestToDisplay = galleryItems[0].id;

    this.currentState = newState;
    return this.currentState;
  }

  async applyShuffleGeneration(state, { rng }) {
    const newState = this.cloneState(state);
    const prev = newState.previouslySelectedGenotypes;

    if (prev.length === 0) {
      const s = await this.applyInitialGeneration(newState);

      this.currentState = s;
      return this.currentState;
    } else {
      const { genotypes } = await Job.request(jobNewGeneration, {
        genotypes: prev,
        populationSize: newState.populationSize,
        traits: newState.traits,
        mutationRate: newState.mutationRate,
        rng
      });

      newState.genotypes = genotypes;
      newState.selectedIndices = [];

      this.currentState = newState;
      return this.currentState;
    }
  }

  async applyNextGeneration(state, { rng }) {
    const newState = this.cloneState(state);
    const pg = newState.genotypes;
    const selectedIndices = newState.selectedIndices;
    const selectedGenos = [];

    for (let i = 0; i < selectedIndices.length; i++) {
      selectedGenos.push(pg[selectedIndices[i]]);
    }

    const { genotypes } = await Job.request(jobNewGeneration, {
      genotypes: selectedGenos,
      populationSize: newState.populationSize,
      traits: newState.traits,
      mutationRate: newState.mutationRate,
      rng
    });

    const previouslySelectedGenotypes = genotypes.slice(0, selectedIndices.length);

    newState.genotypes = genotypes;
    newState.previouslySelectedGenotypes = previouslySelectedGenotypes;
    newState.selectedIndices = [];

    this.currentState = newState;
    return this.currentState;
  }

  async applySetState(newState) {
    this.currentState = newState;
    return this.currentState;
  }

  logMode(mode) {
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
    log(`${actionSetMode}: ${name}`);
  }

  reducer(state, action) {
    switch (action.__type) {
    case actionSetMode:
      if (logToConsole) {
        this.logMode(action.mode);
      }
      return this.applySetMode(state, action);
    case actionSetGenotype:
      // SET_GENOTYPE is only used during the download dialog rendering
      // when the render button is clicked on an image in the evolve gallery
      //
      return this.applySetGenotype(state, action);
    case actionSetScript:
      return this.applySetScript(state, action);
    case actionSetScriptId:
      return this.applySetScriptId(state, action);
    case actionSetSelectedIndices:
      return this.applySetSelectedIndices(state, action);
    case actionInitialGeneration:
      return this.applyInitialGeneration(state);
    case actionNextGeneration:
      return this.applyNextGeneration(state, action);
    case actionShuffleGeneration:
      return this.applyShuffleGeneration(state, action);
    case actionSetState:
      log(`${actionSetState}: ${action.state}`);
      return this.applySetState(action.state);
    case actionGalleryOldestToDisplay:
      return this.applyGalleryOldestToDisplay(state, action);
    case actionSetGalleryItems:
      return this.applySetGalleryItems(state, action);
    default:
      return this.applySetState(state);
    }
  }

  getState() {
    return this.currentState;
  }

  dispatch(action, data) {
    if (data === undefined) {
      data = {};
    }
    data.__type = action;

    log(`dispatch: action = ${data.__type}`);
    return this.reducer(this.currentState, data);
  }
}

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
  if (logToConsole) {
    const before = performance.now();
    // return the 'stop' function
    return (id) => {
      const entry = useDBEntry(id);

      const after = performance.now();
      const duration = after - before;

      addTiming(entry, duration);

      const stats = getStats(entry);

      if (stats) {
        const eid = entry.id;
        const cur = stats.current.toFixed(printPrecision);
        const avg = stats.average.toFixed(printPrecision);
        const min = stats.min.toFixed(printPrecision);
        const max = stats.max.toFixed(printPrecision);
        const num = stats.num;

        const msg1 = `${eid}: ${cur}ms `;
        const msg2 = `(Mean: ${avg}, Min: ${min}, Max: ${max} N:${num})`;

        log(msg1 + msg2);
      }
    };
  } else {
    // do nothing
    return (id) => {};
  }
}

function getTimingEntry(id) {
  return db[id];
}

// --------------------------------------------------------------------------------
// SeniMode

const SeniMode = {
  gallery: 0,
  edit: 1,
  evolve: 2,
  numSeniModes: 3
};

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
        log(`worker ${self.id} initialised`);
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
        log(`assigning ${type} to worker ${worker.getId()}`);
      } else {
        worker = promiseWorkers[worker_id];
        log(`explicitly assigning ${type} to worker ${worker.getId()}`);
      }

      const result = await worker.postMessage(type, data);
      log(`result ${type} id:${worker.getId()}`);

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

    log(`workers::path = ${path}`);
    log(`workers::numWorkers = ${numWorkers}`);

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
// main

let gUI = {};
let gGLRenderer = undefined;

async function getJSON(url) {
  const res = await fetch(url);
  const json = await res.json();
  return json;
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
    log('unknown sen mode');
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

// needs the controller since imageLoadHandler rebinds controller.getState()
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

async function renderGeometryBuffers(meta, memory, buffers, imageElement, w, h) {
  const stopFn = startTiming();

  gGLRenderer.renderGeometryToTexture(meta, gConfig.render_texture_width, gConfig.render_texture_height, memory, buffers);
  gGLRenderer.renderTextureToScreen(meta, w, h);

  await gGLRenderer.copyImageDataTo(imageElement);

  stopFn("rendering all buffers");
}

async function renderGeometryBuffersSection(meta, memory, buffers, imageElement, w, h, section) {
  const stopFn = startTiming();

  gGLRenderer.renderGeometryToTexture(meta, gConfig.render_texture_width, gConfig.render_texture_height, memory, buffers, section);
  gGLRenderer.renderTextureToScreen(meta, w, h);

  await gGLRenderer.copyImageDataTo(imageElement);

  stopFn(`rendering all buffers for section ${section}`);
}

async function renderGeneration(state) {
  // TODO: stop generating  if the user has switched to edit mode
  const script = state.script;
  const genotypes = state.genotypes;
  const phenotypes = gUI.phenotypes;
  let hackTitle = "hackTitle";
  const promises = [];

  const stopFn = startTiming();

  for (let i = 0; i < phenotypes.length; i++) {
    const workerJob = renderScript({
      script,
      genotype: genotypes[i],
    }, phenotypes[i].imageElement);

    promises.push(workerJob);
  }

  await Promise.all(promises);

  stopFn(`renderGeneration-${hackTitle}`);
}

// invoked when the evolve screen is displayed after the edit screen
async function setupEvolveUI(controller) {
  await afterLoadingPlaceholderImages(controller.getState());
  const state = await controller.dispatch(actionInitialGeneration);
  // render the phenotypes
  updateSelectionUI(state);
  await renderGeneration(state);

  return state;
}

function showScriptInEditor(state) {
  const editor = gUI.editor;

  editor.getDoc().setValue(state.script);
  editor.refresh();
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
  return new Promise((resolve, reject) => {
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
  const re = /^[\w-]+.png/;

  if (url.match(re)) {
    // requesting a bitmap just by filename, so get it from /img/immutable/
    return "img/immutable/" + url;
  } else {
    // change nothing, try and fetch the url
    return url;
  }
}

async function renderScript(parameters, imageElement) {
  const stopFn = startTiming();

  let width = parameters.assumeWidth ? parameters.assumeWidth : imageElement.clientWidth;
  let height = parameters.assumeHeight ? parameters.assumeHeight : imageElement.clientHeight;

  let { meta, memory, buffers } = await renderJob(parameters);
  await renderGeometryBuffers(meta, memory, buffers, imageElement, width, height);

  if (meta.title === '') {
    stopFn(`renderScript`);
  } else {
    stopFn(`renderScript-${meta.title}`);
  }
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
    log(`worker ${__worker_id}: bitmap request: ${filename}`);
    const imageData = await loadBitmapImageData(filename);
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

async function renderEditorScript(state) {
  const imageElement = gUI.renderImage;
  await renderScript({
    script: state.script
  }, imageElement);
}

// function that takes a read-only state and updates the UI
//
async function updateUI(state) {
  showCurrentMode(state);

  switch (state.currentMode) {
  case SeniMode.gallery :
    break;
  case SeniMode.edit :
    fitRenderImgToRenderPanel();
    showScriptInEditor(state);
    await renderEditorScript(state);
    break;
  case SeniMode.evolve :
    // will only get here from History.restoreState
    // NOTE: the popstate event listener is handling this case
    break;
  default:
    log('unknown SeniMode');
    break;
  }
}

async function ensureMode(controller, mode) {
  if (mode === SeniMode.gallery && controller.getState().galleryLoaded === false) {
    // want to show the gallery but it hasn't been loaded yet. This occurs when
    // editing a particular sketch by loading it's id directly into the URL
    // e.g. http://localhost:3210/#61
    //
    await getGallery(controller);
  }

  if (controller.getState().currentMode !== mode) {
    const state = await controller.dispatch(actionSetMode, { mode });
    History.pushState(state);

    if (mode === SeniMode.evolve) {
      showCurrentMode(state);
      const latestState = await setupEvolveUI(controller);
      // make sure that the history for the first evolve generation
      // has the correct genotypes
      History.replaceState(latestState);
    } else {
      await updateUI(state);
    }
  }
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

function downloadDialogShow() {
  const container = document.getElementById('download-dialog');
  container.classList.remove('hidden');
}

function downloadDialogHide() {
  const container = document.getElementById('download-dialog');
  container.classList.add('hidden');
}

async function renderHighResSection(state, section) {
  const container = document.getElementById('download-dialog');
  const loader = document.getElementById('download-dialog-loader');
  const image = document.getElementById('render-img');

  container.classList.remove('hidden');
  loader.classList.remove('hidden');
  image.classList.add('hidden');

  const stopFn = startTiming();

  const { meta, memory, buffers } = await renderJob({
    script: state.script,
    genotype: undefined,
  });
  const [width, height] = state.highResolution;
  await renderGeometryBuffersSection(meta, memory, buffers, image, width, height, section);
  stopFn(`renderHighResSection-${meta.title}-${section}`);
  image.classList.remove('hidden');
  loader.classList.add('hidden');
}

// updates the controller's script variable and then generates the traits
// in a ww and updates the controller again
//
function setScript(controller, script) {
  return controller.dispatch(actionSetScript, { script });
}

async function showEditFromEvolve(controller, element) {
  const [index, _] = getPhenoIdFromDom(element);

  if (index !== -1) {
    const state = controller.getState();
    const genotypes = state.genotypes;
    const { script } = await Job.request(jobUnparse, {
      script: state.script,
      genotype: genotypes[index]
    });

    await controller.dispatch(actionSetScript, { script });
    await ensureMode(controller, SeniMode.edit);
  }
}

async function onNextGen(controller) {
  try {
    // get the selected genotypes for the next generation
    const populationSize = controller.getState().populationSize;
    const phenotypes = gUI.phenotypes;
    const selectedIndices = [];

    for (let i = 0; i < populationSize; i++) {
      const element = phenotypes[i].phenotypeElement;
      if (element.classList.contains('selected')) {
        selectedIndices.push(i);
      }
    }

    let state = await controller.dispatch(actionSetSelectedIndices, { selectedIndices });
    if (selectedIndices.length === 0) {
      // no phenotypes were selected
      return;
    }

    // update the last history state
    History.replaceState(state);

    showPlaceholderImages(state);

    state = await controller.dispatch(actionNextGeneration, { rng: 4242 });
    if (state === undefined) {
      return;
    }

    History.pushState(state);
    // render the genotypes
    updateSelectionUI(state);
    await renderGeneration(state);

  } catch(error) {
    // handle error
    console.error(`error of ${error}`);
  }
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
async function restoreEvolveUI(controller) {
  await afterLoadingPlaceholderImages(controller.getState());
  updateSelectionUI(controller.getState());
  // render the phenotypes
  await renderGeneration(controller.getState());
}

async function loadScriptWithId(controller, id) {
  const response = await fetch(`gallery/${id}`);
  const script = await response.text();

  await controller.dispatch(actionSetScript, { script });
  await controller.dispatch(actionSetScriptId, { id });
  await ensureMode(controller, SeniMode.edit);
}

async function showEditFromGallery(controller, element) {
  const [index, _] = getIdNumberFromDom(element, /gallery-item-(\d+)/);
  if (index !== -1) {
    await loadScriptWithId(controller, index);
  }
}

// take the height of the navbar into consideration
function resizeContainers() {
  const navbar = document.getElementById('seni-navbar');

  const edit = document.getElementById('edit-container');
  edit.style.height = `${window.innerHeight - navbar.offsetHeight}px`;

  const evolve = document.getElementById('evolve-container');
  evolve.style.height = `${window.innerHeight - navbar.offsetHeight}px`;

  fitRenderImgToRenderPanel();
}

async function evalMainScript(controller) {
  try {
    const script = getScriptFromEditor();
    const state = await controller.dispatch(actionSetScript, { script });
    await renderEditorScript(state);
  } catch (error) {
    console.error(`evalMainScript error: ${error}`);
  }
}

function createEditor(controller, editorTextArea) {
  const blockIndent = function (editor, from, to) {
    editor.operation(() => {
      for (let i = from; i < to; ++i) {
        editor.indentLine(i, 'smart');
      }
    });
  };

  const extraKeys = {
    'Ctrl-E': async () => {
      await evalMainScript(controller);
      return false;
    },
    'Ctrl-I': () => {
      const editor = gUI.editor;
      const numLines = editor.doc.size;
      blockIndent(editor, 0, numLines);
      log(`indenting ${numLines} lines`);
      return false;
    }
  };

  return Editor.createEditor(editorTextArea, {
    theme: 'default',
    extraKeys
  });
}

function ensureFilenameIsPNG(filename) {
  if(filename.endsWith(".png")) {
    return filename;
  } else {
    return filename + ".png";
  }
}

function fitRenderImgToRenderPanel() {
  let smallestDim = gUI.renderPanel.clientHeight;
  if (gUI.renderPanel.clientWidth < smallestDim) {
    smallestDim = gUI.renderPanel.clientWidth;
  }

  // reduce the dimensions by 5% to provide a nicer looking gap between the renderImg and renderPanel
  smallestDim *= gUI.renderImageSizeFactor;

  gUI.renderImage.width = smallestDim;
  gUI.renderImage.height = smallestDim;
}

function setupUI(controller) {
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
    renderPanel: d.getElementById('render-panel'),
    renderImageSizeFactor: 0.9,
    // console CodeMirror element in the edit screen
    editor: createEditor(controller, editorTextArea)
  };

  setupResizeability();

  showButtonsFor(SeniMode.gallery);

  addClickEvent('home', async event => {
    event.preventDefault();
    await ensureMode(controller, SeniMode.gallery);
  });

  addClickEvent('evolve-btn', async event => {
    try {
      event.preventDefault();
      // get the latest script from the editor
      const script = getScriptFromEditor();
      const state = await controller.dispatch(actionSetScript, { script });
      History.replaceState(state);
      await ensureMode(controller, SeniMode.evolve);
    } catch (error) {
      // handle error
      console.error(`evolve-btn:click : error of ${error}`);
    }
  });

  addClickEvent('render-btn', event => {
    downloadDialogShow();
    event.preventDefault();
  });

  addClickEvent('shuffle-btn', async event => {
    try {
      event.preventDefault();
      showPlaceholderImages(controller.getState());
      const rng = Math.random() * 9999;
      const state = await controller.dispatch(actionShuffleGeneration, { rng });
      updateSelectionUI(state);
      await renderGeneration(state);
    } catch (error) {
      // handle error
      console.error(`shuffle-btn:click : error of ${error}`);
    }
  });

  addClickEvent('eval-btn', async event => {
    event.preventDefault();
    await evalMainScript(controller);
  });

  addClickEvent('gallery-list', async event => {
    event.preventDefault();
    const target = event.target;
    if (target.classList.contains('show-edit')) {
      await showEditFromGallery(controller, target);
    }
  });

  addClickEvent('evolve-container', async event => {
    const target = event.target;
    const [index, phenoElement] = getPhenoIdFromDom(target);

    event.preventDefault();
    if (target.classList.contains('render')) {
      if (index !== -1) {
        const genotypes = controller.getState().genotypes;
        const genotype = genotypes[index];

        await controller.dispatch(actionSetGenotype, { genotype });

        downloadDialogShow();
      }
    } else if (target.classList.contains('edit')) {
      showEditFromEvolve(controller, target);
    } else {
      if (index !== -1) {
        phenoElement.classList.toggle('selected');
      }
    }
  });

  addClickEvent('next-btn', () => {
    onNextGen(controller);
  });

  addClickEvent('gallery-more-btn', () => {
    createGalleryDisplayChunk(controller);
  });

  addClickEvent('download-dialog-button-ok', async event => {
    // in an async function so call preventDefault before the first await
    event.preventDefault();

    const state = controller.getState();

    const loader = document.getElementById('download-dialog-loader');
    const image = document.getElementById('render-img');

    const image_resolution_elem = document.getElementById('download-dialog-field-resolution');
    let image_resolution = parseInt(image_resolution_elem.value, 10);

    const image_dim_elem = document.getElementById('download-dialog-field-tiledim');
    let image_dim = parseInt(image_dim_elem.value, 10);

    loader.classList.remove('hidden');

    const stopFn = startTiming();

    const { meta, memory, buffers } = await renderJob({
      script: state.script,
      genotype: state.genotype,
    });

    const [width, height] = [image_resolution, image_resolution];

    await renderGeometryBuffers(meta, memory, buffers, image, width, height);

    stopFn(`renderHighRes-${meta.title}`);

    loader.classList.add('hidden');

    const image_name_elem = document.getElementById('download-dialog-field-filename');
    const filename = ensureFilenameIsPNG(image_name_elem.value);
    gGLRenderer.localDownload(filename);

    // todo: is this the best place to reset the genotype?
    await controller.dispatch(actionSetGenotype, { genotype: undefined });
  });

  addClickEvent('download-dialog-button-close', event => {
    downloadDialogHide();
    event.preventDefault();
  });

  // Ctrl-D renders the next generation
  const dKey = 68;
  document.addEventListener('keydown', event => {
    if (event.ctrlKey && event.keyCode === dKey &&
        controller.getState().currentMode === SeniMode.evolve) {
      event.preventDefault();
      onNextGen(controller);
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

  const populationSize = controller.getState().populationSize;
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

  window.addEventListener('popstate', async event => {
    try {
      if (event.state) {
        const savedState = History.restoreState(event.state);
        const state = await controller.dispatch('SET_STATE', { state: savedState });
        await updateUI(state);
        if (state.currentMode === SeniMode.evolve) {
          await restoreEvolveUI(controller);
        }
      } else {
        // no event.state so behave as if the user has visited
        // the '/' of the state
        await ensureMode(controller, SeniMode.gallery);
      }
    } catch (error) {
        // handle error
        console.error(`${actionSetState}: error of ${error}`);
    }
  });

  return controller;
}

async function getGallery(controller) {
  const galleryItems = await getJSON('gallery');

  await controller.dispatch(actionSetGalleryItems, { galleryItems });
  await createGalleryDisplayChunk(controller);
}

async function createGalleryDisplayChunk(controller) {
  const state = controller.getState();

  const createGalleryElement = galleryItem => {
    const container = document.createElement('div');

    container.className = 'card-holder';
    container.id = `gallery-item-${galleryItem.id}`;

    container.innerHTML = `
      <a href="#" class="show-edit">
        <img class="card-image show-edit"
             id="gallery-image-${galleryItem.id}"
             src="${state.placeholder}">
      </a>
      <div class="card-action">
        <span>${galleryItem.name}</span>
      </div>`;

    return container;
  };

  const row = document.getElementById('gallery-list-cards');
  const assumeWidth = 300;
  const assumeHeight = 300;

  let least = Math.max(state.galleryOldestToDisplay - state.galleryDisplaySize, 0);

  const promises = [];

  for (let i=state.galleryOldestToDisplay; i>least; i--) {
    const item = state.galleryItems[i];
    const e = createGalleryElement(item);
    row.appendChild(e);

    const workerJob = renderScript({
      script: item.script,
      assumeWidth,
      assumeHeight
    }, document.getElementById(`gallery-image-${item.id}`));

    promises.push(workerJob);
  }
  // console.log(`oldest id to display is now ${least}`);
  if (least === 0) {
    // hide the button
    document.getElementById('gallery-more-btn').classList.add('hidden');;
  }

  await Promise.all(promises);
  await controller.dispatch(actionGalleryOldestToDisplay, { oldestId: least});
}

function allocateWorkers(state) {
  const defaultNumWorkers = 4;
  let numWorkers = navigator.hardwareConcurrency || defaultNumWorkers;
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

async function loadShaders(scriptUrls) {
  const fetchPromises = scriptUrls.map(s => fetch(s));
  const responses = await Promise.all(fetchPromises);

  const textPromises = responses.map(r => r.text());
  const shaders = await Promise.all(textPromises);

  const res = {};
  for (const [i, url] of scriptUrls.entries()) {
    res[url] = shaders[i];
  }

  return res;
}

async function main() {
  const state = createInitialState();
  const controller = new Controller(state);

  allocateWorkers(state);

  const canvasElement = document.getElementById('render-canvas');

  // load the shaders asynchronously here as constructors can't do that.
  //
  const shaders = await loadShaders(['shader/main-vert.glsl',
                                     'shader/main-frag.glsl',
                                     'shader/blit-vert.glsl',
                                     'shader/blit-frag.glsl']);
  gGLRenderer = new GLRenderer(canvasElement, shaders);

  try {
    await gGLRenderer.loadTexture('img/texture.png');

    setupUI(controller);

    const matched = window.location.hash.match(/^\#(\d+)/);
    if (window.location.pathname === '/' && matched) {
      const id = parseInt(matched[1], 10);
      await loadScriptWithId(controller, id);
    } else {
      await ensureMode(controller, SeniMode.gallery);
    }
  } catch (error) {
    console.error(error);
  }
}

document.addEventListener('DOMContentLoaded', () => {
  compatibilityHacks();
  main();
});
