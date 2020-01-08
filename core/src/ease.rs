// Copyright (C) 2020 Inderjit Gill <email@indy.io>

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::keywords::Keyword;
use crate::mathutil;

#[derive(Copy, Clone, Debug)]
pub enum Easing {
    Linear,
    Quick,
    SlowIn,
    SlowInOut,
    QuadraticIn,
    QuadraticOut,
    QuadraticInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    QuarticIn,
    QuarticOut,
    QuarticInOut,
    QuinticIn,
    QuinticOut,
    QuinticInOut,
    SinIn,
    SinOut,
    SinInOut,
    CircularIn,
    CircularOut,
    CircularInOut,
    ExponentialIn,
    ExponentialOut,
    ExponentialInOut,
    ElasticIn,
    ElasticOut,
    ElasticInOut,
    BackIn,
    BackOut,
    BackInOut,
    BounceIn,
    BounceOut,
    BounceInOut,
}

pub fn easing_from_keyword(kw: Keyword) -> Option<Easing> {
    match kw {
        Keyword::Linear => Some(Easing::Linear),
        Keyword::EaseQuick => Some(Easing::Quick),
        Keyword::EaseSlowIn => Some(Easing::SlowIn),
        Keyword::EaseSlowInOut => Some(Easing::SlowInOut),
        Keyword::EaseQuadraticIn => Some(Easing::QuadraticIn),
        Keyword::EaseQuadraticOut => Some(Easing::QuadraticOut),
        Keyword::EaseQuadraticInOut => Some(Easing::QuadraticInOut),
        Keyword::EaseCubicIn => Some(Easing::CubicIn),
        Keyword::EaseCubicOut => Some(Easing::CubicOut),
        Keyword::EaseCubicInOut => Some(Easing::CubicInOut),
        Keyword::EaseQuarticIn => Some(Easing::QuarticIn),
        Keyword::EaseQuarticOut => Some(Easing::QuarticOut),
        Keyword::EaseQuarticInOut => Some(Easing::QuarticInOut),
        Keyword::EaseQuinticIn => Some(Easing::QuinticIn),
        Keyword::EaseQuinticOut => Some(Easing::QuinticOut),
        Keyword::EaseQuinticInOut => Some(Easing::QuinticInOut),
        Keyword::EaseSinIn => Some(Easing::SinIn),
        Keyword::EaseSinOut => Some(Easing::SinOut),
        Keyword::EaseSinInOut => Some(Easing::SinInOut),
        Keyword::EaseCircularIn => Some(Easing::CircularIn),
        Keyword::EaseCircularOut => Some(Easing::CircularOut),
        Keyword::EaseCircularInOut => Some(Easing::CircularInOut),
        Keyword::EaseExponentialIn => Some(Easing::ExponentialIn),
        Keyword::EaseExponentialOut => Some(Easing::ExponentialOut),
        Keyword::EaseExponentialInOut => Some(Easing::ExponentialInOut),
        Keyword::EaseElasticIn => Some(Easing::ElasticIn),
        Keyword::EaseElasticOut => Some(Easing::ElasticOut),
        Keyword::EaseElasticInOut => Some(Easing::ElasticInOut),
        Keyword::EaseBackIn => Some(Easing::BackIn),
        Keyword::EaseBackOut => Some(Easing::BackOut),
        Keyword::EaseBackInOut => Some(Easing::BackInOut),
        Keyword::EaseBounceIn => Some(Easing::BounceIn),
        Keyword::EaseBounceOut => Some(Easing::BounceOut),
        Keyword::EaseBounceInOut => Some(Easing::BounceInOut),
        _ => None,
    }
}

pub fn easing(from: f32, easing: Easing) -> f32 {
    match easing {
        Easing::Linear => from,
        Easing::Quick => mathutil::map_quick_ease(from),
        Easing::SlowIn => mathutil::map_slow_ease_in(from),
        Easing::SlowInOut => mathutil::map_slow_ease_in_ease_out(from),
        Easing::QuadraticIn => quadratic_ease_in(from),
        Easing::QuadraticOut => quadratic_ease_out(from),
        Easing::QuadraticInOut => quadratic_ease_in_out(from),
        Easing::CubicIn => cubic_ease_in(from),
        Easing::CubicOut => cubic_ease_out(from),
        Easing::CubicInOut => cubic_ease_in_out(from),
        Easing::QuarticIn => quartic_ease_in(from),
        Easing::QuarticOut => quartic_ease_out(from),
        Easing::QuarticInOut => quartic_ease_in_out(from),
        Easing::QuinticIn => quintic_ease_in(from),
        Easing::QuinticOut => quintic_ease_out(from),
        Easing::QuinticInOut => quintic_ease_in_out(from),
        Easing::SinIn => sin_ease_in(from),
        Easing::SinOut => sin_ease_out(from),
        Easing::SinInOut => sin_ease_in_out(from),
        Easing::CircularIn => circular_ease_in(from),
        Easing::CircularOut => circular_ease_out(from),
        Easing::CircularInOut => circular_ease_in_out(from),
        Easing::ExponentialIn => exponential_ease_in(from),
        Easing::ExponentialOut => exponential_ease_out(from),
        Easing::ExponentialInOut => exponential_ease_in_out(from),
        Easing::ElasticIn => elastic_ease_in(from),
        Easing::ElasticOut => elastic_ease_out(from),
        Easing::ElasticInOut => elastic_ease_in_out(from),
        Easing::BackIn => back_ease_in(from),
        Easing::BackOut => back_ease_out(from),
        Easing::BackInOut => back_ease_in_out(from),
        Easing::BounceIn => bounce_ease_in(from),
        Easing::BounceOut => bounce_ease_out(from),
        Easing::BounceInOut => bounce_ease_in_out(from),
    }
}

