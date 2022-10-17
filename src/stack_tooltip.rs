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

//! A stack based tooltip widget.

use crate::{Stack, StackChildParams, StackChildPosition};
use druid::{
    widget::{
        DefaultScopePolicy, Either, Label, LabelText, LensScopeTransfer, LensWrap, Scope, SizedBox,
    },
    Data, Lens, Point, RenderContext, Selector, SingleUse, Size, Widget, WidgetExt, WidgetId,
    WidgetPod,
};

const FORWARD: Selector<SingleUse<(WidgetId, Point)>> = Selector::new("tooltip.forward");
const POINT_UPDATED: Selector = Selector::new("tooltip.label.point_updated");
pub(crate) const ADVISE_TOOLTIP_SHOW: Selector<Point> =
    Selector::new("tooltip.advise_show_tooltip");
pub(crate) const CANCEL_TOOLTIP_SHOW: Selector = Selector::new("tooltip.cancel_show_tooltip");

type StackTooltipActual<T> = Scope<
    DefaultScopePolicy<
        fn(T) -> TooltipState<T>,
        LensScopeTransfer<tooltip_state_derived_lenses::data<T>, T, TooltipState<T>>,
    >,
    StackTooltipInternal<T>,
>;

pub struct StackTooltip<T: Data>(StackTooltipActual<T>);

impl<T: Data> StackTooltip<T> {
    pub fn new<W: Widget<T> + 'static>(widget: W, label: impl Into<LabelText<T>>) -> Self {
        Self(StackTooltipInternal::new(widget, label))
    }
}

impl<T: Data> Widget<T> for StackTooltip<T> {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut T,
        env: &druid::Env,
    ) {
        self.0.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &T,
        env: &druid::Env,
    ) {
        self.0.lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &T, data: &T, env: &druid::Env) {
        self.0.update(ctx, old_data, data, env)
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &T,
        env: &druid::Env,
    ) -> Size {
        self.0.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &T, env: &druid::Env) {
        self.0.paint(ctx, data, env)
    }
}

#[derive(Clone, Data, Lens)]
struct TooltipState<T> {
    data: T,
    show: bool,
    position: StackChildPosition,
    label_size: Option<Size>,
}

struct StackTooltipInternal<T> {
    widget: WidgetPod<TooltipState<T>, Stack<TooltipState<T>>>,
    label_id: Option<WidgetId>,
}

fn make_state<T: Data>(data: T) -> TooltipState<T> {
    TooltipState {
        data,
        show: false,
        position: StackChildPosition::new().height(Some(0.0)),
        label_size: None,
    }
}

impl<T: Data> StackTooltipInternal<T> {
    fn new<W: Widget<T> + 'static>(
        widget: W,
        label: impl Into<LabelText<T>>,
    ) -> StackTooltipActual<T> {
        let label_id = WidgetId::next();
        let stack = Stack::new()
            .with_child(widget.lens(TooltipState::data))
            .with_positioned_child(
                Either::new(
                    |state: &TooltipState<T>, _| state.show && is_some_position(&state.position),
                    TooltipLabel::new(label, label_id),
                    SizedBox::empty(),
                ),
                StackChildParams::dynamic(|TooltipState { position, .. }: &TooltipState<T>, _| {
                    position
                }),
            );

        Scope::from_lens(
            make_state as fn(T) -> TooltipState<T>,
            TooltipState::data,
            Self {
                widget: WidgetPod::new(stack),
                label_id: Some(label_id),
            },
        )
    }
}

