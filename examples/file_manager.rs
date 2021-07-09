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
use std::cmp::Ordering;
use std::marker::PhantomData;

use druid::kurbo::Size;
use druid::widget::{Button, Controller, ControllerHost, Either, Flex, Label, Scroll, TextBox};
use druid::{
    AppLauncher, ArcStr, BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle,
    LifeCycleCtx, LocalizedString, Menu, MenuItem, PaintCtx, Point, Target, UpdateCtx, Widget,
    WidgetExt, WidgetId, WidgetPod, WindowDesc,
};
use druid_widget_nursery::tree::{Tree, TreeNode, TREE_CHILD_REMOVE, TREE_OPEN, TREE_OPEN_PARENT};

use druid_widget_nursery::selectors;

selectors! {
    FOCUS_EDIT_BOX,
    NEW_FILE,
}

#[derive(Clone, Debug)]
enum FSNodeType {
    File,
    Directory,
}

#[derive(Clone, Lens, Debug)]
struct FSNode {
    name: ArcStr,
    editing: bool,
    children: Vec<FSNode>,
    node_type: FSNodeType,
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
            node_type: FSNodeType::File,
        }
    }

    fn sort(&mut self) {
        self.children.sort_by(|a, b| return (&a.name).cmp(&b.name));
    }

    fn new_dir(name: &'static str) -> Self {
        FSNode {
            name: ArcStr::from(name),
            editing: false,
            children: Vec::new(),
            node_type: FSNodeType::Directory,
        }
    }

    fn add_child(mut self, child: Self) -> Self {
        self.children.push(child);
        self.sort();
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

    fn is_branch(&self) -> bool {
        if let FSNodeType::Directory = self.node_type {
            true
        } else {
            false
        }
    }

    fn rm_child(&mut self, index: usize) {
        self.children.remove(index);
    }
}

struct MyOpener<T> {
    label: WidgetPod<String, Label<String>>,
    phantom: PhantomData<T>,
}

impl<T> MyOpener<T> {
    fn label(open: bool) -> String {
        if open { "📂" } else { "📁" }.to_owned()
    }
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
        let label = MyOpener::<T>::label(data.0);
        self.label.lifecycle(ctx, event, &label, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &(bool, T), data: &(bool, T), env: &Env) {
        if old_data.0 != data.0 {
            let label = MyOpener::<T>::label(data.0);
            self.label.update(ctx, &label, env);
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &(bool, T),
        env: &Env,
    ) -> Size {
        let label = MyOpener::<T>::label(data.0);
        self.label.set_origin(ctx, &label, env, Point::ORIGIN);
        self.label.layout(ctx, bc, &label, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &(bool, T), env: &Env) {
        let label = MyOpener::<T>::label(data.0);
        self.label.paint(ctx, &label, env)
    }
}

fn make_context_menu(widget_id: WidgetId) -> Menu<FSNode> {
    Menu::empty().entry(MenuItem::new(LocalizedString::new("New File")).on_activate(
        move |ctx, data: &mut FSNode, _env| {
            ctx.submit_command(NEW_FILE.to(Target::Widget(widget_id)));
            eprintln!("submit New child to {:?}", widget_id);
            // data.ref_add_child({
            //     let mut child = FSNode::new("");
            //     child.editing = true;
            //     child
            // });
            // // The Tree widget must be notified about the change
            // ctx.submit_command(TREE_OPEN_PARENT.to(Target::Widget(widget_id)));
        },
    ))
    // .entry(MenuItem::new(LocalizedString::new("Delete")).on_activate(
    //     |_ctx, data: &mut FSNode, _env| data.menu_count = data.menu_count.saturating_sub(1),
    // ))
    // .entry(
    //     MenuItem::new(LocalizedString::new("Rename"))
    //         .on_activate(|_ctx, data: &mut FSNode, _env| data.glow_hot = !data.glow_hot),
    // )
}

struct TreeItemController;

impl<W: Widget<FSNode>> Controller<FSNode, W> for TreeItemController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut FSNode,
        env: &Env,
    ) {
        let new_event = match event {
            Event::MouseDown(ref mouse) if mouse.button.is_right() => {
                ctx.show_context_menu(make_context_menu(child.id().unwrap()), mouse.pos);
                None
            }
            Event::Command(cmd) if cmd.is(TREE_OPEN_PARENT) => {
                eprintln!("{:?}", cmd);
                if let Target::Widget(id) = cmd.target() {
                    if id == child.id().unwrap() {
                        eprintln!("submit_notification");
                        ctx.submit_notification(TREE_OPEN);
                        ctx.submit_command(FOCUS_EDIT_BOX.to(Target::Widget(child.id().unwrap())));
                        ctx.children_changed();
                        ctx.request_update();
                        ctx.set_handled();
                        None
                    } else {
                        Some(event.clone())
                    }
                } else {
                    Some(event.clone())
                }
            }
            Event::Command(cmd) if cmd.is(FOCUS_EDIT_BOX) => {
                Some(Event::Command(FOCUS_EDIT_BOX.to(Target::Auto)))
            }
            _ => Some(event.clone()),
        };
        if let Some(evt) = new_event {
            child.event(ctx, &evt, data, env)
        }
    }
}

