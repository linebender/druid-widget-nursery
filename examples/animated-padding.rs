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

use druid::{AppLauncher, Data, Insets, Lens, Widget, WidgetExt, WindowDesc};
use druid::widget::{Button, Flex, Label};

use druid_widget_nursery::AnimatedPadding;
use druid_widget_nursery::animation::AnimationCurve;

#[derive(Clone, Data, Lens)]
struct AppState {
    add_space: bool,
}

fn build_ui() -> impl Widget<AppState> {
    Flex::column()
        .with_child(
            Button::new("Change padding").on_click(|_ctx, data: &mut AppState, _env| {
                data.add_space = !data.add_space;
            })
        )
        .with_child(
            AnimatedPadding::new(
                // Seems setting the text width fixes the animation problem.
                Label::new("Animated Padding").fix_width(200.),
                // Label::new("Animated Padding"),
                |data: &AppState, _env| Insets::uniform(if data.add_space { 30.0 } else { 10.0 }),
            ).duration(0.3).curve(AnimationCurve::EASE_IN_OUT)
        )
        .debug_paint_layout()
}

pub fn main() {
    let main_window = WindowDesc::new(build_ui())
        .title("Animated Padding Test");

    let state = AppState {
        add_space: false,
    };

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(state)
        .expect("launch failed");
}
