import { GLContainer } from './GLContainer';
import { Buffer } from './Buffer';
import { MatrixStack } from './MatrixStack';
import { renderBezier, getBezierFn } from './shapes';

export class Renderer {
  constructor(canvasElement) {
    this.glContainer = new GLContainer(canvasElement);
    this.matrixStack = new MatrixStack();
    this.buffer = new Buffer(this.glContainer, this.matrixStack);

    this.mvMatrix = mat4.create();
    this.pMatrix = mat4.create();
    mat4.ortho(this.pMatrix, 0, 1000, 0, 1000, 10, -10);

    initGLState(this.glContainer.gl);


//    this.matrixStack.translate(-40, -50);
//    this.matrixStack.rotate(0.2);
  }

  getMatrixStack() {
    return this.matrixStack;
  }

  getGLContainer() {
    return this.glContainer;
  }

  getBuffer() {
    return this.buffer;
  }

  preDrawScene() {
    var glContainer = this.glContainer;
    var gl = glContainer.gl;
    var shaderProgram = glContainer.shaderProgram;

    gl.viewport(0, 0, gl.viewportWidth, gl.viewportHeight);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);


    gl.uniformMatrix4fv(shaderProgram.pMatrixUniform, false, this.pMatrix);
    gl.uniformMatrix4fv(shaderProgram.mvMatrixUniform, false, this.mvMatrix);

    this.matrixStack.reset();
  }

  postDrawScene() {
    this.buffer.flushTriangles(this.glContainer);
  }
}

function initGLState(gl) {
  gl.clearColor(1.0, 1.0, 1.0, 1.0);
  //gl.clearColor(0.0, 0.0, 0.0, 1.0);

  // http://www.andersriggelsen.dk/glblendfunc.php
  //gl.blendFunc(gl.SRC_ALPHA, gl.ONE);
  //  gl.blendFunc(gl.GL_SRC_ALPHA, gl.GL_ONE_MINUS_SRC_ALPHA);
  //gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_DST_COLOR);
  gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

  gl.enable(gl.BLEND);
  gl.disable(gl.DEPTH_TEST);
}
