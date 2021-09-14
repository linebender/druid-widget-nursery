// Copyright 2020 The Druid Authors.
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

//! A tree widget.

use std::convert::TryFrom;
use std::fmt::Display;
use std::marker::PhantomData;
use std::sync::Arc;

use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::Label;
use druid::{theme, Lens, LensExt};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, Selector, UpdateCtx, Widget, WidgetId, WidgetPod,
};

use crate::selectors;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChrootStatus {
    YES,
    NO,
    ROOT,
}

// TODO:
//   - TREE_CLOSE command that mirrors TreeOpen
//   - TREE_OPEN_ALL command to open recursively
//   - TREE_CLOSE_ALL command to close recursively
selectors! {
    /// Notification to send from the widget that requires removal
    TREE_NODE_REMOVE,
    /// Internal use, sent by TreeNodeWidgets with their index for their parent TreeNodeWidget
    /// TODO: should not be public
    TREE_CHILD_REMOVE_INTERNAL: i32,
    /// Notification that opens the first encountered branch node.
    TREE_OPEN,
    /// Command sent to children on open
    TREE_CHILD_SHOW,
    /// Command sent to children on close
    TREE_CHILD_HIDE,
    /// Notification that moves the root of the tree to the first encountered branch node.
    TREE_CHROOT,
    /// Notification that ask a parent to register a chroot a this index.
    /// Mostly for internal use.
    TREE_CHROOT_CHILD: usize,
    /// Notification that moves the chroot to the parent of the current virtual root.
    /// Does nothing on the actual root.
    TREE_CHROOT_UP,
    /// Notify a child that it's now the chroot
    TREE_NOTIFY_CHROOT: ChrootStatus,
    /// Submitted as a notification, from the user's widget, the Selector payload is submitted as
    /// a command to its parent. It's a workaround to simulate notifications between user's tree
    /// widgets.
    TREE_NOTIFY_PARENT: Selector,
    /// Notify an opener's widget on click.
    TREE_ACTIVATE_NODE,
}

/// A tree widget for a collection of items organized in a hierarchical way.
pub struct Tree<T, L>
where
    T: TreeNode,
    L: Lens<T, bool>,
{
    /// The root node of this tree
    root_node: WidgetPod<T, TreeNodeWidget<T, L>>,
    chroot: WidgetId,
}

/// A tree node `Data`. This is the data expected by the tree widget.
///
/// Implementors of this trait must know the number of children of each node
/// and be able to provide a children based on the index of the child widget.
/// This implies that the implementation of the collection of children may be
/// abstracted away in the data as long as `children_count()`, `get_child()`,
/// rm_child() and for_child_mut()` accessors give coherent results. This is
/// a way to implement filtering and sorting at the app data level.
pub trait TreeNode
where
    Self: Data + std::fmt::Debug,
{
    /// Returns how many children are below this node. It could be zero if this is a leaf.
    fn children_count(&self) -> usize;

    /// Returns a reference to the node's child at the given index
    fn get_child(&self, index: usize) -> &Self;

    /// Returns a mutable reference to the node's child at the given index
    fn for_child_mut(&mut self, index: usize, cb: impl FnMut(&mut Self, usize));

    /// If a node is somewhere between the root and the virtual root return the
    /// index of the next children in the this path otherwise return `None`
    ///
    /// The default implementation returns always `None`, disallowing chrooting.
    ///
    /// Here's a tree with get_chroot result for each node:
    ///
    /// - actual_root Some(1)
    ///   - node0 None
    ///     - node0-0 None
    ///   - node1 Some(0)
    ///     - virtual_root None
    ///     - node1-1 None
    ///
    /// In this case the virtual root is `/actual_root/node1/virtual_root`
    fn get_chroot(&self) -> Option<usize> {
        None
    }

    /// Set the chroot of this widget to the provided index
    #[allow(unused_variables)]
    fn chroot(&mut self, idx: Option<usize>) {}

    /// `is_branch` must return `true` if the data is considered as a branch.
    /// The default implementation returns `true` when `children_count()` is
    /// more than 0.
    fn is_branch(&self) -> bool {
        self.children_count() > 0
    }

    /// Remove the child at `index`
    #[allow(unused_variables)]
    fn rm_child(&mut self, index: usize) {}
}

