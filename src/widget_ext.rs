use druid::widget::prelude::*;
use druid::widget::{ControllerHost, LabelText};
use druid::{Point, Selector, WidgetExt as _, WindowHandle};

use crate::on_cmd::OnCmd;
use crate::stack_tooltip::{StackTooltip, ADVISE_TOOLTIP_SHOW, CANCEL_TOOLTIP_SHOW, PlainOrRich};
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

    /// A convenience method to cancel the display of a tooltip from a parent/ancestor widget.
    fn cancel_stack_tooltip(self) -> ControllerHost<Self, OnCmd<Point, T>> {
        self.controller(OnCmd::new(ADVISE_TOOLTIP_SHOW, move |ctx, point, _| {
            let window_rect = ctx.size().to_rect().with_origin(ctx.window_origin());
            if window_rect.contains(*point) {
                ctx.submit_notification(CANCEL_TOOLTIP_SHOW)
            }
        }))
    }

    /// Open a stack based tooltip when the mouse is hovered over this widget
    fn stack_tooltip(self, label: impl Into<PlainOrRich>) -> StackTooltip<T> {
        StackTooltip::new(self, label)
    }
}

impl<T: Data, W: Widget<T> + 'static> WidgetExt<T> for W {}