// parabola y = x^2
fn quadratic_ease_in(p: f32) -> f32 {
    p * p
}

// parabola y = -x^2 + 2x
fn quadratic_ease_out(p: f32) -> f32 {
    -(p * (p - 2.0))
}

// piecewise quadratic
// y = (1/2)((2x)^2)             ; [0, 0.5)
// y = -(1/2)((2x-1)*(2x-3) - 1) ; [0.5, 1]
fn quadratic_ease_in_out(p: f32) -> f32 {
    if p < 0.5 {
        2.0 * p * p
    } else {
        (-2.0 * p * p) + (4.0 * p) - 1.0
    }
}

// cubic y = x^3
fn cubic_ease_in(p: f32) -> f32 {
    p * p * p
}

// cubic y = (x - 1)^3 + 1
fn cubic_ease_out(p: f32) -> f32 {
    let f = p - 1.0;
    f * f * f + 1.0
}

// piecewise cubic
// y = (1/2)((2x)^3)       ; [0, 0.5)
// y = (1/2)((2x-2)^3 + 2) ; [0.5, 1]
fn cubic_ease_in_out(p: f32) -> f32 {
    if p < 0.5 {
        4.0 * p * p * p
    } else {
        let f = (2.0 * p) - 2.0;
        0.5 * f * f * f + 1.0
    }
}

// quartic x^4
fn quartic_ease_in(p: f32) -> f32 {
    p * p * p * p
}

// quartic y = 1 - (x - 1)^4
fn quartic_ease_out(p: f32) -> f32 {
    let f = p - 1.0;
    f * f * f * (1.0 - p) + 1.0
}

// piecewise quartic
// y = (1/2)((2x)^4)        ; [0, 0.5)
// y = -(1/2)((2x-2)^4 - 2) ; [0.5, 1]
fn quartic_ease_in_out(p: f32) -> f32 {
    if p < 0.5 {
        8.0 * p * p * p * p
    } else {
        let f = p - 1.0;
        -8.0 * f * f * f * f + 1.0
    }
}

// quintic y = x^5
fn quintic_ease_in(p: f32) -> f32 {
    p * p * p * p * p
}

// quintic y = (x - 1)^5 + 1
fn quintic_ease_out(p: f32) -> f32 {
    let f = p - 1.0;
    f * f * f * f * f + 1.0
}

// piecewise quintic
// y = (1/2)((2x)^5)       ; [0, 0.5)
// y = (1/2)((2x-2)^5 + 2) ; [0.5, 1]
fn quintic_ease_in_out(p: f32) -> f32 {
    if p < 0.5 {
        16.0 * p * p * p * p * p
    } else {
        let f = (2.0 * p) - 2.0;
        0.5 * f * f * f * f * f + 1.0
    }
}

// Modeled after quarter-cycle of sine wave
fn sin_ease_in(p: f32) -> f32 {
    ((p - 1.0) * mathutil::PI_BY_2).sin() + 1.0
}

// Modeled after quarter-cycle of sine wave (different phase)
fn sin_ease_out(p: f32) -> f32 {
    (p * mathutil::PI_BY_2).sin()
}

// Modeled after half sine wave
fn sin_ease_in_out(p: f32) -> f32 {
    0.5 * (1.0 - (p * mathutil::PI).cos())
}

// Modeled after shifted quadrant IV of unit circle
fn circular_ease_in(p: f32) -> f32 {
    1.0 - (1.0 - (p * p)).sqrt()
}

// Modeled after shifted quadrant II of unit circle
fn circular_ease_out(p: f32) -> f32 {
    ((2.0 - p) * p).sqrt()
}

// piecewise circular function
// y = (1/2)(1 - sqrtf(1 - 4x^2))           ; [0, 0.5)
// y = (1/2)(sqrtf(-(2x - 3)*(2x - 1)) + 1) ; [0.5, 1]
fn circular_ease_in_out(p: f32) -> f32 {
    if p < 0.5 {
        0.5 * (1.0 - (1.0 - 4.0 * (p * p)).sqrt())
    } else {
        0.5 * ((-((2.0 * p) - 3.0) * ((2.0 * p) - 1.0)).sqrt() + 1.0)
    }
}

