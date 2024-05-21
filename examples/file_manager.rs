// Copyright 2019 the Druid Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Demos advanced tree widget and tree manipulations.

// This is a pseudo tree file manager (no interaction with your actual
// filesystem whatsoever). It's intended to use most of the features of
// the `Tree` widget in a familiar context. It's by no mean polished, and
// probably lacks a lot of features, we want to focus on the tree widget here.
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
    ChrootStatus, Tree, TreeNode, TREE_ACTIVATE_NODE, TREE_CHILD_SHOW, TREE_CHROOT, TREE_CHROOT_UP,
    TREE_NODE_REMOVE, TREE_NOTIFY_CHROOT, TREE_NOTIFY_PARENT, TREE_OPEN,
};

use druid_widget_nursery::selectors;

selectors! {
    /// Set the focus to current textbox
    FOCUS_EDIT_BOX,
    /// Command to tell a directory to create a new file
    NEW_FILE,
    /// Command to tell a directory to create a new subdir
    NEW_DIR,
    /// Start a rename
    RENAME,
    /// Delete the node
    DELETE,
    /// Tell that the edition of a node name name (on creation/rename) is now completed
    EDIT_FINISHED,
    /// Tell that the edition of a node name name (on creation/rename) has just started
    EDIT_STARTED,
    /// Command sent by the context menu to chroot to the targeted directory
    CHROOT,

    /// Internal wiring, mostly to update the filetype and the sorting
    UPDATE_DIR_VIEW,
    UPDATE_FILE,
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
    /// Name of the node, that is diplayed in the tree
    name: ArcStr,
    /// Wether the user is currently editing the node name
    editing: bool,
    /// Children FSNodes. We wrap them in an Arc to avoid a ugly side effect of Vector (discussed in examples/tree.rs)
    children: Vector<Arc<FSNode>>,
    /// Explicit storage of the type (file or directory)
    node_type: FSNodeType,
    /// File type to display cute animals next to the files
    filetype: FileType,
    /// Keep track of the expanded state
    expanded: bool,
    /// Keep track of the chroot state (see TreeNode::get_chroot for description of the chroot mechanism)
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
            expanded: false,
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
            expanded: false,
            chroot_: None,
        }
    }

    /// The sorting is directories first and alphanumeric order.
    /// This is called upon insertion or update of a child, by the
    /// FSNodeWidget.
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
        // TODO: we should update the virtual root here, if its index has changed.
        //       Or... maybe... the whole chroot system is to be redesigned, even
        //       in the Tree widget :/
    }

    fn add_child(mut self, child: Self) -> Self {
        self.children.push_back(Arc::new(child));
        self.update();
        self
    }

    fn ref_add_child(&mut self, child: Self) {
        self.children.push_back(Arc::new(child));
        self.update();
    }

    fn get_filetype(&mut self) {
        // A quick and dirty filetype detection to add eye-candy to the demo.
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
        // Apply the closure to a clone of the child and update the `self.children` vector
        // with the clone iff it's changed to avoid unnecessary calls to `update(...)`

        // TODO: there must be a more idiomatic way to do this
        let orig = &self.children[index];
        let mut new = orig.as_ref().clone();
        cb(&mut new, index);
        if !orig.as_ref().same(&new) {
            self.children.remove(index);
            self.children.insert(index, Arc::new(new));
        }
    }

    fn is_branch(&self) -> bool {
        // The default implementation would consider empty dirs as files.
        matches!(self.node_type, FSNodeType::Directory)
    }

    fn rm_child(&mut self, index: usize) {
        self.children.remove(index);
    }

    // those two accessors are the most simple implementation to enable chroot, and should
    // be enough for most use cases.
    fn chroot(&mut self, idx: Option<usize>) {
        self.chroot_ = idx;
    }

    fn get_chroot(&self) -> Option<usize> {
        self.chroot_
    }
}

/// FSOpener is the opener widget, the small icon the user interacts with to
/// expand directories.
struct FSOpener {
    label: WidgetPod<String, Label<String>>,
    filetype: FileType,
    chroot_status: ChrootStatus,
}

impl FSOpener {
    fn label(&self, data: &FSNode) -> String {
        if data.is_branch() {
            match self.chroot_status {
                ChrootStatus::NO | ChrootStatus::ROOT => {
                    // this is either the actual root or not the virtual root. We
                    // show a directory emoji based on the expand state
                    if data.expanded {
                        "📂"
                    } else {
                        "📁"
                    }
                }
                // for the chroot we show that the user can move the virtual root up a dir
                ChrootStatus::YES => "↖️",
            }
            .to_owned()
        } else {
            format!("{}", self.filetype)
        }
    }
}

