use druid::widget::prelude::*;
use druid::widget::ControllerHost;
use druid::{Selector, WidgetExt as _};

use crate::on_cmd::OnCmd;
use crate::OnChange;

pub trait WidgetExt<T: Data>: Widget<T> + Sized + 'static {
    fn on_command<CT: 'static>(
        self,
        selector: Selector<CT>,
        handler: impl Fn(&mut EventCtx, &CT, &mut T) + 'static,
    ) -> ControllerHost<Self, OnCmd<CT, T>> {
        self.controller(OnCmd::new(selector, handler))
    }

    /// Calls the function when data changes **in a child widget**
    ///
    /// `&T` is the old data and `&mut T` is the new data
    fn on_change(
        self,
        f: impl Fn(&mut EventCtx, &T, &mut T, &Env) + 'static,
    ) -> ControllerHost<Self, OnChange<T>> {
        self.controller(OnChange::new(f))
    }
}

impl<T: Data, W: Widget<T> + 'static> WidgetExt<T> for W {}
