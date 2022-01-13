use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, Size, UpdateCtx, Widget, WidgetPod,
};

#[cfg(feature = "derive")]
pub use druid_widget_nursery_derive::Prism;

/// A trait similar to druid::Lens that represents data which is not always present
///
/// This is just a simple prototype for me to work with until [`#1136`] is merged.
/// There is also discussion about Prisms in [`#1135`].
///
/// [`#1136`]: https://github.com/linebender/druid/pull/1136
/// [`#1135`]: https://github.com/linebender/druid/issues/1135
pub trait Prism<T, U> {
    ///Extract the data (if present) from the outer type
    fn get(&self, data: &T) -> Option<U>;
    ///Store the data back in the outer type
    fn put(&self, data: &mut T, inner: U);
}

/// A trait implemented by PrismWrappers to check if this widget can handle the current data.
pub trait PrismWidget<T>: Widget<T> {
    fn is_active_for(&self, data: &T) -> bool;
}

impl<T: Data> Widget<T> for Box<dyn PrismWidget<T>> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        (**self).event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        (**self).lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        (**self).update(ctx, old_data, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        (**self).layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        (**self).paint(ctx, data, env);
    }
}

/// A Widget wrapper which disables the inner widget if its data is not present. If you dont need to disable
/// the widget, use PrismWrap instead.
/// The main use case are enum variants
pub struct DisablePrismWrap<W, U, P> {
    widget: WidgetPod<U, W>,
    current_data: U,
    prism: P,
    enabled: bool,
}

impl<W, U, P> DisablePrismWrap<W, U, P>
where
    U: Data,
    W: Widget<U>,
{
    /// creates a new PartialWidget from the inner widget, the initial data
    /// and a Prism, which can extract U from T if present and store it back
    ///
    /// Prisms work similar to druid::Lens except that get returns Option<U>
    /// instead of U which makes it useful for Enums.
    pub fn new(widget: W, initial_data: U, prism: P) -> Self {
        Self {
            widget: WidgetPod::new(widget),
            current_data: initial_data,
            prism,
            enabled: false,
        }
    }

    /// Injects the this widgets internal data (the data before this widget got disabled, if it was
    /// never enabled this is initial data) into data.
    /// If data was this widgets external data, the widget will get enabled
    /// during the next update call.
    ///
    /// You can use this method from a parent widget (Controller or similar) to enable
    /// this widget, since it has no means to do it itself.
    pub fn enable<T>(&self, data: &mut T)
    where
        P: Prism<T, U>,
    {
        self.prism.put(data, self.current_data.clone());
    }

    /// Returns if the current widget is active. This is true if get(data) returned Some() during
    /// the last call of update
    ///
    /// For more info view enable
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the internal data of the widget. This works also when the widget is disabled.
    /// By calling enable the current internal data gets injected into the external data.
    pub fn internal_data(&self) -> &U {
        &self.current_data
    }
}

impl<W, T, U, P> PrismWidget<T> for DisablePrismWrap<W, U, P>
where
    U: Data,
    W: Widget<U>,
    P: Prism<T, U>,
{
    fn is_active_for(&self, data: &T) -> bool {
        self.prism.get(data).is_some()
    }
}

impl<W, T, U, P> Widget<T> for DisablePrismWrap<W, U, P>
where
    U: Data,
    W: Widget<U>,
    P: Prism<T, U>,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Some(mut inner_data) = self.prism.get(data) {
            self.widget.event(ctx, event, &mut inner_data, env);
            self.prism.put(data, inner_data);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if env.get(Env::DEBUG_WIDGET) {
            let lifecycle_event = event;
            dbg!(lifecycle_event);
        }
        if let LifeCycle::WidgetAdded = event {
            if let Some(data) = self.prism.get(data) {
                self.enabled = true;
                self.current_data = data;
            } else {
                self.enabled = false;
            }

            ctx.set_disabled(!self.enabled);
        }
        self.widget.lifecycle(ctx, event, &self.current_data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        if let Some(data) = self.prism.get(data) {
            self.enabled = true;
            self.current_data = data;
        } else {
            self.enabled = false;
        }
        self.widget.update(ctx, &self.current_data, env);
        ctx.set_disabled(!self.enabled);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &T, env: &Env) -> Size {
        let size = self.widget.layout(ctx, bc, &self.current_data, env);
        ctx.set_baseline_offset(self.widget.baseline_offset());
        self.widget
            .set_origin(ctx, &self.current_data, env, Point::ORIGIN);
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &T, env: &Env) {
        self.widget.paint(ctx, &self.current_data, env);
    }
}

