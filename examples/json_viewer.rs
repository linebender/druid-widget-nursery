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

//! A json viewer using the tree widget
use std::{
    fmt, fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use druid::{
    commands::{OPEN_FILE, QUIT_APP, SHOW_OPEN_PANEL},
    theme,
    widget::{Flex, Label, Maybe},
    AppDelegate, AppLauncher, Color, Command, Data, DelegateCtx, Env, Handled, Lens,
    LocalizedString, Menu, MenuItem, Target, Widget, WidgetExt, WindowDesc, WindowId,
};
use druid_widget_nursery::{Tree, TreeNode};
use qu::ick_use::*;

#[derive(Clone, Lens, Data, Debug)]
struct JsonNode {
    // None for arrays
    key: Option<String>,
    value: JsonValue,
    expanded: bool,
}

#[derive(Clone, Data, Debug)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Arc<Vec<JsonNode>>),
    Object(Arc<Vec<JsonNode>>),
}

impl JsonNode {
    fn new(key: Option<String>, value: serde_json::Value) -> Self {
        JsonNode {
            key,
            value: JsonValue::new(value),
            expanded: false,
        }
    }
}

impl JsonValue {
    fn new(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => JsonValue::Null,
            serde_json::Value::Bool(v) => JsonValue::Bool(v),
            serde_json::Value::Number(v) => JsonValue::Number(v.as_f64().unwrap()),
            serde_json::Value::String(v) => JsonValue::String(v),
            serde_json::Value::Array(v) => JsonValue::Array(Arc::new(
                v.into_iter().map(|val| JsonNode::new(None, val)).collect(),
            )),
            serde_json::Value::Object(v) => JsonValue::Object(Arc::new(
                v.into_iter()
                    .map(|(key, val)| JsonNode::new(Some(key), val))
                    .collect(),
            )),
        }
    }
}

impl TreeNode for JsonNode {
    fn children_count(&self) -> usize {
        match &self.value {
            JsonValue::Array(v) => v.len(),
            JsonValue::Object(v) => v.len(),
            _ => 0,
        }
    }

    fn get_child(&self, index: usize) -> &Self {
        match &self.value {
            JsonValue::Array(v) => &v[index],
            JsonValue::Object(v) => &v[index],
            _ => unreachable!(),
        }
    }

    fn for_child_mut(&mut self, index: usize, mut cb: impl FnMut(&mut Self, usize)) {
        match &mut self.value {
            JsonValue::Array(v) => cb(&mut Arc::make_mut(v)[index], index),
            JsonValue::Object(v) => cb(&mut Arc::make_mut(v)[index], index),
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for JsonNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.key {
            Some(key) => write!(f, "{}: {}", key, self.value),
            None => write!(f, "{}", self.value),
        }
    }
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValue::Null => f.write_str("null"),
            JsonValue::Bool(b) => write!(f, "{}", b),
            JsonValue::Number(num) => write!(f, "{}", num),
            JsonValue::String(s) => f.write_str(s),
            JsonValue::Array(_) => f.write_str("[ ]"),
            JsonValue::Object(_) => f.write_str("{ }"),
        }
    }
}

fn ui_builder() -> impl Widget<JsonNode> {
    Tree::new(
        || {
            Flex::row()
                .with_child(
                    Maybe::or_empty(|| Flex::row().with_child(Label::raw()).with_default_spacer())
                        .lens(JsonNode::key),
                )
                .with_child(
                    Label::new(|value: &JsonValue, _: &_| value.to_string())
                        .env_scope(|env, value| {
                            let color = match value {
                                JsonValue::Null => Color::rgb(0.6, 0.6, 0.6),
                                JsonValue::Bool(_) => Color::rgb(0.8, 0.6, 0.0),
                                JsonValue::Number(_) => Color::rgb(0.5, 0.9, 0.5),
                                JsonValue::String(_) => Color::WHITE,
                                JsonValue::Array(_) => Color::WHITE,
                                JsonValue::Object(_) => Color::WHITE,
                            };
                            env.set(theme::TEXT_COLOR, color);
                        })
                        .lens(JsonNode::value),
                )
        },
        JsonNode::expanded,
    )
    .scroll()
}

fn menu(_: Option<WindowId>, _: &JsonNode, _: &Env) -> Menu<JsonNode> {
    Menu::new(LocalizedString::new("json-viewer.menu")).entry(
        Menu::new(LocalizedString::new("json-viewer.menu.file").with_placeholder("File"))
            .entry(
                MenuItem::new(
                    LocalizedString::new("json-viewer.menu.file.open").with_placeholder("Open"),
                )
                .command(SHOW_OPEN_PANEL.with(Default::default())),
            )
            .separator()
            .entry(
                MenuItem::new(
                    LocalizedString::new("json-viewer.menu.file.quit").with_placeholder("Quit"),
                )
                .command(QUIT_APP),
            ),
    )
}

struct Delegate;

impl AppDelegate<JsonNode> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut JsonNode,
        _env: &Env,
    ) -> Handled {
        if let Some(file) = cmd.get(OPEN_FILE) {
            *data = JsonNode::new(None, load_json(&file.path));
            Handled::Yes
        } else {
            Handled::No
        }
    }
}

#[derive(Debug, Parser)]
struct Opt {
    json_file: Option<PathBuf>,
}

#[qu::ick]
pub fn main(opt: Opt) -> Result {
    // Create the main window
    let main_window = WindowDesc::new(ui_builder())
        .title(LocalizedString::new("json-viewer-window-title").with_placeholder("Json Viewer"))
        .menu(menu);

    let json = match &opt.json_file {
        Some(path) => load_json(path),
        None => serde_json::from_str(
            r#"
            {
                "name": "example json (open a file in the \"File\" menu)",
                "name2": [
                    1,
                    2
                ]
            }
            "#,
        )
        .unwrap(),
    };
    let node = JsonNode::new(None, json);

    // start the application
    AppLauncher::with_window(main_window)
        .delegate(Delegate)
        .log_to_console()
        .launch(node)?;
    Ok(())
}

fn load_json(path: &Path) -> serde_json::Value {
    fs::read_to_string(path)
        .map_err(|e| e.to_string())
        .and_then(|json| serde_json::from_str(&json).map_err(|e| e.to_string()))
        .unwrap_or_else(|e| {
            serde_json::Value::String(format!("error opening file \"{}\": {}", path.display(), e))
        })
}
