// Copyright 2021 The Druid Authors.
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
//
// Author: Dietmar Maurer <dietmar@proxmox.com>

use druid::widget::{
    Align, Container, CrossAxisAlignment, Flex, Label, Stepper, TextBox, WidgetExt,
};
use druid::{theme, AppLauncher, Color, Data, Env, Lens, UnitPoint, Widget, WindowDesc};

use druid_widget_nursery::table::{
    FlexTable, TableCellVerticalAlignment, TableColumnWidth, TableRow,
};

#[derive(Clone, Data, Default, Lens)]
struct DemoState {
    pub input_username: String,
    pub input_password: String,
    age: f64,
}

fn make_imput_form_example() -> impl Widget<DemoState> {
    use TableColumnWidth::*;

    FlexTable::new()
        .inner_border(theme::BORDER_LIGHT, 1.)
        .with_column_width(Intrinsic)
        .with_column_width((Flex(1.), 100.))
        .with_row(
            TableRow::new()
                .with_child(Label::new("Username:").align_horizontal(UnitPoint::RIGHT))
                .with_child(
                    TextBox::new()
                        .with_placeholder("Username")
                        .expand_width()
                        .lens(DemoState::input_username),
                ),
        )
        .with_row(
            TableRow::new()
                .with_child(Label::new("Password:").align_horizontal(UnitPoint::RIGHT))
                .with_child(
                    TextBox::new()
                        .with_placeholder("Password")
                        .expand_width()
                        .lens(DemoState::input_password),
                ),
        )
        .border(Color::WHITE, 1.)
}

fn make_row_alignment_example() -> impl Widget<DemoState> {
    let mut table = FlexTable::new()
        .inner_border(theme::BORDER_LIGHT, 1.)
        .default_column_width(TableColumnWidth::Intrinsic);

    use TableCellVerticalAlignment::*;

    let mut row = TableRow::new().vertical_alignment(Baseline);

    row.add_child(Label::new("Baseline"));

    row.add_child(Label::new("Your name is:"));

    row.add_child(
        TextBox::new()
            .with_placeholder("Username")
            .expand_width()
            .lens(DemoState::input_username),
    );

    row.add_child(Label::new("Your age is:").lens(DemoState::age));
    row.add_child(Label::new(|v: &f64, _: &Env| v.to_string()).lens(DemoState::age));
    row.add_child(
        Stepper::new()
            .with_range(0.0, 120.0)
            .with_step(1.0)
            .with_wraparound(true)
            .lens(DemoState::age),
    );

    table.add_row(row);

    for align in [Bottom, Middle, Top, Fill] {
        let mut row = TableRow::new().min_height(40.).vertical_alignment(align);

        row.add_child(
            Label::new(format!("{align:?}"))
                .with_text_color(Color::BLACK)
                .background(theme::BORDER_LIGHT)
                .border(Color::WHITE, 1.)
                .center()
                .expand_width()
                .background(Color::rgb(0.4, 0.4, 0.4)),
        );

        for row_num in 1..6 {
            row.add_child(
                Label::new(format!("{row_num}"))
                    .with_text_color(Color::BLACK)
                    .background(theme::BORDER_LIGHT)
                    .border(Color::WHITE, 1.)
                    .center()
                    .expand_width()
                    .background(Color::rgb(0.4, 0.4, 0.4)),
            );
        }

        table.add_row(row);
    }

    table.border(Color::WHITE, 1.)
}

fn make_cell_alignment_example() -> impl Widget<DemoState> {
    let alignments = [
        UnitPoint::TOP_LEFT,
        UnitPoint::TOP,
        UnitPoint::TOP_RIGHT,
        UnitPoint::RIGHT,
        UnitPoint::BOTTOM_RIGHT,
        UnitPoint::BOTTOM,
        UnitPoint::BOTTOM_LEFT,
        UnitPoint::LEFT,
        UnitPoint::CENTER,
    ];

    let mut row = TableRow::new().min_height(40.);

    for (i, alignment) in alignments.iter().enumerate() {
        row.add_child(
            Align::new(*alignment, {
                let label = Label::new(format!("{i}"))
                    .with_text_color(Color::BLACK)
                    .center();
                Container::new(label)
                    .background(theme::BORDER_LIGHT)
                    .border(Color::WHITE, 1.)
                    .fix_width(20.)
                    .fix_height(20.)
            })
            .expand(),
        );
    }

    FlexTable::new()
        .inner_border(theme::BORDER_LIGHT, 1.)
        .with_row(row)
        .border(Color::WHITE, 1.)
        .fix_height(42.)
}

fn make_ui() -> impl Widget<DemoState> {
    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Fill)
        .must_fill_main_axis(true)
        .with_child(Label::new("A simple input form."))
        .with_child(make_imput_form_example())
        .with_default_spacer()
        .with_child(Label::new("Row alignment example."))
        .with_child(make_row_alignment_example())
        .with_default_spacer()
        .with_child(Label::new("Cell alignment example."))
        .with_child(make_cell_alignment_example())
        .with_default_spacer()
        .padding(10.0)
        .fix_height(500.)
}

pub fn main() {
    let main_window = WindowDesc::new(make_ui())
        .window_size((500., 500.))
        .with_min_size((50., 50.))
        .title("Flex Table Example");

    let demo_state = DemoState::default();

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(demo_state)
        .expect("Failed to launch application");
}
