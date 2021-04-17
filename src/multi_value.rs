use crate::animation::{Animated, AnimationCurve, Interpolate, SimpleCurve};
use crate::prism::{DisablePrismWrap, OptionSome, Prism};
use druid::widget::{Checkbox, Radio};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, RenderContext, Size, UpdateCtx, Vec2, Widget, WidgetPod,
};
use std::fmt::Debug;
use std::time::Duration;

///A Radio which has further configuration for the value it represents
pub struct MultiRadio<W, T, U, P> {
    inner: WidgetPod<T, DisablePrismWrap<W, U, P>>,
    radio: WidgetPod<bool, Radio<bool>>,
    layout: IndentLayout,
}

impl<W, T, U, P> MultiRadio<W, T, U, P>
where
    T: Data,
    U: Data,
    P: Prism<T, U>,
    W: Widget<U>,
{
    /// creates a new MultiRadio from the inner widget, the initial data
    /// and a Prism which decides the part of the data represented here
    /// the external state.
    ///
    /// Prisms work similar to druid::Lens except that get returns Option<U>
    /// instead of U which makes it useful for Enums.
    pub fn new(name: &str, widget: W, initial_data: U, prism: P) -> Self {
        Self {
            inner: WidgetPod::new(DisablePrismWrap::new(widget, initial_data, prism)),
            radio: WidgetPod::new(Radio::new(name, true)),
            layout: IndentLayout::new(),
        }
    }

    /// Set the indent of the inner widget
    pub fn set_indent(&mut self, indent: f64) {
        self.layout.indent = indent;
    }

    /// Builder-style method to set the indent of the inner widget
    pub fn with_indent(mut self, indent: f64) -> Self {
        self.layout.indent = indent;
        self
    }

    /// Set the indent of the inner widget
    pub fn set_space(&mut self, space: f64) {
        self.layout.space = space;
    }

    /// Builder-style method to set the indent of the inner widget
    pub fn with_space(mut self, space: f64) -> Self {
        self.layout.space = space;
        self
    }

    /// Set show_when_disabled, the default is false.
    pub fn set_show_when_disabled(&mut self, show_when_disabled: bool) {
        self.layout.always_visible = show_when_disabled;
    }

    /// Builder-style method to set show_when_disabled to true.
    /// The default is false.
    pub fn show_when_disabled(mut self) -> Self {
        self.layout.always_visible = true;
        self
    }

    /// A Builder-style method to set the duration for the transition
    /// between shown and hidden.
    pub fn set_transition_duration(&mut self, duration: Duration) {
        self.layout.height.set_duration(duration);
    }

    /// Set the duration for the transition between shown and hidden.
    pub fn with_transition_duration(mut self, duration: Duration) -> Self {
        self.layout.height.set_duration(duration);
        self
    }

    /// A Builder-style method to set the curve for the transition between
    /// shown and hidden.
    pub fn set_transition_curve(&mut self, curve: impl Into<AnimationCurve>) {
        self.layout.height.set_curve(curve.into());
    }

    /// Set the curve for the transition between shown and hidden.
    pub fn with_transition_curve(mut self, curve: impl Into<AnimationCurve>) -> Self {
        self.layout.height.set_curve(curve.into());
        self
    }

    /// Injects the this widgets internal data (the data before this widget got disabled, if it was
    /// never enabled this is initial data) into data.
    /// If data was this widgets external data, the widget will get enabled
    /// during the next update call.
    ///
    /// You can use this method from a parent widget (Controller or similar) to enable
    /// this widget.
    pub fn enable(&self, data: &mut T) {
        self.inner.widget().enable(data);
    }

    /// Returns if the current widget is active. This is true if get(data) returned Some() during
    /// the last call of update
    ///
    /// For more info view enable
    pub fn is_enabled(&self) -> bool {
        self.inner.widget().is_enabled()
    }

    /// Returns the internal data of the widget. This works also when the widget is disabled.
    /// By calling enable the current internal data gets injected into the external data.
    pub fn internal_data(&self) -> &U {
        &self.inner.widget().internal_data()
    }
}

