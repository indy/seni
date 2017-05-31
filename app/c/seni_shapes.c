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
  buffer = NULL;
  printf("TODO: implement");
}

bool is_buffer_empty(seni_buffer *buffer)
{
  return buffer->num_vertices == 0;
}

void add_vertex(seni_buffer *buffer, seni_matrix *matrix, f32 x, f32 y, rgba colour, v2 uv)
{
  i32 vertex_item_size = 2;
  i32 v_index = buffer->num_vertices * vertex_item_size;
  i32 colour_item_size = 4;
  i32 c_index = buffer->num_vertices * colour_item_size;
  i32 texture_item_size = 2;
  i32 t_index = buffer->num_vertices * texture_item_size;

  f32 out[2];
  matrix_transform_vec2(out, matrix, x, y);
  buffer->vbuf[v_index + 0] = out[0];
  buffer->vbuf[v_index + 1] = out[1];

  buffer->cbuf[c_index + 0] = colour.r;
  buffer->cbuf[c_index + 1] = colour.g;
  buffer->cbuf[c_index + 2] = colour.b;
  buffer->cbuf[c_index + 3] = colour.a;

  buffer->tbuf[t_index + 0] = uv.x; // u
  buffer->tbuf[t_index + 1] = uv.y; // v

  buffer->num_vertices++;
}

void form_degenerate_triangle(seni_buffer *buffer, seni_matrix *matrix, f32 x, f32 y)
{
  i32 vertex_item_size = 2;
  // get the index of the last vertex that was added
  i32 index = (buffer->num_vertices * vertex_item_size) - vertex_item_size;

  // just copy the previous entries
  // note: colour doesn't matter since these triangles won't be rendered
  f32 *last_v = &(buffer->vbuf[index]);

  // todo: don't create an identity matrix for each call
  seni_matrix identity;
  matrix_identity(&identity);
  
  rgba colour;
  colour.r = 0.0f; colour.g = 0.0f; colour.b = 0.0f; colour.a = 0.0f;
  v2 uv;
  uv.x = 0.0f; uv.y = 0.0f; // u v
  add_vertex(buffer, &identity, last_v[0], last_v[1], colour, uv);

  // add the new vertex to complete the degenerate triangle
  add_vertex(buffer, matrix, x, y, colour, uv);
  
  // Note: still need to call addVertex on the first
  // vertex when we 'really' render the strip
}

void prepare_to_add_triangle_strip(seni_buffer *buffer, seni_matrix *matrix, i32 num_vertices, f32 x, f32 y)
{
  if (can_vertices_fit(buffer, num_vertices) == false) {
    flush_triangles(buffer);
  }

  if (is_buffer_empty(buffer) == false) {
    form_degenerate_triangle(buffer, matrix, x, y);
  }
}

void render_line(seni_buffer *buffer,
                 seni_matrix *matrix,
                 f32 from_x, f32 from_y, f32 to_x, f32 to_y,
                 f32 width,
                 rgba colour)
{
  seni_uv_mapping *uv = get_uv_mapping(BRUSH_FLAT, 0);

  f32 hw = (width * uv->width_scale) / 2.0f;

  v2 n = normal(from_x, from_y, to_x, to_y);
  v2 n2 = opposite_normal(n);

  prepare_to_add_triangle_strip(buffer, matrix, 4, from_x + (hw * n.x), from_y + (hw * n.y));
  
  add_vertex(buffer, matrix, from_x + (hw * n.x),  from_y + (hw * n.y),  colour, uv->map[0]);
  add_vertex(buffer, matrix, from_x + (hw * n2.x), from_y + (hw * n2.y), colour, uv->map[1]);
  add_vertex(buffer, matrix, to_x + (hw * n.x),    to_y + (hw * n.y),    colour, uv->map[2]);
  add_vertex(buffer, matrix, to_x + (hw * n2.x),   to_y + (hw * n2.y),   colour, uv->map[3]);
}


void render_rect(seni_buffer *buffer,
                 seni_matrix *matrix,
                 f32 x, f32 y,
                 f32 width, f32 height,
                 rgba colour)
{
  seni_uv_mapping *uv = get_uv_mapping(BRUSH_FLAT, 0);

  f32 half_width = width / 2.0f;
  f32 half_height = height / 2.0f;

  prepare_to_add_triangle_strip(buffer, matrix, 4, x - half_width, y - half_height);
  add_vertex(buffer, matrix, x - half_width, y - half_height, colour, uv->map[0]);
  add_vertex(buffer, matrix, x + half_width, y - half_height, colour, uv->map[1]);
  add_vertex(buffer, matrix, x - half_width, y + half_height, colour, uv->map[2]);
  add_vertex(buffer, matrix, x + half_width, y + half_height, colour, uv->map[3]);
}

