#include "seni_bind_shapes.h"
#include "seni_bind.h"
#include "seni_buffer.h"

#include <stdio.h>
#include <math.h>

// Renderer :: renderRect

bool can_vertices_fit(seni_buffer *buffer, i32 num)
{
  return buffer->num_vertices < (buffer->max_vertices - (num + 2));
}

void flush_triangles(seni_buffer *buffer)
{
  printf("TODO: implement");
}

bool is_buffer_empty(seni_buffer *buffer)
{
  return buffer->num_vertices == 0;
}

void matrix_stack_transform_2d_vector(f32 *out, f32 x, f32 y)
{
  // TODO: implement
  out[0] = x;
  out[1] = y;
}

void add_vertex(seni_buffer *buffer, f32 x, f32 y, f32 r, f32 g, f32 b, f32 a, f32 u, f32 v)
{
  i32 vertex_item_size = 2;
  i32 v_index = buffer->num_vertices * vertex_item_size;
  i32 colour_item_size = 4;
  i32 c_index = buffer->num_vertices * colour_item_size;
  i32 texture_item_size = 2;
  i32 t_index = buffer->num_vertices * texture_item_size;

  buffer->vbuf[v_index + 0] = x;
  buffer->vbuf[v_index + 1] = y;

  buffer->cbuf[c_index + 0] = r;
  buffer->cbuf[c_index + 1] = g;
  buffer->cbuf[c_index + 2] = b;
  buffer->cbuf[c_index + 3] = a;

  buffer->tbuf[t_index + 0] = u;
  buffer->tbuf[t_index + 1] = v;

  buffer->num_vertices++;
}

void form_degenerate_triangle(seni_buffer *buffer, f32 x, f32 y)
{
  i32 vertex_item_size = 2;
  // get the index of the last vertex that was added
  i32 index = (buffer->num_vertices * vertex_item_size) - vertex_item_size;

  // just copy the previous entries
  // note: colour doesn't matter since these triangles won't be rendered
  f32 *last_v = &(buffer->vbuf[index]);
  add_vertex(buffer, last_v[0], last_v[1], 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f);

  // add the new vertex to complete the degenerate triangle
  add_vertex(buffer, x, y, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f);
  
  // Note: still need to call addVertex on the first
  // vertex when we 'really' render the strip
}

void prepare_to_add_triangle_strip(seni_buffer *buffer, i32 num_vertices, f32 x, f32 y)
{
  if (can_vertices_fit(buffer, num_vertices) == false) {
    flush_triangles(buffer);
  }

  if (is_buffer_empty(buffer) == false) {
    f32 out[2];
    matrix_stack_transform_2d_vector(out, x, y);
    form_degenerate_triangle(buffer, out[0], out[1]);
  }
}

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

  f32 half_width = width / 2.0f;
  f32 half_height = height / 2.0f;

  f32 min_uv = 1.0f / 1024.0f;
  f32 max_uv = 2.0f / 1024.0f;

  seni_buffer *buffer = env->buffer;
  printf("before: buffer size %d %d\n", buffer->max_vertices, buffer->num_vertices);

  f32 r = 0.0f; f32 g = 0.0f; f32 b = 0.0f; f32 a = 1.0f;
  get_named_vec4(env, parameters, g_arg_colour, &r, &g, &b, &a);

  prepare_to_add_triangle_strip(buffer, 4, x - half_width, y - half_height);
  add_vertex(buffer, x - half_width, y - half_height, r, g, b, a, max_uv, min_uv);
  add_vertex(buffer, x + half_width, y - half_height, r, g, b, a, max_uv, max_uv);
  add_vertex(buffer, x - half_width, y + half_height, r, g, b, a, min_uv, min_uv);
  add_vertex(buffer, x + half_width, y + half_height, r, g, b, a, min_uv, max_uv);
  
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

  f32 uv = 1.0f / 1024.0f;
  
  prepare_to_add_triangle_strip(buffer, (tessellation * 2) + 2, x, y);

  f32 tau = M_PI * 2.0f;
  f32 unit_angle = tau / tessellation;
  f32 angle, vx, vy;

  for (int i = 0; i < tessellation; i++) {
    angle = unit_angle * i;
    vx = ((f32)(sin(angle)) * width) + x;
    vy = ((f32)(cos(angle)) * height) + y;

    add_vertex(buffer, x, y, r, g, b, a, uv, uv);
    add_vertex(buffer, vx, vy, r, g, b, a, uv, uv);
  }

  angle = 0.0f;
  vx = ((f32)(sin(angle)) * width) + x;
  vy = ((f32)(cos(angle)) * height) + y;

  add_vertex(buffer, x, y, r, g, b, a, uv, uv);
  add_vertex(buffer, vx, vy, r, g, b, a, uv, uv);

  printf("after: buffer size %d %d\n", buffer->max_vertices, buffer->num_vertices);

  return NULL;
}

void bind_shape_declarations(word_lut *wlut)
{
  declare_keyword(wlut, "rect", &eval_fn_rect);
  declare_keyword(wlut, "circle", &eval_fn_circle);
}
