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

use std::ops::Range;

/// Simple table column width specification (without minimin and maximum).
///
/// If you need minimum/maximum settings use [`ComplexTableColumnWidth`].
#[derive(Copy, Clone, Debug)]
pub enum TableColumnWidth {
    /// Fixed size
    Fixed(f64),
    /// Flex layout algorithm.
    Flex(f64),
    /// Use a fraction of the total table width.
    Fraction(f64),
    /// Maximum of the dimensions of all cells in a column.
    Intrinsic,
}

impl TableColumnWidth {
    fn resolve_width(
        &self,
        total_width: f64,
        intrinsic_width: f64,
        px_per_flex: f64,
    ) -> (f64, f64, f64) {
        match self {
            Self::Fixed(w) => (*w, *w, 0f64),
            Self::Flex(f) => (f * px_per_flex, 0f64, *f),
            Self::Fraction(f) => {
                let w = f * total_width;
                (w, w, 0f64)
            }
            Self::Intrinsic => (intrinsic_width, intrinsic_width, 0f64),
        }
    }

    fn need_intrinsic_width(&self) -> bool {
        matches!(self, Self::Intrinsic)
    }
}

impl From<f64> for TableColumnWidth {
    fn from(fixed_width: f64) -> Self {
        TableColumnWidth::Fixed(fixed_width)
    }
}

/// Table column width with optional minimum and maximum limits.
///
/// ```
/// use druid_widget_nursery::table::{ComplexTableColumnWidth, TableColumnWidth::*};
///
/// // flex layout, but with minimum and maximum size
/// ComplexTableColumnWidth::with_min_max(Flex(1.), Fixed(100.), Fixed(300.));
///
/// // flex layout, but use intrinsic size a minimum
/// ComplexTableColumnWidth::with_min(Flex(1.), Intrinsic);
/// ```
///
/// It is usually not necessary to use this type directly, because
/// thert are function to convert from:
///
/// - f64 => Simple(Fixed(f64))
/// - `Into<TableColumnWidth>` => Simple(TableColumnWidth))
/// - `(Into<TableColumnWidth>`, `Range<f64>`) => Limited with min/max from range
/// - `(Into<TableColumnWidth>`, `Into<TableColumnWidth>`) => Limited with minimun
/// - `(Into<TableColumnWidth>`, `Into<TableColumnWidth>`, `Into<TableColumnWidth>`) => Limited with min/max
///
/// Examples:
/// ```
/// use druid_widget_nursery::table::{FlexTable, TableColumnWidth::*};
/// # fn test () -> FlexTable<()> {
/// FlexTable::new()
///    .with_column_width(64.0)
///    .with_column_width(Intrinsic)
///    .with_column_width((Flex(1.0), 60.0))
///    .with_column_width((Flex(1.0), 60.0..200.0))
/// # }
/// ```
#[derive(Copy, Clone, Debug)]
pub enum ComplexTableColumnWidth {
    /// Column without limits
    Simple(TableColumnWidth),
    /// Limited column (width, min, max)
    ///
    /// It is usually better to avoid flex dependent 'min' and 'max'
    /// constraint, because it can lead to unexpected results (with
    /// the current resolver).
    Limited(TableColumnWidth, TableColumnWidth, TableColumnWidth),
}

impl<W: Into<TableColumnWidth>> From<W> for ComplexTableColumnWidth {
    fn from(tcw: W) -> Self {
        let tcw: TableColumnWidth = tcw.into();
        Self::simple(tcw)
    }
}

impl<W: Into<TableColumnWidth>> From<(W, Range<f64>)> for ComplexTableColumnWidth {
    fn from(data: (W, Range<f64>)) -> Self {
        let tcw: TableColumnWidth = data.0.into();
        let min = TableColumnWidth::Fixed(data.1.start);
        let max = TableColumnWidth::Fixed(data.1.end);
        Self::with_min_max(tcw, min, max)
    }
}

impl<W1, W2> From<(W1, W2)> for ComplexTableColumnWidth
where
    W1: Into<TableColumnWidth>,
    W2: Into<TableColumnWidth>,
{
    fn from(data: (W1, W2)) -> Self {
        let tcw: TableColumnWidth = data.0.into();
        let min: TableColumnWidth = data.1.into();
        let max = TableColumnWidth::Fixed(0.0);
        Self::with_min_max(tcw, min, max)
    }
}

impl<W1, W2, W3> From<(W1, W2, W3)> for ComplexTableColumnWidth
where
    W1: Into<TableColumnWidth>,
    W2: Into<TableColumnWidth>,
    W3: Into<TableColumnWidth>,
{
    fn from(data: (W1, W2, W3)) -> Self {
        let tcw: TableColumnWidth = data.0.into();
        let min: TableColumnWidth = data.1.into();
        let max: TableColumnWidth = data.2.into();
        Self::with_min_max(tcw, min, max)
    }
}