void render_circle(seni_buffer *buffer,
                   seni_matrix *matrix,
                   f32 x, f32 y,
                   f32 width, f32 height,
                   rgba colour,
                   i32 tessellation)
{
  v2 uv;
  make_uv(&uv, 1.0f, 1.0f);

  prepare_to_add_triangle_strip(buffer, matrix, (tessellation * 2) + 2, x, y);

  f32 unit_angle = TAU / tessellation;
  f32 angle, vx, vy;

  for (int i = 0; i < tessellation; i++) {
    angle = unit_angle * i;
    vx = ((f32)(sin(angle)) * width) + x;
    vy = ((f32)(cos(angle)) * height) + y;

    add_vertex(buffer, matrix, x, y, colour, uv);
    add_vertex(buffer, matrix, vx, vy, colour, uv);
  }

  angle = 0.0f;
  vx = ((f32)(sin(angle)) * width) + x;
  vy = ((f32)(cos(angle)) * height) + y;

  add_vertex(buffer, matrix, x, y, colour, uv);
  add_vertex(buffer, matrix, vx, vy, colour, uv);
}

void render_bezier(seni_buffer *buffer,
                   seni_matrix *matrix,
                   f32 *coords,
                   f32 line_width_start, f32 line_width_end, i32 line_width_mapping,
                   f32 t_start, f32 t_end,
                   rgba colour,
                   i32 tessellation)
{
  seni_uv_mapping *uv = get_uv_mapping(BRUSH_FLAT, 0);

  line_width_start *= uv->width_scale;
  line_width_end *= uv->width_scale;
  f32 half_width_start = line_width_start / 2.0f;
  f32 half_width_end = line_width_end / 2.0f;
  // create a remapping function here

  f32 x0 = coords[0], x1 = coords[2], x2 = coords[4], x3 = coords[6];
  f32 y0 = coords[1], y1 = coords[3], y2 = coords[5], y3 = coords[7];
  f32 xs, ys, xs_next, ys_next;
  v2 n1, n2, v1, v2;
 
  i32 i;
  f32 unit = (t_end - t_start) / (tessellation - 1.0f);
  f32 t_val, t_val_next;

  f32 tex_t = 1.0f / tessellation;
  f32 uv_t;
  // uvA == uv->map[0] ...

  for (i = 0; i < tessellation - 1; i++) {
    t_val = t_start + ((f32)i * unit);
    t_val_next = t_start + ((f32)(i + 1) * unit);

    xs = bezier_point(x0, x1, x2, x3, t_val);
    ys = bezier_point(y0, y1, y2, y3, t_val);
    xs_next = bezier_point(x0, x1, x2, x3, t_val_next);
    ys_next = bezier_point(y0, y1, y2, y3, t_val_next);

    // addVerticesAsStrip
    n1 = normal(xs, ys, xs_next, ys_next);
    n2 = opposite_normal(n1);

    v1.x = (n1.x * half_width_end) + xs;
    v1.y = (n1.y * half_width_end) + ys;
    v2.x = (n2.x * half_width_end) + xs;
    v2.y = (n2.y * half_width_end) + ys;

    if (i == 0) {
      prepare_to_add_triangle_strip(buffer, matrix, tessellation * 2, v1.x, v1.y);
    }

    uv_t = tex_t * (f32)i;
    
    // todo: interpolate the uv coordinates
    add_vertex(buffer, matrix, v1.x, v1.y, colour, uv->map[0]);
    add_vertex(buffer, matrix, v2.x, v2.y, colour, uv->map[1]);
  }

  // final 2 vertices for the end point
  i = tessellation - 2;

  t_val = t_start + ((f32)i * unit);
  t_val_next = t_start + ((f32)(i + 1) * unit);

  xs = bezier_point(x0, x1, x2, x3, t_val);
  ys = bezier_point(y0, y1, y2, y3, t_val);
  xs_next = bezier_point(x0, x1, x2, x3, t_val_next);
  ys_next = bezier_point(y0, y1, y2, y3, t_val_next);

  n1 = normal(xs, ys, xs_next, ys_next);
  n2 = opposite_normal(n1);

  v1.x = (n1.x * half_width_end) + xs_next;
  v1.y = (n1.y * half_width_end) + ys_next;
  v2.x = (n2.x * half_width_end) + xs_next;
  v2.y = (n2.y * half_width_end) + ys_next;

  add_vertex(buffer, matrix, v1.x, v1.y, colour, uv->map[3]);
  add_vertex(buffer, matrix, v2.x, v2.y, colour, uv->map[2]);
}

