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
use druid::theme;
use druid::widget::{Button, Click, ControllerHost, Label};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, Selector, UpdateCtx, Widget, WidgetPod,
};

use crate::selectors;

selectors! {
    TREE_CHILD_CREATED,
    TREE_NODE_REMOVE,
    TREE_CHILD_REMOVE_INTERNAL: i32,
    TREE_OPEN,
    TREE_CHILD_SHOW,
    TREE_CHILD_HIDE,
    TREE_OPEN_STATE: bool,
    TREE_CHROOT: Vec<usize>,
    TREE_CHROOT_UP,
    TREE_CHROOT_CHILD: usize,
    TREE_NOTIFY_PARENT: Selector,
}

/// A tree widget for a collection of items organized in a hierachical way.
pub struct Tree<T>
where
    T: TreeNode,
{
    /// The root node of this tree
    root_node: WidgetPod<T, TreeNodeWidget<T>>,
    chroot: Vec<usize>,
    chroot_up: WidgetPod<(), ControllerHost<Button<()>, Click<()>>>,
}

/// A tree node, with methods providing its own label and its children.
/// This is the data expected by the tree widget.
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

    fn open(&mut self, state: bool);

    fn is_open(&self) -> bool;

    fn get_chroot(&self) -> Option<usize> {
        None
    }

    #[allow(unused_variables)]
    fn chroot(&mut self, idx: Option<usize>) {}

    fn is_branch(&self) -> bool {
        self.children_count() > 0
    }

    #[allow(unused_variables)]
    fn rm_child(&mut self, index: usize) {}
}

pub struct Opener<T>
where
    T: TreeNode,
{
    widget: WidgetPod<T, Box<dyn Widget<T>>>,
}

impl<T: TreeNode> Opener<T> {
    pub fn new(widget: Box<dyn Widget<T>>) -> Opener<T> {
        Opener {
            widget: WidgetPod::new(widget),
        }
    }
}

pub struct Wedge<T>
where
    T: TreeNode,
{
    phantom: PhantomData<T>,
}

impl<T: TreeNode> Widget<T> for Wedge<T> {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut T, _env: &Env) {}

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &T, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &T, _data: &T, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &T, env: &Env) -> Size {
        let size = env.get(theme::BASIC_WIDGET_HEIGHT);
        bc.constrain(Size::new(size, size))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        if !data.is_branch() {
            return ();
        }
        let stroke_color = if ctx.is_hot() {
            env.get(theme::FOREGROUND_LIGHT)
        } else {
            env.get(theme::FOREGROUND_DARK)
        };

        // Paint the opener
        let mut path = BezPath::new();
        if data.is_open() {
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

/// Implementing Widget for the Opener.
/// This widget's data is simply a boolean telling whether is is expanded or collapsed.
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
                eprintln!("mouse up!!!!!!!!!!!");
                if ctx.is_active() {
                    ctx.set_active(false);
                    if ctx.is_hot() {
                        data.open(!data.is_open());
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
        self.widget.set_origin(ctx, data, env, Point::ORIGIN);
        bc.constrain(self.widget.layout(ctx, bc, data, env))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.widget.paint(ctx, data, env)
    }
}

type TreeItemFactory<T> = Arc<Box<dyn Fn() -> Box<dyn Widget<T>>>>;
type OpenerFactory<T> = dyn Fn() -> Box<dyn Widget<T>>;

fn make_wedge<T: TreeNode>() -> Wedge<T> {
    Wedge {
        phantom: PhantomData,
    }
}

/// An internal widget used to display a single node and its children
/// This is used recursively to build the tree.
struct TreeNodeWidget<T>
where
    T: TreeNode,
{
    // the index of the widget in its parent
    index: usize,
    // The "opener" widget,
    opener: WidgetPod<T, Opener<T>>,
    /// The label for this node
    widget: WidgetPod<T, Box<dyn Widget<T>>>,
    /// Whether the node is expanded or collapsed
    // expanded: bool,
    /// The children of this tree node widget
    children: Vec<WidgetPod<T, Self>>,
    /// A factory closure for building widgets for the children nodes
    make_widget: TreeItemFactory<T>,
    make_opener: Arc<Box<OpenerFactory<T>>>,
}

impl<T: TreeNode> TreeNodeWidget<T> {
    /// Create a TreeNodeWidget from a TreeNode.
    fn new(
        make_widget: TreeItemFactory<T>,
        make_opener: Arc<Box<OpenerFactory<T>>>,
        index: usize,
        // expanded: bool,
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
        }
    }

    /// Expand or collapse the node.
    /// Returns whether new children were created.
    fn update_children(&mut self, data: &T) -> bool {
        let mut changed = false;
        if data.is_open() {
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
                    ))),
                }
            }
        }
        changed
    }

    /// Build the widget for this node, from the provided data
    fn make_widget(&mut self) {
        self.widget = WidgetPod::new((self.make_widget)());
    }
}

