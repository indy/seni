#include "seni_shapes.h"
#include "seni_uv_mapper.h"
#include "seni_mathutil.h"

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

void add_vertex(seni_buffer *buffer, f32 x, f32 y, rgba col, v2 t)
{
  i32 vertex_item_size = 2;
  i32 v_index = buffer->num_vertices * vertex_item_size;
  i32 colour_item_size = 4;
  i32 c_index = buffer->num_vertices * colour_item_size;
  i32 texture_item_size = 2;
  i32 t_index = buffer->num_vertices * texture_item_size;

  buffer->vbuf[v_index + 0] = x;
  buffer->vbuf[v_index + 1] = y;

  buffer->cbuf[c_index + 0] = col.r;
  buffer->cbuf[c_index + 1] = col.g;
  buffer->cbuf[c_index + 2] = col.b;
  buffer->cbuf[c_index + 3] = col.a;

  buffer->tbuf[t_index + 0] = t.u;
  buffer->tbuf[t_index + 1] = t.v;

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

  rgba col;
  col.r = 0.0f; col.g = 0.0f; col.b = 0.0f; col.a = 0.0f;
  v2 t;
  t.u = 0.0f; t.v = 0.0f;
  add_vertex(buffer, last_v[0], last_v[1], col, t);

  // add the new vertex to complete the degenerate triangle
  add_vertex(buffer, x, y, col, t);
  
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

void render_line(seni_buffer *buffer,
                 f32 from_x, f32 from_y, f32 to_x, f32 to_y,
                 f32 width,
                 rgba col)
{
  seni_uv_mapping *m = get_uv_mapping(BRUSH_FLAT, 0);

  f32 hw = (width * m->width_scale) / 2.0f;

  v2 n = normal(from_x, from_y, to_x, to_y);
  v2 n2 = opposite_normal(n);

  printf("%.2f, %.2f, %.2f, %.2f\n", n.x, n.y, n2.x, n2.y);

  prepare_to_add_triangle_strip(buffer, 4, from_x + (hw * n.x), from_y + (hw * n.y));
  
  add_vertex(buffer, from_x + (hw * n.x),  from_y + (hw * n.y),  col, m->map[0]);
  add_vertex(buffer, from_x + (hw * n2.x), from_y + (hw * n2.y), col, m->map[1]);
  add_vertex(buffer, to_x + (hw * n.x),    to_y + (hw * n.y),    col, m->map[2]);
  add_vertex(buffer, to_x + (hw * n2.x),   to_y + (hw * n2.y),   col, m->map[3]);
}


void render_rect(seni_buffer *buffer,
                 f32 x, f32 y,
                 f32 width, f32 height,
                 rgba col)
{
  seni_uv_mapping *m = get_uv_mapping(BRUSH_FLAT, 0);

  f32 half_width = width / 2.0f;
  f32 half_height = height / 2.0f;

  prepare_to_add_triangle_strip(buffer, 4, x - half_width, y - half_height);
  add_vertex(buffer, x - half_width, y - half_height, col, m->map[0]);
  add_vertex(buffer, x + half_width, y - half_height, col, m->map[1]);
  add_vertex(buffer, x - half_width, y + half_height, col, m->map[2]);
  add_vertex(buffer, x + half_width, y + half_height, col, m->map[3]);
}

void render_circle(seni_buffer *buffer,
                   f32 x, f32 y,
                   f32 width, f32 height,
                   rgba col,
                   i32 tessellation)
{
  v2 uv;
  make_uv(&uv, 1.0f, 1.0f);

  prepare_to_add_triangle_strip(buffer, (tessellation * 2) + 2, x, y);

  f32 tau = M_PI * 2.0f;
  f32 unit_angle = tau / tessellation;
  f32 angle, vx, vy;

  for (int i = 0; i < tessellation; i++) {
    angle = unit_angle * i;
    vx = ((f32)(sin(angle)) * width) + x;
    vy = ((f32)(cos(angle)) * height) + y;

    add_vertex(buffer, x, y, col, uv);
    add_vertex(buffer, vx, vy, col, uv);
  }

  angle = 0.0f;
  vx = ((f32)(sin(angle)) * width) + x;
  vy = ((f32)(cos(angle)) * height) + y;

  add_vertex(buffer, x, y, col, uv);
  add_vertex(buffer, vx, vy, col, uv);
}
