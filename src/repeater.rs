use druid::widget::{prelude::*, LensWrap};
use druid::{
    im::{self, HashMap, HashSet},
    Data, Lens, Widget, WidgetPod,
};
use std::{hash::Hash, marker::PhantomData, vec::Vec};

type RepeaterChildWidget<U> =
    WidgetPod<im::Vector<U>, LensWrap<im::Vector<U>, U, ListItemLens<U>, Box<dyn Widget<U>>>>;

pub struct RepeaterChild<U, I> {
    widget: RepeaterChildWidget<U>,
    id: I,
}

impl<U, I> RepeaterChild<U, I> {
    pub fn widget(&mut self) -> &mut RepeaterChildWidget<U> {
        &mut self.widget
    }
}

pub struct ListItemLens<U> {
    pub index: usize,
    phantom_u: PhantomData<U>,
}

impl<U> ListItemLens<U> {
    pub fn new(index: usize) -> ListItemLens<U> {
        ListItemLens {
            index,
            phantom_u: PhantomData,
        }
    }
}

impl<U> Lens<im::Vector<U>, U> for ListItemLens<U>
where
    U: Data,
{
    fn with<V, F: FnOnce(&U) -> V>(&self, state: &im::Vector<U>, f: F) -> V {
        f(&state.get(self.index).unwrap())
    }
    fn with_mut<V, F: FnOnce(&mut U) -> V>(&self, state: &mut im::Vector<U>, f: F) -> V {
        let mut focused = state.get(self.index).unwrap().clone();
        let result = f(&mut focused);
        if !focused.same(&state.get(self.index).unwrap()) {
            state.set(self.index, focused);
        }
        result
    }
}

type LayoutChildrenCallback<T, U, I> =
    Box<dyn Fn(&mut Vec<RepeaterChild<U, I>>, &mut LayoutCtx, &BoxConstraints, &T, &Env) -> ()>;

pub struct Repeater<T, U, I, L, W> {
    children: Vec<RepeaterChild<U, I>>,
    list_lens: L,
    id_getter: Box<dyn Fn(&U) -> I>,
    child_generator: Box<dyn Fn(&U) -> W>,
    layout_children: LayoutChildrenCallback<T, U, I>,
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
        layout_children: LayoutChildrenCallback<T, U, I>,
    ) -> Self {
        // let a = |U| {1}

        Self {
            children: Vec::new(),
            list_lens,
            id_getter,
            child_generator,
            layout_children,
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
    I: Hash + Eq + Clone + Data,
    L: Lens<T, im::Vector<U>>,
    W: Widget<U> + 'static,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        for child in &mut self.children {
            self.list_lens
                .with_mut(data, |data| child.widget.event(ctx, event, data, env));
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        for child in &mut self.children {
            self.list_lens
                .with(data, |data| child.widget.lifecycle(ctx, event, data, env));
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        for child in &mut self.children {
            self.list_lens
                .with(data, |data| child.widget.update(ctx, data, env));
        }

        let list_lens = &self.list_lens;
        let id_getter = &self.id_getter;
        let children = &mut self.children;
        let child_generator = &self.child_generator;

        list_lens.with(old_data, |old_list| {
            list_lens.with(data, |list| {
                // Before we start, check if there are any differences
                let mut should_diff = false;

                if old_list.len() != list.len() {
                    should_diff = true;
                } else {
                    // If the lists have the same length, loop through and
                    // check equality on each ID
                    for i in 0..list.len() {
                        if !(id_getter)(old_list.get(i).unwrap())
                            .same(&(id_getter)(list.get(i).unwrap()))
                        {
                            should_diff = true;
                            break;
                        }
                    }
                }

                if !should_diff {
                    return;
                }

                // Diffing has three stages: remove, add and sort. For each, it
                // figures out what of each operation must be performed to
                // transform old_list into list, and performs those on the
                // widget list.

                // If we didn't add or remove anything, this will remain false.
                let mut children_changed = false;

                // Start
                let mut stale = HashSet::new();
                for list_item in old_list {
                    stale.insert((id_getter)(list_item));
                }

                // Target (key is ID, value is target index)
                let mut fresh = HashMap::new();
                for i in 0..list.len() {
                    fresh.insert((id_getter)(list.get(i).unwrap()), i);
                }

                // TODO: Detect duplicate keys

                // Remove
                for i in (0..old_list.len()).rev() {
                    let id = (id_getter)(&old_list.get(i).unwrap());
                    if !fresh.contains_key(&id) {
                        children.remove(i);
                        children_changed = true;
                    }
                }

                // Add
                for item in list {
                    let id = (id_getter)(item);
                    if !stale.contains(&id) {
                        let lens = ListItemLens::<U>::new(0);

                        children.push(RepeaterChild {
                            widget: WidgetPod::new(LensWrap::new(
                                Box::new((child_generator)(&item)),
                                lens,
                            )),
                            id,
                        });

                        children_changed = true;
                    }
                }

                // Sort by expected index
                children.sort_unstable_by(|a, b| fresh.get(&a.id).cmp(&fresh.get(&b.id)));

                // Update the lens indices
                for i in 0..children.len() {
                    children[i].widget.widget_mut().lens_mut().index = i;
                }

                if children_changed {
                    ctx.children_changed();
                } else {
                    ctx.request_layout();
                }
            });
        });
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        (self.layout_children)(&mut self.children, ctx, bc, data, env);

        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        for child in &mut self.children {
            self.list_lens
                .with(data, |data| child.widget.paint(ctx, data, env));
        }
    }
}
