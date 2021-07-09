use druid::widget::{Button, Flex, Label};
use druid::{AppLauncher, Application, Widget, WidgetExt, WindowDesc};
use druid_widget_nursery::TitleBar;

// TitleBar only works on windows.

fn build_titlebar() -> impl Widget<u32> {
    // Make a row of buttons and title for the titlebar.
    Flex::row()
        // Increases the state by one each time clicked. 
        .with_child(Button::new("+1").on_click(|_event, t, _env| *t += 1))
        // Flex child so it fills up all the space, which means it has to be limited. 
        // This is done by the column in build_ui()
        .with_flex_child(
            // New titlebar with a label
            TitleBar::new(
                // label is dynamic, using the value from data
                Label::dynamic(|data: &u32, _env| format!("TitleBar: Value = {}", data))
                // Center the label in the titlebar.
                .center()
            ), 1.0
        )
        // Button to close the window.
        // There isn't a normal titlebar, so something like this has to exist.
        .with_child(Button::new("Close").on_click(|_,_,_| Application::global().quit()))
}

fn build_ui() -> impl Widget<u32> {
    // A column with a titlebar and then whatever body the app has beneath it.
    Flex::column()
        .with_child(build_titlebar())
        .with_child(Label::new(
            "Whatever is beneath the titlebar.\nYou can also drag the window around by the title! Try it"
        ))
}

fn main() {
    let window = WindowDesc::new(build_ui())
    // Set show_titlebar to false, this hides the normal titlebar.
        .show_titlebar(false);
    // Start titlebar value at 0
    let data = 0u32;
    // Launch
    AppLauncher::with_window(window)
        .launch(data)
        .expect("Failed to launch");
}
