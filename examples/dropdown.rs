use druid::widget::{
    Button, CrossAxisAlignment, Flex, Label, RadioGroup, Scroll, TextBox, WidgetExt,
};
use druid::{AppLauncher, Data, Env, EventCtx, Lens, Widget, WindowDesc};
use druid_widget_nursery::{Dropdown, DROP};

#[derive(Data, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
enum Fruit {
    Apple,
    Pear,
    Orange,
}

#[derive(Data, Clone, Lens)]
struct DropDownState {
    fruit: Fruit,
    place: String,
}

fn main_widget() -> impl Widget<DropDownState> {
    Scroll::new(
        Flex::column()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .with_child(Label::new(|d: &DropDownState, _: &Env| {
                format!("Lets eat {:?}s in {}", d.fruit, d.place)
            }))
            .with_spacer(10.)
            .with_child(
                Dropdown::new(
                    Flex::row()
                        .with_child(TextBox::new())
                        .with_flex_spacer(1.)
                        .with_child(
                            Button::new("V")
                                .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROP)),
                        ),
                    |_, _| {
                        let places: Vec<(&'static str, String)> =
                            vec!["England", "San Tropez", "Antarctica"]
                                .into_iter()
                                .map(|item| (item, item.to_owned()))
                                .collect();
                        RadioGroup::new(places)
                    },
                )
                .align_left()
                .lens(DropDownState::place),
            )
            .with_child(
                Dropdown::new(
                    Flex::row()
                        .with_child(Label::new(|f: &Fruit, _: &Env| format!("{:?}", f)))
                        .with_flex_spacer(1.)
                        .with_child(
                            Button::new("V")
                                .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROP)),
                        ),
                    |_, _| {
                        RadioGroup::new(vec![
                            ("Apple", Fruit::Apple),
                            ("Pear", Fruit::Pear),
                            ("Orange", Fruit::Orange),
                        ])
                    },
                )
                .align_left()
                .lens(DropDownState::fruit),
            )
            .with_spacer(200.)
            .with_child(
                Dropdown::new(
                    Button::new(|f: &Fruit, _: &Env| format!("{:?}", f))
                        .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROP)),
                    |_, _| {
                        RadioGroup::new(vec![
                            ("Apple", Fruit::Apple),
                            ("Pear", Fruit::Pear),
                            ("Orange", Fruit::Orange),
                        ])
                    },
                )
                .align_left()
                .lens(DropDownState::fruit),
            )
            .with_spacer(100.)
            .with_child(Label::new(|d: &DropDownState, _: &Env| {
                format!("Lets eat {:?}s in {}", d.fruit, d.place)
            }))
            .padding(10.)
            .fix_width(250.),
    )
    .fix_height(250.0)
}

pub fn main() {
    let main_window = WindowDesc::new(main_widget())
        .title("Dropdown")
        .window_size((250., 300.));

    // create the initial app state
    let initial_state = DropDownState {
        fruit: Fruit::Apple,
        place: "California".to_owned(),
    };

    // start the application
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(initial_state)
        .expect("Failed to launch application");
}
