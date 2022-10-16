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
use druid_widget_nursery::{stack_tooltip::StackTooltip, WidgetExt as _};

fn main() {
    let base = Flex::column()
        .with_child(
            Flex::row()
                .with_child(
                    Label::new("TOP LEFT").stack_tooltip(String::from("TOOLTIP LOREM IPSUM DORUM")),
                )
                .with_flex_spacer(1.0)
                .with_child(StackTooltip::new(
                    Label::new("TOP RIGHT"),
                    String::from("TOOLTIP LOREM IPSUM DORUM"),
                )),
        )
        .with_flex_spacer(1.0)
        .with_child(
            Flex::row()
                .with_child(
                    StackTooltip::new(
                        Label::new("MID LEFT"),
                        String::from("TOOLTIP LOREM IPSUM DORUM"),
                    )
                    .cancel_stack_tooltip(),
                )
                .with_flex_spacer(1.0)
                .with_child(
                    StackTooltip::new(
                        Label::new("MID RIGHT"),
                        String::from("TOOLTIP LOREM IPSUM DORUM"),
                    )
                    .cancel_stack_tooltip(),
                )
                .background(druid::theme::BACKGROUND_LIGHT)
                .stack_tooltip("IGNORE ME"),
        )
        .with_flex_spacer(1.0)
        .with_child(
            Flex::row()
                .with_child(StackTooltip::new(
                    Label::new("BOTTOM LEFT"),
                    String::from("TOOLTIP LOREM IPSUM DORUM"),
                ))
                .with_flex_spacer(1.0)
                .with_child(StackTooltip::new(
                    Label::new("BOTTOM RIGHT"),
                    String::from("TOOLTIP LOREM IPSUM DORUM"),
                )),
        )
        .expand()
        .expand_height();

    let window = WindowDesc::new(base).window_size((1280., 1024.));

    AppLauncher::with_window(window).launch(()).expect("Launch");
}
