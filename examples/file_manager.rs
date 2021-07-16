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
use std::ffi::OsStr;
use std::fmt::Display;
use std::path::Path;
use std::sync::Arc;

use druid::im::Vector;
use druid::kurbo::Size;
use druid::widget::{Button, Flex, Label, Scroll, TextBox};
use druid::{
    AppLauncher, ArcStr, BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle,
    LifeCycleCtx, LocalizedString, Menu, MenuItem, PaintCtx, Point, Target, UpdateCtx, Widget,
    WidgetExt, WidgetId, WidgetPod, WindowDesc,
};
use druid_widget_nursery::tree::{
    Tree, TreeNode, TREE_CHILD_CREATED, TREE_CHILD_SHOW, TREE_CHROOT, TREE_NODE_REMOVE,
    TREE_NOTIFY_PARENT, TREE_OPEN,
};

use druid_widget_nursery::selectors;

selectors! {
    #[doc = "Set the focus to current textbox"]
    FOCUS_EDIT_BOX,
    NEW_FILE,
    NEW_DIR,
    RENAME,
    DELETE,
    EDIT_FINISHED,
    EDIT_STARTED,
    CHROOT,
    UPDATE_DIR_VIEW,
}

#[derive(Clone, Debug, PartialEq, Data)]
enum FSNodeType {
    File,
    Directory,
}

#[derive(Clone, Debug, PartialEq, Data)]
enum FileType {
    Unknown,
    Rust,
    Toml,
    Python,
}

impl Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use FileType::*;
        match self {
            Unknown => write!(f, "📃"),
            Rust => write!(f, "🦀"),
            Toml => write!(f, "⚙️"),
            Python => write!(f, "🐍"),
        }
    }
}

#[derive(Clone, Lens, Debug, Data)]
struct FSNode {
    name: ArcStr,
    editing: bool,
    // #[data(ignore)]
    children: Vector<Arc<FSNode>>,
    node_type: FSNodeType,
    filetype: FileType,
    open_: bool,
    chroot_: Option<usize>,
}

/// We use FSNode as a tree node, implementing the TreeNode trait.
impl FSNode {
    fn new(name: &'static str) -> Self {
        FSNode {
            name: ArcStr::from(name),
            editing: false,
            children: Vector::new(),
            node_type: FSNodeType::File,
            filetype: FileType::Unknown,
            open_: false,
            chroot_: None,
        }
    }

    fn new_dir(name: &'static str) -> Self {
        FSNode {
            name: ArcStr::from(name),
            editing: false,
            children: Vector::new(),
            node_type: FSNodeType::Directory,
            filetype: FileType::Unknown,
            open_: false,
            chroot_: None,
        }
    }

    fn sort(&mut self) {
        self.children
            .sort_by(|a, b| match (&a.node_type, &b.node_type) {
                // sort directory first, then by name
                (FSNodeType::File, FSNodeType::Directory) => Ordering::Greater,
                (FSNodeType::Directory, FSNodeType::File) => Ordering::Less,
                _ => match (a.name.as_ref(), b.name.as_ref()) {
                    (_, "") => Ordering::Less,
                    ("", _) => Ordering::Greater,
                    _ => a.name.cmp(&b.name),
                },
            });
    }

    fn update(&mut self) {
        self.sort();
    }

    fn add_child(mut self, child: Self) -> Self {
        self.children.push_back(Arc::new(child));
        self.update();
        self
    }

    fn ref_add_child(&mut self, child: Self) {
        self.children.push_back(Arc::new(child));
    }

    // fn get_child_mut(&mut self, index: usize) -> &mut FSNode {
    //     &mut self.children[index]
    // }

    fn get_filetype(&mut self) {
        use FileType::*;
        self.filetype = {
            let fname = self.name.to_string();
            let ext = Path::new(&fname).extension().and_then(OsStr::to_str);
            match ext {
                None => Unknown,
                Some(ext) => match ext {
                    "rs" => Rust,
                    "py" => Python,
                    "toml" => Toml,
                    _ => Unknown,
                },
            }
        };
        eprintln!("{:?}", self.filetype);
    }
}

impl TreeNode for FSNode {
    fn children_count(&self) -> usize {
        self.children.len()
    }

    fn get_child(&self, index: usize) -> &FSNode {
        &self.children[index]
    }

