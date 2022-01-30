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

use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx,
    PaintCtx, Point, Rect, RenderContext, Size, UpdateCtx, Widget, WidgetPod, UnitPoint,
};
use tracing::warn;

use druid::kurbo::Shape;
use crate::animation::{Animated, AnimationCurve, Interpolate};

/// Stack child position
///
/// Stack children are positioned relative to the container edges.
///
/// Horizontal position is determined by `(left, right,
/// width)`. Maximal two of those values may be defined (one value need
/// to be `None`).
///
/// Vertical position is determined by `(top, bottom,
/// height)`. Maximal two of those values may be defined (one value
/// need to be `None`).
///
/// If `width` or `height` is unconstrained, they are positioned
/// according to the [Stack::align] property.
#[derive(Clone, Debug, Default, PartialEq, Data)]
pub struct StackChildPosition {
    /// Disance from left edge.
    pub left: Option<f64>,
    /// Disance from right edge.
    pub right: Option<f64>,
    /// Disance from top edge.
    pub top: Option<f64>,
    /// Disance from bottom edge.
    pub bottom: Option<f64>,
    /// Widget width.
    pub width: Option<f64>,
    /// Widhet height.
    pub height: Option<f64>,
}

impl Interpolate for StackChildPosition {
    fn interpolate(&self, other: &Self, fraction: f64) -> Self {
        let lerp = |a: Option<f64>, b: Option<f64>, f: f64| -> Option<f64> {
            match (a, b) {
                (Some(a), Some(b)) => Some(a + (b -a)*f),
                (Some(a), None) => if fraction < 0.5 { Some(a) } else { None },
                (None, Some(b)) => if fraction < 0.5 { None } else { Some(b) },
                (None, None) => None,
            }
        };
        StackChildPosition {
            left: lerp(self.left, other.left, fraction),
            right: lerp(self.right, other.right, fraction),
            top: lerp(self.top, other.top, fraction),
            bottom: lerp(self.bottom, other.bottom, fraction),
            width: lerp(self.width, other.width, fraction),
            height: lerp(self.height, other.height, fraction),
        }
    }
}

impl StackChildPosition {

    /// Create a new instance, all values set to `None`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder-style method to set distance from left edge.
    pub fn left(mut self, value: Option<f64>) -> Self {
        self.left = value;
        self
    }

    /// Builder-style method to set distance from right edge.
    pub fn right(mut self, value: Option<f64>) -> Self {
        self.right = value;
        self
    }

    /// Builder-style method to set distance from top edge.
    pub fn top(mut self, value: Option<f64>) -> Self {
        self.top = value;
        self
    }

    /// Builder-style method to set distance from bottom edge.
    pub fn bottom(mut self, value: Option<f64>) -> Self {
        self.bottom = value;
        self
    }

    /// Builder-style method to set child width.
    pub fn width(mut self, value: Option<f64>) -> Self {
        self.width = value;
        self
    }

    /// Builder-style method to set child height.
    pub fn height(mut self, value: Option<f64>) -> Self {
        self.height = value;
        self
    }
}

type PositionCallback<T> = Box<dyn for<'a> Fn(&'a T, &Env) -> &'a StackChildPosition>;

enum Position<T> {
    None,
    Fixed(StackChildPosition),
    Dynamic(PositionCallback<T>),
}

/// Stack child configuration
///
/// This struct allows to configure additional aspects like the
/// [`StackChildPosition`] or animation attributes for dynamic
/// positioned children.
pub struct StackChildParams<T> {
    position: Position<T>,
    // We also store the animation state here - just to keep it simple
    animated_position: Animated<StackChildPosition>,
}

impl <T> From<StackChildPosition> for StackChildParams<T> {
    fn from(position: StackChildPosition) -> Self {
        StackChildParams::fixed(position)
    }
}

impl <T> StackChildParams<T> {

    // setup for non-positioned children
    fn new() -> Self {
        Self {
            position: Position::None,
            animated_position: Animated::jump(StackChildPosition::new()).layout(true)
        }
    }

    /// Create a *positioned* stack child
    pub fn fixed(position: StackChildPosition) -> Self {
        Self {
            position: Position::Fixed(position),
            animated_position: Animated::jump(StackChildPosition::new()).layout(true)
        }
    }

