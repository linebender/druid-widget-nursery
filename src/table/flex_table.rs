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

use std::collections::HashMap;

use druid::{
    widget::BackgroundBrush, BoxConstraints, Color, Env, Event, EventCtx, Key, KeyOrValue,
    LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Point, RenderContext, Size, UpdateCtx, Widget,
    WidgetPod,
};

use super::{
    ComplexTableColumnWidth, RowData, TableCellVerticalAlignment, TableColumnWidth, TableData,
    TableRowInternal,
};

#[derive(Debug)]
pub(crate) struct TableBorderStyle {
    pub(crate) width: KeyOrValue<f64>,
    pub(crate) color: KeyOrValue<Color>,
}

/// A container with a flexible table layout.
///
/// Uses the flex layout algorithm (like [druid::widget::Flex]) to layout
/// cells in each row.
///
/// # Examples
/// ```
/// # use std::ops::{Index, IndexMut};
/// # use druid::widget::Label;
/// # use druid::{Data, lens::Constant, im::{vector, Vector}, Widget, WidgetExt};
/// # use druid_widget_nursery::table::{FlexTable, TableData, RowData};
///
/// #[derive(Clone, Data)]
/// struct Row {
///     row: usize,
/// };
///
/// impl RowData for Row {
///     type Id = usize;
///     type Column = usize;
///
///     fn id(&self) -> Self::Id {
///         self.row
///     }
///
///     fn cell(&self, column: &Self::Column) -> Box<dyn Widget<Self>> {
///         let column = *column;
///         Label::dynamic(move |data: &Row, _| format!("Row {} / Column {}", data.row, column)).boxed()
///     }
/// }
///
/// #[derive(Clone, Data)]
/// struct Table {
///     children: Vector<Row>,
///     width: usize,
/// }
///
/// impl Index<usize> for Table {
///     type Output = Row;
///
///     fn index(&self, row: usize) -> &Self::Output {
///         &self.children[row]
///     }
/// }
///
/// impl IndexMut<usize> for Table {
///     fn index_mut(&mut self, row: usize) -> &mut Self::Output {
///         &mut self.children[row]
///     }
/// }
///
/// impl TableData for Table {
///     type Row = Row;
///     type Column = usize;
///
///     fn keys(&self) -> impl Iterator<Item = <Self::Row as RowData>::Id> {
///         0..self.children.len()
///     }
///
///     fn columns(&self) -> impl Iterator<Item = Self::Column> {
///         if self.children.is_empty() {
///             0..0
///         } else {
///             0..self.width
///         }
///     }
/// }
///
/// # fn test() -> impl Widget<Table> {
/// FlexTable::new()
///     .inner_border(druid::theme::BORDER_LIGHT, 1.)
///     .lens(Constant(Table {
///         children: vector!(
///             Row {
///                 row: 1,
///             },
///             Row {
///                 row: 2,
///             }
///         ),
///         width: 2,
///     }))
/// # }
/// ```
pub struct FlexTable<T: TableData> {
    pub(crate) default_column_width: ComplexTableColumnWidth,
    pub(crate) default_vertical_alignment: TableCellVerticalAlignment,
    pub(crate) column_widths: Vec<ComplexTableColumnWidth>,
    pub(crate) children: HashMap<<T::Row as RowData>::Id, TableRowInternal<T::Row>>,
    pub(crate) row_border: Option<TableBorderStyle>,
    pub(crate) col_border: Option<TableBorderStyle>,
    pub(crate) background: Option<BackgroundBrush<T>>,
    pub(crate) row_starts: Option<Vec<f64>>,
    pub(crate) col_starts: Option<Vec<f64>>,
    pub(crate) row_background: Option<BackgroundBrush<T>>,
}

impl<T: TableData> Default for FlexTable<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: TableData> FlexTable<T> {
    pub const ROW_IDX: Key<u64> = Key::new("druid_widget_nursery_fork.flex_table.row_number");
    pub const COL_IDX: Key<u64> = Key::new("druid_widget_nursery_fork.flex_table.row_number");
    pub const TOTAL_COLUMNS: Key<u64> = Key::new("druid_widget_nursery_fork.flex_table.row_number");

