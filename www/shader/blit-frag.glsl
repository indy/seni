precision highp float;

varying vec2 frag_uv;

uniform sampler2D rendered_image;
uniform bool output_linear_colour_space;

// https:en.wikipedia.org/wiki/SRGB
vec3 linear_to_srgb(vec3 linear) {
  float a = 0.055;
  float b = 0.0031308;
  vec3 srgb_lo = 12.92 * linear;
  vec3 srgb_hi = (1.0 + a) * pow(linear, vec3(1.0/2.4)) - vec3(a);

  return vec3(linear.r > b ? srgb_hi.r : srgb_lo.r,
              linear.g > b ? srgb_hi.g : srgb_lo.g,
              linear.b > b ? srgb_hi.b : srgb_lo.b);
}

// https:twitter.com/jimhejl/status/633777619998130176
// vec3 ToneMapFilmic_Hejl2015(vec3 hdr, float whitePt) {
//   vec4 vh = vec4(hdr, whitePt);
//   vec4 va = 1.425 * vh + 0.05;
//   vec4 vf = (vh * va + 0.004) / (vh * (va + 0.55) + 0.0491) - 0.0821;
//   return vf.rgb / vf.www;
// }



mat4 brightness_mat4(float brightness) {
  return mat4(1, 0, 0, 0,
              0, 1, 0, 0,
              0, 0, 1, 0,
              brightness, brightness, brightness, 1);
}

mat4 contrast_mat4(float contrast) {
	float t = (1.0 - contrast) / 2.0;

  return mat4(contrast, 0, 0, 0,
              0, contrast, 0, 0,
              0, 0, contrast, 0,
              t, t, t, 1);
}

mat4 saturation_mat4(float saturation) {
  vec3 luminance = vec3(0.3086, 0.6094, 0.0820);

  float one_minus_sat = 1.0 - saturation;

  vec3 red = vec3(luminance.x * one_minus_sat);
  red += vec3(saturation, 0, 0);

  vec3 green = vec3(luminance.y * one_minus_sat);
  green += vec3(0, saturation, 0);

  vec3 blue = vec3(luminance.z * one_minus_sat);
  blue += vec3(0, 0, saturation);

  return mat4(red,     0,
              green,   0,
              blue,    0,
              0, 0, 0, 1);
}


void main()
{
  /*
    defaults:
    const float brightness = 0.00;
    const float contrast = 1.0;
    const float saturation = 1.0;
  */

  const float brightness = 0.00;
  const float contrast = 1.7;
  const float saturation = 0.6;

  vec4 rendered_image_col = texture2D(rendered_image, frag_uv);

  // note: you _never_ want output_linear_colour_space to be set to true
  // it's only here because some of the older sketchs didn't correctly
  // convert from linear colour space to sRGB colour space during rendering
  // and this shader needs to reproduce them as intended at time of creation
  //
  if (output_linear_colour_space) {
    gl_FragColor = rendered_image_col;
  } else {
    vec4 balanced_col = brightness_mat4(brightness) *
      contrast_mat4(contrast) *
      saturation_mat4(saturation) *
      rendered_image_col;

    gl_FragColor = vec4(linear_to_srgb(balanced_col.rgb), 1.0);
  }
}
