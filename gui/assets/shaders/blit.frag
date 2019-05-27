#version 330 core

in VS_OUTPUT {
    vec2 TextureCoord;
} IN;

// Values that stay constant for the whole mesh.
uniform sampler2D myTextureSampler;

out vec4 Colour;

void main()
{
  Colour = texture( myTextureSampler, IN.TextureCoord );
}
