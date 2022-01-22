use druid::commands::CLOSE_WINDOW;
use druid::widget::prelude::*;
use druid::widget::Controller;
use druid::widget::WidgetExt;
use druid::Target;
use druid::WindowSizePolicy;
use druid::{Point, WindowConfig};
use druid::{WindowId, WindowLevel};

pub struct Dropdown<T> {
    drop: Box<dyn Fn(&T, &Env) -> Box<dyn Widget<T>>>,
    window: Option<WindowId>,
}

crate::selectors! {
    DROPDOWN_SHOW,
    DROPDOWN_HIDE,
    DROPDOWN_CLOSED,
}

impl<T: Data> Dropdown<T> {
    pub fn new<W: 'static + Widget<T>, DW: Widget<T> + 'static>(
        header: W,
        make_drop: impl Fn(&T, &Env) -> DW + 'static,
    ) -> impl Widget<T> {
        // padding for putting header in separate WidgetPod
        // because notifications from same WidgetPod are not sent
        header.padding(0.).controller(Dropdown {
            drop: Box::new(move |d, e| make_drop(d, e).boxed()),
            window: None,
        })
    }

    fn show_dropdown(&mut self, data: &mut T, env: &Env, ctx: &mut EventCtx) {
        let widget = (self.drop)(data, env);
        let mut origin = ctx.to_window(Point::new(0., ctx.size().height));

        let insets = ctx.window().content_insets();
        origin.x += insets.x0;
        origin.y += insets.y0;

        self.window = Some(
            ctx.new_sub_window(
                WindowConfig::default()
                    .set_level(WindowLevel::DropDown(ctx.window().clone()))
                    .set_position(origin)
                    .window_size_policy(WindowSizePolicy::Content)
                    .resizable(false)
                    .show_titlebar(false),
                widget.controller(DropedCtrl {
                    parent: ctx.widget_id(),
                }),
                data.clone(),
                env.clone(),
            ),
        );
        ctx.set_active(true);
    }
}

struct DropedCtrl {
    parent: WidgetId,
}

impl<T, W: Widget<T>> Controller<T, W> for DropedCtrl {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Event::WindowDisconnected = event {
            ctx.submit_command(DROPDOWN_CLOSED.to(self.parent));
        }
        child.event(ctx, event, data, env);
    }
}

impl<T: Data, W: Widget<T>> Controller<T, W> for Dropdown<T> {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        match event {
            Event::Command(c) if c.is(DROPDOWN_SHOW) && self.window.is_none() => {
                self.show_dropdown(data, env, ctx);
                ctx.set_handled();
            }
            Event::Notification(n) if n.is(DROPDOWN_SHOW) && self.window.is_none() => {
                self.show_dropdown(data, env, ctx);
                ctx.set_handled();
            }
            Event::Command(cmd) if cmd.is(DROPDOWN_CLOSED) => {
                ctx.set_active(false);
                self.window = None;
                let inner_cmd = cmd.clone().to(Target::Global);
                // send DROP_END to header
                child.event(ctx, &Event::Command(inner_cmd), data, env);
                ctx.set_handled();
            }

            Event::Command(cmd) if cmd.is(DROPDOWN_HIDE) => {
                if let Some(w) = self.window {
                    ctx.submit_command(CLOSE_WINDOW.to(w));
                }
                ctx.set_handled();
            }

            Event::Notification(cmd) if cmd.is(DROPDOWN_HIDE) => {
                if let Some(w) = self.window {
                    ctx.submit_command(CLOSE_WINDOW.to(w));
                }
                ctx.set_handled();
            }

            // we recieve global mouse downs when widget is_active
            // close on any outside mouse click
            Event::MouseDown(ev) if ctx.is_active() && !ctx.size().to_rect().contains(ev.pos) => {
                if let Some(w) = self.window {
                    ctx.submit_command(CLOSE_WINDOW.to(w));
                }
            }
            _ => {}
        }
        child.event(ctx, event, data, env);
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &T,
        env: &Env,
    ) {
        child.lifecycle(ctx, event, data, env)
    }
}
