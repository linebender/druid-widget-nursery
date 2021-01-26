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

use std::collections::BTreeMap;
use std::fmt::Display;
use std::sync::Arc;

use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::theme;
use druid::widget::Label;
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};

/// A tree widget for a collection of items organized in a hierachical way.
pub struct Tree<T>
where
    T: TreeNode + Data + Default,
{
    /// The root node of this tree
    root_node: TreeNodeWidget<T>,
}

/// A tree node, with methods providing its own label and its children.
/// This is the data expected by the tree widget.
pub trait TreeNode {
    /// Returns how many children are below this node. It could be zero if this is a leaf.
    fn children_count(&self) -> usize;

    /// Returns a reference to the node's child at the given index
    fn get_child(&self, index: usize) -> &Self
    where
        Self: Sized;

    /// Returns a mutable reference to the node's child at the given index
    fn get_child_mut(&mut self, index: usize) -> &mut Self
    where
        Self: Sized;
}

/// Wedge is an arbitrary name for the arrow-like icon marking whether a node is expanded or collapsed.
pub struct Wedge;

// Is "Chevron" a better name?
impl Wedge {
    pub fn new() -> Self {
        Wedge {}
    }
}

impl Default for Wedge {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementing Widget for the wedge.
/// This widget's data is simply a boolean telling whether is is expanded or collapsed.
impl Widget<bool> for Wedge {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, expanded: &mut bool, _env: &Env) {
        match event {
            Event::MouseDown(_) => {
                ctx.set_active(true);
                ctx.request_paint();
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    if ctx.is_hot() {
                        *expanded = !*expanded;
                    }
                    ctx.request_paint();
                }
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &bool, _env: &Env) {
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &bool, _data: &bool, _env: &Env) {}

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &bool,
        env: &Env,
    ) -> Size {
        let size = env.get(theme::BASIC_WIDGET_HEIGHT);
        bc.constrain(Size::new(size, size))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, expanded: &bool, env: &Env) {
        let stroke_color = if ctx.is_hot() {
            env.get(theme::FOREGROUND_LIGHT)
        } else {
            env.get(theme::FOREGROUND_DARK)
        };

        // Paint the wedge
        let mut path = BezPath::new();
        if *expanded {
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

type WidgetFactoryCallback<T> = Arc<Box<dyn Fn(&T) -> Box<dyn Widget<T>>>>;

/// An internal widget used to display a single node and its children
/// This is used recursively to build the tree.
struct TreeNodeWidget<T>
where
    T: TreeNode + Data + Default,
{
    // The "wedge" widget,
    wedge: WidgetPod<bool, Wedge>,

    /// The label for this node
    widget: WidgetPod<T, Box<dyn Widget<T>>>,

    /// Whether the node is expanded or collapsed
    expanded: bool,

    /// The children of this tree node widget
    /// A B-Tree is used in prevision for the case where only a subset of visible nodes
    /// are lazily instanciated
    children: BTreeMap<usize, WidgetPod<T, Self>>,

    /// A factory closure for building widgets for the children nodes
    make_widget: WidgetFactoryCallback<T>,
}

impl<T: TreeNode + Data + Default> TreeNodeWidget<T> {
    /// Create an empty default tree node widget
    fn default(make_widget: WidgetFactoryCallback<T>) -> Self {
        let default_node = T::default();
        Self::from_node(&default_node, make_widget)
    }

    /// Create a TreeNodeWidget from a TreeNode.
    fn from_node(node: &T, make_widget: WidgetFactoryCallback<T>) -> Self {
        TreeNodeWidget {
            wedge: WidgetPod::new(Wedge::new()),
            widget: WidgetPod::new(Box::new((make_widget)(node))),
            expanded: false,
            children: BTreeMap::new(),
            make_widget,
        }
    }

    /// Expand or collapse the node.
    /// Returns whether new children were created.
    fn expand(&mut self, data: &T, expanded: bool) -> bool {
        let mut new_children = false;
        if expanded {
            for index in 0..data.children_count() {
                new_children |= !self.children.contains_key(&index);
                let make_widget = self.make_widget.clone();
                self.children.entry(index).or_insert_with(|| {
                    WidgetPod::new(TreeNodeWidget::from_node(
                        data.get_child(index),
                        make_widget,
                    ))
                });
            }
        }
        self.expanded = expanded;
        new_children
    }

    /// Build the widget for this node, from the provided data
    fn make_widget(&mut self, data: &T) {
        self.widget = WidgetPod::new((self.make_widget)(data));
    }
}

impl<T: TreeNode + Data + Default> Widget<T> for TreeNodeWidget<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        // We first propagate the event down to the label and children nodes,
        // as those may change if this is an "expand" event
        self.widget.event(ctx, event, data, env);
        for (index, child_widget_node) in self.children.iter_mut() {
            let child_tree_node = data.get_child_mut(*index);
            child_widget_node.event(ctx, event, child_tree_node, env);
        }

        // Propagate the event to the wedge
        let mut wegde_expanded = self.expanded;
        self.wedge.event(ctx, event, &mut wegde_expanded, env);

        // Handle possible creation of new children nodes
        if let Event::MouseUp(_) = event {
            if wegde_expanded != self.expanded {
                // The wedge widget has decided to change the expanded/collapsed state of the node,
                // handle it by expanding/collapsing children nodes as required.
                ctx.request_layout();
                self.expanded = wegde_expanded;
                if self.expand(data, wegde_expanded) {
                    // New children were created, inform the context.
                    ctx.children_changed();
                }
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.wedge.lifecycle(ctx, event, &self.expanded, env);
        self.widget.lifecycle(ctx, event, data, env);
        for (index, child_widget_node) in self.children.iter_mut() {
            let child_tree_node = data.get_child(*index);
            child_widget_node.lifecycle(ctx, event, child_tree_node, env);
        }
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &T, _data: &T, _env: &Env) {}

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let basic_size = env.get(theme::BASIC_WIDGET_HEIGHT);
        let indent = env.get(theme::BASIC_WIDGET_HEIGHT); // For a lack of a better definition
        let mut min_width = bc.min().width;
        let mut max_width = bc.max().width;

        // Top left, the wedge
        self.wedge.layout(
            ctx,
            &BoxConstraints::tight(Size::new(basic_size, basic_size)),
            &self.expanded,
            env,
        );
        self.wedge
            .set_origin(ctx, &self.expanded, env, Point::ORIGIN);

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
            for (index, child_widget_node) in self.children.iter_mut() {
                // In case we have lazily instanciated children nodes,
                // we may skip some indices. This catches up the correct height.
                if *index != next_index {
                    size.height += (*index - next_index) as f64 * basic_size;
                }
                next_index = *index + 1;

                // Layout and position a child node
                let child_tree_node = data.get_child(*index);
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
        if data.children_count() > 0 {
            // we paint the wedge only if there are children to expand
            self.wedge.paint(ctx, &self.expanded, env);
        }
        self.widget.paint(ctx, data, env);
        if self.expanded {
            for (index, child_widget_node) in self.children.iter_mut() {
                let child_tree_node = data.get_child(*index);
                child_widget_node.paint(ctx, child_tree_node, env);
            }
        }
    }
}

/// Tree Implementation
impl<T: TreeNode + Data + Default> Tree<T> {
    /// Create a new Tree widget
    pub fn new<W: Widget<T> + 'static>(make_widget: impl Fn(&T) -> W + 'static) -> Self {
        let boxed_closure: WidgetFactoryCallback<T> =
            Arc::new(Box::new(move |n: &T| Box::new(make_widget(n))));
        Tree {
            root_node: TreeNodeWidget::default(boxed_closure),
        }
    }
}

/// Default tree implementation, supplying Label if the nodes implement the Display trait
impl<T: TreeNode + Data + Default + Display> Default for Tree<T> {
    fn default() -> Self {
        let boxed_closure: WidgetFactoryCallback<T> = Arc::new(Box::new(move |n: &T| {
            Box::new(Label::new(format!("{}", n)))
        }));
        Tree {
            root_node: TreeNodeWidget::default(boxed_closure),
        }
    }
}

// Implement the Widget trait for Tree
impl<T: TreeNode + Data + Default> Widget<T> for Tree<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        self.root_node.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            self.root_node.make_widget(data);
            // Always expand the first level
            if self.root_node.expand(data, true) {
                ctx.children_changed();
            }
        }
        self.root_node.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        self.root_node.update(ctx, old_data, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        bc.constrain(self.root_node.layout(ctx, bc, data, env))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        let background_color = env.get(theme::BACKGROUND_LIGHT);
        let clip_rect = ctx.size().to_rect();
        ctx.fill(clip_rect, &background_color);
        self.root_node.paint(ctx, data, env);
    }
}
