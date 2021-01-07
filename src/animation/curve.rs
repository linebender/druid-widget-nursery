use std::fmt;
use std::fmt::{Debug, Formatter};

use druid::Data;

/// A custom animation curve mapping from time in the range 0..1 to progress in a range "around" 0..1.
/// It is permissible to undershoot and overshoot.
pub enum CustomCurve {
    Function(fn(f64) -> f64),
    Closure(Box<dyn FnMut(f64) -> f64>),
}

impl CustomCurve {
    fn translate(&mut self, t: f64) -> f64 {
        match self {
            CustomCurve::Function(f) => f(t),
            CustomCurve::Closure(f) => f(t),
        }
    }
}

impl Debug for CustomCurve {
    // Required as closures are not Debug
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CustomCurve::Function(f) => formatter
                .debug_struct("CustomAnimationCurve::Function")
                .field("f", f)
                .finish(),
            CustomCurve::Closure(_) => formatter
                .debug_struct("CustomAnimationCurve::Closure")
                .finish(),
        }
    }
}

impl From<fn(f64) -> f64> for AnimationCurve {
    fn from(f: fn(f64) -> f64) -> Self {
        AnimationCurve::Custom(CustomCurve::Function(f))
    }
}

impl From<SimpleCurve> for AnimationCurve {
    fn from(s: SimpleCurve) -> Self {
        AnimationCurve::Simple(s)
    }
}

#[derive(Data, Copy, Clone, Debug, Eq, PartialEq)]
/// A simple built in animation curve
pub enum SimpleCurve {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    OutElastic,
    OutBounce,
    OutSine,
}

impl SimpleCurve {
    fn translate(&mut self, t: f64) -> f64 {
        use std::f64::consts::PI;
        match self {
            Self::Linear => t,
            Self::EaseIn => t * t,
            Self::EaseOut => t * (2.0 - t),
            Self::EaseInOut => {
                let t = t * 2.0;
                if t < 1. {
                    0.5 * t * t
                } else {
                    let t = t - 1.;
                    -0.5 * (t * (t - 2.) - 1.)
                }
            }
            Self::OutElastic => {
                let p = 0.3;
                let s = p / 4.0;

                if t < 0.001 {
                    0.
                } else if t > 0.999 {
                    1.
                } else {
                    2.0f64.powf(-10.0 * t) * ((t - s) * (2.0 * PI) / p).sin() + 1.0
                }
            }
            Self::OutSine => (t * PI * 0.5).sin(),
            Self::OutBounce => {
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
        }
    }
}

#[derive(Debug)]
/// An animation curve, mapping from time in the range 0..1 to progress in a range "around" 0..1.
/// It is permissible for progress to undershoot and overshoot.
pub enum AnimationCurve {
    Simple(SimpleCurve),
    //    CubicBezier(CubicBezierAnimationCurve),
    //    Spring(SpringAnimationCurve),
    Custom(CustomCurve),
}

impl Default for AnimationCurve {
    fn default() -> Self {
        AnimationCurve::Simple(SimpleCurve::Linear)
    }
}

impl AnimationCurve {
    pub fn translate(&mut self, t: f64) -> f64 {
        match self {
            Self::Simple(s) => s.translate(t),
            Self::Custom(c) => c.translate(t),
        }
    }

    pub fn from_closure(f: impl FnMut(f64) -> f64 + 'static) -> AnimationCurve {
        AnimationCurve::Custom(CustomCurve::Closure(Box::new(f)))
    }
}
