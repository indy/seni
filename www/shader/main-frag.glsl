// pre-multiply the alpha in the shader
// see http://www.realtimerendering.com/blog/gpus-prefer-premultiplication/
// this needs to happen in linear colour space

precision highp float;

varying vec4 vColour;
varying vec2 vTextureCoord;
varying vec2 vWorldPos;

uniform sampler2D brush;
uniform sampler2D mask;

uniform float canvas_dim;
uniform bool maskInvert;
uniform bool uOutputLinearColourSpace;

// https://en.wikipedia.org/wiki/SRGB
vec3 srgb_to_linear(vec3 srgb) {
  float a = 0.055;
  float b = 0.04045;
  vec3 linear_lo = srgb / 12.92;
  vec3 linear_hi = pow((srgb + vec3(a)) / (1.0 + a), vec3(2.4));
  return vec3(
              srgb.r > b ? linear_hi.r : linear_lo.r,
              srgb.g > b ? linear_hi.g : linear_lo.g,
              srgb.b > b ? linear_hi.b : linear_lo.b);
}


void main(void) {
  vec4 brushTex = texture2D(brush, vTextureCoord);

  // note: you _never_ want uOutputLinearColourSpace to be set to true
  // it's only here because some of the older sketchs didn't correctly
  // convert from linear colour space to sRGB colour space during rendering
  // and this shader needs to reproduce them as intended at time of creation
  //
  if (uOutputLinearColourSpace) {
    // purely for legacy, no new development will occur in this branch of the if conditional
    //
    gl_FragColor.r = brushTex.r * vColour.r * vColour.a;
    gl_FragColor.g = brushTex.r * vColour.g * vColour.a;
    gl_FragColor.b = brushTex.r * vColour.b * vColour.a;
    gl_FragColor.a = brushTex.r * vColour.a;
  } else {
    // all modern scripts should assume correct colour space conversions
    vec4 linearColour = vec4(srgb_to_linear(vColour.rgb), 1.0);
    vec4 maskTex = texture2D(mask, vWorldPos / canvas_dim);

    float maskVal = maskInvert ? 1.0 - maskTex.r : maskTex.r;

    gl_FragColor.r = brushTex.r * linearColour.r * vColour.a * maskVal;
    gl_FragColor.g = brushTex.r * linearColour.g * vColour.a * maskVal;
    gl_FragColor.b = brushTex.r * linearColour.b * vColour.a * maskVal;
    gl_FragColor.a = brushTex.r * linearColour.a * vColour.a * maskVal;
  }
}
