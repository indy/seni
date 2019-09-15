attribute vec2 aVertexPosition;
attribute vec4 aVertexColour;
attribute vec2 aVertexTexture;

uniform mat4 uMVMatrix;
uniform mat4 uPMatrix;

varying vec4 vColour;
varying vec2 vTextureCoord;
varying vec2 vWorldPos;

void main(void) {
  gl_Position = uPMatrix * uMVMatrix * vec4(aVertexPosition, 0.0, 1.0);

  vColour = aVertexColour;
  vTextureCoord = aVertexTexture;
  vWorldPos = aVertexPosition;
}
