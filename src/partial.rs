use druid::{Widget, Data, LifeCycle, EventCtx, PaintCtx, BoxConstraints, LifeCycleCtx, Size, LayoutCtx, Event, Env, UpdateCtx};

/// A Widget which displays data which is not always present
/// The main use case are an enum variants
pub struct PartialWidget<W, U, P> {
    widget: W,
    current_data: U,
    prism: P,
    enabled: bool,
    show_when_disabled: bool,
}

impl<W, U, P> PartialWidget<W, U, P> where
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
            widget,
            current_data: initial_data,
            prism,
            enabled: false,
            show_when_disabled: false,
        }
    }

    /// Set show_when_disabled, the default is false.
    pub fn set_show_when_disabled(&mut self, show_when_disabled: bool) {
        self.show_when_disabled = show_when_disabled;
    }

    /// Builder-style method to set show_when_disabled to true.
    /// The default is false.
    pub fn show_when_disabled(mut self) -> Self {
        self.show_when_disabled = true;
        self
    }

    /// Injects the this widgets internal data (the data before this widget got disabled, if it was
    /// never enabled this is initial data) into data.
    /// If data was this widgets external data, the widget will get enabled
    /// during the next update call.
    ///
    /// You can use this method from a parent widget (Controller or similar) to enable
    /// this widget, since it has no means to do it itself.
    pub fn enable<T>(&self, data: &mut T) where
        P: Prism<T, U>
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

impl<W, T, U, P> Widget<T> for PartialWidget<W, U, P> where
    U: Data,
    W: Widget<U>,
    P: Prism<T, U>
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if self.enabled {
            let mut new_data = self.current_data.clone();

            self.widget.event(ctx, event, &mut new_data, env);

            if !new_data.same(&self.current_data) {
                self.prism.put(data, new_data);
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        //TODO: decide which lifecycle events should get send when the widget is disabled
        if env.get(Env::DEBUG_WIDGET) {
            let lifecycle_event = event;
            dbg!(lifecycle_event);
        }
        match event {
            LifeCycle::WidgetAdded => {
                if let Some(data) = self.prism.get(data) {
                    self.enabled = true;
                    self.current_data = data;
                } else {
                    self.enabled = false;
                }
            }
            _ => {}
        }

        self.widget.lifecycle(ctx, event, &self.current_data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        let was_enabled = self.enabled;

        if let Some(data) = self.prism.get(data) {
            self.enabled = true;

            if !data.same(&self.current_data) {
                self.widget.update(ctx, &self.current_data, &data, env);

                self.current_data = data;
            }

            if !was_enabled && !self.show_when_disabled {
                //Widget was hidden and will be visible
                ctx.request_layout();
            }

        } else {
            self.enabled = false;

            //enabled changed
            if was_enabled && !self.show_when_disabled {
                //Widget was visible and will be hidden
                ctx.request_layout();
            }
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &T, env: &Env) -> Size {
        if self.enabled || self.show_when_disabled {
            self.widget.layout(ctx, bc, &self.current_data, env)
        } else {
            bc.min()
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &T, env: &Env) {
        if self.enabled || self.show_when_disabled {
            //TODO: signal that the widget is disabled
            self.widget.paint(ctx, &self.current_data, env);
        }
    }
}

//TODO: Maybe write a derive macro
///A trait similar to druid::Lens that represents data which is not always present
pub trait Prism<T, U> {
    ///Extract the data (if present) from a move general type
    fn get(&self, data: &T) -> Option<U>;
    ///Store the data back in the original type
    fn put(&self, data: &mut T, inner: U);
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

impl<F, G, T, U> Prism<T, U> for Closures<F, G> where
    F: Fn(&T) -> Option<U>,
    G: Fn(&mut T, U)
{
    fn get(&self, data: &T) -> Option<U> {
        (self.0)(data)
    }

    fn put(&self, data: &mut T, inner: U) {
        (self.1)(data, inner);
    }
}