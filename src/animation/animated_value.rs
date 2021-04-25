use crate::animation::AnimationCurve;
use druid::{Color, Data, EventCtx, Insets, Point, Rect, Size, Vec2};
use std::ops::Deref;
use std::time::Duration;

/// Animated provides simple transition-animations for single values or tuples of values that implement
/// Interpolate.
pub struct Animated<T> {
    start: T,
    end: T,
    full_duration: f64,
    current_duration: f64,
    curve: AnimationCurve,
    layout: bool,
    current: T,
}

impl<T: Interpolate + Data> Animated<T> {
    /// Creates a new animation with a start value, a duration and a curve.
    /// The paint and layout flags indicate if animate should request paint or layout when the value
    /// changes.
    pub fn new(
        value: T,
        duration: Duration,
        curve: impl Into<AnimationCurve>,
        layout: bool,
    ) -> Self {
        Animated {
            start: value.clone(),
            end: value.clone(),
            full_duration: duration.as_secs_f64(),
            current_duration: duration.as_secs_f64(),
            curve: curve.into(),
            layout,

            current: value,
        }
    }

    pub fn jump(value: T, layout: bool) -> Self {
        Animated {
            start: value.clone(),
            end: value.clone(),
            full_duration: 0.0,
            current_duration: 0.0,
            curve: Default::default(),
            layout,
            current: value,
        }
    }

    /// Returns the interpolated value.
    pub fn get(&self) -> T {
        self.current.clone()
    }

    pub fn start(&self) -> T {
        self.start.clone()
    }

    pub fn end(&self) -> T {
        self.end.clone()
    }

    pub fn duration(&self) -> Duration {
        Duration::from_secs_f64(self.full_duration)
    }

    pub fn progress(&self) -> f64 {
        self.current_duration / self.full_duration
    }

    pub fn curve(&self) -> &AnimationCurve {
        &self.curve
    }

    pub fn set_duration(&mut self, duration: Duration) {
        let ratio = self.full_duration / self.current_duration;
        self.full_duration = duration.as_secs_f64();
        self.current_duration = self.full_duration * ratio;
    }

    pub fn set_curve(&mut self, curve: impl Into<AnimationCurve>) {
        self.curve = curve.into();
    }

    pub fn animating(&self) -> bool {
        self.current_duration < self.full_duration
    }

    /// Set the new end value. If the animation is currently running, it will start from the current
    /// value.
    //TODO: change to RequestCtx to automatically update
    pub fn animate(&mut self, value: T) {
        if !value.same(&self.end) {
            self.start = self.current.clone();
            self.end = value.clone();
            self.current_duration = 0.0;
            if self.full_duration == 0.0 {
                self.current = value;
                //if self.layout { ctx.request_layout(); } else { ctx.request_paint(); }
            } // else { ctx.request_anim_frame(); }
        }
    }

    pub fn animate_with(&mut self, value: T, duration: Duration, curve: impl Into<AnimationCurve>) {
        self.set_curve(curve);
        self.set_duration(duration);
        self.animate(value);
    }

    /// Stop the animation and sets the value.
    pub fn jump_to_value(&mut self, value: T) {
        self.start = value.clone();
        self.end = value.clone();
        self.current = value;
        self.current_duration = self.full_duration;
    }

    /// Stop the animation at the current value
    pub fn end_animation(&mut self) {
        self.start = self.current.clone();
        self.end = self.current.clone();
        self.current_duration = self.full_duration;
    }

    /// This method should always be called in Event::AnimationFrame.
    /// It updates the value according to duration and curve.
    /// If the value changes and the specific flags are set paint or layout are requested.
    /// If the transition's end isn't reached an additional animation-frame is requested.
    ///
    pub fn update(&mut self, nanos: u64, ctx: &mut EventCtx) {
        // This must happen before updating the value!
        if self.animating() {
            if self.layout {
                ctx.request_layout();
            } else {
                ctx.request_paint();
            }
        }

        self.current_duration += (nanos as f64) * 0.000000001;
        self.current_duration = self.current_duration.min(self.full_duration);

        if self.animating() {
            ctx.request_anim_frame();
            self.current = self.start.interpolate(
                &self.end,
                self.curve
                    .translate(self.current_duration / self.full_duration),
            );
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

///
pub trait Interpolate: Clone {
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
