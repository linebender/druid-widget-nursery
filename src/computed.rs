// Copyright 2021 the Druid Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Widget to dynamically compute data.
//! It is like Label::dynamic but more general.
use druid::widget::prelude::*;
use druid::{Point, Widget, WidgetPod};

pub struct ComputedWidget<T, U> {
    child: WidgetPod<U, Box<dyn Widget<U>>>,
    data: Option<U>,
    computer: Box<dyn FnMut(&T) -> U>,
}

impl<T, U> ComputedWidget<T, U> {
    pub fn new(child: impl Widget<U> + 'static, computer: impl FnMut(&T) -> U + 'static) -> Self {
        Self {
            child: WidgetPod::new(Box::new(child)),
            data: None,
            computer: Box::new(computer),
        }
    }
}

impl<T, U: Data> Widget<T> for ComputedWidget<T, U> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut T, env: &Env) {
        let old_data = self.data.as_ref().unwrap();
        let mut data = old_data.clone();

        self.child.event(ctx, event, &mut data, env);

        if !data.same(old_data) {
            tracing::warn!("Computed data changed inside an event. Computed data cannot be changed and change will be ignored.");
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            self.data = Some((self.computer)(data));
        }
        self.child
            .lifecycle(ctx, event, self.data.as_ref().unwrap(), env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        self.data = Some((self.computer)(data));
        self.child.update(ctx, self.data.as_ref().unwrap(), env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &T, env: &Env) -> Size {
        let size = self.child.layout(ctx, bc, self.data.as_ref().unwrap(), env);
        self.child.set_origin(ctx, Point::ORIGIN);
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &T, env: &Env) {
        self.child.paint(ctx, self.data.as_ref().unwrap(), env);
    }
}