// Wrapper widget that reacts to clicks by sending a TREE_ACTIVATE_NODE command to
// its inner user-defined widget.
// TODO: Try use a Controller instead of a plain widget.
struct Opener<T>
where
    T: TreeNode,
{
    widget: WidgetPod<T, Box<dyn Widget<T>>>,
}

/// Implementing Widget for the Opener.
impl<T: TreeNode> Widget<T> for Opener<T>
where
    T: Data,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, _env: &Env) {
        match event {
            Event::MouseDown(_) => {
                ctx.set_active(true);
                ctx.request_paint();
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    if ctx.is_hot() {
                        ctx.submit_command(TREE_ACTIVATE_NODE.to(self.widget.id()));
                    }
                    ctx.request_paint();
                }
            }
            _ => (),
        }
        self.widget.event(ctx, event, data, _env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.widget.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        self.widget.update(ctx, data, env)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let size = self.widget.layout(ctx, bc, data, env);
        self.widget.set_origin(ctx, data, env, Point::ORIGIN);
        bc.constrain(size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.widget.paint(ctx, data, env)
    }
}

// The default opener if none is passed to the Tree builder.
struct Wedge<T, L>
where
    T: TreeNode,
    L: Lens<T, bool>,
{
    expand_lens: L,
    phantom: PhantomData<T>,
}

impl<T: TreeNode, L: Lens<T, bool>> Widget<T> for Wedge<T, L> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, _env: &Env) {
        match event {
            Event::Command(cmd) if cmd.is(TREE_ACTIVATE_NODE) => {
                self.expand_lens.put(data, !self.expand_lens.get(data));
                ctx.set_handled();
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &T, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &T, _data: &T, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &T, env: &Env) -> Size {
        let size = env.get(theme::BASIC_WIDGET_HEIGHT);
        bc.constrain(Size::new(size, size))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        if !data.is_branch() {
            return;
        }
        let stroke_color = if ctx.is_hot() {
            env.get(theme::FOREGROUND_LIGHT)
        } else {
            env.get(theme::FOREGROUND_DARK)
        };

        // Paint the opener
        let mut path = BezPath::new();
        if self.expand_lens.get(data) {
            // expanded: 'V' shape
            path.move_to((5.0, 7.0));
            path.line_to((9.0, 13.0));
            path.line_to((13.0, 7.0));
        } else {
            // collapsed: '>' shape
            path.move_to((7.0, 5.0));
            path.line_to((13.0, 9.0));
            path.line_to((7.0, 13.0));
        }
        let style = StrokeStyle::new()
            .line_cap(LineCap::Round)
            .line_join(LineJoin::Round);

        ctx.stroke_styled(path, &stroke_color, 2.5, &style);
    }
}

type TreeItemFactory<T> = Arc<dyn Fn() -> Box<dyn Widget<T>>>;
type OpenerFactory<T> = dyn Fn() -> Box<dyn Widget<T>>;

fn make_wedge<T: TreeNode, L: Lens<T, bool>>(expand_lens: L) -> Wedge<T, L> {
    Wedge {
        phantom: PhantomData,
        expand_lens,
    }
}

/// An internal widget used to display a single node and its children
/// This is used recursively to build the tree.
struct TreeNodeWidget<T, L>
where
    T: TreeNode,
    L: Lens<T, bool>,
{
    // the index of the widget in its parent
    index: usize,
    // The "opener" widget,
    opener: WidgetPod<T, Opener<T>>,
    /// The label for this node
    widget: WidgetPod<T, Box<dyn Widget<T>>>,
    /// The children of this tree node widget
    children: Vec<WidgetPod<T, Self>>,
    /// A factory closure for the user defined widget
    make_widget: TreeItemFactory<T>,
    /// A factory closure for the user defined opener
    make_opener: Arc<OpenerFactory<T>>,
    /// The user must provide a Lens<T, bool> that tells if
    /// the node is expanded or not.
    expand_lens: L,
}

impl<T: TreeNode, L: Lens<T, bool> + Clone> TreeNodeWidget<T, L> {
    /// Create a TreeNodeWidget from a TreeNode.
    fn new(
        make_widget: TreeItemFactory<T>,
        make_opener: Arc<OpenerFactory<T>>,
        index: usize,
        expand_lens: L, // expanded: bool,
    ) -> Self {
        Self {
            index,
            opener: WidgetPod::new(Opener {
                widget: WidgetPod::new(make_opener.clone()()),
            }),
            widget: WidgetPod::new(Box::new((make_widget)())),
            // expanded,
            children: Vec::new(),
            make_widget,
            make_opener,
            expand_lens,
        }
    }

