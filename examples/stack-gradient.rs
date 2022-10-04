// Copyright 2022 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
