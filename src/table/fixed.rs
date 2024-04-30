use std::ops::{Index, IndexMut};

use druid::{
    widget::{BackgroundBrush, Painter},
    Color, Data, KeyOrValue, Lens, Widget, WidgetExt, WidgetPod,
};

use super::{
    ComplexTableColumnWidth, FlexTable, RowData, TableBorderStyle, TableCellVerticalAlignment,
    TableData, TableRowInternal,
};

pub struct FixedFlexTable<T: Data> {
    table: FlexTable<FixedTable<T>>,
}

impl<T: Data> Default for FixedFlexTable<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Data> FixedFlexTable<T> {
    pub fn new() -> Self {
        Self {
            table: FlexTable::new(),
        }
    }

    fn data(&self, data: &T) -> FixedTable<T> {
        FixedTable {
            data: FixedRow { data: data.clone() },
            len: self.table.children.len(),
            columns: if self.table.children.is_empty() {
                0
            } else {
                self.table.children.values().next().unwrap().children.len()
            },
        }
    }

    /// Builder-style method to add a table row.
    pub fn with_row(mut self, row: TableRow<T>) -> Self {
        self.table.insert_row(row.into_internal(&self));
        self
    }

    /// Add a table row
    pub fn add_row(&mut self, row: TableRow<T>) {
        self.table.insert_row(row.into_internal(self))
    }

    /// Builder-style method to set the table background brush
    pub fn background(mut self, brush: impl Into<BackgroundBrush<T>>) -> Self {
        self.set_background(brush);
        self
    }

    /// Set the table background brush
    pub fn set_background(&mut self, brush: impl Into<BackgroundBrush<T>>) {
        self.table.background = Some(Self::convert_brush(brush));
    }

    /// Builder-style method to set the row background brush
    pub fn row_background(mut self, brush: impl Into<BackgroundBrush<T>>) -> Self {
        self.set_row_background(brush);
        self
    }

    /// Set the row background brush
    pub fn set_row_background(&mut self, brush: impl Into<BackgroundBrush<T>>) {
        self.table.row_background = Some(Self::convert_brush(brush));
    }

    /// Builder-style method to set the table inner border
    pub fn inner_border(
        mut self,
        color: impl Into<KeyOrValue<Color>>,
        width: impl Into<KeyOrValue<f64>>,
    ) -> Self {
        self.table.set_inner_border(color, width);
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
        self.table.set_row_border(color.clone(), width.clone());
        self.table.set_column_border(color, width);
    }

    /// Builder-style method to set the table row border.
    pub fn row_border(
        mut self,
        color: impl Into<KeyOrValue<Color>>,
        width: impl Into<KeyOrValue<f64>>,
    ) -> Self {
        self.table.set_row_border(color, width);
        self
    }

