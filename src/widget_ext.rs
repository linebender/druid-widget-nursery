use druid::{widget::ControllerHost, Data, EventCtx, Selector, Widget, WidgetExt as _};

use crate::on_cmd::OnCmd;

pub trait WidgetExt<T: Data>: Widget<T> + Sized + 'static {
    fn on_command<CT: 'static>(
        self,
        selector: Selector<CT>,
        handler: impl Fn(&mut EventCtx, &CT, &mut T) + 'static,
    ) -> ControllerHost<Self, OnCmd<CT, T>> {
        self.controller(OnCmd::new(selector, handler))
    }
}

impl<T: Data, W: Widget<T> + 'static> WidgetExt<T> for W {}
