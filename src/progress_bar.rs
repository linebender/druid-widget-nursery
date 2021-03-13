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
//! allowing them to be overwritten using Option<> values.
//!
//! TODO:
//! More dynamic sizing
//! Allow the widget some internal text (just text or an arbitrary widget)
//! Paint entire bar and truncate so the gradients don't shrink to fit it.

use druid::piet::PaintBrush;
use druid::widget::prelude::*;
use druid::{theme, Color, LinearGradient, Point, Rect, UnitPoint};
use tracing::instrument;

/// A progress bar, displaying a numeric progress value.
///
/// This type impls `Widget<f64>`, expecting a float in the range `0.0..1.0`.
#[derive(Debug, Clone, Default)]
pub struct ProgressBar {
    bar_brush: Option<druid::piet::PaintBrush>,
    background_brush: Option<druid::piet::PaintBrush>,
    corner_radius: Option<f64>,
    border_colour: Option<druid::Color>,
    border_width: Option<f64>,
}

impl ProgressBar {
    /// Return a new `ProgressBar`.
    pub fn new() -> ProgressBar {
        Self::default()
    }

    pub fn with_bar_brush(mut self, cl: druid::piet::PaintBrush) -> Self {
        self.bar_brush = Some(cl);
        self
    }
    pub fn with_back_brush(mut self, cl: druid::piet::PaintBrush) -> Self {
        self.background_brush = Some(cl);
        self
    }
    pub fn with_corner_radius(mut self, c_rad: f64) -> Self {
        self.corner_radius = Some(c_rad);
        self
    }
    pub fn with_border_width(mut self, c_rad: f64) -> Self {
        self.border_width = Some(c_rad);
        self
    }
    pub fn with_border_colour(mut self, cl: druid::Color) -> Self {
        self.border_colour = Some(cl);
        self
    }

    //Internal getters that resolve using theme or control values.
    fn bar_brush(&self, env: &Env) -> druid::piet::PaintBrush {
        self.bar_brush
            .clone()
            .unwrap_or(PaintBrush::Linear(LinearGradient::new(
                UnitPoint::TOP,
                UnitPoint::BOTTOM,
                (env.get(theme::PRIMARY_LIGHT), env.get(theme::PRIMARY_DARK)),
            )))
    }
    fn background_brush(&self, env: &Env) -> druid::piet::PaintBrush {
        self.background_brush
            .clone()
            .unwrap_or(PaintBrush::Linear(LinearGradient::new(
                UnitPoint::TOP,
                UnitPoint::BOTTOM,
                (
                    env.get(theme::BACKGROUND_LIGHT),
                    env.get(theme::BACKGROUND_DARK),
                ),
            )))
    }
    fn corner_radius(&self, env: &Env) -> f64 {
        self.corner_radius
            .clone()
            .unwrap_or(env.get(theme::PROGRESS_BAR_RADIUS))
    }
    //TODO: This should probably be a theme setting like everything else. No good option yet.
    fn border_width(&self, _env: &Env) -> f64 {
        self.border_width.clone().unwrap_or(2.0)
    }
    fn border_colour(&self, env: &Env) -> Color {
        self.border_colour
            .clone()
            .unwrap_or(env.get(theme::BORDER_DARK))
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
        bc.constrain(Size::new(
            bc.max().width,
            env.get(theme::BASIC_WIDGET_HEIGHT),
        ))
    }

    #[instrument(name = "ProgressBar", level = "trace", skip(self, ctx, data, env))]
    fn paint(&mut self, ctx: &mut PaintCtx, data: &f64, env: &Env) {
        let border_width = self.border_width.clone().unwrap_or(2.0);

        let height = env.get(theme::BASIC_WIDGET_HEIGHT);
        let clamped = data.max(0.0).min(1.0);
        let inset = -border_width / 2.0;
        let size = ctx.size();
        let rounded_rect = Size::new(size.width, height)
            .to_rect()
            .inset(inset)
            .to_rounded_rect(self.corner_radius(env));

        // Paint the border
        ctx.stroke(rounded_rect, &self.border_colour(env), border_width);

        // Paint the background
        // This has been changed from a gradient from top to bottom because I thought this made more sense visually.
        //TODO: Perhaps we just want a transparent background?
        ctx.fill(rounded_rect, &self.background_brush(env));

        // Paint the bar
        let calculated_bar_width = clamped * rounded_rect.width();

        let rounded_rect = Rect::from_origin_size(
            Point::new(-inset, 0.),
            Size::new(calculated_bar_width, height),
        )
        .inset((0.0, inset))
        .to_rounded_rect(self.corner_radius(env));

        ctx.fill(rounded_rect, &self.bar_brush(env));
    }
}
