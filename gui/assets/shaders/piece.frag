#version 330 core

in VS_OUTPUT {
    vec4 Colour;
    vec2 TextureCoord;
} IN;

// Values that stay constant for the whole mesh.
uniform sampler2D myTextureSampler;

out vec4 Colour;

void main()
{
  vec4 tex = texture( myTextureSampler, IN.TextureCoord );

  Colour.r = tex.r * IN.Colour.r;
  Colour.g = tex.r * IN.Colour.g;
  Colour.b = tex.r * IN.Colour.b;
  Colour.a = tex.r * IN.Colour.a;
}
