// Copyright 2019 The Druid Authors.
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

//! Demos basic tree widget and tree manipulations.
use std::marker::PhantomData;

use druid::im::Vector;
use druid::kurbo::Size;
use druid::widget::{Button, Either, Flex, Label, Scroll, TextBox};
use druid::{
    AppLauncher, ArcStr, BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle,
    LifeCycleCtx, LocalizedString, PaintCtx, Point, UpdateCtx, Widget, WidgetExt, WidgetPod,
    WindowDesc,
};
use druid_widget_nursery::{Tree, TreeNode, TREE_CHILD_REMOVE, TREE_OPEN_PARENT};

enum FsNodeType {
    File,
    Directory,
}

#[derive(Clone, Lens, Debug)]
struct FSNode {
    name: ArcStr,
    editing: bool,
    children: Vec<FSNode>,
}

impl Data for FSNode {
    fn same(&self, o: &Self) -> bool {
        self.name == o.name
            && self.editing.same(&o.editing)
            && self.children.len() == o.children.len()
            && self
                .children
                .iter()
                .zip(o.children.iter())
                .all(|(a, b)| a.same(b))
    }
}

/// We use FSNode as a tree node, implementing the TreeNode trait.
impl FSNode {
    fn new(name: &'static str) -> Self {
        FSNode {
            name: ArcStr::from(name),
            editing: false,
            children: Vec::new(),
        }
    }

    fn add_child(mut self, child: Self) -> Self {
        self.children.push(child);
        self
    }

    fn ref_add_child(&mut self, child: Self) {
        self.children.push(child);
    }
}

impl TreeNode for FSNode {
    fn children_count(&self) -> usize {
        self.children.len()
    }

    fn get_child(&self, index: usize) -> &FSNode {
        &self.children[index]
    }

    fn get_child_mut(&mut self, index: usize) -> &mut FSNode {
        &mut self.children[index]
    }

    fn rm_child(&mut self, index: usize) {
        self.children.remove(index);
    }
}

pub fn main() {
    // Create the main window
    let main_window = WindowDesc::new(ui_builder())
        .title(LocalizedString::new("tree-demo-window-title").with_placeholder("Tree Demo"));

    // Set our initial data.
    // This is an extract from https://en.wikipedia.org/wiki/Linnaean_taxonomy
    let taxonomy = FSNode::new("Life")
        .add_child(
            FSNode::new("Animalia")
                .add_child(
                    FSNode::new("Aves")
                        .add_child(FSNode::new("Accipitres"))
                        .add_child(FSNode::new("Picae"))
                        .add_child(FSNode::new("Passeres")),
                )
                .add_child(
                    FSNode::new("Amphibia")
                        .add_child(FSNode::new("Reptiles"))
                        .add_child(FSNode::new("Serpentes"))
                        .add_child(FSNode::new("Nantes")),
                )
                .add_child(FSNode::new("Pisces"))
                .add_child(FSNode::new("Insecta")),
        )
        .add_child(
            FSNode::new("Vegetalia")
                .add_child(FSNode::new("Monandria"))
                .add_child(FSNode::new("Diandria"))
                .add_child(FSNode::new("Heptandria")),
        )
        .add_child(
            FSNode::new("Mineralia")
                .add_child(FSNode::new("Petræ"))
                .add_child(FSNode::new("Fossilia"))
                .add_child(FSNode::new("Vitamentra")),
        );

    // start the application
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(taxonomy)
        .expect("launch failed");
}

struct MyOpener<T> {
    label: WidgetPod<String, Label<String>>,
    phantom: PhantomData<T>,
}

impl<T> Widget<(bool, T)> for MyOpener<T> {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut (bool, T), _env: &Env) {}

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &(bool, T),
        env: &Env,
    ) {
        let label = if data.0 { "V" } else { ">" };
        self.label.lifecycle(ctx, event, &label.to_owned(), env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &(bool, T), data: &(bool, T), env: &Env) {
        if old_data.0 != data.0 {
            let label = if data.0 { "V" } else { ">" };
            self.label.update(ctx, &label.to_owned(), env);
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &(bool, T),
        env: &Env,
    ) -> Size {
        let label = &(if data.0 { "V" } else { ">" }).to_owned();
        self.label.set_origin(ctx, label, env, Point::ORIGIN);
        self.label.layout(ctx, bc, label, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &(bool, T), env: &Env) {
        let label = if data.0 { "V" } else { ">" };
        self.label.paint(ctx, &label.to_owned(), env)
    }
}

fn ui_builder() -> impl Widget<FSNode> {
    let tree = Tree::new(|| {
        // Our items are editable. If editing is true, we show a TextBox of the name,
        // otherwise it's a Label
        Either::new(
            |data, _env| (*data).editing,
            Flex::row()
                .with_child(TextBox::new().with_placeholder("new item").lens(
                    druid::lens::Map::new(
                        |data: &FSNode| String::from(data.name.as_ref()),
                        |data: &mut FSNode, name| data.name = ArcStr::from(name),
                    ),
                ))
                .with_child(
                    Button::new("Save").on_click(|_ctx, data: &mut FSNode, _env| {
                        data.editing = false;
                    }),
                ),
            Flex::row()
                // First, there's the Label
                .with_child(Label::dynamic(|data: &FSNode, _env| {
                    String::from(data.name.as_ref())
                }))
                // The "add child" button
                .with_child(Button::new("+").on_click(|ctx, data: &mut FSNode, _env| {
                    data.ref_add_child({
                        let mut child = FSNode::new("");
                        child.editing = true;
                        child
                    });
                    // The Tree widget must be notified about the change
                    ctx.submit_notification(TREE_OPEN_PARENT);
                }))
                // The "delete node" button
                .with_child(
                    Button::new("Edit").on_click(|_ctx, data: &mut FSNode, _env| {
                        data.editing = true;
                    }),
                )
                .with_child(Button::new("-").on_click(|ctx, _data: &mut FSNode, _env| {
                    // Tell the parent to remove the item. The parent handles this notification by
                    // 1. remove the child widget
                    // 2. call TreeNode::rm_child from its data (the parent FSNode node, here)
                    ctx.submit_notification(TREE_CHILD_REMOVE);
                })),
        )
    })
    .with_opener(|| MyOpener {
        label: WidgetPod::new(Label::dynamic(|st: &String, _| st.clone())),
        phantom: PhantomData,
    });
    Scroll::new(tree)
}
