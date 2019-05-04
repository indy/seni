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
    this.glDomElement.toBlob(blob => {
      elem.src = window.URL.createObjectURL(blob);
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

function actionSetGenotype(state, { genotype }) {
  return new Promise((resolve, reject) => {
    const newState = cloneState(state);

    newState.genotype = genotype;
    resolveAsCurrentState(resolve, newState);
  });
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
    highResolution: [2048, 2048], // [4096, 4096],
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
    traits: [],

    genotype: undefined,
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
    case 'SET_GENOTYPE':
      // SET_GENOTYPE is only used during the high-res rendering
      // when the render button is clicked on an image in the evolve gallery
      //
      return actionSetGenotype(state, action);
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
        return;
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

      if (type === jobReceiveBitmapData) {
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

// a crappy findAllWorkers, use sparingly
function crappyFindAllWorkers() {
  return new Promise((resolve, reject) => {
    setTimeout(function go() {

      let allWorkers = [];

      for (let i=0;i<numWorkers;i++) {
        if (promiseWorkers[i].isInitialised() === true &&
            promiseWorkers[i].isWorking() === false) {
          allWorkers.push(promiseWorkers[i]);
        }
      }

      if (allWorkers.length === promiseWorkers.length) {
        for (let i=0;i<numWorkers;i++) {
          allWorkers[i].employ();
        }
        resolve(allWorkers);
        return;
      }

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

  broadcast: function(type, data) {
    return new Promise((resolve, reject) => {
      let workers = undefined;

      crappyFindAllWorkers().then(workers_ => {
        workers = workers_;

        let promises = [];
        for (let i = 0; i < workers.length; i++) {
          promises.push(workers[i].postMessage(type, data));
        }
        return Promise.all(promises);
      }).then(results => {
        for (let i = 0; i < workers.length; i++) {
          workers[i].release();
        }
        resolve(results);
      }).catch(error => {
        if (workers !== undefined) {
          for (let i = 0; i < workers.length; i++) {
            workers[i].release();
          }
        }
        // handle error
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
// jobTypes

const jobRender = 'RENDER';
const jobUnparse = 'UNPARSE';
const jobBuildTraits = 'BUILD_TRAITS';
const jobInitialGeneration = 'INITIAL_GENERATION';
const jobNewGeneration = 'NEW_GENERATION';
const jobGenerateHelp = 'GENERATE_HELP';
const jobSingleGenotypeFromSeed = 'SINGLE_GENOTYPE_FROM_SEED';
const jobSimplifyScript = 'SIMPLIFY_SCRIPT';
const jobReceiveBitmapData = 'RECEIVE_BITMAP_DATA';

// --------------------------------------------------------------------------------
// main

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

  const stopFn = startTiming();

  buffers.forEach(buffer => {
    gGLRenderer.drawBuffer(memory, buffer);
  });

  console.log(`${buffers.length} buffers`);
  stopFn("rendering all buffers", console);


  gGLRenderer.copyImageDataTo(imageElement);
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

  buffers.forEach(buffer => {
    gGLRenderer.drawBuffer(memory, buffer);
  });

  gGLRenderer.copyImageDataTo(imageElement);
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

function downloadDialog(state, genotype) {
  const container = document.getElementById('high-res-container');
  container.classList.remove('hidden');
}

function renderHighResSection(state, section) {
  const container = document.getElementById('high-res-container');
  const loader = document.getElementById('high-res-loader');
  const image = document.getElementById('render-img');

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
    loader.classList.add('hidden');
  }).catch(error => {
    // handle error
    console.error(`worker: error of ${error}`);
    console.error(error);
    image.classList.remove('hidden');
    loader.classList.add('hidden');
  });
}

function setGenotype(store, genotype) {
  return store.dispatch({type: 'SET_GENOTYPE', genotype});
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

function ensureFilenameIsPNG(filename) {
  if(filename.endsWith(".png")) {
    return filename;
  } else {
    return filename + ".png";
  }
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
    downloadDialog();
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

        setGenotype(store, genotype).then(() => {
          downloadDialog();
        });
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

  addClickEvent('high-res-download', event => {
    const state = store.getState();

    const loader = document.getElementById('high-res-loader');
    const image = document.getElementById('render-img');

    const image_resolution_elem = document.getElementById('high-res-resolution');
    let image_resolution = parseInt(image_resolution_elem.value, 10);

    const image_dim_elem = document.getElementById('high-res-tiledim');
    let image_dim = parseInt(image_dim_elem.value, 10);

    loader.classList.remove('hidden');

    const stopFn = startTiming();

    Job.request(jobRender, {
      script: state.script,
      scriptHash: state.scriptHash,
      genotype: state.genotype
    }).then(({ title, memory, buffers }) => {
      // const [width, height] = state.highResolution;
      const [width, height] = [image_resolution, image_resolution];

      renderGeometryBuffers(memory, buffers, image, width, height);

      stopFn(`renderHighRes-${title}`, console);

      loader.classList.add('hidden');

      const image_name_elem = document.getElementById('high-res-filename');
      const filename = ensureFilenameIsPNG(image_name_elem.value);
      gGLRenderer.localDownload(filename);

      // todo: is this the best place to reset the genotype?
      setGenotype(store, undefined);

    }).catch(error => {
      // handle error
      console.error(`worker: error of ${error}`);
      console.error(error);
      loader.classList.add('hidden');
    });

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

      // imageData.data = UInt8ClampedArray (width x height x 4)
      // imageData.width
      // imageData.height

      resolve(imageData);
    };
    img.onerror = () => {
      reject();
    };

    img.src = url;
  });
}

function devBitmapImageLoader(filename) {
  loadBitmapImageData(filename)
    .then(imageData => {
      return Job.broadcast(jobReceiveBitmapData, { filename, imageData });
    })
    .then(results => {
      console.log(results);
    })
    .catch(error => {
      // handle error
      console.error(`worker: error of ${error}`);
    });
}

function main() {
  setupResizeability();

  const state = createInitialState();
  const store = createStore(state);

  allocateWorkers(state);

  devBitmapImageLoader('img/bitmap1.png');

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
