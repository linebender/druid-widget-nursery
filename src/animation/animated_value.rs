use crate::animation::{AnimationCurve, AnimationController};
use druid::{Color, Insets, Point, Rect, Size, Vec2};
use std::ops::Deref;

use crate::RequestCtx;

/// Animated provides simple transition-animations for single values or tuples of values that implement
/// [`Interpolate`].
pub struct Animated<T> {
    start: T,
    end: T,
    controller: AnimationController,
    curve: AnimationCurve,

    current: T,
}

impl<T: Interpolate> Animated<T> {
    /// Creates a new animation with a start value.
    ///
    /// ```
    /// # use druid::Color;
    /// # use druid_widget_nursery::animation::{Animated, AnimationCurve};
    /// let animated = Animated::new(Color::RED)
    ///    .duration(0.8)
    ///    .curve(AnimationCurve::EASE_IN_OUT);
    /// ```
    ///
    pub fn new(value: T) -> Self {
        let controller = AnimationController::new();
        Animated {
            start: value.clone(),
            end: value.clone(),
            controller,
            curve: Default::default(),
            current: value,
        }
    }

    /// Same as [`new`], but set duration to zero.
    ///
    /// [`new`]: #method.new
    pub fn jump(value: T) -> Self {
        let controller = AnimationController::new().duration(0.0);
        Animated {
            start: value.clone(),
            end: value.clone(),
            controller,
            curve: Default::default(),
            current: value,
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
        self.curve = curve;
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
        self.controller.set_duration(duration);
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
        self.controller.set_layout(layout);
    }

    /// Returns the interpolated value.
    pub fn get(&self) -> T {
        self.current.clone()
    }

    /// Returns the start value.
    pub fn start(&self) -> T {
        self.start.clone()
    }

    /// Returns the end value.
    pub fn end(&self) -> T {
        self.end.clone()
    }

    /// Returns the animation progress (between 0.0 and 1.0)
    pub fn progress(&self) -> f64 {
        self.controller.fraction()
    }

    /// Returns true if the animation is running.
    pub fn animating(&self) -> bool {
        self.controller.animating()
    }

    /// Set the new end value.
    ///
    /// If the animation is currently running, it will start from the
    /// current value.
    pub fn animate(&mut self, ctx: &mut impl RequestCtx, value: T) {
        if value != self.end {
            self.start = self.current.clone();
            self.end = value;
            self.controller.reset();
            self.controller.start(ctx);
        }
    }

    /// Set the new end value, curve and duration.
    ///
    /// If the animation is currently running, it will start from the current value.
    pub fn animate_with(
        &mut self,
        ctx: &mut impl RequestCtx,
        value: T,
        duration: f64,
        curve: AnimationCurve,
    ) {
        self.set_curve(curve);
        self.set_duration(duration);
        self.animate(ctx, value);
    }

    /// Stop the animation and set the value.
    pub fn jump_to_value(&mut self, value: T) {
        self.controller.reset();
        self.start = value.clone();
        self.end = value.clone();
        self.current = value;
    }

    /// Stop the animation at the current value
    pub fn end_animation(&mut self) {
        self.controller.reset();
        self.start = self.current.clone();
        self.end = self.current.clone();
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
        self.controller.update(ctx, nanos);
        if self.animating() {
            let fraction = self.controller.fraction();
            self.current = self.start.interpolate(&self.end, self.curve.translate(fraction));
        } else {
            self.current = self.end.clone();
        }
    }
}

impl<T> Deref for Animated<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.current
    }
}

/// Interpolate between two values
///
/// Interpolate between `self` and `other` where `value` is the
/// position (between 0 and 1). For example, a simple linear
/// interpolation is implemented as: `self + (other - self) * value`
pub trait Interpolate: PartialEq + Clone {
    fn interpolate(&self, other: &Self, value: f64) -> Self;
}

impl Interpolate for f64 {
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        self + (other - self) * value
    }
}

//TODO: make this more efficient
impl Interpolate for Color {
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        let (r1, g1, b1, a1) = self.as_rgba();
        let (r2, g2, b2, a2) = other.as_rgba();

        Color::rgba(
            r1.interpolate(&r2, value),
            g1.interpolate(&g2, value),
            b1.interpolate(&b2, value),
            a1.interpolate(&a2, value),
        )
    }
}

impl Interpolate for Vec2 {
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        Vec2::new(
            self.x.interpolate(&other.x, value),
            self.y.interpolate(&other.y, value),
        )
    }
}

impl Interpolate for Point {
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        Point::new(
            self.x.interpolate(&other.x, value),
            self.y.interpolate(&other.y, value),
        )
    }
}

impl Interpolate for Size {
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        Size::new(
            self.width.interpolate(&other.width, value),
            self.height.interpolate(&other.height, value),
        )
    }
}

impl Interpolate for Rect {
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        Rect::from_origin_size(
            self.origin().interpolate(&other.origin(), value),
            self.size().interpolate(&other.size(), value),
        )
    }
}

impl Interpolate for Insets {
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        Insets::new(
            self.x0.interpolate(&other.x0, value),
            self.y0.interpolate(&other.y0, value),
            self.x1.interpolate(&other.x1, value),
            self.y1.interpolate(&other.y1, value),
        )
    }
}

impl<A: Interpolate, B: Interpolate> Interpolate for (A, B) {
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        (
            self.0.interpolate(&other.0, value),
            self.1.interpolate(&other.1, value),
        )
    }
}

impl<A: Interpolate, B: Interpolate, C: Interpolate> Interpolate for (A, B, C) {
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        (
            self.0.interpolate(&other.0, value),
            self.1.interpolate(&other.1, value),
            self.2.interpolate(&other.2, value),
        )
    }
}

impl<A: Interpolate, B: Interpolate, C: Interpolate, D: Interpolate> Interpolate for (A, B, C, D) {
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        (
            self.0.interpolate(&other.0, value),
            self.1.interpolate(&other.1, value),
            self.2.interpolate(&other.2, value),
            self.3.interpolate(&other.3, value),
        )
    }
}

impl<A: Interpolate, B: Interpolate, C: Interpolate, D: Interpolate, E: Interpolate> Interpolate
    for (A, B, C, D, E)
{
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        (
            self.0.interpolate(&other.0, value),
            self.1.interpolate(&other.1, value),
            self.2.interpolate(&other.2, value),
            self.3.interpolate(&other.3, value),
            self.4.interpolate(&other.4, value),
        )
    }
}

impl<
        A: Interpolate,
        B: Interpolate,
        C: Interpolate,
        D: Interpolate,
        E: Interpolate,
        F: Interpolate,
    > Interpolate for (A, B, C, D, E, F)
{
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        (
            self.0.interpolate(&other.0, value),
            self.1.interpolate(&other.1, value),
            self.2.interpolate(&other.2, value),
            self.3.interpolate(&other.3, value),
            self.4.interpolate(&other.4, value),
            self.5.interpolate(&other.5, value),
        )
    }
}