    /// Expand or collapse the node.
    /// Returns whether new children were created.
    fn update_children(&mut self, data: &T) -> bool {
        let mut changed = false;
        if self.expand_lens.get(data) {
            if self.children.len() > data.children_count() {
                self.children.truncate(data.children_count());
                changed = true;
            }
            for index in 0..data.children_count() {
                changed |= index >= self.children.len();
                match self.children.get_mut(index) {
                    Some(c) => c.widget_mut().index = index,
                    None => self.children.push(WidgetPod::new(TreeNodeWidget::new(
                        self.make_widget.clone(),
                        self.make_opener.clone(),
                        index,
                        self.expand_lens.clone(),
                    ))),
                }
            }
        }
        changed
    }
}

impl<T: TreeNode, L: Lens<T, bool> + Clone> Widget<T> for TreeNodeWidget<T, L> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        // match event {
        //     Event::MouseMove(_) => (),
        //     _ => eprintln!("{:?} {:?}", ctx.widget_id(), event),
        // }
        let event = match event {
            Event::Notification(notif) if notif.is(TREE_OPEN) => {
                if data.is_branch() {
                    ctx.set_handled();
                    if !self.expand_lens.get(data) {
                        self.expand_lens.put(data, true);
                        if self.update_children(data) {
                            ctx.children_changed();
                        }
                        for child_widget_node in self.children.iter_mut() {
                            ctx.submit_command(TREE_CHILD_SHOW.to(child_widget_node.id()))
                        }
                    }
                    None
                } else {
                    Some(event)
                }
            }
            Event::Notification(notif) if notif.is(TREE_NODE_REMOVE) => {
                // we were commanded to remove ourselves. Let's tell our parent.
                ctx.submit_notification(TREE_CHILD_REMOVE_INTERNAL.with(self.index as i32));
                ctx.set_handled();
                None
            }
            Event::Notification(notif) if notif.is(TREE_CHILD_REMOVE_INTERNAL) => {
                // get the index to remove from the notification
                let index =
                    usize::try_from(*notif.get(TREE_CHILD_REMOVE_INTERNAL).unwrap()).unwrap();
                // remove the widget and the data
                self.children.remove(index);
                data.rm_child(index);
                // update our children
                self.update_children(data);
                ctx.set_handled();
                ctx.children_changed();
                None
            }
            Event::Notification(notif) if notif.is(TREE_CHROOT) => {
                data.chroot(None);
                ctx.submit_notification(TREE_CHROOT_CHILD.with(self.index));
                ctx.set_handled();
                None
            }
            Event::Notification(notif) if notif.is(TREE_CHROOT_CHILD) => {
                let chroot_idx = notif.get(TREE_CHROOT_CHILD).unwrap();
                data.chroot(Some(*chroot_idx));
                ctx.submit_notification(TREE_CHROOT_CHILD.with(self.index));
                ctx.set_handled();
                None
            }
            Event::Command(cmd) if cmd.is(TREE_NOTIFY_CHROOT) => {
                ctx.submit_command(cmd.clone().to(self.widget.id()));
                ctx.submit_command(cmd.clone().to(self.opener.id()));
                ctx.children_changed();
                ctx.request_paint();
                ctx.set_handled();
                None
            }
            Event::Notification(notif) if notif.is(TREE_CHROOT_UP) => {
                if data.get_chroot().is_some() {
                    data.chroot(None);
                    ctx.set_handled();
                }
                None
            }
            Event::Notification(notif) if notif.is(TREE_NOTIFY_PARENT) => {
                if self.widget.id() != notif.source() {
                    let notif = notif.get(TREE_NOTIFY_PARENT).unwrap();
                    ctx.submit_command(TREE_NOTIFY_PARENT.with(*notif).to(self.widget.id()));
                    ctx.set_handled();
                }
                None
            }
            _ => Some(event),
        };

