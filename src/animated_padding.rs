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

//! A widget that just adds amimated padding during layout.
//
// Code is mostly copied from the druid Padding widget - just added
// the animation code.

use druid::debug_state::DebugState;
use druid::{Data, Env, Insets, Point, Widget, WidgetPod};
use druid::widget_wrapper_pod_body;
use druid::widget::{prelude::*, WidgetWrapper};

use tracing::{instrument, trace};

use crate::animation::{Animated, AnimationCurve};

/// A widget that just adds padding around its child.
pub struct AnimatedPadding<T, W> {
    insets: Animated<Insets>,
    padding_cb: Box<dyn Fn(&T, &Env) -> Insets>,
    child: WidgetPod<T, W>,
}

impl<T, W: Widget<T>> AnimatedPadding<T, W> {
    /// Create a new instance, where `padding_cb` is used to get the
    /// Insets dynamically.
    pub fn new<F>(child: W, padding_cb: F) -> Self
    where F: 'static + Fn(&T, &Env) -> Insets,
    {
        Self {
            insets: Animated::new(Insets::uniform(0.0))
                .curve(AnimationCurve::EASE_OUT)
                .layout(true),
            child: WidgetPod::new(child),
            padding_cb: Box::new(padding_cb),
        }
    }

    /// Builder-style method for specifying the [`AnimationCurve`].
    ///
    /// For the non-builder varient, see [`set_curve`].
    ///
    /// [`set_curve`]: #method.set_curve
    pub fn curve(mut self, curve: AnimationCurve) -> Self {
        self.set_curve(curve);
        self
    }

    /// Set the [`AnimationCurve`].
    pub fn set_curve(&mut self, curve: AnimationCurve) {
        self.insets.set_curve(curve);
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
        self.insets.set_duration(duration);
    }
}

impl<T, W> WidgetWrapper for AnimatedPadding<T, W> {
    widget_wrapper_pod_body!(W, child);
}

impl<T: Data, W: Widget<T>> Widget<T> for AnimatedPadding<T, W> {
    #[instrument(name = "AnimatedPadding", level = "trace", skip(self, ctx, event, data, env))]
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        self.child.event(ctx, event, data, env);
        if let Event::AnimFrame(nanos) = event {
            self.insets.update(ctx, *nanos);
        }
    }

    #[instrument(name = "AnimatedPadding", level = "trace", skip(self, ctx, event, data, env))]
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.child.lifecycle(ctx, event, data, env);
        if let LifeCycle::WidgetAdded = event {
            self.insets.jump_to_value((self.padding_cb)(data, env));
        }
    }

    #[instrument(name = "AnimatedPadding", level = "trace", skip(self, ctx, _old, data, env))]
    fn update(&mut self, ctx: &mut UpdateCtx, _old: &T, data: &T, env: &Env) {
        let new_insets = (self.padding_cb)(data, env);
        if new_insets != self.insets.end() {
            self.insets.animate(ctx, new_insets);
        }
        self.child.update(ctx, data, env);
    }

    #[instrument(name = "AnimatedPadding", level = "trace", skip(self, ctx, bc, data, env))]
    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        bc.debug_check("AnimatedPadding");
        let insets = self.insets.get();

        let hpad = insets.x0 + insets.x1;
        let vpad = insets.y0 + insets.y1;

        let child_bc = bc.shrink((hpad, vpad));
        let size = self.child.layout(ctx, &child_bc, data, env);
        let origin = Point::new(insets.x0, insets.y0);
        self.child.set_origin(ctx, data, env, origin);

        let my_size = Size::new(size.width + hpad, size.height + vpad);
        let my_insets = self.child.compute_parent_paint_insets(my_size);
        ctx.set_paint_insets(my_insets);
        let baseline_offset = self.child.baseline_offset();
        if baseline_offset > 0f64 {
            ctx.set_baseline_offset(baseline_offset + insets.y1);
        }
        trace!("Computed layout: size={}, insets={:?}", my_size, my_insets);
        my_size
    }

    #[instrument(name = "AnimatedPadding", level = "trace", skip(self, ctx, data, env))]
    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.child.paint(ctx, data, env);
    }

    fn debug_state(&self, data: &T) -> DebugState {
        DebugState {
            display_name: self.short_type_name().to_string(),
            children: vec![self.child.widget().debug_state(data)],
            ..Default::default()
        }
    }
}