impl<W, U, T, P> Widget<T> for MultiRadio<W, T, U, P>
where
    T: Data,
    U: Data + Debug,
    P: Prism<T, U>,
    W: Widget<U>,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Event::AnimFrame(nanos) = event {
            self.layout.update(nanos, ctx);
        }

        let mut enabled = self.is_enabled();
        self.radio.event(ctx, event, &mut enabled, env);

        if enabled && !self.is_enabled() {
            self.enable(data);
        }

        self.inner.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.radio.lifecycle(ctx, event, &self.is_enabled(), env);
        self.inner.lifecycle(ctx, event, data, env);
        if let LifeCycle::WidgetAdded = event {
            self.layout.init_visible(self.is_enabled());
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        self.inner.update(ctx, data, env);
        self.radio.update(ctx, &self.is_enabled(), env);
        if self.layout.set_visible(self.is_enabled()) {
            ctx.request_anim_frame();
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let enabled = self.is_enabled();
        self.layout.layout(
            &mut self.radio,
            &mut self.inner,
            &enabled,
            data,
            ctx,
            bc,
            env,
        )
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        let enabled = self.is_enabled();
        self.layout
            .paint(&mut self.radio, &mut self.inner, &enabled, data, ctx, env);
    }
}

pub struct MultiCheckbox<W, T> {
    inner: WidgetPod<Option<T>, DisablePrismWrap<W, T, OptionSome>>,
    check_box: WidgetPod<bool, Checkbox>,
    layout: IndentLayout,
}

impl<W, T> MultiCheckbox<W, T>
where
    T: Data,
    W: Widget<T>,
{
    /// creates a new MultiCheckbox from the name, the inner widget and the initial data.
    ///
    /// The closures work similar to druid::Lens except that get returns Option<U>
    /// instead of U which makes it useful for Enums.
    pub fn new(name: &str, widget: W, initial_data: T) -> Self {
        Self {
            inner: WidgetPod::new(DisablePrismWrap::new(widget, initial_data, OptionSome)),
            check_box: WidgetPod::new(Checkbox::new(name)),
            layout: IndentLayout::new(),
        }
    }

    /// Set the indent of the inner widget
    pub fn set_indent(&mut self, indent: f64) {
        self.layout.indent = indent;
    }

    /// Builder-style method to set the indent of the inner widget
    pub fn with_indent(mut self, indent: f64) -> Self {
        self.layout.indent = indent;
        self
    }

    /// Set the indent of the inner widget
    pub fn set_space(&mut self, space: f64) {
        self.layout.space = space;
    }

    /// Builder-style method to set the indent of the inner widget
    pub fn with_space(mut self, space: f64) -> Self {
        self.layout.space = space;
        self
    }

    /// Set show_when_disabled, the default is false.
    pub fn set_show_when_disabled(&mut self, show_when_disabled: bool) {
        self.layout.always_visible = show_when_disabled;
    }

    /// Builder-style method to set show_when_disabled to true.
    /// The default is false.
    pub fn show_when_disabled(mut self) -> Self {
        self.layout.always_visible = true;
        self
    }

    /// A Builder-style method to set the duration for the transition
    /// between shown and hidden.
    pub fn set_transition_duration(&mut self, duration: Duration) {
        self.layout.height.set_duration(duration);
    }

    /// Set the duration for the transition between shown and hidden.
    pub fn with_transition_duration(mut self, duration: Duration) -> Self {
        self.layout.height.set_duration(duration);
        self
    }

    /// A Builder-style method to set the curve for the transition between
    /// shown and hidden.
    pub fn set_transition_curve(&mut self, curve: impl Into<AnimationCurve>) {
        self.layout.height.set_curve(curve.into());
    }

    /// Set the curve for the transition between shown and hidden.
    pub fn with_transition_curve(mut self, curve: impl Into<AnimationCurve>) -> Self {
        self.layout.height.set_curve(curve.into());
        self
    }

    /// Injects the this widgets internal data (the data before this widget got disabled, if it was
    /// never enabled this is initial data) into data.
    /// If data was this widgets external data, the widget will get enabled
    /// during the next update call.
    ///
    /// You can use this method from a parent widget (Controller or similar) to enable
    /// this widget.
    pub fn enable(&self, data: &mut Option<T>) {
        self.inner.widget().enable(data);
    }

    /// Returns if the current widget is active. This is true if get(data) returned Some() during
    /// the last call of update
    ///
    /// For more info view enable
    pub fn is_enabled(&self) -> bool {
        self.inner.widget().is_enabled()
    }

    /// Returns the internal data of the widget. This works also when the widget is disabled.
    /// By calling enable the current internal data gets injected into the external data.
    pub fn internal_data(&self) -> &T {
        &self.inner.widget().internal_data()
    }
}

