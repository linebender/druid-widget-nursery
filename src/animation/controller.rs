// Copyright 2022 The Druid Authors.
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

use super::{AnimationStatus, AnimationDirection};

use crate::RequestCtx;

/// Control animations. An Animation controller produces values between 0.0 and 1.0 during
/// the given duration. You can run this animation forward, backwards,
/// altered and repeated.
pub struct AnimationController {
    duration: f64,
    direction: AnimationDirection,
    repeat_limit: Option<usize>,
    layout: bool,

    status: AnimationStatus,
    since_start: f64,

    fraction: f64,
}

impl Default for AnimationController {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationController {

    /// Create a new forward animation with duration of one second.
    pub fn new() -> Self {
        Self {
            duration: 1.0,
            direction: AnimationDirection::Forward,
            repeat_limit: Some(1),
            layout: false,

            status: AnimationStatus::NotRunning,
            since_start: 0.0,
            fraction: 0.0,
        }
    }

    /// Builder-style method for specifying the repeat limit.
    ///
    /// For the non-builder varient, see [`set_repeat_limit`].
    ///
    /// [`set_repeat_limit`]: #method.set_repeat_limit
    pub fn repeat_limit(mut self, limit: Option<usize>) -> Self {
        self.set_repeat_limit(limit);
        self
    }

    /// Set the repeat limit.
    pub fn set_repeat_limit(&mut self, limit: Option<usize>) {
        self.repeat_limit = limit;
    }

    /// Builder-style method for specifying the [`AnimationDirection`].
    ///
    /// For the non-builder varient, see [`set_direction`].
    ///
    /// [`set_direction`]: #method.set_direction
     pub fn direction(mut self, direction: AnimationDirection) -> Self {
        self.set_direction(direction);
        self
    }

    /// Set the [`AnimationDirection`].
    pub fn set_direction(&mut self, direction: AnimationDirection) {
        self.direction = direction;
        self.reset()
    }

    /// Builder-style method for specifying the layout flag.
    ///
    /// For the non-builder varient, see [`set_layout`].
    ///
    /// [`set_layout`]: #method.set_layout
    pub fn layout(mut self, layout: bool) -> Self {
        self.set_layout(layout);
        self
    }

    /// Request widget layout after each update (instead of a paint request).
    pub fn set_layout(&mut self, layout: bool) {
        self.layout = layout;
    }

    /// Builder-style method for specifying the duration.
    ///
    /// For the non-builder varient, see [`set_duration`].
    ///
    /// [`set_duration`]: #method.set_duration
    pub fn duration(mut self, duration: f64) -> Self {
        self.set_duration(duration);
        self
    }

    /// Set the animation duration in seconds.
    pub fn set_duration(&mut self, duration: f64) {
        self.duration = duration;
        self.reset()
    }

    /// Get the current animation value (between 0.0 and 1.0).
    pub fn fraction(&self) -> f64 {
        self.fraction
    }


    /// Get the current [`AnimationStatus`].
    pub fn status(&self) -> AnimationStatus {
        self.status
    }

    /// Returns true if the animation is running.
    pub fn animating(&self) -> bool {
        use AnimationStatus::*;
        match &self.status {
             NotRunning | Retiring => false,
             Enlisting | Running | Repeating => true,
         }
    }

    /// Reset the controller.
    pub fn reset(&mut self) {
        use AnimationDirection::*;

        self.since_start = 0.0;
        self.status = AnimationStatus::NotRunning;

         match self.direction {
             Forward => self.fraction = 0.0,
             Reverse => self.fraction = 1.0,
             Alternate => self.fraction = 0.0,
             AlternateReverse => self.fraction = 1.0,
         }
    }

    /// Start the animation.
    pub fn start(&mut self, ctx: &mut impl RequestCtx) {
        self.since_start = 0.0;
        self.fraction = 0.0;

        self.status = AnimationStatus::Enlisting;
        self.update(ctx, 0);
    }

    /// Update animation state.
    ///
    /// This method should always be called in
    /// [`Event::AnimFrame`](druid::Event::AnimFrame). It updates the
    /// value according to the past period (`nanos` is added to that
    /// period first). If the transition's end isn't reached an
    /// additional animation-frame is requested.
    ///
    /// Note: This must be called to drive the animation.
    pub fn update(&mut self, ctx: &mut impl RequestCtx, nanos: u64) {
        use AnimationStatus::*;
        match &self.status {
            NotRunning | Retiring => {
                // do nothing
            }
            Enlisting | Running | Repeating => {

                self.since_start += (nanos as f64) * 0.000000001;

                if self.duration <= 0.0 {
                    let end_fraction = self.direction.end_fraction(true);
                    self.fraction = end_fraction;
                    self.status = Retiring;
                } else {
                    let factor = self.since_start / self.duration;
                    let fraction = factor.fract();
                    let repeat_count = factor as usize;
                    let even_repeat = repeat_count % 2 == 0;

                    let allow_repeat = self.repeat_limit
                        .map_or(true, |limit| repeat_count < limit);

                    if allow_repeat {
                        self.fraction = self.direction.translate(fraction, even_repeat);
                        self.status = Running;
                        ctx.request_anim_frame();
                    } else {
                        let end_fraction = self.direction.end_fraction(!even_repeat);
                        self.fraction = end_fraction;
                        self.status = Retiring;
                    }
                }

                if self.layout {
                    ctx.request_layout();
                } else {
                    ctx.request_paint();
                }
            }
        }
    }
}
