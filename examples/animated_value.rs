use druid::{
    AppLauncher, BoxConstraints, Color, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx,
    PaintCtx, RenderContext, Size, UnitPoint, UpdateCtx, Widget, WidgetExt, WindowDesc,
};
use druid_widget_nursery::animation::{Animated, SimpleCurve};
use std::time::Duration;

static COLORS: [Color; 12] = [
    Color::RED,
    Color::GREEN,
    Color::LIME,
    Color::MAROON,
    Color::BLACK,
    Color::NAVY,
    Color::PURPLE,
    Color::TEAL,
    Color::OLIVE,
    Color::YELLOW,
    Color::BLUE,
    Color::WHITE,
];

struct AnimatedWidget {
    color: Animated<Color>,
    insets: Animated<f64>,
    current_color: usize,
}

impl AnimatedWidget {
    pub fn new() -> Self {
        AnimatedWidget {
            color: Animated::new(
                Color::RED,
                Duration::from_secs_f64(0.8),
                SimpleCurve::EaseInOut,
                false,
            ),
            insets: Animated::new(
                6.0,
                Duration::from_secs_f64(0.2),
                SimpleCurve::EaseOut,
                false,
            ),
            current_color: 0,
        }
    }
    pub fn set_insets(&mut self, hot: bool, active: bool) {
        let insets = if hot {
            if active {
                6.0
            } else {
                1.0
            }
        } else {
            3.0
        };
        self.insets.animate(insets);
    }
}

impl Widget<()> for AnimatedWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _: &mut (), _: &Env) {
        match event {
            Event::MouseDown(_) => {
                ctx.set_active(true);
                ctx.request_anim_frame();
                self.set_insets(ctx.is_hot(), true);
            }
            Event::MouseUp(_) => {
                if ctx.is_hot() {
                    self.current_color += 1;
                    self.color
                        .animate(COLORS[self.current_color % COLORS.len()].clone());
                }
                ctx.set_active(false);
                ctx.request_anim_frame();
                self.set_insets(ctx.is_hot(), false);
            }
            Event::AnimFrame(nanos) => {
                self.insets.update(*nanos, ctx);
                self.color.update(*nanos, ctx);
            }
            _ => {}
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _: &(), _: &Env) {
        if let LifeCycle::HotChanged(hot) = event {
            ctx.request_anim_frame();
            self.set_insets(*hot, ctx.is_active());
        }
    }

    fn update(&mut self, _: &mut UpdateCtx, _: &(), _: &(), _: &Env) {}

    fn layout(&mut self, _: &mut LayoutCtx, bc: &BoxConstraints, _: &(), _: &Env) -> Size {
        bc.constrain((200.0, 80.0))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _: &(), _: &Env) {
        let shape = ctx
            .size()
            .to_rect()
            .inset(-*self.insets)
            .to_rounded_rect(8.0);

        ctx.fill(shape, &*self.color);
    }
}

fn main_widget() -> impl Widget<()> {
    AnimatedWidget::new().align_horizontal(UnitPoint::CENTER)
}

fn main() {
    let main_window = WindowDesc::new(main_widget)
        .title("Animated value")
        .window_size((800.0, 600.0));

    // start the application
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(())
        .expect("Failed to launch application");
}