    /// Set the table row border.
    pub fn set_row_border(
        &mut self,
        color: impl Into<KeyOrValue<Color>>,
        width: impl Into<KeyOrValue<f64>>,
    ) {
        self.table.row_border = Some(TableBorderStyle {
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
        self.table.set_column_border(color, width);
        self
    }

    /// Set the table column border.
    pub fn set_column_border(
        &mut self,
        color: impl Into<KeyOrValue<Color>>,
        width: impl Into<KeyOrValue<f64>>,
    ) {
        self.table.col_border = Some(TableBorderStyle {
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
        self.table.column_widths.push(column_width.into());
        self
    }

    /// Builder-style method to set the table column width.
    ///
    /// If not set, the [`Self::default_column_width`] is used.
    pub fn column_widths(mut self, column_widths: &[ComplexTableColumnWidth]) -> Self {
        self.table.set_column_widths(column_widths);
        self
    }

    /// Set the table column widths.
    ///
    /// If not set, the [`Self::default_column_width`] is used.
    pub fn set_column_widths(&mut self, column_widths: &[ComplexTableColumnWidth]) {
        self.table.column_widths = column_widths.to_vec();
    }

    /// Builder-style method to set the default column width.
    pub fn default_column_width<W: Into<ComplexTableColumnWidth>>(
        mut self,
        default_column_width: W,
    ) -> Self {
        self.table.set_default_column_width(default_column_width);
        self
    }

    /// Set the default column width.
    pub fn set_default_column_width<W: Into<ComplexTableColumnWidth>>(
        &mut self,
        default_column_width: W,
    ) {
        self.table.default_column_width = default_column_width.into();
    }

    /// Builder-style method to set the default vertical cell alignment.
    pub fn default_vertical_alignment(
        mut self,
        default_vertical_alignment: TableCellVerticalAlignment,
    ) -> Self {
        self.table
            .set_default_vertical_alignment(default_vertical_alignment);
        self
    }

    /// Set the default vertical cell alignment.
    pub fn set_default_vertical_alignment(
        &mut self,
        default_vertical_alignment: TableCellVerticalAlignment,
    ) {
        self.table.default_vertical_alignment = default_vertical_alignment;
    }

    /// Returns the column count
    pub fn column_count(&self) -> usize {
        if self.table.children.is_empty() {
            0
        } else {
            self.table.children.values().next().unwrap().children.len()
        }
    }

    /// Return a mutable reference to column widths to allow them to modified at runtime
    pub fn get_column_widths(&mut self) -> &mut Vec<ComplexTableColumnWidth> {
        &mut self.table.column_widths
    }

    /// Clear the table of all rows and thus cells
    pub fn clear(&mut self) {
        self.table.children.clear();
        self.table.row_starts = None;
    }

    fn convert_brush(brush: impl Into<BackgroundBrush<T>>) -> BackgroundBrush<FixedTable<T>> {
        let brush = brush.into();

        match brush {
            BackgroundBrush::Color(color) => BackgroundBrush::Color(color),
            BackgroundBrush::ColorKey(key) => BackgroundBrush::ColorKey(key),
            BackgroundBrush::Linear(linear) => BackgroundBrush::Linear(linear),
            BackgroundBrush::Radial(radial) => BackgroundBrush::Radial(radial),
            BackgroundBrush::Fixed(fixed) => BackgroundBrush::Fixed(fixed),
            BackgroundBrush::Painter(mut painter) => {
                BackgroundBrush::Painter(Painter::new(move |ctx, data: &FixedTable<T>, env| {
                    painter.paint(ctx, &data.data.data, env)
                }))
            }
            _ => unreachable!(),
        }
    }
}

impl<T: Data> Widget<T> for FixedFlexTable<T> {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut T,
        env: &druid::Env,
    ) {
        let mut table_data = self.data(data);
        self.table.event(ctx, event, &mut table_data, env);
        *data = table_data.data.data
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &T,
        env: &druid::Env,
    ) {
        let table_data = self.data(data);
        self.table.lifecycle(ctx, event, &table_data, env);
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &T, data: &T, env: &druid::Env) {
        let old_data = self.data(old_data);
        let data = self.data(data);
        self.table.update(ctx, &old_data, &data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &T,
        env: &druid::Env,
    ) -> druid::Size {
        let data = self.data(data);
        self.table.layout(ctx, bc, &data, env)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &T, env: &druid::Env) {
        let data = self.data(data);
        self.table.paint(ctx, &data, env)
    }
}

pub struct TableRow<T> {
    children: Vec<Box<dyn Widget<T>>>,
    min_height: Option<f64>,
    vertical_alignment: Option<TableCellVerticalAlignment>,
}

impl<T: Data> Default for TableRow<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Data> TableRow<T> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            min_height: None,
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
        self.children.push(child.boxed());
    }

    fn into_internal(self, table: &FixedFlexTable<T>) -> TableRowInternal<FixedRow<T>> {
        let TableRow {
            children,
            min_height,
            vertical_alignment,
        } = self;

        let mut row = TableRowInternal::new(table.table.children.len());
        row.vertical_alignment = vertical_alignment;
        row.min_height = min_height;

        for child in children {
            row.add_child(child)
        }

        row
    }
}

#[derive(Clone, Data, Default, Lens)]
struct FixedRow<T: Data> {
    data: T,
}

impl<T: Data> RowData for FixedRow<T> {
    type Id = usize;
    type Column = usize;

    fn id(&self) -> Self::Id {
        0
    }

    fn cell(&self, _: &Self::Column) -> Box<dyn Widget<Self>> {
        unimplemented!()
    }
}

#[derive(Clone, Data, Default, Lens)]
struct FixedTable<T: Data> {
    data: FixedRow<T>,
    #[data(ignore)]
    len: usize,
    #[data(ignore)]
    columns: usize,
}

impl<T: Data> Index<usize> for FixedTable<T> {
    type Output = FixedRow<T>;

    fn index(&self, _: usize) -> &Self::Output {
        &self.data
    }
}

impl<T: Data> IndexMut<usize> for FixedTable<T> {
    fn index_mut(&mut self, _: usize) -> &mut Self::Output {
        &mut self.data
    }
}

impl<T: Data> TableData for FixedTable<T> {
    type Row = FixedRow<T>;
    type Column = <Self::Row as RowData>::Column;

    fn keys(&self) -> impl Iterator<Item = <Self::Row as RowData>::Id> {
        0..self.len
    }

    fn columns(&self) -> impl Iterator<Item = Self::Column> {
        0..self.columns
    }
}

impl<T: Data> TableRowInternal<FixedRow<T>> {
    /// Add a child widget (table cell).
    ///
    /// See also [`Self::with_child`].
    fn add_child(&mut self, child: impl Widget<T> + 'static) {
        let child: Box<dyn Widget<FixedRow<T>>> = child.lens(FixedRow::data).boxed();
        let child = WidgetPod::new(child);
        self.children.insert(self.children.len(), child);
    }
}
