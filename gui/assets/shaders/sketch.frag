#version 330 core

in VS_OUTPUT {
    vec4 Colour;
    vec2 TextureCoord;
} IN;

// Values that stay constant for the whole mesh.
uniform sampler2D myTextureSampler;

layout(location = 0) out vec4 Colour;


void main()
{
  vec4 tex = texture( myTextureSampler, IN.TextureCoord );

  // pre-multiply the alpha in the shader
  Colour.r = tex.r * IN.Colour.r * IN.Colour.a;
  Colour.g = tex.r * IN.Colour.g * IN.Colour.a;
  Colour.b = tex.r * IN.Colour.b * IN.Colour.a;

  Colour.a = tex.r * IN.Colour.a;
}
