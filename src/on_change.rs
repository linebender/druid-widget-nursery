use druid::widget::prelude::*;
use druid::widget::Controller;

pub struct OnChange<T>(Box<dyn Fn(&mut EventCtx, &T, &mut T, &Env)>);

impl<T> OnChange<T> {
    pub fn new(f: impl Fn(&mut EventCtx, &T, &mut T, &Env) + 'static) -> Self {
        Self(Box::new(f))
    }
}

impl<T: Data, W: Widget<T>> Controller<T, W> for OnChange<T> {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        let old_data = data.clone();
        child.event(ctx, event, data, env);
        if !old_data.same(data) {
            (self.0)(ctx, &old_data, data, env);
        }
    }
}
