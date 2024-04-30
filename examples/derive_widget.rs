use druid::{
    widget::{Button, Flex, Label},
    AppLauncher, Data, Env, Lens, PaintCtx, UpdateCtx, Widget, WidgetExt,
    WidgetPod, WindowDesc, WindowSizePolicy,
};

#[cfg(feature = "derive")]
pub use druid_widget_nursery_derive::Widget;

#[derive(Widget)]
#[widget(widget_pod = 1, paint = paint_impl, update = update_impl)]
pub struct InvisibleIf<T, W>(Box<dyn Fn(&T) -> bool>, WidgetPod<T, W>);

impl<T: Data, W: Widget<T>> InvisibleIf<T, W> {
    pub fn new(test: impl Fn(&T) -> bool + 'static, widget: W) -> Self {
        Self(Box::new(test), WidgetPod::new(widget))
    }

    fn paint_impl(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        if !self.0(data) {
            self.1.paint(ctx, data, env)
        }
    }

    fn update_impl(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        if !old_data.same(data) {
            ctx.request_paint()
        }
        self.1.update(ctx, data, env)
    }
}

#[derive(Clone, Data, Lens)]
struct AppState {
    render_widget: bool,
}

fn ui_builder() -> impl Widget<AppState> {
    Flex::column()
        .with_child(
            Button::new("Toggle render")
                .on_click(|_, data: &mut AppState, _| {
                    data.render_widget = !data.render_widget;
                })
                .padding(5.),
        )
        .with_child(
            InvisibleIf::new(
                |data: &AppState| !data.render_widget,
                Label::new("Rendered"),
            )
            .padding(10.),
        )
}

pub fn main() {
    let main_window = WindowDesc::new(ui_builder())
        .window_size_policy(WindowSizePolicy::Content)
        .title("Load Widget Derive example");

    let state = AppState {
        render_widget: true,
    };

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(state)
        .expect("launch failed");
}
