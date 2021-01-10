// Copyright 2020 The Druid Authors.
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

//! A Seperator widget.

use druid::widget::prelude::*;
use druid::{kurbo::Line, piet::StrokeStyle};
use druid::{theme, Color, KeyOrValue};

/// A seperator widget.
pub struct Seperator {
    size: KeyOrValue<f64>,
    color: KeyOrValue<Color>,
    orientation: Orientation,
    stroke_style: StrokeStyle,
}

pub enum Orientation {
    Vertical,
    Horizontal,
}

impl Default for Seperator {
    fn default() -> Self {
        Seperator {
            size: theme::BUTTON_BORDER_WIDTH.into(),
            color: theme::BORDER_LIGHT.into(),
            orientation: Orientation::Horizontal,
            stroke_style: StrokeStyle::new(),
        }
    }
}

impl Seperator {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_size(mut self, size: impl Into<KeyOrValue<f64>>) -> Self {
        self.size = size.into();
        self
    }

    pub fn set_size(&mut self, size: impl Into<KeyOrValue<f64>>) {
        self.size = size.into();
    }

    pub fn with_color(mut self, color: impl Into<KeyOrValue<Color>>) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_stroke_style(mut self, stroke_style: StrokeStyle) -> Self {
        self.stroke_style = stroke_style;
        self
    }

    pub fn set_color(&mut self, color: impl Into<KeyOrValue<Color>>) {
        self.color = color.into();
    }

    pub fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
}

impl<T> Widget<T> for Seperator {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut T, _env: &Env) {}

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &T, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &T, _data: &T, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &T, env: &Env) -> Size {
        let size = self.size.resolve(env);
        let size = match self.orientation {
            Orientation::Vertical => (size, f64::INFINITY),
            Orientation::Horizontal => (f64::INFINITY, size),
        };
        bc.constrain(size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &T, env: &Env) {
        let line = Line::new((0., 0.), ctx.size().to_vec2().to_point());

        let color = self.color.resolve(env);
        let size = self.size.resolve(env);
        ctx.stroke_styled(line, &color, size, &self.stroke_style);
    }
}
