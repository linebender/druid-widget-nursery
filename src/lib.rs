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
mod canvas;
mod computed;
mod dropdown;
mod dropdown_select;
mod dynamic_sized_box;
pub mod enum_switcher;
mod list_select;
#[macro_use]
mod macros;
mod multi_value;
pub mod navigator;
mod on_monitor;
pub mod prism;
mod progress_bar;
mod separator;
pub mod splits;
pub mod theme_loader;
mod tooltip;
mod tree;

#[cfg(feature = "async")]
mod future_widget;

#[cfg(feature = "hot-reload")]
pub mod hot_reload;
mod list_filter;

pub use canvas::{Canvas, CanvasLayout, CanvasWrap};
pub use computed::ComputedWidget;
pub use dropdown::{Dropdown, DROP};
pub use dropdown_select::DropdownSelect;
pub use dynamic_sized_box::DynamicSizedBox;
pub use list_filter::{FilterIter, ListFilter};
pub use list_select::ListSelect;
pub use multi_value::{MultiCheckbox, MultiRadio};
pub use on_monitor::{OnMonitor, OnMonitorExt};
pub use progress_bar::ProgressBar;
pub use separator::{Orientation, Separator};
pub use tooltip::{TooltipController, TooltipExt};

pub use tree::{Tree, TreeNode, Wedge, TREE_CHILD_CREATED, TREE_CHILD_REMOVE};

#[cfg(feature = "async")]
pub use future_widget::{Delegate as AsyncDelegate, FutureWidget};