    fn for_child_mut(&mut self, index: usize, mut cb: impl FnMut(&mut Self, usize)) {
        let orig = &self.children[index];
        let mut new = orig.as_ref().clone();
        cb(&mut new, index);
        if !orig.as_ref().same(&new) {
            self.children.remove(index);
            self.children.insert(index, Arc::new(new));
        }
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

    fn open(&mut self, state: bool) {
        self.open_ = state;
    }

    fn is_open(&self) -> bool {
        self.open_
    }

    fn chroot(&mut self, idx: Option<usize>) {
        self.chroot_ = idx;
    }

    fn get_chroot(&self) -> Option<usize> {
        self.chroot_
    }
}

struct FSOpener {
    label: WidgetPod<String, Label<String>>,
    filetype: FileType,
}

impl FSOpener {
    fn label(&self, data: &FSNode) -> String {
        if data.is_branch() {
            if data.is_open() { "📂" } else { "📁" }.to_owned()
        } else {
            format!("{}", self.filetype)
        }
    }
}

impl Widget<FSNode> for FSOpener {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut FSNode, _env: &Env) {}

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &FSNode, env: &Env) {
        let label = self.label(data);
        self.label.lifecycle(ctx, event, &label, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &FSNode, data: &FSNode, env: &Env) {
        if old_data.is_open() != data.is_open() {
            let label = self.label(data);
            self.label.update(ctx, &label, env);
        }
        if !data.is_branch() {
            if data.filetype != self.filetype {
                self.filetype = data.filetype.clone();
                self.label.update(ctx, &self.label(data), env);
            }
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &FSNode,
        env: &Env,
    ) -> Size {
        let label = self.label(data);
        self.label.set_origin(ctx, &label, env, Point::ORIGIN);
        self.label.layout(ctx, bc, &label, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &FSNode, env: &Env) {
        let label = self.label(data);
        self.label.paint(ctx, &label, env)
    }
}

fn make_dir_context_menu(widget_id: WidgetId) -> Menu<FSNode> {
    Menu::empty()
        .entry(MenuItem::new(LocalizedString::new("New File")).on_activate(
            move |ctx, _data: &mut FSNode, _env| {
                ctx.submit_command(NEW_FILE.to(Target::Widget(widget_id)));
            },
        ))
        .entry(
            MenuItem::new(LocalizedString::new("New Sub Directory")).on_activate(
                move |ctx, _data: &mut FSNode, _env| {
                    ctx.submit_command(NEW_DIR.to(Target::Widget(widget_id)));
                },
            ),
        )
        .entry(MenuItem::new(LocalizedString::new("Delete")).on_activate(
            move |ctx, _data: &mut FSNode, _env| {
                ctx.submit_command(DELETE.to(Target::Widget(widget_id)));
            },
        ))
        .entry(MenuItem::new(LocalizedString::new("Rename")).on_activate(
            move |ctx, _data: &mut FSNode, _env| {
                ctx.submit_command(RENAME.to(Target::Widget(widget_id)));
            },
        ))
        .entry(MenuItem::new(LocalizedString::new("Chroot")).on_activate(
            move |ctx, _data: &mut FSNode, _env| {
                ctx.submit_command(CHROOT.to(Target::Widget(widget_id)));
            },
        ))
}

fn make_file_context_menu(widget_id: WidgetId) -> Menu<FSNode> {
    Menu::empty()
        .entry(MenuItem::new(LocalizedString::new("Delete")).on_activate(
            move |ctx, _data: &mut FSNode, _env| {
                ctx.submit_command(DELETE.to(Target::Widget(widget_id)));
            },
        ))
        .entry(MenuItem::new(LocalizedString::new("Rename")).on_activate(
            move |ctx, _data: &mut FSNode, _env| {
                ctx.submit_command(RENAME.to(Target::Widget(widget_id)));
            },
        ))
}

pub struct FSNodeWidget {
    edit_widget_id: WidgetId,
    edit_branch: WidgetPod<FSNode, Flex<FSNode>>,
    normal_branch: WidgetPod<FSNode, Flex<FSNode>>,
    editing: bool,
}

impl FSNodeWidget {
    ///
    pub fn new() -> FSNodeWidget {
        let edit_widget = TextBox::new()
            .with_placeholder("new item")
            .with_id(WidgetId::next());
        FSNodeWidget {
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
                            // data.get_filetype();
                        }),
                    ),
            ),
            normal_branch: WidgetPod::new(
                Flex::row()
                    // First, there's the Label
                    .with_child(Label::dynamic(|data: &FSNode, _env| {
                        String::from(data.name.as_ref())
                    })),
            ),
            editing: false,
        }
    }
}

