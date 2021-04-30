use std::time::{SystemTime, UNIX_EPOCH};

use druid::{
    im, AppLauncher, BoxConstraints, Color, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle,
    LifeCycleCtx, LinearGradient, PaintCtx, Point, Rect, RenderContext, Selector, Size, UnitPoint,
    UpdateCtx, Widget, WidgetPod, WindowDesc,
};

use druid_widget_nursery::Repeater;

const CLOSE_WINDOW: Selector<u64> = Selector::new("repeater-example.close-window");

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
            Box::new(|widgets, ctx, _bc, data: &AppState, env| {
                for i in 0..widgets.len() {
                    let widget = widgets[i].widget();
                    let widget_data = &data.windows;
                    let window = &widget_data[i];

                    let _widget_size = widget.layout(
                        ctx,
                        &BoxConstraints::tight((window.size.width, window.size.height).into()),
                        widget_data,
                        env,
                    );
                    widget.set_origin(
                        ctx,
                        widget_data,
                        env,
                        (window.origin.x, window.origin.y).into(),
                    );
                }
            }),
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
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut WindowData, _env: &Env) {
        if let Event::MouseDown(e) = event {
            ctx.set_handled();

            // close button
            if (e.pos.x - 14.).powi(2) + (e.pos.y - 14.).powi(2) <= 144. {
                ctx.submit_notification(CLOSE_WINDOW.with(data.id));
            }
        }
    }

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
        let rect = size.to_rounded_rect(8.);
        ctx.fill(rect, &Color::rgba(1., 1., 1., 1.));
        ctx.stroke(rect, &Color::rgba(0., 0., 0., 1.), 1.);

        let button_size = 12.;
        let button_padding = 8.;

        // Window buttons
        let red_button =
            Rect::from_origin_size((button_padding, button_padding), (button_size, button_size))
                .to_ellipse();
        ctx.fill(red_button, &Color::Rgba32(0xeb514aff));
        ctx.stroke(red_button, &Color::Rgba32(0xd14138ff), 0.5);

        let yellow_button = Rect::from_origin_size(
            (button_size + button_padding * 2., button_padding),
            (button_size, button_size),
        )
        .to_ellipse();
        ctx.fill(yellow_button, &Color::Rgba32(0xf3bd50ff));
        ctx.stroke(yellow_button, &Color::Rgba32(0xd7a13fff), 0.5);

        let green_button = Rect::from_origin_size(
            (button_size * 2. + button_padding * 3., button_padding),
            (button_size, button_size),
        )
        .to_ellipse();
        ctx.fill(green_button, &Color::Rgba32(0x479d36ff));
        ctx.stroke(green_button, &Color::Rgba32(0x3f822bff), 0.5);
    }
}

struct RepeaterExample {
    pub content: WidgetPod<AppState, Box<dyn Widget<AppState>>>,
}

impl Widget<AppState> for RepeaterExample {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, env: &Env) {
        self.content.event(ctx, event, data, env);

        match event {
            Event::MouseDown(e) => {
                if ctx.is_handled() {
                    return;
                }
                let start = SystemTime::now();
                let since_the_epoch = start
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards");
                data.windows.push_back(WindowData {
                    id: since_the_epoch.as_millis() as u64,
                    origin: Point::new(e.pos.x, e.pos.y),
                    size: Size::new(300., 300.),
                });
            }
            Event::Notification(notification) => {
                if notification.is(CLOSE_WINDOW) {
                    let id = notification.get(CLOSE_WINDOW).unwrap();
                    let mut index_to_remove = 0usize;
                    for i in 0..data.windows.len() {
                        if data.windows[i].id == *id {
                            index_to_remove = i;
                            break;
                        }
                    }
                    data.windows.remove(index_to_remove);
                }
            }
            _ => {}
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
        let rect = ctx.size().to_rect();
        let gradient = LinearGradient::new(
            UnitPoint::LEFT,
            UnitPoint::RIGHT,
            (
                Color::Rgba32(0x8a2387ff),
                Color::Rgba32(0xe94057ff),
                Color::Rgba32(0xf27121ff),
            ),
        );
        ctx.fill(rect, &gradient);

        self.content.paint(ctx, data, env);
    }
}
