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
