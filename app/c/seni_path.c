#include "seni_path.h"

#include "seni_keyword_iname.h"
#include "seni_lang.h"
#include "seni_mathutil.h"
#include "seni_matrix.h"
#include "seni_vm_compiler.h"
#include "seni_vm_interpreter.h"

#include <math.h>

// invoke a function with 3 args: step, position and t
void invoke_function(seni_vm *vm, i32 fn, f32 step, f32 t, f32 x, f32 y)
{
  seni_program *program = vm->program;
  seni_fn_info *fn_info = &(program->fn_info[fn]);
  seni_var *dest;
  seni_var src;

  vm_setup_function(vm, fn_info);
  
  i32 arg = vm->fp - 1;

  i32 istep = get_argument_mapping(fn_info, INAME_N);
  if (istep != -1) {
    // value for step
    dest = &(vm->stack[arg - istep]);
    src.type = VAR_FLOAT;
    src.value.f = step;
    var_copy(dest, &src);
  }

  i32 ipos = get_argument_mapping(fn_info, INAME_POSITION);
  if (ipos != -1) {
    // value for position
    dest = &(vm->stack[arg - ipos]);
    src.type = VAR_2D;
    src.value.i = 0;
    src.f32_array[0] = x;
    src.f32_array[1] = y;
    var_copy(dest, &src);
  }

  i32 it = get_argument_mapping(fn_info, INAME_T);
  if (it != -1) {
    // value for t
    dest = &(vm->stack[arg - it]);
    src.type = VAR_FLOAT;
    src.value.f = t;
    var_copy(dest, &src);
  }
    
  vm_call_function(vm);  
}

void path_linear(seni_vm *vm, i32 fn, i32 steps, f32 t_start, f32 t_end, f32 a_x, f32 a_y, f32 b_x, f32 b_y)
{
  f32 x_unit = (b_x - a_x) / (f32)(steps - 1);
  f32 y_unit = (b_y - a_y) / (f32)(steps - 1);
  f32 x, y, t, step;

  // todo: actually use t_start and t_end

  x = t_end;                    // prevent compiler warning for t_end
  x = t_start;

  for (i32 i = 0; i < steps; i++) {
    step = (f32)i;
    t = (f32)i / (f32)steps;
    x = a_x + (i * x_unit);
    y = a_y + (i * y_unit);
    invoke_function(vm, fn, step, t, x, y);
  }
}

void path_circle(seni_vm *vm, i32 fn, i32 steps, f32 t_start, f32 t_end, f32 pos_x, f32 pos_y, f32 radius)
{
  f32 unit = (t_end - t_start) / (f32)steps;
  f32 unit_angle = unit * TAU;

  f32 angle, vx, vy;

  f32 step, t;
  
  for (i32 i = 0; i < steps; i++) {
    step = (f32)i;
    angle = (unit_angle * step) + (t_start * TAU);
    vx = ((f32)sin(angle) * radius) + pos_x;
    vy = ((f32)cos(angle) * radius) + pos_y;
    t = t_start + (unit * step);

    invoke_function(vm, fn, step, t, vx, vy);
  }
}

void path_spline(seni_vm *vm, i32 fn, i32 steps, f32 t_start, f32 t_end, f32 *coords)
{
  f32 unit = (t_end - t_start) / ((f32)steps - 1.0f);
  f32 t, x, y;
  
  for (i32 i = 0; i < steps; i++) {
    t = t_start + (i * unit);

    x = quadratic_point(coords[0], coords[2], coords[4], t);
    y = quadratic_point(coords[1], coords[3], coords[5], t);

    invoke_function(vm, fn, (f32)i, t, x, y);
  }
}

void path_bezier(seni_vm *vm, i32 fn, i32 steps, f32 t_start, f32 t_end, f32 *coords)
{
  f32 unit = (t_end - t_start) / ((f32)steps - 1.0f);
  f32 t, x, y;
  
  for (i32 i = 0; i < steps; i++) {
    t = t_start + (i * unit);

    x = bezier_point(coords[0], coords[2], coords[4], coords[6], t);
    y = bezier_point(coords[1], coords[3], coords[5], coords[7], t);

    invoke_function(vm, fn, (f32)i, t, x, y);
  }
}
