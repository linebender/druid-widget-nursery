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

use druid::widget::BackgroundBrush;
use druid::{
    BoxConstraints, Color, Data, Env, Event, EventCtx, KeyOrValue, LayoutCtx, LifeCycle,
    LifeCycleCtx, PaintCtx, Point, RenderContext, Size, UpdateCtx, Widget,
};

use super::{ComplexTableColumnWidth, TableCellVerticalAlignment, TableColumnWidth, TableRow};

#[derive(Debug)]
struct TableBorderStyle {
    width: KeyOrValue<f64>,
    color: KeyOrValue<Color>,
}

/// A container with a flexible table layout.
///
/// Uses the flex layout algorithm (like [druid::widget::Flex]) to layout
/// cells in each row.
///
/// # Examples
/// ```
/// # use druid::widget::Label;
/// # use druid::Widget;
/// # use druid_widget_nursery::table::{TableRow, FlexTable};
/// # fn test() -> impl Widget<()> {
/// FlexTable::new()
///     .inner_border(druid::theme::BORDER_LIGHT, 1.)
///     .with_row(
///         TableRow::new()
///             .with_child(Label::new("Row 1 / Column 1"))
///             .with_child(Label::new("Row 2 / Column 2"))
///     )
///     .with_row(
///         TableRow::new()
///             .with_child(Label::new("Row 1 / Column 1"))
///             .with_child(Label::new("Row 2 / Column 2"))
///      )
/// # }
/// ```
pub struct FlexTable<T> {
    default_column_width: ComplexTableColumnWidth,
    default_vertical_alignment: TableCellVerticalAlignment,
    column_widths: Vec<ComplexTableColumnWidth>,
    children: Vec<TableRow<T>>,
    row_border: Option<TableBorderStyle>,
    col_border: Option<TableBorderStyle>,
    background: Option<BackgroundBrush<T>>,
    row_starts: Option<Vec<f64>>,
    col_starts: Option<Vec<f64>>,
}

impl<T: Data> Default for FlexTable<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Data> FlexTable<T> {
    /// Create a new empty table.
    pub fn new() -> Self {
        Self {
            default_column_width: TableColumnWidth::Flex(1.0).into(),
            default_vertical_alignment: TableCellVerticalAlignment::Middle,
            column_widths: Vec::new(),
            children: Vec::new(),
            row_border: None,
            col_border: None,
            row_starts: None,
            col_starts: None,
            background: None,
        }
    }

    /// Builder-style method to set the table background brush
    pub fn background(mut self, brush: impl Into<BackgroundBrush<T>>) -> Self {
        self.set_background(brush);
        self
    }

    /// Set the table background brush
    pub fn set_background(&mut self, brush: impl Into<BackgroundBrush<T>>) {
        self.background = Some(brush.into());
    }

    /// Builder-style method to set the table inner border
    pub fn inner_border(
        mut self,
        color: impl Into<KeyOrValue<Color>>,
        width: impl Into<KeyOrValue<f64>>,
    ) -> Self {
        self.set_inner_border(color, width);
        self
    }

    /// Set the table inner border.
    pub fn set_inner_border(
        &mut self,
        color: impl Into<KeyOrValue<Color>>,
        width: impl Into<KeyOrValue<f64>>,
    ) {
        let color = color.into();
        let width = width.into();
        self.set_row_border(color.clone(), width.clone());
        self.set_column_border(color, width);
    }

    /// Builder-style method to set the table row border.
    pub fn row_border(
        mut self,
        color: impl Into<KeyOrValue<Color>>,
        width: impl Into<KeyOrValue<f64>>,
    ) -> Self {
        self.set_row_border(color, width);
        self
    }

    /// Set the table row border.
    pub fn set_row_border(
        &mut self,
        color: impl Into<KeyOrValue<Color>>,
        width: impl Into<KeyOrValue<f64>>,
    ) {
        self.row_border = Some(TableBorderStyle {
            width: width.into(),
            color: color.into(),
        });
    }

    /// Builder-style method to set the table column border.
    pub fn column_border(
        mut self,
        color: impl Into<KeyOrValue<Color>>,
        width: impl Into<KeyOrValue<f64>>,
    ) -> Self {
        self.set_column_border(color, width);
        self
    }

