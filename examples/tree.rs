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

//! Demos the most basic tree widget. See the file_manager example for a full demo.
use std::fmt;

use druid::im::Vector;
use druid::{AppLauncher, Data, Lens, LocalizedString, Widget, WindowDesc};
use druid_widget_nursery::{Tree, TreeNode};

#[derive(Clone, Lens, Debug)]
struct Taxonomy {
    name: String,
    editing: bool,
    children: Vector<Taxonomy>,
    expanded_: bool,
}

/// We use Taxonomy as a tree node, implementing the TreeNode trait.
impl Taxonomy {
    fn new(name: &'static str) -> Self {
        Taxonomy {
            name: name.to_string(),
            editing: false,
            children: Vector::new(),
            expanded_: false,
        }
    }

    fn add_child(mut self, child: Self) -> Self {
        self.children.push_back(child);
        self
    }
}

impl Data for Taxonomy {
    // If we derive simply derive `Data`, the children Vector is changed at every
    // event pass (as Vector updates its children pointers in its implementation
    // of `iter_mut(), regardless of the actual sameness of the data). We have to explicitly
    // check the sameness of children nodes.
    //
    // The other workaround is to use a `Vector<Arc<Taxonomy>>` at the expense
    // of a more complex `for_child_mut()` implementation (being able to use imutable children  was
    // the main argument in favor of `for_child_mut(&self, index, callback)`vs the former simpler
    // `get_child_mut(&self, index)`. This workaround is implemented in the `file_manager` example.
    fn same(&self, other: &Self) -> bool {
        self.expanded_ == other.expanded_
            && self.name == other.name
            && self.editing == other.editing
            && self.children.len() == other.children.len()
            && self
                .children
                .iter()
                .zip(other.children.iter())
                .all(|(a, b)| a.same(b))
    }
}

impl TreeNode for Taxonomy {
    fn children_count(&self) -> usize {
        self.children.len()
    }

    fn get_child(&self, index: usize) -> &Taxonomy {
        &self.children[index]
    }

    fn for_child_mut(&mut self, index: usize, mut cb: impl FnMut(&mut Self, usize)) {
        cb(&mut self.children[index], index);
    }

    fn expand(&mut self, state: bool) {
        self.expanded_ = state;
    }

    fn is_expanded(&self) -> bool {
        self.expanded_
    }
}

impl fmt::Display for Taxonomy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

fn ui_builder() -> impl Widget<Taxonomy> {
    // Taxonomy implements Display. We can use the default tree.
    Tree::default()
}

pub fn main() {
    // Create the main window
    let main_window = WindowDesc::new(ui_builder())
        .title(LocalizedString::new("tree-demo-window-title").with_placeholder("Tree Demo"));

    // Set our data.
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
