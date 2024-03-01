// Copyright 2021 the Druid Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use druid::{
    widget::{Flex, Label, TextBox},
    AppLauncher, Data, Env, Lens, Widget, WidgetExt, WindowDesc,
};

use druid_widget_nursery::WidgetExt as _;

fn main() {
    let window = WindowDesc::new(ui());
    AppLauncher::with_window(window)
        .launch(AppState {
            message: "Boo!".to_string(),
        })
        .unwrap();
}
#[derive(Clone, Data, Lens)]
struct AppState {
    message: String,
}

fn ui() -> impl Widget<AppState> {
    let label = Label::new("Hover me for a secret message!")
        .tooltip(|data: &AppState, _env: &Env| data.message.clone());
    let text_box = TextBox::new().lens(AppState::message);

    Flex::column()
        .with_child(label)
        .with_default_spacer()
        .with_child(text_box)
}
