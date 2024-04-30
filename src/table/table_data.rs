use std::{
    fmt::Debug,
    hash::Hash,
    ops::{Deref, Index, IndexMut},
    sync::Arc,
};

use druid::{
    im::Vector,
    lens::{Identity, InArc},
    Data, Widget, WidgetExt,
};

pub trait RowData: Data {
    type Id: Hash + Eq + Clone + Debug;
    type Column: Hash + Eq;

    fn id(&self) -> Self::Id;

    fn cell(&self, column: &Self::Column) -> Box<dyn Widget<Self>>;
}

impl<T: RowData> RowData for Arc<T> {
    type Id = T::Id;
    type Column = T::Column;

    fn id(&self) -> Self::Id {
        self.deref().id()
    }

    fn cell(&self, column: &Self::Column) -> Box<dyn Widget<Self>> {
        self.deref()
            .cell(column)
            .lens(InArc::new::<T, T>(Identity))
            .boxed()
    }
}

pub trait TableData:
    Data
    + Index<<Self::Row as RowData>::Id, Output = Self::Row>
    + IndexMut<<Self::Row as RowData>::Id, Output = Self::Row>
{
    type Row: RowData<Column = Self::Column>;
    type Column: Hash + Eq + Clone;

    fn keys(&self) -> impl Iterator<Item = <Self::Row as RowData>::Id>;

    fn columns(&self) -> impl Iterator<Item = Self::Column>;
}

pub type WidgetFactoryRow = Vector<Arc<dyn Fn() -> Box<dyn Widget<()>>>>;

impl RowData for (usize, WidgetFactoryRow) {
    type Id = usize;
    type Column = usize;

    fn id(&self) -> Self::Id {
        self.0
    }

    fn cell(&self, column: &Self::Column) -> Box<dyn Widget<Self>> {
        (self.1[*column])().lens(druid::lens::Unit).boxed()
    }
}

pub type WidgetFactoryTable = Vector<(usize, WidgetFactoryRow)>;

impl TableData for WidgetFactoryTable {
    type Row = (usize, WidgetFactoryRow);
    type Column = usize;

    fn keys(&self) -> impl Iterator<Item = <Self::Row as RowData>::Id> {
        0..self.len()
    }

    fn columns(&self) -> impl Iterator<Item = Self::Column> {
        if self.is_empty() {
            0..0
        } else {
            0..self[0].1.len()
        }
    }
}

impl RowData for () {
    type Id = usize;
    type Column = usize;

    fn id(&self) -> Self::Id {
        0
    }

    fn cell(&self, _: &Self::Column) -> Box<dyn Widget<Self>> {
        unreachable!()
    }
}

impl TableData for [(); 0] {
    type Row = ();
    type Column = usize;

    fn keys(&self) -> impl Iterator<Item = <Self::Row as RowData>::Id> {
        0..0
    }

    fn columns(&self) -> impl Iterator<Item = Self::Column> {
        0..0
    }
}
