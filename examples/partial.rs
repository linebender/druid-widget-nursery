use druid::widget::{Flex, Radio};
// use druid::widget::TextBox;
use druid::{AppLauncher, Data, UnitPoint, Widget, WidgetExt, WindowDesc};
// use druid_widget_nursery::partial::PrismWrap;

#[derive(Data, Clone, PartialEq)]
enum TestData {
    A,
    B(String),
    C,
}

fn main_widget() -> impl Widget<TestData> {
    let selections = Flex::column()
        .with_child(Radio::new("Variant A", TestData::A))
        .with_child(Radio::new("Variant B", TestData::B(String::from("hi"))))
        .with_child(Radio::new("Variant C", TestData::C));

    // let partial = PrismWrap::with_closures(
    //     TextBox::new(),
    //     String::new(),
    //     |outer: &TestData| {
    //         if let TestData::B(str) = outer {
    //             Some(str.clone())
    //         } else {
    //             None
    //         }
    //     },
    //     TestData::B,
    // );

    Flex::column()
        .with_child(selections)
        .with_spacer(30.0)
        //.with_child(partial)
        .padding(5.0)
        .align_horizontal(UnitPoint::CENTER)
}

fn main() {
    let main_window = WindowDesc::new(main_widget())
        .title("Select")
        .window_size((250., 300.));

    // create the initial app state
    let app_data = TestData::A;

    // start the application
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(app_data)
        .expect("Failed to launch application");
}
