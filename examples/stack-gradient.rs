// Copyright 2022 the Druid Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use druid::widget::{Align, Container, Label};
use druid::{
    AppLauncher, Color, Data, Lens, LinearGradient, UnitPoint, Widget, WidgetExt, WindowDesc,
};
use druid_widget_nursery::Stack;

#[derive(Clone, Default, Data, Lens)]
struct AppState {}

fn build_ui() -> impl Widget<AppState> {
    let gradient = LinearGradient::new(
        UnitPoint::BOTTOM,
        UnitPoint::TOP,
        (Color::WHITE, Color::BLACK),
    );

    Stack::new()
        .with_child(
            Container::new(Label::new(""))
                .fix_width(250.)
                .fix_height(250.)
                .background(Color::WHITE),
        )
        .with_child(
            Container::new(
                Align::new(
                    UnitPoint::CENTER,
                    Label::new("Foreground Text").with_text_size(20.),
                )
                .expand(),
            )
            .background(gradient)
            .padding(5.),
        )
        .fix_width(250.)
        .fix_height(250.)
}

pub fn main() {
    let main_window = WindowDesc::new(build_ui().center()).title("Stack Test");

    let state = AppState::default();

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(state)
        .expect("launch failed");
}
