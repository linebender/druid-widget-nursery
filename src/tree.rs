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
use druid::widget::Label;
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};

use crate::selectors;

selectors! {
    TREE_CHILD_CREATED,
    TREE_CHILD_REMOVE,
    TREE_CHILD_REMOVE_INTERNAL: i32,
    TREE_OPEN,
    TREE_OPEN_PARENT,
    TREE_CHILD_SHOW,
    TREE_CHILD_HIDE,
}

/// A tree widget for a collection of items organized in a hierachical way.
pub struct Tree<T>
where
    T: TreeNode,
{
    /// The root node of this tree
    root_node: WidgetPod<T, TreeNodeWidget<T>>,
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
    widget: WidgetPod<(bool, T), Box<dyn Widget<(bool, T)>>>,
}

impl<T: TreeNode> Opener<T> {
    pub fn new(widget: Box<dyn Widget<(bool, T)>>) -> Opener<T> {
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

impl<T: TreeNode> Widget<(bool, T)> for Wedge<T> {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut (bool, T), _env: &Env) {}

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &(bool, T),
        _env: &Env,
    ) {
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx,
        _old_data: &(bool, T),
        _data: &(bool, T),
        _env: &Env,
    ) {
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &(bool, T),
        env: &Env,
    ) -> Size {
        let size = env.get(theme::BASIC_WIDGET_HEIGHT);
        bc.constrain(Size::new(size, size))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &(bool, T), env: &Env) {
        if !data.1.is_branch() {
            return ();
        }
        let stroke_color = if ctx.is_hot() {
            env.get(theme::FOREGROUND_LIGHT)
        } else {
            env.get(theme::FOREGROUND_DARK)
        };

        // Paint the opener
        let mut path = BezPath::new();
        if data.0 {
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
impl<T: TreeNode> Widget<(bool, T)> for Opener<T>
where
    T: Data,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut (bool, T), _env: &Env) {
        match event {
            Event::MouseDown(_) => {
                ctx.set_active(true);
                ctx.request_paint();
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    if ctx.is_hot() {
                        data.0 = !data.0;
                    }
                    ctx.request_paint();
                }
            }
            _ => (),
        }
        self.widget.event(ctx, event, data, _env);
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &(bool, T),
        env: &Env,
    ) {
        self.widget.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &(bool, T), data: &(bool, T), env: &Env) {
        self.widget.update(ctx, data, env)
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &(bool, T),
        env: &Env,
    ) -> Size {
        self.widget.set_origin(ctx, data, env, Point::ORIGIN);
        bc.constrain(self.widget.layout(ctx, bc, data, env))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &(bool, T), env: &Env) {
        self.widget.paint(ctx, data, env)
    }
}

