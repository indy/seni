// pre-multiply the alpha in the shader
// see http://www.realtimerendering.com/blog/gpus-prefer-premultiplication/
// this needs to happen in linear colour space

precision highp float;

varying vec4 vColour;
varying vec2 vTextureCoord;

uniform sampler2D texture;
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
  vec4 tex = texture2D(texture, vTextureCoord);

  // note: you _never_ want uOutputLinearColourSpace to be set to true
  // it's only here because some of the older sketchs didn't correctly
  // convert from linear colour space to sRGB colour space during rendering
  // and this shader needs to reproduce them as intended at time of creation
  //
  if (uOutputLinearColourSpace) {
    gl_FragColor.r = tex.r * vColour.r * vColour.a;
    gl_FragColor.g = tex.r * vColour.g * vColour.a;
    gl_FragColor.b = tex.r * vColour.b * vColour.a;
    gl_FragColor.a = tex.r * vColour.a;
  } else {
    vec4 linearColour = vec4(srgb_to_linear(vColour.rgb), vColour.a);
    gl_FragColor.r = tex.r * linearColour.r * linearColour.a;
    gl_FragColor.g = tex.r * linearColour.g * linearColour.a;
    gl_FragColor.b = tex.r * linearColour.b * linearColour.a;
    gl_FragColor.a = tex.r * linearColour.a;
  }
}
