use druid::widget::prelude::*;
use druid::widget::Controller;
use druid::Selector;

type HandlerFn<CT, WT> = Box<dyn Fn(&mut EventCtx, &CT, &mut WT)>;

pub struct OnCmd<CT, WT> {
    selector: Selector<CT>,
    handler: HandlerFn<CT, WT>,
}

impl<CT, WT> OnCmd<CT, WT> {
    pub fn new(
        selector: Selector<CT>,
        handler: impl Fn(&mut EventCtx, &CT, &mut WT) + 'static,
    ) -> Self {
        Self {
            selector,
            handler: Box::new(handler),
        }
    }
}

impl<WT: Data, W: Widget<WT>, CT: 'static> Controller<WT, W> for OnCmd<CT, WT> {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut WT,
        env: &Env,
    ) {
        match event {
            Event::Command(c) if c.is(self.selector) => {
                (self.handler)(ctx, c.get_unchecked(self.selector), data);
            }
            _ => {}
        }
        child.event(ctx, event, data, env);
    }
}
