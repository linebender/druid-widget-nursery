// Copyright 2022 the Druid Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use druid::widget::{Container, Label};
use druid::{AppLauncher, Color, Data, Lens, Widget, WidgetExt, WindowDesc};

use druid_widget_nursery::{Stack, StackChildPosition};

#[derive(Clone, Default, Data, Lens)]
struct AppState {}

fn build_ui() -> impl Widget<AppState> {
    Stack::new()
        .fit(true)
        .with_child(
            Container::new(Label::new(
                "Expanded non-positional Stack child (fit = true)",
            ))
            .padding(20.)
            .background(Color::BLUE),
        )
        .with_positioned_child(
            Container::new(Label::new("Positional Stack child"))
                .padding(10.)
                .background(Color::RED),
            StackChildPosition::new()
                .left(Some(20.))
                .top(Some(80.))
                .right(Some(20.))
                .bottom(Some(20.)),
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
