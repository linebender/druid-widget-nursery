use druid::widget::Controller;
use druid::widget::prelude::*;

pub struct AutoFocus;
impl<W: Widget<T>, T> Controller<T, W> for AutoFocus {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Event::WindowConnected = event {
            ctx.request_focus()
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
        if let LifeCycle::BuildFocusChain = event {
            ctx.register_for_focus()
        }
        child.lifecycle(ctx, event, data, env)
    }
}
