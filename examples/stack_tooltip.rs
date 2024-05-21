// Copyright 2022 the Druid Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

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
                    .with_crosshair(true)
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
