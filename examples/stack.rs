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

use druid::widget::{Container, Controller, Flex, Label, Slider, TextBox};
use druid::{
    AppLauncher, Color, Data, Env, Event, EventCtx, Lens, UnitPoint, Widget, WidgetExt, WindowDesc,
    WindowSizePolicy,
};

use druid_widget_nursery::{Stack, StackChildParams, StackChildPosition};

#[derive(Clone, Default, Data, Lens)]
struct AppState {
    mytext1: String,
    mytext2: String,
    position: StackChildPosition,
    slider_top: f64,
}

// Controller to translate slider position into child top position
struct UpdatePosition;

impl<W: Widget<AppState>> Controller<AppState, W> for UpdatePosition {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        data.position.top = Some(data.slider_top);
        child.event(ctx, event, data, env)
    }
}

fn build_ui() -> impl Widget<AppState> {
    Flex::column()
        .with_child(build_toolbar_ui())
        .with_child(build_stack_ui())
}

fn build_toolbar_ui() -> impl Widget<AppState> {
    Flex::row()
        .with_child(Label::new("Dynamic TOP position"))
        .with_child(
            Slider::new()
                .with_range(0f64, 400.0)
                .lens(AppState::slider_top)
                .fix_width(250.)
                .controller(UpdatePosition),
        )
}

fn build_stack_ui() -> impl Widget<AppState> {
    Stack::new()
        .align(UnitPoint::RIGHT)
        .with_child(
            Container::new(Label::new("The Stack"))
                .fix_width(500.)
                .fix_height(400.)
                .background(Color::GREEN),
        )
        .with_child(TextBox::new().with_text_size(100.).lens(AppState::mytext1))
        .with_positioned_child(
            TextBox::new().with_text_size(50.).lens(AppState::mytext2),
            StackChildPosition::new().bottom(Some(0.)),
        )
        .with_positioned_child(
            Label::new("Animated").with_text_size(50.),
            StackChildParams::dynamic(|state: &AppState, _| &state.position).duration(1.0),
        )
        .border(Color::WHITE, 1.0)
        .debug_paint_layout()
}

pub fn main() {
    let main_window = WindowDesc::new(build_ui())
        .window_size_policy(WindowSizePolicy::Content)
        .title("Stack Test");

    let state = AppState {
        mytext1: "TEXT1".into(),
        mytext2: "TEXT2".into(),
        ..Default::default()
    };

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(state)
        .expect("launch failed");
}
