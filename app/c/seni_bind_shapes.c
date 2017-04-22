#include "seni_bind_shapes.h"
#include "seni_shapes.h"
#include "seni_bind.h"
#include "seni_buffer.h"

#include <stdio.h>

/*
  renders a rectangle, centered in position with the given width, height

  ['position', 'a position vector'],
  ['width', 'width'],
  ['height', 'height'],
  ['colour', 'Colour.defaultColour']

  (rect position: [100 100])
*/
seni_var *eval_fn_rect(seni_env *env, seni_node *expr)
{
  // get the values/defaults
  //
  seni_node *parameters = safe_next(expr);

  f32 width = get_named_f32(env, parameters, g_arg_width, 500.0f);
  f32 height = get_named_f32(env, parameters, g_arg_height, 500.0f);

  f32 x = 500.0f; f32 y = 500.0f;
  get_named_vec2(env, parameters, g_arg_position, &x, &y);

  seni_buffer *buffer = env->buffer;
  printf("before: buffer size %d %d\n", buffer->max_vertices, buffer->num_vertices);

  f32 r = 0.0f; f32 g = 0.0f; f32 b = 0.0f; f32 a = 1.0f;
  get_named_vec4(env, parameters, g_arg_colour, &r, &g, &b, &a);

  render_rect(buffer, x, y, width, height, r, g, b, a);
  
  printf("after: buffer size %d %d\n", buffer->max_vertices, buffer->num_vertices);

  return NULL;
}

seni_var *eval_fn_circle(seni_env *env, seni_node *expr)
{
  // get the values/defaults
  //
  seni_node *parameters = safe_next(expr);

  f32 width = get_named_f32(env, parameters, g_arg_width, 500.0f);
  f32 height = get_named_f32(env, parameters, g_arg_height, 500.0f);
  f32 radius = get_named_f32(env, parameters, g_arg_radius, 100.0f);
  i32 tessellation = get_named_i32(env, parameters, g_arg_tessellation, 5);

  f32 x = 500.0f; f32 y = 500.0f;
  get_named_vec2(env, parameters, g_arg_position, &x, &y);

  if (has_named_node(parameters, g_arg_radius)) {
    // use the radius for both width and height if it's given
    width = radius;
    height = radius;
  }

  f32 r = 0.0f; f32 g = 0.0f; f32 b = 0.0f; f32 a = 1.0f;
  get_named_vec4(env, parameters, g_arg_colour, &r, &g, &b, &a);

  seni_buffer *buffer = env->buffer;
  printf("before: buffer size %d %d\n", buffer->max_vertices, buffer->num_vertices);

  render_circle(buffer, x, y, width, height, r, g, b, a, tessellation);

  printf("after: buffer size %d %d\n", buffer->max_vertices, buffer->num_vertices);

  return NULL;
}

void bind_shape_declarations(word_lut *wlut)
{
  declare_keyword(wlut, "rect", &eval_fn_rect);
  declare_keyword(wlut, "circle", &eval_fn_circle);
}
