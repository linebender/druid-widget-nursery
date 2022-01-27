// Copyright 2019 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt;
use std::fmt::{Debug, Formatter};
use std::f64::consts::PI;

impl From<fn(f64) -> f64> for AnimationCurve {
    fn from(f: fn(f64) -> f64) -> Self {
        AnimationCurve::Function(f)
    }
}

/// Animation Curve
///
/// An animation curve, mapping from time in the range 0..1 to progress in a range "around" 0..1.
/// It is permissible for progress to undershoot and overshoot.
///
/// Inspired by Robert Penner’s [easing] functions.
///
/// [easing]: http://robertpenner.com/easing/
pub enum AnimationCurve {
    /// Defined with constant function.
    Function(fn(f64) -> f64),
    /// Defined with closure.
    Closure(Box<dyn Fn(f64) -> f64>),
    /// Defined as Cubic Bezier curve.
    CubicBezier(CubicBezierAnimationCurve),
}

impl Default for AnimationCurve {
    fn default() -> Self {
        AnimationCurve::LINEAR
    }
}

impl Debug for AnimationCurve {
    // Required as closures are not Debug
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AnimationCurve::Function(f) => formatter
                .debug_struct("AnimationCurve::Function")
                .field("f", f)
                .finish(),
            AnimationCurve::Closure(_) => formatter
                .debug_struct("AnimationCurve::Closure")
                .finish(),
            AnimationCurve::CubicBezier(b) => formatter
                .debug_struct("AnimationCurve::CubicBezier")
                .field("x1", &b.x1)
                .field("y1", &b.y1)
                .field("x2", &b.x2)
                .field("y2", &b.y2)
                .finish(),
        }
    }
}

impl AnimationCurve {

    /// F(t) -> t
    pub const LINEAR: Self = Self::Function(|t| t);

    /// F(t) -> t²
    pub const EASE_IN: Self = Self::Function(ease_in);
    /// Flipped  [`EASE_IN`](AnimationCurve::EASE_IN)
    pub const EASE_OUT: Self = Self::Function(|t| flip_curve(ease_in, t));
    /// combines [`EASE_IN`](AnimationCurve::EASE_IN) and [`EASE_OUT`](AnimationCurve::EASE_OUT)
    pub const EASE_IN_OUT: Self = Self::Function(|t| combine_in_out(ease_in, t));

    /// Oscillating curve that grows in magnitude while overshooting its bounds.
    pub const EASE_IN_ELASTIC: Self = Self::Function(|t| flip_curve(ease_out_elastic, t));
    /// Oscillating curve that shrink in magnitude while overshooting its bounds.
    pub const EASE_OUT_ELASTIC: Self = Self::Function(ease_out_elastic);
    /// Oscillating curve that grows and then shrinks in magnitude
    /// while overshooting its bounds.
    pub const EASE_IN_OUT_ELASTIC: Self = Self::Function(|t| combine_in_out_rev(ease_out_elastic, t));

    /// F(t) -> 1 - cos(tπ/2)
    pub const EASE_IN_SINE: Self = Self::Function(|t| 1.0 - (t * PI * 0.5).cos());
    /// F(t) -> sin(tπ/2)
    pub const EASE_OUT_SINE: Self = Self::Function(|t| (t * PI * 0.5).sin());
    /// combines [`EASE_IN_SINE`](AnimationCurve::EASE_IN_SINE) and [`EASE_OUT_SINE`](AnimationCurve::EASE_OUT_SINE)
    pub const EASE_IN_OUT_SINE: Self = Self::Function(|t| -0.5 * (t * PI).cos() + 0.5);

    /// F(t) -> 2¹⁰⁽ᵗ⁻¹⁾
    pub const EASE_IN_EXPO: Self = Self::Function(ease_in_expo);
    ///  Flipped  [`EASE_IN_EXPO`](AnimationCurve::EASE_IN_EXPO)
    pub const EASE_OUT_EXPO: Self = Self::Function(|t| flip_curve(ease_in_expo, t));
    /// combines [`EASE_IN_EXPO`](AnimationCurve::EASE_IN_EXPO) and [`EASE_OUT_EXPO`](AnimationCurve::EASE_OUT_EXPO)
    pub const EASE_IN_OUT_EXPO: Self = Self::Function(|t| combine_in_out(ease_in_expo, t));

    /// A Cubic curve that undershoots slowly a start and ends quickly.
    pub const EASE_IN_BACK: Self = Self::cubic(0.36, 0.0, 0.66, -0.56);
    /// A Cubic curve that starts quickly and ends with slowly, with an overshoot at the end.
    pub const EASE_OUT_BACK: Self = Self::cubic(0.34, 1.56, 0.64, 1.0);
    /// combines [`EASE_IN_BACK`](AnimationCurve::EASE_IN_BACK) and [`EASE_OUT_BACK`](AnimationCurve::EASE_OUT_BACK)
    pub const EASE_IN_OUT_BACK: Self = Self::cubic(0.68, -0.6, 0.32, 1.6);