    /// Set the table column border.
    pub fn set_column_border(
        &mut self,
        color: impl Into<KeyOrValue<Color>>,
        width: impl Into<KeyOrValue<f64>>,
    ) {
        self.col_border = Some(TableBorderStyle {
            width: width.into(),
            color: color.into(),
        });
    }

    /// Builder-style method to add a table column width.
    ///
    /// Examples:
    /// ```
    /// use druid_widget_nursery::table::{FlexTable, TableColumnWidth::*};
    /// # fn test () -> FlexTable<()> {
    ///
    /// FlexTable::new()
    ///    .with_column_width(64.0)                     // column 1: fixed width
    ///    .with_column_width(Intrinsic)                // column 2: intrinsic width
    ///    .with_column_width((Flex(1.0), 60.0))        // column 3: flex with minimum size
    ///    .with_column_width((Flex(1.0), 60.0..200.0)) // column 4: flex with minimum and maximum
    ///
    ///    // column 5: flex with intrinsic size as minimum and fixed size maximum
    ///    .with_column_width((Flex(1.0), Intrinsic, 200.0))
    /// # }
    /// ```
    pub fn with_column_width<W: Into<ComplexTableColumnWidth>>(mut self, column_width: W) -> Self {
        self.column_widths.push(column_width.into());
        self
    }

    /// Builder-style method to set the table column width.
    ///
    /// If not set, the [`Self::default_column_width`] is used.
    pub fn column_widths(mut self, column_widths: &[ComplexTableColumnWidth]) -> Self {
        self.set_column_widths(column_widths);
        self
    }

    /// Set the table column widths.
    ///
    /// If not set, the [`Self::default_column_width`] is used.
    pub fn set_column_widths(&mut self, column_widths: &[ComplexTableColumnWidth]) {
        self.column_widths = column_widths.to_vec();
    }

    /// Builder-style method to set the default column width.
    pub fn default_column_width<W: Into<ComplexTableColumnWidth>>(
        mut self,
        default_column_width: W,
    ) -> Self {
        self.set_default_column_width(default_column_width);
        self
    }

    /// Set the default column width.
    pub fn set_default_column_width<W: Into<ComplexTableColumnWidth>>(
        &mut self,
        default_column_width: W,
    ) {
        self.default_column_width = default_column_width.into();
    }

    /// Builder-style method to set the default vertical cell alignment.
    pub fn default_vertical_alignment(
        mut self,
        default_vertical_alignment: TableCellVerticalAlignment,
    ) -> Self {
        self.set_default_vertical_alignment(default_vertical_alignment);
        self
    }

    /// Set the default vertical cell alignment.
    pub fn set_default_vertical_alignment(
        &mut self,
        default_vertical_alignment: TableCellVerticalAlignment,
    ) {
        self.default_vertical_alignment = default_vertical_alignment;
    }

    /// Returns the column count
    pub fn column_count(&self) -> usize {
        if self.children.is_empty() {
            0
        } else {
            self.children[0].children.len()
        }
    }

    /// Builder-style method to add a table row.
    ///
    /// All row must have equal number of cells. Panics if not!
    pub fn with_row(mut self, row: TableRow<T>) -> Self {
        self.add_row(row);
        self
    }

    /// Add a table row.
    ///
    /// All row must have equal number of cells. Panics if not!
    pub fn add_row(&mut self, row: TableRow<T>) {
        if !self.children.is_empty() && row.children.len() != self.column_count() {
            panic!("Table::add_row - wrong row length");
        }
        self.children.push(row);
    }
}

