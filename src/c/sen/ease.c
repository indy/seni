#include "ease.h"

#include "keyword_iname.h"
#include "mathutil.h"

#include <math.h>

// parabola y = x^2
f32 quadratic_ease_in(f32 p) { return p * p; }

// parabola y = -x^2 + 2x
f32 quadratic_ease_out(f32 p) { return -(p * (p - 2)); }

// piecewise quadratic
// y = (1/2)((2x)^2)             ; [0, 0.5)
// y = -(1/2)((2x-1)*(2x-3) - 1) ; [0.5, 1]
f32 quadratic_ease_in_out(f32 p) {
  if (p < 0.5) {
    return 2 * p * p;
  } else {
    return (-2 * p * p) + (4 * p) - 1;
  }
}

// cubic y = x^3
f32 cubic_ease_in(f32 p) { return p * p * p; }

// cubic y = (x - 1)^3 + 1
f32 cubic_ease_out(f32 p) {
  f32 f = (p - 1);
  return f * f * f + 1;
}

// piecewise cubic
// y = (1/2)((2x)^3)       ; [0, 0.5)
// y = (1/2)((2x-2)^3 + 2) ; [0.5, 1]
f32 cubic_ease_in_out(f32 p) {
  if (p < 0.5) {
    return 4 * p * p * p;
  } else {
    f32 f = ((2 * p) - 2);
    return 0.5 * f * f * f + 1;
  }
}

// quartic x^4
f32 quartic_ease_in(f32 p) { return p * p * p * p; }

// quartic y = 1 - (x - 1)^4
f32 quartic_ease_out(f32 p) {
  f32 f = (p - 1);
  return f * f * f * (1 - p) + 1;
}

// piecewise quartic
// y = (1/2)((2x)^4)        ; [0, 0.5)
// y = -(1/2)((2x-2)^4 - 2) ; [0.5, 1]
f32 quartic_ease_in_out(f32 p) {
  if (p < 0.5) {
    return 8 * p * p * p * p;
  } else {
    f32 f = (p - 1);
    return -8 * f * f * f * f + 1;
  }
}

// quintic y = x^5
f32 quintic_ease_in(f32 p) { return p * p * p * p * p; }

// quintic y = (x - 1)^5 + 1
f32 quintic_ease_out(f32 p) {
  f32 f = (p - 1);
  return f * f * f * f * f + 1;
}

// piecewise quintic
// y = (1/2)((2x)^5)       ; [0, 0.5)
// y = (1/2)((2x-2)^5 + 2) ; [0.5, 1]
f32 quintic_ease_in_out(f32 p) {
  if (p < 0.5) {
    return 16 * p * p * p * p * p;
  } else {
    f32 f = ((2 * p) - 2);
    return 0.5 * f * f * f * f * f + 1;
  }
}

// Modeled after quarter-cycle of sine wave
f32 sin_ease_in(f32 p) { return sinf((p - 1) * PI_BY_2) + 1; }

// Modeled after quarter-cycle of sine wave (different phase)
f32 sin_ease_out(f32 p) { return sinf(p * PI_BY_2); }

// Modeled after half sine wave
f32 sin_ease_in_out(f32 p) { return 0.5 * (1 - cos(p * PI)); }

// Modeled after shifted quadrant IV of unit circle
f32 circular_ease_in(f32 p) { return 1 - sqrtf(1 - (p * p)); }

// Modeled after shifted quadrant II of unit circle
f32 circular_ease_out(f32 p) { return sqrtf((2 - p) * p); }

// piecewise circular function
// y = (1/2)(1 - sqrtf(1 - 4x^2))           ; [0, 0.5)
// y = (1/2)(sqrtf(-(2x - 3)*(2x - 1)) + 1) ; [0.5, 1]
f32 circular_ease_in_out(f32 p) {
  if (p < 0.5) {
    return 0.5 * (1 - sqrtf(1 - 4 * (p * p)));
  } else {
    return 0.5 * (sqrtf(-((2 * p) - 3) * ((2 * p) - 1)) + 1);
  }
}

// exponential function y = 2^(10(x - 1))
f32 exponential_ease_in(f32 p) { return (p == 0.0) ? p : pow(2, 10 * (p - 1)); }

// exponential function y = -2^(-10x) + 1
f32 exponential_ease_out(f32 p) { return (p == 1.0) ? p : 1 - pow(2, -10 * p); }

// piecewise exponential
// y = (1/2)2^(10(2x - 1))         ; [0,0.5)
// y = -(1/2)*2^(-10(2x - 1))) + 1 ; [0.5,1]
f32 exponential_ease_in_out(f32 p) {
  if (p == 0.0 || p == 1.0)
    return p;

  if (p < 0.5) {
    return 0.5 * pow(2, (20 * p) - 10);
  } else {
    return -0.5 * pow(2, (-20 * p) + 10) + 1;
  }
}

// damped sine wave y = sinf(13pi/2*x)*pow(2, 10 * (x - 1))
f32 elastic_ease_in(f32 p) {
  return sinf(13 * PI_BY_2 * p) * pow(2, 10 * (p - 1));
}

// damped sine wave y = sinf(-13pi/2*(x + 1))*pow(2, -10x) + 1
f32 elastic_ease_out(f32 p) {
  return sinf(-13 * PI_BY_2 * (p + 1)) * pow(2, -10 * p) + 1;
}

