// pre-multiply the alpha in the shader
// see http://www.realtimerendering.com/blog/gpus-prefer-premultiplication/
// this needs to happen in linear colour space

precision highp float;

varying vec4 frag_col;
varying vec2 frag_uv;
varying vec2 world_pos;

uniform sampler2D brush;
uniform sampler2D mask;

uniform float canvas_dim;
uniform bool mask_invert;
uniform bool output_linear_colour_space;

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
  vec4 brush_col = texture2D(brush, frag_uv);

  // note: you _never_ want output_linear_colour_space to be set to true
  // it's only here because some of the older sketchs didn't correctly
  // convert from linear colour space to sRGB colour space during rendering
  // and this shader needs to reproduce them as intended at time of creation
  //
  if (output_linear_colour_space) {
    // purely for legacy, no new development will occur in this branch of the if conditional
    //
    gl_FragColor.r = brush_col.r * frag_col.r * frag_col.a;
    gl_FragColor.g = brush_col.r * frag_col.g * frag_col.a;
    gl_FragColor.b = brush_col.r * frag_col.b * frag_col.a;
    gl_FragColor.a = brush_col.r * frag_col.a;
  } else {
    // all modern scripts should assume correct colour space conversions
    vec4 linear_col = vec4(srgb_to_linear(frag_col.rgb), 1.0);
    vec4 mask_col = texture2D(mask, world_pos / canvas_dim);

    float mask_val = mask_invert ? 1.0 - mask_col.r : mask_col.r;

    gl_FragColor.r = brush_col.r * linear_col.r * frag_col.a * mask_val;
    gl_FragColor.g = brush_col.r * linear_col.g * frag_col.a * mask_val;
    gl_FragColor.b = brush_col.r * linear_col.b * frag_col.a * mask_val;
    gl_FragColor.a = brush_col.r * linear_col.a * frag_col.a * mask_val;
  }
}
