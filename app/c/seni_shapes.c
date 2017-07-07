#include "seni_shapes.h"
#include "seni_types.h"
#include "seni_uv_mapper.h"
#include "seni_mathutil.h"
#include "seni_render_packet.h"
#include "seni_keyword_iname.h"
#include "seni_colour.h"
#include "seni_prng.h"

#include <math.h>

seni_matrix g_identity;
seni_colour g_unseen_colour;
v2 g_unseen_uv;

void seni_shapes_init_globals()
{
  matrix_identity(&g_identity);
  colour_set(&g_unseen_colour, RGB, 0.0f, 0.0f, 0.0f, 0.0f);
  g_unseen_uv.x = 0.0f; g_unseen_uv.y = 0.0f; // u v
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

  // pre-multiply the alpha
  // see http://www.realtimerendering.com/blog/gpus-prefer-premultiplication/
  f32 alpha = rgb->element[3];

  render_packet->cbuf[c_index + 0] = rgb->element[0] * alpha;
  render_packet->cbuf[c_index + 1] = rgb->element[1] * alpha;
  render_packet->cbuf[c_index + 2] = rgb->element[2] * alpha;
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

  add_vertex(render_packet, &g_identity, last_v[0], last_v[1], &g_unseen_colour, g_unseen_uv);

  // add the new vertex to complete the degenerate triangle
  add_vertex(render_packet, matrix, x, y, &g_unseen_colour, g_unseen_uv);
  
  // Note: still need to call addVertex on the first
  // vertex when we 'really' render the strip
}

bool can_vertices_fit(seni_render_packet *render_packet, i32 num, i32 max_vertices)
{
  return render_packet->num_vertices < (max_vertices - (num + 2));
}