// piecewise exponentially-damped sine wave:
// y = (1/2)*sinf(13pi/2*(2*x))*pow(2, 10 * ((2*x) - 1))      ; [0,0.5)
// y = (1/2)*(sinf(-13pi/2*((2x-1)+1))*pow(2,-10(2*x-1)) + 2) ; [0.5, 1]
f32 elastic_ease_in_out(f32 p) {
  if (p < 0.5) {
    return 0.5 * sinf(13 * PI_BY_2 * (2 * p)) * pow(2, 10 * ((2 * p) - 1));
  } else {
    return 0.5 * (sinf(-13 * PI_BY_2 * ((2 * p - 1) + 1)) *
                      pow(2, -10 * (2 * p - 1)) +
                  2);
  }
}

// overshooting cubic y = x^3-x*sinf(x*pi)
f32 back_ease_in(f32 p) { return p * p * p - p * sinf(p * PI); }

// Modeled after overshooting cubic y = 1-((1-x)^3-(1-x)*sinf((1-x)*pi))
f32 back_ease_out(f32 p) {
  f32 f = (1 - p);
  return 1 - (f * f * f - f * sinf(f * PI));
}

// piecewise overshooting cubic function:
// y = (1/2)*((2x)^3-(2x)*sinf(2*x*pi))           ; [0, 0.5)
// y = (1/2)*(1-((1-x)^3-(1-x)*sinf((1-x)*pi))+1) ; [0.5, 1]
f32 back_ease_in_out(f32 p) {
  if (p < 0.5) {
    f32 f = 2 * p;
    return 0.5 * (f * f * f - f * sinf(f * PI));
  } else {
    f32 f = (1 - (2 * p - 1));
    return 0.5 * (1 - (f * f * f - f * sinf(f * PI))) + 0.5;
  }
}

f32 bounce_ease_out(f32 p) {
  if (p < 4 / 11.0) {
    return (121 * p * p) / 16.0;
  } else if (p < 8 / 11.0) {
    return (363 / 40.0 * p * p) - (99 / 10.0 * p) + 17 / 5.0;
  } else if (p < 9 / 10.0) {
    return (4356 / 361.0 * p * p) - (35442 / 1805.0 * p) + 16061 / 1805.0;
  } else {
    return (54 / 5.0 * p * p) - (513 / 25.0 * p) + 268 / 25.0;
  }
}

f32 bounce_ease_in(f32 p) { return 1 - bounce_ease_out(1 - p); }

f32 bounce_ease_in_out(f32 p) {
  if (p < 0.5) {
    return 0.5 * bounce_ease_in(p * 2);
  } else {
    return 0.5 * bounce_ease_out(p * 2 - 1) + 0.5;
  }
}

f32 easing(f32 from, i32 mapping) {
  switch (mapping) {
  case INAME_LINEAR:
    return from;
  case INAME_EASE_QUICK:
    return map_quick_ease(from);
  case INAME_EASE_SLOW_IN:
    return map_slow_ease_in(from);
  case INAME_EASE_SLOW_IN_OUT:
    return map_slow_ease_in_ease_out(from);
  case INAME_EASE_QUADRATIC_IN:
    return quadratic_ease_in(from);
  case INAME_EASE_QUADRATIC_OUT:
    return quadratic_ease_out(from);
  case INAME_EASE_QUADRATIC_IN_OUT:
    return quadratic_ease_in_out(from);
  case INAME_EASE_CUBIC_IN:
    return cubic_ease_in(from);
  case INAME_EASE_CUBIC_OUT:
    return cubic_ease_out(from);
  case INAME_EASE_CUBIC_IN_OUT:
    return cubic_ease_in_out(from);
  case INAME_EASE_QUARTIC_IN:
    return quartic_ease_in(from);
  case INAME_EASE_QUARTIC_OUT:
    return quartic_ease_out(from);
  case INAME_EASE_QUARTIC_IN_OUT:
    return quartic_ease_in_out(from);
  case INAME_EASE_QUINTIC_IN:
    return quintic_ease_in(from);
  case INAME_EASE_QUINTIC_OUT:
    return quintic_ease_out(from);
  case INAME_EASE_QUINTIC_IN_OUT:
    return quintic_ease_in_out(from);
  case INAME_EASE_SIN_IN:
    return sin_ease_in(from);
  case INAME_EASE_SIN_OUT:
    return sin_ease_out(from);
  case INAME_EASE_SIN_IN_OUT:
    return sin_ease_in_out(from);
  case INAME_EASE_CIRCULAR_IN:
    return circular_ease_in(from);
  case INAME_EASE_CIRCULAR_OUT:
    return circular_ease_out(from);
  case INAME_EASE_CIRCULAR_IN_OUT:
    return circular_ease_in_out(from);
  case INAME_EASE_EXPONENTIAL_IN:
    return exponential_ease_in(from);
  case INAME_EASE_EXPONENTIAL_OUT:
    return exponential_ease_out(from);
  case INAME_EASE_EXPONENTIAL_IN_OUT:
    return exponential_ease_in_out(from);
  case INAME_EASE_ELASTIC_IN:
    return elastic_ease_in(from);
  case INAME_EASE_ELASTIC_OUT:
    return elastic_ease_out(from);
  case INAME_EASE_ELASTIC_IN_OUT:
    return elastic_ease_in_out(from);
  case INAME_EASE_BACK_IN:
    return back_ease_in(from);
  case INAME_EASE_BACK_OUT:
    return back_ease_out(from);
  case INAME_EASE_BACK_IN_OUT:
    return back_ease_in_out(from);
  case INAME_EASE_BOUNCE_IN:
    return bounce_ease_in(from);
  case INAME_EASE_BOUNCE_OUT:
    return bounce_ease_out(from);
  case INAME_EASE_BOUNCE_IN_OUT:
    return bounce_ease_in_out(from);
  default:
    return from;
  }
}
