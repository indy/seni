#include "seni_shapes.h"
#include "seni_types.h"
#include "seni_uv_mapper.h"
#include "seni_mathutil.h"
#include "seni_render_packet.h"

#include <stdio.h>
#include <math.h>

// extern global keyword variables - used to reference bezier line_width_mapping
#define KEYWORD(val,_,name) extern i32 g_keyword_iname_##name;
#include "seni_keywords.h"
#undef KEYWORD

// Renderer :: renderRect

bool can_vertices_fit(seni_render_packet *render_packet, i32 num, i32 max_vertices)
{
  return render_packet->num_vertices < (max_vertices - (num + 2));
}

bool is_render_packet_empty(seni_render_packet *render_packet)
{
  return render_packet->num_vertices == 0;
}

void add_vertex(seni_render_packet *render_packet, seni_matrix *matrix, f32 x, f32 y, seni_colour *rgb, v2 uv)
{
  i32 vertex_item_size = 2;
  i32 v_index = render_packet->num_vertices * vertex_item_size;
  i32 colour_item_size = 4;
  i32 c_index = render_packet->num_vertices * colour_item_size;
  i32 texture_item_size = 2;
  i32 t_index = render_packet->num_vertices * texture_item_size;

  f32 out[2];
  matrix_transform_vec2(out, matrix, x, y);
  render_packet->vbuf[v_index + 0] = out[0];
  render_packet->vbuf[v_index + 1] = out[1];

  render_packet->cbuf[c_index + 0] = rgb->element[0];
  render_packet->cbuf[c_index + 1] = rgb->element[1];
  render_packet->cbuf[c_index + 2] = rgb->element[2];
  render_packet->cbuf[c_index + 3] = rgb->element[3];

  render_packet->tbuf[t_index + 0] = uv.x; // u
  render_packet->tbuf[t_index + 1] = uv.y; // v

  render_packet->num_vertices++;
}

void form_degenerate_triangle(seni_render_packet *render_packet, seni_matrix *matrix, f32 x, f32 y)
{
  i32 vertex_item_size = 2;
  // get the index of the last vertex that was added
  i32 index = (render_packet->num_vertices * vertex_item_size) - vertex_item_size;

  // just copy the previous entries
  // note: colour doesn't matter since these triangles won't be rendered
  f32 *last_v = &(render_packet->vbuf[index]);

  // todo: don't create an identity matrix for each call
  seni_matrix identity;
  matrix_identity(&identity);

  seni_colour colour;
  colour_set(&colour, RGB, 0.0f, 0.0f, 0.0f, 0.0f);
  v2 uv;
  uv.x = 0.0f; uv.y = 0.0f; // u v
  add_vertex(render_packet, &identity, last_v[0], last_v[1], &colour, uv);

  // add the new vertex to complete the degenerate triangle
  add_vertex(render_packet, matrix, x, y, &colour, uv);
  
  // Note: still need to call addVertex on the first
  // vertex when we 'really' render the strip
}

void prepare_to_add_triangle_strip(seni_render_data *render_data, seni_matrix *matrix, i32 num_vertices, f32 x, f32 y)
{
  if (can_vertices_fit(render_data->current_render_packet, num_vertices, render_data->max_vertices) == false) {
    add_render_packet(render_data);
  }

  seni_render_packet *render_packet = render_data->current_render_packet;

  if (is_render_packet_empty(render_packet) == false) {
    form_degenerate_triangle(render_packet, matrix, x, y);
  }
}

