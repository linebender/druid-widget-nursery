// Copyright 2019 the Druid Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use druid::widget::{CrossAxisAlignment, Flex, Label, Scroll};
use druid::{AppLauncher, Data, Env, Insets, Lens, Widget, WidgetExt, WindowDesc};
use druid_widget_nursery::{DropdownSelect, ListSelect};

#[derive(Clone, Copy, Data, Debug, PartialEq)]
enum Destination {
    Sydney,
    Petaluma,
    Tokyo,
    Paris,
}

//#[derive(Clone, Copy, Data, Debug, PartialEq)]
#[derive(Data, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
enum Transportation {
    Car,
    Train,
    Plane,
    Submarine,
}

#[derive(Data, Clone, Lens)]
struct AppData {
    destination: Destination,
    transportation: Transportation,
}

fn main_widget() -> impl Widget<AppData> {
    let mut row = Flex::row().cross_axis_alignment(CrossAxisAlignment::Start);
    row.add_flex_child(
        Scroll::new(
            ListSelect::new(vec![
                ("to Sydney", Destination::Sydney),
                ("to Petaluma", Destination::Petaluma),
                ("to Tokyo", Destination::Tokyo),
                ("to Paris", Destination::Paris),
            ])
            .on_select(|_, item, _| println!("Selected destination: {item:?}")),
        )
        .vertical()
        .lens(AppData::destination),
        1.0,
    );
    row.add_default_spacer();
    row.add_flex_child(
        DropdownSelect::new(vec![
            ("by car", Transportation::Car),
            ("by train", Transportation::Train),
            ("by plane", Transportation::Plane),
            ("in a yellow submarine", Transportation::Submarine),
        ])
        .align_left()
        .lens(AppData::transportation),
        1.0,
    );

    let mut col = Flex::column().cross_axis_alignment(CrossAxisAlignment::Start);
    col.add_child(
        Label::new(|d: &AppData, _: &Env| {
            format!("Let's go to {:?} by {:?}", d.destination, d.transportation)
        })
        .padding(Insets::uniform_xy(5., 5.)),
    );
    col.add_child(row);
    col
}

fn main() {
    let main_window = WindowDesc::new(main_widget())
        .title("Select")
        .window_size((250., 300.));

    // create the initial app state
    let app_data = AppData {
        transportation: Transportation::Car,
        destination: Destination::Tokyo,
    };

    // start the application
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(app_data)
        .expect("Failed to launch application");
}
