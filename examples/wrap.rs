// Copyright 2019 the Druid Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use druid::widget::prelude::*;
use druid::widget::Label;
use druid::{AppLauncher, Color, WidgetExt, WindowDesc};
use druid_widget_nursery::wrap::Wrap;
use druid_widget_nursery::wrap::WrapAlignment;
use druid_widget_nursery::wrap::WrapCrossAlignment;

pub fn main() {
    // describe the main window
    let main_window = WindowDesc::new(build_root_widget());

    // start the application. Here we pass in the application state.
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(())
        .expect("Failed to launch application");
}

fn chip(t: &str) -> impl Widget<()> {
    Label::new(t)
        .padding((10., 5.))
        .background(Color::GREEN)
        .rounded(20.)
}

fn build_root_widget() -> impl Widget<()> {
    let mut w = Wrap::new()
        .spacing(5.)
        .run_spacing(5.)
        .run_alignment(WrapAlignment::Center)
        .cross_alignment(WrapCrossAlignment::Center)
        .alignment(WrapAlignment::Center);

    for _ in 0..30 {
        w.add_child(Box::new(chip("foobar")));
    }
    w
}
