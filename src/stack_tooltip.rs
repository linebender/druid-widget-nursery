// Copyright 2020 The Druid Authors.
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

//! A stack based tooltip widget + manager.

use crate::{Stack, StackChildParams, StackChildPosition, WidgetExt as _};
use druid::{
    lens,
    widget::{Label, Scope, SizedBox, ViewSwitcher},
    Data, Point, Selector, SingleUse, Widget, WidgetExt, WidgetId, WidgetPod,
};

const SHOW_AT: Selector<SingleUse<(Point, String)>> = Selector::new("tooltip.show_at");
const HIDE: Selector = Selector::new("tooltip.hide");
const FORWARD: Selector<SingleUse<Point>> = Selector::new("tooltip.forward");
const UPDATE_ID: Selector<SingleUse<WidgetId>> = Selector::new("tooltip.update.id");
const POINT_UPDATED: Selector = Selector::new("tooltip.label.get_dims");

type StatefulT<T> = (T, Option<String>, StackChildPosition);

pub struct TooltipFactory {
    id: Option<WidgetId>,
}

impl Default for TooltipFactory {
    fn default() -> Self {
        Self { id: None }
    }
}

impl TooltipFactory {
    pub fn wrapper<T: Data, W: Widget<T> + 'static>(
        &mut self,
        widget: W,
    ) -> Option<impl Widget<T>> {
        if let Some(id) = self.id {
            Some(Scope::from_lens(
                |input| (input, None, StackChildPosition::new().height(Some(0.0))),
                lens!(StatefulT<T>, 0),
                TooltipWrapper::new(widget, id).with_id(id),
            ))
        } else {
            None
        }
    }

    pub fn wrapper_id(&self) -> Option<WidgetId> {
        self.id.clone()
    }

    pub fn tooltip<T: Data, W: Widget<T> + 'static>(
        &mut self,
        trigger: W,
        label: impl AsRef<str>,
    ) -> impl Widget<T> {
        let id = if let Some(id) = self.id {
            id.clone()
        } else {
            let id = WidgetId::next();
            self.id = Some(id.clone());
            id
        };
        let label = label.as_ref().to_owned();

        TooltipTrigger::new(trigger, label, id)
    }
}

pub struct TooltipTrigger<W, T> {
    widget: WidgetPod<W, T>,
    label: String,
    id: WidgetId,
}

impl<T: Data, W: Widget<T>> TooltipTrigger<T, W> {
    fn new(widget: W, label: String, id: WidgetId) -> Self {
        Self {
            widget: WidgetPod::new(widget),
            label,
            id,
        }
    }
}

impl<T: Data, W: Widget<T>> Widget<T> for TooltipTrigger<T, W> {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut T,
        env: &druid::Env,
    ) {
        if let druid::Event::MouseMove(mouse) = event {
            if ctx.is_hot() && ctx.size().to_rect().contains(mouse.pos) {
                ctx.submit_command(
                    SHOW_AT
                        .with(SingleUse::new((
                            mouse.window_pos.clone(),
                            self.label.clone(),
                        )))
                        .to(self.id),
                );
            } else {
                ctx.submit_command(HIDE.to(self.id));
            }
            return;
        } else if let druid::Event::Command(cmd) = event {
            if let Some(point) = cmd.get(FORWARD).and_then(SingleUse::take) {
                let rect = ctx.size().to_rect().with_origin(ctx.window_origin());
                if rect.contains(point) {
                    ctx.submit_command(
                        SHOW_AT
                            .with(SingleUse::new((point, self.label.clone())))
                            .to(self.id),
                    );
                } else {
                    ctx.submit_command(HIDE.to(self.id));
                }
            }
        }

        self.widget.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &T,
        env: &druid::Env,
    ) {
        self.widget.lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, _old_data: &T, data: &T, env: &druid::Env) {
        self.widget.update(ctx, data, env)
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &T,
        env: &druid::Env,
    ) -> druid::Size {
        self.widget.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &T, env: &druid::Env) {
        self.widget.paint(ctx, data, env)
    }
}

pub struct TooltipWrapper<T> {
    widget: WidgetPod<T, Stack<T>>,
    label_id: Option<WidgetId>,
}

