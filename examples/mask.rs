// Copyright 2022 the Druid Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use druid::widget::{Button, Flex, TextBox};
use druid::{AppLauncher, Color, Data, Lens, Widget, WidgetExt, WindowDesc, WindowSizePolicy};

use druid_widget_nursery::Mask;

#[derive(Clone, Data, Lens)]
struct AppState {
    show_mask: bool,
    text: String,
}

fn ui_builder() -> impl Widget<AppState> {
    Flex::column()
        .with_child(
            Button::new("Toggle mask")
                .on_click(|_, data: &mut AppState, _| {
                    data.show_mask = !data.show_mask;
                })
                .padding(5.),
        )
        .with_child({
            let child = TextBox::new().with_text_size(100.).lens(AppState::text);
            Mask::new(child)
                .with_text_mask("Loading...")
                .dynamic(|state: &AppState, _| state.show_mask)
                .border(Color::WHITE, 1.0)
        })
}

pub fn main() {
    let main_window = WindowDesc::new(ui_builder())
        .window_size_policy(WindowSizePolicy::Content)
        .title("Load Mask Example");

    let state = AppState {
        show_mask: true,
        text: String::from("Masked content"),
    };

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(state)
        .expect("launch failed");
}
