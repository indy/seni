attribute vec2 aVertexPosition;
attribute vec2 aVertexTexture;

uniform mat4 uMVMatrix;
uniform mat4 uPMatrix;

varying vec2 vTextureCoord;

void main(void) {
  gl_Position = uPMatrix * uMVMatrix * vec4(aVertexPosition, 0.0, 1.0);
  vTextureCoord = aVertexTexture;
}
