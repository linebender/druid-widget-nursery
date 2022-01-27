//! Druid animation library

mod animated_value;
mod animator;
mod context;
mod controller;
mod curve;
mod interpolate;
mod state;
mod storage;
#[cfg(test)]
mod test;

pub use animated_value::Animated;
pub use animator::Animator;
pub use context::AnimationCtx;
pub use controller::AnimationController;
pub use curve::{AnimationCurve, CubicBezierAnimationCurve};
pub use interpolate::Interpolate;
pub use storage::AnimationId;

use druid::Data;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::time::Duration;

use crate::animation::state::AnimationHandle;
use state::{AnimationState, AnimationStatusInternal};
use storage::AnimationStorage;

type Nanos = f64;
type DelayNanos = Nanos; // delay after ready
type StartNanos = Nanos; // start time
type Animations = AnimationStorage<AnimationState>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// The status of a currently running animation
pub enum AnimationStatus {
    NotRunning,
    Enlisting,
    Running,
    Repeating,
    Retiring,
}

/// Which direction should the animation run, and how should it repeat
#[derive(Debug, Data, Copy, Clone, PartialOrd, PartialEq)]
pub enum AnimationDirection {
    Forward,
    Reverse,
    Alternate,
    AlternateReverse,
}

impl Default for AnimationDirection {
    fn default() -> Self {
        Self::Forward
    }
}

impl AnimationDirection {
    fn translate(&self, frac: f64, even_repeat: bool) -> f64 {
        match self {
            Self::Forward => frac,
            Self::Reverse => 1.0 - frac,
            Self::Alternate => {
                if even_repeat {
                    frac
                } else {
                    1. - frac
                }
            }
            Self::AlternateReverse => {
                if !even_repeat {
                    frac
                } else {
                    1. - frac
                }
            }
        }
    }

    fn end_fraction(&self, even_repeat: bool) -> f64 {
        match self {
            Self::Forward => 1.,
            Self::Reverse => 0.,
            Self::Alternate => {
                if even_repeat {
                    1.
                } else {
                    0.
                }
            }
            Self::AlternateReverse => {
                if !even_repeat {
                    1.
                } else {
                    0.
                }
            }
        }
    }
}

/// The name of an animation event
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct AnimationEventName(pub &'static str);

/// An event in the animator.
/// This can be used as a trigger to set off other animations.
#[derive(Eq, PartialEq, Hash, Debug)]
pub enum AnimationEvent {
    /// A named event provided by the user.
    Named(AnimationEventName),
    /// An animation has finished.
    Ended(AnimationId),
}

impl From<AnimationEventName> for AnimationEvent {
    fn from(name: AnimationEventName) -> Self {
        AnimationEvent::Named(name)
    }
}

pub(in crate::animation) fn clamp_fraction(f: f64) -> f64 {
    // f.clamp is unstable
    f.max(0.).min(1.)
}
