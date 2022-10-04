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
