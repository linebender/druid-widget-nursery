// Copyright 2022 the Druid Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use druid::widget::{Container, Label};
use druid::{AppLauncher, Color, Data, Lens, Widget, WidgetExt, WindowDesc};
use druid_widget_nursery::Stack;

#[derive(Clone, Default, Data, Lens)]
struct AppState {}

fn build_ui() -> impl Widget<AppState> {
    Stack::new()
        .with_child(
            Container::new(Label::new("RED"))
                .fix_width(200.)
                .fix_height(200.)
                .background(Color::RED),
        )
        .with_child(
            Container::new(Label::new("GREEN"))
                .fix_width(180.)
                .fix_height(180.)
                .background(Color::GREEN),
        )
        .with_child(
            Container::new(Label::new("BLUE"))
                .fix_width(160.)
                .fix_height(160.)
                .background(Color::BLUE),
        )
}

pub fn main() {
    let main_window = WindowDesc::new(build_ui().center()).title("Stack Test");

    let state = AppState::default();

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(state)
        .expect("launch failed");
}