        // get the unhandled event or return
        let event = if let Some(evt) = event { evt } else { return };

        // don't go further with unhandled notifications
        if let Event::Notification(_) = event {
            return;
        }

        // propagate the event to the inner widget if needed.
        let chrooted = data.get_chroot();
        if chrooted.is_none() | event.should_propagate_to_hidden() {
            self.widget.event(ctx, event, data, env);
        }

        if data.is_branch() {
            // send the event to the opener if the widget is visible or the event also targets
            // hidden widgets.
            let before = self.expand_lens.get(data);
            if chrooted.is_none() | event.should_propagate_to_hidden() {
                self.opener.event(ctx, event, data, env);
            }
            let expanded = self.expand_lens.get(data);

            if expanded != before {
                // The opener widget has decided to change the expanded/collapsed state of the node,
                // handle it by expanding/collapsing children nodes as required.

                let cmd: Selector;
                if expanded {
                    cmd = TREE_CHILD_SHOW;
                    // create child widgets if needed.
                    if self.update_children(data) {
                        // New children were created, inform the context.
                        ctx.children_changed();
                    }
                } else {
                    cmd = TREE_CHILD_HIDE;
                    // self.children = vec![];
                };
                for child_widget_node in self.children.iter_mut() {
                    ctx.submit_command(cmd.to(child_widget_node.id()))
                }
                ctx.request_layout();
            }
            // Forward to children nodes
            if event.should_propagate_to_hidden() {
                // forward unconditionally
                for (index, child_widget_node) in self.children.iter_mut().enumerate() {
                    data.for_child_mut(index, |data: &mut T, _index: usize| {
                        if child_widget_node.is_initialized() {
                            child_widget_node.event(ctx, event, data, env)
                        }
                    });
                }
            } else if expanded & before {
                match chrooted {
                    None => {
                        for (index, child_widget_node) in self.children.iter_mut().enumerate() {
                            data.for_child_mut(index, |data: &mut T, _index: usize| {
                                child_widget_node.event(ctx, event, data, env)
                            });
                        }
                    }
                    Some(idx) => {
                        let child_widget_node = &mut self.children[idx];
                        data.for_child_mut(idx, |data: &mut T, _index: usize| {
                            child_widget_node.event(ctx, event, data, env)
                        });
                    }
                }
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let Some(idx) = data.get_chroot() {
            if !event.should_propagate_to_hidden() {
                return self.children[idx].lifecycle(ctx, event, data.get_child(idx), env);
            }
        }
        self.opener.lifecycle(ctx, event, data, env);
        self.widget.lifecycle(ctx, event, data, env);
        if data.is_branch() & (event.should_propagate_to_hidden() | self.expand_lens.get(data)) {
            for (index, child_widget_node) in self.children.iter_mut().enumerate() {
                let child_tree_node = data.get_child(index);
                child_widget_node.lifecycle(ctx, event, child_tree_node, env);
            }
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        self.widget.update(ctx, data, env);
        self.opener.update(ctx, data, env);

        if self.update_children(data) {
            if self.expand_lens.get(data) {
                for child_widget_node in self.children.iter_mut() {
                    // TODO: this is not true except for the new child. `update_children` should tell
                    // which child was added/removed...
                    ctx.submit_command(TREE_CHILD_SHOW.to(child_widget_node.id()))
                }
            }
            ctx.children_changed();
        }

        for (index, child_widget_node) in self.children.iter_mut().enumerate() {
            if child_widget_node.is_initialized() {
                let child_tree_node = data.get_child(index);
                child_widget_node.update(ctx, child_tree_node, env);
            }
        }
    }

    // TODO: the height calculation ignores the inner widget height. issue #61
    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        // if we're in the chroot path, just compute and return the chroot child's layout
        if let Some(idx) = data.get_chroot() {
            let chroot = &mut self.children[idx];
            let data = data.get_child(idx);
            let size = chroot.layout(ctx, bc, data, env);
            chroot.set_origin(ctx, data, env, Point::ORIGIN);
            return size;
        }

        let basic_size = env.get(theme::BASIC_WIDGET_HEIGHT);
        let indent = env.get(theme::BASIC_WIDGET_HEIGHT); // For a lack of a better definition
        let mut min_width = bc.min().width;
        let mut max_width = bc.max().width;

        // Top left, the opener
        self.opener.layout(
            ctx,
            &BoxConstraints::tight(Size::new(basic_size, basic_size)),
            data,
            env,
        );
        self.opener.set_origin(ctx, data, env, Point::ORIGIN);

        // Immediately on the right, the node widget
        let widget_size = self.widget.layout(
            ctx,
            &BoxConstraints::new(
                Size::new(min_width, basic_size),
                Size::new(max_width, basic_size),
            ),
            data,
            env,
        );
        self.widget
            .set_origin(ctx, data, env, Point::new(basic_size, 0.0));

        // This is the computed size of this node. We start with the size of the widget,
        // and will increase for each child node.
        let mut size = Size::new(indent + widget_size.width, basic_size);

        // Below, the children nodes, but only if expanded
        if self.expand_lens.get(data) && max_width > indent {
            if min_width > indent {
                min_width -= min_width;
            } else {
                min_width = 0.0;
            }
            max_width -= indent;

            let mut next_index: usize = 0;
            for (index, child_widget_node) in self.children.iter_mut().enumerate() {
                // In case we have lazily instantiated children nodes,
                // we may skip some indices. This catches up the correct height.
                if index != next_index {
                    size.height += (index - next_index) as f64 * basic_size;
                }
                next_index = index + 1;

                // Layout and position a child node
                let child_tree_node = data.get_child(index);
                let child_bc = BoxConstraints::new(
                    Size::new(min_width, 0.0),
                    Size::new(max_width, f64::INFINITY),
                );
                let child_size = child_widget_node.layout(ctx, &child_bc, child_tree_node, env);
                let child_pos = Point::new(indent, size.height); // We position the child at the current height
                child_widget_node.set_origin(ctx, child_tree_node, env, child_pos);
                size.height += child_size.height; // Increment the height of this node by the height of this child node
                if indent + child_size.width > size.width {
                    size.width = indent + child_size.width;
                }
            }
        }
        bc.constrain(size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        // if we're in the chroot path, paint the chroot child.
        if let Some(idx) = data.get_chroot() {
            return self.children[idx].paint(ctx, data.get_child(idx), env);
        }
        self.opener.paint(ctx, data, env);
        self.widget.paint(ctx, data, env);
        if data.is_branch() & self.expand_lens.get(data) {
            for (index, child_widget_node) in self.children.iter_mut().enumerate() {
                let child_tree_node = data.get_child(index);
                child_widget_node.paint(ctx, child_tree_node, env);
            }
        }
    }
}

/// Tree Implementation
impl<T: TreeNode, L: Lens<T, bool> + Clone + 'static> Tree<T, L> {
    /// Create a new Tree widget
    pub fn new<W: Widget<T> + 'static>(
        make_widget: impl Fn() -> W + 'static,
        expand_lens: L,
    ) -> Self {
        let make_widget: TreeItemFactory<T> = Arc::new(move || Box::new(make_widget()));
        let el = expand_lens.clone();
        let make_opener: Arc<Box<OpenerFactory<T>>> =
            Arc::new(Box::new(move || Box::new(make_wedge(el.clone()))));
        Tree {
            root_node: WidgetPod::new(TreeNodeWidget::new(
                make_widget,
                make_opener,
                0,
                expand_lens,
            )),
            // dummy chroot id at creation.
            chroot: WidgetId::next(),
        }
    }