bool is_render_packet_empty(seni_render_packet *render_packet)
{
  return render_packet->num_vertices == 0;
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
  add_vertex(render_data->current_render_packet, matrix, to_x   + (hw * n.x),    to_y + (hw * n.y),  rgb, uv->map[2]);
  add_vertex(render_data->current_render_packet, matrix, to_x   + (hw * n2.x),   to_y + (hw * n2.y), rgb, uv->map[3]);
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

void render_quadratic(seni_render_data *render_data,
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
  seni_brush_type brush_type = (seni_brush_type)(brush - INAME_BRUSH_FLAT);
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

  f32 x0 = coords[0], x1 = coords[2], x2 = coords[4];
  f32 y0 = coords[1], y1 = coords[3], y2 = coords[5];
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

    xs = quadratic_point(x0, x1, x2, t_val);
    ys = quadratic_point(y0, y1, y2, t_val);
    xs_next = quadratic_point(x0, x1, x2, t_val_next);
    ys_next = quadratic_point(y0, y1, y2, t_val_next);

    // addVerticesAsStrip
    n1 = normal(xs, ys, xs_next, ys_next);
    n2 = opposite_normal(n1);
    
    from_interp = (from_m * t_val) + from_c;
    switch(line_width_mapping) {
    case INAME_QUICK:
      to_interp = map_quick_ease(from_interp);
      break;
    case INAME_SLOW_IN:
      to_interp = map_slow_ease_in(from_interp);
      break;
    case INAME_SLOW_IN_OUT:
      to_interp = map_slow_ease_in_ease_out(from_interp);
      break;
    default:
      to_interp = from_interp;    // default behaviour as though 'linear' was chosen
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

  xs = quadratic_point(x0, x1, x2, t_val);
  ys = quadratic_point(y0, y1, y2, t_val);
  xs_next = quadratic_point(x0, x1, x2, t_val_next);
  ys_next = quadratic_point(y0, y1, y2, t_val_next);

  n1 = normal(xs, ys, xs_next, ys_next);
  n2 = opposite_normal(n1);

  v_1.x = (n1.x * half_width_end) + xs_next;
  v_1.y = (n1.y * half_width_end) + ys_next;
  v_2.x = (n2.x * half_width_end) + xs_next;
  v_2.y = (n2.y * half_width_end) + ys_next;

  add_vertex(render_data->current_render_packet, matrix, v_1.x, v_1.y, rgb, uv_d);
  add_vertex(render_data->current_render_packet, matrix, v_2.x, v_2.y, rgb, uv_c);
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
  seni_brush_type brush_type = (seni_brush_type)(brush - INAME_BRUSH_FLAT);
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
    switch(line_width_mapping) {
    case INAME_QUICK:
      to_interp = map_quick_ease(from_interp);
      break;
    case INAME_SLOW_IN:
      to_interp = map_slow_ease_in(from_interp);
      break;
    case INAME_SLOW_IN_OUT:
      to_interp = map_slow_ease_in_ease_out(from_interp);
      break;
    default:
      to_interp = from_interp;    // default behaviour as though 'linear' was chosen
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


void render_stroked_bezier(seni_render_data *render_data,
                           seni_matrix *matrix,
                           f32 *coords,
                           seni_colour *colour, i32 tessellation,
                           f32 stroke_line_width_start, f32 stroke_line_width_end, f32 stroke_noise,
                           i32 stroke_tessellation, f32 colour_volatility, f32 seed,
                           i32 line_width_mapping, i32 brush, i32 brush_subtype)
{
  f32 x1 = coords[0], x2 = coords[2], x3 = coords[4], x4 = coords[6];
  f32 y1 = coords[1], y2 = coords[3], y3 = coords[5], y4 = coords[7];

  i32 si_num = tessellation + 2;
  f32 si_unit = 1.0f / (f32)(si_num - 1);

  seni_colour lab;
  colour_clone_as(&lab, colour, LAB);
  f32 lab_l = lab.element[0];

  f32 tvals0, tvals1, tvals2;
  f32 xx1, xx2, xx3;
  f32 yy1, yy2, yy3;
  f32 ns;
  f32 quad_coords[] = { 100.0f, 500.0f, 300.0f, 300.0f, 600.0f, 700.0f };

  for (i32 i = 0; i < tessellation; i++) {
    tvals0 = (i + 0) * si_unit;
    tvals1 = (i + 1) * si_unit;
    tvals2 = (i + 2) * si_unit;
    
    // get 3 points on the bezier curve
    xx1 = bezier_point(x1, x2, x3, x4, tvals0);
    xx2 = bezier_point(x1, x2, x3, x4, tvals1);
    xx3 = bezier_point(x1, x2, x3, x4, tvals2);

    yy1 = bezier_point(y1, y2, y3, y4, tvals0);
    yy2 = bezier_point(y1, y2, y3, y4, tvals1);
    yy3 = bezier_point(y1, y2, y3, y4, tvals2);

    ns = stroke_noise;

    lab.element[0] = lab_l + (seni_perlin(xx1, xx1, xx1) * colour_volatility);
      
    quad_coords[0] = xx1 + (ns * seni_perlin(xx1, xx1, seed));
    quad_coords[1] = yy1 + (ns * seni_perlin(yy1, yy1, seed));
    quad_coords[2] = xx2 + (ns * seni_perlin(xx2, xx1, seed));
    quad_coords[3] = yy2 + (ns * seni_perlin(yy2, yy1, seed));
    quad_coords[4] = xx3 + (ns * seni_perlin(xx3, xx1, seed));
    quad_coords[5] = yy3 + (ns * seni_perlin(yy3, yy1, seed));

      
    render_quadratic(render_data, matrix, quad_coords,
                     stroke_line_width_start, stroke_line_width_end, line_width_mapping,
                     0.0f, 1.0f, &lab, stroke_tessellation, brush, brush_subtype);
  }
}


void render_stroked_bezier_rect(seni_render_data *render_data,
                                seni_matrix *matrix,
                                f32 *position, f32 width, f32 height, f32 volatility, f32 overlap, f32 iterations,
                                f32 seed,
                                i32 tessellation, i32 stroke_tessellation, f32 stroke_noise,
                                seni_colour *colour, f32 colour_volatility,
                                i32 brush, i32 brush_subtype)
{
  f32 x = position[0];
  f32 y = position[1];

  f32 x_start = x - (width / 2.0f);
  f32 y_start = y - (height / 2.0f);

  f32 th_width = width / 3.0f;
  f32 th_height = height / 3.0f;
  f32 vol = volatility;  

  f32 h_delta = height / iterations;
  f32 h_strip_width = height / iterations;

  f32 v_delta = width / iterations;
  f32 v_strip_width = width / iterations;

  seni_colour half_alpha_col;
  colour_clone_as(&half_alpha_col, colour, LAB);
  half_alpha_col.element[3] = half_alpha_col.element[3] / 2.0f;

  seni_prng_state prng_state;
  seni_prng_set_state(&prng_state, (u64)seed);

  i32 i;
  i32 iiterations = (i32)iterations;

  f32 coords[] = { 100.0f, 500.0f, 300.0f, 300.0f, 600.0f, 700.0f, 900.0f, 900.0f };
  f32 stroke_line_width_start = overlap + h_strip_width;
  f32 stroke_line_width_end = overlap + h_strip_width;
  f32 stroke_line_half_width = stroke_line_width_start / 2.0f;

  // horizontal strokes
  //
  f32 h;
  for (i = iiterations; i > 0; i--) {
    h = (i * h_delta) + y_start - stroke_line_half_width;
    
    coords[0] = (seni_prng_f32_range(&prng_state, -1.0f, 1.0f) * vol) + x_start + (0 * th_width);
    coords[1] = h + (seni_prng_f32_range(&prng_state, -1.0f, 1.0f) * vol);

    coords[2] = (seni_prng_f32_range(&prng_state, -1.0f, 1.0f) * vol) + x_start + (1 * th_width);
    coords[3] = h + (seni_prng_f32_range(&prng_state, -1.0f, 1.0f) * vol);

    coords[4] = (seni_prng_f32_range(&prng_state, -1.0f, 1.0f) * vol) + x_start + (2 * th_width);
    coords[5] = h + (seni_prng_f32_range(&prng_state, -1.0f, 1.0f) * vol);

    coords[6] = (seni_prng_f32_range(&prng_state, -1.0f, 1.0f) * vol) + x_start + (3 * th_width);
    coords[7] = h + (seni_prng_f32_range(&prng_state, -1.0f, 1.0f) * vol);

    render_stroked_bezier(render_data, matrix, coords, &half_alpha_col, tessellation,
                          stroke_line_width_start, stroke_line_width_end, stroke_noise,
                          stroke_tessellation, colour_volatility, seni_prng_f32(&prng_state),
                          INAME_LINEAR, brush, brush_subtype);

  }


  stroke_line_width_start = overlap + v_strip_width;
  stroke_line_width_end = overlap + v_strip_width;
  stroke_line_half_width = stroke_line_width_start / 2.0f;

  f32 v;
  for (i = iiterations; i > 0; i--) {
    v = (i * v_delta) + x_start - stroke_line_half_width;
    
    coords[0] = v + (seni_prng_f32_range(&prng_state, -1.0f, 1.0f) * vol);
    coords[1] = (seni_prng_f32_range(&prng_state, -1.0f, 1.0f) * vol) + y_start + (0 * th_height);

    coords[2] = v + (seni_prng_f32_range(&prng_state, -1.0f, 1.0f) * vol);
    coords[3] = (seni_prng_f32_range(&prng_state, -1.0f, 1.0f) * vol) + y_start + (1 * th_height);

    coords[4] = v + (seni_prng_f32_range(&prng_state, -1.0f, 1.0f) * vol);
    coords[5] = (seni_prng_f32_range(&prng_state, -1.0f, 1.0f) * vol) + y_start + (2 * th_height);

    coords[6] = v + (seni_prng_f32_range(&prng_state, -1.0f, 1.0f) * vol);
    coords[7] = (seni_prng_f32_range(&prng_state, -1.0f, 1.0f) * vol) + y_start + (3 * th_height);

    render_stroked_bezier(render_data, matrix, coords, &half_alpha_col, tessellation,
                          stroke_line_width_start, stroke_line_width_end, stroke_noise,
                          stroke_tessellation, colour_volatility, seni_prng_f32(&prng_state),
                          INAME_LINEAR, brush, brush_subtype);
  }

}
