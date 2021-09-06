use druid::widget::{Flex, Slider};
use druid::{AppLauncher, Color, Widget, WidgetExt, WindowDesc};
use druid_widget_nursery::{AdvancedSlider};

fn build_ui() -> impl Widget<f64> {
    Flex::column()
    .with_spacer(30.0)
        .with_child(Slider::new().with_range(0.0, 80.0))
        .with_spacer(10.0)
        .with_flex_child(
            AdvancedSlider::new()
                .with_range(10.0, 70.0)
                .with_start_val(15.0)
                .with_step_size(1.0)
                .with_significant(2)
                .with_text_offset(0.0)
                .background(Color::rgb8(30, 30, 30)),
            1.0
        )
        .background(Color::rgb8(30, 30, 30))
}

fn main() {
    let window = WindowDesc::new(build_ui())
        .title("Advanced Slider")
        .window_size((300.0, 250.0));

    AppLauncher::with_window(window)
        .launch(0.0)
        .expect("launch failed");
}
