use druid::im::Vector;
use druid::widget::{Container, Flex, Label, Scroll, WidgetExt};
use druid::{AppLauncher, Data, Env, Lens, Widget, WindowDesc};
use druid_widget_nursery::splits::Splits;

#[derive(Data, Clone, Lens)]
struct AppState {
    collection: Vector<String>,
}

fn main_widget() -> impl Widget<AppState> {
    Flex::column()
        .main_axis_alignment(druid::widget::MainAxisAlignment::Start)
        .cross_axis_alignment(druid::widget::CrossAxisAlignment::Start)
        .with_child(Container::new(
            Scroll::new(
                Splits::new(|| {
                    Label::new(|text: &String, _: &Env| format!("Collection: {}", text))
                        .fix_height(120.)
                })
                .horizontal()
                .min_size(180.)
                .draggable(true)
                .bar_size(6.),
            )
            .horizontal()
            .lens(AppState::collection),
        ))
}

pub fn main() {
    let main_window = WindowDesc::new(main_widget())
        .title("Dropdown")
        .window_size((500., 140.));

    let mut collection = Vector::new();
    collection.push_back("Column 1".to_string());
    collection.push_back("Column 2".to_string());
    collection.push_back("Column 3".to_string());

    let initial_state = AppState { collection };

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(initial_state)
        .expect("Failed to launch application");
}
