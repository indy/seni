precision highp float;

varying vec2 vTextureCoord;

uniform sampler2D texture;
uniform bool uOutputLinearColourSpace;

// https:en.wikipedia.org/wiki/SRGB
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

// https:twitter.com/jimhejl/status/633777619998130176
vec3 ToneMapFilmic_Hejl2015(vec3 hdr, float whitePt) {
  vec4 vh = vec4(hdr, whitePt);
  vec4 va = 1.425 * vh + 0.05;
  vec4 vf = (vh * va + 0.004) / (vh * (va + 0.55) + 0.0491) - 0.0821;
  return vf.rgb / vf.www;
}

void main()
{
  vec4 col = texture2D( texture, vTextureCoord );

  // note: you _never_ want uOutputLinearColourSpace to be set to true
  // it's only here because some of the older sketchs didn't correctly
  // convert from linear colour space to sRGB colour space during rendering
  // and this shader needs to reproduce them as intended at time of creation
  //
  if (uOutputLinearColourSpace) {
    gl_FragColor = col;
  } else {
    gl_FragColor = vec4(linear_to_srgb(col.rgb), 1.0);
  }
}
