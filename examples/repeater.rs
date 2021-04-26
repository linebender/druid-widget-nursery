use druid::{im, widget::SizedBox, AppLauncher, Data, Lens, Point, Size, Widget, WindowDesc};

use druid_widget_nursery::Repeater;

#[derive(Clone, Data)]
pub struct Window {
    pub origin: Point,
    pub size: Size,
    pub id: u64,
}

#[derive(Clone, Data, Lens)]
pub struct AppState {
    windows: im::Vector<Window>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            windows: im::Vector::new(),
        }
    }
}

fn main_widget() -> impl Widget<AppState> {
    Repeater::new(
        AppState::windows,
        Box::new(|window: &Window| window.id),
        Box::new(|_window: &Window| SizedBox::empty()),
    )
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
