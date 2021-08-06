use druid::im::Vector;
use druid::widget::ListIter;
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Size,
    UpdateCtx, Widget,
};
use std::marker::PhantomData;
use std::ops::Range;

#[derive(Data, Clone)]
pub struct FilterIter<I> {
    data: I,
    indices: Vector<usize>,
}

impl<I> FilterIter<I> {
    pub fn new(data: I, indices: Vector<usize>) -> Self {
        FilterIter { data, indices }
    }
}

impl<T: Data, I: ListIter<T>> ListIter<T> for FilterIter<I> {
    fn for_each(&self, mut cb: impl FnMut(&T, usize)) {
        let mut indices = self.indices.iter();
        let mut next = indices.next();
        let mut counter = 0;
        self.data.for_each(|element, index| {
            if let Some(next_index) = next {
                if index == *next_index {
                    cb(element, counter);
                    next = indices.next();
                    counter += 1;
                }
            }
        });
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut T, usize)) {
        let mut indices = self.indices.iter();
        let mut next = indices.next();
        let mut counter = 0;
        self.data.for_each_mut(|element, index| {
            if let Some(next_index) = next {
                if index == *next_index {
                    cb(element, counter);
                    next = indices.next();
                    counter += 1;
                }
            }
        });
    }

    fn data_len(&self) -> usize {
        self.indices.len()
    }
}

type FilterUpdate<I, D> = dyn Fn(&mut Vector<usize>, usize, &I, Range<usize>, &D);

/// A widget which filters a list for its inner widget.
pub struct ListFilter<D, T, I> {
    accepted: Vector<usize>,
    filter_update: Box<FilterUpdate<I, D>>,
    inner: Box<dyn Widget<FilterIter<I>>>,
    phantom: PhantomData<T>,
}

impl<D: Data, T: Data, I: ListIter<T>> ListFilter<D, T, I> {
    pub fn new(
        inner: impl Widget<FilterIter<I>> + 'static,
        filter: impl Fn(&T, &D) -> bool + 'static,
    ) -> Self {
        Self {
            accepted: Vector::new(),
            filter_update: Box::new(
                move |indices, mut insert_index, elements, update_range, filter_option| {
                    elements.for_each(|element, index| {
                        if index >= update_range.start
                            && index < update_range.end
                            && filter(element, filter_option)
                        {
                            indices.insert(insert_index, index);
                            insert_index += 1;
                        }
                    })
                },
            ),
            inner: Box::new(inner),
            phantom: PhantomData,
        }
    }
}

impl<T: Data, D: Data, I: ListIter<T>> Widget<(I, D)> for ListFilter<D, T, I> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut (I, D), env: &Env) {
        let mut inner_data = FilterIter::new(data.0.clone(), self.accepted.clone());
        self.inner.event(ctx, event, &mut inner_data, env);
        data.0 = inner_data.data;
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &(I, D), env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            (self.filter_update)(
                &mut self.accepted,
                0,
                &data.0,
                0..(data.0.data_len()),
                &data.1,
            );
        }
        let inner_data = FilterIter::new(data.0.clone(), self.accepted.clone());
        self.inner.lifecycle(ctx, event, &inner_data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &(I, D), data: &(I, D), env: &Env) {
        let old_inner = FilterIter::new(old_data.0.clone(), self.accepted.clone());

        if !old_data.same(data) {
            //TODO: do real diffing here
            self.accepted.clear();
            (self.filter_update)(
                &mut self.accepted,
                0,
                &data.0,
                0..(data.0.data_len()),
                &data.1,
            );
        }
        let inner_data = FilterIter::new(data.0.clone(), self.accepted.clone());
        self.inner.update(ctx, &old_inner, &inner_data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &(I, D),
        env: &Env,
    ) -> Size {
        let inner_data = FilterIter::new(data.0.clone(), self.accepted.clone());
        self.inner.layout(ctx, bc, &inner_data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &(I, D), env: &Env) {
        let inner_data = FilterIter::new(data.0.clone(), self.accepted.clone());
        self.inner.paint(ctx, &inner_data, env);
    }
}