impl ComplexTableColumnWidth {
    /// Create instance without limits
    pub fn simple(tcw: TableColumnWidth) -> Self {
        Self::Simple(tcw)
    }

    /// Create instance with minimum limit
    pub fn with_min<W1, W2>(tcw: W1, min: W2) -> Self
    where
        W1: Into<TableColumnWidth>,
        W2: Into<TableColumnWidth>,
    {
        let tcw: TableColumnWidth = tcw.into();
        let min: TableColumnWidth = min.into();
        Self::with_min_max(tcw, min, TableColumnWidth::Fixed(0f64))
    }

    /// Create instance with maximum limit
    pub fn with_max<W1, W2>(tcw: W1, max: W2) -> Self
    where
        W1: Into<TableColumnWidth>,
        W2: Into<TableColumnWidth>,
    {
        let tcw: TableColumnWidth = tcw.into();
        let max: TableColumnWidth = max.into();
        Self::with_min_max(tcw, TableColumnWidth::Fixed(0f64), max)
    }

    /// Create instance with minimum and maximum limit
    pub fn with_min_max<W1, W2, W3>(tcw: W1, min: W2, max: W3) -> Self
    where
        W1: Into<TableColumnWidth>,
        W2: Into<TableColumnWidth>,
        W3: Into<TableColumnWidth>,
    {
        let tcw: TableColumnWidth = tcw.into();
        let min: TableColumnWidth = min.into();
        let max: TableColumnWidth = max.into();
        Self::Limited(tcw, min, max)
    }

    pub(crate) fn need_intrinsic_width(&self) -> bool {
        match self {
            Self::Simple(tcw) => tcw.need_intrinsic_width(),
            Self::Limited(tcw, min, max) => {
                tcw.need_intrinsic_width()
                    || min.need_intrinsic_width()
                    || max.need_intrinsic_width()
            }
        }
    }

    fn resolve_width(
        &self,
        total_width: f64,
        intrinsic_width: f64,
        px_per_flex: f64,
        apply_limits: bool,
    ) -> (bool, f64, f64, f64) {
        match self {
            Self::Simple(tcw) => {
                let (col_width, col_fixed, col_flex) =
                    tcw.resolve_width(total_width, intrinsic_width, px_per_flex);
                (false, col_width, col_fixed, col_flex)
            }
            Self::Limited(tcw, min, max) => {
                let (col_width, col_fixed, col_flex) =
                    tcw.resolve_width(total_width, intrinsic_width, px_per_flex);

                if apply_limits {
                    let (min_width, _min_fixed, _min_flex) =
                        min.resolve_width(total_width, intrinsic_width, px_per_flex);
                    let (max_width, _max_fixed, _max_flex) =
                        max.resolve_width(total_width, intrinsic_width, px_per_flex);

                    if (max_width > 0f64) && (col_width > max_width) {
                        return (true, max_width, max_width, 0f64);
                    }
                    if (min_width > 0f64) && (col_width < min_width) {
                        return (true, min_width, min_width, 0f64);
                    }
                }
                (false, col_width, col_fixed, col_flex)
            }
        }
    }

    /// We may use this for several table widget implementations
    pub(crate) fn compute_column_widths(
        column_widths: &mut [ComplexTableColumnWidth],
        intrinsic_widths: &[f64],
        max_table_width: f64,
    ) -> Vec<f64> {
        let column_count = column_widths.len();
        let mut col_widths = vec![0f64; column_count];

        // Note: This needs only one step unless you use MIN/MAX
        // column constraints
        'outer: for _ in 0..column_count {
            // Step 1: Compute pixels per flex

            let px_per_flex = {
                let mut fixed_width = 0f64;
                let mut flex_sum = 0f64;

                for col_num in 0..column_count {
                    let (_, _width, fixed_share, flex_share) = column_widths[col_num]
                        .resolve_width(max_table_width, intrinsic_widths[col_num], 0f64, false);
                    fixed_width += fixed_share;
                    flex_sum += flex_share;
                }

                let remaining = (max_table_width - fixed_width).max(0.0);
                if flex_sum != 0f64 {
                    remaining / flex_sum
                } else {
                    0f64
                }
            };

            // Step 2: Apply min/max limits

            for col_num in 0..column_count {
                let (limit, width, _fixed_share, _flex_share) = column_widths[col_num]
                    .resolve_width(
                        max_table_width,
                        intrinsic_widths[col_num],
                        px_per_flex,
                        true,
                    );
                col_widths[col_num] = width;

                if limit {
                    // convert into a fixed size column
                    column_widths[col_num] = TableColumnWidth::Fixed(width).into();
                    continue 'outer;
                }
            }

            break; // done
        }

        col_widths
    }
}