    /// Pass a closure to define your own opener widget
    pub fn with_opener<W: Widget<T> + 'static>(
        mut self,
        closure: impl Fn() -> W + 'static,
    ) -> Self {
        self.root_node.widget_mut().make_opener = Arc::new(move || Box::new(closure()));
        self.root_node.widget_mut().opener = WidgetPod::new(Opener {
            widget: WidgetPod::new(self.root_node.widget_mut().make_opener.clone()()),
        });
        self
    }

    fn get_chroot_from<'a>(
        widget: &'a mut WidgetPod<T, TreeNodeWidget<T, L>>,
        data: &'a T,
    ) -> (&'a mut WidgetPod<T, TreeNodeWidget<T, L>>, &'a T) {
        match data.get_chroot() {
            Some(idx) => Tree::<T, L>::get_chroot_from(
                &mut widget.widget_mut().children[idx],
                data.get_child(idx),
            ),
            None => (widget, data),
        }
    }
}

/// Default tree, supplying Label if the nodes implement the Display trait.
/// TODO: this DOES NOT implement `Default`, as we must pass the expand_lens.
///       At least, find a less confusing name.
impl<T: TreeNode + Display, L: Lens<T, bool> + Clone + 'static> Tree<T, L> {
    pub fn default(expand_lens: L) -> Self {
        let make_widget: TreeItemFactory<T> =
            Arc::new(|| Box::new(Label::dynamic(|data: &T, _env| format!("{}", data))));
        let el = expand_lens.clone();
        let make_opener: Arc<Box<OpenerFactory<T>>> =
            Arc::new(Box::new(move || Box::new(make_wedge(el.clone()))));
        Tree {
            root_node: WidgetPod::new(TreeNodeWidget::new(
                make_widget,
                make_opener,
                0,
                expand_lens,
            )),
            chroot: WidgetId::next(),
        }
    }
}

