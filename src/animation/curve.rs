use std::fmt;
use std::fmt::{Debug, Formatter};
use std::f64::consts::PI;

impl From<fn(f64) -> f64> for AnimationCurve {
    fn from(f: fn(f64) -> f64) -> Self {
        AnimationCurve::Function(f)
    }
}

/// An animation curve, mapping from time in the range 0..1 to progress in a range "around" 0..1.
/// It is permissible for progress to undershoot and overshoot.
pub enum AnimationCurve {
    Function(fn(f64) -> f64),
    Closure(Box<dyn FnMut(f64) -> f64>),
    CubicBezier(CubicBezierAnimationCurve),
    //    Spring(SpringAnimationCurve),
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
                .field("a", &b.a)
                .field("b", &b.b)
                .field("c", &b.c)
                .field("d", &b.d)
                .finish(),
        }
    }
}

impl AnimationCurve {

    pub const LINEAR: Self = Self::Function(|t| t);

    pub const EASE_IN: Self = Self::Function(|t| t * t);

    pub const EASE_OUT: Self = Self::Function(|t| t * (2.0 - t));

    pub const EASE_IN_OUT: Self = Self::Function(|t| {
        let t = t * 2.0;
        if t < 1. {
            0.5 * t * t
        } else {
            let t = t - 1.;
            -0.5 * (t * (t - 2.) - 1.)
        }
    });

    pub const EASE_OUT_ELASTIC: Self = Self::Function(|t| {
        let p = 0.3;
        let s = p / 4.0;

        if t < 0.001 {
            0.
        } else if t > 0.999 {
            1.
        } else {
            2.0f64.powf(-10.0 * t) * ((t - s) * (2.0 * PI) / p).sin() + 1.0
        }
    });

    pub const EASE_OUT_SINE: Self = Self::Function(|t| (t * PI * 0.5).sin());

    pub const EASE_IN_BACK: Self = Self::cubic(0.36, 0.0, 0.66, -0.56);
    pub const EASE_OUT_BACK: Self = Self::cubic(0.34, 1.56, 0.64, 1.0);
    pub const EASE_IN_OUT_BACK: Self = Self::cubic(0.68, -0.6, 0.32, 1.6);

    pub const BOUNCE_OUT: Self = Self::Function(|t| bounce(t));

    pub const fn cubic(a: f64, b: f64, c: f64, d: f64) -> Self {
        Self::CubicBezier(CubicBezierAnimationCurve { a, b, c, d })
    }

    pub fn translate(&mut self, t: f64) -> f64 {
        match self {
            Self::Function(f) => f(t),
            Self::Closure(c) => c(t),
            Self::CubicBezier(b) => b.translate(t),
        }
    }

    pub fn from_closure(f: impl FnMut(f64) -> f64 + 'static) -> AnimationCurve {
        AnimationCurve::Closure(Box::new(f))
    }
}

pub struct CubicBezierAnimationCurve {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
}

impl CubicBezierAnimationCurve {

    fn evaluate_cubic(a: f64, b: f64, m: f64) -> f64 {
        3.0 * a * (1.0 - m) * (1.0 - m) * m +
        3.0 * b * (1.0 - m) *             m * m +
                                          m * m * m
    }

    pub fn translate(&self, t: f64) -> f64 {
        let mut start = 0.0;
        let mut end = 1.0;

        const CUBIC_ERROR_BOUND: f64 = 0.001;

        loop {
            let midpoint = (start + end) / 2.0;
            let estimate = Self::evaluate_cubic(self.a, self.c, midpoint);
            if (t - estimate).abs() < CUBIC_ERROR_BOUND {
                return Self::evaluate_cubic(self.b, self.d, midpoint);
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
