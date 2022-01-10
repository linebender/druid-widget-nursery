use druid::widget::prelude::*;
use druid::widget::{ControllerHost, LabelText};
use druid::{Selector, WidgetExt as _, WindowHandle};

use crate::on_cmd::OnCmd;
use crate::tooltip::TooltipState;
use crate::{OnChange, OnMonitor, TooltipController};

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

    /// Open a tooltip when the mouse is hovered over this widget.
    fn tooltip<LT: Into<LabelText<T>>>(
        self,
        text: LT,
    ) -> ControllerHost<Self, TooltipController<T>> {
        self.controller(TooltipController {
            text: text.into(),
            state: TooltipState::Off,
        })
    }

    /// A convenience method for ensuring that this widget is fully visible on the same monitor as
    /// some other window.
    fn on_monitor(self, parent: &WindowHandle) -> OnMonitor<Self> {
        OnMonitor {
            inner: self,
            parent: parent.clone(),
        }
    }
}

impl<T: Data, W: Widget<T> + 'static> WidgetExt<T> for W {}