impl<T: Data> TooltipWrapper<StatefulT<T>> {
    pub fn new<W: Widget<T> + 'static>(widget: W, id: WidgetId) -> impl Widget<StatefulT<T>> {
        let stack = Stack::new()
            .with_child(widget.lens(lens!(StatefulT<T>, 0)))
            .with_positioned_child(
                ViewSwitcher::new(|(_, show, _): &StatefulT<T>, _| show.clone(), {
                    let id = id.clone();
                    move |show: &Option<String>, (_, _, position): &StatefulT<T>, _| {
                        println!("rebuilding");
                        if let Some(label) = show {
                            if is_some_position(position) {
                                return TooltipLabel::new(label.clone(), id.clone())
                                    .background(druid::theme::BACKGROUND_DARK)
                                    .border(druid::theme::BORDER_DARK, 2.0)
                                    .on_added(move |_, ctx, _, _| {
                                        ctx.submit_command(
                                            UPDATE_ID
                                                .with(SingleUse::new(ctx.widget_id()))
                                                .to(id.clone()),
                                        )
                                    })
                                    .on_command(POINT_UPDATED, |ctx, _, (_, _, position)| {
                                        if let Some(left) = position.left {
                                            let window_width = ctx.window().get_size().width;
                                            if left + ctx.size().width
                                                > window_width
                                            {
                                                position.left = None;
                                                position.right.replace(window_width - left);
                                            }
                                        }
                                        if let Some(top) = position.top {
                                            let window_height = ctx.window().get_size().height;
                                            if top + ctx.size().height
                                                > window_height
                                            {
                                                position.top = None;
                                                position.bottom.replace(window_height - top);
                                            }
                                        }
                                    })
                                    .boxed();
                            }
                        }

                        SizedBox::empty().boxed()
                    }
                }),
                StackChildParams::dynamic(|(_, _, position): &StatefulT<T>, _| dbg!(position)),
            );
        TooltipWrapper {
            widget: WidgetPod::new(stack),
            label_id: None,
        }
    }
}

impl<T: Data> Widget<StatefulT<T>> for TooltipWrapper<StatefulT<T>> {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut StatefulT<T>,
        env: &druid::Env,
    ) {
        let (_, label, position) = data;
        if let druid::Event::Command(cmd) = event {
            if cmd.target() == druid::Target::Widget(ctx.widget_id()) {
                if let Some((coord, new_label)) = cmd.get(SHOW_AT).and_then(SingleUse::take) {
                    let diff = coord - ctx.window_origin();
                    println!("setting coords");
                    *position = StackChildPosition::new()
                        .left(Some(diff.x))
                        .top(Some(diff.y))
                        .height(None);

                    label.replace(new_label);

                    if let Some(label_id) = self.label_id {
                        ctx.submit_command(POINT_UPDATED.to(label_id))
                    }
                } else if cmd.is(HIDE) {
                    label.take();
                } else if let Some(label_id) = cmd.get(UPDATE_ID).and_then(SingleUse::take) {
                    self.label_id = Some(label_id)
                }
            }
        }
        self.widget.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &StatefulT<T>,
        env: &druid::Env,
    ) {
        self.widget.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        _old_data: &StatefulT<T>,
        data: &StatefulT<T>,
        env: &druid::Env,
    ) {
        self.widget.update(ctx, data, env)
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &StatefulT<T>,
        env: &druid::Env,
    ) -> druid::Size {
        self.widget.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &StatefulT<T>, env: &druid::Env) {
        self.widget.paint(ctx, data, env)
    }
}

struct TooltipLabel<T> {
    label: WidgetPod<StatefulT<T>, Label<StatefulT<T>>>,
    id: WidgetId,
}

impl<T: Data> TooltipLabel<T> {
    pub fn new(label: impl AsRef<str>, id: WidgetId) -> Self {
        Self {
            label: WidgetPod::new(Label::new(label.as_ref().to_owned())),
            id,
        }
    }
}

impl<T: Data> Widget<StatefulT<T>> for TooltipLabel<T> {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut StatefulT<T>,
        env: &druid::Env,
    ) {
        if let druid::Event::MouseMove(mouse) = event {
            ctx.submit_command(
                FORWARD
                    .with(SingleUse::new(mouse.window_pos.clone()))
                    .to(self.id),
            )
        }

        self.label.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &StatefulT<T>,
        env: &druid::Env,
    ) {
        self.label.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        _old_data: &StatefulT<T>,
        data: &StatefulT<T>,
        env: &druid::Env,
    ) {
        self.label.update(ctx, data, env)
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &StatefulT<T>,
        env: &druid::Env,
    ) -> druid::Size {
        self.label.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &StatefulT<T>, env: &druid::Env) {
        self.label.paint(ctx, data, env)
    }
}

fn is_some_position(position: &StackChildPosition) -> bool {
    position.top.is_some()
        || position.bottom.is_some()
        || position.left.is_some()
        || position.right.is_some()
}
