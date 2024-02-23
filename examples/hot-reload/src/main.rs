// Copyright 2021 the Druid Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use druid_widget_nursery::hot_reload::{AppLauncher, WindowDesc};
use hot_reload::AppData;

fn main() {
    let window = WindowDesc::new("target/debug/hot_reload.dll", "view");
    unsafe {
        AppLauncher::with_window(window)
            .launch(AppData::default())
            .unwrap();
    }
}
