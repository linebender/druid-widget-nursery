use druid::{Data, Widget, EventCtx, LifeCycle, PaintCtx, LifeCycleCtx, BoxConstraints, Size, LayoutCtx, Event, Env, UpdateCtx, WidgetPod};
use crate::prism::{Prism, PrismWidget, PrismWidgetImpl};

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
    pub fn with_variant<U: Data, P: Prism<T, U> + Clone, W: Widget<U>>(mut self, prism: P, builder: impl Fn() -> W) -> Self {
        self.builder.push(Box::new(move|data|{
            prism.get(data).map(|_|Box::new(PrismWidgetImpl {
                inner: builder(),
                prism: prism.clone(),
                cached_data: None
            }))
        }));
        self
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
    pub fn with_variant<U: Data, P: Prism<T, U>>(mut self, prism: P, widget: impl Widget<U>) -> Self {
        self.widgets.push(WidgetPod::new(Box::new(PrismWidgetImpl {
            inner: widget,
            prism,
            cached_data: None
        })));
        self
    }
}