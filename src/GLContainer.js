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

function getShader(gl, id) {
  var shaderScript = document.getElementById(id);
  if (!shaderScript) {
    return null;
  }

  var str = "";
  var k = shaderScript.firstChild;
  while (k) {
    if (k.nodeType == 3) {
      str += k.textContent;
    }
    k = k.nextSibling;
  }

  var shader;
  if (shaderScript.type == "x-shader/x-fragment") {
    shader = gl.createShader(gl.FRAGMENT_SHADER);
  } else if (shaderScript.type == "x-shader/x-vertex") {
    shader = gl.createShader(gl.VERTEX_SHADER);
  } else {
    return null;
  }

  gl.shaderSource(shader, str);
  gl.compileShader(shader);

  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    //alert(gl.getShaderInfoLog(shader));
    return null;
  }

  return shader;
}


function initShaders(gl, {fragment, vertex}) {

  var shaderProgram = gl.createProgram();

  var fragmentShader = getShader(gl, fragment);
  var vertexShader = getShader(gl, vertex);

  gl.attachShader(shaderProgram, vertexShader);
  gl.attachShader(shaderProgram, fragmentShader);
  gl.linkProgram(shaderProgram);

  // commented out because of jshint
  //  if (!gl.getProgramParameter(shaderProgram, gl.LINK_STATUS)) {
    //alert("Could not initialise shaders");
//  }

  gl.useProgram(shaderProgram);

  shaderProgram.positionAttribute = gl.getAttribLocation(shaderProgram, "aVertexPosition");
  gl.enableVertexAttribArray(shaderProgram.positionAttribute);

  shaderProgram.colourAttribute = gl.getAttribLocation(shaderProgram, "aVertexColor");
  gl.enableVertexAttribArray(shaderProgram.colourAttribute);

  shaderProgram.pMatrixUniform = gl.getUniformLocation(shaderProgram, "uPMatrix");
  shaderProgram.mvMatrixUniform = gl.getUniformLocation(shaderProgram, "uMVMatrix");

  return shaderProgram;
}