    /// Create a new empty table.
    pub fn new() -> Self {
        Self {
            default_column_width: TableColumnWidth::Flex(1.0).into(),
            default_vertical_alignment: TableCellVerticalAlignment::Middle,
            column_widths: Vec::new(),
            children: HashMap::new(),
            row_border: None,
            col_border: None,
            row_starts: None,
            col_starts: None,
            background: None,
            row_background: None,
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

    /// Builder-style method to set the row background brush
    pub fn row_background(mut self, brush: impl Into<BackgroundBrush<T>>) -> Self {
        self.set_row_background(brush);
        self
    }

    /// Set the row background brush
    pub fn set_row_background(&mut self, brush: impl Into<BackgroundBrush<T>>) {
        self.row_background = Some(brush.into());
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
    /// # fn test () -> FlexTable<[(); 0]> {
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
            self.children.values().next().unwrap().children.len()
        }
    }

    /// Add a table row.
    pub(crate) fn insert_row(&mut self, row: TableRowInternal<T::Row>) {
        self.children.insert(row.id().clone(), row);
    }

    /// Return a mutable reference to column widths to allow them to modified at runtime
    pub fn get_column_widths(&mut self) -> &mut Vec<ComplexTableColumnWidth> {
        &mut self.column_widths
    }

    /// Clear the table of all rows and thus cells
    pub fn clear(&mut self) {
        self.children.clear();
        self.row_starts = None;
    }

    pub fn cell_from_closure(
        maker: impl Fn() -> Box<dyn Widget<()>> + 'static,
    ) -> std::sync::Arc<dyn Fn() -> Box<dyn Widget<()>>> {
        std::sync::Arc::new(maker) as std::sync::Arc<dyn Fn() -> Box<dyn Widget<()>>>
    }
}

impl<T: TableData> Widget<T> for FlexTable<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        let keys: Vec<_> = data.keys().collect();
        for (row_num, row_id) in keys.into_iter().enumerate() {
            let columns: Vec<_> = data.columns().collect();
            if let Some(row) = self.children.get_mut(&row_id) {
                let row_data = &mut data[row_id];
                for column in columns {
                    if let Some(cell) = &mut row.children.get_mut(&column) {
                        let env = env.clone().adding(Self::ROW_IDX, row_num as u64);
                        cell.event(ctx, event, row_data, &env);
                    }
                }
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        let columns: Vec<_> = data.columns().collect();
        if let LifeCycle::WidgetAdded = event {
            let mut changed = false;
            for row_data in data.keys().map(|k| &data[k]) {
                let row_id = row_data.id();
                let row = if let Some(row) = self.children.get_mut(&row_id) {
                    row
                } else {
                    changed = true;
                    self.insert_row(TableRowInternal::new(row_data.id().clone()));
                    self.children.get_mut(&row_id).unwrap()
                };

                for column in &columns {
                    if !row.children().contains_key(column) {
                        changed = true;
                        row.children()
                            .insert(column.clone(), WidgetPod::new(row_data.cell(column)));
                    }
                }
            }
            if changed {
                ctx.children_changed();
            }
        }

        let keys: Vec<_> = data.keys().collect();
        for (row_num, row_id) in keys.into_iter().enumerate() {
            if let Some(row) = self.children.get_mut(&row_id) {
                let row_data = &data[row_id];
                for column in &columns {
                    if let Some(cell) = row.children.get_mut(column) {
                        let env = env.clone().adding(Self::ROW_IDX, row_num as u64);
                        cell.lifecycle(ctx, event, row_data, &env);
                    }
                }
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

        let columns: Vec<_> = data.columns().collect();

        if self.column_widths.len() != columns.len() {
            self.column_widths.resize_with(columns.len(), || {
                ComplexTableColumnWidth::Simple(TableColumnWidth::Flex(1.0))
            })
        }

        for (row_num, row_id) in data.keys().enumerate() {
            if let Some(row) = self.children.get_mut(&row_id) {
                let row_data = &data[row_id];
                for column in &columns {
                    if let Some(cell) = row.children.get_mut(column) {
                        let env = env.clone().adding(Self::ROW_IDX, row_num as u64);
                        cell.update(ctx, row_data, &env)
                    } else {
                        row.children
                            .insert(column.clone(), WidgetPod::new(row_data.cell(column)));
                        ctx.children_changed()
                    }
                }
            } else {
                let mut row = TableRowInternal::new(row_id.clone());
                let row_data = &data[row_id];
                row.children = data
                    .columns()
                    .map(|c| (c.clone(), WidgetPod::new(row_data.cell(&c))))
                    .collect();
                self.insert_row(row);
                ctx.children_changed();
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
        let mut row_starts = vec![0f64; data.keys().count()];

        let col_border_width = self
            .col_border
            .as_ref()
            .map(|b| b.width.resolve(env))
            .unwrap_or(0f64);
        let col_border_width_sum = col_border_width * (column_count - 1) as f64;
        let max_table_width = bc.max().width - col_border_width_sum;

        let rows = data.keys().count();
        let row_border_width = self
            .row_border
            .as_ref()
            .map(|b| b.width.resolve(env))
            .unwrap_or(0f64);
        let row_border_width_sum = row_border_width * (rows.saturating_sub(1)) as f64;
        let max_table_height = bc.max().height - row_border_width_sum;

        use TableColumnWidth::*;

        // pass 1: compute intrinsic sizes if needed
        for (col_num, column) in data.columns().enumerate() {
            let cw = column_widths[col_num];
            if cw.need_intrinsic_width() {
                let mut row_width = 0f64;
                let mut found_size = false;
                let keys: Vec<_> = data.keys().collect();
                for (row_num, row_id) in keys.into_iter().enumerate() {
                    let row = self.children.get_mut(&row_id).unwrap();
                    let row_data = &data[row_id];
                    if let Some(cell) = row.children.get_mut(&column) {
                        let child_bc = BoxConstraints::new(
                            Size::new(0., 0.),
                            Size::new(std::f64::INFINITY, std::f64::INFINITY),
                        );

                        let env = env.clone().adding(Self::ROW_IDX, row_num as u64);
                        let size = cell.layout(ctx, &child_bc, row_data, &env);
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

        let keys: Vec<_> = data.keys().collect();
        for (row_num, row_id) in keys.into_iter().enumerate() {
            let mut row_height = 0f64;
            let mut found_height = false;
            let mut max_above_baseline = 0f64;
            let mut max_below_baseline = 0f64;

            let mut fix_columns = Vec::new();

            if row_num > 0 {
                table_height += row_border_width;
            }

            let columns: Vec<_> = data.columns().collect();
            let row = self.children.get_mut(&row_id).unwrap();
            let row_data = &data[row_id];
            for (col_num, column) in columns.iter().enumerate() {
                let child_bc = BoxConstraints::new(
                    Size::new(0., 0.),
                    Size::new(col_widths[col_num], std::f64::INFINITY),
                );

                if let Some(cell) = row.children.get_mut(column) {
                    let env = env.clone().adding(Self::ROW_IDX, row_num as u64);
                    let size = cell.layout(ctx, &child_bc, row_data, &env);

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
            }

            if !found_height {
                // all children have INF height
                row_height = max_table_height;
            }

            for col_num in fix_columns {
                if let Some(cell) = data
                    .columns()
                    .nth(col_num)
                    .and_then(|col| row.children.get_mut(&col))
                {
                    let child_bc = BoxConstraints::new(
                        Size::new(0., 0.),
                        Size::new(col_widths[col_num], row_height),
                    );
                    let size = cell.layout(ctx, &child_bc, row_data, env);

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
            for (col_num, column) in columns.iter().enumerate() {
                let vertical_alignment = row
                    .vertical_alignment
                    .unwrap_or(self.default_vertical_alignment);
                if let Some(cell) = row.children().get_mut(column) {
                    if col_num > 0 {
                        row_width += col_border_width;
                    }
                    let size = cell.layout_rect().size();

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
                                let _size = cell.layout(ctx, &child_bc, row_data, env);
                            }
                            0f64
                        }
                    };

                    let child_pos = Point::new(row_width, table_height + dh);
                    cell.set_origin(ctx, child_pos);
                    row_width += col_widths[col_num];
                }
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

        let keys: Vec<_> = data.keys().collect();
        for (row_num, row_id) in keys.into_iter().enumerate() {
            if let Some(ref row_starts) = self.row_starts {
                if row_num > 0 && row_border_width > 0.0 {
                    let row_start = row_starts[row_num] - half_row_border_width;
                    let start = Point::new(0.0, row_start);
                    let end = Point::new(size.width, row_start);
                    let line = Line::new(start, end);
                    ctx.stroke(line, &row_border_color, row_border_width);
                }

                if let Some(row_painter) = self.row_background.as_mut() {
                    let mut row_size = ctx.size();
                    row_size.height = row_starts.get(row_num + 1).unwrap_or(&row_size.height)
                        - row_starts[row_num];
                    let row_rect = row_size.to_rect().with_origin((0.0, row_starts[row_num]));
                    ctx.with_save(|ctx| {
                        ctx.clip(row_rect);
                        let env = env.clone().adding(Self::ROW_IDX, row_num as u64);
                        row_painter.paint(ctx, data, &env);
                    })
                }
            }

            let columns: Vec<_> = data.columns().collect();
            let row = self.children.get_mut(&row_id).unwrap();
            let row_data = &data[row_id];
            let column_count = columns.len() as u64;
            for (col_num, column) in columns.iter().enumerate() {
                if let Some(cell) = row.children.get_mut(column) {
                    if col_num > 0 && col_border_width > 0.0 {
                        if let Some(ref col_starts) = self.col_starts {
                            let col_start = col_starts[col_num] - half_col_border_width;
                            let start = Point::new(col_start, size.height);
                            let end = Point::new(col_start, 0.0);
                            let line = Line::new(start, end);
                            ctx.stroke(line, &col_border_color, col_border_width);
                        }
                    }

                    let env = env
                        .clone()
                        .adding(Self::ROW_IDX, row_num as u64)
                        .adding(Self::COL_IDX, col_num as u64)
                        .adding(Self::TOTAL_COLUMNS, column_count);
                    cell.paint(ctx, row_data, &env);
                }
            }
        }
    }
}
