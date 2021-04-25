use druid::piet;
use druid::piet::{Color, GradientStop, UnitPoint};
use druid::widget::{Button, Flex, Label, WidgetExt};
use druid::{
    AppDelegate, AppLauncher, Command, Data, DelegateCtx, Env, ExtEventSink, Handled, Lens,
    PlatformError, Selector, Target, Widget, WindowDesc,
};

use druid_widget_nursery::ProgressBar;

use std::fmt::Debug;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Data, Lens, Debug)]
pub struct AppState {
    slow_progress: f64,
    fast_progress: f64,
    leaping_progress: f64,
    #[data(ignore)]
    msg_tx: mpsc::Sender<AppMessage>,
}

enum AppMessage {
    StartProgressCycle,
}

pub const UPDATE_PROGRESS: Selector<AppState> = Selector::new("progress_bar.update_progress");

fn main() -> Result<(), PlatformError> {
    let window = WindowDesc::new(build_ui())
        .window_size((600.0, 600.0))
        .title("New Progress Bar Example");
    let launcher = AppLauncher::with_window(window).delegate(Delegate {});
    let (tx, rx) = mpsc::channel::<AppMessage>();

    let state = AppState {
        slow_progress: 0.2,
        fast_progress: 0.2,
        leaping_progress: 0.2,
        msg_tx: tx,
    };
    let other_state = state.clone();

    let ui_handle = launcher.get_external_handle();
    let _game_thread = thread::spawn(move || other_thread(&other_state, rx, ui_handle));

    launcher.launch(state)
}
fn build_ui() -> impl Widget<AppState> {
    let mut flex = Flex::column(); //.cross_axis_alignment(CrossAxisAlignment::Start);

    flex.add_child(
        Button::new("Start Progress Cycle").on_click(|_, data: &mut AppState, _| {
            data.msg_tx
                .send(AppMessage::StartProgressCycle)
                .expect("Message to game loop from ui failed to send")
        }),
    );
    flex.add_spacer(10.0);
    flex.add_child(
        Flex::column()
            .with_child(Label::new("Slow Progress Bar"))
            .with_child(
                ProgressBar::new()
                    .with_bar_brush(piet::PaintBrush::Color(Color::RED))
                    .with_corner_radius(2.0)
                    .with_border_width(2.0)
                    .lens(AppState::slow_progress),
            )
            .with_child(
                ProgressBar::new()
                    .with_bar_brush(piet::PaintBrush::Linear(druid::LinearGradient::new(
                        UnitPoint::LEFT,
                        UnitPoint::RIGHT,
                        (Color::WHITE, Color::BLUE),
                    )))
                    .with_corner_radius(2.0)
                    .with_border_width(2.0)
                    .lens(AppState::slow_progress),
            ),
    );
    flex.add_spacer(10.0);
    flex.add_child(
        Flex::column()
            .with_child(Label::new("Fast Progress Bar"))
            .with_child(
                ProgressBar::new()
                    .with_bar_brush(piet::PaintBrush::Color(Color::RED))
                    .with_corner_radius(2.0)
                    .with_border_width(2.0)
                    .lens(AppState::fast_progress),
            )
            .with_child(
                ProgressBar::new()
                    .with_bar_brush(piet::PaintBrush::Linear(druid::LinearGradient::new(
                        UnitPoint::LEFT,
                        UnitPoint::RIGHT,
                        (Color::WHITE, Color::BLUE),
                    )))
                    .with_corner_radius(2.0)
                    .with_border_width(2.0)
                    .lens(AppState::fast_progress),
            ),
    );
    flex.add_spacer(10.0);
    flex.add_child(
        Flex::column()
            .with_child(Label::new("Sporadic Progress Bar"))
            .with_child(
                ProgressBar::new()
                    .with_bar_brush(piet::PaintBrush::Color(Color::RED))
                    .with_corner_radius(2.0)
                    .with_border_width(2.0)
                    .lens(AppState::leaping_progress),
            )
            .with_child(
                ProgressBar::new()
                    .with_bar_brush(piet::PaintBrush::Linear(druid::LinearGradient::new(
                        UnitPoint::LEFT,
                        UnitPoint::RIGHT,
                        (Color::WHITE, Color::BLUE),
                    )))
                    .with_corner_radius(2.0)
                    .with_border_width(2.0)
                    .lens(AppState::leaping_progress),
            ),
    );
    flex.add_spacer(10.0);
    flex.add_child(
        Flex::column()
            .with_child(Label::new("Fixed Gradient Progress Bar"))
            .with_child(
                ProgressBar::new()
                    .with_bar_brush(piet::PaintBrush::Linear(druid::LinearGradient::new(
                        UnitPoint::LEFT,
                        UnitPoint::RIGHT,
                        vec![
                            GradientStop {
                                pos: 0.0,
                                color: Color::RED,
                            },
                            GradientStop {
                                pos: 0.3,
                                color: Color::RED,
                            },
                            GradientStop {
                                pos: 0.3,
                                color: Color::YELLOW,
                            },
                            GradientStop {
                                pos: 0.5,
                                color: Color::YELLOW,
                            },
                            GradientStop {
                                pos: 0.5,
                                color: Color::BLUE,
                            },
                            GradientStop {
                                pos: 0.8,
                                color: Color::BLUE,
                            },
                            GradientStop {
                                pos: 0.8,
                                color: Color::GREEN,
                            },
                            GradientStop {
                                pos: 1.0,
                                color: Color::GREEN,
                            },
                        ],
                    )))
                    .with_corner_radius(2.0)
                    .with_border_width(2.0)
                    .lens(AppState::slow_progress),
            ),
    );
    flex.add_spacer(10.0);
    flex.add_child(
        Flex::column()
            .with_child(Label::new("Theme Configured Progress Bar"))
            .with_child(ProgressBar::new().lens(AppState::leaping_progress)),
    );

    flex
}

fn other_thread(state: &AppState, msg_rx: mpsc::Receiver<AppMessage>, event_handle: ExtEventSink) {
    let mut inst = Instant::now();
    let mut my_state = state.clone();
    loop {
        for msg in msg_rx.try_iter() {
            match msg {
                AppMessage::StartProgressCycle => {
                    my_state.slow_progress = 0.0;
                    my_state.fast_progress = 0.0;
                    my_state.leaping_progress = 0.0;
                }
            }
        }

        let mut new_state = my_state.clone();
        if my_state.slow_progress < 1.0 {
            new_state.slow_progress += 0.01;
        }
        if my_state.fast_progress < 1.0 {
            new_state.fast_progress += 0.05;
        }
        if my_state.leaping_progress < 1.0 && inst.elapsed().as_millis() > 1000 {
            new_state.leaping_progress += 0.1;
            inst = Instant::now();
        }

        if !my_state.same(&new_state) {
            my_state = new_state.clone();
            if event_handle
                .submit_command(UPDATE_PROGRESS, my_state.clone(), Target::Auto)
                .is_err()
            {
                println!("Druid Command Error - UPDATE_PROGRESS");
            }
        }
        //println!("loop {}", my_state.fast_progress);

        thread::sleep(Duration::from_millis(50));
    }
}

struct Delegate;

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppState,
        _env: &Env,
    ) -> Handled {
        //Get commands from ExtEventSink structs on other threads here.
        if let Some(d) = cmd.get(UPDATE_PROGRESS) {
            *data = d.clone();
            Handled::Yes
        } else {
            Handled::No
        }
    }
}
