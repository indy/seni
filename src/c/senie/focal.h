#pragma once

#include "types.h"

/*
focal-point (focal/point position: [{500 (gen/int max: canvas/width)}
                                      {800 (gen/int max: canvas/height)}]
                         distance: {300 (gen/int max: canvas/width)})

volatility (* (focal/call using: focal-point position: p) focal-power)

*/

typedef enum { FOCAL_UNKNOWN = 0, FOCAL_POINT, FOCAL_HLINE, FOCAL_VLINE } senie_focal_type;

// returns how 'interesting' a point (given by x,y) should be
//
f32 focal_point(f32 x, f32 y, f32 distance, i32 mapping, f32 centre_x, f32 centre_y);
f32 focal_hline(f32 y, f32 distance, i32 mapping, f32 centre_y);
f32 focal_vline(f32 x, f32 distance, i32 mapping, f32 centre_x);
