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

use druid::{
    widget::{Flex, Label},
    AppLauncher, WidgetExt, WindowDesc,
};
use druid_widget_nursery::stack_tooltip::TooltipFactory;

fn main() {
    let mut factory = TooltipFactory::default();

    let base = Flex::column()
        .with_child(factory.tooltip(
            Label::new("Trigger").expand_width().debug_paint_layout(),
            "BAAAAAAAR",
        ))
        .main_axis_alignment(druid::widget::MainAxisAlignment::Center)
        .cross_axis_alignment(druid::widget::CrossAxisAlignment::End)
        .expand()
        .expand_height();

    let wrapped = factory.wrapper(base).unwrap();

    let window = WindowDesc::new(wrapped).window_size((1280., 1024.));

    AppLauncher::with_window(window).launch(()).expect("Launch");
}
