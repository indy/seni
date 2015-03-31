/*jslint bitwise:true*/

import GLContainer from './GLContainer';
import Buffer from './Buffer';
import MatrixStack from './MatrixStack';

function initGLState(gl) {
  gl.clearColor(1.0, 1.0, 1.0, 1.0);

  gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
  //gl.clearColor(0.0, 0.0, 0.0, 1.0);

  // http://www.andersriggelsen.dk/glblendfunc.php
  //gl.blendFunc(gl.SRC_ALPHA, gl.ONE);
  //  gl.blendFunc(gl.GL_SRC_ALPHA, gl.GL_ONE_MINUS_SRC_ALPHA);
  //gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_DST_COLOR);

  //gl.blendFunc(gl.SRC_COLOR, gl.ONE_MINUS_SRC_COLOR);
  gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

  //  gl.blendFuncSeparate(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA,
  //                       gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
  gl.blendEquation(gl.FUNC_ADD);
  gl.enable(gl.BLEND);

  gl.disable(gl.DEPTH_TEST);
}

class Renderer {
  constructor(canvasElement) {

    this.glDomElement = document.getElementById(canvasElement);

    this.glContainer = new GLContainer(this.glDomElement);
    this.matrixStack = new MatrixStack();
    this.buffer = new Buffer(this.glContainer, this.matrixStack);

    this.mvMatrix = mat4.create();
    this.pMatrix = mat4.create();
    mat4.ortho(this.pMatrix, 0, 1000, 0, 1000, 10, -10);

    initGLState(this.glContainer.gl);

    //    this.matrixStack.translate(-40, -50);
    //    this.matrixStack.rotate(0.2);
  }

  getImageData() {
    return this.glDomElement.toDataURL();
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
    const glContainer = this.glContainer;
    const gl = glContainer.gl;
    const shaderProgram = glContainer.shaderProgram;

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

export default Renderer;
