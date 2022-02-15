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

use druid::theme;
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx,
    PaintCtx, Point, Size, UpdateCtx, Widget, WidgetExt, WidgetPod,
};
use druid::widget::{Align, BackgroundBrush, Flex, Label, LabelText, Spinner};


/// A widget that conditionally masks the child content and displays
/// other content instead (the mask).
pub struct Mask<T> {
    child: WidgetPod<T, Box<dyn Widget<T>>>,
    mask: WidgetPod<T, Box<dyn Widget<T>>>,
    show_mask_cb: Option<Box<dyn Fn(&T, &Env) -> bool>>,
    show_mask: bool,
}

impl <T: Data> Mask<T> {

    /// Create a new instance with a child.
    ///
    /// The Mask widget simply shows the `child` unless the
    /// `show_mask` flag is set. In this case the mask widget is
    /// displayed and the input to the `child` is supressed.
    ///
    /// The default mask widget is a simple [Spinner] displayed in the
    /// center.
    pub fn new(child: impl Widget<T> + 'static) -> Self {
        let mask = Align::centered(Spinner::new().fix_height(24.));
        Self {
            child: WidgetPod::new(child.boxed()),
            mask: WidgetPod::new(mask.boxed()),
            show_mask_cb: None,
            show_mask: false,
        }
    }

    /// Builder-style method for setting the `show_mask` flag.
    pub fn show_mask(mut self, show_mask: bool) -> Self {
        self.set_show_mask(show_mask);
        self
    }

    /// Set the `show_mask` flag.
    pub fn set_show_mask(&mut self, show_mask: bool) {
        self.show_mask_cb = None;
        self.show_mask = show_mask;
    }

    /// Builder-style method to dynamically compute the `show_mask`
    /// flag using a closure.
    pub fn dynamic(mut self, show_mask_cb: impl  Fn(&T, &Env) -> bool + 'static) -> Self {
        self.show_mask_cb = Some(Box::new(show_mask_cb));
        self
    }

    /// Builder-style method for setting the mask widget.
    pub fn with_mask(mut self, mask: impl Widget<T> + 'static) -> Self {
        self.set_mask(mask);
        self
    }

    /// Set the mask widget.
    pub fn set_mask(&mut self, mask: impl Widget<T> + 'static) {
        self.mask = WidgetPod::new(mask.boxed());
    }

    /// Builder-style method to create a mask with a spinner and a text.
    pub fn with_text_mask(mut self, text: impl Into<LabelText<T>>) -> Self {
        self.set_text_mask(text);
        self
    }

    /// Create a mask with a spinner and a text.
    pub fn set_text_mask(&mut self, text: impl Into<LabelText<T>>) {
        let mask = Flex::row()
            .with_child(Spinner::new())
            .with_spacer(5.0)
            .with_child(Label::new(text))
            .center();

        self.set_mask(mask);
    }
}

impl<T: Data> Widget<T> for Mask<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if self.show_mask {
            self.mask.event(ctx, event, data, env);
        } else {
            self.child.event(ctx, event, data, env);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            if let Some(show_mask_cb) = &self.show_mask_cb {
                self.show_mask = (show_mask_cb)(data, env);
            }
        }
        self.child.lifecycle(ctx, event, data, env);
        self.mask.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        if let Some(show_mask_cb) = &self.show_mask_cb {
            let new_show_mask = (show_mask_cb)(data, env);
            if new_show_mask != self.show_mask {
                ctx.request_paint();
            }
            self.show_mask = new_show_mask;
        }
        self.child.update(ctx, data, env);
        self.mask.update(ctx, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> druid::Size {
        let size = self.child.layout(ctx, bc, data, env);
        let mask_bc = BoxConstraints::new(Size::ZERO, size);
        let _mask_size = self.mask.layout(ctx, &mask_bc, data, env);
        let origin = Point::new(0f64, 0f64);
        self.child.set_origin(ctx, data, env, origin);
        self.mask.set_origin(ctx, data, env, origin);

        let baseline_offset = self.child.baseline_offset();
        if baseline_offset > 0f64 {
            ctx.set_baseline_offset(baseline_offset);
        }

        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.child.paint(ctx, data, env);

        if self.show_mask {
            let bg_color = env.get(theme::WINDOW_BACKGROUND_COLOR).with_alpha(0.5);
            let mut brush = BackgroundBrush::Color(bg_color);

            brush.paint(ctx, data, env);

            self.mask.paint(ctx, data, env);
        }
    }
}
