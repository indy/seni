#include "seni_bind_shapes.h"
#include "seni_shapes.h"
#include "seni_bind.h"
#include "seni_buffer.h"

#include "seni_lang.h"
#include "seni_vm.h"

#include <stdio.h>


// extern global keyword variables
#define KEYWORD(val,_,name) extern i32 g_keyword_iname_##name;
#include "seni_keywords.h"
#undef KEYWORD

seni_var *eval_fn_line(seni_env *env, seni_node *expr)
{
  // get the values/defaults
  //
  seni_node *parameters = safe_next(expr);

  f32 width = get_named_f32(env, parameters, g_keyword_iname_width, 5.0f);

  f32 from_x = 50.0f; f32 from_y = 50.0f;
  get_named_vec2(env, parameters, g_keyword_iname_from, &from_x, &from_y);

  f32 to_x = 500.0f; f32 to_y = 500.0f;
  get_named_vec2(env, parameters, g_keyword_iname_to, &to_x, &to_y);

  f32 r = 0.0f; f32 g = 0.0f; f32 b = 0.0f; f32 a = 1.0f;
  get_named_vec4(env, parameters, g_keyword_iname_colour, &r, &g, &b, &a);
  rgba col;
  col.r = r; col.g = g; col.b = b; col.a = a;


  seni_buffer *buffer = env->buffer;
  //printf("before: buffer size %d %d\n", buffer->max_vertices, buffer->num_vertices);

  render_line(buffer, from_x, from_y, to_x, to_y, width, col);
  
  //printf("after: buffer size %d %d\n", buffer->max_vertices, buffer->num_vertices);

  return NULL;
}

seni_var *eval_fn_rect(seni_env *env, seni_node *expr)
{
  // get the values/defaults
  //
  seni_node *parameters = safe_next(expr);

  f32 width = get_named_f32(env, parameters, g_keyword_iname_width, 500.0f);
  f32 height = get_named_f32(env, parameters, g_keyword_iname_height, 500.0f);

  f32 x = 500.0f; f32 y = 500.0f;
  get_named_vec2(env, parameters, g_keyword_iname_position, &x, &y);

  f32 r = 0.0f; f32 g = 0.0f; f32 b = 0.0f; f32 a = 1.0f;
  get_named_vec4(env, parameters, g_keyword_iname_colour, &r, &g, &b, &a);
  rgba col;
  col.r = r; col.g = g; col.b = b; col.a = a;

  seni_buffer *buffer = env->buffer;
  //printf("before: buffer size %d %d\n", buffer->max_vertices, buffer->num_vertices);

  render_rect(buffer, x, y, width, height, col);
  
  //printf("after: buffer size %d %d\n", buffer->max_vertices, buffer->num_vertices);

  return NULL;
}

seni_var *eval_fn_circle(seni_env *env, seni_node *expr)
{
  // get the values/defaults
  //
  seni_node *parameters = safe_next(expr);

  f32 width = get_named_f32(env, parameters, g_keyword_iname_width, 500.0f);
  f32 height = get_named_f32(env, parameters, g_keyword_iname_height, 500.0f);
  f32 radius = get_named_f32(env, parameters, g_keyword_iname_radius, 100.0f);
  i32 tessellation = get_named_i32(env, parameters, g_keyword_iname_tessellation, 5);

  f32 x = 500.0f; f32 y = 500.0f;
  get_named_vec2(env, parameters, g_keyword_iname_position, &x, &y);

  if (has_named_node(parameters, g_keyword_iname_radius)) {
    // use the radius for both width and height if it's given
    width = radius;
    height = radius;
  }

  f32 r = 0.0f; f32 g = 0.0f; f32 b = 0.0f; f32 a = 1.0f;
  get_named_vec4(env, parameters, g_keyword_iname_colour, &r, &g, &b, &a);
  rgba col;
  col.r = r; col.g = g; col.b = b; col.a = a;

  seni_buffer *buffer = env->buffer;
  //printf("before: buffer size %d %d\n", buffer->max_vertices, buffer->num_vertices);

  render_circle(buffer, x, y, width, height, col, tessellation);

  //printf("after: buffer size %d %d\n", buffer->max_vertices, buffer->num_vertices);

  return NULL;
}

