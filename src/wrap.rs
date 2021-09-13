// Copyright 2021 The Druid Authors.
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

use druid::widget::prelude::*;
use druid::widget::Axis;
use druid::{KeyOrValue, Widget, WidgetPod};

pub struct Wrap<T> {
    children: Vec<WidgetPod<T, Box<dyn Widget<T>>>>,
    direction: Axis,
    run_spacing: KeyOrValue<f64>,
    spacing: KeyOrValue<f64>,
    run_alignment: WrapAlignment,
    alignment: WrapAlignment,
    cross_alignment: WrapCrossAlignment,
}

pub enum WrapAlignment {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceEvenly,
    SpaceAround,
}

pub enum WrapCrossAlignment {
    Start,
    End,
    Center,
}

impl<T> Default for Wrap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Wrap<T> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            direction: Axis::Horizontal,
            run_spacing: 0.0.into(),
            spacing: 0.0.into(),
            run_alignment: WrapAlignment::Start,
            cross_alignment: WrapCrossAlignment::Start,
            alignment: WrapAlignment::Start,
        }
    }

    // allow Box<dyn Widget> in add_child
    pub fn add_child(&mut self, child: Box<dyn Widget<T>>) {
        self.children.push(WidgetPod::new(child))
    }

    pub fn with_child(mut self, child: impl Widget<T> + 'static) -> Self {
        self.add_child(Box::new(child));
        self
    }

    /// Set the wrap's direction.
    pub fn set_direction(&mut self, direction: Axis) {
        self.direction = direction;
    }

    /// Builder style method to set the wrap's direction.
    pub fn direction(mut self, direction: Axis) -> Self {
        self.set_direction(direction);
        self
    }

    /// Set the run spacing.
    pub fn set_run_spacing(&mut self, run_spacing: impl Into<KeyOrValue<f64>>) {
        self.run_spacing = run_spacing.into();
    }

    /// Builder style method to set the wrap's run spacing.
    pub fn run_spacing(mut self, run_spacing: impl Into<KeyOrValue<f64>>) -> Self {
        self.run_spacing = run_spacing.into();
        self
    }

    /// Set the wrap's spacing.
    pub fn set_spacing(&mut self, spacing: impl Into<KeyOrValue<f64>>) {
        self.spacing = spacing.into();
    }

    /// Builder style method to set the wrap's spacing.
    pub fn spacing(mut self, spacing: impl Into<KeyOrValue<f64>>) -> Self {
        self.spacing = spacing.into();
        self
    }

    /// Set the wrap's run alignment.
    pub fn set_run_alignment(&mut self, run_alignment: WrapAlignment) {
        self.run_alignment = run_alignment;
    }

    /// buidler style method to set the wrap's run alignment.
    pub fn run_alignment(mut self, run_alignment: WrapAlignment) -> Self {
        self.run_alignment = run_alignment;
        self
    }

    /// Set the wrap's alignment.
    pub fn set_alignment(&mut self, alignment: WrapAlignment) {
        self.alignment = alignment;
    }

    /// Builder style method to set the wrap's alignment.
    pub fn alignment(mut self, alignment: WrapAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Set the wrap's cross alignment.
    pub fn set_cross_alignment(&mut self, cross_alignment: WrapCrossAlignment) {
        self.cross_alignment = cross_alignment;
    }

    /// Builder style method to set the wrap's cross alignment.
    pub fn cross_alignment(mut self, cross_alignment: WrapCrossAlignment) -> Self {
        self.cross_alignment = cross_alignment;
        self
    }
}