/// A Widget wrapper similar to PrismWrapDisable, but with the limitation that this widget should
/// only be visible if its data is present. In return you dont need to provide the initial data.
pub struct PrismWrap<W, P, U> {
    inner: WidgetPod<U, W>,
    prism: P,
    cached_data: Option<U>,
}

impl<W: Widget<U>, P, U> PrismWrap<W, P, U> {
    pub fn new(widget: W, prism: P) -> Self {
        PrismWrap {
            inner: WidgetPod::new(widget),
            prism,
            cached_data: None,
        }
    }
}

impl<T, U: Data, P: Prism<T, U>, W: Widget<U>> PrismWidget<T> for PrismWrap<W, P, U> {
    fn is_active_for(&self, data: &T) -> bool {
        self.prism.get(data).is_some()
    }
}

impl<T, U: Data, P: Prism<T, U>, W: Widget<U>> Widget<T> for PrismWrap<W, P, U> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Some(mut inner_data) = self.prism.get(data) {
            if self.cached_data.is_some() {
                self.inner.event(ctx, event, &mut inner_data, env);
                self.prism.put(data, inner_data);
            }
        } else if let Some(mut data) = self.cached_data.clone() {
            self.inner.event(ctx, event, &mut data, env);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            self.cached_data = self.prism.get(data);
        }
        if let Some(data) = &self.cached_data {
            self.inner.lifecycle(ctx, event, data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        if let Some(data) = self.prism.get(data) {
            if self.cached_data.is_some() {
                self.inner.update(ctx, &data, env);
            }
            self.cached_data = Some(data);
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &T, env: &Env) -> Size {
        if let Some(data) = &self.cached_data {
            let size = self.inner.layout(ctx, bc, data, env);
            self.inner.set_origin(ctx, data, env, Point::ORIGIN);
            size
        } else {
            bc.min()
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &T, env: &Env) {
        if let Some(data) = &self.cached_data {
            self.inner.paint(ctx, data, env);
        }
    }
}

pub struct OptionSome;

impl<T: Data> Prism<Option<T>, T> for OptionSome {
    fn get(&self, data: &Option<T>) -> Option<T> {
        (*data).clone()
    }

    fn put(&self, data: &mut Option<T>, inner: T) {
        *data = Some(inner)
    }
}

pub struct OptionNone;

impl<T> Prism<Option<T>, ()> for OptionNone {
    fn get(&self, data: &Option<T>) -> Option<()> {
        if data.is_none() {
            Some(())
        } else {
            None
        }
    }

    fn put(&self, data: &mut Option<T>, _: ()) {
        *data = None;
    }
}

pub struct ResultOk;

impl<T: Data, E: Data> Prism<Result<T, E>, T> for ResultOk {
    fn get(&self, data: &Result<T, E>) -> Option<T> {
        data.clone().ok()
    }

    fn put(&self, data: &mut Result<T, E>, inner: T) {
        *data = Ok(inner);
    }
}

pub struct ResultErr;

impl<T: Data, E: Data> Prism<Result<T, E>, E> for ResultErr {
    fn get(&self, data: &Result<T, E>) -> Option<E> {
        data.clone().err()
    }

    fn put(&self, data: &mut Result<T, E>, inner: E) {
        *data = Err(inner);
    }
}

pub struct Closures<F, G>(pub F, pub G);

impl<F, G, T, U> Prism<T, U> for Closures<F, G>
where
    F: Fn(&T) -> Option<U>,
    G: Fn(&mut T, U),
{
    fn get(&self, data: &T) -> Option<U> {
        (self.0)(data)
    }

    fn put(&self, data: &mut T, inner: U) {
        (self.1)(data, inner);
    }
}
