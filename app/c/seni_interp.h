#ifndef SENI_INTERP_H
#define SENI_INTERP_H

#include "seni_types.h"

f32 seni_interp_cos(f32 amplitude, f32 frequency, f32 t);
f32 seni_interp_sin(f32 amplitude, f32 frequency, f32 t);
v2 seni_interp_bezier(f32 *coords, f32 t);
v2 seni_interp_bezier_tangent(f32 *coords, f32 t);
v2 seni_interp_circle(f32 *position, f32 radius, f32 t);

#endif
