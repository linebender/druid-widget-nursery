use druid::{widget::Flex, AppLauncher, Color, LocalizedString, Widget, WidgetExt, WindowDesc};
use druid_widget_nursery::material_icons::{
    normal::action::{ABC, ADD_CARD, ADD_TASK, ADD_TO_DRIVE},
    Icon,
};
use qu::ick_use::*;

// Helps to make the icons visible.
fn show_icon(icon: impl Widget<()> + 'static) -> impl Widget<()> {
    icon.padding(10.)
}

fn ui_builder() -> impl Widget<()> {
    Flex::row()
        .with_child(show_icon(Icon::new(ABC)))
        // if we make it bigger it will preserve aspect ratio if possible
        .with_child(show_icon(Icon::new(ADD_TASK).fix_width(100.)))
        // demo non-uniform scale
        .with_child(show_icon(Icon::new(ADD_CARD).fix_size(24., 100.)))
        // different color
        .with_child(show_icon(Icon::new(ADD_TO_DRIVE).with_color(Color::MAROON)))
        .center()
}

#[qu::ick]
pub fn main() -> Result {
    // Create the main window
    let main_window = WindowDesc::new(ui_builder())
        .title(LocalizedString::new("material-icons").with_placeholder("Material Icons demo"));

    // start the application
    AppLauncher::with_window(main_window).launch(())?;
    Ok(())
}
