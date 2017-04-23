#include "seni_bind.h"
#include "seni_bind_core.h"
#include "seni_bind_shapes.h"

// a register like seni_var for holding intermediate values
seni_var g_reg;

i32 g_arg_colour = 0;
i32 g_arg_coords = 0;
i32 g_arg_from = 0;
i32 g_arg_height = 0;
i32 g_arg_increment = 0;
i32 g_arg_line_width = 0;
i32 g_arg_line_width_end = 0;
i32 g_arg_line_width_mapping = 0;
i32 g_arg_line_width_start = 0;
i32 g_arg_position = 0;
i32 g_arg_radius = 0;
i32 g_arg_steps = 0;
i32 g_arg_t_end = 0;
i32 g_arg_t_start = 0;
i32 g_arg_tessellation = 0;
i32 g_arg_to = 0;
i32 g_arg_upto = 0;
i32 g_arg_width = 0;

void interpreter_declare_keywords(word_lut *wlut)
{
#ifdef SENI_DEBUG_MODE
  g_reg.debug_allocatable = false;
#endif

  wlut->keywords_count = 0;

  // common parameters used by keywords and the standard api
  declare_common_arg(wlut, "colour",             &g_arg_colour);
  declare_common_arg(wlut, "coords",             &g_arg_coords);
  declare_common_arg(wlut, "from",               &g_arg_from);
  declare_common_arg(wlut, "height",             &g_arg_height);
  declare_common_arg(wlut, "increment",          &g_arg_increment);
  declare_common_arg(wlut, "line-width",         &g_arg_line_width);
  declare_common_arg(wlut, "line-width-end",     &g_arg_line_width_end);
  declare_common_arg(wlut, "line-width-mapping", &g_arg_line_width_mapping);
  declare_common_arg(wlut, "line-width-start",   &g_arg_line_width_start);
  declare_common_arg(wlut, "position",           &g_arg_position);
  declare_common_arg(wlut, "radius",             &g_arg_radius);
  declare_common_arg(wlut, "steps",              &g_arg_steps);
  declare_common_arg(wlut, "t-end",              &g_arg_t_end);
  declare_common_arg(wlut, "t-start",            &g_arg_t_start);
  declare_common_arg(wlut, "tessellation",       &g_arg_tessellation);
  declare_common_arg(wlut, "to",                 &g_arg_to);
  declare_common_arg(wlut, "upto",               &g_arg_upto);
  declare_common_arg(wlut, "width",              &g_arg_width);
  

  bind_core_declarations(wlut);
  bind_shape_declarations(wlut);
}
