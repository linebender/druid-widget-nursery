use druid::{
    widget::{Flex, Label, LineBreaking, MainAxisAlignment, SizedBox},
    AppLauncher, Color, Data, Env, Lens, Widget, WidgetExt, WindowDesc,
};
use druid_widget_nursery::DynamicSizedBox;

fn main() {
    let window = WindowDesc::new(ui());
    let dyn_message = "Hello there, this is a dynamically sized box and you will see it change based on the size of its parent.";
    let fixed_message =
        "Hello there, this is a fixed size box and it will not change no matter what.";
    AppLauncher::with_window(window)
        .launch(AppState {
            dynamic_box: dyn_message.to_string(),
            fixed_box: fixed_message.to_string(),
        })
        .unwrap();
}
#[derive(Clone, Data, Lens)]
struct AppState {
    dynamic_box: String,
    fixed_box: String,
}

fn ui() -> impl Widget<AppState> {
    let dynamic_label = Label::new(|data: &String, _env: &Env| data.clone())
        .with_text_color(Color::BLACK)
        .with_line_break_mode(LineBreaking::WordWrap)
        .center()
        .lens(AppState::dynamic_box);
    let dynamic_box = DynamicSizedBox::new(dynamic_label)
        // widget will be half the height of the box constraints
        // the parent gives
        .with_height(0.5)
        // widget will be 1/3 the width of its parent given box constraints
        .with_width(0.33)
        .background(Color::WHITE);

    let fixed_label = Label::new(|data: &String, _env: &Env| data.clone())
        .with_text_color(Color::BLACK)
        .with_line_break_mode(LineBreaking::WordWrap)
        .center()
        .lens(AppState::fixed_box);
    let fixed_box = SizedBox::new(fixed_label)
        .height(250.)
        .width(250.)
        .background(Color::WHITE);
    Flex::column()
        .with_child(fixed_box)
        // use flex_child function to give dynamic box concrete constraints
        // otherwise it would try size itself using infinity as a parameter
        .with_flex_child(dynamic_box, 1.0)
        .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
}
