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

use crate::RequestCtx;
use super::{AnimationController, AnimationCurve};

/// Apply an [AnimationCurve] to values from an [AnimationController]
pub struct CurvedAnimation {
    curve: AnimationCurve,
    controller: AnimationController,
}

impl Default for CurvedAnimation {
    fn default() -> Self {
        Self {
            curve: AnimationCurve::EASE_IN_OUT,
            controller: AnimationController::default(),
        }
    }
}

impl CurvedAnimation {

    /// Creates a new instance using the provided `curve` and `controller`.
    pub fn new(curve: impl Into<AnimationCurve>, controller: AnimationController) -> Self {
        Self { curve: curve.into(), controller }
    }

    /// Get the current animation value.
    ///
    /// Depenmding on the curve, the result can overshoot or
    /// undershoot the default range of 0.0 to 1.0.
    pub fn progress(&mut self) -> f64 {
        self.curve.translate(self.controller.fraction())
    }

    /// Provides access to the underlying [AnimationController].
    ///
    /// This gives full access to the underlying controller. While
    /// this struct provides wrappers for [`start`] and [`update`],
    /// all other methods need to be called on the controller.
    pub fn controller(&mut self) -> &AnimationController {
        &mut self.controller
    }

    /// Start the animation.
    ///
    /// Wrapper for [AnimationController::start]
    pub fn start(&mut self, ctx: &mut impl RequestCtx) {
        self.controller.start(ctx);
    }

    /// Update animation state.
    ///
    /// Wrapper for [AnimationController::update]
    pub fn update(&mut self, ctx: &mut impl RequestCtx, nanos: u64) {
        self.controller.update(ctx, nanos)
    }
}
