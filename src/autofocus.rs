use druid::widget::prelude::*;
use druid::{widget::Controller, Command, Selector};

const TAKE_FOCUS: Selector<()> = Selector::new("auto_focus.take_focus");

pub struct AutoFocus;

impl<W: Widget<T>, T> Controller<T, W> for AutoFocus {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Event::Command(cmd) = event {
            if cmd.is(TAKE_FOCUS) {
                ctx.request_focus();
            }
        }

        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &T,
        env: &Env,
    ) {
        match event {
            LifeCycle::BuildFocusChain => ctx.register_for_focus(),
            LifeCycle::WidgetAdded => {
                ctx.submit_command(Command::new(TAKE_FOCUS, (), ctx.widget_id()))
            }
            _ => (),
        }

        child.lifecycle(ctx, event, data, env)
    }
}