void render_line(seni_render_data *render_data,
                 seni_matrix *matrix,
                 f32 from_x, f32 from_y, f32 to_x, f32 to_y,
                 f32 width,
                 seni_colour *colour)
{
  seni_uv_mapping *uv = get_uv_mapping(BRUSH_FLAT, 0, true);

  f32 hw = (width * uv->width_scale) / 2.0f;

  v2 n = normal(from_x, from_y, to_x, to_y);
  v2 n2 = opposite_normal(n);

  seni_colour *rgb, rgb_colour;
  if (colour->format == RGB) {
    rgb = colour;
  } else {
    colour_clone_as(&rgb_colour, colour, RGB);
    rgb = &rgb_colour;
  }

  prepare_to_add_triangle_strip(render_data, matrix, 4, from_x + (hw * n.x), from_y + (hw * n.y));
  add_vertex(render_data->current_render_packet, matrix, from_x + (hw * n.x),  from_y + (hw * n.y),  rgb, uv->map[0]);
  add_vertex(render_data->current_render_packet, matrix, from_x + (hw * n2.x), from_y + (hw * n2.y), rgb, uv->map[1]);
  add_vertex(render_data->current_render_packet, matrix, to_x + (hw * n.x),    to_y + (hw * n.y),    rgb, uv->map[2]);
  add_vertex(render_data->current_render_packet, matrix, to_x + (hw * n2.x),   to_y + (hw * n2.y),   rgb, uv->map[3]);
}


void render_rect(seni_render_data *render_data,
                 seni_matrix *matrix,
                 f32 x, f32 y,
                 f32 width, f32 height,
                 seni_colour *colour)
{
  seni_uv_mapping *uv = get_uv_mapping(BRUSH_FLAT, 0, true);

  f32 half_width = width / 2.0f;
  f32 half_height = height / 2.0f;

  seni_colour *rgb, rgb_colour;
  if (colour->format == RGB) {
    rgb = colour;
  } else {
    colour_clone_as(&rgb_colour, colour, RGB);
    rgb = &rgb_colour;
  }

  prepare_to_add_triangle_strip(render_data, matrix, 4, x - half_width, y - half_height);
  add_vertex(render_data->current_render_packet, matrix, x - half_width, y - half_height, rgb, uv->map[0]);
  add_vertex(render_data->current_render_packet, matrix, x + half_width, y - half_height, rgb, uv->map[1]);
  add_vertex(render_data->current_render_packet, matrix, x - half_width, y + half_height, rgb, uv->map[2]);
  add_vertex(render_data->current_render_packet, matrix, x + half_width, y + half_height, rgb, uv->map[3]);
}

void render_circle(seni_render_data *render_data,
                   seni_matrix *matrix,
                   f32 x, f32 y,
                   f32 width, f32 height,
                   seni_colour *colour,
                   i32 tessellation)
{
  v2 uv;
  make_uv(&uv, 1.0f, 1.0f);

  prepare_to_add_triangle_strip(render_data, matrix, (tessellation * 2) + 2, x, y);

  f32 unit_angle = TAU / tessellation;
  f32 angle, vx, vy;

  seni_colour *rgb, rgb_colour;
  if (colour->format == RGB) {
    rgb = colour;
  } else {
    colour_clone_as(&rgb_colour, colour, RGB);
    rgb = &rgb_colour;
  }

  for (int i = 0; i < tessellation; i++) {
    angle = unit_angle * i;
    vx = ((f32)(sin(angle)) * width) + x;
    vy = ((f32)(cos(angle)) * height) + y;

    add_vertex(render_data->current_render_packet, matrix, x, y, rgb, uv);
    add_vertex(render_data->current_render_packet, matrix, vx, vy, rgb, uv);
  }

  angle = 0.0f;
  vx = ((f32)(sin(angle)) * width) + x;
  vy = ((f32)(cos(angle)) * height) + y;

  add_vertex(render_data->current_render_packet, matrix, x, y, rgb, uv);
  add_vertex(render_data->current_render_packet, matrix, vx, vy, rgb, uv);
}

