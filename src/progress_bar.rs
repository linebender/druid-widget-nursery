// Copyright 2021 The Druid Authors.
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

//! A modified progress bar widget demo. The goal here is to put more
//! configuration on the control, using theme colours as defaults and
//! allowing them to be overwritten using Option<T> and KeyOrValue<T> values.
//!
//! Building on the existing progress bar in druid.
//! Making styling options more configurable, using theme values as defaults, with options in the widget to override.
//! Removed constraint on widget width, the widget will now expand to fit its container.
//! Future Idea: Add optional configuration for text that would go over the progress bar.
//! TODO: Should width and height be completely configurable, both sized to expand into their container?
//! TODO: review theme values more generally, concerned that they might not be getting used consistently.
//! TODO: Use druid::BackgroundBrush instead of druid::piet::PaintBrush, but it ruins all my derives.

use druid::kurbo::RoundedRectRadii;
use druid::piet::PaintBrush;
use druid::widget::prelude::*;
// use druid::widget::BackgroundBrush;
use druid::{theme, Color, KeyOrValue, LinearGradient, Point, Rect, UnitPoint};
use tracing::instrument;

/// A progress bar, displaying a numeric progress value.
///
/// This type impls `Widget<f64>`, expecting a float in the range `0.0..1.0`.
#[derive(Debug, Clone)]
pub struct ProgressBar {
    bar_brush: Option<PaintBrush>,
    background_brush: Option<PaintBrush>,
    corner_radius: KeyOrValue<RoundedRectRadii>,
    border_colour: KeyOrValue<Color>,
    border_width: KeyOrValue<f64>,
}

impl ProgressBar {
    /// Return a new `ProgressBar`.
    pub fn new() -> ProgressBar {
        Self::default()
    }

    //'with' functions returning self.
    pub fn with_bar_brush(mut self, cl: PaintBrush) -> Self {
        self.bar_brush = Some(cl);
        self
    }
    pub fn with_back_brush(mut self, cl: PaintBrush) -> Self {
        self.background_brush = Some(cl);
        self
    }
    pub fn with_corner_radius(mut self, c_rad: f64) -> Self {
        self.corner_radius = KeyOrValue::Concrete(RoundedRectRadii::from(c_rad));
        self
    }
    pub fn with_border_width(mut self, c_rad: f64) -> Self {
        self.border_width = KeyOrValue::Concrete(c_rad);
        self
    }
    pub fn with_border_colour(mut self, cl: Color) -> Self {
        self.border_colour = KeyOrValue::Concrete(cl);
        self
    }
    //Set functions, returning
    pub fn set_bar_brush(mut self, cl: PaintBrush) {
        self.bar_brush = Some(cl);
    }
    pub fn set_back_brush(mut self, cl: PaintBrush) {
        self.background_brush = Some(cl);
    }
    pub fn set_corner_radius(mut self, c_rad: f64) {
        self.corner_radius = KeyOrValue::Concrete(RoundedRectRadii::from(c_rad));
    }
    pub fn set_border_width(mut self, c_rad: f64) {
        self.border_width = KeyOrValue::Concrete(c_rad);
    }
    pub fn set_border_colour(mut self, cl: Color) {
        self.border_colour = KeyOrValue::Concrete(cl);
    }

    //Internal getters that resolve using theme or control values.
    fn bar_brush(&self, env: &Env) -> PaintBrush {
        self.bar_brush.clone().unwrap_or_else(|| {
            PaintBrush::Linear(LinearGradient::new(
                UnitPoint::TOP,
                UnitPoint::BOTTOM,
                (env.get(theme::PRIMARY_LIGHT), env.get(theme::PRIMARY_DARK)),
            ))
        })
    }
    fn background_brush(&self, env: &Env) -> PaintBrush {
        self.background_brush.clone().unwrap_or_else(|| {
            PaintBrush::Linear(LinearGradient::new(
                UnitPoint::TOP,
                UnitPoint::BOTTOM,
                (
                    env.get(theme::BACKGROUND_LIGHT),
                    env.get(theme::BACKGROUND_DARK),
                ),
            ))
        })
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        ProgressBar {
            bar_brush: None,
            background_brush: None,
            corner_radius: KeyOrValue::Key(theme::PROGRESS_BAR_RADIUS),
            border_colour: KeyOrValue::Key(theme::BORDER_DARK),
            border_width: KeyOrValue::Key(theme::BUTTON_BORDER_WIDTH),
        }
    }
}

impl Widget<f64> for ProgressBar {
    #[instrument(
        name = "ProgressBar",
        level = "trace",
        skip(self, _ctx, _event, _data, _env)
    )]
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut f64, _env: &Env) {}

    #[instrument(
        name = "ProgressBar",
        level = "trace",
        skip(self, _ctx, _event, _data, _env)
    )]
    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &f64, _env: &Env) {}

    #[instrument(
        name = "ProgressBar",
        level = "trace",
        skip(self, ctx, _old_data, _data, _env)
    )]
    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &f64, _data: &f64, _env: &Env) {
        ctx.request_paint();
    }

    #[instrument(
        name = "ProgressBar",
        level = "trace",
        skip(self, _layout_ctx, bc, _data, env)
    )]
    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &f64,
        env: &Env,
    ) -> Size {
        bc.debug_check("ProgressBar");
        // bc.constrain(Size::new(
        //     bc.max().width,
        //     env.get(theme::BASIC_WIDGET_HEIGHT),
        // ))
        bc.constrain(Size::new(
            env.get(theme::WIDE_WIDGET_WIDTH),
            env.get(theme::BASIC_WIDGET_HEIGHT),
        ))
    }

    #[instrument(name = "ProgressBar", level = "trace", skip(self, ctx, data, env))]
    fn paint(&mut self, ctx: &mut PaintCtx, data: &f64, env: &Env) {
        let border_width = self.border_width.resolve(env);

        let height = env.get(theme::BASIC_WIDGET_HEIGHT);
        let inset = -border_width / 2.0;
        let size = ctx.size();
        let full_rect = Size::new(size.width, height)
            .to_rect()
            .inset(inset)
            .to_rounded_rect(self.corner_radius.resolve(env));

        // Paint the border
        ctx.stroke(full_rect, &self.border_colour.resolve(env), border_width);

        // Paint the background
        // This has been changed from a gradient from top to bottom because I thought this made more sense visually.
        ctx.fill(full_rect, &self.background_brush(env));

        // Paint the bar
        let calculated_bar_width = data.max(0.0).min(1.0) * full_rect.width();

        let bar_rect = Rect::from_origin_size(
            Point::new(-inset, 0.),
            Size::new(calculated_bar_width, height),
        )
        .inset((0.0, inset))
        .to_rounded_rect(self.corner_radius.resolve(env));

        //Old method wouldn't apply brush to the full bar.
        // ctx.fill(bar_rect, &self.bar_brush(env));

        //Renders full bar and clips.
        ctx.render_ctx
            .save()
            .expect("Could not save render context in, ProgressBar Widget.");
        ctx.render_ctx.clip(bar_rect);
        ctx.fill(full_rect, &self.bar_brush(env));
        ctx.render_ctx
            .restore()
            .expect("Could not restore render context in, ProgressBar Widget.");
    }
}
