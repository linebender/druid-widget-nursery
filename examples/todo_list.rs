use druid::im::Vector;
use druid::widget::{Button, Checkbox, CrossAxisAlignment, Flex, Label, List, Radio, TextBox};
use druid::{AppLauncher, ArcStr, UnitPoint, Widget, WidgetExt};
use druid::{Data, Lens, WindowDesc};
use druid_widget_nursery::ListFilter;
use std::sync::Arc;

#[derive(Clone, Data, Lens)]
struct ListItem {
    name: ArcStr,
    finished: bool,
}

#[derive(Clone, Data, Lens)]
struct AppData {
    pending: String,
    filtered: Option<bool>,
    elements: Vector<ListItem>,
}

fn item_ui() -> impl Widget<ListItem> {
    Flex::row()
        .with_child(Checkbox::new(" ").lens(ListItem::finished))
        .with_child(Label::dynamic(|data: &ListItem, _| data.name.to_string()))
}

fn main_ui() -> impl Widget<AppData> {
    let filter = Flex::row()
        .with_child(Radio::new("all", None))
        .with_default_spacer()
        .with_child(Radio::new("finished", Some(true)))
        .with_default_spacer()
        .with_child(Radio::new("unfinished", Some(false)))
        .lens(AppData::filtered);

    let new_element = Flex::row()
        .with_child(TextBox::new().lens(AppData::pending))
        .with_child(
            Button::new("Add")
                .on_click(|_, data: &mut AppData, _| {
                    data.elements.push_back(ListItem {
                        name: Arc::from(&*data.pending),
                        finished: false,
                    });
                    data.pending.clear();
                })
                .disabled_if(|data: &AppData, _| data.pending.is_empty()),
        );

    Flex::column()
        .with_child(filter)
        .with_default_spacer()
        .with_child(
            ListFilter::new(
                List::new(item_ui),
                |element: &ListItem, filter_option: &Option<bool>| {
                    (*filter_option).map_or(true, |x| element.finished == x)
                },
            )
            .lens(druid::lens::Map::new(
                |data: &AppData| (data.elements.clone(), data.filtered),
                |data: &mut AppData, inner| {
                    data.elements = inner.0;
                    data.filtered = inner.1;
                },
            )),
        )
        .with_default_spacer()
        .with_child(new_element)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .align_horizontal(UnitPoint::CENTER)
}

fn main() {
    let window = WindowDesc::new(main_ui());

    AppLauncher::with_window(window)
        .launch(AppData {
            pending: String::new(),
            filtered: None,
            elements: Default::default(),
        })
        .unwrap()
}
