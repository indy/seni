attribute vec2 pos;
attribute vec2 uv;

uniform mat4 proj_matrix;

varying vec2 frag_uv;

void main(void) {
  gl_Position = proj_matrix * vec4(pos, 0.0, 1.0);
  frag_uv = uv;
}