    /// Create a dynamically *positioned* stack child
    pub fn dynamic<F>(position: F) -> Self
    where F: 'static + for<'a> Fn(&'a T, &Env) -> &'a StackChildPosition
    {
        Self {
            position: Position::Dynamic(Box::new(position)),
            animated_position: Animated::new(StackChildPosition::new())
                .curve(AnimationCurve::EASE_OUT)
                .duration(0.3)
                .layout(true),
        }
    }

    /// Builder-style method for specifying the [`AnimationCurve`].
    ///
    /// For the non-builder varient, see [`set_curve`].
    ///
    /// [`set_curve`]: #method.set_curve
    pub fn curve(mut self, curve: AnimationCurve) -> Self {
        self.animated_position.set_curve(curve);
        self
    }

    /// Set the [`AnimationCurve`].
    ///
    /// The curve is used by dynamically positioned children to
    /// animate the position change.
    pub fn set_curve(&mut self, curve: AnimationCurve) {
        self.animated_position.set_curve(curve);
    }

    /// Builder-style method for specifying the animation duration.
    ///
    /// For the non-builder varient, see [`set_duration`].
    ///
    /// [`set_duration`]: #method.set_duration
    pub fn duration(mut self, duration: f64) -> Self {
        self.animated_position.set_duration(duration);
        self
    }

    /// Set the animation duration in seconds.
    ///
    /// The duration is used by dynamically positioned children to
    /// animate the position change.
    pub fn set_duration(&mut self, duration: f64) {
        self.animated_position.set_duration(duration);
    }
}

struct StackChild<T> {
    widget: WidgetPod<T, Box<dyn Widget<T>>>,
    params: StackChildParams<T>,
}

impl <T: Data> StackChild<T> {
    pub fn new(widget: impl Widget<T> + 'static, params: StackChildParams<T>) -> Self {
        Self {
            widget: WidgetPod::new(Box::new(widget)),
            params,
        }
    }
}

/// Stack of widgets
///
/// Stack provides an easy way to stack widgets on top of each
/// other. Children are positioned relative to the container edges.
///
/// Children are either *positioned* or *non-positioned*.
///
/// *Non-positioned* children are used to compute the size of the
/// Stack widget itself. They are aligned using the [Stack::align] setting.
///
/// *Positioned* children are layed-out after *non-positioned*
/// children. Their position is relative to the container edges (see
/// [`StackChildPosition`]).
pub struct Stack<T> {
    children: Vec<StackChild<T>>,
    align: UnitPoint,
    fit: bool,
    clip: bool,
}

impl <T: Data>  Default for Stack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl <T: Data> Stack<T> {

    /// Create a new Stack widget.
    ///
    /// Child alignment is set to [UnitPoint::TOP_LEFT], `fit` and
    /// `clip` flags are set to `false`.
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            align: UnitPoint::TOP_LEFT,
            fit: false,
            clip: false,
        }
    }

    /// Builder-style method for specifying the `fit` attribute.
    pub fn fit(mut self, fit: bool) -> Self {
        self.set_fit(fit);
        self
    }

    /// Set the `fit` attribute.
    ///
    /// Fit *non-positioned* children to the size of the container.
    pub fn set_fit(&mut self, fit: bool) {
        self.fit = fit;
    }

    /// Builder-style method for specifying the `clip` attribute.
    pub fn clip(mut self, clip: bool) -> Self {
        self.set_clip(clip);
        self
    }

    /// Set the `clip` attribute.
    ///
    /// Clip paint region at container boundaries.
    pub fn set_clip(&mut self, clip: bool) {
        self.clip = clip;
    }


    /// Builder-style method for specifying the default child alignment.
    pub fn align(mut self, align: UnitPoint) -> Self {
        self.set_align(align);
        self
    }

    /// Set the default child alignment.
    pub fn set_align(&mut self, align: UnitPoint) {
        self.align = align;
    }

    /// Builder-style variant of `add_child`.
    pub fn with_child(mut self, child: impl Widget<T> + 'static) -> Self {
        self.add_child(child);
        self
    }

    /// Add another stack child.
    pub fn add_child(&mut self, child: impl Widget<T> + 'static) {
        let child = StackChild::new(child, StackChildParams::new());
        self.children.push(child);
    }

    /// Builder-style variant of `add_positioned_child`.
    pub fn with_positioned_child(
        mut self,
        child: impl Widget<T> + 'static,
        params: impl Into<StackChildParams<T>>,
    ) -> Self {
        self.add_positioned_child(child, params);
        self
    }

    /// Add another *positioned* child.
    pub fn add_positioned_child(
        &mut self,
        child: impl Widget<T> + 'static,
        params: impl Into<StackChildParams<T>>,
    ) {
        let child = StackChild::new(child, params.into());
        self.children.push(child);
    }
}

