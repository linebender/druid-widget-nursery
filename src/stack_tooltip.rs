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

use std::{sync::Arc, rc::Rc, cell::RefCell, convert::{TryFrom, TryInto}};

use crate::{Stack, StackChildParams, StackChildPosition};
use druid::{
    widget::{
        DefaultScopePolicy, Either, Label, LensScopeTransfer, Scope, SizedBox, RawLabel, WidgetWrapper,
    },
    Data, Lens, Point, RenderContext, Selector, SingleUse, Size, Widget, WidgetExt, WidgetId,
    WidgetPod, piet::{Text, TextLayoutBuilder, TextStorage, TextAttribute}, text::{RichText, Attribute}, KeyOrValue, Color,
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
    pub fn new<W: Widget<T> + 'static>(widget: W, label: impl Into<PlainOrRich>) -> Self {
        Self(StackTooltipInternal::new(widget, label))
    }

    pub fn set_text_attribute(&mut self, attribute: Attribute) {
        self.0.wrapped_mut().set_text_attribute(attribute);
    }

    pub fn with_text_attribute(mut self, attribute: Attribute) -> Self {
        self.set_text_attribute(attribute);

        self
    }

    pub fn set_background_color(&mut self, color: impl Into<KeyOrValue<Color>>) {
        self.0.wrapped_mut().set_background_color(color)
    }

    pub fn with_background_color(mut self, color: impl Into<KeyOrValue<Color>>) -> Self {
        self.set_background_color(color);

        self
    }

    pub fn set_border_width(&mut self, width: f64) {
        self.0.wrapped_mut().set_border_width(width)
    }

    pub fn with_border_width(mut self, width: f64) -> Self {
        self.set_border_width(width);

        self
    }

    pub fn set_border_color(&mut self, color: impl Into<KeyOrValue<Color>>) {
        self.0.wrapped_mut().set_border_color(color);
    }

    pub fn with_border_color(mut self, color: impl Into<KeyOrValue<Color>>) -> Self {
        self.set_border_color(color);

        self
    }

    pub fn set_crosshair(&mut self, crosshair: bool) {
        self.0.wrapped_mut().set_crosshair(crosshair)
    }

    pub fn with_crosshair(mut self, crosshair: bool) -> Self {
        self.set_crosshair(crosshair);

        self
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

type RichTextCell = Rc<RefCell<(RichText, Vec<YetAnotherAttribute>)>>;
type BackgroundCell = Rc<RefCell<Option<KeyOrValue<Color>>>>;
type BorderCell = Rc<RefCell<(Option<KeyOrValue<Color>>, Option<f64>)>>;

struct StackTooltipInternal<T> {
    widget: WidgetPod<TooltipState<T>, Stack<TooltipState<T>>>,
    label_id: Option<WidgetId>,
    text: RichTextCell,
    background: BackgroundCell,
    border: BorderCell,
    use_crosshair: bool,
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
        label: impl Into<PlainOrRich>,
    ) -> StackTooltipActual<T> {
        let rich_text = match label.into() {
            PlainOrRich::Plain(plain) => RichText::new(plain.into()),
            PlainOrRich::Rich(rich) => rich
        };
        let attrs = vec![];

        let text = Rc::new(RefCell::new((rich_text, attrs)));
        let background = BackgroundCell::default();
        let border = BorderCell::default();
        let label_id = WidgetId::next();
        let stack = Stack::new()
            .with_child(widget.lens(TooltipState::data))
            .with_positioned_child(
                Either::new(
                    |state: &TooltipState<T>, _| state.show && is_some_position(&state.position),
                    TooltipLabel::new(text.clone(), label_id, background.clone(), border.clone()),
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
                text,
                background,
                border,
                use_crosshair: false
            },
        )
    }

    pub fn set_text_attribute(&mut self, attribute: Attribute) {
        self.text.borrow_mut().0.add_attribute(0.., attribute.clone());
        match attribute.try_into() {
            Ok(attr) => self.text.borrow_mut().1.push(attr),
            Err(attrs) => {
                self.text.borrow_mut().1.extend(attrs)
            },
        };
    }

    pub fn set_background_color(&mut self, color: impl Into<KeyOrValue<Color>>) {
        self.background.borrow_mut().replace(color.into());
    }

    pub fn set_border_width(&mut self, width: f64) {
        self.border.borrow_mut().1.replace(width);
    }

    pub fn set_border_color(&mut self, color: impl Into<KeyOrValue<Color>>) {
        self.border.borrow_mut().0.replace(color.into());
    }

    pub fn set_crosshair(&mut self, crosshair: bool) {
        self.use_crosshair = crosshair
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

                if self.use_crosshair {
                    ctx.set_cursor(&druid::Cursor::Crosshair);
                }

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

struct TooltipLabel {
    id: WidgetId,
    label: WidgetPod<RichText, RawLabel<RichText>>,
    text: RichTextCell,
    background: BackgroundCell,
    border: BorderCell,
}

impl TooltipLabel {
    pub fn new(text: RichTextCell, id: WidgetId, background: BackgroundCell, border: BorderCell) -> Self {
        let label = WidgetPod::new(Label::raw());

        Self {
            id,
            label,
            text,
            background,
            border,
        }
    }
}

impl<T: Data> Widget<TooltipState<T>> for TooltipLabel {
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

        self.label.event(ctx, event, &mut self.text.borrow_mut().0, env)
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        _data: &TooltipState<T>,
        env: &druid::Env,
    ) {
        self.label.lifecycle(ctx, event, &self.text.borrow().0, env)
    }

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        _old_data: &TooltipState<T>,
        _data: &TooltipState<T>,
        env: &druid::Env,
    ) {
        self.label.update(ctx, &self.text.borrow().0, env)
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        _data: &TooltipState<T>,
        env: &druid::Env,
    ) -> druid::Size {
        self.label.layout(ctx, bc, &self.text.borrow().0, env)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, _data: &TooltipState<T>, env: &druid::Env) {
        let mut rect = ctx.size().to_rect();
        rect.x0 -= 2.0;
        rect.y1 += 2.0;

        let fill_brush = ctx.solid_brush(if let Some(background) = self.background.borrow().as_ref() {
            background.resolve(env)
        } else {
            env.get(druid::theme::BACKGROUND_DARK)
        });
        let border_brush = ctx.solid_brush(if let Some(border) = self.border.borrow().0.as_ref() {
            border.resolve(env)
        } else {
            env.get(druid::theme::BORDER_DARK)
        });
        let border_width = if let Some(width) = self.border.borrow().1.as_ref() {
            *width
        } else {
            env.get(druid::theme::TEXTBOX_BORDER_WIDTH)
        };

        let mut text = ctx.text().new_text_layout(<&str as Into<Arc<str>>>::into(self.text.borrow().0.as_str()));
        text = text.default_attribute(TextAttribute::FontFamily(env.get(druid::theme::UI_FONT).family));
        text = text.default_attribute(TextAttribute::FontSize(env.get(druid::theme::UI_FONT).size));
        text = text.default_attribute(TextAttribute::Style(env.get(druid::theme::UI_FONT).style));
        text = text.default_attribute(TextAttribute::Weight(env.get(druid::theme::UI_FONT).weight));
        text = text.default_attribute(TextAttribute::TextColor(env.get(druid::theme::TEXT_COLOR)));
        for attribute in self.text.borrow().1.iter() {
            text = text.default_attribute(attribute.clone().resolve(env));
        }
        if let Ok(text) = text.build() {
            ctx.paint_with_z_index(1_000_000, move |ctx| {
                ctx.fill(rect, &fill_brush);

                ctx.draw_text(&text, (0.0, 0.0));
                
                ctx.stroke(rect, &border_brush, border_width);
            });
        };

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

pub enum PlainOrRich {
    Plain(String),
    Rich(RichText)
}

impl From<String> for PlainOrRich {
    fn from(plain: String) -> Self {
        PlainOrRich::Plain(plain)
    }
}

impl From<&str> for PlainOrRich {
    fn from(plain: &str) -> Self {
        PlainOrRich::Plain(plain.to_owned())
    }
}

impl From<Arc<str>> for PlainOrRich {
    fn from(plain: Arc<str>) -> Self {
        PlainOrRich::Plain(plain.to_string())
    }
}

impl From<RichText> for PlainOrRich {
    fn from(rich: RichText) -> Self {
        PlainOrRich::Rich(rich)
    }
}

#[derive(Clone)]
enum YetAnotherAttribute {
    Unresolved(Attribute),
    UnresolvedFamily(Attribute),
    UnresolvedSize(Attribute),
    UnresolvedWeight(Attribute),
    UnresolvedStyle(Attribute),
    Resolved(TextAttribute),
}

impl YetAnotherAttribute {
    fn resolve(self, env: &druid::Env) -> TextAttribute {
        match self {
            YetAnotherAttribute::Unresolved(unresolved) => match unresolved {
                Attribute::FontSize(size) => TextAttribute::FontSize(size.resolve(env)),
                Attribute::TextColor(color) => TextAttribute::TextColor(color.resolve(env)),
                _ => unreachable!()
            },
            YetAnotherAttribute::UnresolvedFamily(desc) => if let Attribute::Descriptor(desc) = desc {
                TextAttribute::FontFamily(desc.resolve(env).family)
            } else {
                unreachable!()
            },
            YetAnotherAttribute::UnresolvedSize(desc) => if let Attribute::Descriptor(desc) = desc {
                TextAttribute::FontSize(desc.resolve(env).size)
            } else {
                unreachable!()
            },
            YetAnotherAttribute::UnresolvedWeight(desc) => if let Attribute::Descriptor(desc) = desc {
                TextAttribute::Weight(desc.resolve(env).weight)
            } else {
                unreachable!()
            },
            YetAnotherAttribute::UnresolvedStyle(desc) => if let Attribute::Descriptor(desc) = desc {
                TextAttribute::Style(desc.resolve(env).style)
            } else {
                unreachable!()
            },
            YetAnotherAttribute::Resolved(attr) => attr,
        }
    }
}

impl TryFrom<Attribute> for YetAnotherAttribute {
    type Error = [YetAnotherAttribute; 4];

    fn try_from(value: Attribute) -> Result<Self, Self::Error> {
        let res = match value {
            Attribute::FontFamily(family) => Self::Resolved(TextAttribute::FontFamily(family)),
            Attribute::Weight(attr) => Self::Resolved(TextAttribute::Weight(attr)),
            Attribute::Style(attr) => Self::Resolved(TextAttribute::Style(attr)),
            Attribute::Underline(attr) => Self::Resolved(TextAttribute::Underline(attr)),
            Attribute::Strikethrough(attr) => Self::Resolved(TextAttribute::Strikethrough(attr)),
            unresolved @ Attribute::FontSize(_) | unresolved @ Attribute::TextColor(_) => YetAnotherAttribute::Unresolved(unresolved),
            descriptor @ Attribute::Descriptor(_) => {
                Err([
                    YetAnotherAttribute::UnresolvedFamily(descriptor.clone()),
                    YetAnotherAttribute::UnresolvedSize(descriptor.clone()),
                    YetAnotherAttribute::UnresolvedWeight(descriptor.clone()),
                    YetAnotherAttribute::UnresolvedStyle(descriptor),
                ])?
            },
        };
    
        Ok(res)
    }
}