void render_bezier(seni_render_data *render_data,
                   seni_matrix *matrix,
                   f32 *coords,
                   f32 line_width_start, f32 line_width_end, i32 line_width_mapping,
                   f32 t_start, f32 t_end,
                   seni_colour *colour,
                   i32 tessellation,
                   i32 brush, i32 brush_subtype)
{
  // get the uv co-ordinates for the specified brush
  //
  seni_brush_type brush_type = (seni_brush_type)(brush - g_keyword_iname_brush_flat);
  printf("brush type %d\n", brush_type);
  seni_uv_mapping *uv = get_uv_mapping(brush_type, brush_subtype, true);
  v2 uv_a = uv->map[0];
  v2 uv_b = uv->map[1];
  v2 uv_c = uv->map[2];
  v2 uv_d = uv->map[3];

  // modify the width so that the brush textures provide good coverage
  //
  line_width_start *= uv->width_scale;
  line_width_end *= uv->width_scale;

  // variables for interpolating the curve's width
  //
  f32 half_width_start = line_width_start / 2.0f;
  f32 half_width_end = line_width_end / 2.0f;
  f32 from_m = mc_m(t_start, 0.0f, t_end, 1.0f);
  f32 from_c = mc_c(t_start, 0.0f, from_m);
  f32 to_m = mc_m(0.0f, half_width_start, 1.0f, half_width_end);
  f32 to_c = mc_c(0.0f, half_width_start, to_m);
  f32 from_interp, to_interp, half_width;

  f32 x0 = coords[0], x1 = coords[2], x2 = coords[4], x3 = coords[6];
  f32 y0 = coords[1], y1 = coords[3], y2 = coords[5], y3 = coords[7];
  f32 xs, ys, xs_next, ys_next;
  v2 n1, n2, v_1, v_2;
 
  i32 i;
  f32 unit = (t_end - t_start) / (tessellation - 1.0f);
  f32 t_val, t_val_next;

  f32 tex_t = 1.0f / tessellation;
  f32 uv_t;
  v2 t_uv;

  // vertex colours have to be in rgb space
  seni_colour *rgb, rgb_colour;
  if (colour->format == RGB) {
    rgb = colour;
  } else {
    colour_clone_as(&rgb_colour, colour, RGB);
    rgb = &rgb_colour;
  }
  
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
    
    from_interp = (from_m * t_val) + from_c;
    to_interp = from_interp;    // default behaviour as though 'linear' was chosen
    if (line_width_mapping == g_keyword_iname_quick) {
      to_interp = map_quick_ease(from_interp);
    } else if (line_width_mapping == g_keyword_iname_slow_in) {
      to_interp = map_slow_ease_in(from_interp);
    } else if (line_width_mapping == g_keyword_iname_slow_in_out) {
      to_interp = map_slow_ease_in_ease_out(from_interp);
    }
    half_width = (to_m * to_interp) + to_c;

    v_1.x = (n1.x * half_width) + xs;
    v_1.y = (n1.y * half_width) + ys;
    v_2.x = (n2.x * half_width) + xs;
    v_2.y = (n2.y * half_width) + ys;

    if (i == 0) {
      prepare_to_add_triangle_strip(render_data, matrix, tessellation * 2, v_1.x, v_1.y);
    }

    uv_t = tex_t * (f32)i;

    t_uv.x = lerp(uv_t, uv_b.x, uv_d.x);
    t_uv.y = lerp(uv_t, uv_b.y, uv_d.y);
    add_vertex(render_data->current_render_packet, matrix, v_1.x, v_1.y, rgb, t_uv);

    t_uv.x = lerp(uv_t, uv_a.x, uv_c.x);
    t_uv.y = lerp(uv_t, uv_a.y, uv_c.y);
    add_vertex(render_data->current_render_packet, matrix, v_2.x, v_2.y, rgb, t_uv);
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

  v_1.x = (n1.x * half_width_end) + xs_next;
  v_1.y = (n1.y * half_width_end) + ys_next;
  v_2.x = (n2.x * half_width_end) + xs_next;
  v_2.y = (n2.y * half_width_end) + ys_next;

  add_vertex(render_data->current_render_packet, matrix, v_1.x, v_1.y, rgb, uv_d);
  add_vertex(render_data->current_render_packet, matrix, v_2.x, v_2.y, rgb, uv_c);
}

