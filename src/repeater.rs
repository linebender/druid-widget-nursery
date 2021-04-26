use druid::{im, Data, Lens, Widget, WidgetPod};
use druid::{
    widget::{prelude::*, LensWrap},
    Color,
};
use std::{marker::PhantomData, vec::Vec};

pub struct Repeater<T, U, I, L, W> {
    children: Vec<WidgetPod<im::Vector<U>, Box<dyn Widget<im::Vector<U>>>>>,
    list_lens: L,
    id_getter: Box<dyn Fn(&U) -> I>,
    child_generator: Box<dyn Fn(&U) -> W>,
    phantom_t: PhantomData<T>,
    phantom_i: PhantomData<I>,
    phantom_w: PhantomData<W>,
}

impl<T, U, I, L, W> Repeater<T, U, I, L, W>
where
    I: PartialEq,
{
    pub fn new(
        list_lens: L,
        id_getter: Box<dyn Fn(&U) -> I>,
        child_generator: Box<dyn Fn(&U) -> W>,
    ) -> Self {
        // let a = |U| {1}

        Self {
            children: Vec::new(),
            list_lens,
            id_getter,
            child_generator,
            phantom_t: PhantomData,
            phantom_i: PhantomData,
            phantom_w: PhantomData,
        }
    }
}

impl<T, U, I, L, W> Widget<T> for Repeater<T, U, I, L, W>
where
    T: Data,
    U: Data,
    I: Data,
    L: Lens<T, im::Vector<U>>,
    W: Widget<U>,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        for child in &mut self.children {
            self.list_lens
                .with_mut(data, |data| child.event(ctx, event, data, env));
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        for child in &mut self.children {
            self.list_lens
                .with(data, |data| child.lifecycle(ctx, event, data, env));
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        for child in &mut self.children {
            self.list_lens
                .with(data, |data| child.update(ctx, data, env));
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        // TODO
        for child in &mut self.children {
            self.list_lens.with(data, |data| {
                child.layout(ctx, bc, data, env);
            });
        }

        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        for child in &mut self.children {
            self.list_lens
                .with(data, |data| child.paint(ctx, data, env));
        }
    }
}
