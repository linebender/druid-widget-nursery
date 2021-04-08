use std::{cmp::Ordering, collections::HashMap, fmt, hash::Hash, unreachable};

use druid::{widget::prelude::*, Point, WidgetPod};

/// This widget navigates through the widgets it stores using the Application Data
/// to manage which widget is currently in view. This most likely will be the root
/// widget or a root widget for your application.
///
/// It requires the Application state to have a backing data structure that the
/// navigator will use to update its child widgets.
pub struct Navigator<T, H> {
    state: Vec<WidgetPod<T, Box<dyn Widget<T>>>>,
    views: Views<H, T>,
}
type Views<H, T> = HashMap<H, Box<dyn Fn() -> Box<dyn Widget<T>>>>;

impl<T: Data, H: View> Navigator<T, H> {
    /// Creates new Navigator widget with the initial view.
    pub fn new(name: H, ui_builder: impl Fn() -> Box<dyn Widget<T>> + 'static) -> Self {
        let mut views = Vec::new();
        let current_view = (ui_builder)();
        let current_view = WidgetPod::new(current_view);
        views.push(current_view);
        let mut this = Self {
            state: views,
            views: HashMap::new(),
        };
        if this.views.insert(name, Box::new(ui_builder)).is_some() {
            unreachable!("Map should be empty at this point");
        }
        this
    }

    /// Takes a function to build a widget and a name that will be used to refer to it.
    pub fn with_view_builder(
        mut self,
        name: H,
        view_builder: impl Fn() -> Box<dyn Widget<T>> + 'static,
    ) -> Self {
        if self.views.insert(name, Box::new(view_builder)).is_some() {
            log::warn!("Views should never update. They should be set at navigator creation and never change.");
        }
        self
    }

    /// Pushes a new view into navigator's state to be displayed
    fn push_view(&mut self, view: H) {
        let ui_builder = self.views.get(&view).unwrap();
        let new_view = (ui_builder)();
        let widget = WidgetPod::new(new_view);
        self.state.push(widget);
    }

    /// Removes a view from navigator's state
    fn truncate_views(&mut self, new_len: usize) {
        if self.state.len() == 1 {
            log::warn!("The view state should always have at least one child view");
        }
        self.state.truncate(new_len);
    }
}
/// This gives your Application State the behavior necessary to manipulate its views.
///
/// You will want to have your AppState or any State implement this so the navigator
/// can get relevant state to update its child widgets.
pub trait ViewController<T: Hash + PartialEq + Eq + Clone> {
    /// Pushes a new view to be displayed.
    fn add_view(&mut self, view: T);
    /// Removes the current view from display.
    fn pop_view(&mut self);
    /// Gets the current view that is being displayed.
    fn current_view(&self) -> &T;
    /// Gets the length of the backing View data structure.
    ///
    /// Views will probably be backed by some kind of array.
    fn len(&self) -> usize;
    // figure out why I have this here
    fn is_empty(&self) -> bool;
}

/// A view will act as representation for the child widget within Navigator.
pub trait View: Hash + PartialEq + Eq + Clone + fmt::Debug {}

impl<H: View, T: Data + ViewController<H>> Widget<T> for Navigator<T, H> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        // each child view gets to handle the event before a view might be popped
        if event.should_propagate_to_hidden() {
            for view in self.state.iter_mut() {
                view.event(ctx, event, data, env);
            }
        } else {
            self.state.last_mut().unwrap().event(ctx, event, data, env);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            if data.is_empty() && !self.state.is_empty() {
                log::warn!("The data backing the Navigator widget is empty. It must not be empty on initialization.");
            }
            ctx.children_changed();
        }
        if event.should_propagate_to_hidden() {
            for view in self.state.iter_mut() {
                view.lifecycle(ctx, event, data, env);
            }
        } else {
            self.state
                .last_mut()
                .unwrap()
                .lifecycle(ctx, event, data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        match data.len().cmp(&old_data.len()) {
            Ordering::Greater => {
                self.push_view(data.current_view().clone());
                ctx.children_changed();
            }
            Ordering::Less => {
                self.truncate_views(data.len());
                ctx.children_changed();
            }
            Ordering::Equal => {}
        }
        let current_view = self.state.last_mut().unwrap();

        if current_view.is_initialized() {
            current_view.update(ctx, data, env);
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let current_view = self.state.last_mut().unwrap();
        let child_size = current_view.layout(ctx, bc, data, env);
        // I think the origin is (0,0) which should be the top left corner of the parent
        current_view.set_origin(ctx, data, env, Point::ORIGIN);

        child_size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.state.last_mut().unwrap().paint(ctx, data, env)
    }
}
