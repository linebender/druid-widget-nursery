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
use std::fmt;

use druid::im::Vector;
use druid::widget::{Button, Either, Flex, Label, Scroll, TextBox};
use druid::{AppLauncher, Data, Lens, LocalizedString, Widget, WidgetExt, WindowDesc};
use druid_widget_nursery::{Tree, TreeNode, TREE_CHILD_REMOVE, TREE_OPEN_PARENT};

#[derive(Clone, Lens, Debug, Data)]
struct Taxonomy {
    name: String,
    editing: bool,
    children: Vector<Taxonomy>,
}

/// We use Taxonomy as a tree node, implementing the TreeNode trait.
impl Taxonomy {
    fn new(name: &'static str) -> Self {
        Taxonomy {
            name: name.to_string(),
            editing: false,
            children: Vector::new(),
        }
    }

    fn add_child(mut self, child: Self) -> Self {
        self.children.push_back(child);
        self
    }

    fn ref_add_child(&mut self, child: Self) {
        self.children.push_back(child);
    }
}

impl TreeNode for Taxonomy {
    fn children_count(&self) -> usize {
        self.children.len()
    }

    fn get_child(&self, index: usize) -> &Taxonomy {
        &self.children[index]
    }

    fn get_child_mut(&mut self, index: usize) -> &mut Taxonomy {
        &mut self.children[index]
    }

    fn rm_child(&mut self, index: usize) {
        self.children.remove(index);
    }
}

impl fmt::Display for Taxonomy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

pub fn main() {
    // Create the main window
    let main_window = WindowDesc::new(ui_builder())
        .title(LocalizedString::new("tree-demo-window-title").with_placeholder("Tree Demo"));

    // Set our initial data.
    // This is an extract from https://en.wikipedia.org/wiki/Linnaean_taxonomy
    let taxonomy = Taxonomy::new("Life")
        .add_child(
            Taxonomy::new("Animalia")
                .add_child(
                    Taxonomy::new("Aves")
                        .add_child(Taxonomy::new("Accipitres"))
                        .add_child(Taxonomy::new("Picae"))
                        .add_child(Taxonomy::new("Passeres")),
                )
                .add_child(
                    Taxonomy::new("Amphibia")
                        .add_child(Taxonomy::new("Reptiles"))
                        .add_child(Taxonomy::new("Serpentes"))
                        .add_child(Taxonomy::new("Nantes")),
                )
                .add_child(Taxonomy::new("Pisces"))
                .add_child(Taxonomy::new("Insecta")),
        )
        .add_child(
            Taxonomy::new("Vegetalia")
                .add_child(Taxonomy::new("Monandria"))
                .add_child(Taxonomy::new("Diandria"))
                .add_child(Taxonomy::new("Heptandria")),
        )
        .add_child(
            Taxonomy::new("Mineralia")
                .add_child(Taxonomy::new("Petræ"))
                .add_child(Taxonomy::new("Fossilia"))
                .add_child(Taxonomy::new("Vitamentra")),
        );

    // start the application
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(taxonomy)
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<Taxonomy> {
    Scroll::new(
        // Tree takes a closure to build the tree items widgets
        Tree::new(|| {
            // Our items are editable. If editing is true, we show a TextBox of the name,
            // otherwise it's a Label
            Either::new(
                |data, _env| (*data).editing,
                Flex::row()
                    .with_child(
                        TextBox::new()
                            .with_placeholder("new item")
                            .lens(Taxonomy::name),
                    )
                    .with_child(
                        Button::new("Save").on_click(|_ctx, data: &mut Taxonomy, _env| {
                            data.editing = false;
                        }),
                    ),
                Flex::row()
                    // First, there's the Label
                    .with_child(Label::dynamic(|data: &Taxonomy, _env| data.name.clone()))
                    // The "add child" button
                    .with_child(Button::new("+").on_click(|ctx, data: &mut Taxonomy, _env| {
                        data.ref_add_child({
                            let mut child = Taxonomy::new("");
                            child.editing = true;
                            child
                        });
                        // The Tree widget must be notified about the change
                        ctx.submit_notification(TREE_OPEN_PARENT);
                    }))
                    // The "delete node" button
                    .with_child(
                        Button::new("Edit").on_click(|_ctx, data: &mut Taxonomy, _env| {
                            data.editing = true;
                        }),
                    )
                    .with_child(
                        Button::new("-").on_click(|ctx, _data: &mut Taxonomy, _env| {
                            // Tell the parent to remove the item. The parent handles this notification by
                            // 1. remove the child widget
                            // 2. call TreeNode::rm_child from its data (the parent Taxonomy node, here)
                            ctx.submit_notification(TREE_CHILD_REMOVE);
                        }),
                    ),
            )
        }),
    )
}
