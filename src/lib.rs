// Copyright 2018 The Druid Authors.
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

//! A collection of widgets for the druid GUI framework

#![allow(clippy::new_ret_no_self)]

pub mod animation;
mod autofocus;
mod canvas;
mod computed;
mod configure_env;
mod context_traits;
pub mod dropdown;
mod dropdown_select;
mod dyn_lens;
mod dynamic_sized_box;
pub mod enum_switcher;
mod list_select;
#[macro_use]
mod macros;
mod advanced_slider;
mod mask;
mod multi_value;
pub mod navigator;
mod on_change;
mod on_cmd;
mod on_monitor;
pub mod prism;
mod progress_bar;
mod separator;
pub mod splits;
mod stack;
pub mod table;
pub mod theme_loader;
mod titlebar;
mod tooltip;
pub mod tree;
mod versioned;
pub mod wedge;
mod widget_ext;
pub mod wrap;

#[cfg(feature = "material-icons")]
pub mod material_icons;

#[cfg(feature = "async")]
mod future_widget;

#[cfg(feature = "hot-reload")]
pub mod hot_reload;
mod list_filter;

pub use advanced_slider::AdvancedSlider;
pub use autofocus::AutoFocus;
pub use canvas::{Canvas, CanvasLayout, CanvasWrap};
pub use computed::ComputedWidget;
pub use configure_env::configure_env;
pub use context_traits::{AnyCtx, CommandCtx, CursorCtx, LaidOutCtx, RequestCtx};
pub use dropdown::Dropdown;
pub use dropdown_select::DropdownSelect;
pub use dyn_lens::DynLens;
pub use dynamic_sized_box::DynamicSizedBox;
pub use list_filter::{FilterIter, ListFilter};
pub use list_select::ListSelect;
pub use mask::Mask;
pub use multi_value::{MultiCheckbox, MultiRadio};
pub use on_change::OnChange;
pub use on_cmd::OnCmd;
pub use on_monitor::OnMonitor;
pub use progress_bar::ProgressBar;
pub use separator::{Orientation, Separator};
pub use stack::{Stack, StackChildParams, StackChildPosition};
pub use titlebar::TitleBar;
pub use tooltip::TooltipController;
pub use tree::{Tree, TreeNode, TREE_NODE_REMOVE};
pub use versioned::Versioned;
pub use wedge::Wedge;
pub use widget_ext::WidgetExt;

#[cfg(feature = "async")]
pub use future_widget::FutureWidget;