impl Widget<FSNode> for FSNodeWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut FSNode, env: &Env) {
        let new_event = match event {
            Event::MouseDown(ref mouse) if mouse.button.is_right() => {
                eprintln!("mousedown... {:?}", ctx.widget_id());
                if !self.editing {
                    if data.is_branch() {
                        ctx.show_context_menu(make_dir_context_menu(ctx.widget_id()), mouse.pos);
                    } else {
                        ctx.show_context_menu(make_file_context_menu(ctx.widget_id()), mouse.pos);
                    }
                    None
                } else {
                    Some(event)
                }
            }
            Event::Command(cmd) if cmd.is(EDIT_FINISHED) => {
                eprintln!("############## {:?} ##############", cmd);
                data.get_filetype();
                ctx.submit_notification(TREE_NOTIFY_PARENT.with(UPDATE_DIR_VIEW));
                None
            }
            Event::Command(cmd) if cmd.is(TREE_CHILD_SHOW) => {
                eprintln!("-------- show -------- {:?}", ctx.widget_id());
                data.get_filetype();
                if self.editing {
                    ctx.set_focus(self.edit_widget_id);
                }
                None
            }
            Event::Command(cmd) if cmd.is(NEW_FILE) => {
                eprintln!("-------- new file -------- {:?}", ctx.widget_id());
                data.ref_add_child({
                    let mut child = FSNode::new("");
                    child.editing = true;
                    child
                });
                ctx.submit_notification(TREE_CHILD_CREATED);
                ctx.submit_notification(TREE_OPEN);
                None
            }
            Event::Command(cmd) if cmd.is(NEW_DIR) => {
                eprintln!("-------- new dir -------- {:?}", ctx.widget_id());
                data.ref_add_child({
                    let mut child = FSNode::new_dir("");
                    child.editing = true;
                    child
                });
                ctx.submit_notification(TREE_CHILD_CREATED);
                ctx.submit_notification(TREE_OPEN);
                None
            }
            Event::Command(cmd) if cmd.is(DELETE) => {
                eprintln!("-------- delete -------- {:?}", ctx.widget_id());
                ctx.submit_notification(TREE_NODE_REMOVE);
                None
            }
            Event::Command(cmd) if cmd.is(RENAME) => {
                eprintln!("-------- delete -------- {:?}", ctx.widget_id());
                data.editing = true;
                ctx.set_focus(self.edit_widget_id);
                None
            }
            Event::Command(cmd) if cmd.is(CHROOT) => {
                ctx.submit_notification(TREE_CHROOT.with(vec![]));
                None
            }
            Event::Command(cmd) if cmd.is(TREE_NOTIFY_PARENT) => {
                let cmd_data = cmd.get(TREE_NOTIFY_PARENT).unwrap();
                if *cmd_data == UPDATE_DIR_VIEW {
                    data.update();
                    ctx.children_changed();
                    ctx.set_handled();
                    None
                } else {
                    Some(event)
                }
            }
            _ => Some(event),
        };
        if let Some(evt) = new_event {
            if evt.should_propagate_to_hidden() {
                self.edit_branch.event(ctx, evt, data, env);
                self.normal_branch.event(ctx, evt, data, env);
            } else {
                self.current_widget().event(ctx, evt, data, env)
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
            if self.editing {
                ctx.submit_command(EDIT_FINISHED.to(ctx.widget_id()));
            } else {
                ctx.submit_command(EDIT_STARTED);
            }
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
    })
    .with_opener(|| FSOpener {
        label: WidgetPod::new(Label::dynamic(|st: &String, _| st.clone())),
        filetype: FileType::Unknown,
    });
    Scroll::new(tree).debug_widget_id()
}

pub fn main() {
    // Create the main window
    let main_window = WindowDesc::new(ui_builder())
        .window_size((600.0, 600.0))
        .title(LocalizedString::new("tree-demo-window-title").with_placeholder("Tree Demo"));

    // Set our initial data.
    // This is an extract from https://en.wikipedia.org/wiki/Linnaean_taxonomy
    let taxonomy = FSNode::new_dir("project")
        .add_child(
            FSNode::new_dir("src")
                .add_child(FSNode::new("lib.rs"))
                .add_child(FSNode::new("ui.rs"))
                .add_child(FSNode::new_dir("backend").add_child(FSNode::new("mod.rs"))),
        )
        .add_child(
            FSNode::new_dir("examples")
                .add_child(FSNode::new("do_stuff.rs"))
                .add_child(FSNode::new("do_other_stuff.rs")),
        )
        .add_child(FSNode::new("Cargo.toml"))
        .add_child(FSNode::new("Cargo.lock"))
        .add_child(
            FSNode::new_dir(".git")
                .add_child(FSNode::new("config"))
                .add_child(FSNode::new("HEAD"))
                .add_child(FSNode::new("index")),
        );

    // start the application
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(taxonomy)
        .expect("launch failed");
}
