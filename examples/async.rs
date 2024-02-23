// Copyright 2019 the Druid Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::time::Duration;

use druid::widget::{Flex, Label, Spinner};
use druid::{AppLauncher, Widget, WidgetExt, WindowDesc};
use druid_widget_nursery::FutureWidget;
use tokio::time;

#[tokio::main]
async fn main() {
    let window = WindowDesc::new(build_root_widget());
    AppLauncher::with_window(window)
        .log_to_console()
        .launch(())
        .unwrap();
}

fn build_root_widget() -> impl Widget<()> {
    FutureWidget::new(
        |_data, _env| async {
            time::sleep(Duration::from_millis(5000)).await;
            2021
        },
        Flex::column()
            .with_child(Spinner::new())
            .with_spacer(10.0)
            .with_child(Label::new("Loading ...")),
        |value, _data, _env| {
            // data is mut and value is owned
            Label::new(format!("Your number is {}", value)).boxed()
        },
    )
    .center()
}
