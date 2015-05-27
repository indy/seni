class RenderPacket {
  constructor() {
    // buffer code...
    // each buffer can hold 1000 'items' where an item is a vertex, colour etc
    this.bufferSize = 1000;
    this.vertexItemSize = 2; // xy
    this.colourItemSize = 4; // rgba
    this.vertexBuffer = new Float32Array(this.vertexItemSize * this.bufferSize);
    this.colourBuffer = new Float32Array(this.colourItemSize * this.bufferSize);
    // the level of both the vertex and colour buffer
    // to find the actual index position multiply bufferLevel
    // by the relevant itemSize of the buffer
    this.bufferLevel = 0;
  }

  // can the number of vertices fit into this RenderPacket?
  canVerticesFit(numVertices) {
    return this.bufferLevel < this.bufferSize - (numVertices + 2);
  }

  isRenderPacketEmpty() {
    return this.bufferLevel === 0;
  }

  appendDegenerateVertices(p0) {
    // add two vertex entries which will form degenerate triangles
    const lastVertexIndex = (this.bufferLevel - 1) * this.vertexItemSize;
    // just copy the previous entries
    // note: colour doesn't matter since these triangles won't be rendered
    this.appendVertex(
      [this.vertexBuffer[lastVertexIndex + 0],
       this.vertexBuffer[lastVertexIndex + 1]],
      [0, 0, 0, 0]);

    this.appendVertex(p0, [0, 0, 0, 0]);

    // Note: still need to call addVertex on the first
    // vertex when we 'really' render the strip
  }

  // appends the vertex with position p and colour c
  appendVertex(p, c) {
    let bl = this.bufferLevel * this.vertexItemSize;
    this.vertexBuffer[bl + 0] = p[0];
    this.vertexBuffer[bl + 1] = p[1];

    bl = this.bufferLevel * this.colourItemSize;
    this.colourBuffer[bl + 0] = c[0];
    this.colourBuffer[bl + 1] = c[1];
    this.colourBuffer[bl + 2] = c[2];
    this.colourBuffer[bl + 3] = c[3];

    this.bufferLevel += 1;
  }
}

export default RenderPacket;
