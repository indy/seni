attribute vec2 pos;
attribute vec4 col;
attribute vec2 uv;

uniform mat4 proj_matrix;

varying vec4 frag_col;
varying vec2 frag_uv;
varying vec2 world_pos;

void main(void) {
  gl_Position = proj_matrix * vec4(pos, 0.0, 1.0);

  frag_col = col;
  frag_uv = uv;
  world_pos = pos;
}
