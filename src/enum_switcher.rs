use crate::prism::{Prism, PrismWidget, PrismWrap};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Size,
    UpdateCtx, Widget, WidgetPod,
};

pub struct LazySwitcher<T: Data> {
    builder: Vec<Box<dyn Fn(&T) -> Option<Box<dyn PrismWidget<T>>>>>,
    current: Option<WidgetPod<T, Box<dyn PrismWidget<T>>>>,
}

impl<T: Data> LazySwitcher<T> {
    pub fn new() -> Self {
        LazySwitcher {
            builder: vec![],
            current: None,
        }
    }
    pub fn with_variant<U: Data, P: Prism<T, U> + Clone, W: Widget<U>>(
        mut self,
        prism: P,
        builder: impl Fn() -> W,
    ) -> Self {
        self.builder.push(Box::new(move |data| {
            prism
                .get(data)
                .map(|_| Box::new(PrismWrap::new(builder(), prism.clone())))
        }));
        self
    }

    fn rebuild(&mut self, data: &T) -> bool {
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

impl<T: Data> Widget<T> for LazySwitcher<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Some(inner) = &mut self.current {
            inner.event(ctx, event, data, env);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            self.rebuild(data);
        }

        if let Some(inner) = &mut self.current {
            inner.lifecycle(ctx, event, data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _: &T, data: &T, env: &Env) {
        if let Some(inner) = &mut self.current {
            if inner.widget().is_active_for(data) {
                inner.update(ctx, data, env);
            } else {
                if self.rebuild(data) {
                    ctx.children_changed();
                    ctx.request_layout();
                }
            }
        } else {
            if self.rebuild(data) {
                ctx.children_changed();
                ctx.request_layout();
            }
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        if let Some(inner) = &mut self.current {
            let size = inner.layout(ctx, bc, data, env);
            ctx.set_baseline_offset(inner.baseline_offset());
            size
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

pub struct Switcher<T: Data> {
    widgets: Vec<WidgetPod<T, Box<dyn PrismWidget<T>>>>,
    current: Option<usize>,
}

impl<T: Data> Switcher<T> {
    pub fn new() -> Self {
        Switcher {
            widgets: vec![],
            current: None,
        }
    }
    pub fn with_variant<U: Data, P: Prism<T, U>>(
        mut self,
        prism: P,
        widget: impl Widget<U>,
    ) -> Self {
        self.widgets
            .push(WidgetPod::new(Box::new(PrismWrap::new(widget, prism))));
        self
    }
    pub fn rebuild(&mut self, data: &T) -> bool {
        let old = self.current;
        let new = self
            .widgets
            .iter()
            .position(|widget| widget.widget().is_active_for(data));
        self.current = new;
        old != self.current
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
            self.rebuild(data);
        }
        for (index, child) in self.widgets.iter_mut().enumerate() {
            if event.should_propagate_to_hidden() || self.current == Some(index) {
                child.lifecycle(ctx, event, data, env);
            }
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _: &T, data: &T, env: &Env) {
        if let Some(index) = self.current {
            if !self.widgets[index].widget().is_active_for(data) {
                if self.rebuild(data) {
                    ctx.children_changed();
                    ctx.request_layout();
                }
            }
        } else {
            if self.rebuild(data) {
                ctx.children_changed();
                ctx.request_layout();
            }
        }
        if let Some(index) = self.current {
            self.widgets[index].update(ctx, data, env);
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        if let Some(index) = self.current {
            let size = self.widgets[index].layout(ctx, bc, data, env);
            ctx.set_baseline_offset(self.widgets[index].baseline_offset());
            size
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
