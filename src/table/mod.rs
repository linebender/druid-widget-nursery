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

use druid::{Widget, WidgetPod};

mod table_column_width;
pub use table_column_width::*;

mod table_data;
pub use table_data::*;

mod flex_table;
pub use flex_table::*;

mod fixed;
pub use fixed::*;

/// The vertical alignment of the table cell.
///
/// If a widget is smaller than the table cell, this determines
/// where it is positioned.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TableCellVerticalAlignment {
    /// Align on the baseline.
    ///
    /// Widgets are aligned along the calculated baseline.
    Baseline,
    /// Align on top.
    Top,
    /// Align on bottom.
    Bottom,
    /// Fill the available space.
    ///
    /// The height is the size of the largest widget in the table row.
    /// other widgets must fill that space.
    Fill,
    /// Cells are vertically centered.
    Middle,
}

pub type TableChildren<T> = HashMap<<T as RowData>::Column, WidgetPod<T, Box<dyn Widget<T>>>>;

/// A table row is a horizontal group of widgets.
///
/// All rows in a table must have the same number of children.
pub(crate) struct TableRowInternal<T: RowData> {
    id: T::Id,
    min_height: Option<f64>,
    vertical_alignment: Option<TableCellVerticalAlignment>,
    children: TableChildren<T>,
}

impl<T: RowData> Default for TableRowInternal<T>
where
    T::Id: Default,
{
    fn default() -> Self {
        Self::new(T::Id::default())
    }
}

impl<T: RowData> TableRowInternal<T> {
    /// Create a new, empty table
    pub fn new(id: T::Id) -> Self {
        Self {
            id,
            min_height: None,
            children: HashMap::new(),
            vertical_alignment: None,
        }
    }

    pub fn children(&mut self) -> &mut TableChildren<T> {
        &mut self.children
    }

    pub fn id(&self) -> &T::Id {
        &self.id
    }
}
