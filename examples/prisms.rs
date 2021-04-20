use druid::widget::{CrossAxisAlignment, Flex, Slider, TextBox};
use druid::{AppLauncher, Data, UnitPoint, Widget, WidgetExt, WindowDesc};
use druid_widget_nursery::prism::{Closures, Prism};
use druid_widget_nursery::{MultiCheckbox, MultiRadio};

#[derive(Data, Clone, PartialEq)]
enum TestData {
    A(f64),
    B(String),
    C(Option<String>),
}

struct TestDataA;

impl Prism<TestData, f64> for TestDataA {
    fn get(&self, data: &TestData) -> Option<f64> {
        if let TestData::A(value) = data {
            Some(*value)
        } else {
            None
        }
    }

    fn put(&self, data: &mut TestData, inner: f64) {
        *data = TestData::A(inner);
    }
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
                    Some(*value)
                } else {
                    None
                }
            },
            |data: &mut TestData, inner| *data = TestData::A(inner),
        ),
    );

    let b = MultiRadio::new("Variant B", TextBox::new(), String::new(), TestDataB);

    let c_inner = MultiCheckbox::new("inner value", TextBox::new(), String::from("initial data"));

    let c = MultiRadio::new("Variant C", c_inner, None, TestDataC);

    let left = Flex::column()
        .with_child(a)
        .with_default_spacer()
        .with_child(b)
        .with_default_spacer()
        .with_child(c)
        .cross_axis_alignment(CrossAxisAlignment::Start);

    let a = MultiRadio::new(
        "Variant A",
        Slider::new().with_range(0.0, 10.0),
        0.0,
        Closures(
            |outer: &TestData| {
                if let TestData::A(value) = outer {
                    Some(*value)
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

    let c_inner = MultiCheckbox::new("inner value", TextBox::new(), String::from("initial data"))
        .show_when_disabled();

    let c = MultiRadio::new("Variant C", c_inner, None, TestDataC).show_when_disabled();

    let middle = Flex::column()
        .with_child(a)
        .with_default_spacer()
        .with_child(b)
        .with_default_spacer()
        .with_child(c)
        .cross_axis_alignment(CrossAxisAlignment::Start);

    let right = druid_widget_nursery::enum_switcher::Switcher::new()
        .with_variant(TestDataA, Slider::new().with_range(0.0, 10.0))
        .with_variant(TestDataB, TextBox::new())
        .with_variant(
            TestDataC,
            MultiCheckbox::new("optional data", TextBox::new(), "".to_string()),
        );

    Flex::row()
        .with_child(left)
        .with_default_spacer()
        .with_child(middle)
        .with_default_spacer()
        .with_child(right)
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .padding(10.0)
        .align_horizontal(UnitPoint::TOP)
}

fn main() {
    let main_window = WindowDesc::new(main_widget())
        .title("Select")
        .window_size((400., 280.))
        .set_position((700.0, 300.0));

    // create the initial app state
    let app_data = TestData::A(2.0);

    // start the application
    AppLauncher::with_window(main_window)
        .launch(app_data)
        .expect("Failed to launch application");
}