impl<W, T> Widget<Option<T>> for MultiCheckbox<W, T>
where
    T: Data,
    W: Widget<T>,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Option<T>, env: &Env) {
        if let Event::AnimFrame(nanos) = event {
            self.layout.update(nanos, ctx);
        }

        self.inner.event(ctx, event, data, env);

        let mut enabled = self.is_enabled();
        self.check_box.event(ctx, event, &mut enabled, env);

        if enabled && !self.is_enabled() {
            self.enable(data);
        }
        if !enabled && self.is_enabled() {
            *data = None;
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &Option<T>,
        env: &Env,
    ) {
        self.check_box
            .lifecycle(ctx, event, &self.is_enabled(), env);
        self.inner.lifecycle(ctx, event, data, env);
        if let LifeCycle::WidgetAdded = event {
            self.layout.init_visible(self.is_enabled());
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &Option<T>, data: &Option<T>, env: &Env) {
        self.inner.update(ctx, data, env);
        self.check_box.update(ctx, &self.is_enabled(), env);
        if self.layout.set_visible(self.is_enabled()) {
            ctx.request_anim_frame();
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &Option<T>,
        env: &Env,
    ) -> Size {
        let enabled = self.is_enabled();
        self.layout.layout(
            &mut self.check_box,
            &mut self.inner,
            &enabled,
            data,
            ctx,
            bc,
            env,
        )
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Option<T>, env: &Env) {
        let enabled = self.is_enabled();
        self.layout.paint(
            &mut self.check_box,
            &mut self.inner,
            &enabled,
            data,
            ctx,
            env,
        );
    }
}

struct IndentLayout {
    space: f64,
    indent: f64,
    always_visible: bool,
    height: Animated<f64>,
}

impl IndentLayout {
    pub fn new() -> Self {
        IndentLayout {
            space: 10.0,
            indent: 30.0,
            always_visible: false,
            height: Animated::new(
                0.0,
                Duration::from_secs_f64(0.2),
                SimpleCurve::EaseOut,
                true,
            ),
        }
    }

    pub fn update(&mut self, nanos: &u64, ctx: &mut EventCtx) {
        self.height.update(*nanos, ctx);
    }

    pub fn set_visible(&mut self, visible: bool) -> bool {
        //TODO: update this when context traits are stabilised
        let new_value = if visible || self.always_visible { 1.0 } else { 0.0 };
        if (new_value - self.height.end()).abs() > f64::EPSILON {
            self.height.animate(new_value);
            true
        } else {
            false
        }
    }

    pub fn init_visible(&mut self, visible: bool) {
        self.height
            .jump_to_value(if visible || self.always_visible {
                1.0
            } else {
                0.0
            });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn layout<A: Data, B: Data>(
        &self,
        header: &mut WidgetPod<A, impl Widget<A>>,
        body: &mut WidgetPod<B, impl Widget<B>>,
        data_a: &A,
        data_b: &B,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        env: &Env,
    ) -> Size {
        let radio_size = header.layout(ctx, bc, data_a, env);
        header.set_origin(ctx, data_a, env, Point::ZERO);

        let inner_origin = Vec2::new(self.indent, radio_size.height + self.space);
        let inner_bc = bc.shrink(inner_origin.to_size());

        let inner_size = body.layout(ctx, &inner_bc, data_b, env);
        body.set_origin(ctx, data_b, env, inner_origin.to_point());

        if !inner_size.is_empty() {
            Size::new(
                radio_size.width.max(inner_size.width + inner_origin.x),
                radio_size
                    .height
                    .interpolate(&(inner_origin.y + inner_size.height), self.height.get()),
            )
        } else {
            radio_size
        }
    }

    pub fn paint<A: Data, B: Data>(
        &self,
        header: &mut WidgetPod<A, impl Widget<A>>,
        body: &mut WidgetPod<B, impl Widget<B>>,
        data_a: &A,
        data_b: &B,
        ctx: &mut PaintCtx,
        env: &Env,
    ) {
        header.paint(ctx, data_a, env);
        if self.height.animating() {
            let paint_rect = ctx.size().to_rect();
            ctx.clip(paint_rect);
        }
        body.paint(ctx, data_b, env);
    }
}