type TreeItemFactory<T> = Arc<Box<dyn Fn() -> Box<dyn Widget<T>>>>;
type OpenerFactory<T> = dyn Fn() -> Box<dyn Widget<(bool, T)>>;

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
    opener: WidgetPod<(bool, T), Opener<T>>,
    /// The label for this node
    widget: WidgetPod<T, Box<dyn Widget<T>>>,
    /// Whether the node is expanded or collapsed
    expanded: bool,
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
        expanded: bool,
    ) -> Self {
        Self {
            index,
            opener: WidgetPod::new(Opener {
                widget: WidgetPod::new(make_opener.clone()()),
            }),
            widget: WidgetPod::new(Box::new((make_widget)())),
            expanded,
            children: Vec::new(),
            make_widget,
            make_opener,
        }
    }

    /// Expand or collapse the node.
    /// Returns whether new children were created.
    fn update_children(&mut self, data: &T) -> bool {
        if self.expanded {
            let mut new_children = false;
            for index in 0..data.children_count() {
                new_children |= index >= self.children.len();
                match self.children.get_mut(index) {
                    Some(c) => c.widget_mut().index = index,
                    None => self.children.push(WidgetPod::new(TreeNodeWidget::new(
                        self.make_widget.clone(),
                        self.make_opener.clone(),
                        index,
                        false,
                    ))),
                }
            }
            new_children
        } else {
            false
        }
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
                if self.expanded {
                    for child_widget_node in self.children.iter_mut() {
                        // TODO: this is not true except for the new child. `updage_children` should tell
                        // which child was added/removed...
                        ctx.submit_command(TREE_CHILD_SHOW.to(child_widget_node.id()))
                    }
                }
                ctx.children_changed();
            } else if notif.is(TREE_OPEN) {
                ctx.set_handled();
                if !self.expanded {
                    self.expanded = true;
                    self.update_children(data);
                    ctx.children_changed();
                    for child_widget_node in self.children.iter_mut() {
                        ctx.submit_command(TREE_CHILD_SHOW.to(child_widget_node.id()))
                    }
                }
            } else if notif.is(TREE_CHILD_REMOVE) {
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
            }
            return;
        }
        self.widget.event(ctx, event, data, env);

        for (index, child_widget_node) in self.children.iter_mut().enumerate() {
            data.for_child_mut(index, |data: &mut T, _index: usize| {
                child_widget_node.event(ctx, event, data, env)
            });
        }

        // Propagate the event to the opener
        let mut opener_data = (self.expanded, data.clone());
        self.opener.event(ctx, event, &mut opener_data, env);
        let wedge_expanded = opener_data.0;

        if wedge_expanded != self.expanded {
            // The opener widget has decided to change the expanded/collapsed state of the node,
            // handle it by expanding/collapsing children nodes as required.
            self.expanded = wedge_expanded;
            if self.update_children(data) {
                // New children were created, inform the context.
                ctx.children_changed();
            }

            let cmd = if wedge_expanded {
                TREE_CHILD_SHOW
            } else {
                TREE_CHILD_HIDE
            };
            for child_widget_node in self.children.iter_mut() {
                ctx.submit_command(cmd.to(child_widget_node.id()))
            }

            ctx.request_layout();
            ctx.request_update();
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        // eprintln!("{:?}", event);
        self.opener
            .lifecycle(ctx, event, &(self.expanded, data.clone()), env);
        self.widget.lifecycle(ctx, event, data, env);
        for (index, child_widget_node) in self.children.iter_mut().enumerate() {
            let child_tree_node = data.get_child(index);
            child_widget_node.lifecycle(ctx, event, child_tree_node, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        // eprintln!("update_widget {}", self.expanded);
        self.opener.update(
            ctx,
            &(self.expanded, data.clone()),
            // &(self.expanded, data.clone()),
            env,
        );

        // the data may not change but the update may comme from explicit request_update() call in
        // if an node is open/closed...
        // TODO: is this a good idea? should we push the open state on the application state (thus
        //       forcing the user to handle somehow the opening state, like list_filter does with
        //       the filtered indices list)
        if !old_data.same(data) {
            eprintln!("not same");
            eprintln!("{:?}", old_data);
            eprintln!("{:?}", data);
            self.widget.update(ctx, data, env);
        }
        // ...that's why we have to rebuild the widget tree for each and every event... :/
        for (index, child_widget_node) in self.children.iter_mut().enumerate() {
            let child_tree_node = data.get_child(index);
            child_widget_node.update(ctx, child_tree_node, env);
        }
        ctx.children_changed();
    }

    // TODO: the height calculation seems to ignore the inner widget (at least on X11). issue #61
    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let basic_size = env.get(theme::BASIC_WIDGET_HEIGHT);
        let indent = env.get(theme::BASIC_WIDGET_HEIGHT); // For a lack of a better definition
        let mut min_width = bc.min().width;
        let mut max_width = bc.max().width;

        // Top left, the opener
        let data2 = (self.expanded, data.clone());
        self.opener.layout(
            ctx,
            &BoxConstraints::tight(Size::new(basic_size, basic_size)),
            &data2,
            env,
        );
        self.opener
            .set_origin(ctx, &(self.expanded, data.clone()), env, Point::ORIGIN);

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
        if self.expanded && max_width > indent {
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
        self.opener.paint(ctx, &(self.expanded, data.clone()), env);
        self.widget.paint(ctx, data, env);
        if data.is_branch() & self.expanded {
            for (index, child_widget_node) in self.children.iter_mut().enumerate() {
                let child_tree_node = data.get_child(index);
                child_widget_node.paint(ctx, child_tree_node, env);
            }
        }
        // self.widget.paint(ctx, data, env);
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
            root_node: WidgetPod::new(TreeNodeWidget::new(make_widget, make_opener, 0, false)),
        }
    }

    pub fn with_opener<W: Widget<(bool, T)> + 'static>(
        mut self,
        closure: impl Fn() -> W + 'static,
    ) -> Self {
        self.root_node.widget_mut().make_opener = Arc::new(Box::new(move || Box::new(closure())));
        self.root_node.widget_mut().opener = WidgetPod::new(Opener {
            widget: WidgetPod::new(self.root_node.widget_mut().make_opener.clone()()),
        });
        self
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
            root_node: WidgetPod::new(TreeNodeWidget::new(make_widget, make_opener, 0, false)),
        }
    }
}

// Implement the Widget trait for Tree
impl<T: TreeNode> Widget<T> for Tree<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        // eprintln!("{:?}", event);
        self.root_node.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            self.root_node.widget_mut().make_widget();
        }
        self.root_node.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        self.root_node.update(ctx, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        self.root_node.set_origin(ctx, data, env, Point::ORIGIN);
        bc.constrain(self.root_node.layout(ctx, bc, data, env))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        let background_color = env.get(theme::BACKGROUND_LIGHT);
        let clip_rect = ctx.size().to_rect();
        ctx.fill(clip_rect, &background_color);
        self.root_node.paint(ctx, data, env);
    }
}
