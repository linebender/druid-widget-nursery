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

//! A widget that allows for arbitrary layout of it's children.
use druid::kurbo::Rect;
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, Size, UpdateCtx, Widget, WidgetPod,
};

///A container that allows for arbitrary layout.
///
///This widget allows you to lay widgets out at any point, and to allow that positioning to be dependent on the data.
///This is facilitated by the [`CanvasLayout`] trait, and will most typically be used by wrapping your desired widgets
///in a [`CanvasWrap`] wrapper.
///
///[`CanvasLayout`]: trait.CanvasLayout.html
///[`CanvasWrap`]: struct.CanvasWrap.html
pub struct Canvas<T: Data> {
    children: Vec<(Rect, Box<dyn CanvasLayout<T>>)>,
}

impl<T: Data> Default for Canvas<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Data> Canvas<T> {
    pub fn new() -> Self {
        Self { children: vec![] }
    }
    pub fn with_child(mut self, child: impl CanvasLayout<T> + 'static) -> Self {
        self.children.push((Rect::ZERO, Box::new(child)));
        self
    }

    pub fn add_child(&mut self, ctx: &mut EventCtx, child: impl CanvasLayout<T> + 'static) {
        self.children.push((Rect::ZERO, Box::new(child)));
        ctx.children_changed();
    }
}

impl<T: Data> Widget<T> for Canvas<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        //we're letting their own filtering handle event filtering
        //we may want to revisit that decision
        for (_, child) in &mut self.children {
            child.event(ctx, event, data, env);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        for (_, child) in &mut self.children {
            child.lifecycle(ctx, event, data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        for (_, child) in &mut self.children {
            child.update(ctx, old_data, data, env);
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        for (rect, child) in &mut self.children {
            let (origin, size) = child.canvas_layout(ctx, data, env);
            *rect = Rect::from_origin_size(origin, size);
        }

        //We always take the max size.
        let size = bc.max();
        if size.width.is_infinite() {
            log::warn!("Infinite width passed to Canvas");
        }
        if size.height.is_infinite() {
            log::warn!("Infinite height passed to Canvas");
        }
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        //TODO: filter painting based on our extents? (don't draw widgets entirely outside our bounds?)
        //It's the main reason we keep and update the rect
        for (_, child) in &mut self.children {
            child.paint(ctx, data, env);
        }
    }
}

pub struct CanvasWrap<W: Widget<T>, T: Data, F: Fn(&T) -> Point> {
    inner: WidgetPod<T, W>,
    closure: F,
}
impl<W: Widget<T>, T: Data, F: Fn(&T) -> Point> CanvasWrap<W, T, F> {
    pub fn new(widget: W, closure: F) -> Self {
        Self {
            inner: WidgetPod::new(widget),
            closure,
        }
    }
}

impl<W: Widget<T>, T: Data, F: Fn(&T) -> Point> CanvasLayout<T> for CanvasWrap<W, T, F> {
    fn canvas_layout(&mut self, ctx: &mut LayoutCtx, data: &T, env: &Env) -> (Point, Size) {
        let desired_origin = (self.closure)(data);
        let desired_size = self.inner.layout(
            ctx,
            &BoxConstraints::new(Size::ZERO, Size::new(f64::INFINITY, f64::INFINITY)),
            data,
            env,
        );
        println!("{} {}", desired_origin, desired_size);
        self.inner.set_layout_rect(
            ctx,
            data,
            env,
            Rect::from_origin_size(desired_origin, desired_size),
        );
        (desired_origin, desired_size)
    }
}

impl<W: Widget<T>, T: Data, F: Fn(&T) -> Point> Widget<T> for CanvasWrap<W, T, F> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        self.inner.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.inner.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        self.inner.update(ctx, data, env);
        if (self.closure)(data) != (self.closure)(old_data) {
            ctx.request_layout();
            //println!("Repaint requested");
        }
    }

    //NOTE: This is not called when we're being laid out on a canvas, so we act transparently.
    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        self.inner.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, paint_ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.inner.paint(paint_ctx, data, env);
    }
}

///
pub trait CanvasLayout<T: Data>: Widget<T> {
    fn canvas_layout(&mut self, ctx: &mut LayoutCtx, data: &T, env: &Env) -> (Point, Size);
}