seni_var *eval_fn_bezier(seni_env *env, seni_node *expr)
{
  // get the values/defaults
  //
  seni_node *parameters = safe_next(expr);

  i32 tessellation = get_named_i32(env, parameters, g_keyword_iname_tessellation, 5);
  f32 line_width = get_named_f32(env, parameters, g_keyword_iname_line_width, 5.0f);
  f32 line_width_start = get_named_f32(env, parameters, g_keyword_iname_line_width_start, 5.0f);
  f32 line_width_end = get_named_f32(env, parameters, g_keyword_iname_line_width_end, 5.0f);
  i32 line_width_mapping = get_named_i32(env, parameters, g_keyword_iname_line_width_mapping, 0);
  seni_var *coords_var = get_named_var(env, parameters, g_keyword_iname_coords);
  v2 coords[4];
  if (coords_var && var_vector_length(coords_var) == 4) {
    seni_var *rc = coords_var->value.v;
    seni_var *e = rc->value.v;
    var_as_vec2(&(coords[0].x), &(coords[0].y), e);
    e = e->next;
    var_as_vec2(&(coords[1].x), &(coords[1].y), e);
    e = e->next;
    var_as_vec2(&(coords[2].x), &(coords[2].y), e);
    e = e->next;
    var_as_vec2(&(coords[3].x), &(coords[3].y), e);
    e = e->next;
  } else {
    // default values for coords
  }
  f32 t_start = get_named_f32(env, parameters, g_keyword_iname_t_start, 0.0f);
  f32 t_end = get_named_f32(env, parameters, g_keyword_iname_t_end, 1.0f);
  f32 r = 0.0f; f32 g = 0.0f; f32 b = 0.0f; f32 a = 1.0f;
  get_named_vec4(env, parameters, g_keyword_iname_colour, &r, &g, &b, &a);
  rgba col; col.r = r; col.g = g; col.b = b; col.a = a;

  seni_buffer *buffer = env->buffer;
  //printf("before: buffer size %d %d\n", buffer->max_vertices, buffer->num_vertices);

  render_bezier(buffer,
                coords,
                line_width, line_width_start, line_width_end, line_width_mapping,
                t_start, t_end,
                col, tessellation);
  
  //printf("after: buffer size %d %d\n", buffer->max_vertices, buffer->num_vertices);
  return NULL;
}

void bind_shape_declarations(seni_word_lut *wlut)
{
  declare_keyword(wlut, "line", &eval_fn_line);
  declare_keyword(wlut, "rect", &eval_fn_rect);
  declare_keyword(wlut, "circle", &eval_fn_circle);
  declare_keyword(wlut, "bezier", &eval_fn_bezier);
}

void native_fn_line(seni_virtual_machine *vm, i32 num_args)
{
  // default values for line
  f32 width = 4.0f;
  f32 height = 10.0f;

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(width);
  READ_STACK_ARG_F32(height);
  READ_STACK_ARGS_END;

  seni_var res;
  res.type = VAR_FLOAT;
  res.value.f = 17.0f;

  printf("native_fn_line width: %.2f height: %.2f\n", width, height);


  
  // push the return value onto the stack
  WRITE_STACK(res);
}

void native_fn_rect(seni_virtual_machine *vm, i32 num_args)
{
  // default values for line
  f32 width = 4.0f;
  f32 height = 10.0f;
  f32 position[] = {10.0f, 23.0f};
  f32 colour[] = { 0.0f, 1.0f, 0.0f, 1.0f };

  // update with values from stack
  READ_STACK_ARGS_BEGIN;
  READ_STACK_ARG_F32(width);
  READ_STACK_ARG_F32(height);
  READ_STACK_ARG_VEC2(position);
  READ_STACK_ARG_VEC4(colour);
  READ_STACK_ARGS_END;

  seni_var res;
  res.type = VAR_BOOLEAN;
  res.value.i = 1;

  rgba col;
  col.r = colour[0]; col.g = colour[1]; col.b = colour[2]; col.a = colour[3];

  render_rect(vm->buffer, position[0], position[1], width, height, col);

  // push the return value onto the stack
  WRITE_STACK(res);
}


void bind_vm_shape_declarations(seni_word_lut *wlut, seni_vm_environment *e)
{
  declare_vm_native(wlut, "line", e, &native_fn_line);
  declare_vm_native(wlut, "rect", e, &native_fn_rect);
  declare_vm_native(wlut, "circle", e, &native_fn_line);
  declare_vm_native(wlut, "bezier", e, &native_fn_line);
}
