// Copyright 2022 The Druid Authors.
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
//
// Author: Dietmar Maurer <dietmar@proxmox.com>

use druid::kurbo::{BezPath, Circle, Line};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::{Flex, Label};
use druid::{
    AppLauncher, BoxConstraints, Color, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle,
    LifeCycleCtx, PaintCtx, Size, UpdateCtx, Widget, WidgetExt, WindowDesc, WindowSizePolicy,
};

use druid_widget_nursery::animation::{AnimationController, AnimationCurve, AnimationDirection};

#[derive(Clone, Default, Data)]
struct AppState;

fn build_labeled_graph(label_text: &str, curve: AnimationCurve) -> impl Widget<AppState> {
    Flex::column()
        .with_child(Label::new(label_text))
        .with_child(AnimationCurveGraph::new(curve))
        .padding((10.0, 10.0, 10.0, 20.0))
}

fn ui_builder() -> impl Widget<AppState> {
    Flex::column()
        .with_child(
            Flex::row()
                .with_child(build_labeled_graph("EASE_IN", AnimationCurve::EASE_IN))
                .with_child(build_labeled_graph("EASE_OUT", AnimationCurve::EASE_OUT))
                .with_child(build_labeled_graph(
                    "EASE_IN_OUT",
                    AnimationCurve::EASE_IN_OUT,
                )),
        )
        .with_child(
            Flex::row()
                .with_child(build_labeled_graph(
                    "EASE_IN_EXPO",
                    AnimationCurve::EASE_IN_EXPO,
                ))
                .with_child(build_labeled_graph(
                    "EASE_OUT_EXPO",
                    AnimationCurve::EASE_OUT_EXPO,
                ))
                .with_child(build_labeled_graph(
                    "EASE_IN_OUT_EXPO",
                    AnimationCurve::EASE_IN_OUT_EXPO,
                )),
        )
        .with_child(
            Flex::row()
                .with_child(build_labeled_graph("BOUNCE_IN", AnimationCurve::BOUNCE_IN))
                .with_child(build_labeled_graph(
                    "BOUNCE_OUT",
                    AnimationCurve::BOUNCE_OUT,
                ))
                .with_child(build_labeled_graph(
                    "BOUNCE_IN_OUT",
                    AnimationCurve::BOUNCE_IN_OUT,
                )),
        )
        .with_child(
            Flex::row()
                .with_child(build_labeled_graph(
                    "EASE_IN_SINE",
                    AnimationCurve::EASE_IN_SINE,
                ))
                .with_child(build_labeled_graph(
                    "EASE_OUT_SINE",
                    AnimationCurve::EASE_OUT_SINE,
                ))
                .with_child(build_labeled_graph(
                    "EASE_IN_OUT_SINE",
                    AnimationCurve::EASE_IN_OUT_SINE,
                )),
        )
        .with_child(
            Flex::row()
                .with_child(build_labeled_graph(
                    "EASE_IN_ELASTIC",
                    AnimationCurve::EASE_IN_ELASTIC,
                ))
                .with_child(build_labeled_graph(
                    "EASE_OUT_ELASTIC",
                    AnimationCurve::EASE_OUT_ELASTIC,
                ))
                .with_child(build_labeled_graph(
                    "EASE_IN_OUT_ELASTIC",
                    AnimationCurve::EASE_IN_OUT_ELASTIC,
                )),
        )
        .with_child(
            Flex::row()
                .with_child(build_labeled_graph(
                    "EASE_IN_BACK",
                    AnimationCurve::EASE_IN_BACK,
                ))
                .with_child(build_labeled_graph(
                    "EASE_OUT_BACK",
                    AnimationCurve::EASE_OUT_BACK,
                ))
                .with_child(build_labeled_graph(
                    "EASE_IN_OUT_BACK",
                    AnimationCurve::EASE_IN_OUT_BACK,
                )),
        )
}

pub fn main() {
    let main_window = WindowDesc::new(ui_builder())
        .window_size_policy(WindowSizePolicy::Content)
        .title("Show Animation Curves");

    let state = AppState;

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(state)
        .expect("launch failed");
}

/// A widget to display AnimationCurves
pub struct AnimationCurveGraph {
    curve: AnimationCurve,
    animation: AnimationController,

