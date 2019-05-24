#version 330 core

layout (location = 0) in vec2 Position;
layout (location = 1) in vec4 Colour;
layout (location = 2) in vec2 UV;

uniform mat4 uMVMatrix;
uniform mat4 uPMatrix;

out VS_OUTPUT {
    vec4 Colour;
    vec2 TextureCoord;
} OUT;

void main()
{
  gl_Position = uPMatrix * uMVMatrix * vec4(Position, 0.0, 1.0);
  OUT.Colour = Colour;
  OUT.TextureCoord = UV;
}
