export class Buffer {
    constructor(glContainer) {
        // each buffer can hold 1000 'items' where an item is a vertex, colour etc
        this.bufferSize = 1000; 
        this.vertexItemSize = 3; // xyz
        this.colourItemSize = 4; // rgba
        this.vertexBuffer = new Float32Array(this.vertexItemSize * this.bufferSize);
        this.colourBuffer = new Float32Array(this.colourItemSize * this.bufferSize);

        // the level of both the vertex and colour buffer
        // to find the actual index position multiply bufferLevel
        // by the relevant itemSize of the buffer
        this.bufferLevel = 0;

        var {glVertexBuffer, glColourBuffer} = initGLBuffers(glContainer.gl);

        this.glVertexBuffer = glVertexBuffer;
        this.glColourBuffer = glColourBuffer;
    }

    /**
     * make sure the buffer has enough space to add n vertices 
     * which will be rendered as a triangle strip
     * @param numVertices
     * @param p0 the first vertex position
     */
    prepareToAddTriangleStrip(glContainer, numVertices, p0) {

        if(this.bufferLevel >= this.bufferSize - (numVertices + 2)) {
            this.flushTriangles(glContainer);
        }

        if(this.bufferLevel !== 0) {
            // add two vertex entries which will form degenerate triangles
            var lastVertexIndex = (this.bufferLevel - 1) * this.vertexItemSize;
            this.addVertex([this.vertexBuffer[lastVertexIndex + 0],
                            this.vertexBuffer[lastVertexIndex + 1]],
                           [0, 0, 0, 0]);

            this.addVertex(p0, [0, 0, 0, 0]);

            // Note: still need to call addVertex on the first
            // vertex when we 'really' render the strip
        }
    }

    /**
     * this assumes that the buffer will have enough space to add the vertex
     * @param glContainer
     * @param p
     * @param c
     */
    addVertex(p, c) {
        var bl = this.bufferLevel * this.vertexItemSize;
        this.vertexBuffer[bl + 0] = p[0];
        this.vertexBuffer[bl + 1] = p[1];
        this.vertexBuffer[bl + 2] = 0;

        bl = this.bufferLevel * this.colourItemSize;
        this.colourBuffer[bl + 0] = c[0];
        this.colourBuffer[bl + 1] = c[1];
        this.colourBuffer[bl + 2] = c[2];
        this.colourBuffer[bl + 3] = c[3];

        this.bufferLevel += 1;
    }

    flushTriangles(glContainer) {

        if(this.bufferLevel === 0) {
            return;
        }

        var gl = glContainer.gl;
        var shaderProgram = glContainer.shaderProgram;

        var glVertexBuffer = this.glVertexBuffer;
        var glColourBuffer = this.glColourBuffer;

        gl.bindBuffer(gl.ARRAY_BUFFER, glVertexBuffer);
        gl.bufferData(gl.ARRAY_BUFFER, this.vertexBuffer, gl.STATIC_DRAW);
        gl.vertexAttribPointer(shaderProgram.positionAttribute,
                               this.vertexItemSize, gl.FLOAT, false, 0, 0);

        gl.bindBuffer(gl.ARRAY_BUFFER, glColourBuffer);
        gl.bufferData(gl.ARRAY_BUFFER, this.colourBuffer, gl.STATIC_DRAW);
        gl.vertexAttribPointer(shaderProgram.colourAttribute,
                               this.colourItemSize, gl.FLOAT, false, 0, 0);

        gl.drawArrays(gl.TRIANGLE_STRIP, 0, this.bufferLevel);

        this.bufferLevel = 0;
    }
}


function initGLBuffers(gl) {

    var glVertexBuffer = gl.createBuffer();

    var glColourBuffer = gl.createBuffer();

    return {glVertexBuffer: glVertexBuffer, glColourBuffer: glColourBuffer};
}
