use druid::widget::{CrossAxisAlignment, Flex, Slider, TextBox};
use druid::{AppLauncher, Data, UnitPoint, Widget, WidgetExt, WindowDesc};
use druid_widget_nursery::partial::{Closures, Prism};
use druid_widget_nursery::{MultiCheckbox, MultiRadio};

#[derive(Data, Clone, PartialEq)]
enum TestData {
    A(f64),
    B(String),
    C(Option<String>),
}

struct TestDataB;

impl Prism<TestData, String> for TestDataB {
    fn get(&self, data: &TestData) -> Option<String> {
        if let TestData::B(str) = data {
            Some(str.to_string())
        } else {
            None
        }
    }

    fn put(&self, data: &mut TestData, inner: String) {
        *data = TestData::B(inner);
    }
}

struct TestDataC;

impl Prism<TestData, Option<String>> for TestDataC {
    fn get(&self, data: &TestData) -> Option<Option<String>> {
        if let TestData::C(value) = data {
            Some((*value).clone())
        } else {
            None
        }
    }

    fn put(&self, data: &mut TestData, inner: Option<String>) {
        *data = TestData::C(inner);
    }
}

fn main_widget() -> impl Widget<TestData> {
    let a = MultiRadio::new(
        "Variant A",
        Slider::new().with_range(0.0, 10.0),
        0.0,
        Closures(
            |outer: &TestData| {
                if let TestData::A(value) = outer {
                    Some(value.clone())
                } else {
                    None
                }
            },
            |data: &mut TestData, inner| *data = TestData::A(inner),
        ),
    )
    .show_when_disabled();

    let b =
        MultiRadio::new("Variant B", TextBox::new(), String::new(), TestDataB).show_when_disabled();

    let c_inner = MultiCheckbox::new("inner value", TextBox::new(), String::from("initial data"));

    let c = MultiRadio::new("Variant C", c_inner, None, TestDataC).show_when_disabled();

    Flex::column()
        .with_child(a)
        .with_child(b)
        .with_child(c)
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .padding((50.0, 10.0, 10.0, 10.0))
        .align_horizontal(UnitPoint::TOP_LEFT)
}

fn main() {
    let main_window = WindowDesc::new(main_widget())
        .title("Select")
        .window_size((250., 300.));

    // create the initial app state
    let app_data = TestData::A(2.0);

    // start the application
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(app_data)
        .expect("Failed to launch application");
}
