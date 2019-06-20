#version 330 core

in VS_OUTPUT {
    vec2 TextureCoord;
} IN;

// Values that stay constant for the whole mesh.
uniform sampler2D myTextureSampler;

out vec4 Colour;


vec3 linear_to_srgb(vec3 linear) {
    float a = 0.055;
    float b = 0.0031308;
    vec3 srgb_lo = 12.92 * linear;
    vec3 srgb_hi = (1.0 + a) * pow(linear, vec3(1.0/2.4)) - vec3(a);
    return vec3(
        linear.r > b ? srgb_hi.r : srgb_lo.r,
        linear.g > b ? srgb_hi.g : srgb_lo.g,
        linear.b > b ? srgb_hi.b : srgb_lo.b);
}

// https://twitter.com/jimhejl/status/633777619998130176
vec3 ToneMapFilmic_Hejl2015(vec3 hdr, float whitePt) {
    vec4 vh = vec4(hdr, whitePt);
    vec4 va = 1.425 * vh + 0.05;
    vec4 vf = (vh * va + 0.004) / (vh * (va + 0.55) + 0.0491) - 0.0821;
    return vf.rgb / vf.www;
}



void main()
{
  // Colour = texture( myTextureSampler, IN.TextureCoord );

  // Colour = texture( myTextureSampler, IN.TextureCoord );
  // Colour = vec4(clamp(Colour.r, 0.0, 1.0), clamp(Colour.g, 0.0, 1.0), clamp(Colour.b, 0.0, 1.0), 1.0);

  // vec4 pieceColour = texture( myTextureSampler, IN.TextureCoord );
  // Colour = vec4(linear_to_srgb(ToneMapFilmic_Hejl2015(pieceColour.rgb, 1.0)), 1.0);

  vec4 pieceColour = texture( myTextureSampler, IN.TextureCoord );
  Colour = vec4(linear_to_srgb(pieceColour.rgb), 1.0);

  // vec4 pieceColour = texture( myTextureSampler, IN.TextureCoord );
  // Colour = vec4(ToneMapFilmic_Hejl2015(pieceColour.rgb, 1.0), 1.0);
}
