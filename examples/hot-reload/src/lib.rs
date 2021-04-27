use druid::widget::{Flex, TextBox};
use druid::{Data, Lens, Widget, WidgetExt};

#[derive(Debug, Default, Data, Clone, Lens)]
pub struct AppData {
    name: String,
    checked: String,
}

#[no_mangle]
pub fn view() -> Box<dyn Widget<AppData>> {
    Flex::column()
        .with_child(TextBox::new().lens(AppData::name))
        .with_spacer(90.)
        .with_child(TextBox::new().lens(AppData::checked))
        .boxed()
}