// exponential function y = 2^(10(x - 1))
fn exponential_ease_in(p: f32) -> f32 {
    if p == 0.0 {
        p
    } else {
        2.0_f32.powf(10.0 * (p - 1.0))
    }
}

// exponential function y = -2^(-10x) + 1
fn exponential_ease_out(p: f32) -> f32 {
    if (p - 1.0).abs() < std::f32::EPSILON {
        p
    } else {
        1.0 - 2.0_f32.powf(-10.0 * p)
    }
}

// piecewise exponential
// y = (1/2)2^(10(2x - 1))         ; [0,0.5)
// y = -(1/2)*2^(-10(2x - 1))) + 1 ; [0.5,1]
fn exponential_ease_in_out(p: f32) -> f32 {
    if p.abs() < std::f32::EPSILON || (p - 1.0).abs() < std::f32::EPSILON {
        return p;
    }

    if p < 0.5 {
        0.5 * 2.0_f32.powf((20.0 * p) - 10.0)
    } else {
        -0.5 * 2.0_f32.powf((-20.0 * p) + 10.0) + 1.0
    }
}

// damped sine wave y = sinf(13pi/2*x)*pow(2, 10 * (x - 1))
fn elastic_ease_in(p: f32) -> f32 {
    (13.0 * mathutil::PI_BY_2 * p).sin() * 2.0_f32.powf(10.0 * (p - 1.0))
}

// damped sine wave y = sinf(-13pi/2*(x + 1))*pow(2, -10x) + 1
fn elastic_ease_out(p: f32) -> f32 {
    (-13.0 * mathutil::PI_BY_2 * (p + 1.0)).sin() * 2.0_f32.powf(-10.0 * p) + 1.0
}

// piecewise exponentially-damped sine wave:
// y = (1/2)*sinf(13pi/2*(2*x))*pow(2, 10 * ((2*x) - 1))      ; [0,0.5)
// y = (1/2)*(sinf(-13pi/2*((2x-1)+1))*pow(2,-10(2*x-1)) + 2) ; [0.5, 1]
fn elastic_ease_in_out(p: f32) -> f32 {
    if p < 0.5 {
        0.5 * (13.0 * mathutil::PI_BY_2 * (2.0 * p)).sin() * 2.0_f32.powf(10.0 * ((2.0 * p) - 1.0))
    } else {
        0.5 * ((-13.0 * mathutil::PI_BY_2 * ((2.0 * p - 1.0) + 1.0)).sin()
            * 2.0_f32.powf(-10.0 * (2.0 * p - 1.0))
            + 2.0)
    }
}

// overshooting cubic y = x^3-x*sinf(x*pi)
fn back_ease_in(p: f32) -> f32 {
    p * p * p - p * (p * mathutil::PI).sin()
}

// Modeled after overshooting cubic y = 1-((1-x)^3-(1-x)*sinf((1-x)*pi))
fn back_ease_out(p: f32) -> f32 {
    let f = 1.0 - p;
    1.0 - (f * f * f - f * (f * mathutil::PI).sin())
}

// piecewise overshooting cubic function:
// y = (1/2)*((2x)^3-(2x)*sinf(2*x*pi))           ; [0, 0.5)
// y = (1/2)*(1-((1-x)^3-(1-x)*sinf((1-x)*pi))+1) ; [0.5, 1]
fn back_ease_in_out(p: f32) -> f32 {
    if p < 0.5 {
        let f = 2.0 * p;
        0.5 * (f * f * f - f * (f * mathutil::PI).sin())
    } else {
        let f = 1.0 - (2.0 * p - 1.0);
        0.5 * (1.0 - (f * f * f - f * (f * mathutil::PI).sin())) + 0.5
    }
}

fn bounce_ease_out(p: f32) -> f32 {
    if p < 4.0 / 11.0 {
        (121.0 * p * p) / 16.0
    } else if p < 8.0 / 11.0 {
        (363.0 / 40.0 * p * p) - (99.0 / 10.0 * p) + 17.0 / 5.0
    } else if p < 9.0 / 10.0 {
        (4356.0 / 361.0 * p * p) - (35442.0 / 1805.0 * p) + 16061.0 / 1805.0
    } else {
        (54.0 / 5.0 * p * p) - (513.0 / 25.0 * p) + 268.0 / 25.0
    }
}

fn bounce_ease_in(p: f32) -> f32 {
    1.0 - bounce_ease_out(1.0 - p)
}

fn bounce_ease_in_out(p: f32) -> f32 {
    if p < 0.5 {
        0.5 * bounce_ease_in(p * 2.0)
    } else {
        0.5 * bounce_ease_out(p * 2.0 - 1.0) + 0.5
    }
}