    /// Oscillating curve that grows larger, mimicking a bounce effect
    pub const BOUNCE_IN: Self = Self::Function(|t| flip_curve(bounce, t));
    /// Flipped [`BOUNCE_IN`](AnimationCurve::BOUNCE_IN)
    pub const BOUNCE_OUT: Self = Self::Function(bounce);
    /// combines [`BOUNCE_IN`](AnimationCurve::BOUNCE_IN) and [`BOUNCE_OUT`](AnimationCurve::BOUNCE_OUT)
    pub const BOUNCE_IN_OUT: Self = Self::Function(|t|  combine_in_out_rev(bounce, t));

    /// Create a Cubic Bezier curve.
    pub const fn cubic(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self::CubicBezier(CubicBezierAnimationCurve { x1, y1, x2, y2 })
    }

    /// Returns the value of the curve at point `t`.
    pub fn translate(&self, t: f64) -> f64 {
        match self {
            Self::Function(f) => f(t),
            Self::Closure(c) => c(t),
            Self::CubicBezier(b) => b.translate(t),
        }
    }

    /// Create an instance with the given closure.
    pub fn from_closure(f: impl Fn(f64) -> f64 + 'static) -> AnimationCurve {
        AnimationCurve::Closure(Box::new(f))
    }
}

/// A [Cubic Bezier] curve where P0 is (0, 0) and P3 is (1, 1)
///
/// This is normally used with [AnimationCurve], for eaxple:
///
/// ```
/// use druid_widget_nursery::animation::AnimationCurve;
///
/// let curve = AnimationCurve::cubic(0.68, -0.6, 0.32, 1.6);
///
/// ```
///
/// [Cubic Bezier]: https://en.wikipedia.org/wiki/B%C3%A9zier_curve
pub struct CubicBezierAnimationCurve {
    /// X coordinate of P1
    pub x1: f64,
    /// Y coordinate of P1
    pub y1: f64,
    /// X coordinate of P2
    pub x2: f64,
    /// Y coordinate of P2
    pub y2: f64,
}

impl CubicBezierAnimationCurve {

    fn evaluate_cubic(a: f64, b: f64, m: f64) -> f64 {
        3.0 * a * (1.0 - m) * (1.0 - m) * m +
        3.0 * b * (1.0 - m) *             m * m +
                                          m * m * m
    }

    /// Returns the value of the curve at point `t`.
    pub fn translate(&self, t: f64) -> f64 {
        let mut start = 0.0;
        let mut end = 1.0;

        const CUBIC_ERROR_BOUND: f64 = 0.001;

        loop {
            let midpoint = (start + end) / 2.0;
            let estimate = Self::evaluate_cubic(self.x1, self.x2, midpoint);
            if (t - estimate).abs() < CUBIC_ERROR_BOUND {
                return Self::evaluate_cubic(self.y1, self.y2, midpoint);
            }
            if estimate < t {
                start = midpoint;
            } else {
                end = midpoint;
            }
        }
    }
}

fn bounce(t: f64) -> f64 {
    if t < (1. / 2.75) {
        7.5625 * t * t
    } else if t < (2. / 2.75) {
        let t = t - (1.5 / 2.75);
        7.5625 * t * t + 0.75
    } else if t < (2.5 / 2.75) {
        let t = t - (2.25 / 2.75);
        7.5625 * t * t + 0.9375
    } else {
        let t = t - (2.625 / 2.75);
        7.5625 * t * t + 0.984375
    }
}

fn ease_in(t: f64) -> f64 {
    t * t
}

fn ease_in_expo(t: f64) -> f64 {
    2.0f64.powf(10.0 * (t - 1.0))
}

fn ease_out_elastic(t: f64) -> f64 {
    let p = 0.4;
    let s = p / 4.0;

    if t < 0.001 {
        0.
    } else if t > 0.999 {
        1.
    } else {
        2.0f64.powf(-10.0 * t) * ((t - s) * (2.0 * PI) / p).sin() + 1.0
    }
}

fn flip_curve(f: fn(f64) -> f64, t: f64) -> f64 {
    1.0 - f(1.0 - t)
}

fn combine_in_out(f: fn(f64) -> f64, t: f64) -> f64 {
    if t < 0.5 {
        0.5 * f(t * 2.0)
    } else {
        0.5 * flip_curve(f, t * 2.0 - 1.0) + 0.5
    }
}

fn combine_in_out_rev(f: fn(f64) -> f64, t: f64) -> f64 {
    if t < 0.5 {
        0.5 * flip_curve(f, t * 2.0)
    } else {
        0.5 * f(t * 2.0 - 1.0) + 0.5
    }
}