impl<T: Data> Widget<T> for Stack<T> {
    fn event(&mut self, ctx: &mut EventCtx<'_, '_>, event: &Event, data: &mut T, env: &Env) {
        for child in self.children.iter_mut().rev() {
            if ctx.is_handled() {
                return;
            }

            let rect = child.widget.layout_rect();
            let pos_match = match event {
                Event::MouseMove(mouse_event) | Event::MouseDown(mouse_event) |
                Event::MouseUp(mouse_event) | Event::Wheel(mouse_event) => {
                    rect.winding(mouse_event.pos) != 0
                }
                _ => false,
            };

            child.widget.event(ctx, event, data, env);

            // only send to one widget (top widget)
            if pos_match { break; }
        }

        if let Event::AnimFrame(nanos) = event {
            for child in self.children.iter_mut() {
                if let Position::Dynamic(_) = &child.params.position {
                    child.params.animated_position.update(ctx, *nanos);
                }
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx<'_, '_>, event: &LifeCycle, data: &T, env: &Env) {
        for child in &mut self.children {
            child.widget.lifecycle(ctx, event, data, env);
            if let LifeCycle::WidgetAdded = event {
                if let Position::Dynamic(position_cb) = &child.params.position {
                    child.params.animated_position.jump_to_value(position_cb(data, env).clone());
                 }
            }
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx<'_, '_>, _old_data: &T, data: &T, env: &Env) {
        for child in &mut self.children {
            child.widget.update(ctx, data, env);
            // update position for dynamic children
            if let Position::Dynamic(position_cb) = &child.params.position {
                let new_position = position_cb(data, env);
                if new_position != &child.params.animated_position.end() {
                    child.params.animated_position.animate(ctx, new_position.clone());
                }
            };
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_, '_>, bc: &BoxConstraints, data: &T, env: &Env) -> druid::Size {

        let child_bc = if self.fit { BoxConstraints::tight(bc.max()) } else { bc.loosen() };

        // Compute size for non-positioned children
        let mut stack_width = 0f64;
        let mut stack_height = 0f64;
        for child in &mut self.children {
            if !matches!(child.params.position, Position::None) { continue; }
            let child_size = child.widget.layout(ctx, &child_bc, data, env);
            stack_width = stack_width.max(child_size.width);
            stack_height = stack_height.max(child_size.height);
            child.widget.set_origin(ctx, data, env, Point::ORIGIN);
        }

        let size = Size::new(stack_width, stack_height);

        // Compute size for positioned children
        for child in &mut self.children {
            let animated_position = child.params.animated_position.get();
            let position = match &child.params.position {
                Position::None => continue,
                Position::Fixed(position) => position,
                Position::Dynamic(_) => &animated_position,
            };

            let mut min_width = 0f64;
            let mut max_width = std::f64::INFINITY;

            match (position.left, position.right, position.width) {
                (Some(left), Some(right), unused) => {
                    let width = (stack_width - right - left).max(0.);
                    min_width = width;
                    max_width = width;
                    if unused.is_some() {
                        warn!("detected over-constrained stack element");
                    }
                }
                (_, _, Some(width)) => {
                    min_width = width;
                    max_width = width;
                }
                _ => { /* no width constraint */ }
            }

            let mut min_height = 0f64;
            let mut max_height = std::f64::INFINITY;

             match (position.top, position.bottom, position.height) {
                (Some(top), Some(bottom), unused) => {
                    let height = (stack_height - bottom - top).max(0.);
                    min_height = height;
                    max_height = height;
                    if unused.is_some() {
                        warn!("detected over-constrained stack element");
                    }
                }
                (_, _, Some(height)) => {
                    min_height = height;
                    max_height = height;
                }
                _ => { /* no height constraint */ }
            }

            let child_bc = BoxConstraints::new(
                Size::new(min_width, min_height),
                Size::new(max_width, max_height),
            );

            let child_size = child.widget.layout(ctx, &child_bc, data, env);
            let align = self.align;

            let offset_x = match (position.left, position.right) {
                (Some(left), _) => left,
                (None, Some(right)) => stack_width - right - child_size.width,
                (None, None) => {
                    let extra_width = stack_width - child_size.width;
                    align.resolve(Rect::new(0., 0., extra_width, 0.)).expand().x
                }
            };

            let offset_y = match (position.top, position.bottom) {
                (Some(top), _) => top,
                (None, Some(bottom)) => stack_height - bottom - child_size.height,
                (None, None) => {
                    let extra_height = stack_height - child_size.height;
                    align.resolve(Rect::new(0., 0., 0., extra_height)).expand().y
                }
            };

            let origin = Point::new(offset_x, offset_y);
            child.widget.set_origin(ctx, data, env, origin);
        }

        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx<'_, '_, '_>, data: &T, env: &Env) {
        let size = ctx.size();

        if self.clip {
            ctx.clip(size.to_rect());
        }
        for child in &mut self.children {
            child.widget.paint(ctx, data, env);
        }
    }
}
