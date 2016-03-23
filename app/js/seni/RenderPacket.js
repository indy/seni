/*
 *  Seni
 *  Copyright (C) 2016 Inderjit Gill <email@indy.io>
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

// the size of transferrable data held by each RenderPacket
const renderPacketSize = 1 * 1024 * 1024; // 1MB

// number of float32 elements in each data type
const vertexItemSize = 2; // xy
const colourItemSize = 4; // rgba

const f32Size = 4; // 4 bytes in a float32

// each buffer can hold bufferSize 'items' where each item is a vertex+colour
const itemSize = f32Size * (vertexItemSize + colourItemSize);
const bufferSize = parseInt(renderPacketSize / itemSize, 10);

export default class RenderPacket {
  constructor() {
    // pass in sizes so that GLExec can access them
    this.vertexItemSize = vertexItemSize;
    this.colourItemSize = colourItemSize;

    this.abVertex = new ArrayBuffer(vertexItemSize * f32Size * bufferSize);
    this.abColour = new ArrayBuffer(colourItemSize * f32Size * bufferSize);

    this.vertexBuffer = new Float32Array(this.abVertex);
    this.colourBuffer = new Float32Array(this.abColour);
    // the level of both the vertex and colour buffer
    // to find the actual index position multiply bufferLevel
    // by the relevant itemSize of the buffer
    this.bufferLevel = 0;
    this.vertexBufferLevel = 0;
    this.colourBufferLevel = 0;
  }

  // can the number of vertices fit into this RenderPacket?
  canVerticesFit(numVertices) {
    return this.bufferLevel < bufferSize - (numVertices + 2);
  }

  getBufferLevel() {
    return this.bufferLevel;
  }

  isRenderPacketEmpty() {
    return this.bufferLevel === 0;
  }

  formDegenerateTriangle(p0) {
    // add two vertex entries which will form degenerate triangles

    // get the index of the last vertex that was added
    const index = this.vertexBufferLevel - vertexItemSize;
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
    this.vertexBuffer[this.vertexBufferLevel + 0] = p[0];
    this.vertexBuffer[this.vertexBufferLevel + 1] = p[1];

    this.colourBuffer[this.colourBufferLevel + 0] = c[0];
    this.colourBuffer[this.colourBufferLevel + 1] = c[1];
    this.colourBuffer[this.colourBufferLevel + 2] = c[2];
    this.colourBuffer[this.colourBufferLevel + 3] = c[3];

    this.bufferLevel += 1;
    this.vertexBufferLevel += vertexItemSize;
    this.colourBufferLevel += colourItemSize;
  }
}
