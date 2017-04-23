#include "seni_bind_shapes.h"
#include "seni_shapes.h"
#include "seni_bind.h"
#include "seni_buffer.h"

#include <stdio.h>

seni_var *eval_fn_line(seni_env *env, seni_node *expr)
{
  // get the values/defaults
  //
  seni_node *parameters = safe_next(expr);

  f32 width = get_named_f32(env, parameters, g_arg_width, 5.0f);

  f32 from_x = 50.0f; f32 from_y = 50.0f;
  get_named_vec2(env, parameters, g_arg_from, &from_x, &from_y);

  f32 to_x = 500.0f; f32 to_y = 500.0f;
  get_named_vec2(env, parameters, g_arg_to, &to_x, &to_y);

  f32 r = 0.0f; f32 g = 0.0f; f32 b = 0.0f; f32 a = 1.0f;
  get_named_vec4(env, parameters, g_arg_colour, &r, &g, &b, &a);
  rgba col;
  col.r = r; col.g = g; col.b = b; col.a = a;


  seni_buffer *buffer = env->buffer;
  printf("before: buffer size %d %d\n", buffer->max_vertices, buffer->num_vertices);

  render_line(buffer, from_x, from_y, to_x, to_y, width, col);
  
  printf("after: buffer size %d %d\n", buffer->max_vertices, buffer->num_vertices);

  return NULL;
}

seni_var *eval_fn_rect(seni_env *env, seni_node *expr)
{
  // get the values/defaults
  //
  seni_node *parameters = safe_next(expr);

  f32 width = get_named_f32(env, parameters, g_arg_width, 500.0f);
  f32 height = get_named_f32(env, parameters, g_arg_height, 500.0f);

  f32 x = 500.0f; f32 y = 500.0f;
  get_named_vec2(env, parameters, g_arg_position, &x, &y);

  f32 r = 0.0f; f32 g = 0.0f; f32 b = 0.0f; f32 a = 1.0f;
  get_named_vec4(env, parameters, g_arg_colour, &r, &g, &b, &a);
  rgba col;
  col.r = r; col.g = g; col.b = b; col.a = a;

  seni_buffer *buffer = env->buffer;
  printf("before: buffer size %d %d\n", buffer->max_vertices, buffer->num_vertices);

  render_rect(buffer, x, y, width, height, col);
  
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
  rgba col;
  col.r = r; col.g = g; col.b = b; col.a = a;

  seni_buffer *buffer = env->buffer;
  printf("before: buffer size %d %d\n", buffer->max_vertices, buffer->num_vertices);

  render_circle(buffer, x, y, width, height, col, tessellation);

  printf("after: buffer size %d %d\n", buffer->max_vertices, buffer->num_vertices);

  return NULL;
}

void bind_shape_declarations(word_lut *wlut)
{
  declare_keyword(wlut, "line", &eval_fn_line);
  declare_keyword(wlut, "rect", &eval_fn_rect);
  declare_keyword(wlut, "circle", &eval_fn_circle);
}
