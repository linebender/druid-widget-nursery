use crate::partial::{OptionSome, PartialWidget, Prism};
use druid::widget::{Checkbox, Radio};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, Size, UpdateCtx, Vec2, Widget, WidgetPod,
};
use std::fmt::Debug;

///A Radio which has further configuration for the value it represents
pub struct MultiRadio<W, T, U, P> {
    inner: WidgetPod<T, PartialWidget<W, U, P>>,
    radio: WidgetPod<bool, Radio<bool>>,
    indent: f64,
    space: f64,
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
            inner: WidgetPod::new(PartialWidget::new(widget, initial_data, prism)),
            radio: WidgetPod::new(Radio::new(name, true)),
            indent: 20.0,
            space: 10.0,
        }
    }

    /// Set show_when_disabled, the default is false.
    pub fn set_show_when_disabled(&mut self, show_when_disabled: bool) {
        self.inner
            .widget_mut()
            .set_show_when_disabled(show_when_disabled);
    }

    /// Builder-style method to set show_when_disabled to true.
    /// The default is false.
    pub fn show_when_disabled(mut self) -> Self {
        self.inner.widget_mut().set_show_when_disabled(true);
        self
    }

    /// Set the indent of the inner widget
    pub fn set_indent(&mut self, indent: f64) {
        self.indent = indent;
    }

    /// Builder-style method to set the indent of the inner widget
    pub fn with_indent(mut self, indent: f64) -> Self {
        self.indent = indent;
        self
    }

    /// Set the indent of the inner widget
    pub fn set_space(&mut self, space: f64) {
        self.space = space;
    }

    /// Builder-style method to set the indent of the inner widget
    pub fn with_space(mut self, space: f64) -> Self {
        self.space = space;
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
        self.inner.event(ctx, event, data, env);

        let mut enabled = self.is_enabled();
        self.radio.event(ctx, event, &mut enabled, env);

        if enabled && !self.is_enabled() {
            self.enable(data);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.inner.lifecycle(ctx, event, data, env);
        self.radio.lifecycle(ctx, event, &self.is_enabled(), env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        self.inner.update(ctx, data, env);
        self.radio.update(ctx, &self.is_enabled(), env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let radio_size = self.radio.layout(ctx, bc, &self.is_enabled(), env);
        self.radio
            .set_origin(ctx, &self.is_enabled(), env, Point::ZERO);

        let inner_origin = Vec2::new(self.indent, radio_size.height + self.space);
        let inner_bc = bc.shrink(inner_origin.to_size());

        let inner_size = self.inner.layout(ctx, &inner_bc, data, env);
        self.inner
            .set_origin(ctx, data, env, inner_origin.to_point());

        if !inner_size.is_empty() {
            Size::new(
                radio_size.width.max(inner_size.width + inner_origin.x),
                inner_origin.y + inner_size.height,
            )
        } else {
            radio_size
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.radio.paint(ctx, &self.is_enabled(), env);
        self.inner.paint(ctx, data, env);
    }
}

pub struct MultiCheckbox<W, T> {
    inner: WidgetPod<Option<T>, PartialWidget<W, T, OptionSome>>,
    check_box: WidgetPod<bool, Checkbox>,
    indent: f64,
    space: f64,
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
            inner: WidgetPod::new(PartialWidget::new(widget, initial_data, OptionSome)),
            check_box: WidgetPod::new(Checkbox::new(name)),
            indent: 20.0,
            space: 10.0,
        }
    }

    /// Set show_when_disabled, the default is false.
    pub fn set_show_when_disabled(&mut self, show_when_disabled: bool) {
        self.inner
            .widget_mut()
            .set_show_when_disabled(show_when_disabled);
    }

    /// Builder-style method to set show_when_disabled to true.
    /// The default is false.
    pub fn show_when_disabled(mut self) -> Self {
        self.inner.widget_mut().set_show_when_disabled(true);
        self
    }

    /// Set the indent of the inner widget
    pub fn set_indent(&mut self, indent: f64) {
        self.indent = indent;
    }

    /// Builder-style method to set the indent of the inner widget
    pub fn with_indent(mut self, indent: f64) -> Self {
        self.indent = indent;
        self
    }

    /// Set the indent of the inner widget
    pub fn set_space(&mut self, space: f64) {
        self.space = space;
    }

    /// Builder-style method to set the indent of the inner widget
    pub fn with_space(mut self, space: f64) -> Self {
        self.space = space;
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
        self.inner.lifecycle(ctx, event, data, env);
        self.check_box
            .lifecycle(ctx, event, &self.is_enabled(), env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &Option<T>, data: &Option<T>, env: &Env) {
        self.inner.update(ctx, data, env);
        self.check_box.update(ctx, &self.is_enabled(), env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &Option<T>,
        env: &Env,
    ) -> Size {
        let radio_size = self.check_box.layout(ctx, bc, &self.is_enabled(), env);
        self.check_box
            .set_origin(ctx, &self.is_enabled(), env, Point::ZERO);

        let inner_origin = Vec2::new(self.indent, radio_size.height + self.space);
        let inner_bc = bc.shrink(inner_origin.to_size());

        let inner_size = self.inner.layout(ctx, &inner_bc, data, env);
        self.inner
            .set_origin(ctx, data, env, inner_origin.to_point());

        if !inner_size.is_empty() {
            Size::new(
                radio_size.width.max(inner_size.width + inner_origin.x),
                inner_origin.y + inner_size.height,
            )
        } else {
            radio_size
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Option<T>, env: &Env) {
        self.check_box.paint(ctx, &self.is_enabled(), env);
        self.inner.paint(ctx, data, env);
    }
}