    // paint cache
    curve_path: BezPath,
}

const AXIS_INSET: f64 = 10.0;
const CURVE_WIDTH: f64 = 300.0;
const CURVE_HEIGHT: f64 = 120.0;

impl AnimationCurveGraph {
    pub fn new(curve: AnimationCurve) -> Self {
        let curve_path = Self::curve_to_path(&curve);
        Self {
            curve,
            curve_path,
            animation: AnimationController::new()
                .duration(1.0)
                .direction(AnimationDirection::Alternate)
                .repeat_limit(Some(4)),
        }
    }

    fn curve_to_path(curve: &AnimationCurve) -> BezPath {
        let dx = CURVE_WIDTH / 100.0;
        let mut path = BezPath::new();

        let x0 = AXIS_INSET;
        let y0 = AXIS_INSET + CURVE_HEIGHT;
        for i in 0..100 {
            if i == 0 {
                path.move_to((x0, y0));
            }

            let x1 = x0 + dx * ((i + 1) as f64);
            let y1 = y0 - curve.translate(((i + 1) as f64) / 100.0) * CURVE_HEIGHT;
            path.line_to((x1, y1));
        }

        path
    }
}

impl<T: Data> Widget<T> for AnimationCurveGraph {
    fn event(&mut self, ctx: &mut EventCtx<'_, '_>, event: &Event, _data: &mut T, _env: &Env) {
        match event {
            Event::MouseDown(_) => {
                self.animation.start(ctx);
            }
            Event::AnimFrame(nanos) => {
                self.animation.update(ctx, *nanos);
            }
            _ => {}
        }
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx<'_, '_>,
        _event: &LifeCycle,
        _data: &T,
        _env: &Env,
    ) {
    }

    fn update(&mut self, _ctx: &mut UpdateCtx<'_, '_>, _old_data: &T, _data: &T, _env: &Env) {}

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_, '_>,
        _bc: &BoxConstraints,
        _data: &T,
        _env: &Env,
    ) -> druid::Size {
        Size::new(
            CURVE_WIDTH + 2.0 * AXIS_INSET,
            CURVE_HEIGHT + 2.0 * AXIS_INSET,
        )
    }

    fn paint(&mut self, ctx: &mut PaintCtx<'_, '_, '_>, _data: &T, _env: &Env) {
        let axis_color = Color::grey8(140);

        let style = StrokeStyle::new()
            .line_cap(LineCap::Round)
            .line_join(LineJoin::Round);

        let line = Line::new(
            (AXIS_INSET, AXIS_INSET / 2.0),
            (AXIS_INSET, CURVE_HEIGHT + AXIS_INSET * 1.5),
        );
        ctx.stroke_styled(line, &axis_color, 3.0, &style);
        let line = Line::new(
            (AXIS_INSET / 2.0, CURVE_HEIGHT + AXIS_INSET),
            (AXIS_INSET + CURVE_WIDTH, CURVE_HEIGHT + AXIS_INSET),
        );
        ctx.stroke_styled(line, &axis_color, 3.0, &style);

        let dashed_style = StrokeStyle::new()
            .dash_pattern(&[5.0, 5.0])
            .line_cap(LineCap::Round)
            .line_join(LineJoin::Round);

        let line = Line::new(
            (AXIS_INSET / 2.0, AXIS_INSET),
            (AXIS_INSET + CURVE_WIDTH, AXIS_INSET),
        );
        ctx.stroke_styled(line, &axis_color, 3.0, &dashed_style);

        ctx.stroke_styled(&self.curve_path, &Color::BLUE, 3.0, &style);

        let fraction = self.animation.fraction();
        if fraction > 0.0 && fraction < 1.0 {
            let anim_y = AXIS_INSET + CURVE_HEIGHT - self.curve.translate(fraction) * CURVE_HEIGHT;
            let line = Line::new(
                (AXIS_INSET / 2.0, anim_y),
                (AXIS_INSET + CURVE_WIDTH, anim_y),
            );
            ctx.stroke_styled(line, &axis_color, 3.0, &style);

            let circle = Circle::new((AXIS_INSET + CURVE_WIDTH * fraction, anim_y), 6.0);
            ctx.fill(circle, &axis_color);
        }
    }
}
