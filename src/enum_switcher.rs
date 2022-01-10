use crate::prism::{Prism, PrismWidget, PrismWrap};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Size,
    UpdateCtx, Widget,
};

type WidgetBuilder<T> = Box<dyn Fn(&T) -> Option<Box<dyn PrismWidget<T>>>>;

/// A widget like switcher, but the inner widgets are created on demand. This is useful for tree-like
/// structures, which you can't represent with Switcher recursively.
pub struct LazySwitcher<T: Data> {
    builder: Vec<WidgetBuilder<T>>,
    current: Option<Box<dyn PrismWidget<T>>>,
}

impl<T: Data> LazySwitcher<T> {
    pub fn new() -> Self {
        LazySwitcher {
            builder: vec![],
            current: None,
        }
    }

    /// Adds a new variant to the widget. This variant is show as long as the prism returns `Some()`
    /// for the current data.
    pub fn with_variant<U: Data, P: Prism<T, U> + Clone + 'static, W: Widget<U> + 'static>(
        mut self,
        prism: P,
        builder: impl Fn() -> W + 'static,
    ) -> Self {
        self.builder.push(Box::new(move |data| {
            prism
                .get(data)
                .map(|_| Box::new(PrismWrap::new(builder(), prism.clone())) as _)
        }));
        self
    }

    /// updates the inner widget and returns true if the widget changed
    fn rebuild_if_needed(&mut self, data: &T) -> bool {
        if let Some(current) = &self.current {
            if current.is_active_for(data) {
                return false;
            }
        }

        let had_child = self.current.is_none();
        let new = self
            .builder
            .iter()
            .filter_map(|builder| (builder)(data))
            .next();
        self.current = new;

        had_child || self.current.is_some()
    }
}

impl<T: Data> Default for LazySwitcher<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Data> Widget<T> for LazySwitcher<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Some(inner) = &mut self.current {
            inner.event(ctx, event, data, env);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            self.rebuild_if_needed(data);
        }

        if let Some(inner) = &mut self.current {
            inner.lifecycle(ctx, event, data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        if self.rebuild_if_needed(data) {
            ctx.children_changed();
            ctx.request_layout();
        }

        if let Some(inner) = &mut self.current {
            inner.update(ctx, old_data, data, env);
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        if let Some(inner) = &mut self.current {
            inner.layout(ctx, bc, data, env)
        } else {
            bc.min()
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        if let Some(inner) = &mut self.current {
            inner.paint(ctx, data, env);
        }
    }
}

/// A widget which displays the first widget of which the associated prism returned `Some()` for the
/// current data.
pub struct Switcher<T: Data> {
    widgets: Vec<Box<dyn PrismWidget<T>>>,
    current: Option<usize>,
}

impl<T: Data> Switcher<T> {
    pub fn new() -> Self {
        Switcher {
            widgets: vec![],
            current: None,
        }
    }

    /// Adds a new variant to the widget. This variant is show as long as the prism returns `Some()`
    /// for the current data.
    pub fn with_variant<U: Data, P: Prism<T, U> + 'static>(
        mut self,
        prism: P,
        widget: impl Widget<U> + 'static,
    ) -> Self {
        self.widgets.push(Box::new(PrismWrap::new(widget, prism)));
        self
    }

    /// updates the inner widget and returns true if the widget changed
    fn rebuild_if_needed(&mut self, data: &T) -> bool {
        if let Some(current) = self.current {
            //In most cases the variant stays the same, therefore we check the current widget first.
            if self.widgets[current].is_active_for(data) {
                return false;
            }
        }

        let old = self.current;
        let new = self
            .widgets
            .iter()
            .position(|widget| widget.is_active_for(data));
        self.current = new;
        old != self.current
    }
}

impl<T: Data> Default for Switcher<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Data> Widget<T> for Switcher<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        for (index, child) in self.widgets.iter_mut().enumerate() {
            if event.should_propagate_to_hidden() || self.current == Some(index) {
                child.event(ctx, event, data, env);
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            self.rebuild_if_needed(data);
        }
        for (index, child) in self.widgets.iter_mut().enumerate() {
            if event.should_propagate_to_hidden() || self.current == Some(index) {
                child.lifecycle(ctx, event, data, env);
            }
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        if self.rebuild_if_needed(data) {
            ctx.request_layout();
            ctx.children_changed();
        }

        if let Some(index) = self.current {
            self.widgets[index].update(ctx, old_data, data, env);
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        if let Some(index) = self.current {
            self.widgets[index].layout(ctx, bc, data, env)
        } else {
            bc.min()
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        if let Some(index) = self.current {
            self.widgets[index].paint(ctx, data, env);
        }
    }
}
