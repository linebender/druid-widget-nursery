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

use druid::{Data, Widget, WidgetPod};

mod table_column_width;
pub use table_column_width::*;

mod flex_table;
pub use flex_table::*;

/// The vertical alignment of the table cell.
///
/// If a widget is smaller than the table cell, this determines
/// where it is positioned.
#[derive(Copy, Clone, Debug, PartialEq)]
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

/// A table row is a horizontal group of widgets.
///
/// All rows in a table must have the same number of children.
pub struct TableRow<T> {
    min_height: Option<f64>,
    vertical_alignment: Option<TableCellVerticalAlignment>,
    children: Vec<WidgetPod<T, Box<dyn Widget<T>>>>,
}

impl<T: Data> Default for TableRow<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Data> TableRow<T> {
    /// Create a new, empty table
    pub fn new() -> Self {
        Self {
            min_height: None,
            children: Vec::new(),
            vertical_alignment: None,
        }
    }

    /// Builder-style method for specifying the table row minimum height.
    pub fn min_height(mut self, min_height: f64) -> Self {
        self.min_height = Some(min_height);
        self
    }

    /// Set the table row minimun height.
    pub fn set_min_height(&mut self, min_height: f64) {
        self.min_height = Some(min_height);
    }

    /// Builder-style method for specifying the childrens' [`TableCellVerticalAlignment`].
    pub fn vertical_alignment(mut self, align: TableCellVerticalAlignment) -> Self {
        self.vertical_alignment = Some(align);
        self
    }

    /// Set the childrens' [`TableCellVerticalAlignment`].
    pub fn set_vertical_alignment(&mut self, align: TableCellVerticalAlignment) {
        self.vertical_alignment = Some(align);
    }

    /// Builder-style variant of [`Self::add_child`].
    pub fn with_child(mut self, child: impl Widget<T> + 'static) -> Self {
        self.add_child(child);
        self
    }

    /// Add a child widget (table cell).
    ///
    /// See also [`Self::with_child`].
    pub fn add_child(&mut self, child: impl Widget<T> + 'static) {
        let child: Box<dyn Widget<T>> = Box::new(child);
        let child = WidgetPod::new(child);
        self.children.push(child);
    }
}