impl Widget<FSNode> for FSOpener {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut FSNode, _env: &Env) {
        if data.is_branch() {
            match event {
                // The wrapping tree::Opener widget transforms a click to this command.
                Event::Command(cmd) if cmd.is(TREE_ACTIVATE_NODE) => {
                    // We care only for branches (we could of course imagine interactions with files too)
                    if data.is_branch() {
                        match self.chroot_status {
                            // not on chroot ? expand
                            ChrootStatus::NO | ChrootStatus::ROOT => data.expanded = !data.expanded,
                            // on chroot ? chroot up
                            ChrootStatus::YES => ctx.submit_notification(TREE_CHROOT_UP),
                        }
                    }
                }
                // The Tree widget sends this command when the node's virtual root status change.
                // This is because the data of a virtual root is not enough to tell. We keep the
                // info on the widget at the moment.
                Event::Command(cmd) if cmd.is(TREE_NOTIFY_CHROOT) => {
                    let new_status = cmd.get(TREE_NOTIFY_CHROOT).unwrap().clone();
                    if self.chroot_status != new_status {
                        self.chroot_status = new_status;
                        if let ChrootStatus::YES = self.chroot_status {
                            data.expanded = true;
                        }
                        ctx.children_changed();
                        ctx.request_update();
                    }
                }
                _ => (),
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &FSNode, env: &Env) {
        let label = self.label(data);
        self.label.lifecycle(ctx, event, &label, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &FSNode, data: &FSNode, env: &Env) {
        if old_data.expanded != data.expanded {
            let label = self.label(data);
            self.label.update(ctx, &label, env);
        }
        if !data.is_branch() {
            if data.filetype != self.filetype {
                self.filetype = data.filetype.clone();
                self.label.update(ctx, &self.label(data), env);
            }
        } else {
            self.label.update(ctx, &self.label(data), env);
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
        let size = self.label.layout(ctx, bc, &label, env);
        self.label.set_origin(ctx, Point::ORIGIN);
        size
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

/// THis is the user widget we pass to the Tree constructor, to display `FSNode`s
/// It is a variation of `druid::widget::Either` that displays a Label or a TextBox
/// according to `editing`.
pub struct FSNodeWidget {
    edit_widget_id: WidgetId,
    edit_branch: WidgetPod<FSNode, Flex<FSNode>>,
    normal_branch: WidgetPod<FSNode, Flex<FSNode>>,
    editing: bool,
    file_type: Option<FileType>,
}

impl FSNodeWidget {
    #[allow(clippy::new_without_default)]
    pub fn new() -> FSNodeWidget {
        let edit_widget = TextBox::new()
            .with_placeholder("new item")
            .with_id(WidgetId::next());
        FSNodeWidget {
            edit_widget_id: edit_widget.id().unwrap(),
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
            normal_branch: WidgetPod::new(Flex::row().with_default_spacer().with_child(
                Label::dynamic(|data: &FSNode, _env| String::from(data.name.as_ref())),
            )),
            editing: false,
            file_type: None,
        }
    }
}

impl Widget<FSNode> for FSNodeWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut FSNode, env: &Env) {
        // Well, a lot of stuff is going on here, but not directly related to the tree widget. It's
        // mostly plubming for the FSNodeWidget interactions... The exercise is left to the reader.
        // (i.e. I'm tired documenting, I'm not even sure how much this may change in a near future.)
        let new_event = match event {
            Event::MouseDown(ref mouse) if mouse.button.is_right() => {
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
            // Tell that the edition of a node name name (on creation/rename) is now completed
            Event::Command(cmd) if cmd.is(EDIT_FINISHED) => {
                ctx.submit_command(UPDATE_FILE.to(ctx.widget_id()));
                None
            }
            Event::Command(cmd) if cmd.is(UPDATE_FILE) => {
                data.get_filetype();
                self.file_type = Some(data.filetype.clone());
                ctx.submit_notification(TREE_NOTIFY_PARENT.with(UPDATE_DIR_VIEW));
                None
            }
            Event::Command(cmd) if cmd.is(TREE_CHILD_SHOW) => {
                if self.file_type.is_none() {
                    data.get_filetype();
                    self.file_type = Some(data.filetype.clone());
                }
                if self.editing {
                    ctx.set_focus(self.edit_widget_id);
                }
                None
            }
            Event::Command(cmd) if cmd.is(NEW_FILE) => {
                data.ref_add_child({
                    let mut child = FSNode::new("");
                    child.editing = true;
                    child
                });
                ctx.submit_notification(TREE_OPEN);
                None
            }
            Event::Command(cmd) if cmd.is(NEW_DIR) => {
                data.ref_add_child({
                    let mut child = FSNode::new_dir("");
                    child.editing = true;
                    child
                });
                ctx.submit_notification(TREE_OPEN);
                None
            }
            Event::Command(cmd) if cmd.is(DELETE) => {
                ctx.submit_notification(TREE_NODE_REMOVE);
                None
            }
            Event::Command(cmd) if cmd.is(RENAME) => {
                data.editing = true;
                ctx.set_focus(self.edit_widget_id);
                None
            }
            Event::Command(cmd) if cmd.is(CHROOT) => {
                ctx.submit_notification(TREE_CHROOT);
                None
            }
            Event::Command(cmd) if cmd.is(TREE_NOTIFY_PARENT) => {
                let cmd_data = cmd.get(TREE_NOTIFY_PARENT).unwrap();
                if *cmd_data == UPDATE_DIR_VIEW {
                    data.update();
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
                // Tell that the edition of a node name name (on creation/rename) is now completed
                ctx.submit_command(EDIT_FINISHED.to(ctx.widget_id()));
            } else {
                ctx.submit_command(EDIT_STARTED);
            }
            self.editing = data.editing;
        } else if !self.editing & (_old_data.name != data.name) {
            ctx.submit_command(UPDATE_FILE.to(ctx.widget_id()));
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
        current_widget.set_origin(ctx, Point::ORIGIN);
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
    let tree = Tree::new(
        || {
            // Our items are editable. If editing is true, we show a TextBox of the name,
            // otherwise it's a Label
            FSNodeWidget::new()
        },
        FSNode::expanded,
    )
    .with_opener(|| FSOpener {
        label: WidgetPod::new(Label::dynamic(|st: &String, _| st.clone())),
        filetype: FileType::Unknown,
        chroot_status: ChrootStatus::NO,
    });
    Scroll::new(tree)
    //.debug_widget_id()
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
