use druid::kurbo::{Circle, Point, Size};
use druid::widget::{
    Button, CrossAxisAlignment, Flex, Label, LabelText, Parse, RadioGroup, TextBox, WidgetExt,
};
use druid::{
    theme, AppLauncher, BoxConstraints, Color, Data, Env, Event, EventCtx, LayoutCtx, Lens,
    LifeCycle, LifeCycleCtx, PaintCtx, UpdateCtx, Widget, WindowDesc,
};
use druid_widget_nursery::animation::{
    CurvedAnimation, AnimationController, AnimationCurve, AnimationDirection,
};

use druid::widget::prelude::RenderContext;


#[derive(Data, Clone, Copy, PartialEq)]
enum Curve {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseOutElastic,
    BounceOut,
    EaseOutSine,
}

impl From<Curve> for AnimationCurve {
    fn from(curve: Curve) -> AnimationCurve {
        match curve {
            Curve::Linear => AnimationCurve::LINEAR,
            Curve::EaseIn => AnimationCurve::EASE_IN,
            Curve::EaseOut => AnimationCurve::EASE_OUT,
            Curve::EaseInOut => AnimationCurve::EASE_IN_OUT,
            Curve::EaseOutElastic => AnimationCurve::EASE_OUT_ELASTIC,
            Curve::BounceOut => AnimationCurve::BOUNCE_OUT,
            Curve::EaseOutSine => AnimationCurve::EASE_OUT_SINE,
        }
    }
}

fn main_widget() -> impl Widget<AnimState> {
    fn group<T: Data>(t: impl Into<LabelText<T>>, w: impl Widget<T> + 'static) -> impl Widget<T> {
        Flex::column()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .with_child(
                Label::new(t)
                    .padding(5.)
                    .background(theme::PLACEHOLDER_COLOR)
                    .expand_width(),
            )
            .with_child(w)
            .border(Color::WHITE, 0.5)
            .padding(5.)
    }

    let controls = Flex::column()
        .with_child(group(
            "Curve",
            RadioGroup::new(vec![
                ("Linear", Curve::Linear),
                ("EaseIn", Curve::EaseIn),
                ("EaseOut", Curve::EaseOut),
                ("EaseInOut", Curve::EaseInOut),
                ("EaseOutElastic", Curve::EaseOutElastic),
                ("BounceOut", Curve::BounceOut),
                ("EaseOutSine", Curve::EaseOutSine),
            ])
            .lens(AnimState::curve),
        ))
        .with_child(group(
            "Duration ms",
            Parse::new(TextBox::new())
                .expand_width()
                .lens(AnimState::duration),
        ))
        .with_child(
            group(
                "Direction",
                RadioGroup::new(vec![
                    ("Forward", AnimationDirection::Forward),
                    ("Reverse", AnimationDirection::Reverse),
                    ("Alternate", AnimationDirection::Alternate),
                    ("AlternateReverse", AnimationDirection::AlternateReverse),
                ]),
            )
            .lens(AnimState::direction),
        )
        .with_child(
            group("Repeat", Parse::new(TextBox::new()))
                .expand_width()
                .lens(AnimState::repeat_limit),
        )
        .with_child(
            Flex::row()
                .with_child(Label::new("Animate:"))
                .with_child(Button::new("Size").on_click(|_, state: &mut AnimState, _| {
                    state.toggle_size = !state.toggle_size;
                }))
                .with_child(
                    Button::new("Alpha").on_click(|_, state: &mut AnimState, _| {
                        state.toggle_alpha = !state.toggle_alpha;
                    }),
                ),
        )
        .with_flex_spacer(1.)
        .fix_width(200.0)
        .border(Color::WHITE, 0.5);

    Flex::row()
        .with_child(controls)
        .with_flex_child(AnimatedWidget::default(), 1.)
}

fn main() {
    let main_window = WindowDesc::new(main_widget())
        .title("Animation")
        .window_size((800.0, 600.0));

    // create the initial app state
    let initial_state = AnimState {
        curve: Curve::Linear,
        duration: Some(1000),
        direction: AnimationDirection::Forward,
        toggle_size: false,
        toggle_alpha: false,
        repeat_limit: Some(1),
    };

    // start the application
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(initial_state)
        .expect("Failed to launch application");
}

#[derive(Clone, Data, Lens)]
struct AnimState {
    curve: Curve,
    duration: Option<usize>,
    direction: AnimationDirection,
    repeat_limit: Option<usize>,
    toggle_size: bool,
    toggle_alpha: bool,
}

struct DrawState {
    circle: Circle,
    color: Color,
    max_radius: f64,
}

impl Default for DrawState {
    fn default() -> Self {
        Self {
            circle: Default::default(),
            color: Color::rgb8(0xFF, 0, 0),
            max_radius: 0.0,
        }
    }
}

#[derive(Default)]
struct AnimatedWidget {
    radius_animation: CurvedAnimation,
    alpha_animation: CurvedAnimation,
    draw: DrawState,
}

impl AnimatedWidget {
    fn create_animation(data: &AnimState) -> CurvedAnimation {
        CurvedAnimation::new(
            data.curve,
            AnimationController::new()
                .repeat_limit(data.repeat_limit)
                .direction(data.direction)
                .duration(data.duration.map(|ms| ms as f64 / 1000.0).unwrap_or(1.0))
        )
    }
}

impl Widget<AnimState> for AnimatedWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut AnimState, _env: &Env) {
        if let Event::AnimFrame(nanos) = event {
            self.radius_animation.update(ctx, *nanos);
            self.alpha_animation.update(ctx, *nanos);

            let draw = &mut self.draw;

            draw.circle.radius = self.radius_animation.progress() * draw.max_radius;
            draw.color = draw.color.clone().with_alpha(self.alpha_animation.progress())
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        _data: &AnimState,
        _env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            self.radius_animation.start(ctx);
            self.alpha_animation.start(ctx);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &AnimState, data: &AnimState, _env: &Env) {
        if old_data.toggle_size != data.toggle_size {
            self.radius_animation = Self::create_animation(data);
            self.radius_animation.start(ctx);
        }
        if old_data.toggle_alpha != data.toggle_alpha {
            self.alpha_animation = Self::create_animation(data);
            self.alpha_animation.start(ctx);
        }
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &AnimState,
        _env: &Env,
    ) -> Size {
        let size = bc.max();
        self.draw.circle.center = Point::new(size.width / 2., size.height / 2.);
        self.draw.max_radius = size.width.min(size.height) / 2.;
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &AnimState, _env: &Env) {
        let rect = ctx.size().to_rect();
        ctx.clip(rect);
        ctx.fill(self.draw.circle, &self.draw.color)
    }
}
