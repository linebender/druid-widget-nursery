use super::{hot_reload_lib::HotReloadLib, RELOAD};
use druid::{widget::prelude::*, WidgetPod};

pub struct HotReloaderWidget<T> {
    pub(super) lib: HotReloadLib,
    pub(super) inner: Option<WidgetPod<T, Box<dyn Widget<T>>>>,
    pub(super) view_fn_name: &'static str,
}

impl<T> HotReloaderWidget<T> {
    fn update_lib(&mut self) {
        // droping it before unloading the library
        drop(self.inner.take());
        self.lib.update();
        let load = unsafe {
            self.lib
                .load_symbol::<fn() -> Box<dyn Widget<T>>>(self.view_fn_name)
                .unwrap()
        };
        let returned_widget = load();
        self.inner = Some(WidgetPod::new(returned_widget));
    }
}

impl<T: Data> Widget<T> for HotReloaderWidget<T> {
    fn event(&mut self, ctx: &mut druid::EventCtx, event: &Event, data: &mut T, env: &druid::Env) {
        if let Event::Command(cmd) = event {
            if cmd.is(RELOAD) {
                self.update_lib();
                ctx.children_changed();
                return;
            }
        }
        self.inner.as_mut().unwrap().event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &LifeCycle,
        data: &T,
        env: &druid::Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            // just update library first time
            if self.inner.is_none() {
                self.update_lib();
                ctx.children_changed();
                ctx.request_layout();
            }
        }
        self.inner
            .as_mut()
            .unwrap()
            .lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, _old_data: &T, data: &T, env: &druid::Env) {
        self.inner.as_mut().unwrap().update(ctx, data, env)
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &T,
        env: &druid::Env,
    ) -> druid::Size {
        self.inner
            .as_mut()
            .unwrap()
            .set_origin(ctx, data, env, (0.0, 0.0).into());
        self.inner.as_mut().unwrap().layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &T, env: &druid::Env) {
        self.inner.as_mut().unwrap().paint(ctx, data, env)
    }
}
