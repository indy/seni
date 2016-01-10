/*
 Seni
 Copyright (C) 2016 Inderjit Gill <email@indy.io>

 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.

 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with this program. If not, see <http://www.gnu.org/licenses/>.
 */


// each buffer can hold bufferSize 'items' where each item is a vertex+colour
const bufferSize = 1000;
// note: using different values for bufferSize doesn't affect render time
// (tried with values of 100, 1000, 10000, 100000)
const vertexItemSize = 2; // xy
const colourItemSize = 4; // rgba

export default class RenderPacket {
  constructor() {
    // pass in sizes so that GLExec can access them
    this.vertexItemSize = vertexItemSize;
    this.colourItemSize = colourItemSize;
    this.vertexBuffer = new Float32Array(vertexItemSize * bufferSize);
    this.colourBuffer = new Float32Array(colourItemSize * bufferSize);
    // the level of both the vertex and colour buffer
    // to find the actual index position multiply bufferLevel
    // by the relevant itemSize of the buffer
    this.bufferLevel = 0;
  }

  // can the number of vertices fit into this RenderPacket?
  canVerticesFit(numVertices) {
    return this.bufferLevel < bufferSize - (numVertices + 2);
  }

  isRenderPacketEmpty() {
    return this.bufferLevel === 0;
  }

  formDegenerateTriangle(p0) {
    // add two vertex entries which will form degenerate triangles

    // get the index of the last vertex that was added
    const index = (this.bufferLevel - 1) * vertexItemSize;
    // just copy the previous entries
    // note: colour doesn't matter since these triangles won't be rendered
    this.addVertex([this.vertexBuffer[index + 0], this.vertexBuffer[index + 1]],
                   [0, 0, 0, 0]);

    // add the new vertex to complete the degenerate triangle
    this.addVertex(p0, [0, 0, 0, 0]);

    // Note: still need to call addVertex on the first
    // vertex when we 'really' render the strip
  }

  // appends the vertex with position p and colour c
  addVertex(p, c) {
    let bl = this.bufferLevel * vertexItemSize;
    this.vertexBuffer[bl + 0] = p[0];
    this.vertexBuffer[bl + 1] = p[1];

    bl = this.bufferLevel * colourItemSize;
    this.colourBuffer[bl + 0] = c[0];
    this.colourBuffer[bl + 1] = c[1];
    this.colourBuffer[bl + 2] = c[2];
    this.colourBuffer[bl + 3] = c[3];

    this.bufferLevel += 1;
  }
}
