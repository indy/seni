#version 330 core

in vec2 Position;
in vec2 UV;

uniform mat4 uMVMatrix;
uniform mat4 uPMatrix;

out VS_OUTPUT {
    vec2 TextureCoord;
} OUT;

void main()
{
  gl_Position = uPMatrix * uMVMatrix * vec4(Position, 0.0, 1.0);
  OUT.TextureCoord = UV;
}