impl<T: TreeNode> Widget<T> for TreeNodeWidget<T>
where
    T: TreeNode,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        // eprintln!("{:?}", event);
        if let Event::Notification(notif) = event {
            if notif.is(TREE_CHILD_CREATED) {
                ctx.set_handled();
                self.update_children(data);
                if data.is_open() {
                    for child_widget_node in self.children.iter_mut() {
                        // TODO: this is not true except for the new child. `updage_children` should tell
                        // which child was added/removed...
                        ctx.submit_command(TREE_CHILD_SHOW.to(child_widget_node.id()))
                    }
                }
                ctx.children_changed();
            } else if notif.is(TREE_OPEN) {
                ctx.set_handled();
                if !data.is_open() {
                    data.open(true);
                    self.update_children(data);
                    ctx.children_changed();
                    for child_widget_node in self.children.iter_mut() {
                        ctx.submit_command(TREE_CHILD_SHOW.to(child_widget_node.id()))
                    }
                }
            } else if notif.is(TREE_NODE_REMOVE) {
                // we were comanded to remove ourselves. Let's tell our parent.
                ctx.submit_notification(TREE_CHILD_REMOVE_INTERNAL.with(self.index as i32));
                ctx.set_handled();
            } else if notif.is(TREE_CHILD_REMOVE_INTERNAL) {
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
            } else if notif.is(TREE_CHROOT) {
                eprintln!("{:?}", notif);
                // let chroot = notif.get(TREE_CHROOT).unwrap();
                // eprintln!("{:?}", chroot);
                // let mut new_chroot = vec![self.index];
                // new_chroot.splice(1..1, chroot.iter().cloned());
                // ctx.submit_notification(TREE_CHROOT.with(new_chroot));
                ctx.submit_notification(TREE_CHROOT_CHILD.with(self.index));
                ctx.set_handled();
            } else if notif.is(TREE_CHROOT_CHILD) {
                eprintln!("{:?}", notif);
                let chroot_idx = notif.get(TREE_CHROOT_CHILD).unwrap();
                eprintln!("{:?}", chroot_idx);
                data.chroot(Some(*chroot_idx));
                ctx.submit_notification(TREE_CHROOT_CHILD.with(self.index));
                ctx.set_handled();
            } else if notif.is(TREE_NOTIFY_PARENT) {
                eprintln!("{:?}", notif);
                if self.widget.id() != notif.source() {
                    let notif = notif.get(TREE_NOTIFY_PARENT).unwrap();
                    ctx.submit_command(TREE_NOTIFY_PARENT.with(notif.clone()).to(self.widget.id()));
                    ctx.set_handled();
                }
                // ctx.submit_notification(TREE_CHROOT_CHILD.with(self.index));
            } else {
                if self.widget.id() != notif.source() {
                    eprintln!("RESEND NOTIFICATION");
                    self.widget.event(ctx, event, data, env);
                    ctx.set_handled();
                }
            }
            return;
        }
        let chrooted = data.get_chroot();
        if chrooted.is_none() | event.should_propagate_to_hidden() {
            self.widget.event(ctx, event, data, env);
        }

        if data.is_branch() {
            let before = data.is_open();
            if chrooted.is_none() | event.should_propagate_to_hidden() {
                self.opener.event(ctx, event, data, env);
            }
            let expanded = data.is_open();

            if expanded != before {
                // The opener widget has decided to change the expanded/collapsed state of the node,
                // handle it by expanding/collapsing children nodes as required.

                let cmd: Selector;
                if expanded {
                    cmd = TREE_CHILD_SHOW;
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
            if event.should_propagate_to_hidden() {
                for (index, child_widget_node) in self.children.iter_mut().enumerate() {
                    data.for_child_mut(index, |data: &mut T, _index: usize| {
                        child_widget_node.event(ctx, event, data, env)
                    });
                }
            } else if expanded & before {
                if chrooted.is_none() {
                    for (index, child_widget_node) in self.children.iter_mut().enumerate() {
                        data.for_child_mut(index, |data: &mut T, _index: usize| {
                            child_widget_node.event(ctx, event, data, env)
                        });
                    }
                } else {
                    let idx = chrooted.unwrap();
                    let child_widget_node = &mut self.children[idx];
                    data.for_child_mut(idx, |data: &mut T, _index: usize| {
                        child_widget_node.event(ctx, event, data, env)
                    });
                }
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        // eprintln!("{:?}", event);
        if let Some(idx) = data.get_chroot() {
            if !event.should_propagate_to_hidden() {
                return self.children[idx].lifecycle(ctx, event, data.get_child(idx), env);
            }
        }
        self.opener.lifecycle(ctx, event, data, env);
        self.widget.lifecycle(ctx, event, data, env);
        if data.is_branch() {
            if event.should_propagate_to_hidden() | data.is_open() {
                for (index, child_widget_node) in self.children.iter_mut().enumerate() {
                    let child_tree_node = data.get_child(index);
                    child_widget_node.lifecycle(ctx, event, child_tree_node, env);
                }
            }
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        eprintln!(
            "NNNNNNNNNOOOOOOOOOOODDDDDDDDDDDEEEEEEEEE {:?}",
            ctx.widget_id()
        );
        eprintln!("{:?}", old_data);
        eprintln!("{:?}", data);
        self.widget.update(ctx, data, env);
        self.opener.update(ctx, data, env);

        for (index, child_widget_node) in self.children.iter_mut().enumerate() {
            let child_tree_node = data.get_child(index);
            child_widget_node.update(ctx, child_tree_node, env);
        }
        ctx.children_changed();
    }

    // TODO: the height calculation seems to ignore the inner widget (at least on X11). issue #61
    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        if bc.max().height == 0.0 {
            self.opener.set_origin(ctx, data, env, Point::ORIGIN);
            self.widget.set_origin(ctx, data, env, Point::ORIGIN);
            return Size::new(0.0, 0.0);
        }
        if let Some(idx) = data.get_chroot() {
            return self.children[idx].layout(ctx, bc, data.get_child(idx), env);
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
        if data.is_open() && max_width > indent {
            if min_width > indent {
                min_width -= min_width;
            } else {
                min_width = 0.0;
            }
            max_width -= indent;

            let mut next_index: usize = 0;
            for (index, child_widget_node) in self.children.iter_mut().enumerate() {
                // In case we have lazily instanciated children nodes,
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
        if let Some(idx) = data.get_chroot() {
            return self.children[idx].paint(ctx, data.get_child(idx), env);
        }
        self.opener.paint(ctx, data, env);
        self.widget.paint(ctx, data, env);
        if data.is_branch() & data.is_open() {
            for (index, child_widget_node) in self.children.iter_mut().enumerate() {
                let child_tree_node = data.get_child(index);
                child_widget_node.paint(ctx, child_tree_node, env);
            }
        }
    }
}

/// Tree Implementation
impl<T: TreeNode> Tree<T> {
    /// Create a new Tree widget
    pub fn new<W: Widget<T> + 'static>(make_widget: impl Fn() -> W + 'static) -> Self {
        let make_widget: TreeItemFactory<T> = Arc::new(Box::new(move || Box::new(make_widget())));
        let make_opener: Arc<Box<OpenerFactory<T>>> =
            Arc::new(Box::new(|| Box::new(make_wedge::<T>())));
        Tree {
            root_node: WidgetPod::new(TreeNodeWidget::new(make_widget, make_opener, 0)),
            chroot: vec![],
            chroot_up: WidgetPod::new(Button::new("..".to_owned()).on_click(|ctx, data, env| {
                ctx.submit_notification(TREE_CHROOT_UP);
            })),
        }
    }

    pub fn with_opener<W: Widget<T> + 'static>(
        mut self,
        closure: impl Fn() -> W + 'static,
    ) -> Self {
        self.root_node.widget_mut().make_opener = Arc::new(Box::new(move || Box::new(closure())));
        self.root_node.widget_mut().opener = WidgetPod::new(Opener {
            widget: WidgetPod::new(self.root_node.widget_mut().make_opener.clone()()),
        });
        self
    }

    fn get_chrooted<'a>(&'a mut self) -> &mut WidgetPod<T, TreeNodeWidget<T>> {
        let mut root_node = &mut self.root_node;
        if self.chroot.len() > 0 {
            for idx in &self.chroot {
                root_node = &mut root_node.widget_mut().children[*idx];
            }
        }
        root_node
    }

    fn get_chroot_from<'a>(
        widget: &'a mut WidgetPod<T, TreeNodeWidget<T>>,
        data: &'a T,
    ) -> (&'a mut WidgetPod<T, TreeNodeWidget<T>>, &'a T) {
        match data.get_chroot() {
            Some(idx) => Tree::<T>::get_chroot_from(
                &mut widget.widget_mut().children[idx],
                data.get_child(idx),
            ),
            None => (widget, data),
        }
    }
}

/// Default tree implementation, supplying Label if the nodes implement the Display trait
impl<T: TreeNode + Display> Default for Tree<T> {
    fn default() -> Self {
        let make_widget: TreeItemFactory<T> = Arc::new(Box::new(|| {
            Box::new(Label::dynamic(|data: &T, _env| format!("{}", data)))
        }));
        let make_opener: Arc<Box<OpenerFactory<T>>> =
            Arc::new(Box::new(|| Box::new(make_wedge::<T>())));
        Tree {
            root_node: WidgetPod::new(TreeNodeWidget::new(make_widget, make_opener, 0)),
            chroot: vec![],
            chroot_up: WidgetPod::new(Button::new("..".to_owned()).on_click(|ctx, data, env| {})),
        }
    }
}

// Implement the Widget trait for Tree
impl<T: TreeNode> Widget<T> for Tree<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        eprintln!("{:?}", ctx.widget_id());
        if let Event::Notification(notif) = event {
            eprintln!("############################# {:?}", event);
            if notif.is(TREE_CHROOT) {
                eprintln!("tree... {:?}", notif);
                self.chroot = notif.get(TREE_CHROOT).unwrap()[1..].to_vec();
                eprintln!("{:?}", self.chroot);
                ctx.set_handled();
                ctx.children_changed();
            }
            if notif.is(TREE_CHROOT_CHILD) {
                ctx.set_handled();
                ctx.children_changed();
            }
            if notif.is(TREE_CHROOT_UP) {
                self.chroot.pop();
                ctx.set_handled();
                ctx.children_changed();
            }
            return;
        }
        // self.chroot_up.event(ctx, event, &mut (), env);
        self.root_node.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            self.root_node.widget_mut().make_widget();
        }
        self.chroot_up.lifecycle(ctx, event, &(), env);
        self.root_node.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        eprintln!("RRRRRRRRRRRRRRRRRRRRROOOOOOOOOTTTT{:?}", ctx.widget_id());
        self.root_node.update(ctx, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        // self.root_node.layout(
        //     ctx,
        //     &BoxConstraints::new(Size::new(0.0, 0.0), Size::new(0.0, 0.0)),
        //     data,
        //     env,
        // );
        let chroot = self.chroot.clone();
        let mut chroot_data = data;
        for idx in &chroot {
            chroot_data = chroot_data.get_child(*idx);
        }

        let origin = if chroot.len() > 0 {
            self.chroot_up.set_origin(ctx, &(), env, Point::ORIGIN);
            let btn_sz = self.chroot_up.layout(ctx, bc, &(), env);
            Point::new(btn_sz.width, 0.0)
        } else {
            // self.chroot_up.layout(ctx, BoxConstraints::
            Point::ORIGIN
        };
        // let root = self.get_chrooted();
        let (root, chroot_data) = Self::get_chroot_from(&mut self.root_node, data);
        root.set_origin(ctx, chroot_data, env, origin);
        let root_size = root.layout(ctx, bc, chroot_data, env);
        Size::new(root_size.width + origin.x, root_size.height)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        // let background_color = env.get(theme::BACKGROUND_LIGHT);
        // let clip_rect = ctx.size().to_rect();
        // ctx.fill(clip_rect, &background_color);

        let chroot = self.chroot.clone();
        let mut chroot_data = data;
        for idx in &chroot {
            chroot_data = chroot_data.get_child(*idx);
        }
        if chroot.len() > 0 {
            self.chroot_up.paint(ctx, &(), env);
        }
        // let root = self.get_chrooted();
        let (root, chroot_data) = Self::get_chroot_from(&mut self.root_node, data);
        root.paint(ctx, chroot_data, env);
    }
}
