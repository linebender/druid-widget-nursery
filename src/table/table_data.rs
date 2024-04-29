use std::{
    cell::RefCell,
    fmt::Debug,
    hash::Hash,
    ops::{Deref, Index, IndexMut},
    sync::Arc,
};

use druid::{
    lens::{Identity, InArc},
    Data, Lens, Widget, WidgetExt,
};

use self::private::Sealed;

use super::FlexTable;

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

pub(super) mod private {
    pub enum Local {}

    pub trait Sealed {}

    impl Sealed for Local {}
}

pub trait TableData:
    Data
    + for<'a> Index<&'a <Self::Row as RowData>::Id, Output = Self::Row>
    + for<'a> IndexMut<&'a <Self::Row as RowData>::Id, Output = Self::Row>
{
    type Row: RowData<Column = Self::Column>;
    type Column: Hash + Eq + Clone;

    fn keys(&self) -> impl Iterator<Item = <Self::Row as RowData>::Id>;

    fn columns(&self) -> impl Iterator<Item = Self::Column>;

    #[doc(hidden)]
    fn _or_fixed<T: Sealed>(&self, _table: &FlexTable<Self>) -> &Self {
        self
    }
}

#[derive(Clone, Data, Default, Lens)]
pub struct FixedRow<T: Data> {
    data: T,
}

impl<T: Data> FixedTable<T> {
    pub fn new(data: T) -> Self {
        Self {
            dummy: FixedRow {
                data,
            },
            columns: Default::default(),
            len: Default::default(),
        }
    }
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
pub struct FixedTable<T: Data> {
    dummy: FixedRow<T>,
    #[data(ignore)]
    len: RefCell<usize>,
    #[data(ignore)]
    columns: RefCell<usize>,
}

impl<T: Data> Index<&usize> for FixedTable<T> {
    type Output = FixedRow<T>;

    fn index(&self, _: &usize) -> &Self::Output {
        &self.dummy
    }
}

impl<T: Data> IndexMut<&usize> for FixedTable<T> {
    fn index_mut(&mut self, _: &usize) -> &mut Self::Output {
        &mut self.dummy
    }
}

impl<T: Data> TableData for FixedTable<T> {
    type Row = FixedRow<T>;
    type Column = <Self::Row as RowData>::Column;

    fn keys(&self) -> impl Iterator<Item = <Self::Row as RowData>::Id> {
        0..*self.len.borrow()
    }

    fn columns(&self) -> impl Iterator<Item = Self::Column> {
        0..*self.columns.borrow()
    }

    fn _or_fixed<U: Sealed>(&self, table: &FlexTable<Self>) -> &Self {
        *self.columns.borrow_mut() = table.column_count();
        *self.len.borrow_mut() = table.rows().count();

        self
    }
}