impl<T: Data> Widget<T> for Wrap<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        for x in &mut self.children {
            x.event(ctx, event, data, env);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        for x in &mut self.children {
            x.lifecycle(ctx, event, data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        for x in &mut self.children {
            x.update(ctx, data, env);
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        if self.children.is_empty() {
            return bc.min();
        }
        let dir = self.direction;
        let child_bc =
            BoxConstraints::tight(dir.pack(dir.major(bc.max()), f64::INFINITY).into()).loosen();
        let main_axis_limit = dir.major(bc.max());
        let spacing = self.spacing.resolve(env);
        let run_spacing = self.run_spacing.resolve(env);

        let mut main_axis_extent = 0.0;
        let mut cross_axis_extent = 0.0;
        let mut run_main_axis_extent = 0.0;
        let mut run_cross_axis_extent = 0.0;
        let mut child_count = 0;
        let mut run_metrics = Vec::new();
        for child in &mut self.children {
            let child_size = child.layout(ctx, &child_bc, data, env);
            let child_main_axis_extent = dir.major(child_size);
            let child_cross_axis_extent = dir.minor(child_size);
            if child_count > 0
                && run_main_axis_extent + spacing + child_main_axis_extent > main_axis_limit
            {
                main_axis_extent = f64::max(main_axis_extent, run_main_axis_extent);
                cross_axis_extent += run_cross_axis_extent;
                if !run_metrics.is_empty() {
                    cross_axis_extent += run_spacing;
                }
                run_metrics.push((run_main_axis_extent, run_cross_axis_extent, child_count));
                run_main_axis_extent = 0.0;
                run_cross_axis_extent = 0.0;
                child_count = 0;
            }
            run_main_axis_extent += child_main_axis_extent;
            if child_count > 0 {
                run_main_axis_extent += spacing;
            }
            run_cross_axis_extent = f64::max(run_cross_axis_extent, child_cross_axis_extent);
            child_count += 1;
        }
        if child_count > 0 {
            main_axis_extent = f64::max(main_axis_extent, run_main_axis_extent);
            cross_axis_extent += run_cross_axis_extent;
            if !run_metrics.is_empty() {
                cross_axis_extent += run_spacing;
            }
            run_metrics.push((run_main_axis_extent, run_cross_axis_extent, child_count));
        }

        let run_count = run_metrics.len();
        assert!(run_count > 0);

        let size = bc.constrain(dir.pack(main_axis_extent, cross_axis_extent));
        let container_main_axis_extent = dir.major(size);
        let container_cross_axis_extent = dir.minor(size);

        let cross_axis_free_space = (container_cross_axis_extent - cross_axis_extent).max(0.);
        let (run_leading_space, mut run_between_spacing) = match self.run_alignment {
            WrapAlignment::Start => (0., 0.),
            WrapAlignment::End => (cross_axis_free_space, 0.),
            WrapAlignment::Center => (cross_axis_free_space / 2., 0.),
            WrapAlignment::SpaceBetween if run_count > 1 => {
                (0., cross_axis_free_space / (run_count as f64 - 1.))
            }
            WrapAlignment::SpaceBetween => (0., 0.),
            WrapAlignment::SpaceAround => (
                cross_axis_free_space / run_count as f64 / 2.,
                cross_axis_free_space / run_count as f64,
            ),
            WrapAlignment::SpaceEvenly => (
                cross_axis_free_space / (run_count as f64 + 1.),
                cross_axis_free_space / (run_count as f64 + 1.),
            ),
        };

        run_between_spacing += run_spacing;
        let mut cross_axis_offset = run_leading_space;

        let mut childs = self.children.iter_mut();
        for (run_main_axis_extent, run_cross_axis_extent, child_count) in run_metrics {
            let main_axis_free_space =
                f64::max(0.0, container_main_axis_extent - run_main_axis_extent);

            let (child_leading_space, mut child_between_space) = match self.alignment {
                WrapAlignment::Start => (0., 0.),
                WrapAlignment::End => (main_axis_free_space, 0.),
                WrapAlignment::Center => (main_axis_free_space / 2., 0.),
                WrapAlignment::SpaceBetween if run_count > 1 => {
                    (0., main_axis_free_space / (run_count as f64 - 1.))
                }
                WrapAlignment::SpaceBetween => (0., 0.),
                WrapAlignment::SpaceAround => (
                    main_axis_free_space / run_count as f64 / 2.,
                    main_axis_free_space / run_count as f64,
                ),
                WrapAlignment::SpaceEvenly => (
                    main_axis_free_space / (run_count as f64 + 1.),
                    main_axis_free_space / (run_count as f64 + 1.),
                ),
            };
            child_between_space += spacing;
            let mut child_main_position = child_leading_space;

            for child in (&mut childs).take(child_count) {
                let child_size = child.layout_rect().size();
                let free_space = run_cross_axis_extent - dir.minor(child_size);

                let child_cross_axis_offset = match self.cross_alignment {
                    WrapCrossAlignment::Start => cross_axis_offset,
                    WrapCrossAlignment::End => cross_axis_offset + free_space,
                    WrapCrossAlignment::Center => cross_axis_offset + free_space / 2.,
                };

                child.set_origin(
                    ctx,
                    data,
                    env,
                    dir.pack(child_main_position, child_cross_axis_offset)
                        .into(),
                );
                child_main_position += dir.major(child_size) + child_between_space;
            }

            cross_axis_offset += run_cross_axis_extent + run_between_spacing;
        }
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        for x in &mut self.children {
            x.paint(ctx, data, env);
        }
    }
}
