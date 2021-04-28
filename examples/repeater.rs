use std::time::{SystemTime, UNIX_EPOCH};

use druid::{
    im, AppLauncher, BoxConstraints, Color, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle,
    LifeCycleCtx, PaintCtx, Point, RenderContext, Size, UpdateCtx, Widget, WidgetPod, WindowDesc,
};

use druid_widget_nursery::Repeater;

#[derive(Clone, Data)]
pub struct WindowData {
    pub origin: Point,
    pub size: Size,
    pub id: u64,
}

#[derive(Clone, Data, Lens)]
pub struct AppState {
    windows: im::Vector<WindowData>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            windows: im::Vector::new(),
        }
    }
}

fn main_widget() -> impl Widget<AppState> {
    // TODO: Builder pattern
    RepeaterExample {
        content: WidgetPod::new(Box::new(Repeater::new(
            AppState::windows,
            Box::new(|window: &WindowData| window.id),
            Box::new(|_window: &WindowData| Window),
        ))),
    }
}

pub fn main() {
    let main_window = WindowDesc::new(main_widget())
        .title("Repeater")
        .window_size((800.0, 600.0));

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(AppState::new())
        .expect("Failed to launch application");
}

pub struct Window;

impl Widget<WindowData> for Window {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut WindowData, _env: &Env) {}

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &WindowData,
        _env: &Env,
    ) {
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx,
        _old_data: &WindowData,
        _data: &WindowData,
        _env: &Env,
    ) {
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &WindowData,
        _env: &Env,
    ) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &WindowData, _env: &Env) {
        let size = ctx.size();
        let rect = size.to_rect();
        ctx.fill(rect, &Color::rgba(1., 1., 1., 0.1));
    }
}

struct RepeaterExample {
    pub content: WidgetPod<AppState, Box<dyn Widget<AppState>>>,
}

impl Widget<AppState> for RepeaterExample {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, env: &Env) {
        self.content.event(ctx, event, data, env);

        if let Event::MouseDown(_) = event {
            let start = SystemTime::now();
            let since_the_epoch = start
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            data.windows.push_back(WindowData {
                id: since_the_epoch.as_millis() as u64,
                origin: Point::new(0., 0.),
                size: Size::new(100., 100.),
            });
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &AppState, env: &Env) {
        self.content.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &AppState, data: &AppState, env: &Env) {
        self.content.update(ctx, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &AppState,
        env: &Env,
    ) -> Size {
        let _content_size = self.content.layout(ctx, bc, data, env);
        self.content.set_origin(ctx, data, env, Point::new(0., 0.));

        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, env: &Env) {
        self.content.paint(ctx, data, env);
    }
}
