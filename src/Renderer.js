import { GLContainer } from './GLContainer';
import { Buffer } from './Buffer';

export class Renderer {
    constructor(canvasElement) {
        this.glContainer = new GLContainer(canvasElement);
        this.buffer = new Buffer(this.glContainer);

        var mvMatrix = mat4.create();
        mat4.identity(mvMatrix);
        this.mvMatrix = mvMatrix;

        this.pMatrix = mat4.create();
        mat4.ortho(this.pMatrix, 0, 1000, 0, 1000, 10, -10);

        initGLState(this.glContainer.gl);
    }

    preDrawScene() {
        var glContainer = this.glContainer;
        var gl = glContainer.gl;
        var shaderProgram = glContainer.shaderProgram;

        gl.viewport(0, 0, gl.viewportWidth, gl.viewportHeight);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);


        gl.uniformMatrix4fv(shaderProgram.pMatrixUniform, false, this.pMatrix);
        gl.uniformMatrix4fv(shaderProgram.mvMatrixUniform, false, this.mvMatrix);

    }

    drawScene() {
        var buffer = this.buffer;
        var glContainer = this.glContainer;


        buffer.prepareToAddTriangleStrip(glContainer, 6, [100, 100]);

        buffer.addVertex([100, 100], [1.0, 0.0, 0.0, 1.0]);
        buffer.addVertex([200, 100], [1.0, 0.2, 0.0, 1.0]);
        buffer.addVertex([100, 200], [1.0, 0.4, 0.0, 1.0]);

        buffer.addVertex([200, 200], [1.0, 0.0, 0.2, 1.0]);
        buffer.addVertex([100, 300], [1.0, 0.0, 0.4, 1.0]);
        buffer.addVertex([200, 300], [1.0, 0.0, 0.6, 1.0]);


        buffer.prepareToAddTriangleStrip(glContainer, 6, [100, 400]);

        buffer.addVertex([100, 400], [1.0, 0.0, 0.2, 1.0]);
        buffer.addVertex([200, 400], [1.0, 0.0, 0.4, 1.0]);
        buffer.addVertex([100, 500], [1.0, 0.0, 0.6, 1.0]);

        buffer.addVertex([200, 500], [1.0, 0.0, 0.2, 1.0]);
        buffer.addVertex([100, 600], [1.0, 0.0, 0.4, 1.0]);
        buffer.addVertex([200, 600], [1.0, 0.0, 0.6, 1.0]);
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