struct EditItemController;

impl<W: Widget<FSNode>> Controller<FSNode, W> for EditItemController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut FSNode,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(FOCUS_EDIT_BOX) => {
                eprintln!("coucou {:?} {}", cmd, data.editing);
                // if data.editing {
                eprintln!("{:?}", data);
                eprintln!("set_focus...");
                ctx.set_focus(child.id().unwrap());
                ctx.request_update();
                // }
            }
            _ => child.event(ctx, event, data, env),
        }
    }

    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &FSNode,
        data: &FSNode,
        env: &Env,
    ) {
        eprintln!("_______________ update _____________");
        if !ctx.has_focus() {
            eprintln!("_______________ no focus _____________");
            ctx.submit_command(FOCUS_EDIT_BOX.to(Target::Widget(child.id().unwrap())));
        }
        child.update(ctx, old_data, data, env)
    }
}

pub struct FSNodeWidget {
    id: WidgetId,
    edit_widget_id: WidgetId,
    edit_branch: WidgetPod<FSNode, Flex<FSNode>>,
    normal_branch: WidgetPod<FSNode, Flex<FSNode>>,
    editing: bool,
}

impl FSNodeWidget {
    /// Create a new widget that switches between two views.
    ///
    /// The given closure is evaluated on data change. If its value is `true`, then
    /// the `true_branch` widget is shown, otherwise `false_branch`.
    pub fn new() -> FSNodeWidget {
        let edit_widget = TextBox::new()
            .with_placeholder("new item")
            .with_id(WidgetId::next());
        FSNodeWidget {
            id: WidgetId::next(),
            edit_widget_id: edit_widget.id().unwrap().clone(),
            edit_branch: WidgetPod::new(
                Flex::row()
                    .with_child(edit_widget.lens(druid::lens::Map::new(
                        |data: &FSNode| String::from(data.name.as_ref()),
                        |data: &mut FSNode, name| data.name = ArcStr::from(name),
                    )))
                    .with_child(
                        Button::new("Save").on_click(|_ctx, data: &mut FSNode, _env| {
                            data.editing = false;
                        }),
                    ),
            ),
            normal_branch: WidgetPod::new(
                Flex::row()
                    // First, there's the Label
                    .with_child(Label::dynamic(|data: &FSNode, _env| {
                        String::from(data.name.as_ref())
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
            ),
            editing: false,
        }
    }
}

impl Widget<FSNode> for FSNodeWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut FSNode, env: &Env) {
        eprintln!("+++++++++++ {:?}", event);
        if self.editing {
            ctx.set_focus(self.edit_widget_id);
        }

        let new_event = match event {
            Event::MouseDown(ref mouse) if mouse.button.is_right() => {
                eprintln!("mousedown... {:?}", self.id);
                if !self.editing {
                    ctx.show_context_menu(make_context_menu(self.id), mouse.pos);
                    None
                } else {
                    Some(event.clone())
                }
            }
            Event::Command(cmd) if cmd.is(NEW_FILE) => {
                eprintln!("-------- new file -------- {:?}", self.id);
                eprintln!("{:?}", cmd);
                data.ref_add_child({
                    let mut child = FSNode::new("");
                    child.editing = true;
                    child
                });
                ctx.children_changed();
                ctx.submit_notification(TREE_OPEN);
                Some(event.clone())
            }
            _ => Some(event.clone()),
        };
        if let Some(evt) = new_event {
            if evt.should_propagate_to_hidden() {
                self.edit_branch.event(ctx, &evt, data, env);
                self.normal_branch.event(ctx, &evt, data, env);
            } else {
                self.current_widget().event(ctx, &evt, data, env)
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &FSNode, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            self.editing = data.editing;
        }

        if event.should_propagate_to_hidden() {
            self.edit_branch.lifecycle(ctx, event, data, env);
            self.normal_branch.lifecycle(ctx, event, data, env);
        } else {
            self.current_widget().lifecycle(ctx, event, data, env)
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &FSNode, data: &FSNode, env: &Env) {
        if data.editing != self.editing {
            self.editing = data.editing;
            ctx.children_changed();
        }
        self.current_widget().update(ctx, data, env)
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &FSNode,
        env: &Env,
    ) -> Size {
        let current_widget = self.current_widget();
        let size = current_widget.layout(ctx, bc, data, env);
        current_widget.set_origin(ctx, data, env, Point::ORIGIN);
        ctx.set_paint_insets(current_widget.paint_insets());
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &FSNode, env: &Env) {
        self.current_widget().paint(ctx, data, env)
    }
}

impl FSNodeWidget {
    fn current_widget(&mut self) -> &mut WidgetPod<FSNode, Flex<FSNode>> {
        if self.editing {
            &mut self.edit_branch
        } else {
            &mut self.normal_branch
        }
    }
}

fn ui_builder() -> impl Widget<FSNode> {
    let tree = Tree::new(|| {
        // Our items are editable. If editing is true, we show a TextBox of the name,
        // otherwise it's a Label
        FSNodeWidget::new()
        // ControllerHost::new(
        //     Either::new(
        //         |data, _env| (*data).editing,
        //         Flex::row()
        //             .with_child(ControllerHost::new(
        //                 TextBox::new()
        //                     .with_placeholder("new item")
        //                     .with_id(WidgetId::next())
        //                     .lens(druid::lens::Map::new(
        //                         |data: &FSNode| String::from(data.name.as_ref()),
        //                         |data: &mut FSNode, name| data.name = ArcStr::from(name),
        //                     )),
        //                 EditItemController,
        //             ))
        //             .with_child(
        //                 Button::new("Save").on_click(|_ctx, data: &mut FSNode, _env| {
        //                     data.editing = false;
        //                 }),
        //             ),
        //         Flex::row()
        //             // First, there's the Label
        //             .with_child(Label::dynamic(|data: &FSNode, _env| {
        //                 String::from(data.name.as_ref())
        //             }))
        //             // The "delete node" button
        //             .with_child(
        //                 Button::new("Edit").on_click(|_ctx, data: &mut FSNode, _env| {
        //                     data.editing = true;
        //                 }),
        //             )
        //             .with_child(Button::new("-").on_click(|ctx, _data: &mut FSNode, _env| {
        //                 // Tell the parent to remove the item. The parent handles this notification by
        //                 // 1. remove the child widget
        //                 // 2. call TreeNode::rm_child from its data (the parent FSNode node, here)
        //                 ctx.submit_notification(TREE_CHILD_REMOVE);
        //             })),
        //     )
        //     .with_id(WidgetId::next()),
        //     TreeItemController,
        // )
    })
    .with_opener(|| MyOpener {
        label: WidgetPod::new(Label::dynamic(|st: &String, _| st.clone())),
        phantom: PhantomData,
    });
    // WidgetPod::new(tree)
    Scroll::new(tree).debug_widget_id()
}

pub fn main() {
    // Create the main window
    let main_window = WindowDesc::new(ui_builder())
        .title(LocalizedString::new("tree-demo-window-title").with_placeholder("Tree Demo"));

    // Set our initial data.
    // This is an extract from https://en.wikipedia.org/wiki/Linnaean_taxonomy
    let taxonomy = FSNode::new_dir("Life")
        .add_child(
            FSNode::new_dir("Animalia")
                .add_child(
                    FSNode::new_dir("Aves")
                        .add_child(FSNode::new("Accipitres"))
                        .add_child(FSNode::new("Picae"))
                        .add_child(FSNode::new("Passeres")),
                )
                .add_child(
                    FSNode::new_dir("Amphibia")
                        .add_child(FSNode::new("Reptiles"))
                        .add_child(FSNode::new("Serpentes"))
                        .add_child(FSNode::new("Nantes")),
                )
                .add_child(FSNode::new_dir("Pisces"))
                .add_child(FSNode::new("Insecta")),
        )
        .add_child(
            FSNode::new_dir("Vegetalia")
                .add_child(FSNode::new("Monandria"))
                .add_child(FSNode::new("Diandria"))
                .add_child(FSNode::new("Heptandria")),
        )
        .add_child(
            FSNode::new_dir("Mineralia")
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