// Implement the Widget trait for Tree
impl<T: TreeNode, L: Lens<T, bool> + Clone + 'static> Widget<T> for Tree<T, L> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Event::Notification(notif) = event {
            if notif.is(TREE_CHROOT_CHILD) {
                ctx.set_handled();
                let root_node_id = self.root_node.id();
                let (chroot, _) = Tree::<T, L>::get_chroot_from(&mut self.root_node, data);
                if chroot.id() != self.chroot {
                    ctx.submit_command(
                        TREE_NOTIFY_CHROOT
                            .with(if chroot.id() == root_node_id {
                                ChrootStatus::ROOT
                            } else {
                                ChrootStatus::YES
                            })
                            .to(chroot.id()),
                    );
                    ctx.submit_command(TREE_NOTIFY_CHROOT.with(ChrootStatus::NO).to(self.chroot));
                    self.chroot = chroot.id();
                }
                ctx.children_changed();
            }
            return;
        }
        // self.chroot_up.event(ctx, event, &mut (), env);
        self.root_node.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            // self.root_node.widget_mut().make_widget();
            // init the chroot state.
            let root_node_id = self.root_node.id();
            let (chroot, _) = Tree::<T, L>::get_chroot_from(&mut self.root_node, data);
            if chroot.id() != root_node_id {
                ctx.submit_command(TREE_NOTIFY_CHROOT.with(ChrootStatus::YES).to(chroot.id()));
            }
            ctx.submit_command(TREE_NOTIFY_CHROOT.with(ChrootStatus::ROOT).to(root_node_id));
        }
        self.root_node.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        let root_node_id = self.root_node.id();
        let (chroot, _) = Tree::<T, L>::get_chroot_from(&mut self.root_node, data);
        if chroot.id() != self.chroot {
            ctx.submit_command(
                TREE_NOTIFY_CHROOT
                    .with(if chroot.id() == root_node_id {
                        ChrootStatus::ROOT
                    } else {
                        ChrootStatus::YES
                    })
                    .to(chroot.id()),
            );
            ctx.submit_command(TREE_NOTIFY_CHROOT.with(ChrootStatus::NO).to(self.chroot));
            self.chroot = chroot.id();
        }
        self.root_node.update(ctx, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let size = self.root_node.layout(ctx, bc, data, env);
        self.root_node.set_origin(ctx, data, env, Point::ORIGIN);
        // TODO: ctx.set_paint_insets...
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        let (root, chroot_data) = Tree::<T, L>::get_chroot_from(&mut self.root_node, data);
        root.paint(ctx, chroot_data, env);
    }
}
