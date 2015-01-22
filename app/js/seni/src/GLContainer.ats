export class GLContainer {

  constructor(canvasId) {

    var canvas = document.getElementById(canvasId);

    var gl = initGL(canvas);
    this.gl = gl;
    this.shaderProgram = initShaders(gl, {fragment: "shader-fs", vertex: "shader-vs"});
  }
}

function initGL(canvas) {
  try {
    var gl = canvas.getContext("experimental-webgl");
    // commented out because of jshint
    //    if (!gl) {
    //alert("Could not initialise WebGL, sorry :-(");
    //    }
    gl.viewportWidth = canvas.width;
    gl.viewportHeight = canvas.height;

    return gl;
  } catch (e) {
  }
}

function compileShader(gl, type, src) {
  var shader = gl.createShader(type);
  gl.shaderSource(shader, src);
  gl.compileShader(shader);

  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    //alert(gl.getShaderInfoLog(shader));
    return null;
  }
  return shader;
}


function initShaders(gl, {fragment, vertex}) {

  var shaderProgram = gl.createProgram();

  var fragmentSrc = `
  precision mediump float;
  varying vec4 vColor;
  
  void main(void) {
    gl_FragColor = vColor;
  }
  `;

  var vertexSrc = `
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
  

  var vertexShader = compileShader(gl, gl.VERTEX_SHADER, vertexSrc);
  var fragmentShader = compileShader(gl, gl.FRAGMENT_SHADER, fragmentSrc);

  gl.attachShader(shaderProgram, vertexShader);
  gl.attachShader(shaderProgram, fragmentShader);

  gl.linkProgram(shaderProgram);

  // commented out because of jshint
  //  if (!gl.getProgramParameter(shaderProgram, gl.LINK_STATUS)) {
  //alert("Could not initialise shaders");
  //  }

  gl.useProgram(shaderProgram);

  shaderProgram.positionAttribute =
    gl.getAttribLocation(shaderProgram, "aVertexPosition");
  gl.enableVertexAttribArray(shaderProgram.positionAttribute);

  shaderProgram.colourAttribute =
    gl.getAttribLocation(shaderProgram, "aVertexColor");
  gl.enableVertexAttribArray(shaderProgram.colourAttribute);

  shaderProgram.pMatrixUniform =
    gl.getUniformLocation(shaderProgram, "uPMatrix");
  shaderProgram.mvMatrixUniform =
    gl.getUniformLocation(shaderProgram, "uMVMatrix");

  return shaderProgram;
}