impl<T: Data> Widget<T> for FlexTable<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        for row in self.children.iter_mut() {
            for cell in row.children.iter_mut() {
                cell.event(ctx, event, data, env);
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        for row in self.children.iter_mut() {
            for cell in row.children.iter_mut() {
                cell.lifecycle(ctx, event, data, env);
            }
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        if let Some(brush) = self.background.as_mut() {
            brush.update(ctx, old_data, data, env);
        }

        if let Some(border) = &self.row_border {
            if ctx.env_key_changed(&border.width) {
                ctx.request_layout();
            }
            if ctx.env_key_changed(&border.color) {
                ctx.request_paint();
            }
        }

        if let Some(border) = &self.col_border {
            if ctx.env_key_changed(&border.width) {
                ctx.request_layout();
            }
            if ctx.env_key_changed(&border.color) {
                ctx.request_paint();
            }
        }

        for row in self.children.iter_mut() {
            for cell in row.children.iter_mut() {
                cell.update(ctx, data, env);
            }
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let column_count = self.column_count();
        if column_count == 0 {
            return Size::ZERO;
        }

        if self.column_widths.len() < column_count {
            // make sure we have all elements so that we can directly
            // access column_widths[col_num]
            self.column_widths
                .resize(column_count, self.default_column_width);
        }

        let mut column_widths = self.column_widths.clone();

        let mut intrinsic_widths = vec![0f64; column_count];
        let mut row_starts = vec![0f64; self.children.len()];

        let col_border_width = self
            .col_border
            .as_ref()
            .map(|b| b.width.resolve(env))
            .unwrap_or(0f64);
        let col_border_width_sum = col_border_width * (column_count - 1) as f64;
        let max_table_width = bc.max().width - col_border_width_sum;

        let rows = self.children.len();
        let row_border_width = self
            .row_border
            .as_ref()
            .map(|b| b.width.resolve(env))
            .unwrap_or(0f64);
        let row_border_width_sum = row_border_width * (rows - 1) as f64;
        let max_table_height = bc.max().height - row_border_width_sum;

        use TableColumnWidth::*;

        // pass 1: compute intrinsic sizes if needed
        for col_num in 0..column_count {
            let cw = column_widths[col_num];
            if cw.need_intrinsic_width() {
                let mut row_width = 0f64;
                let mut found_size = false;
                for row in self.children.iter_mut() {
                    if let Some(cell) = row.children.get_mut(col_num) {
                        let child_bc = BoxConstraints::new(
                            Size::new(0., 0.),
                            Size::new(std::f64::INFINITY, std::f64::INFINITY),
                        );
                        let size = cell.layout(ctx, &child_bc, data, env);
                        if size.width.is_finite() {
                            row_width = row_width.max(size.width);
                            found_size = true;
                        }
                    }
                }
                if found_size {
                    intrinsic_widths[col_num] = row_width;
                } else {
                    let flex = 1.0;
                    column_widths[col_num] = match column_widths[col_num] {
                        ComplexTableColumnWidth::Simple(_) => Flex(flex).into(),
                        ComplexTableColumnWidth::Limited(_, min, max) => {
                            ComplexTableColumnWidth::Limited(Flex(flex), min, max)
                        }
                    };
                }
            }
        }

        let col_widths = ComplexTableColumnWidth::compute_column_widths(
            &mut column_widths,
            &intrinsic_widths,
            max_table_width,
        );

        let table_width = col_widths.iter().sum::<f64>() + col_border_width_sum;
        let mut table_height = 0f64;

        for (row_num, row) in self.children.iter_mut().enumerate() {
            let mut row_height = 0f64;
            let mut found_height = false;
            let mut max_above_baseline = 0f64;
            let mut max_below_baseline = 0f64;

            let mut fix_columns = Vec::new();

            if row_num > 0 {
                table_height += row_border_width;
            }

            for (col_num, cell) in row.children.iter_mut().enumerate() {
                let child_bc = BoxConstraints::new(
                    Size::new(0., 0.),
                    Size::new(col_widths[col_num], std::f64::INFINITY),
                );
                let size = cell.layout(ctx, &child_bc, data, env);

                if size.height.is_finite() {
                    found_height = true;
                    row_height = row_height.max(size.height);
                    let baseline_offset = cell.baseline_offset();
                    let above_baseline = size.height - baseline_offset;

                    max_above_baseline = max_above_baseline.max(above_baseline);
                    max_below_baseline = max_below_baseline.max(baseline_offset);
                } else {
                    fix_columns.push(col_num);
                }
            }

            if !found_height {
                // all children have INF height
                row_height = max_table_height;
            }

            for col_num in fix_columns {
                if let Some(cell) = row.children.get_mut(col_num) {
                    let child_bc = BoxConstraints::new(
                        Size::new(0., 0.),
                        Size::new(col_widths[col_num], row_height),
                    );
                    let size = cell.layout(ctx, &child_bc, data, env);

                    let baseline_offset = cell.baseline_offset();
                    let above_baseline = size.height - baseline_offset;

                    max_above_baseline = max_above_baseline.max(above_baseline);
                    max_below_baseline = max_below_baseline.max(baseline_offset);
                }
            }

            let real_height = row
                .min_height
                .unwrap_or(0f64)
                .max(max_above_baseline + max_below_baseline);

            let mut row_width = 0f64;
            for (col_num, cell) in row.children.iter_mut().enumerate() {
                if col_num > 0 {
                    row_width += col_border_width;
                }
                let size = cell.layout_rect().size();

                let vertical_alignment = row
                    .vertical_alignment
                    .unwrap_or(self.default_vertical_alignment);

                let dh = match vertical_alignment {
                    TableCellVerticalAlignment::Baseline => {
                        let baseline_offset = cell.baseline_offset();

                        let above_baseline = size.height - baseline_offset;

                        max_above_baseline - above_baseline
                    }
                    TableCellVerticalAlignment::Top => 0f64,
                    TableCellVerticalAlignment::Bottom => (real_height - size.height).max(0.0),
                    TableCellVerticalAlignment::Middle => {
                        (real_height - size.height).max(0.0) / 2.0
                    }
                    TableCellVerticalAlignment::Fill => {
                        if size.height < real_height {
                            let child_bc =
                                BoxConstraints::tight(Size::new(size.width, real_height));
                            let _size = cell.layout(ctx, &child_bc, data, env);
                        }
                        0f64
                    }
                };

                let child_pos = Point::new(row_width, table_height + dh);
                cell.set_origin(ctx, data, env, child_pos);
                row_width += col_widths[col_num];
            }

            row_starts[row_num] = table_height;
            table_height += real_height;
        }

        // Note: Convert col_widths to start offset
        let mut col_starts = col_widths;
        let mut col_start = 0f64;
        for (i, width) in col_starts.iter_mut().enumerate() {
            if i > 0 {
                col_start += col_border_width;
            }
            let old_width = *width;
            *width = col_start;
            col_start += old_width;
        }

        self.col_starts = Some(col_starts);
        self.row_starts = Some(row_starts);

        Size::new(table_width, table_height)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        let mut row_border_width = 0f64;
        let mut half_row_border_width = 0f64;
        let mut row_border_color = Color::WHITE; // not used
        let mut col_border_width = 0f64;
        let mut half_col_border_width = 0f64;
        let mut col_border_color = Color::WHITE; // not used

        if let Some(row_border) = &self.row_border {
            row_border_width = row_border.width.resolve(env);
            half_row_border_width = row_border_width / 2.0;
            row_border_color = row_border.color.resolve(env);
        }

        if let Some(col_border) = &self.col_border {
            col_border_width = col_border.width.resolve(env);
            half_col_border_width = col_border_width / 2.0;
            col_border_color = col_border.color.resolve(env);
        }

        let size = ctx.size();

        use druid::kurbo::Line;

        if let Some(background) = self.background.as_mut() {
            let panel = size.to_rect();
            ctx.with_save(|ctx| {
                ctx.clip(panel);
                background.paint(ctx, data, env);
            });
        }

        for (row_num, row) in self.children.iter_mut().enumerate() {
            if row_num > 0 && row_border_width > 0.0 {
                if let Some(ref row_starts) = self.row_starts {
                    let row_start = row_starts[row_num] - half_row_border_width;
                    let start = Point::new(0.0, row_start);
                    let end = Point::new(size.width, row_start);
                    let line = Line::new(start, end);
                    ctx.stroke(line, &row_border_color, row_border_width);
                }
            }

            for (col_num, cell) in row.children.iter_mut().enumerate() {
                if col_num > 0 && col_border_width > 0.0 {
                    if let Some(ref col_starts) = self.col_starts {
                        let col_start = col_starts[col_num] - half_col_border_width;
                        let start = Point::new(col_start, size.height);
                        let end = Point::new(col_start, 0.0);
                        let line = Line::new(start, end);
                        ctx.stroke(line, &col_border_color, col_border_width);
                    }
                }

                cell.paint(ctx, data, env);
            }
        }
    }
}