impl<T: Data> Widget<TooltipState<T>> for StackTooltipInternal<T> {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut TooltipState<T>,
        env: &druid::Env,
    ) {
        if let Some(pos) = if let druid::Event::MouseMove(mouse) = event {
            Some(mouse.pos)
        } else if let druid::Event::Command(cmd) = event {
            cmd.get(FORWARD)
                .and_then(SingleUse::take)
                .and_then(|(id, point)| {
                    self.label_id
                        .filter(|label_id| label_id == &id)
                        .and(Some(point))
                })
                .map(|point| (point - ctx.window_origin()).to_point())
        } else {
            None
        } {
            if ctx.is_hot() && ctx.size().to_rect().contains(pos) {
                let mut x = pos.x;
                let mut y = pos.y;

                if let Some(size) = data.label_size {
                    if x + size.width + ctx.window_origin().x
                        > ctx.window().get_size().width - ctx.window().content_insets().x_value()
                    {
                        x -= size.width
                    };
                    if y + size.height + ctx.window_origin().y
                        > ctx.window().get_size().height - ctx.window().content_insets().y_value()
                    {
                        y -= size.height
                    };
                }

                data.position = StackChildPosition::new()
                    .left(Some(x))
                    .top(Some(y))
                    .height(None);

                data.show = true;

                if let Some(label_id) = self.label_id {
                    if data.label_size.is_none() {
                        ctx.submit_command(POINT_UPDATED.to(label_id));
                    }
                    ctx.submit_command(ADVISE_TOOLTIP_SHOW.with(ctx.to_window(pos)));
                }
            } else {
                reset_position(&mut data.position);
                data.position.height = Some(0.0);
                data.show = false;
            }

            if let druid::Event::Command(_) = event {
                return;
            }
        } else if let druid::Event::Notification(notif) = event {
            if notif.is(CANCEL_TOOLTIP_SHOW) && notif.route() == self.widget.id() {
                reset_position(&mut data.position);
                data.position.height = Some(0.0);
                data.show = false;

                ctx.set_handled();
            }
        };

        self.widget.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &TooltipState<T>,
        env: &druid::Env,
    ) {
        self.widget.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        _old_data: &TooltipState<T>,
        data: &TooltipState<T>,
        env: &druid::Env,
    ) {
        self.widget.update(ctx, data, env)
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &TooltipState<T>,
        env: &druid::Env,
    ) -> druid::Size {
        self.widget.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &TooltipState<T>, env: &druid::Env) {
        self.widget.paint(ctx, data, env)
    }
}

struct TooltipLabel<T> {
    id: WidgetId,
    label: WidgetPod<
        TooltipState<T>,
        LensWrap<TooltipState<T>, T, tooltip_state_derived_lenses::data<T>, Label<T>>,
    >,
}

impl<T: Data> TooltipLabel<T> {
    pub fn new(label: impl Into<LabelText<T>>, id: WidgetId) -> Self {
        let label = WidgetPod::new(Label::new(label.into()).lens(TooltipState::data));

        Self { id, label }
    }
}

impl<T: Data> Widget<TooltipState<T>> for TooltipLabel<T> {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut TooltipState<T>,
        env: &druid::Env,
    ) {
        if let druid::Event::MouseMove(mouse) = event {
            ctx.submit_command(FORWARD.with(SingleUse::new((ctx.widget_id(), mouse.window_pos))))
        } else if let druid::Event::Command(cmd) = event {
            if cmd.is(POINT_UPDATED) {
                if let Some(left) = data.position.left {
                    let label_width = ctx.size().width;
                    if left + label_width + ctx.window_origin().x > ctx.window().get_size().width {
                        data.position.left.replace(left - label_width);
                    }
                }
                if let Some(top) = data.position.top {
                    let label_height = ctx.size().height;
                    if top + label_height + ctx.window_origin().y > ctx.window().get_size().height {
                        data.position.top.replace(top - label_height);
                    }
                }

                if !ctx.size().is_empty() {
                    data.label_size.replace(ctx.size());
                }

                ctx.request_paint();
            }
        }

        self.label.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &TooltipState<T>,
        env: &druid::Env,
    ) {
        self.label.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        _old_data: &TooltipState<T>,
        data: &TooltipState<T>,
        env: &druid::Env,
    ) {
        self.label.update(ctx, data, env)
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &TooltipState<T>,
        env: &druid::Env,
    ) -> druid::Size {
        self.label.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &TooltipState<T>, env: &druid::Env) {
        let mut rect = ctx.size().to_rect();
        rect.x0 -= 2.0;
        rect.y1 += 2.0;

        let fill_brush = ctx.solid_brush(env.get(druid::theme::BACKGROUND_DARK));
        ctx.fill(rect, &fill_brush);

        self.label.paint(ctx, data, env);

        let border_brush = ctx.solid_brush(env.get(druid::theme::BORDER_DARK));
        ctx.stroke(rect, &border_brush, 1.0);
    }

    fn id(&self) -> Option<WidgetId> {
        Some(self.id)
    }
}

fn is_some_position(position: &StackChildPosition) -> bool {
    position.top.is_some()
        || position.bottom.is_some()
        || position.left.is_some()
        || position.right.is_some()
}

fn reset_position(position: &mut StackChildPosition) {
    position.top = None;
    position.bottom = None;
    position.left = None;
    position.right = None;
    position.width = None;
    position.height = None;
}
