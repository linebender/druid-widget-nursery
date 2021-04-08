// Copyright 2019 The Druid Authors.
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

use std::time::Duration;

use druid::widget::{Flex, Label, Spinner};
use druid::{AppLauncher, Widget, WidgetExt, WindowDesc};
use druid_widget_nursery::{AsyncDelegate, FutureWidget};
use tokio::time;

fn main() {
    let window = WindowDesc::new(build_root_widget());
    AsyncDelegate::new(AppLauncher::with_window(window))
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
